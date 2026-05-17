use std::net::SocketAddr;
use std::str::FromStr;

use axum::Router;
use axum::routing::get;
use axum::routing::post;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::Level;

use crate::handler::HandlerState;

mod handler;

fn setup_environment() -> String {
    let env_level = {
        let default = String::from("DEBUG");
        std::env::var("LOG_LEVEL").unwrap_or(default)
    };

    let level = Level::from_str(&env_level).unwrap_or(Level::DEBUG);
    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_target(false)
        .compact()
        .init();

    tracing::info!("Logging: {level}");
    let port = {
        let default = String::from("8080");
        std::env::var("PORT").unwrap_or(default)
    };

    format!("0.0.0.0:{port}")
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let host = setup_environment();

    let listener = TcpListener::bind(&host).await?;
    let state = HandlerState::new().await?;

    let middleware = axum::middleware::from_fn_with_state(state.clone(), handler::middleware);
    // NOTE: Logs response status & latency ONLY in debug builds.
    let tracing = TraceLayer::new_for_http();

    let router = Router::new()
        .route("/", get(handler::index))
        .route("/idempotency", post(handler::idempotency::core))
        .route("/rate-limiter", get(handler::ratelimiter::core))
        .layer(middleware)
        .layer(tracing)
        .with_state(state)
        // Allows for reading the request IP through an extension.
        .into_make_service_with_connect_info::<SocketAddr>();

    tracing::info!("Starting: {host}");
    axum::serve(listener, router).await?;

    Ok(())
}
