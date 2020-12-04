// static file management api
//
// `Assets` is a facade over `include_dir::Dir`
//
// - `Assets` hides directories from users, only returns file paths
// - `Assets` can write static data to the filesystem
// - `Assets` can load static data as a template and expand it using `crate::config::Config`
//    to a string or to the filesystem

use {
  include_dir::DirEntry,
  thiserror,
  unstructured::Document,
  crate::template,
};

pub use include_dir::Dir;

#[derive(thiserror::Error, Debug)]
pub enum Error {
  
  #[error("Utf8 conversion error: `{0}`")]
  Utf8(String),

  #[error("Asset missing: `{0}`")]
  AssetMissing(String),

  #[error(transparent)] GlobPatternError(#[from] glob::PatternError),
  #[error(transparent)] StdIo(#[from] std::io::Error),
  #[error(transparent)] LuraFs(#[from] crate::fs::Error),
  #[error(transparent)] LuraTemplate(#[from] crate::template::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug)]
pub struct Assets<'a>(&'a Dir<'a>);

impl<'a> Assets<'a> {

  pub fn new(dir: &'a Dir<'a>) -> Self {
    // return a new `Assets` instance for `dir`

    Self(dir)
  }

  pub fn find(&self, glob: &str) -> Result<impl Iterator<Item = &str>> {
    // find path names matching `glob`

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
    // list all file paths

    Ok(self.find("*")?)
  }

  pub fn present(&self, path: &str) -> bool {
    // return `true` if file path is present, else `false`

    self.0.contains(path)
   }
  
  pub fn as_bytes(&self, path: &str) -> Result<&[u8]> {
    // return static data as bytes

    Ok(self.0
      .get_file(path)
      .ok_or_else(|| Error::AssetMissing(path.to_owned()))
      .and_then(|file| Ok(file.contents()))?)
  }
  
  pub fn as_str(&self, path: &str) -> Result<&str> {
    // return static data as `&str`

    Ok(self.0
      .get_file(path)
      .ok_or_else(|| Error::AssetMissing(path.to_owned()))
      .and_then(|file| {
        file
          .contents_utf8()
          .ok_or_else(|| Error::Utf8(path.to_owned()))
      })?)
  }
  
  pub fn to_file(&self, path: &str, dst: &str) -> Result<()> {
    // write static data to a file

    Ok(std::fs::write(dst, self.as_str(path)?)?)
  }
  
  pub fn expand_string(&self, name: &str, document: Document) -> Result<String> {
    // expand static template data to a `String` using `config`

    Ok(template::to_string(self.as_str(name)?, document)?)
  }
  
  pub fn expand_file(&self, name: &str, document: Document, path: &str) -> Result<()> {
    // expand static template data to a file using `config`

    Ok(template::to_file(self.as_str(name)?, document, path)?)
  }
}
