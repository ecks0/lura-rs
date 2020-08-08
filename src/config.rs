// Config file api
//
// - `Config` instances can be merged with other instances
// - `Config` values are accessed by path string, e.g. `"foo.bar.baz"`
// - `Config` instances can be used as a template expansion environment via
//   the `crate::template::expand_*()` functions
// - `Config` instances can be sent from rust to lua, and from lua to rust

use {
  thiserror::Error,
  templar::Document,
  toml::Value,
  crate::merge::merge_toml,
};

#[derive(Error, Debug)]
pub enum Error {

  #[error("Value error: {0}")]
  Value(&'static str),

  #[error("Config key not found: `{0}`")]
  KeyMissing(String),

  #[error("{1}: error converting to {0}")]
  ConvertFailed(&'static str, String),

  #[error(transparent)]
  TomlError(#[from] toml::de::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

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

  pub fn as_str<'a>(&'a self, key: &str) -> Result<&'a str> {
    Ok(self
      .get(key)?
      .as_str().ok_or(Error::ConvertFailed("str", key.to_owned()))?)
  }

  pub fn as_bool(&self, key: &str) -> Result<bool> {
    Ok(self
      .get(key)?
      .as_bool().ok_or(Error::ConvertFailed("bool", key.to_owned()))?)
  }

  pub fn as_int(&self, key: &str) -> Result<i64> {
    Ok(self
      .get(key)?
      .as_integer().ok_or(Error::ConvertFailed("integer", key.to_owned()))?)
  }

  pub fn as_float(&self, key: &str) -> Result<f64> {
    Ok(self
      .get(key)?
      .as_float().ok_or(Error::ConvertFailed("float", key.to_owned()))?)
  }
}

impl From<Config> for Document {

  fn from(config: Config) -> Document {
    crate::template::toml_to_document(&config.value()).unwrap_or(Document::default())
  }
}

impl From<&Config> for Document {

  fn from(config: &Config) -> Document {
    crate::template::toml_to_document(&config.value()).unwrap_or(Document::default())
  }
}

#[cfg(feature = "lua")]
use {
  log::debug,
  rlua::{ 
    Context, Error as LuaError, FromLua, Result as LuaResult, Table, UserData, UserDataMethods
  },
  std::sync::Arc,
};

#[cfg(feature = "lua")]
const MOD: &str = std::module_path!();

#[cfg(feature = "lua")]
impl From<Error> for LuaError {
  fn from(err: Error) -> LuaError {
    LuaError::ExternalError(Arc::new(err))
  }
}

#[cfg(feature = "lua")]
impl<'lua> FromLua<'lua> for Config {

  fn from_lua(lua_value: rlua::Value<'lua>, _: rlua::Context<'lua>) -> LuaResult<Config> {
    Ok(match &lua_value {
      rlua::Value::UserData(config) => {
        let config = config.borrow::<Config>()
          .map_err(|_| Error::Value("Lua `UserData` is not a `Config`"))?;
        Ok(Self(config.0.clone()))
      },
      _ => Err(Error::Value("Lua `Value` is not a `Config`")),
    }?)
  }
}

#[cfg(feature = "lua")]
impl UserData for Config {

  fn add_methods<'lua, T: UserDataMethods<'lua, Self>>(methods: &mut T) {
    methods.add_method("as_str", |_, this, args: (String,)| {
      this.as_str(&args.0).ok_or_else(|| Error::ConfigValueMissing(args.0).into())
    });
    methods.add_method("as_bool", |_, this, args: (String,)| {
      this.as_bool(&args.0).ok_or_else(|| Error::ConfigValueMissing(args.0).into())
    });
    methods.add_method("as_int", |_, this, args: (String,)| {
      this.as_int(&args.0).ok_or_else(|| Error::ConfigValueMissing(args.0).into())
    });
    methods.add_method("as_float", |_, this, args: (String,)| {
      this.as_float(&args.0).ok_or_else(|| Error::ConfigValueMissing(args.0).into())
    });
  }
}

#[cfg(feature = "lua")]
pub(crate) fn lua_init(ctx: &Context) -> LuaResult<()> {

  debug!(target: MOD, "Lua init");
  let config = ctx.create_table()?;
  config.set("new", ctx.create_function(|_, args: (String,)| { Ok(Config::new(&args.0)?) })?)?;
  ctx
    .globals()
    .get::<_, Table>("lura")?
    .set("config", config)?;
  Ok(())
}
