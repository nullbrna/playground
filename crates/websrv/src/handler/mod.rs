use axum::extract::Request;
use axum::extract::State;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::response::Response;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

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

impl From<anyhow::Error> for HandlerError {
    fn from(value: anyhow::Error) -> Self {
        tracing::error!(err = %value, "[UNEXPECTED]");
        Self(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

impl From<sqlx::Error> for HandlerError {
    fn from(value: sqlx::Error) -> Self {
        if matches!(value, sqlx::Error::RowNotFound) {
            // Empty responses shouldn't be an error. Queries should optionally
            // fetch results or map the error to a default.
            tracing::warn!("[DATABASE] Unhandled empty response");
            return Self(StatusCode::OK);
        }

        tracing::error!(err = %value, "[DATABASE]");
        Self(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

#[derive(Clone)]
pub struct HandlerState {
    pool: PgPool,
}

impl HandlerState {
    /// NOTE: Can panic. Ensure this ONLY runs during startup.
    pub async fn new() -> Self {
        let db_uri = std::env::var("DATABASE_URI").expect("Missing \"DATABASE_URI\" variable");
        let pool = PgPoolOptions::new()
            .max_connections(25)
            .min_connections(5)
            .connect(&db_uri)
            .await
            .expect("Opening first pool connection");

        Self { pool }
    }
}

pub async fn index(State(state): State<HandlerState>) -> HandlerResult<impl IntoResponse> {
    let user_count = get_user_count(&state.pool).await?;
    let user_count = user_count.to_string();

    Ok(user_count)
}

async fn get_user_count(pool: &PgPool) -> HandlerResult<i64> {
    let statement = r#"
        SELECT COUNT(*) FROM users;
    "#;

    sqlx::query_scalar(statement)
        .fetch_one(pool)
        .await
        .map_err(HandlerError::from)
}

pub async fn middleware(request: Request, next: Next) -> impl IntoResponse {
    next.run(request).await
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use reqwest::Client;
    use reqwest::Response;

    const ENDPOINT: &str = "http://localhost:8080";

    async fn make_request() -> Response {
        Client::new().get(ENDPOINT).send().await.unwrap()
    }

    #[tokio::test]
    async fn should_200_with_user_count() {
        let response = make_request().await;
        assert_eq!(response.status(), StatusCode::OK);

        let body = response.text().await.unwrap();
        assert_eq!(body.len(), 6);
    }
}
