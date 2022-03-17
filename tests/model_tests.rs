extern crate geodata_rest;
use geodata_rest::database::Database;
use geodata_rest::models::Models;
use geodata_rest::settings::Settings;
use geodata_rest::models::account::{Account};
use geodata_rest::context::Context;
use geodata_rest::common::models::ModelExt;
use std::env;
use std::error::Error;
use bson::doc;

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn models_and_context_load() -> Result<(), Box<dyn Error>> {
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

    let context = Context::new(models, settings.clone());

    context.models.account.delete_many(doc! {}).await?;
    assert_eq!(context.models.account.count(doc! {}).await?, 0);
    let account = Account::new(String::from("test"), String::from("test@test.com"), String::from("test"), vec!["user".to_string()]);
    let account = context.models.account.create(account).await?;

    assert_eq!(context.models.account.count(doc! {}).await?, 1);
    assert_eq!(account.name, "test");
    
    Ok(())
  }
}
