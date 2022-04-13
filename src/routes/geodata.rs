use crate::common::anchor;
use crate::common::models::ModelExt;
use crate::common::token::TokenAccount;
use crate::context::Context;
use crate::errors::Error;
use crate::models::geodata;
use crate::models::geodata::{Geodata, HashableGeodata, Location, PublicGeodata};
use crate::models::validation::{Validation, Validity};
use axum::{
  extract::{Extension, Query},
  routing::{get, post},
  Json, Router,
};
use bson::doc;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::common::token::{ADMIN_PATH, USER_PATH};
use wither::mongodb::options::FindOptions;

#[derive(Serialize, Deserialize, Debug)]
struct NearQueryParams {
  lon: f32,
  lat: f32,
  min: i32,
  max: i32,
}

pub fn create_route() -> Router {
  let create_geodata_path = format!("{}{}", ADMIN_PATH, "/geodata");
  let get_geodata_near_path = format!("{}{}", USER_PATH, "/geodata/near");
  let query_geodata_path = format!("{}{}", USER_PATH, "/geodata");
  Router::new()
    .route(&create_geodata_path, post(create_geodata))
    .route(&query_geodata_path, get(query_geodata))
    .route(&get_geodata_near_path, get(get_geodata_near))
}

async fn create_geodata(
  account: TokenAccount,
  Extension(context): Extension<Context>,
  Json(body): Json<CreateGeodata>,
) -> Result<Json<PublicGeodata>, Error> {
  // create geodata doc
  let geodata = Geodata::new(
    account.id,
    body.location,
    body.geotype,
    body.value,
    body.source,
    body.quality,
  );

  let geodata = context.models.geodata.create(geodata).await?;
  let geodata_id = &geodata.id.unwrap().to_hex();

  let hashable = HashableGeodata::from(geodata.clone());
  let j_hashable = serde_json::to_string(&hashable).unwrap();
  let hash = geodata::hash_data(j_hashable).await?;

  let anchor_hash = hash.clone();
  let account_id = account.id.to_hex();
  let nanos: u64 = geodata.created.to_chrono().timestamp_nanos() as u64;

  // anchor
  let result = anchor::anchor_geodata(
    geodata_id,
    &account_id,
    &anchor_hash,
    nanos,
  )
  .await?;

  // create top level Validation doc for this geodata, and supply initial validity
  let validity = Validity::new(account.id, hash);
  let validation = Validation::new(account.id, geodata.id.unwrap(), vec![validity]);
  context.models.validation.create(validation).await?;
  let res = PublicGeodata::from(geodata);
  Ok(Json(res))
}

async fn query_geodata(
  _account: TokenAccount,
  Extension(context): Extension<Context>,
) -> Result<Json<Vec<PublicGeodata>>, Error> {
  let limit = FindOptions::builder().limit(10).build();
  let geodata = context
    .models
    .geodata
    .find(doc! {}, limit)
    .await?
    .into_iter()
    .map(Into::into)
    .collect::<Vec<PublicGeodata>>();

  debug!("Returning geodata");
  Ok(Json(geodata))
}

async fn get_geodata_near(
  _account: TokenAccount,
  Extension(context): Extension<Context>,
  params: Query<NearQueryParams>,
) -> Result<Json<Vec<PublicGeodata>>, Error> {
  debug!("params: {:?}", &params);
  let geodata = context
    .models
    .geodata
    .find(
      doc! { "location": {
            "$near": {
               "$geometry": { "type": "Point", "coordinates": [
                          params.lon,
                          params.lat
                      ]
                }, "$minDistance": params.min, "$maxDistance": params.max
            }
          }
      },
      None,
    )
    .await?
    .into_iter()
    .map(Into::into)
    .collect::<Vec<PublicGeodata>>();

  debug!("Returning geodata");
  Ok(Json(geodata))
}

#[derive(Serialize, Deserialize, Debug)]
struct CreateGeodata {
  location: Location,
  geotype: String,
  value: f64,
  source: String,
  quality: i32,
}
