use bson::doc;
use geodata_rest::common::models::ModelExt;
use geodata_rest::models::account::{self, Account};
use geodata_rest::models::role::Role;
use std::error::Error;
mod common;
use common::*;

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn create_roles_and_accounts() -> Result<(), Box<dyn Error>> {
    let context = get_context().await;
    context.models.role.delete_many(doc! {}).await?;
    assert_eq!(context.models.role.count(doc! {}).await?, 0);

    let role_user = Role::new("user".to_string());
    let role_user = context.models.role.create(role_user).await?;
    let role_admin = Role::new("admin".to_string());
    let role_admin = context.models.role.create(role_admin).await?;
    let role_validator = Role::new("validator".to_string());
    let role_validator = context.models.role.create(role_validator).await?;
    assert_eq!(context.models.role.count(doc! {}).await?, 3);

    context.models.account.delete_many(doc! {}).await?;
    assert_eq!(context.models.account.count(doc! {}).await?, 0);

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


}
