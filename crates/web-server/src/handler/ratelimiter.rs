use std::net::SocketAddr;

use axum::extract::ConnectInfo;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use redis::pipe;
use tracing::error;
use tracing::info;

use crate::AppState;
use crate::handler::HandlerResult;

// Number of requests allowed within the window.
const LIMIT_COUNT: i64 = 10;
// TTL (seconds) for a key.
const LIMIT_WINDOW: i64 = 2;

pub async fn core(
    ConnectInfo(address): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
) -> HandlerResult<impl IntoResponse> {
    let mut connection = state.redis.clone();
    let limiter_key = format!("rate_limiter:{}", address.ip());

    // Increment/initialise (to 1) the request count against the IP. Each
    // request within the window resets the keys expiry timer.
    //
    // Patterns for rate-limiting & counting requests:
    // 1. Fixed: Time buckets e.g. per 60 seconds
    // 2. Sliding: Moving buckets e.g. last 60 seconds from now
    // 3. Rolling/Token Bucket: Each request uses a token, refills over time
    let (count, _): (i64, i32) = pipe()
        .atomic()
        .incr(&limiter_key, 1)
        .expire(&limiter_key, LIMIT_WINDOW)
        .query_async(&mut connection)
        .await?;

    if count > LIMIT_COUNT {
        error!("[RATE_LIMITER] {limiter_key} suspended");
        return Err(StatusCode::TOO_MANY_REQUESTS)?;
    }

    info!("[RATE_LIMITER] {limiter_key} has requested {count} time(s)");
    if count == 1 {
        return Ok((StatusCode::OK, "LIMIT_FIRST"));
    }

    Ok((StatusCode::OK, "LIMIT_ONGOING"))
}
