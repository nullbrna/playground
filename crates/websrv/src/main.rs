use axum::Router;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use crate::handler::HandlerState;

mod handler;

fn setup_environment() {
  let envfilter = EnvFilter::from_default_env();

  if cfg!(debug_assertions) {
    tracing_subscriber::fmt()
      .with_env_filter(envfilter)
      .with_target(false)
      .without_time()
      .compact()
      .init();
  } else {
    tracing_subscriber::fmt().with_env_filter(envfilter).json().init();
  }
}

async fn create_configured_router() -> Router {
  let handlerstate = HandlerState::new().await;
  let tracinglyr = TraceLayer::new_for_http().on_request(()).on_body_chunk(()).on_eos(());

  Router::new()
    .route("/", axum::routing::get(handler::health_check))
    .layer(tracinglyr)
    .with_state(handlerstate)
}

#[tokio::main]
async fn main() {
  setup_environment();

  let approuter = create_configured_router().await;
  let listener = {
    let hostaddr = std::env::var("HOST_ADDR")
      .ok()
      .filter(|val| val.len() > 0)
      .expect("Missing/unset HOST_ADDR variable");

    tracing::info!(addr = hostaddr, "Starting server...");
    TcpListener::bind(hostaddr).await.expect("Binding a TCP listener")
  };

  axum::serve(listener, approuter)
    .await
    .expect("Unexpected error bubbled up to the root");
}
