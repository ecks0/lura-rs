use include_dir::DirEntry;
use thiserror;
use std::path::Path;
use crate::template::expand;

pub use include_dir::Dir;
pub use templar::Document;

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
  
  pub fn to_file<P: AsRef<Path>>(&self, path: &str, dst: P) -> Result<()> {
    Ok(crate::fs::dump(dst, self.as_str(path)?)?)
  }
  
  pub fn expand_str(&self, name: &str, config: crate::config::Config) -> Result<String> {
    Ok(expand(self.as_str(name)?, config.into())?)
  }
  
  pub fn expand_file<P: AsRef<Path>>(
    &self, name: &str, config: crate::config::Config, path: P
  ) -> Result<()>
  {
    Ok(crate::fs::dump(path, &self.expand_str(name, config)?)?)
  }
}
