use templar::{
  Context,
  StandardContext,
  Templar,
};
use thiserror;
use toml::Value;
use crate::config::Config;

pub use templar::Document;

#[derive(thiserror::Error, Debug)]
pub enum Error {

  #[error(transparent)]
  Templar(#[from] templar::TemplarError),
}

type Result<T> = std::result::Result<T, Error>;

pub fn toml_to_document(value: &Value) -> Option<Document> {
  match value {
    Value::Table(table) => {
      let mut document = Document::default();
      for (key, val) in table {
        match toml_to_document(val) {
          Some(val) => document[key] = val,
          None => continue,
        }
      }
      Some(document)
    },
    Value::Boolean(val) => Some(val.into()),
    Value::Integer(val) => Some(val.into()),
    Value::Float(val) => Some(val.into()),
    Value::String(val) => Some(val.into()),
    Value::Array(_) => None, // FIXME
    Value::Datetime(_) => None, // FIXME
  }
}

impl From<Config> for Document {

  fn from(config: Config) -> Document {
    toml_to_document(&config.value()).unwrap_or(Document::default())
  }
}

pub fn expand(template: &str, env: Document) -> Result<String> {
  let template = Templar::global().parse(template)?;
  let context = StandardContext::new();
  context.set(env)?;
  Ok(template.render(&context)?)
}
