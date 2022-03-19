use bson::serde_helpers::bson_datetime_as_rfc3339_string;
use bson::serde_helpers::serialize_object_id_as_hex_string;
use serde::{Deserialize, Serialize};
use tokio::task;
use validator::Validate;
use wither::bson::{doc, oid::ObjectId};
use wither::Model as WitherModel;

use crate::common::date;
use crate::common::date::Date;
use crate::common::models::ModelExt;
use crate::database::Database;
use crate::errors::Error;

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
  type T = Role;
  fn get_database(&self) -> &Database {
    &self.db
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, WitherModel, Validate)]
#[model(
  index(keys = r#"doc!{ "name": 1 }"#, options = r#"doc!{ "unique": true }"#),
  index(keys = r#"doc!{ "path": 1 }"#, options = r#"doc!{ "unique": true }"#),
)]
pub struct Role {
  #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
  pub id: Option<ObjectId>,
  #[validate(length(min = 1))]
  pub name: String,
  pub path: String,
  pub created_at: Date,
}

impl Role {
  pub fn new(name: String, path: String) -> Self {
    let now = date::now();
    Self {
      id: None,
      name,
      path,
      created_at: now,
    }
  }
}
