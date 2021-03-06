use axum::{extract::Extension, routing::post, Json, Router};
use bson::doc;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::context::Context;
use crate::errors::BadRequest;
use crate::errors::NotFound;
use crate::errors::{AuthenticateError, Error};
use crate::common::token;
use crate::models::account::PublicAccount;
use crate::common::models::ModelExt;

pub fn create_route() -> Router {
  Router::new()
    .route("/accounts/authenticate", post(authenticate_account))
}

async fn authenticate_account(
  Extension(context): Extension<Context>,
  Json(body): Json<AuthorizeBody>,
) -> Result<Json<AuthenticateResponse>, Error> {
  let email = &body.email;
  let password = &body.password;

  if email.is_empty() {
    debug!("Missing email, returning 400 status code");
    return Err(Error::BadRequest(BadRequest::new(
      "email".to_owned(),
      "Missing email attribute".to_owned(),
    )));
  }

  if password.is_empty() {
    debug!("Missing password, returning 400 status code");
    return Err(Error::BadRequest(BadRequest::new(
      "password".to_owned(),
      "Missing password attribute".to_owned(),
    )));
  }

  let account = context
    .models
    .account
    .find_one(doc! { "email": email }, None)
    .await?;
    
  let account = match account {
    Some(account) => account,
    None => {
      debug!("Account not found, returning 401");
      return Err(Error::NotFound(NotFound::new(String::from("account"))));
    }
  };

  if !account.is_password_match(password) {
    debug!("Account password is incorrect, returning 401 status code");
    return Err(Error::Authenticate(AuthenticateError::WrongCredentials));
  }

  if account.locked_at.is_some() {
    debug!("Account is locked, returning 401");
    return Err(Error::Authenticate(AuthenticateError::Locked));
  }

  let secret = context.settings.auth.secret.as_str();
  let token = token::create(account.clone(), secret)
    .map_err(|_| Error::Authenticate(AuthenticateError::TokenCreation))?;

  let res = AuthenticateResponse {
    access_token: token,
    account: PublicAccount::from(account),
  };

  Ok(Json(res))
}

#[derive(Debug, Deserialize)]
struct AuthorizeBody {
  email: String,
  password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AuthenticateResponse {
  access_token: String,
  account: PublicAccount,
}
