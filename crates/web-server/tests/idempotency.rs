use std::sync::OnceLock;

use reqwest::Client;
use reqwest::Response;
use reqwest::StatusCode;
use uuid::Uuid;

const ENDPOINT: &str = "http://localhost:8080/idempotency";
const SUCCESS_TEXT: &str = "CACHE_HIT";
const FAILURE_TEXT: &str = "CACHE_MISS";

static CLIENT: OnceLock<Client> = OnceLock::new();

fn http_client() -> &'static Client {
    let setter = || Client::builder().build().unwrap();
    CLIENT.get_or_init(setter)
}

async fn send_request(id: Option<&str>) -> Response {
    let id = id.map(String::from).unwrap_or(Uuid::new_v4().to_string());

    http_client()
        .post(ENDPOINT)
        .header("Idempotency-Key", id)
        .send()
        .await
        .unwrap()
}

#[tokio::test]
async fn initial_request_misses_cache() {
    let response = send_request(None).await;

    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(response.text().await.unwrap(), FAILURE_TEXT);
}

#[tokio::test]
async fn repeated_request_hits_cache() {
    let id = Uuid::new_v4().to_string();

    send_request(Some(&id)).await;
    let response = send_request(Some(&id)).await;

    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(response.text().await.unwrap(), SUCCESS_TEXT);
}

#[tokio::test]
async fn different_key_misses_cache() {
    send_request(None).await;
    let response = send_request(None).await;

    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(response.text().await.unwrap(), FAILURE_TEXT);
}

#[tokio::test]
async fn missing_key_is_bad_request() {
    let response = http_client().post(ENDPOINT).send().await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

// #[tokio::test]
// async fn internal_example() {
//     let response = http_client()
//         .post("http://localhost:8080/internal/idempotency")
//         .send()
//         .await
//         .unwrap();
//
//     assert_eq!(response.status(), StatusCode::OK);
// }
