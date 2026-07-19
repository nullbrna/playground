use std::net::SocketAddr;

use axum::extract::ConnectInfo;
use axum::extract::Request;
use axum::extract::State;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::response::Response;
use redis::Client;
use redis::RedisError;
use redis::aio::ConnectionManager;
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
        if value.is_server_error() || value.is_client_error() {
            tracing::warn!(code = ?value, "Negative status code returned");
        }

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

impl From<RedisError> for HandlerError {
    fn from(value: RedisError) -> Self {
        tracing::error!(err = %value, "[REDIS]");
        Self(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

#[derive(Debug, Clone)]
pub struct HandlerState {
    pool:  PgPool,
    cache: ConnectionManager,
}

impl HandlerState {
    /// NOTE: Will panic for a missing environment variable. Ensure this only
    /// runs during startup so any panic is immediate and not at "runtime".
    pub async fn new() -> Self {
        let dburi = std::env::var("DATABASE_URI").expect("Missing \"DATABASE_URI\" variable");
        let cacheuri = std::env::var("REDIS_URI").expect("Missing \"REDIS_URI\" variable");

        let pool = PgPoolOptions::new()
            .max_connections(25)
            .min_connections(5)
            .connect(&dburi)
            .await
            .expect("Opening first pool connection");

        let client = Client::open(cacheuri).expect("Validating cache connection URI");
        let cache = ConnectionManager::new(client)
            .await
            .expect("Initialising cache connection");

        Self { pool, cache }
    }
}

pub async fn index(State(state): State<HandlerState>) -> impl IntoResponse {
    tracing::debug!(?state, "Handler state");
    "Hello, world!"
}

pub async fn middleware(
    ConnectInfo(info): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> impl IntoResponse {
    tracing::trace!(addr = %info.ip(), "Middleware hit");
    next.run(request).await
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use reqwest::Client;
    use reqwest::Response;

    const ENDPOINT: &str = "http://localhost:8080/";

    async fn make_request() -> Response {
        Client::new()
            .get(ENDPOINT)
            .send()
            .await
            .expect("Sending HTTP request")
    }

    #[tokio::test]
    async fn should_200_status_code() {
        let response = make_request().await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn should_return_string_body() {
        let response = make_request().await;
        let body = response.text().await.expect("Decoding response body");

        assert_eq!(body, "Hello, world!");
    }
}
