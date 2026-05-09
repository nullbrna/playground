use std::env::var;
use std::net::SocketAddr;
use std::str::FromStr;

use axum::Router;
use axum::extract::connect_info::IntoMakeServiceWithConnectInfo;
use axum::middleware::from_fn_with_state;
use axum::routing::get;
use axum::routing::post;
use axum::serve;
use redis::Client;
use redis::aio::ConnectionManager;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tower_http::trace::DefaultMakeSpan;
use tower_http::trace::DefaultOnResponse;
use tower_http::trace::TraceLayer;
use tracing::Level;
use tracing::info;
use tracing_subscriber::fmt;

use crate::handler::idempotency;
use crate::handler::index;
use crate::handler::ratelimiter;

mod handler;

fn start_logger() -> Level {
    let log_level = {
        let fallback_level = String::from("DEBUG");
        let env_level = var("LOG_LEVEL").unwrap_or(fallback_level);

        Level::from_str(&env_level).unwrap_or(Level::DEBUG)
    };

    fmt()
        .with_max_level(log_level)
        .with_target(false)
        .compact()
        .init();

    info!("logging level: {log_level}");
    log_level
}

#[derive(Debug, Clone)]
struct AppState {
    /// Connection pool for Postgres.
    /// NOTE: [`PgPool`] under-the-hood is an [`std::sync::Arc`].
    pool: PgPool,
    /// Connection to Redis. Automatically reconnects when needed.
    redis: ConnectionManager,
}

impl AppState {
    async fn new() -> Self {
        // NOTE: Prefer unwrapping for stack trace at start-up. Will fail on a
        // missing database URI or opening/connecting-to the pool.
        let pool = {
            let resource = var("DATABASE_URL").unwrap();
            PgPoolOptions::new().connect(&resource).await.unwrap()
        };

        // NOTE: Prefer unwrapping for stack trace at start-up. Will fail on
        // URI validation or the initial connection.
        let redis = {
            let resource = var("REDIS_URL").unwrap();
            let client = Client::open(resource).unwrap();

            ConnectionManager::new(client).await.unwrap()
        };

        Self { pool, redis }
    }
}

type RouterWithIP = IntoMakeServiceWithConnectInfo<Router, SocketAddr>;
async fn setup_service(log_level: Level) -> (TcpListener, RouterWithIP) {
    let host = {
        let fallback_port = String::from("8080");
        let port = var("PORT").unwrap_or(fallback_port);

        format!("0.0.0.0:{port}")
    };

    // NOTE: Prefer unwrapping for stack trace at start-up. Will fail on
    // assigning to the specified port.
    let listener = TcpListener::bind(&host).await.unwrap();
    let state = AppState::new().await;

    let middleware = from_fn_with_state(state.clone(), handler::middleware);
    let tracing = {
        let req_span = DefaultMakeSpan::new().level(log_level);
        let res_span = DefaultOnResponse::new().level(log_level);

        TraceLayer::new_for_http()
            .make_span_with(req_span)
            .on_response(res_span)
    };

    let mut router = Router::new()
        .route("/", get(index))
        .route("/idempotency", post(idempotency::core))
        .route("/rate-limiter", get(ratelimiter::core));

    #[cfg(debug_assertions)]
    {
        let internal_routes = {
            let routes = Router::new().nest("/idempotency", idempotency::internal::routes());
            Router::new().nest("/internal", routes)
        };

        router = router.merge(internal_routes);
    }

    let router = router
        .layer(middleware)
        .layer(tracing)
        .with_state(state)
        // Allows for reading the request IP through an extension.
        .into_make_service_with_connect_info::<SocketAddr>();

    info!("starting: {host}");
    (listener, router)
}

#[tokio::main]
async fn main() {
    let log_level = start_logger();
    let (listener, router) = setup_service(log_level).await;

    // NOTE: This can be unwrapped as the error handling is to sleep & retry.
    //
    // "Errors on the TCP socket will be handled by sleeping for a short while
    // (currently, one second)" - axum::serve
    serve(listener, router).await.unwrap();
}
