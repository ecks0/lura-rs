use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use std::{
  io::ErrorKind as IoErrorKind,
  env,
  path::Path,
  result::Result as StdResult,
  io::Error as IoError,
};
use thiserror;

#[derive(thiserror::Error, Debug)]
pub enum Error {

  #[error("Utf8 conversion error")]
  Utf8,

  #[error(transparent)]
  Io(#[from] std::io::Error),
}

type Result<T> = std::result::Result<T, Error>;

#[inline]
pub fn mkdir<P: AsRef<Path>>(path: P) -> StdResult<(), IoError> {
  std::fs::create_dir(&path)
}

#[inline]
pub fn rm<P: AsRef<Path>>(path: P) -> StdResult<(), IoError> {
  match path.as_ref().is_dir() {
    true => std::fs::remove_dir_all(path),
    false => std::fs::remove_file(path),
  }
}

#[inline]
pub fn load<P: AsRef<Path>>(path: P) -> StdResult<Vec<u8>, IoError> {
  std::fs::read(path)
}

#[inline]
pub fn loads<P: AsRef<Path>>(path: P) -> StdResult<String, IoError> {
  std::fs::read_to_string(path)
}

#[inline]
pub fn dump<P: AsRef<Path>, D: AsRef<[u8]>>(path: P, data: D) -> StdResult<(), IoError> {
  std::fs::write(path, data)
}

pub fn tempdir(prefix: &str) -> Result<String> {

  fn randstr() -> String {
    thread_rng()
      .sample_iter(&Alphanumeric)
      .take(8)
      .collect()
  }

  let temp_root_file = env::temp_dir();
  let temp_root = temp_root_file.to_str().ok_or(Error::Utf8)?;
  loop {
    let temp_dir = format!("{0}/{1}.{2}", temp_root, prefix, randstr());
    if Path::new(&temp_dir).exists() {
      continue;
    }
    match mkdir(&temp_dir) {
      Ok(()) => return Ok(temp_dir),
      Err(err) => {
        match err.kind() {
          IoErrorKind::AlreadyExists => continue,
          _ => return Err(Error::from(err)),
        }
      },
    }
  }
}
