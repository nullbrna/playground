use axum::Router;
use axum::routing::IntoMakeService;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use crate::handler::HandlerState;

mod handler;

/// NOTE: Can panic. Ensure this ONLY runs during startup.
fn setup_environment() -> String {
    let filter = EnvFilter::from_default_env();
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_file(false)
        .json()
        .init();

    let port = std::env::var("PORT").expect("Missing \"PORT\" variable");
    assert!(port.parse::<u16>().is_ok(), "Invalid \"PORT\" variable");

    format!("0.0.0.0:{port}")
}

async fn create_configured_router() -> IntoMakeService<Router> {
    let state = HandlerState::new().await;

    let middleware = axum::middleware::from_fn(handler::middleware);
    let tracing = TraceLayer::new_for_http()
        .on_request(())
        .on_body_chunk(())
        .on_eos(());

    Router::new()
        .route("/", axum::routing::get(handler::index))
        .route("/sock", axum::routing::get(handler::sock::core))
        .layer(middleware)
        .layer(tracing)
        .with_state(state)
        .into_make_service()
}

#[tokio::main]
async fn main() {
    let host = setup_environment();
    let listener = TcpListener::bind(&host)
        .await
        .expect("Binding to the host address");

    let router = create_configured_router().await;
    tracing::info!("Hello, world!");

    axum::serve(listener, router)
        .await
        .expect("An error on the socket bubbled up");
}
