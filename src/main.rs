use axum::extract::Extension;
use axum::{handler::Handler, http::StatusCode, response::IntoResponse, Router};
use axum::http::header;
use std::net::SocketAddr;
use tower_http::{
  compression::CompressionLayer, propagate_header::PropagateHeaderLayer,
  sensitive_headers::SetSensitiveHeadersLayer, trace,
};
use tracing::{info, debug};

mod context;
mod database;
mod errors;
mod common;
mod logger;
mod models;
mod routes;
mod settings;

use context::Context;
use database::Database;
use logger::Logger;
use models::Models;
use settings::Settings;

#[tokio::main]
async fn main() {
  let settings = match Settings::new() {
    Ok(value) => value,
    Err(err) => panic!("Failed to setup configuration. Error: {}", err),
  };

  Logger::setup(&settings);

  let db = match Database::setup(&settings).await {
    Ok(value) => value,
    Err(_) => panic!("Failed to setup database connection"),
  };

  let models = match Models::setup(db.clone()).await {
    Ok(value) => value,
    Err(err) => panic!("Failed to setup models {}", err),
  };

  let context = Context::new(models, settings.clone());

  let app = Router::new()
    .merge(routes::account::create_route())
    .merge(routes::geodata::create_route())
    .merge(routes::validation::create_route())
    .fallback(handler_404.into_service())
    // High level logging of requests and responses
    .layer(
      trace::TraceLayer::new_for_http()
        .make_span_with(trace::DefaultMakeSpan::new().include_headers(true))
        .on_request(trace::DefaultOnRequest::new().level(tracing::Level::INFO))
        .on_response(trace::DefaultOnResponse::new().level(tracing::Level::INFO)),
    )
    // Mark the `Authorization` request header as sensitive so it doesn't
    // show in logs.
    .layer(SetSensitiveHeadersLayer::new(std::iter::once(
      header::AUTHORIZATION,
    )))
    // Compress responses
    .layer(CompressionLayer::new())
    // Propagate `X-Request-Id`s from requests to responses
    .layer(PropagateHeaderLayer::new(header::HeaderName::from_static(
      "x-request-id",
    )))
    .layer(Extension(context));

  let port = settings.server.port;
  let address = SocketAddr::from(([127, 0, 0, 1], port));
  debug!("contract address: {}", settings.contract.address);
  debug!("admin address: {}", settings.contract.admin);
  info!("listening on {}", &address);

  axum::Server::bind(&address)
    .serve(app.into_make_service())
    .await
    .expect("Failed to start server");
}

async fn handler_404() -> impl IntoResponse {
  (StatusCode::NOT_FOUND, "Not found")
}
