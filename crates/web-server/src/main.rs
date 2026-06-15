use std::net::SocketAddr;
use std::str::FromStr;

use axum::Router;
use axum::extract::connect_info::IntoMakeServiceWithConnectInfo;
use axum::routing::get;
use axum::routing::post;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::Level;

use crate::handler::HandlerState;

mod handler;

fn setup_environment() -> String {
    let env_or_default = |key: &str, fallback: &str| {
        let fallback = fallback.into();
        std::env::var(key).unwrap_or(fallback)
    };

    let env_level = env_or_default("LOG_LEVEL", "DEBUG");
    let port = env_or_default("PORT", "8080");

    let level = Level::from_str(&env_level).unwrap_or(Level::DEBUG);
    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_target(false)
        .compact()
        .init();

    tracing::info!("Logging: {level}");
    format!("0.0.0.0:{port}")
}

async fn create_router() -> anyhow::Result<IntoMakeServiceWithConnectInfo<Router, SocketAddr>> {
    let state = HandlerState::new().await?;
    let middleware = axum::middleware::from_fn_with_state(state.clone(), handler::middleware);

    // NOTE: Logs response status and latency ONLY in debug builds.
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

    Ok(router)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let host = setup_environment();

    let listener = TcpListener::bind(&host).await?;
    let router = create_router().await?;

    tracing::info!("Starting: {host}");
    axum::serve(listener, router).await?;

    Ok(())
}
