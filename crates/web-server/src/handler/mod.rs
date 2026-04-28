use axum::extract::Request;
use axum::extract::State;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::response::Response;
use sqlx::PgPool;
use sqlx::query_scalar;
use tracing::error;
use tracing::info;

use crate::AppState;

pub struct HandlerError(StatusCode);
type HandlerResult<T> = Result<T, HandlerError>;

impl IntoResponse for HandlerError {
    fn into_response(self) -> Response {
        self.0.into_response()
    }
}

impl From<StatusCode> for HandlerError {
    fn from(value: StatusCode) -> Self {
        Self(value)
    }
}

impl From<sqlx::Error> for HandlerError {
    fn from(value: sqlx::Error) -> Self {
        if matches!(value, sqlx::Error::RowNotFound) {
            return Self(StatusCode::NOT_FOUND);
        }

        error!("[DATABASE] {value}");
        Self(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

impl From<anyhow::Error> for HandlerError {
    fn from(value: anyhow::Error) -> Self {
        error!("[UNEXPECTED] {value}");
        Self(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

pub async fn middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> HandlerResult<impl IntoResponse> {
    info!("[MIDDLEWARE] hit: {state:?}");
    Ok(next.run(request).await)
}

pub async fn index(State(state): State<AppState>) -> HandlerResult<impl IntoResponse> {
    let first_name = find_name_by_id(&state.pool, 1).await?;
    Ok(first_name)
}

async fn find_name_by_id(pool: &PgPool, id: i32) -> HandlerResult<String> {
    let statement = r#"
        SELECT first_name
        FROM users
        WHERE id = $1
    "#;

    let first_name = query_scalar(statement).bind(id).fetch_one(pool).await?;
    Ok(first_name)
}

#[cfg(test)]
mod tests {
    use std::sync::OnceLock;

    use reqwest::Client;
    use reqwest::StatusCode;

    static CLIENT: OnceLock<Client> = OnceLock::new();

    fn http_client() -> &'static Client {
        let setter = || {
            Client::builder()
                .pool_max_idle_per_host(10)
                .build()
                .unwrap()
        };

        CLIENT.get_or_init(setter)
    }

    #[tokio::test]
    async fn index() {
        let response = http_client()
            .get("http://localhost:8080")
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.text().await.unwrap(), "John");
    }
}
