use crate::models::Models;
use crate::settings::Settings;

#[derive(Clone)]
pub struct Context {
  pub models: Models,
  pub settings: Settings,
}

impl Context {
  pub fn new(models: Models, settings: Settings) -> Self {
    Self { models, settings }
  }
}