use thiserror::Error;
use toml::Value;
use crate::merge::merge_toml;

#[derive(Error, Debug)]
pub enum Error {

  #[error(transparent)]
  TomlError(#[from] toml::de::Error),
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Config(Value);

impl Config {

  pub fn new(contents: &str) -> Result<Self> {
    Ok(Self(contents.parse::<Value>()?))
  }

  pub fn update(&mut self, other: &Config) {
    merge_toml(&mut self.0, &other.0)
  }

  pub fn value(&self) -> Value {
    self.0.clone()
  }

  pub fn get(&self, key: &str) -> Option<&Value> { 

    fn get<'a>(keys: &[&str], value: &'a Value) -> Option<&'a Value> {
      let next_value = match &value.get(keys[0]) {
        Some(next_value) => *next_value,
        None => return None,
      };
      match keys.len() {
        1 => Some(next_value),
        _ => get(&keys[1..], next_value),
      }
    }

    return get(&key.split('.').collect::<Vec<&str>>()[..], &self.0);
  }

  pub fn as_str(&self, key: &str) -> Option<String> {
    self
      .get(key)
      .and_then(|v| {
        v.as_str()
          .and_then(|v| {
            Some(v.to_owned())
          })
      })
  }

  pub fn as_bool(&self, key: &str) -> Option<bool> {
    self.get(key).and_then(|v| v.as_bool())
  }

  pub fn as_int(&self, key: &str) -> Option<i64> {
    self.get(key).and_then(|v| v.as_integer())
  }

  pub fn as_float(&self, key: &str) -> Option<f64> {
    self.get(key).and_then(|v| v.as_float())
  }
}
