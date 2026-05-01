use axum::extract::Request;
use axum::extract::State;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::response::Response;
use tracing::error;
use tracing::info;

use crate::AppState;

pub mod idempotency;

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
    Ok(format!("Hello, world! - {:?}", state))
}
