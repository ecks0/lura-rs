// Config file api
//
// - `Config` is a facade for `toml::Value`
// - `Config` instances can be merged with other instances
// - `Config` values are accessed by path string, e.g. `"foo.bar.baz"`
// - `Config` instances can be used as a template expansion environment via
//   the `crate::template::expand_*()` functions
// - `Config` methods return `Result` rather than `Option`

use {
  std::collections::BTreeMap,
  thiserror::Error,
  crate::merge,
};

pub use {
  toml::{
    value::{Array, Table},
    map::Map,
    Value,
  },
  unstructured::Document,
};

#[derive(Error, Debug)]
pub enum Error {

  #[error("{1}: error converting to {0}")]
  ConvertFailed(&'static str, String),

  #[error("Config key not found: `{0}`")]
  KeyMissing(String),

  #[error(transparent)]
  TomlError(#[from] toml::de::Error),

  #[error("Value error: {0}")]
  Value(&'static str),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug)]
pub struct Config(Value);

impl Config {

  pub fn new(contents: &str) -> Result<Self> {
    Ok(Self(contents.parse::<Value>()?))
  }

  pub fn update(&mut self, other: &Config) {
    merge::toml(&mut self.0, &other.0)
  }

  pub fn value(&self) -> &Value {
    &self.0
  }

  pub fn get(&self, key: &str) -> Result<&Value> { 

    fn get<'a>(keys: &[&str], value: &'a Value) -> Result<&'a Value> {
      match value.get(keys[0]) {
        None => Err(Error::KeyMissing(keys[0].to_owned())),
        Some(next_value) => {
          match keys.len() {
            1 => Ok(next_value),
            _ => get(&keys[1..], &next_value),
          }
        },
      }
    }

    match get(&key.split('.').collect::<Vec<&str>>()[..], &self.0) {
      Err(Error::KeyMissing(subkey)) => Err(Error::KeyMissing(format!("{} ({})", key, subkey))),
      ok => ok,
    }
  }

  pub fn contains(&self, key: &str) -> bool {
    match self.get(key) {
      Ok(_) => true,
      Err(_) => false,
    }
  }

  pub fn as_str<'a>(&'a self, key: &str) -> Result<&'a str> {
    self
      .get(key)?
      .as_str().ok_or(Error::ConvertFailed("str", key.to_owned()))
  }

  pub fn as_bool(&self, key: &str) -> Result<bool> {
    self
      .get(key)?
      .as_bool().ok_or(Error::ConvertFailed("bool", key.to_owned()))
  }

  pub fn as_int(&self, key: &str) -> Result<i64> {
    self
      .get(key)?
      .as_integer().ok_or(Error::ConvertFailed("integer", key.to_owned()))
  }

  pub fn as_float(&self, key: &str) -> Result<f64> {
    self
      .get(key)?
      .as_float().ok_or(Error::ConvertFailed("float", key.to_owned()))
  }

  pub fn as_map(&self, key: &str) -> Result<&Map<String, Value>> {
    self
      .get(key)?
      .as_table()
      .ok_or(Error::ConvertFailed("map", key.to_owned()))
  }

  pub fn as_vec(&self, key: &str) -> Result<&Vec<Value>> {
    self
      .get(key)?
      .as_array()
      .ok_or(Error::ConvertFailed("array", key.to_owned()))
  }
}

pub fn toml_to_document(value: &Value) -> Document {
  // convert a toml `Value` to an `unstructured::Document`

  match value {
    Value::Array(val) =>
      val
        .iter()
        .map(|v| toml_to_document(v))
        .collect::<Vec<Document>>()
        .into(),
    Value::Table(table) =>
      table
        .iter()
        .map(|(k, v)| (k.into(), toml_to_document(v)))
        .collect::<BTreeMap<Document, Document>>()
        .into(),
    Value::Boolean(val) => val.into(),
    Value::Integer(val) => val.into(),
    Value::Float(val) => val.into(),
    Value::String(val) => val.into(),
    Value::Datetime(val) => val.to_string().into(),
  }
}

impl From<Config> for Document {

  fn from(config: Config) -> Document {
    toml_to_document(&config.value())
  }
}

impl From<&Config> for Document {

  fn from(config: &Config) -> Document {
    toml_to_document(&config.value())
  }
}
