use bson::serde_helpers::bson_datetime_as_rfc3339_string;
use bson::serde_helpers::serialize_object_id_as_hex_string;
use serde::{Deserialize, Serialize};
use validator::Validate;
use wither::bson::{doc, oid::ObjectId};
use wither::Model as WitherModel;

use crate::database::Database;
use crate::common::date;
use crate::common::date::Date;
use crate::common::models::ModelExt;

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
  type T = Validation;
  fn get_database(&self) -> &Database {
    &self.db
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validity {
  pub account: ObjectId,
  pub hash: String,
  pub created: Date,
}

impl Validity {
  pub fn new(
    account: ObjectId,
    hash: String,
  ) -> Self {
    Self {
      account,
      hash,
      created: date::now(),
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, WitherModel, Validate)]
#[model(index(keys = r#"doc!{ "account": 1 }"#))]
pub struct Validation {
  #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
  pub id: Option<ObjectId>,
  #[serde(serialize_with = "serialize_object_id_as_hex_string")]
  pub account: ObjectId,
  #[serde(serialize_with = "serialize_object_id_as_hex_string")]
  pub geodata: ObjectId,
  pub validities: Vec<Validity>,
  pub created: Date,
}

impl Validation {
  pub fn new(
    account: ObjectId,
    geodata: ObjectId,
    validities: Vec<Validity>,
  ) -> Self {
    Self {
      id: None,
      account,
      geodata,
      validities,
      created: date::now(),
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationResult {
  #[serde(serialize_with = "serialize_object_id_as_hex_string")]
  pub account: ObjectId,
  #[serde(serialize_with = "serialize_object_id_as_hex_string")]
  pub geodata: ObjectId,
  pub validated: bool,
  #[serde(with = "bson_datetime_as_rfc3339_string")]
  pub created: Date,
}
impl ValidationResult {
  pub fn new(
    account: ObjectId,
    geodata: ObjectId,
    validated: bool,
  ) -> Self {
    Self {
      account,
      geodata,
      validated,
      created: date::now(),
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationResults {
  pub results: Vec<ValidationResult>,
  #[serde(with = "bson_datetime_as_rfc3339_string")]
  pub created: Date,
}

impl ValidationResults {
  pub fn new(
    results: Vec<ValidationResult>,
  ) -> Self {
    Self {
      results,
      created: date::now(),
    }
  }
}