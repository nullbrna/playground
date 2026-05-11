use std::net::SocketAddr;
use std::str::FromStr;

use axum::Router;
use axum::routing::get;
use axum::routing::post;
use tokio::net::TcpListener;
use tower_http::trace::DefaultMakeSpan;
use tower_http::trace::DefaultOnResponse;
use tower_http::trace::TraceLayer;
use tracing::Level;

use crate::handler::HandlerState;

mod handler;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let level = {
        let fallback_level = String::from("DEBUG");
        let env_level = std::env::var("LOG_LEVEL").unwrap_or(fallback_level);

        let level = Level::from_str(&env_level).unwrap_or(Level::DEBUG);
        tracing_subscriber::fmt()
            .with_max_level(level)
            .with_target(false)
            .compact()
            .init();

        tracing::info!("logging level: {level}");
        level
    };

    let host = {
        let fallback_port = String::from("8080");
        let port = std::env::var("PORT").unwrap_or(fallback_port);

        format!("0.0.0.0:{port}")
    };

    let listener = TcpListener::bind(&host).await?;
    let state = HandlerState::new().await?;

    let middleware = axum::middleware::from_fn_with_state(state.clone(), handler::middleware);
    // Log response status & latency. SQL queries are logged in the ORM crate.
    let tracing = {
        let req_span = DefaultMakeSpan::new().level(level);
        let res_span = DefaultOnResponse::new().level(level);

        TraceLayer::new_for_http()
            .make_span_with(req_span)
            .on_response(res_span)
    };

    let router = Router::new()
        .route("/", get(handler::index))
        .route("/idempotency", post(handler::idempotency::core))
        .route("/rate-limiter", get(handler::ratelimiter::core))
        .layer(middleware)
        .layer(tracing)
        .with_state(state)
        // Allows for reading the request IP through an extension.
        .into_make_service_with_connect_info::<SocketAddr>();

    tracing::info!("starting: {host}");
    axum::serve(listener, router).await?;

    Ok(())
}
