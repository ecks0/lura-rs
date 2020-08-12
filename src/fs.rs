use {
  rand::{thread_rng, Rng},
  rand::distributions::Alphanumeric,
  regex::Regex,
  std::{
    env,
    ffi::OsString,
    fs::{Permissions, set_permissions},
    ops::Deref,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf}
  },
  tempdir as tempdir_rs,
  thiserror,
};

#[derive(thiserror::Error, Debug)]
pub enum Error {

  #[error("Utf8 conversion error")]
  Utf8(OsString),

  #[error(transparent)]
  Io(#[from] std::io::Error),

  #[error(transparent)]
  Regex(#[from] regex::Error),
  
  #[cfg(feature = "sync")]
  #[error(transparent)]
  LuraRun(#[from] crate::run::Error),
}

type Result<T> = std::result::Result<T, Error>;

// wrap tempdir::TempDir to set 0700

#[derive(Debug)]
pub struct TempDir {
  dir: tempdir_rs::TempDir,
  path: String,
}

impl TempDir {
  
  pub fn new(prefix: &str) -> Result<Self> {
    let dir = tempdir_rs::TempDir::new(prefix)?;
    let path = path_to_string(dir.path())?;
    chmod(&path, 0o700)?;
    Ok(Self { dir, path })
  }

  pub fn as_str<'a>(&'a self) -> &'a str {
    &self.path
  }

  pub fn to_string(&self) -> String {
    self.path.clone()
  }
}

impl Deref for TempDir {
  type Target = tempdir_rs::TempDir;

  fn deref(&self) -> &Self::Target {
    &self.dir
  }
}

pub fn tempdir(prefix: &str) -> Result<String> {
  // create and return a temporary directory in `std::env::temp_dir()` which begins with
  // `prefix`. the directory IS NOT automatically deleted. initial permissions of the directory
  // will be 0o700

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

pub fn mkdir(path: &str) -> Result<()> {
  // create a directory

  Ok(std::fs::create_dir(path)?)
}

pub fn chmod(path: &str, mode: u32) -> Result<()> {
  // set permissions for a path

  Ok(set_permissions(path, Permissions::from_mode(mode))?)
}

pub fn cp(src: &str, dst: &str) -> Result<()> {
  // move a file or directory recursively

  crate::run::run("cp", ["-R", src, dst].iter())?; // FIXME
  Ok(())
}

pub fn mv(src: &str, dst: &str) -> Result<()> {
  // move a file or directory recursively

  crate::run::run("mv", ["-f", src, dst].iter())?; // FIXME
  Ok(())
}

pub fn rm(path: &str) -> Result<()> {
  // remove a path. directories are removed recursively - be careful

  let path = Path::new(path);
  Ok(if path.is_dir() {
    std::fs::remove_dir_all(path)?
  } else {
     std::fs::remove_file(path)?
  })
}

pub fn exists(path: &str) -> bool {
  Path::new(path).exists()
}

pub fn is_file(path: &str) -> bool {
  Path::new(path).is_file()
}

pub fn is_dir(path: &str) -> bool {
  Path::new(path).is_dir()
}

pub fn load(path: &str) -> Result<Vec<u8>> {
  // load data from a file as bytes

  Ok(std::fs::read(path)?)
}

pub fn loads(path: &str) -> Result<String> {
  // load data from a file as `String`

  Ok(std::fs::read_to_string(path)?)
}

pub fn dump<D: AsRef<[u8]>>(path: &str, data: D) -> Result<()> {
  // write data to a file

  Ok(std::fs::write(path, data)?)
}

pub fn basename<'a>(path: &'a str) -> Option<&'a str> {
  match path.rfind('/') {
    Some(pos) => Some(&path[pos..]),
    None => None,
  }
}

pub fn dirname<'a>(path: &'a str) -> Option<&'a str> {
  match path.rfind('/') {
    Some(pos) => Some(&path[..pos]),
    None => None,
  }
}

pub fn path_to_string(path: &Path) -> Result<String> {
  // convert a `Path` to a `String`

  let result = path
    .to_path_buf()
    .into_os_string()
    .into_string()
    .map_err(|e| Error::Utf8(e))?;
  Ok(result)
}

pub fn path_buf_to_string(path: PathBuf) -> Result<String> {
  // convert a `PathBuf` to a `String`

  let result = path
    .into_os_string()
    .into_string()
    .map_err(|e| Error::Utf8(e))?;
  Ok(result)
}

pub fn replace_line(path: &str, regexp: &str, replace: &str) -> Result<usize> {
  // replace the pattern `regexp` with `replace` in file at `path`. named back-references may be
  // used. returns the number of lines that were relpaced. data will be written to `path` only if
  // at least one match is found

  let re = Regex::new(regexp)?;
  let mut matched = 0usize;
  let mut output = String::new();
  for line in loads(&path)?.split("\n") { // FIXME
    if re.is_match(line) {
      matched += 1;
      output.push_str(&re.replace_all(line, replace).into_owned());
    } else {
      output.push_str(line);
    };
    output.push_str("\n");
  }
  if matched > 0 { dump(&path, output)?; }
  Ok(matched)
}
