use std::net::SocketAddr;

use axum::Router;
use axum::extract::connect_info::IntoMakeServiceWithConnectInfo;
use axum::routing::get;
use axum::routing::post;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

use crate::handler::HandlerState;

mod handler;

fn setup_environment() -> String {
    let env_filter = EnvFilter::from_default_env();
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .compact()
        .init();

    let port = std::env::var("PORT").expect("Missing PORT env");
    format!("0.0.0.0:{port}")
}

async fn create_configured_router() -> IntoMakeServiceWithConnectInfo<Router, SocketAddr> {
    let state = HandlerState::new().await;
    let middleware = axum::middleware::from_fn_with_state(state.clone(), handler::middleware);

    let router = Router::new()
        .route("/", get(handler::index))
        .route("/idempotency", post(handler::idempotency::core))
        .route("/rate-limiter", get(handler::ratelimiter::core))
        .layer(middleware)
        .with_state(state)
        .into_make_service_with_connect_info::<SocketAddr>();

    router
}

#[tokio::main]
async fn main() {
    let host = setup_environment();
    let listener = TcpListener::bind(&host)
        .await
        .expect("Binding to the host address");

    let router = create_configured_router().await;
    axum::serve(listener, router)
        .await
        .expect("An error on the socket somehow bubbled up");
}
