use axum::extract::Request;
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

pub async fn index() -> HandlerResult<&'static str> {
    Ok("Hello, world!")
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
    async fn should_200_status_code() {
        let response = make_request().await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn should_return_string_body() {
        let response = make_request().await;
        let body = response.text().await.unwrap();

        assert_eq!(body, "Hello, world!");
    }
}
