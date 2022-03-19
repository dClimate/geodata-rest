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
  async fn create_roles() -> Result<(), Box<dyn Error>> {
    let context = get_context().await;
    context.models.role.delete_many(doc! {}).await?;
    assert_eq!(context.models.role.count(doc! {}).await?, 0);
    let role = Role::new("user".to_string(), "/6b0866".to_string());
    context.models.role.create(role).await?;
    let role = Role::new("admin".to_string(), "/6a2dda".to_string());
    context.models.role.create(role).await?;
    let role = Role::new("validator".to_string(), "/5be0da".to_string());
    context.models.role.create(role).await?;
    assert_eq!(context.models.role.count(doc! {}).await?, 3);
    Ok(())
  }


}
