use axum::extract::Request;
use axum::extract::State;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::response::Response;
use sqlx::PgPool;
use sqlx::query_scalar;
use tracing::error;
use tracing::info;

use crate::AppState;

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
        error!("[DATABASE] {value}");

        if matches!(value, sqlx::Error::RowNotFound) {
            return Self(StatusCode::NOT_FOUND);
        }

        Self(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

impl From<anyhow::Error> for HandlerError {
    fn from(value: anyhow::Error) -> Self {
        error!("[UNEXPECTED] {value}");
        Self(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

pub async fn middleware_handler(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> HandlerResult<impl IntoResponse> {
    info!("[MIDDLEWARE] hit: {state:?}");
    Ok(next.run(request).await)
}

pub async fn index_handler(State(state): State<AppState>) -> HandlerResult<impl IntoResponse> {
    let user = find_user(&state.pool, 1).await?;
    Ok(user)
}

async fn find_user(pool: &PgPool, id: i32) -> HandlerResult<String> {
    let statement = r#"
        SELECT first_name
        FROM users
        WHERE id = $1
    "#;

    let user_name = query_scalar(statement).bind(id).fetch_one(pool).await?;
    Ok(user_name)
}
