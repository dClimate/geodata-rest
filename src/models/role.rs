use serde::{Deserialize, Serialize};
use validator::Validate;
use wither::bson::{doc, oid::ObjectId};
use wither::Model as WitherModel;

use crate::common::date;
use crate::common::date::Date;
use crate::common::models::ModelExt;
use crate::database::Database;

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
)]
pub struct Role {
  #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
  pub id: Option<ObjectId>,
  #[validate(length(min = 1))]
  pub name: String,
  pub created_at: Date,
}

#[allow (dead_code)]
impl Role {
  pub fn new(name: String) -> Self {
    let now = date::now();
    Self {
      id: None,
      name,
      created_at: now,
    }
  }
}
