#![allow(dead_code)]
use bson::serde_helpers::bson_datetime_as_rfc3339_string;
use bson::serde_helpers::serialize_object_id_as_hex_string;
use serde::{Deserialize, Serialize};
use tokio::task;
use validator::Validate;
use wither::bson::{doc, oid::ObjectId};
use wither::Model as WitherModel;

use crate::database::Database;
use crate::errors::Error;
use crate::common::date;
use crate::common::date::Date;
use crate::common::models::ModelExt;
use crate::models::role::Role;

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
  type T = Account;
  fn get_database(&self) -> &Database {
    &self.db
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, WitherModel, Validate)]
#[model(index(keys = r#"doc!{ "email": 1 }"#, options = r#"doc!{ "unique": true }"#))]
pub struct Account {
  #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
  pub id: Option<ObjectId>,
  #[validate(length(min = 1))]
  pub name: String,
  #[validate(email)]
  pub email: String,
  pub password: String,
  pub roles: Vec<Role>,
  pub updated_at: Date,
  pub created_at: Date,
  pub locked_at: Option<Date>,
}

impl Account {
  pub fn new(name: String, email: String, password_hash: String, roles: Vec<Role>) -> Self {
    let now = date::now();
    Self {
      id: None,
      name,
      email,
      password: password_hash,
      roles,
      updated_at: now,
      created_at: now,
      locked_at: None,
    }
  }

  pub fn is_password_match(&self, password: &str) -> bool {
    bcrypt::verify(password, self.password.as_ref()).unwrap_or(false)
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublicAccount {
  #[serde(alias = "_id", serialize_with = "serialize_object_id_as_hex_string")]
  pub id: ObjectId,
  pub name: String,
  pub email: String,
  pub roles: Vec<Role>,
  #[serde(with = "bson_datetime_as_rfc3339_string")]
  pub updated_at: Date,
  #[serde(with = "bson_datetime_as_rfc3339_string")]
  pub created_at: Date,
}

impl From<Account> for PublicAccount {
  fn from(account: Account) -> Self {
    Self {
      id: account.id.unwrap(),
      name: account.name.clone(),
      email: account.email.clone(),
      roles: account.roles,
      updated_at: account.updated_at,
      created_at: account.created_at,
    }
  }
}

pub async fn hash_password<P>(password: P) -> Result<String, Error>
where
  P: AsRef<str> + Send + 'static,
{
  task::spawn_blocking(move || bcrypt::hash(password.as_ref(), bcrypt::DEFAULT_COST))
    .await
    .map_err(Error::RunSyncTask)?
    .map_err(Error::HashPassword)
}
