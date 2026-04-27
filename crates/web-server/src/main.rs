use std::env::var;
use std::fmt::Display;
use std::process::exit;
use std::str::FromStr;

use axum::Router;
use axum::middleware::from_fn_with_state;
use axum::routing::get;
use axum::serve;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tower_http::trace::DefaultMakeSpan;
use tower_http::trace::DefaultOnResponse;
use tower_http::trace::TraceLayer;
use tracing::Level;
use tracing::error;
use tracing::info;
use tracing_subscriber::fmt;

use crate::handler::index_handler;
use crate::handler::middleware_handler;

mod handler;

fn log_error<T: Display>(err: &T) {
    error!("[ROOT] {err}");
}

fn init_logger() -> Level {
    let level = {
        let default = String::from("DEBUG");
        let lvl = var("LOG_LEVEL").unwrap_or(default);

        Level::from_str(&lvl).unwrap_or(Level::DEBUG)
    };

    fmt()
        .with_max_level(level)
        .with_target(false)
        .compact()
        .init();

    info!("logging level: {level}");
    level
}

#[derive(Debug, Clone)]
struct AppState {
    pool: PgPool,
}

impl AppState {
    async fn new() -> Self {
        let Ok(connection) = var("DATABASE_URL").inspect_err(log_error) else {
            exit(1);
        };

        let Ok(pool) = PgPoolOptions::new()
            .connect(&connection)
            .await
            .inspect_err(log_error)
        else {
            exit(1);
        };

        Self { pool }
    }
}

async fn new_router(log_level: Level) -> Router {
    let state = AppState::new().await;

    let middleware = from_fn_with_state(state.clone(), middleware_handler);
    let tracing = {
        let req_span = DefaultMakeSpan::new().level(log_level);
        let res_span = DefaultOnResponse::new().level(log_level);

        TraceLayer::new_for_http()
            .make_span_with(req_span)
            .on_response(res_span)
    };

    Router::new()
        .route("/", get(index_handler))
        .layer(middleware)
        .layer(tracing)
        .with_state(state)
}

#[tokio::main]
async fn main() {
    let log_level = init_logger();

    let host = {
        let default = String::from("8080");
        let port = var("PORT").unwrap_or(default);

        format!("0.0.0.0:{port}")
    };

    let Ok(listener) = TcpListener::bind(&host).await else {
        exit(1);
    };

    // NOTE: This can be unwrapped as the error handling is to sleep & retry.
    //
    // "Errors on the TCP socket will be handled by sleeping for a short while
    // (currently, one second)" - axum::serve
    info!("starting: {host}");
    serve(listener, new_router(log_level).await).await.unwrap();
}
