//! tests workflow:
//! authorization: admin, user, validator
//! routes for above based on role permissions
//! Admin authenticates and creates geodata instance
//! User authenticates and queries all geodata then issues a "near" query
//! Validator authenticates and runs validation
use axum::{extract::Extension, Router};
use bson::{doc, oid::ObjectId};
use geodata_rest::common::models::ModelExt;
use geodata_rest::common::token::{ADMIN_PATH, USER_PATH, VALIDATOR_PATH};
use geodata_rest::context::Context;
use geodata_rest::logger::Logger;
use geodata_rest::models::account::PublicAccount;
use geodata_rest::models::geodata::{Geometry, Location, PublicGeodata};
use geodata_rest::models::validation::ValidationResults;
use geodata_rest::routes;
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
  use wither::mongodb::options::FindOptions;

  #[tokio::test]
  async fn test_workflow() {
    let limit = FindOptions::builder().limit(10).build();

    let context = get_testdb_context().await;
    initialize_testdb(&context).await.unwrap();
    let listener = TcpListener::bind("0.0.0.0:0".parse::<SocketAddr>().unwrap()).unwrap();
    let addr = listener.local_addr().unwrap();

    let validation_model = context.models.validation.clone();

    tokio::spawn(async move {
      axum::Server::from_tcp(listener)
        .unwrap()
        .serve(app(context).into_make_service())
        .await
        .unwrap();
    });

    let client = hyper::Client::new();
    // test: authenticate admin with invalid password
    let mut body = AuthorizeBody {
      email: "admin@test.com".to_string(),
      password: "invalid".to_string(),
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

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // test: authenticate admin with valid password
    body.password = "test".to_string();

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
    let res_body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let res_body: Value = serde_json::from_slice(&res_body).unwrap();
    let res: AuthenticateResponse = serde_json::from_value(res_body).unwrap();

    assert_eq!(res.account.name, "admin".to_string());

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

    // no validations yet
    assert_eq!(validation_model.count(doc! {}).await.unwrap(), 0u64);
    let auth_bearer = format!("Bearer {}", admin_token);
    // test: call post /geodata as admin with valid token
    let response = client
      .request(
        Request::builder()
          .method(http::Method::POST)
          .uri(format!("http://{}{}/geodata", addr, ADMIN_PATH))
          .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
          .header(http::header::AUTHORIZATION, auth_bearer)
          .body(Body::from(serde_json::to_vec(&json!(body)).unwrap()))
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let res_body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let res_body: Value = serde_json::from_slice(&res_body).unwrap();
    let res: PublicGeodata = serde_json::from_value(res_body).unwrap();
    assert_eq!(res.account, admin_id);
    // create geodata also creates initial validation
    // future validations will create hash and compare with original hash
    assert_eq!(validation_model.count(doc! {}).await.unwrap(), 1u64);
    let validations = validation_model.find(doc! {}, limit.clone()).await.unwrap();
    // initial validation/validity by admin
    assert_eq!(validations[0].validities[0].account, admin_id);
    assert_eq!(validations[0].validities.len(), 1);

    // test: call post /geodata without token (UNAUTHORIZED)
    let response = client
      .request(
        Request::builder()
          .method(http::Method::POST)
          .uri(format!("http://{}{}/geodata", addr, ADMIN_PATH))
          .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
          .body(Body::from(serde_json::to_vec(&json!(&body)).unwrap()))
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let res_body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let res_body = String::from_utf8(res_body.to_vec()).unwrap();
    let error: Value = serde_json::from_str(&res_body).unwrap();
    assert_eq!(error["message"], "Invalid authentication credentials");

    // test: authenticate user with valid password
    let body = AuthorizeBody {
      email: "user@test.com".to_string(),
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
    let res_body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let res_body: Value = serde_json::from_slice(&res_body).unwrap();
    let res: AuthenticateResponse = serde_json::from_value(res_body).unwrap();

    assert_eq!(res.account.name, "user".to_string());
    let user_token = res.access_token;

    // test: get geodata for user
    let auth_bearer = format!("Bearer {}", user_token);
    let response = client
      .request(
        Request::builder()
          .uri(format!("http://{}{}/geodata", addr, USER_PATH))
          .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
          .header(http::header::AUTHORIZATION, &auth_bearer)
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let res_body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let res_body: Value = serde_json::from_slice(&res_body).unwrap();
    let res: Vec<PublicGeodata> = serde_json::from_value(res_body).unwrap();
    assert_eq!(res.len(), 1);

    // test: get geodata/near for user
    let response = client
      .request(
        Request::builder()
          .uri(format!(
            "http://{}{}/geodata/near?lon=-73.91320&lat=40.68405&min=0&max=10000",
            addr, USER_PATH
          ))
          .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
          .header(http::header::AUTHORIZATION, &auth_bearer)
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let res_body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let res_body: Value = serde_json::from_slice(&res_body).unwrap();
    let res: Vec<PublicGeodata> = serde_json::from_value(res_body).unwrap();
    assert_eq!(res.len(), 1);

    // test: authenticate validator with valid password
    let body = AuthorizeBody {
      email: "validator@test.com".to_string(),
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
    let res_body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let res_body: Value = serde_json::from_slice(&res_body).unwrap();
    let res: AuthenticateResponse = serde_json::from_value(res_body).unwrap();

    assert_eq!(res.account.name, "validator".to_string());
    let validator_token = res.access_token;
    let validator_id = res.account.id;

    // test: get validation for validator
    let auth_bearer = format!("Bearer {}", validator_token);
    let response = client
      .request(
        Request::builder()
          .uri(format!("http://{}{}/validation", addr, VALIDATOR_PATH))
          .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
          .header(http::header::AUTHORIZATION, &auth_bearer)
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let res_body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let res_body: Value = serde_json::from_slice(&res_body).unwrap();
    // returns results of all validations
    let validations: ValidationResults = serde_json::from_value(res_body).unwrap();
    assert_eq!(validations.results.len(), 1);

    // validation succeeded but hash not shown
    assert_eq!(validations.results[0].validated, true);
    assert_eq!(validation_model.count(doc! {}).await.unwrap(), 1u64);
    
    // one validation instance for each geodata instance, with multiple validities
    let validations = validation_model.find(doc! {}, limit).await.unwrap();

    // this validation process created another validity for
    // the geodata instance within same validation instance
    assert_eq!(validations[0].validities.len(), 2);

    // original validity by admin
    assert_eq!(validations[0].validities[0].account, admin_id);

    // this validity by validator
    assert_eq!(validations[0].validities[1].account, validator_id);

    // hash from this validation matches hash from original validation
    // i.e. validation suceeded
    assert_eq!(
      validations[0].validities[0].hash,
      validations[0].validities[1].hash
    );
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
