use {
  include_dir::DirEntry,
  thiserror,
  crate::{
    config::Config,
    template::{expand_file, expand_str},
  },
};

pub use include_dir::Dir;

#[derive(thiserror::Error, Debug)]
pub enum Error {
  
  #[error("Utf8 conversion error: `{0}`")]
  Utf8(String),

  #[error(transparent)]
  GlobPatternError(#[from] glob::PatternError),

  #[error("Relic missing: `{0}`")]
  RelicMissing(String),

  #[error(transparent)]
  StdIo(#[from] std::io::Error),

  #[error(transparent)]
  LuraFs(#[from] crate::fs::Error),

  #[error(transparent)]
  LuraTemplate(#[from] crate::template::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug)]
pub struct Relics<'a>(&'a Dir<'a>);

impl<'a> Relics<'a> {

  pub fn new(dir: &'a Dir<'a>) -> Self {
    Self(dir)
  }

  pub fn find(&self, glob: &str) -> Result<impl Iterator<Item = &str>> {
    Ok(self.0
      .find(glob)?
      .filter_map(|ent| {
        match &ent {
          DirEntry::File(file) => file.path().to_str(),
          DirEntry::Dir(_) => None,
        }
        }))
  }

  pub fn list(&self) -> Result<impl Iterator<Item = &str>> {
    Ok(self.find("*")?)
  }

  pub fn present(&self, path: &str) -> bool {
    self.0.contains(path)
   }
  
  pub fn as_bytes(&self, path: &str) -> Result<&[u8]> {
    Ok(self.0
      .get_file(path)
      .ok_or_else(|| Error::RelicMissing(path.to_owned()))
      .and_then(|file| Ok(file.contents()))?)
  }
  
  pub fn as_str(&self, path: &str) -> Result<&str> {
    Ok(self.0
      .get_file(path)
      .ok_or_else(|| Error::RelicMissing(path.to_owned()))
      .and_then(|file| {
        file
          .contents_utf8()
          .ok_or_else(|| Error::Utf8(path.to_owned()))
      })?)
  }
  
  pub fn to_file(&self, path: &str, dst: &str) -> Result<()> {
    Ok(crate::fs::dump(dst, self.as_str(path)?)?)
  }
  
  pub fn expand_str(&self, name: &str, config: &Config) -> Result<String> {
    Ok(expand_str(self.as_str(name)?, config)?)
  }
  
  pub fn expand_file(&self, name: &str, config: &Config, path: &str) -> Result<()> {
    Ok(expand_file(self.as_str(name)?, config, path)?)
  }
}

#[cfg(feature = "lua")]
use {
  rlua::{ Error as LuaError, UserData, UserDataMethods },
  std::sync::Arc,
};

#[cfg(feature = "lua")]
impl From<Error> for LuaError {
  fn from(err: Error) -> LuaError {
    LuaError::ExternalError(Arc::new(err))
  }
}

#[cfg(feature = "lua")]
impl<'a> UserData for Relics<'a> {
  fn add_methods<'lua, T: UserDataMethods<'lua, Self>>(methods: &mut T) {
    methods.add_method("find", |_, this, args: (String,)| {
      Ok(this.find(&args.0)?.map(|i| i.to_owned()).collect::<Vec<String>>())
    });
    methods.add_method("list", |_, this, _: ()| {
      Ok(this.list()?.map(|i| i.to_owned()).collect::<Vec<String>>())
    });
    methods.add_method("as_bytes", |_, this, args: (String,)| {
      Ok(this.as_bytes(&args.0)?.iter().cloned().collect::<Vec<u8>>())
    });
    methods.add_method("as_str", |_, this, args: (String,)| {
      Ok(this.as_str(&args.0)?.to_owned())
    });
    methods.add_method("to_file", |_, this, args: (String, String)| {
      Ok(this.to_file(&args.0, &args.1)?)
    });
    methods.add_method("expand_str", |_, this, args: (String, Config)| {
      Ok(this.expand_str(&args.0, &args.1)?)
    });
    methods.add_method("expand_file", |_, this, args: (String, Config, String)| {
      Ok(this.expand_file(&args.0, &args.1, &args.2)?)
    });
  }
}