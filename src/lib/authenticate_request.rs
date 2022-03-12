use axum::{
  async_trait,
  extract::{FromRequest, RequestParts, TypedHeader},
  headers::{authorization::Bearer, Authorization},
};
use tracing::debug;

use crate::context::Context;
use crate::errors::AuthenticateError;
use crate::errors::Error;
use crate::lib::token;
use crate::lib::token::TokenAccount;
use crate::lib::token::ADMIN_PATH;
use crate::lib::token::USER_PATH;
use crate::lib::token::VALIDATOR_PATH;

#[async_trait]
impl<B> FromRequest<B> for TokenAccount
where
  B: Send,
{
  type Rejection = Error;

  async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
    let TypedHeader(Authorization(bearer)) =
      TypedHeader::<Authorization<Bearer>>::from_request(req)
        .await
        .map_err(|_| AuthenticateError::InvalidToken)?;

    let extensions = req.extensions().ok_or(Error::ReadContext)?;
    let context = extensions.get::<Context>().ok_or(Error::ReadContext)?;
    let secret = context.settings.auth.secret.as_str();
    let token_data =
      token::decode(bearer.token(), secret).map_err(|_| AuthenticateError::InvalidToken)?;
    debug!("token_data: {:?}", token_data);
    if req.uri().path().starts_with(&ADMIN_PATH) {
      debug!("admin request");
      if !token_data.claims.account.roles.iter().any(|i| i == "admin") {
        return Err(Error::Authenticate(AuthenticateError::WrongCredentials));
      }
    } else if req.uri().path().starts_with(&USER_PATH) {
      debug!("user request");
      if !token_data.claims.account.roles.iter().any(|i| i == "user") {
        return Err(Error::Authenticate(AuthenticateError::WrongCredentials));
      }
    } else if req.uri().path().starts_with(&VALIDATOR_PATH) {
      debug!("validator request");
      if !token_data.claims.account.roles.iter().any(|i| i == "validator") {
        return Err(Error::Authenticate(AuthenticateError::WrongCredentials));
      }
    } else {
      debug!("unprotected request")
    }
    Ok(token_data.claims.account)
  }
}
