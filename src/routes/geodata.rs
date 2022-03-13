use crate::context::Context;
use crate::errors::Error;
use crate::lib::models::ModelExt;
use crate::lib::token::TokenAccount;
// TODO: import this from geodata-anchor
use crate::lib::msg;
use crate::models::geodata;
use crate::models::geodata::{Geodata, HashableGeodata, Location, PublicGeodata};
use crate::models::validation::{Validation, Validity};
use axum::{
  extract::{Extension, Query},
  routing::{get, post},
  Json, Router,
};
use bson::doc;
use cosmrs::{
  cosmwasm::MsgExecuteContract,
  crypto::secp256k1,
  tx::{self, AccountNumber, Fee, Msg, SignDoc, SignerInfo},
  AccountId, Coin
};
use cosmwasm_std::Timestamp;
use serde::{Deserialize, Serialize};
use tracing::debug;
use wither::mongodb::options::FindOptions;
const DENOM: &str = "ujunox";

#[derive(Serialize, Deserialize, Debug)]
struct NearQueryParams {
  lon: f32,
  lat: f32,
  min: i32,
  max: i32,
}

use crate::lib::token::ADMIN_PATH;
use crate::lib::token::USER_PATH;

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
  let oid = &geodata.id.unwrap();
  debug!("geodata.id (oid): {:?}", oid);
  let geodata_id = &geodata.id.unwrap().to_hex();
  debug!("geodata.id (str): {:?}", geodata_id);

  let hashable = HashableGeodata::from(geodata.clone());
  let j_hashable = serde_json::to_string(&hashable).unwrap();
  debug!("j_hashable: {:?}", &j_hashable);
  let hash = geodata::hash_data(j_hashable).await?;
  debug!("hash: {}", &hash);

  let amount = Coin {
    amount: 100u8.into(),
    denom: DENOM.parse().unwrap(),
  };
 
  let admin_account_id = context.settings.contract.admin
    .parse::<AccountId>().unwrap();
  let contract_account_id = context.settings.contract.address
    .parse::<AccountId>().unwrap();
  let nanos: u64 = geodata.created.to_chrono().timestamp_nanos() as u64;
  let created = Timestamp::from_nanos(nanos); 

  let create_msg = msg::CreateMsg {
    id: geodata.id.unwrap().to_hex(),
    account: geodata.account.to_hex(),
    hash: hash.clone(),
    created: created,
  };

  // Msg json encoded message to be passed to the contract
  let create_msg_json = serde_json::to_string(&create_msg).unwrap();
  // let create_b64: serde_json::Value = serde_json::from_str(&create_msg_json).unwrap();
  // let mut msg: Vec<u8> = Vec::new();
  let msg : Vec<u8> = serde_json::to_vec(&create_msg_json).unwrap();
  let msg_execute = MsgExecuteContract {
    sender: admin_account_id,
    contract: contract_account_id,
    msg: msg,
    funds: vec![amount.clone()],
  }
  .to_any()
  .unwrap();

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
