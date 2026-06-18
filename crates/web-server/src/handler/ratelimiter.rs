// Rate limiting controls the rate of requests received, protecting resources
// from being overwhelmed. If a client exceeds a threshold, further requests are
// blocked until the limit resets.
//
// Patterns for rate limiting and counting requests:
// 1. Fixed: Time buckets e.g. per 60 seconds
// 2. Sliding: Moving buckets e.g. last 60 seconds from now
// 3. Rolling/Token Bucket: Each request uses a token and refills over time

use std::net::SocketAddr;

use axum::Extension;
use axum::extract::ConnectInfo;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::handler::HandlerResult;
use crate::handler::HandlerState;

// Requests allowed within a window.
const LIMIT_COUNT: i64 = 10;
// TTL (seconds) for a key.
const LIMIT_WINDOW: i64 = 2;

const FIRST_TEXT: &str = "LIMIT_FIRST";
const ONGOING_TEXT: &str = "LIMIT_ONGOING";

pub async fn core(
    Extension(identifier): Extension<String>,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
    State(state): State<HandlerState>,
) -> HandlerResult<impl IntoResponse> {
    let mut redis_conn = state.redis.clone();

    let ip_addr = address.ip();
    let limiter_key = format!("{}:rate_limiter:{}", identifier, ip_addr);

    // Increment (or initialise to 1) the request count against the IP. Each
    // request within the window resets the keys expiry timer.
    let (count, _): (i64, i32) = redis::pipe()
        .atomic()
        .incr(&limiter_key, 1)
        .expire(&limiter_key, LIMIT_WINDOW)
        .query_async(&mut redis_conn)
        .await?;

    if count > LIMIT_COUNT {
        tracing::error!(addr = %ip_addr, "[RATE_LIMITER] Suspended for exceeding limit");
        return Err(StatusCode::TOO_MANY_REQUESTS)?;
    } else if count == 1 {
        tracing::info!(addr = %ip_addr, "[RATE_LIMITER] Request window started");
        return Ok((StatusCode::OK, FIRST_TEXT));
    }

    Ok((StatusCode::OK, ONGOING_TEXT))
}

#[cfg(test)]
mod tests {
    use crate::handler::HandlerState;
    use crate::handler::TEST_ID_HEADER_KEY;
    use crate::handler::ratelimiter::FIRST_TEXT;
    use crate::handler::ratelimiter::LIMIT_COUNT;
    use crate::handler::ratelimiter::ONGOING_TEXT;

    use axum::http::StatusCode;
    use reqwest::Client;
    use reqwest::Response;

    const ENDPOINT: &str = "http://localhost:8080/rate-limiter";

    async fn make_request(identifier: &str) -> Response {
        Client::new()
            .get(ENDPOINT)
            .header(TEST_ID_HEADER_KEY, identifier)
            .send()
            .await
            .expect("Sending HTTP request")
    }

    #[tokio::test]
    async fn should_200_initialise_cache() {
        let identifier = HandlerState::setup_for_test().await;

        let response = make_request(&identifier).await;
        assert_eq!(response.status(), StatusCode::OK);

        let body = response.text().await.expect("Parsing first response body");
        assert_eq!(body, FIRST_TEXT);
    }

    #[tokio::test]
    async fn should_200_ongoing_increment() {
        let identifier = HandlerState::setup_for_test().await;

        make_request(&identifier).await;
        for _ in 0..LIMIT_COUNT - 1 {
            let response = make_request(&identifier).await;
            assert_eq!(response.status(), StatusCode::OK);

            let body = response
                .text()
                .await
                .expect("Parsing ongoing response body");

            assert_eq!(body, ONGOING_TEXT);
        }
    }

    #[tokio::test]
    async fn should_429_over_limit() {
        let identifier = HandlerState::setup_for_test().await;

        for _ in 0..LIMIT_COUNT {
            make_request(&identifier).await;
        }

        let response = make_request(&identifier).await;
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    }
}
