use {
  log::debug,
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

const MOD: &str = std::module_path!();

#[derive(thiserror::Error, Debug)]
pub enum Error {

  #[error("Utf8 conversion error")]
  Utf8(OsString),

  #[error(transparent)]
  Io(#[from] std::io::Error),

  #[error(transparent)]
  Regex(#[from] regex::Error),
  
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

pub fn replace_line<P: AsRef<Path>>(path: P, regexp: &str, replace: &str) -> Result<usize> {
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

#[cfg(feature = "lua")]
use {
  rlua::{ Context, Error as LuaError, Result as LuaResult, Table },
  std::sync::Arc,
};


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

  fs.set("mkdir", ctx.create_function(|_, args: (String,)| Ok(mkdir(&args.0)?))?)?;
  fs.set("chmod", ctx.create_function(|_, args: (String, u32)| Ok(chmod(&args.0, args.1)?))?)?;
  fs.set("rm", ctx.create_function(|_, args: (String,)| Ok(rm(&args.0)?))?)?;
  fs.set("loads", ctx.create_function(|_, args: (String,)| Ok(loads(&args.0)?))?)?;
  fs.set("dump", ctx.create_function(|_, args: (String, String)| Ok(dump(&args.0, &args.1)?))?)?;
  fs.set("replace_line", ctx.create_function(|_, args: (String, String, String)| {
    Ok(replace_line(&args.0, &args.1, &args.2)?)
  })?)?;
  fs.set("tempdir", ctx.create_function(|_, args: (String,)| Ok(tempdir(&args.0)?))?)?;

  ctx
    .globals()
    .get::<_, Table>("lura")?
    .set("fs", fs)?;

  Ok(())
}
