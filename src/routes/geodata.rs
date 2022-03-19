use crate::context::Context;
use crate::errors::Error;
use crate::common::models::ModelExt;
use crate::common::token::TokenAccount;
// TODO: import this from geodata-anchor
use crate::common::msg;
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
  tx::{self, Fee, Msg, SignDoc, SignerInfo},
  AccountId, Coin, Any
};
use cosmwasm_std::Timestamp;
use serde::{Deserialize, Serialize};
use tracing::debug;

use wither::mongodb::options::FindOptions;
const DENOM: &str = "ujunox";
const RPC_PORT: u16 = 26657;

#[derive(Serialize, Deserialize, Debug)]
struct NearQueryParams {
  lon: f32,
  lat: f32,
  min: i32,
  max: i32,
}

use crate::common::token::ADMIN_PATH;
use crate::common::token::USER_PATH;

const CHAIN_ID: &str = "testing";

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
  // TODO: call create_msg and send_msg

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

fn create_msg(
  hash: &str,
  geodata: Geodata,
  context: Context,
) -> Result<Any, Error> {
  let amount = Coin {
    amount: 100u8.into(),
    denom: DENOM.parse().unwrap(),
  };

  let admin_account_id = context
    .settings
    .contract
    .admin
    .parse::<AccountId>()
    .unwrap();
  let contract_account_id = context
    .settings
    .contract
    .address
    .parse::<AccountId>()
    .unwrap();
  let nanos: u64 = geodata.created.to_chrono().timestamp_nanos() as u64;
  let created = Timestamp::from_nanos(nanos);

  let create_msg = msg::CreateMsg {
    id: geodata.id.unwrap().to_hex(),
    account: geodata.account.to_hex(),
    hash: hash.to_string(),
    created: created,
  };
  // Msg json encoded message to be passed to the contract
  let create_msg_json = serde_json::to_string(&create_msg).unwrap();
  // let create_b64: serde_json::Value = serde_json::from_str(&create_msg_json).unwrap();
  // let mut msg: Vec<u8> = Vec::new();
  let msg: Vec<u8> = serde_json::to_vec(&create_msg_json).unwrap();
  let msg_execute = MsgExecuteContract {
    sender: admin_account_id,
    contract: contract_account_id,
    msg: msg,
    funds: vec![amount.clone()],
  }
  .to_any()
  .unwrap();

  return Ok(msg_execute);
}

// fn send_msg(
//   msg: MsgExecuteContract,
//   geodata: Geodata,
//   context: Context,
// ) -> Result<Any, Error> {
//   let sender_private_key = secp256k1::SigningKey::random();
//   let sender_public_key = sender_private_key.public_key();
//   let sequence_number = 0;
//   let gas = 100_000;
//   let fee = Fee::from_amount_and_gas(amount, gas);
//   let timeout_height = 9001u16;
//   let tx_body = tx::Body::new(vec![msg], "test memo", timeout_height);
//   let chain_id = CHAIN_ID.parse().unwrap();
//   let auth_info =
//     SignerInfo::single_direct(Some(sender_public_key), sequence_number).auth_info(fee);
//   let sign_doc = SignDoc::new(&tx_body, &auth_info, &chain_id, 1).unwrap();
//   let tx_raw = sign_doc.sign(&sender_private_key).unwrap();
//   // etc.
// }


#[derive(Serialize, Deserialize, Debug)]
struct CreateGeodata {
  location: Location,
  geotype: String,
  value: f64,
  source: String,
  quality: i32,
}
