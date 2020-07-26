use std::collections::HashMap;
use thiserror::Error;
use toml::Value;
use crate::merge::merge_toml;

#[derive(Error, Debug)]
pub enum Error {

  #[error("No configuration found for key `{0}`")]
  ConfigMissing(String),

  #[error("No configuration value named `{0}`")]
  ConfigValueMissing(String),

  #[error(transparent)]
  TomlError(#[from] toml::de::Error),
}

type Result<T> = std::result::Result<T, Error>;

pub struct Config(Value);

impl Config {

  pub fn new(configs: HashMap<&str, &str>, key: &str) -> Result<Self> {
    let target = configs.get(key).ok_or_else(|| Error::ConfigMissing(key.to_owned()))?;
    let target = target.parse::<Value>()?;
    match configs.get("default") {
      Some(default) => {
        let mut config = default.parse::<Value>()?;
        merge_toml(&mut config, &target);
        Ok(Self(config))
      },
      _ => Ok(Self(target)),
    }
  }

  pub fn get(&self, key: &str) -> Option<Value> { 

    fn get(keys: Vec<&str>, value: &Value) -> Option<Value> {
      let next_value = match &value.get(keys[0]) {
        Some(next_value) => *next_value,
        None => return None,
      };
      match keys.len() {
        1 => Some(next_value.clone()),
        _ => get(keys[1..].iter().cloned().collect(), next_value),
      }
    }

    return get(key.split('.').collect(), &self.0);
  }

  pub fn as_str(&self, key: &str) -> Option<String> {
    self
      .get(key)
      .and_then(|k| k.as_str().and_then(|k| Some(k.to_owned())))
  }

  pub fn as_bool(&self, key: &str) -> Option<bool> {
    self.get(key).and_then(|k| k.as_bool())
  }

  pub fn as_int(&self, key: &str) -> Option<i64> {
    self.get(key).and_then(|k| k.as_integer())
  }

  pub fn as_float(&self, key: &str) -> Option<f64> {
    self.get(key).and_then(|k| k.as_float())
  }
}
