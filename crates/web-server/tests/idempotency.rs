use std::sync::OnceLock;

use reqwest::Client;
use reqwest::StatusCode;
use uuid::Uuid;

const ENDPOINT: &str = "http://localhost:8080/idempotency";
// Industry-standard header.
const HEADER_KEY: &str = "Idempotency-Key";
// Status indicating if the key was previously stored or not.
const SUCCESS_TEXT: &str = "CACHE_HIT";
const FAILURE_TEXT: &str = "CACHE_MISS";

static CLIENT: OnceLock<Client> = OnceLock::new();

fn http_client() -> &'static Client {
    let setter = || Client::builder().build().unwrap();
    CLIENT.get_or_init(setter)
}

#[tokio::test]
async fn initial_request_misses_cache() {
    // One-off ID generated that shouldn't be in-cache.
    let id = Uuid::new_v4().to_string();

    let response = http_client()
        .post(ENDPOINT)
        .header(HEADER_KEY, &id)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(response.text().await.unwrap(), FAILURE_TEXT);
}

#[tokio::test]
async fn repeated_request_hits_cache() {
    // One-off ID generated to be used multiple times & stored in-cache.
    let id = Uuid::new_v4().to_string();
    let make_request = async || {
        http_client()
            .post(ENDPOINT)
            .header(HEADER_KEY, &id)
            .send()
            .await
            .unwrap()
    };

    make_request().await;
    let response = make_request().await;

    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(response.text().await.unwrap(), SUCCESS_TEXT);
}

#[tokio::test]
async fn different_key_misses_cache() {
    // Each request generates a new ID, hitting no cached key.
    let make_request = async || {
        let id = Uuid::new_v4().to_string();
        http_client()
            .post(ENDPOINT)
            .header(HEADER_KEY, &id)
            .send()
            .await
            .unwrap()
    };

    make_request().await;
    let response = make_request().await;

    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(response.text().await.unwrap(), FAILURE_TEXT);
}

#[tokio::test]
async fn missing_key_is_bad_request() {
    let response = http_client().post(ENDPOINT).send().await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
