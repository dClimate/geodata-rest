use bson::serde_helpers::bson_datetime_as_rfc3339_string;
use bson::serde_helpers::serialize_object_id_as_hex_string;
use serde::{Deserialize, Serialize};
use validator::Validate;
use tokio::task;
use wither::bson::{doc, oid::ObjectId};
use wither::Model as WitherModel;

use crate::database::Database;
use crate::lib::date;
use crate::lib::hasher;
use crate::errors::Error;
use crate::lib::date::Date;
use crate::lib::models::ModelExt;

#[derive(Clone)]
pub struct Model {
  pub db: Database,
}

impl Model {
  pub fn new(db: Database) -> Self {
    Self { db }
  }
}

impl ModelExt for Model {
  type T = Geodata;
  fn get_database(&self) -> &Database {
    &self.db
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Geometry {
  pub r#type: String,
  pub coordinates: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
  pub r#type: String,
  pub geometries: Vec<Geometry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, WitherModel, Validate)]
#[model(index(keys = r#"doc!{ "account": 1 }"#))]
pub struct Geodata {
  #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
  pub id: Option<ObjectId>,
  pub account: ObjectId,
  pub location: Location,
  pub geotype: String,
  pub value: f64,
  pub source: String,
  pub quality: i32,
  pub created: Date,
}

impl Geodata {
  pub fn new(
    account: ObjectId,
    location: Location,
    geotype: String,
    value: f64,
    source: String,
    quality: i32,
  ) -> Self {
    Self {
      id: None,
      account,
      location,
      geotype,
      value,
      source,
      quality,
      created: date::now(),
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublicGeodata {
  #[serde(alias = "_id", serialize_with = "serialize_object_id_as_hex_string")]
  pub id: ObjectId,
  #[serde(serialize_with = "serialize_object_id_as_hex_string")]
  pub account: ObjectId,
  pub location: Location,
  pub geotype: String,
  pub value: f64,
  pub source: String,
  pub quality: i32,
  #[serde(with = "bson_datetime_as_rfc3339_string")]
  pub created: Date,
}

impl From<Geodata> for PublicGeodata {
  fn from(geodata: Geodata) -> Self {
    Self {
      id: geodata.id.unwrap(),
      account: geodata.account,
      location: geodata.location.clone(),
      geotype: geodata.geotype.clone(),
      value: geodata.value.clone(),
      source: geodata.source.clone(),
      quality: geodata.quality.clone(),
      created: geodata.created,
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HashableGeodata {
  pub location: Location,
  pub geotype: String,
  pub value: f64,
  pub source: String,
  pub quality: i32,
  #[serde(with = "bson_datetime_as_rfc3339_string")]
  pub created: Date,
}

impl From<Geodata> for HashableGeodata {
  fn from(geodata: Geodata) -> Self {
    Self {
      location: geodata.location.clone(),
      geotype: geodata.geotype.clone(),
      value: geodata.value.clone(),
      source: geodata.source.clone(),
      quality: geodata.quality.clone(),
      created: geodata.created,
    }
  }
}

pub async fn hash_data<P>(data: P) -> Result<String, Error>
where
  P: AsRef<str> + Send + 'static,
{
  task::spawn_blocking(move || hasher::hash(data.as_ref()))
    .await
    .map_err(Error::RunSyncTask)
}