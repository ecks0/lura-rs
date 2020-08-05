use {
  log::debug,
  thiserror::Error,
  templar::Document,
  toml::Value,
  crate::merge::merge_toml,
};

const MOD: &str = std::module_path!();

#[derive(Error, Debug)]
pub enum Error {

  #[error("Value error: {0}")]
  Value(&'static str),

  #[error("Config missing: `{0}`")]
  ConfigMissing(String),

  #[error("Config value missing: `{0}`")]
  ConfigValueMissing(String),

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
  rlua::{ 
    Context, Error as LuaError, FromLua, Result as LuaResult, Table, UserData, UserDataMethods
  },
  std::sync::Arc,
};

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
