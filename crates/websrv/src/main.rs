use std::net::SocketAddr;

use axum::Router;
use axum::extract::connect_info::IntoMakeServiceWithConnectInfo;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

use crate::handler::HandlerState;

mod handler;

/// NOTE: Can panic. Ensure this ONLY runs during startup.
fn setup_environment() -> String {
    let filter = EnvFilter::from_default_env();

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_file(false)
        .compact()
        .init();

    let port = std::env::var("PORT").expect("Missing \"PORT\" variable");
    assert!(port.parse::<u16>().is_ok(), "Invalid \"PORT\" variable");

    format!("0.0.0.0:{port}")
}

async fn create_configured_router() -> IntoMakeServiceWithConnectInfo<Router, SocketAddr> {
    let state = HandlerState::new().await;
    let middleware = axum::middleware::from_fn(handler::middleware);

    Router::new()
        .route("/", axum::routing::get(handler::index))
        .layer(middleware)
        .with_state(state)
        .into_make_service_with_connect_info::<SocketAddr>()
}

#[tokio::main]
async fn main() {
    let host = setup_environment();
    let listener = TcpListener::bind(&host)
        .await
        .expect("Binding to the host address");

    let router = create_configured_router().await;
    tracing::debug!("Ready for connections");

    axum::serve(listener, router)
        .await
        .expect("An error on the socket bubbled up");
}
