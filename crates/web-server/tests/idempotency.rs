use std::sync::OnceLock;

use reqwest::Client;
use reqwest::StatusCode;
use uuid::Uuid;

const ENDPOINT: &str = "http://localhost:8080/idempotency";

static CLIENT: OnceLock<Client> = OnceLock::new();

fn http_client() -> &'static Client {
    let setter = || {
        Client::builder()
            .pool_max_idle_per_host(10)
            .build()
            .unwrap()
    };

    CLIENT.get_or_init(setter)
}

#[tokio::test]
async fn initial_request_misses_cache() {
    let id = Uuid::new_v4().to_string();
    let response = http_client()
        .post(ENDPOINT)
        .header("Idempotency-Key", &id)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(response.text().await.unwrap(), "CACHE_MISS");
}

#[tokio::test]
async fn repeated_request_hits_cache() {
    let id = Uuid::new_v4().to_string();
    let make_request = async || {
        http_client()
            .post(ENDPOINT)
            .header("Idempotency-Key", &id)
            .send()
            .await
            .unwrap()
    };

    make_request().await;
    let response = make_request().await;

    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(response.text().await.unwrap(), "CACHE_HIT");
}

#[tokio::test]
async fn different_key_misses_cache() {
    let make_request = async || {
        let id = Uuid::new_v4().to_string();
        http_client()
            .post(ENDPOINT)
            .header("Idempotency-Key", &id)
            .send()
            .await
            .unwrap()
    };

    make_request().await;
    let response = make_request().await;

    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(response.text().await.unwrap(), "CACHE_MISS");
}

#[tokio::test]
async fn missing_key_is_bad_request() {
    let response = http_client().post(ENDPOINT).send().await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
