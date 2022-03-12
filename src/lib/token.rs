use bson::oid::ObjectId;
use jsonwebtoken::{errors::Error, DecodingKey, EncodingKey, Header, TokenData, Validation};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::models::account::Account;

type TokenResult = Result<TokenData<Claims>, Error>;

// TODO: move these to enum struct
pub static ADMIN_PATH: &str = "/6a2dda";
pub static VALIDATOR_PATH: &str = "/5be0da";
pub static USER_PATH: &str = "/6b0866";

static VALIDATION: Lazy<Validation> = Lazy::new(Validation::default);
static HEADER: Lazy<Header> = Lazy::new(Header::default);

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenAccount {
  pub id: ObjectId,
  pub name: String,
  pub email: String,
  pub roles: Vec<String>,
}

impl From<Account> for TokenAccount {
  fn from(account: Account) -> Self {
    Self {
      id: account.id.unwrap(),
      name: account.name.clone(),
      email: account.email,
      roles: account.roles,
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
  pub exp: usize, // Expiration time (as UTC timestamp). validate_exp defaults to true in validation
  pub iat: usize, // Issued at (as UTC timestamp)
  pub account: TokenAccount,
}

impl Claims {
  pub fn new(account: Account) -> Self {
    Self {
      exp: (chrono::Local::now() + chrono::Duration::days(30)).timestamp() as usize,
      iat: chrono::Local::now().timestamp() as usize,
      account: TokenAccount::from(account),
    }
  }
}

pub fn create(account: Account, secret: &str) -> Result<String, Error> {
  let encoding_key = EncodingKey::from_secret(secret.as_ref());
  let claims = Claims::new(account);

  jsonwebtoken::encode(&HEADER, &claims, &encoding_key)
}

pub fn decode(token: &str, secret: &str) -> TokenResult {
  let decoding_key = DecodingKey::from_secret(secret.as_ref());

  jsonwebtoken::decode::<Claims>(token, &decoding_key, &VALIDATION)
}
