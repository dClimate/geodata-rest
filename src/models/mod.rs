pub mod account;
pub mod geodata;
pub mod validation;
use crate::lib::models::ModelExt;
use crate::Database;
use crate::Error;

#[derive(Clone)]
pub struct Models {
  pub account: account::Model,
  pub geodata: geodata::Model,
  pub validation: validation::Model,
}

impl Models {
  pub async fn setup(db: Database) -> Result<Self, Error> {
    let account = account::Model::new(db.clone());
    let geodata = geodata::Model::new(db.clone());
    let validation = validation::Model::new(db.clone());
    let this = Self { account, geodata, validation };

    this.sync_indexes().await?;
    Ok(this)
  }

  pub async fn sync_indexes(&self) -> Result<(), Error> {
    self.account.sync_indexes().await?;
    self.geodata.sync_indexes().await?;
    self.validation.sync_indexes().await?;

    Ok(())
  }
}