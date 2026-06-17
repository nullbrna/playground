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
        if !value.is_informational() && !value.is_success() {
            tracing::warn!(value = ?value, "Non-positive status code returned");
        }

        Self(value)
    }
}

impl From<anyhow::Error> for HandlerError {
    fn from(value: anyhow::Error) -> Self {
        tracing::error!("[UNEXPECTED] {value}");
        Self(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

impl From<sqlx::Error> for HandlerError {
    fn from(value: sqlx::Error) -> Self {
        if matches!(value, sqlx::Error::RowNotFound) {
            tracing::warn!("[DATABASE] Unhandled empty response");
            return Self(StatusCode::OK);
        }

        tracing::error!("[DATABASE] {value}");
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
    /// NOTE: [`PgPool`] is an [`std::sync::Arc`].
    pool: PgPool,
    /// Connection to Redis. Automatically reconnects when needed.
    /// NOTE: [`redis::aio::ConnectionManager`] is an [`std::sync::Arc`].
    redis: ConnectionManager,
}

impl HandlerState {
    pub async fn new() -> Self {
        let database_resource = std::env::var("DATABASE_URL").expect("Missing DATABASE_URL env");
        let redis_resource = std::env::var("REDIS_URL").expect("Missing REDIS_URL env");

        // Tests need a dedicated connection. Tests run in parallel so each
        // schema setup could overwrite the search path and create data races.
        let connection_count = if cfg!(test) { 1 } else { 10 };
        let pool = PgPoolOptions::new()
            .max_connections(connection_count)
            .connect(&database_resource)
            .await
            .expect("Opening first pool connection");

        sqlx::migrate!()
            .run(&pool)
            .await
            .expect("Running migration script(s)");

        let client = Client::open(redis_resource).expect("Validating Redis connection URI");
        let redis = ConnectionManager::new(client)
            .await
            .expect("Initialising Redis connection");

        Self { pool, redis }
    }

    #[cfg(test)]
    async fn setup_for_test() -> String {
        use uuid::Uuid;

        let state = Self::new().await;
        let identifier = Uuid::new_v4().to_string();

        // 1. Create an isolated schema for the current test by the unique ID.
        let statement = format!("CREATE SCHEMA IF NOT EXISTS \"{identifier}\"");
        sqlx::query(&statement)
            .execute(&state.pool)
            .await
            .expect("Creating unique schema as default");

        // 2. Pin all subsequent queries to the schema.
        let statement = format!("SET search_path TO \"{identifier}\"");
        sqlx::query(&statement)
            .execute(&state.pool)
            .await
            .expect("Setting schema as the query default");

        // 3. Apply migrations into the schema.
        sqlx::migrate!()
            .run(&state.pool)
            .await
            .expect("Applying migrations into schema");

        identifier
    }
}

pub async fn middleware(
    #[cfg(feature = "testing")] headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> HandlerResult<impl IntoResponse> {
    #[cfg(feature = "testing")]
    if let Some(header) = headers.get(TEST_ID_HEADER_KEY)
        && let Ok(header) = header.to_str()
    {
        let identifier = String::from(header);
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
