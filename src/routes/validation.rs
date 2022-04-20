use crate::context::Context;
use crate::errors::Error;
use crate::common::token::TokenAccount;
use crate::models::geodata;
use crate::models::geodata::{HashableGeodata};
use crate::models::validation::{ValidationResult, ValidationResults, Validation, Validity};
use crate::common::models::ModelExt;
use crate::common::anchor;
use axum::{
  extract::{Extension},
  routing::get,
  Json, Router,
};
use bson::doc;
use wither::mongodb::options::FindOptions;

use crate::common::token::VALIDATOR_PATH;

pub fn create_route() -> Router {
  let validation_path = format!("{}{}", VALIDATOR_PATH, "/validation");
  Router::new().route(&validation_path, get(query_validation))
}
// TODO: move the functionality of this endpoint to an independent daemon process
async fn query_validation(
  account: TokenAccount,
  Extension(context): Extension<Context>,
) -> Result<Json<ValidationResults>, Error> {
  let mut v_results =  ValidationResults::new(vec![]);
  let limit = FindOptions::builder().limit(10).build();
  let mut validations = context
    .models
    .validation
    .find(doc! {}, limit)
    .await?
    .into_iter()
    .map(Into::into)
    .collect::<Vec<Validation>>();
  for validation in &mut validations {
    let geodata = context
      .models
      .geodata
      .find_one(doc! { "_id": validation.geodata}, None)
      .await?
      .map(HashableGeodata::from);
    let j_hashable = serde_json::to_string(&geodata).unwrap();
    let hash = geodata::hash_data(j_hashable).await?;

    // validity check compares current hash result with original when created (validation.validities[0].hash)
    // debug!("this hash: {} original hash: {:?}", &hash, &validation.validities[0].hash);
    let validity = Validity::new(account.id, hash.clone());
    validation.validities.push(validity);

    let v_result = ValidationResult::new(account.id, validation.geodata, hash == validation.validities[0].hash);
    let succeeded = v_result.validated;
    v_results.results.push(v_result);
    let v_doc = bson::to_document(&validation).unwrap();
    let validation = context
    .models
    .validation
    .find_one_and_update(
      doc! { "_id": &validation.id},
      doc! { "$set": v_doc },
    )
    .await?
    .map(Validation::from);
    if succeeded {
      let nanos: u64 = geodata.unwrap().created.to_chrono().timestamp_nanos() as u64;
      anchor::validate_geodata(
        &validation.unwrap().geodata.to_hex(),
        &account.id.to_hex(),
        &hash,
        nanos,
      )
      .await?;
    }   
  }

  Ok(Json(v_results))
}
