use axum::extract::Extension;
use axum::{handler::Handler, http::StatusCode, response::IntoResponse, Router};
use geodata_rest::models::account::{self, Account};
use geodata_rest::routes;
use http::header;
use std::error::Error;
use std::net::SocketAddr;
use tower_http::{compression::CompressionLayer, propagate_header::PropagateHeaderLayer};
mod common;
use common::*;

#[tokio::main]
async fn main() {
  let context = get_context().await;
  let app = Router::new()
    .merge(routes::account::create_route())
    .merge(routes::geodata::create_route())
    .merge(routes::validation::create_route())
    .fallback(handler_404.into_service())
    // Compress responses
    .layer(CompressionLayer::new())
    // Propagate `X-Request-Id`s from requests to responses
    .layer(PropagateHeaderLayer::new(header::HeaderName::from_static(
      "x-request-id",
    )))
    .layer(Extension(context));

  let port = 8080;
  let address = SocketAddr::from(([127, 0, 0, 1], port));

  axum::Server::bind(&address)
    .serve(app.into_make_service())
    .await
    .expect("Failed to start server");
}

async fn handler_404() -> impl IntoResponse {
  (StatusCode::NOT_FOUND, "Not found")
}

#[cfg(test)]
mod tests {
  use super::*;
  use axum::{
    body::Body,
    http::{self, Request, StatusCode},
  };
  use serde_json::{json, Value};
  use std::net::{SocketAddr, TcpListener};
  use tower::ServiceExt; // for `app.oneshot()`

  #[tokio::test]
  async fn test_auth() {
    let listener = TcpListener::bind("0.0.0.0:0".parse::<SocketAddr>().unwrap()).unwrap();
    let addr = listener.local_addr().unwrap();

    let client = hyper::Client::new();

    let response = client
      .request(
        Request::builder()
          .uri(format!("http://{}", addr))
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    assert_eq!(&body[..], b"Hello, World!");
  }
}
