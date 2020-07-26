use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use std::io::ErrorKind as IoErrorKind;
use std::env;
use std::fs::{
  create_dir,
  read_to_string,
  remove_dir_all,
  write as write_file,
};
use std::path::Path;
use thiserror;

#[derive(thiserror::Error, Debug)]
pub enum Error {

  #[error("Utf8 conversion error")]
  Utf8,

  #[error(transparent)]
  IoError(#[from] std::io::Error),
}

type Result<T> = std::result::Result<T, Error>;

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
    let temp_dir = format!("{0}/cdhub.{1}.{2}", temp_root, prefix, randstr());
    if Path::new(&temp_dir).exists() {
      continue;
    }
    match create_dir(&temp_dir) {
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

#[inline]
pub fn rm(path: &str) -> Result<()> {
  remove_dir_all(path)?;
  Ok(())
}

#[inline]
pub fn dump<P: AsRef<Path>, D: AsRef<[u8]>>(path: P, data: D) -> Result<()> {
  write_file(path, data)?;
  Ok(())
}

#[inline]
pub fn loads<P: AsRef<Path>>(path: P) -> Result<String> {
  Ok(read_to_string(path)?)
}
