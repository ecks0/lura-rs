use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use std::{
  env,
  ffi::OsString,
  fs::{
    Permissions,
    set_permissions,
  },
  ops::Deref,
  os::unix::fs::PermissionsExt,
  path::{
    Path,
    PathBuf,
  },
};
use tempdir as tempdir_rs;
use thiserror;

#[derive(thiserror::Error, Debug)]
pub enum Error {

  #[error("Utf8 conversion error")]
  Utf8(OsString),

  #[error(transparent)]
  Io(#[from] std::io::Error),

  #[error(transparent)]
  Run(#[from] crate::run::Error),
}

type Result<T> = std::result::Result<T, Error>;

// wrap tempdir::TempDir to set 0700

#[derive(Debug)]
pub struct TempDir(tempdir_rs::TempDir);

impl TempDir {
  pub fn new(prefix: &str) -> Result<Self> {
    let this = Self(tempdir_rs::TempDir::new(prefix)?);
    chmod(this.0.path(), 0o700)?;
    Ok(this)
  }
}

impl Deref for TempDir {
  type Target = tempdir_rs::TempDir;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

#[inline]
pub fn mkdir<P: AsRef<Path>>(path: P) -> Result<()> {
  Ok(std::fs::create_dir(&path)?)
}

#[inline]
pub fn chmod<P: AsRef<Path>>(path: P, mode: u32) -> Result<()> {
  Ok(set_permissions(path, Permissions::from_mode(mode))?)
}

#[inline]
pub fn rm<P: AsRef<Path>>(path: P) -> Result<()> {
  let result = match path.as_ref().is_dir() {
    true => std::fs::remove_dir_all(path)?,
    false => std::fs::remove_file(path)?,
  };
  Ok(result)
}

#[inline]
pub fn load<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
  Ok(std::fs::read(path)?)
}

#[inline]
pub fn loads<P: AsRef<Path>>(path: P) -> Result<String> {
  Ok(std::fs::read_to_string(path)?)
}

#[inline]
pub fn dump<P: AsRef<Path>, D: AsRef<[u8]>>(path: P, data: D) -> Result<()> {
  Ok(std::fs::write(path, data)?)
}

#[inline]
pub fn path_to_string(path: &Path) -> Result<String> {
  let result = path
    .to_path_buf()
    .into_os_string()
    .into_string()
    .map_err(|e| Error::Utf8(e))?;
  Ok(result)
}

#[inline]
pub fn path_buf_to_string(path: PathBuf) -> Result<String> {
  let result = path
    .into_os_string()
    .into_string()
    .map_err(|e| Error::Utf8(e))?;
  Ok(result)
}

pub fn tempdir(prefix: &str) -> Result<String> {

  fn randstr() -> String {
    thread_rng()
      .sample_iter(&Alphanumeric)
      .take(12)
      .collect()
  }

  loop {
    let temp_dir = format!("{0}/{1}.{2}", path_buf_to_string(env::temp_dir())?, prefix, randstr());
    match mkdir(&temp_dir) {
      Ok(()) => {
        chmod(&temp_dir, 0o700)?;
        return Ok(temp_dir);
      },
      Err(err) => {
        if let Error::Io(err) = &err {
          if let std::io::ErrorKind::AlreadyExists = err.kind() { continue; }
        }
        return Err(err);
      },
    }
  }
}
