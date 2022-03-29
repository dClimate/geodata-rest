//! provides context for localhost/test and an initialization function for integration test
//! roles: admin, user, validator
//! accounts: admin (role admin, user), user (role user), validator (role validator)
use bson::doc;
use geodata_rest::common::models::ModelExt;
use geodata_rest::models::{Models, account::{self, Account}, role::Role};
use geodata_rest::settings::Settings;
use geodata_rest::context::Context;
use geodata_rest::database::Database;
use std::error::Error;

use std::env;

#[allow(dead_code)]
pub async fn get_testdb_context() -> Context {
    env::set_var("RUN_MODE", "test");
    let settings = match Settings::new() {
      Ok(value) => value,
      Err(err) => panic!("Failed to setup configuration. Error: {}", err),
    };

    assert_eq!(settings.database.name, "test");

    let db = match Database::setup(&settings).await {
      Ok(value) => value,
      Err(_) => panic!("Failed to setup database connection"),
    };

    let models = match Models::setup(db.clone()).await {
      Ok(value) => value,
      Err(err) => panic!("Failed to setup models {}", err),
    };

    Context::new(models, settings.clone())
}

#[allow(dead_code)]
pub async fn initialize_testdb(context: &Context) -> Result<(), Box<dyn Error>> {
  // clear db
  context.models.geodata.delete_many(doc! {}).await?;
  assert_eq!(context.models.geodata.count(doc! {}).await?, 0);

  context.models.validation.delete_many(doc! {}).await?;
  assert_eq!(context.models.validation.count(doc! {}).await?, 0);

  context.models.role.delete_many(doc! {}).await?;
  assert_eq!(context.models.role.count(doc! {}).await?, 0);

  context.models.account.delete_many(doc! {}).await?;
  assert_eq!(context.models.account.count(doc! {}).await?, 0);

  // create roles
  let role_user = Role::new("user".to_string());
  let role_user = context.models.role.create(role_user).await?;
  let role_admin = Role::new("admin".to_string());
  let role_admin = context.models.role.create(role_admin).await?;
  let role_validator = Role::new("validator".to_string());
  let role_validator = context.models.role.create(role_validator).await?;
  assert_eq!(context.models.role.count(doc! {}).await?, 3);

  // create users
  let password_hash = account::hash_password("test").await?;
  let admin = Account::new(
    "admin".to_string(),
    "admin@test.com".to_string(),
    password_hash.clone(),
    vec![role_user.clone(), role_admin],
  );
  context.models.account.create(admin).await?;

  let user = Account::new(
    "user".to_string(),
    "user@test.com".to_string(),
    password_hash.clone(),
    vec![role_user.clone()],
  );
  context.models.account.create(user).await?;

  let validator = Account::new(
    "validator".to_string(),
    "validator@test.com".to_string(),
    password_hash.clone(),
    vec![role_validator],
  );
  context.models.account.create(validator).await?;

  assert_eq!(context.models.account.count(doc! {}).await?, 3);
  Ok(())
}