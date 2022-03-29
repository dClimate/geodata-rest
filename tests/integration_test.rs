//! tests workflow:
//! authorization: admin, user, validator
//! routes for above based on role permissions
use bson::oid::ObjectId;
use axum::{extract::Extension, Router};
use geodata_rest::common::token::ADMIN_PATH;
use geodata_rest::context::Context;
use geodata_rest::models::account::PublicAccount;
use geodata_rest::models::geodata::{Geometry, Location, PublicGeodata};
use geodata_rest::routes;
use geodata_rest::logger::Logger;
use http::header;
use serde::{Deserialize, Serialize};
use tower_http::{
  compression::CompressionLayer, propagate_header::PropagateHeaderLayer,
  sensitive_headers::SetSensitiveHeadersLayer, trace,
};

mod common;
use common::*;
use tracing::debug;

#[allow(dead_code)]
fn app(context: Context) -> Router {
  Logger::setup(&context.settings);
  Router::new()
    .merge(routes::account::create_route())
    .merge(routes::geodata::create_route())
    .merge(routes::validation::create_route())
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
    .layer(Extension(context))
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

  #[tokio::test]
  async fn test_workflow() {
    let context = get_testdb_context().await;
    initialize_testdb(&context).await.unwrap();
    let listener = TcpListener::bind("0.0.0.0:0".parse::<SocketAddr>().unwrap()).unwrap();
    let addr = listener.local_addr().unwrap();
    // let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 8080));

    tokio::spawn(async move {
      axum::Server::from_tcp(listener)
        .unwrap()
        .serve(app(context).into_make_service())
        .await
        .unwrap();
    });

    let client = hyper::Client::new();
    let body = AuthorizeBody {
      email: "admin@test.com".to_string(),
      password: "test".to_string(),
    };

    let response = client
      .request(
        Request::builder()
          .method(http::Method::POST)
          .uri(format!("http://{}/accounts/authenticate", addr))
          .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
          .body(Body::from(serde_json::to_vec(&json!(body)).unwrap()))
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body: Value = serde_json::from_slice(&body).unwrap();
    let res: AuthenticateResponse = serde_json::from_value(body).unwrap();

    assert_eq!(res.account.name, "admin".to_string());

    //TODO:
    // Provide test with invalid pw
    // call post /geodata without token
    let admin_token = res.access_token;
    let admin_id = res.account.id;

    // build post /geodata request body
    let geometry = Geometry {
      r#type: "Point".to_string(),
      coordinates: vec![-73.91320, 40.68405],
    };

    let location = Location {
      r#type: "GeometryCollection".to_string(),
      geometries: vec![geometry],
    };

    let body = CreateGeodata {
      account: res.account.id,
      location,
      geotype: "Wind".to_string(),
      value: 11.1,
      source: "Google Earth Engine".to_string(),
      quality: 5,
    };

    let admin_auth_bearer = format!("Bearer {}", admin_token);

    let response = client
      .request(
        Request::builder()
          .method(http::Method::POST)
          .uri(format!("http://{}{}/geodata", addr, ADMIN_PATH))
          .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
          .header(http::header::AUTHORIZATION, admin_auth_bearer)
          .body(Body::from(serde_json::to_vec(&json!(body)).unwrap()))
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body: Value = serde_json::from_slice(&body).unwrap();
    let res: PublicGeodata = serde_json::from_value(body).unwrap();
    assert_eq!(res.account, admin_id);
    //TODO:
    // Add auth for user and validator accounts
    // call gets for user
    // call get for validator
  }
}
#[derive(Debug, Serialize)]
struct AuthorizeBody {
  email: String,
  password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AuthenticateResponse {
  access_token: String,
  account: PublicAccount,
}

#[derive(Serialize, Deserialize, Debug)]
struct CreateGeodata {
  account: ObjectId,
  location: Location,
  geotype: String,
  value: f64,
  source: String,
  quality: i32,
}
