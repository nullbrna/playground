use axum::extract::Request;
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

#[cfg(debug_assertions)]
const TEST_ID_HEADER: &str = "X-Test-Identifier";

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
        let pool = {
            let resource = std::env::var("DATABASE_URL")?;

            PgPoolOptions::new()
                .max_connections(10)
                .connect(&resource)
                .await?
        };

        let redis = {
            let resource = std::env::var("REDIS_URL")?;
            let client = Client::open(resource)?;

            ConnectionManager::new(client).await?
        };

        Ok(Self { pool, redis })
    }

    #[cfg(test)]
    async fn new_unique_test_schema(&self, identifier: &str) -> anyhow::Result<()> {
        use sqlx::migrate;
        use sqlx::query;

        // Acquire ONE connection to ensure schema creation, path setting, and
        // migration run in the same context.
        let mut conn = self.pool.acquire().await?;

        // Each test generates a unique identifier used for its dedicated SQL
        // schema and any related test resource.
        let statement = format!(r#"CREATE SCHEMA IF NOT EXISTS "{}""#, identifier);
        query(&statement).execute(&mut *conn).await?;

        // Before running the migration scripts to initialise the tables; point
        // to the newly created schema.
        let statement = format!(r#"SET search_path TO "{}""#, identifier);
        query(&statement).execute(&mut *conn).await?;

        migrate!().run(&mut *conn).await?;
        // NOT necessary but reset the connections schema back to default.
        query("RESET search_path").execute(&mut *conn).await?;

        Ok(())
    }
}

pub async fn middleware(
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> HandlerResult<impl IntoResponse> {
    #[cfg(not(debug_assertions))]
    {
        let identifier = String::from("public");
        request.extensions_mut().insert(identifier);

        return Ok(next.run(request).await);
    }

    // Extract the per-test ID matching a set-up schema. Pass along to the
    // handlers to query that tests unique data.
    if let Some(header) = headers.get(TEST_ID_HEADER)
        && let Ok(identifier) = header.to_str().map(String::from)
    {
        request.extensions_mut().insert(identifier);
        return Ok(next.run(request).await);
    }

    let identifier = String::from("local");
    request.extensions_mut().insert(identifier);

    Ok(next.run(request).await)
}

pub async fn index() -> HandlerResult<impl IntoResponse> {
    Ok("Hello, world!")
}
