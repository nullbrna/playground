use axum::extract::Request;
#[cfg(feature = "testing")]
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::response::Response;
use redis::Client;
use redis::aio::ConnectionManager;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

pub mod idempotency;
pub mod ratelimiter;

#[cfg(any(feature = "testing", test))]
const TEST_ID_HEADER_KEY: &str = "X-Test-Identifier";

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

        tracing::error!("[DATABASE] {value}");
        Self(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

impl From<anyhow::Error> for HandlerError {
    fn from(value: anyhow::Error) -> Self {
        tracing::error!("[UNEXPECTED] {value}");
        Self(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

impl From<redis::RedisError> for HandlerError {
    fn from(value: redis::RedisError) -> Self {
        tracing::error!("[REDIS] {value}");
        Self(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

#[derive(Clone)]
pub struct HandlerState {
    /// Connection pool for Postgres.
    /// NOTE: [`PgPool`] under-the-hood is an [`std::sync::Arc`].
    pool: PgPool,
    /// Connection to Redis. Automatically reconnects when needed.
    redis: ConnectionManager,
}

impl HandlerState {
    pub async fn new() -> anyhow::Result<Self> {
        let database_resource = std::env::var("DATABASE_URL")?;
        let redis_resource = std::env::var("REDIS_URL")?;

        // Tests use a dedicated single-connection pool as each test creates
        // their own state. Once the schema is created, the connection is free.
        let connection_count = if cfg!(test) { 1 } else { 10 };
        let pool = PgPoolOptions::new()
            .max_connections(connection_count)
            .connect(&database_resource)
            .await?;

        let client = Client::open(redis_resource)?;
        let redis = ConnectionManager::new(client).await?;

        Ok(Self { pool, redis })
    }

    #[cfg(test)]
    async fn setup_test_state() -> anyhow::Result<String> {
        use sqlx::migrate;
        use sqlx::query;
        use uuid::Uuid;

        let state = Self::new().await?;
        let identifier = Uuid::new_v4().to_string();

        // 1. Create an isolated schema for the current test by the unique ID.
        let statement = format!("CREATE SCHEMA IF NOT EXISTS \"{}\"", identifier);
        query(&statement).execute(&state.pool).await?;

        // 2. Pin all subsequent queries on this single connection to the
        // test-specific schema.
        let statement = format!("SET search_path TO \"{}\"", identifier);
        query(&statement).execute(&state.pool).await?;

        // 3. Because of the above SET command, apply migrations into the test
        // schema without needing to specify in-script.
        migrate!().run(&state.pool).await?;

        Ok(identifier)
    }
}

pub async fn middleware(
    #[cfg(feature = "testing")] headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> HandlerResult<impl IntoResponse> {
    // Extract the per-test generated ID. The handlers use this to query test
    // schemas, write & read test keys etc. for test runs.
    #[cfg(feature = "testing")]
    if let Some(header) = headers.get(TEST_ID_HEADER_KEY)
        && let Ok(header_value) = header.to_str()
    {
        let identifier = String::from(header_value);
        request.extensions_mut().insert(identifier);

        return Ok(next.run(request).await);
    }

    let identifier = String::from("public");
    request.extensions_mut().insert(identifier);

    Ok(next.run(request).await)
}

pub async fn index() -> HandlerResult<impl IntoResponse> {
    Ok("Hello, world!")
}
