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

impl From<StatusCode> for HandlerError {
  fn from(statcode: StatusCode) -> Self {
    Self(statcode)
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
  dbpool: PgPool,
}

impl HandlerState {
  /// NOTE: Prefer to panic on startup to exit ASAP. Ensure this is only called
  /// at server startup, never before the server boots.
  pub async fn new() -> Self {
    let urifromenv = std::env::var("DATABASE_URI")
      .ok()
      .filter(|val| val.len() > 0)
      .expect("Missing/unset DATABASE_URI variable");

    let dbpool = PgPoolOptions::new()
      .max_connections(15)
      .connect(&urifromenv)
      .await
      .expect("Connecting to/pinging new database pool");

    Self { dbpool }
  }
}

pub async fn health_check(State(state): State<HandlerState>) -> HandlerResult<impl IntoResponse> {
  let statement = r#"
    SELECT 1;
  "#;

  sqlx::query(statement)
    .execute(&state.dbpool)
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
    let healthresp = Client::new().get(HEALTH_CHECK_URL).send().await.unwrap();
    assert_eq!(healthresp.status(), StatusCode::OK);
  }
}
