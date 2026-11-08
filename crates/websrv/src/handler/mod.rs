use axum::extract::State;
use axum::http::StatusCode;
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

impl From<anyhow::Error> for HandlerError {
    fn from(err: anyhow::Error) -> Self {
        tracing::error!(?err, "Unexpected error returned from handler");
        Self(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

impl From<sqlx::Error> for HandlerError {
    fn from(err: sqlx::Error) -> Self {
        if matches!(err, sqlx::Error::RowNotFound) {
            tracing::warn!(%err, "Expected a specific database row but found none");
            return Self(StatusCode::NOT_FOUND);
        }

        tracing::error!(%err, "Database error returned from handler");
        Self(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

#[derive(Clone)]
pub struct HandlerState {
    db_pool: PgPool,
}

impl HandlerState {
    /// NOTE: Can panic. Ensure this only runs at startup.
    pub async fn new() -> Self {
        let db_uri_from_env = std::env::var("DATABASE_URI")
            .ok()
            .filter(|value| value.len() > 0)
            .expect("Missing/unset DATABASE_URI variable");
        let db_pool = PgPoolOptions::new()
            .max_connections(15)
            .connect(&db_uri_from_env)
            .await
            .expect("Connecting to/pinging new database pool");

        Self { db_pool }
    }
}

pub async fn health_check(State(state): State<HandlerState>) -> HandlerResult<impl IntoResponse> {
    let statement = r#"
        SELECT 1;
    "#;

    sqlx::query(statement)
        .execute(&state.db_pool)
        .await
        .map(|_| StatusCode::OK)
        .map_err(HandlerError::from)
}

#[cfg(test)]
mod tests {
    use reqwest::Client;
    use reqwest::StatusCode;

    const HEALTH_CHECK_URL: &str = "http://localhost:8080";

    #[tokio::test]
    async fn health_check_should_200() {
        let health_check_resp = Client::new().get(HEALTH_CHECK_URL).send().await.unwrap();
        assert_eq!(health_check_resp.status(), StatusCode::OK)
    }
}
