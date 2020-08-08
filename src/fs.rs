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
  
  #[error(transparent)]
  LuraRun(#[from] crate::run::Error),
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

pub fn mkdir<P: AsRef<Path>>(path: P) -> Result<()> {
  // create a directory

  Ok(std::fs::create_dir(&path)?)
}

pub fn chmod<P: AsRef<Path>>(path: P, mode: u32) -> Result<()> {
  // set permissions for a path

  Ok(set_permissions(path, Permissions::from_mode(mode))?)
}

pub fn mv<P: AsRef<Path>>(src: P, dst: P) -> Result<()> {
  // move a file or directory recursively

  let src = &path_to_string(src.as_ref())?;
  let dst = &path_to_string(dst.as_ref())?;
  crate::run::run("mv", ["-f", src, dst].iter())?; // FIXME
  Ok(())
}

pub fn rm<P: AsRef<Path>>(path: P) -> Result<()> {
  // remove a path. directories are removed recursively - be careful

  Ok(if path.as_ref().is_dir() {
    std::fs::remove_dir_all(path)?
  } else {
     std::fs::remove_file(path)?
  })
}

pub fn is_file<P: AsRef<Path>>(path: P) -> bool {
  path.as_ref().is_file()
}

pub fn is_dir<P: AsRef<Path>>(path: P) -> bool {
  path.as_ref().is_dir()
}

pub fn load<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
  // load data from a file as bytes

  Ok(std::fs::read(path)?)
}


pub fn loads<P: AsRef<Path>>(path: P) -> Result<String> {
  // load data from a file as `String`

  Ok(std::fs::read_to_string(path)?)
}

pub fn dump<P: AsRef<Path>, D: AsRef<[u8]>>(path: P, data: D) -> Result<()> {
  // write data to a file

  Ok(std::fs::write(path, data)?)
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

pub fn replace_line<P: AsRef<Path>>(path: P, regexp: &str, replace: &str) -> Result<usize> {
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

#[cfg(feature = "lua")]
use {
  log::debug,
  rlua::{ Context, Error as LuaError, Result as LuaResult, Table },
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
pub(crate) fn lua_init(ctx: &Context) -> LuaResult<()> {

  debug!(target: MOD, "Lua init");

  let fs = ctx.create_table()?;

  fs.set("tempdir", ctx.create_function(|_, args: (String,)| Ok(tempdir(&args.0)?))?)?;
  fs.set("mkdir", ctx.create_function(|_, args: (String,)| Ok(mkdir(&args.0)?))?)?;
  fs.set("chmod", ctx.create_function(|_, args: (String, u32)| Ok(chmod(&args.0, args.1)?))?)?;
  fs.set("mv", ctx.create_function(|_, args: (String, String)| Ok(mv(&args.0, &args.1)?))?)?;
  fs.set("rm", ctx.create_function(|_, args: (String,)| Ok(rm(&args.0)?))?)?;
  fs.set("loads", ctx.create_function(|_, args: (String,)| Ok(loads(&args.0)?))?)?;
  fs.set("dump", ctx.create_function(|_, args: (String, String)| Ok(dump(&args.0, &args.1)?))?)?;
  fs.set("replace_line", ctx.create_function(|_, args: (String, String, String)| {
    Ok(replace_line(&args.0, &args.1, &args.2)?)
  })?)?;

  ctx
    .globals()
    .get::<_, Table>("lura")?
    .set("fs", fs)?;

  Ok(())
}
