use geodata_rest::context::Context;
use geodata_rest::database::Database;
use geodata_rest::models::Models;
use geodata_rest::settings::Settings;
use std::env;

pub async fn get_context() -> Context {
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
