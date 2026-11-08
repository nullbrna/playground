use axum::Router;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use crate::handler::HandlerState;

mod handler;

fn setup_environment() {
    let filter_from_env = EnvFilter::from_default_env();

    if cfg!(debug_assertions) {
        tracing_subscriber::fmt()
            .with_env_filter(filter_from_env)
            .with_target(false)
            .without_time()
            .compact()
            .init();
    } else {
        tracing_subscriber::fmt().with_env_filter(filter_from_env).json().init();
    }
}

async fn create_configured_router() -> Router {
    let handler_state = HandlerState::new().await;
    let tracing_layer = TraceLayer::new_for_http().on_request(()).on_body_chunk(()).on_eos(());

    Router::new()
        .route("/", axum::routing::get(handler::health_check))
        .layer(tracing_layer)
        .with_state(handler_state)
}

#[tokio::main]
async fn main() {
    setup_environment();
    let app_router = create_configured_router().await;

    tracing::info!("Starting server...");
    let listener = TcpListener::bind("0.0.0.0:8080").await.expect("Binding a TCP listener");
    axum::serve(listener, app_router)
        .await
        .expect("Unexpected error bubbled up to the root");
}
