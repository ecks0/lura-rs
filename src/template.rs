use {
  log::debug,
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

const MOD: &str = std::module_path!();

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
  Ok(dump(path, expand_str(template, config)?)?)
}

#[cfg(feature = "lua")]
use {
  rlua::{ Context as LuaContext, Error as LuaError, Result as LuaResult, Table },
  std::sync::Arc,
};

#[cfg(feature = "lua")]
impl From<Error> for LuaError {
  fn from(err: Error) -> LuaError {
    LuaError::ExternalError(Arc::new(err))
  }
}

#[cfg(feature = "lua")]
pub(crate) fn lua_init(ctx: &LuaContext) -> LuaResult<()> {
 
  debug!(target: MOD, "Lua init");

  let template = ctx.create_table()?;

  template.set("expand_str", ctx.create_function(|_, args: (String, Config)| {
    Ok(expand_str(&args.0, &args.1)?)
  })?)?;
  template.set("expand_file", ctx.create_function(|_, args: (String, Config, String)| {
    Ok(expand_file(&args.0, &args.1, &args.2)?)
  })?)?;

  ctx
    .globals()
    .get::<_, Table>("lura")?
    .set("template", template)?;

  Ok(())
}