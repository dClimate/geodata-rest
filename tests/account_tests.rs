use bson::doc;
use geodata_rest::common::models::ModelExt;
use geodata_rest::models::account::Account;
use geodata_rest::models::role::Role;
use std::error::Error;
mod common;
use common::*;

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn add_account() -> Result<(), Box<dyn Error>> {
    let context = get_context().await;
    context.models.account.delete_many(doc! {}).await?;
    assert_eq!(context.models.account.count(doc! {}).await?, 0);
    let account = Account::new(
      "test".to_string(),
      "test@test.com".to_string(),
      "test".to_string(),
      vec!["user".to_string()],
    );
    let account = context.models.account.create(account).await?;

    assert_eq!(context.models.account.count(doc! {}).await?, 1);
    assert_eq!(account.name, "test");

    Ok(())
  }
}
