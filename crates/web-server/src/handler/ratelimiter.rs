use std::net::SocketAddr;

use axum::extract::ConnectInfo;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use redis::cmd;

use crate::AppState;
use crate::handler::HandlerError;
use crate::handler::HandlerResult;

const LIMIT_COUNT: i32 = 10;
const LIMIT_WINDOW: usize = 60;

pub async fn core(
    State(state): State<AppState>,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
) -> HandlerResult<impl IntoResponse> {
    let limiter_key = format!("rate_limiter:{}", address.ip());
    // Clone the connection manager for a new slot from the pool.
    let mut connection = state.redis.clone();

    // Atomically increment to avoid concurrent requests potentially facing a
    // race condition e.g. requests A & B both increment the same value.
    let incremented_count: i32 = cmd("INCR")
        .arg(&limiter_key)
        .query_async(&mut connection)
        .await?;

    // The above will ensure that at minimum, the count is 1 if non-existing. In
    // this case, the expiration for the key needs to be set.
    if incremented_count == 1 {
        // NOTE: Compiler can't infer what the return type is.
        let _: () = cmd("EXPIRE")
            .arg(&limiter_key)
            .arg(LIMIT_WINDOW)
            .query_async(&mut connection)
            .await?;
    }

    if incremented_count > LIMIT_COUNT {
        Err(HandlerError::from(StatusCode::TOO_MANY_REQUESTS))
    } else if incremented_count == 1 {
        Ok((StatusCode::OK, "LIMIT_FIRST"))
    } else {
        Ok((StatusCode::OK, "LIMIT_ONGOING"))
    }
}
