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

        // 1. Acquire a connection to ensure the default schema path change will
        // rollback on failure to be safe.
        // NOTE: Connection is dropped on return.
        let mut tx = self.pool.begin().await?;

        // 2. Each test generates a unique identifier used for its dedicated SQL
        // schema and any related test resource. This identifier will become the
        // dedicated test data schema.
        let statement = format!(r#"CREATE SCHEMA IF NOT EXISTS "{}""#, identifier);
        query(&statement).execute(&mut *tx).await?;

        // 3. Prepare for the migration scripts by configuring them to write to
        // the above created schema.
        // NOTE: Handlers use the identifier to query the schema directly.
        let statement = format!(r#"SET LOCAL search_path TO "{}""#, identifier);
        query(&statement).execute(&mut *tx).await?;

        // 4. Run the migration scripts against the transaction-default schema
        // to initialise the table layout.
        migrate!().run(&mut *tx).await?;
        tx.commit().await?;

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
