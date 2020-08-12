// template expansion utilities
//
// this module provides methods to expand `templar` templates either to a `String` or
// to a file. `crate::config::Config` is able to be converted to a `templar::Document`
// and is intended to be used as the expansion environment

use {
  templar::{
    Context,
    Document,
    StandardContext,
    Templar,
  },
  thiserror,
  toml::Value,
  crate::{
    config::Config,
    fs::dump,
  },
};

#[derive(thiserror::Error, Debug)]
pub enum Error {

  #[error(transparent)]
  LuraFs(#[from] crate::fs::Error),

  #[error(transparent)]
  Templar(#[from] templar::TemplarError),
}

type Result<T> = std::result::Result<T, Error>;

pub(crate) fn toml_to_document(value: &Value) -> Option<Document> {
  // convert a toml `Value` to a templar `Document`

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

pub fn expand_str(template: &str, config: &Config) -> Result<String> {
  // expand a `template` string using `env`
  
  let template = Templar::global().parse(template)?;
  let context = StandardContext::new();
  context.set(config)?;
  Ok(template.render(&context)?)
}

pub fn expand_file(template: &str, config: &Config, path: &str) -> Result<()> {
  // expand a `template` string using `env` to the file at `path`

  Ok(dump(path, expand_str(template, config)?)?)
}
