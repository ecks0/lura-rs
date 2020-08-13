// template expansion utilities
//
// this module provides methods to expand `templar` templates either to a `String` or
// to a file. `crate::config::Config` is able to be converted to a `templar::Document`
// and is intended to be used as the expansion environment

use {
  lazy_static::lazy_static,
  std::{
    collections::BTreeMap,
    sync::Arc,
  },
  templar::{
    Context,
    Data,
    Document,
    StandardContext,
    Templar,
    TemplarBuilder,
    TemplarError,
  },
  thiserror,
  toml::Value,
  crate::{
    config::Config,
    fs::dump,
  },
};

lazy_static! {
  static ref TEMPLAR: Templar = {
    let mut builder = TemplarBuilder::default();
    builder.add_function("range", range);
    builder.build()
  };
}

#[derive(thiserror::Error, Debug)]
pub enum Error {

  #[error("Value error: {0}")]
  Value(&'static str),

  #[error(transparent)]
  LuraFs(#[from] crate::fs::Error),

  #[error(transparent)]
  Templar(#[from] templar::TemplarError),
}

type Result<T> = std::result::Result<T, Error>;

impl From<Error> for templar::error::TemplarError {
    
  fn from(err: Error) -> Self {
    Self::Other(Arc::new(Box::new(err)))
  }
}

pub(crate) fn toml_to_document(value: &Value) -> Document {
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

pub fn range(args: Data) -> Data {
  // range function for templar templates

  let mut range_args = vec![];
  match args.into_result() {
    Ok(Document::Seq(i)) => {
      for arg in i.iter() {
        match arg.as_i64() {
          Some(arg) => range_args.push(arg),
          None => return TemplarError::RenderFailure("range(): expected integer argument".into()).into(),
        }
      }
    },
    Ok(other) => {
      range_args.push(0);
      match other.as_i64() {
        Some(arg) => range_args.push(arg),
        None => return TemplarError::RenderFailure("range(): expected integer argument".into()).into(),
      }
    },
    Err(e) => return e.into(),
  }
  Document::Seq(
    (range_args[0]..range_args[1])
    .map(|i| i.into())
    .collect::<Vec<Document>>()
    .into()
  ).into()
}

pub fn expand_str(template: &str, config: &Config) -> Result<String> {
  // expand a `template` string using `env`
  
  let template = TEMPLAR.parse(template)?;
  let context = StandardContext::new();
  context.set(config)?;
  Ok(template.render(&context)?)
}

pub fn expand_file(template: &str, config: &Config, path: &str) -> Result<()> {
  // expand a `template` string using `env` to the file at `path`

  Ok(dump(path, expand_str(template, config)?)?)
}
