use std::net::SocketAddr;

use axum::Extension;
use axum::extract::ConnectInfo;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::handler::HandlerResult;
use crate::handler::HandlerState;

const LIMIT_COUNT: i64 = 10; // Number of requests allowed within the window.
const LIMIT_WINDOW: i64 = 2; // TTL (seconds) for a key.
const FIRST_TEXT: &str = "LIMIT_FIRST";
const ONGOING_TEXT: &str = "LIMIT_ONGOING";

pub async fn core(
    Extension(identifier): Extension<String>,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
    State(state): State<HandlerState>,
) -> HandlerResult<impl IntoResponse> {
    let mut connection = state.redis.clone();

    let ip = address.ip();
    let limiter_key = format!("{}:rate_limiter:{}", identifier, ip);

    // Increment/initialise (to 1) the request count against the IP. Each
    // request within the window resets the keys expiry timer.
    //
    // Patterns for rate-limiting & counting requests:
    // 1. Fixed: Time buckets e.g. per 60 seconds
    // 2. Sliding: Moving buckets e.g. last 60 seconds from now
    // 3. Rolling/Token Bucket: Each request uses a token, refills over time
    let (count, _): (i64, i32) = redis::pipe()
        .atomic()
        .incr(&limiter_key, 1)
        .expire(&limiter_key, LIMIT_WINDOW)
        .query_async(&mut connection)
        .await?;

    if count > LIMIT_COUNT {
        tracing::error!("[RATE_LIMITER] {ip} suspended");
        return Err(StatusCode::TOO_MANY_REQUESTS)?;
    }

    tracing::info!("[RATE_LIMITER] {ip} has requested {count} time(s)");
    if count == 1 {
        return Ok((StatusCode::OK, FIRST_TEXT));
    }

    Ok((StatusCode::OK, ONGOING_TEXT))
}

#[cfg(test)]
mod tests {
    use crate::handler::TEST_ID_HEADER;

    use super::*;

    use reqwest::Client;
    use reqwest::Response;
    use uuid::Uuid;

    const ENDPOINT: &str = "http://localhost:8080/rate-limiter";

    async fn setup() -> String {
        let app = HandlerState::new().await.unwrap();
        let identifier = Uuid::new_v4().to_string();

        app.new_unique_test_schema(&identifier).await.unwrap();
        identifier
    }

    async fn make_request(identifier: &str) -> Response {
        Client::new()
            .get(ENDPOINT)
            .header(TEST_ID_HEADER, identifier)
            .send()
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn should_200_initialise_cache() {
        let identifier = setup().await;

        let response = make_request(&identifier).await;

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.text().await.unwrap(), FIRST_TEXT);
    }

    #[tokio::test]
    async fn should_200_ongoing_increment() {
        let identifier = setup().await;

        make_request(&identifier).await;
        for _ in 0..LIMIT_COUNT - 1 {
            let response = make_request(&identifier).await;

            assert_eq!(response.status(), StatusCode::OK);
            assert_eq!(response.text().await.unwrap(), ONGOING_TEXT);
        }
    }

    #[tokio::test]
    async fn should_429_over_limit() {
        let identifier = setup().await;

        for _ in 0..LIMIT_COUNT {
            make_request(&identifier).await;
        }

        let response = make_request(&identifier).await;
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    }
}
