// template expansion utilities
//
// this module provides methods to expand `templar` templates either to a `String` or
// to a file. `crate::config::Config` can be converted to an `unstructured::Document`,
// which is understood by templar, and is intended to be used as the expansion environment
//
// this module also provides the `range()` function for templates to use

use {
  lazy_static::lazy_static,
  templar::{
    Context,
    Data,
    StandardContext,
    Templar,
    TemplarBuilder,
    TemplarError,
  },
  thiserror,
  unstructured::Document,
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

fn range(args: Data) -> Data {
  // range function for templar templates

  #[inline]
  fn err() -> Data {
    TemplarError::RenderFailure("range(): expected 1 or 2 integer arguments".into()).into()
  }
  
  let mut range = [0, 0];
  match args.into_result() {
    Ok(Document::Seq(s)) => {
      if s.len() != 2 { return err(); }
      for i in 0..2 {
        match s[i].as_i64() {
          Some(arg) => range[i] = arg,
          None => return err(),
        }
      }
    },
    Ok(other) => {
      match other.as_i64() {
        Some(arg) => range[1] = arg,
        _ => return err(),
      }
    },
    Err(e) => return e.into(),
  }
  Document::Seq(
    (range[0]..range[1])
    .map(|i| i.into())
    .collect::<Vec<Document>>()
    .into()
  ).into()
}

pub fn to_string(template: &str, config: &Config) -> Result<String> {
  // expand a `template` string using `env`
  
  let template = TEMPLAR.parse(template)?;
  let context = StandardContext::new();
  context.set(config)?;
  Ok(template.render(&context)?)
}

pub fn to_file(template: &str, config: &Config, path: &str) -> Result<()> {
  // expand a `template` string using `env` to the file at `path`

  Ok(dump(path, to_string(template, config)?)?)
}
