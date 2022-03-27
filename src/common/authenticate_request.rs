use axum::{
  async_trait,
  extract::{FromRequest, RequestParts, TypedHeader},
  headers::{authorization::Bearer, Authorization},
};
use tracing::debug;

use crate::context::Context;
use crate::errors::AuthenticateError;
use crate::errors::Error;
use crate::common::token;
use crate::common::token::TokenAccount;
use crate::common::token::ADMIN_PATH;
use crate::common::token::USER_PATH;
use crate::common::token::VALIDATOR_PATH;

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
    debug!("from_request");
    let extensions = req.extensions().ok_or(Error::ReadContext)?;
    let context = extensions.get::<Context>().ok_or(Error::ReadContext)?;
    let secret = context.settings.auth.secret.as_str();
    let token_data =
      token::decode(bearer.token(), secret).map_err(|_| AuthenticateError::InvalidToken)?;
    debug!("token_data: {:?}", token_data);

    match &req.uri().path()[..7] {
      ADMIN_PATH => {
        debug!("admin request");
        if !token_data.claims.account.roles.iter().any(|i| i.name == "admin") {
          return Err(Error::Authenticate(AuthenticateError::WrongCredentials));
        }
      },
      USER_PATH => {
        debug!("user request");
        if !token_data.claims.account.roles.iter().any(|i| i.name == "user") {
          return Err(Error::Authenticate(AuthenticateError::WrongCredentials));
        }
      },
      VALIDATOR_PATH => {
        debug!("validator request");
        if !token_data.claims.account.roles.iter().any(|i| i.name == "validator") {
          return Err(Error::Authenticate(AuthenticateError::WrongCredentials));
        }
      },
      _ => debug!("unprotected request"),
    }

    Ok(token_data.claims.account)
  }
}
