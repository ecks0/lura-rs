use {
  anyhow,
  log::{debug, error, info},
  std::{
    collections::BTreeMap,
    ffi::{OsStr, OsString},
    io::{BufRead, BufReader},
    process::Stdio,
    sync::mpsc,
    thread,
  },
  thiserror,
  which::which,
};

#[cfg(feature = "async")]
use {
  tokio,
  tokio::io::{AsyncBufReadExt, BufReader as AsyncBufReader},
};

const MOD: &str = std::module_path!();

/////
// Errors

#[derive(thiserror::Error, Debug)]
pub enum Error {

  #[error("Command exited with unexpected status code `{0}`")]
  UnexpectedExitCode(i32),

  #[error("Child process returned no exit code")]
  ExitCodeMissing,

  #[error("Child process missing stdio file handle: `{0}`")]
  StdioHandleMissing(&'static str),

  #[error("bash nor sh were found in $PATH")]
  ShellMissing,

  #[error(transparent)]
  StdIo(#[from] std::io::Error),

  #[error("Failed to join stdio thread: `{0}`")]
  StdioJoin(&'static str),

  #[error(transparent)]
  StdSyncMpscRecv(#[from] std::sync::mpsc::RecvError),

  #[cfg(feature = "async")]
  #[error(transparent)]
  TokioRuntimeTaskJoin(#[from] tokio::task::JoinError),

  // used for mpsc send errors
  #[error(transparent)]
  Any(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

/////
// Output, the result of running a command

#[derive(Clone, Debug)]
pub struct Output {
  code: i32,
  pub stdout: Option<String>,
  pub stderr: Option<String>,
}

impl Output {

  fn new(code: i32, stdout: Option<String>, stderr: Option<String>) -> Self {
    Self { code, stdout, stderr }
  }

  pub fn code(&self) -> i32 { self.code }

  pub fn zero(&self) -> bool { self.code == 0 }

  pub fn stdout(&self) -> Option<&str> { self.stdout.as_deref() }

  pub fn stderr(&self) -> Option<&str> { self.stderr.as_deref() }
}

pub struct Runner {
  cwd: Option<String>,
  env_clear: bool,
  env_remove: Vec<OsString>,
  env: BTreeMap<OsString, OsString>,
  receive_stdout: Vec<fn(&str)>,
  receive_stderr: Vec<fn(&str)>,
  enforce_code: Option<i32>,
  capture: bool,
}

/////
// Runner, run commands using `process::Command`

impl Runner {

  pub fn new() -> Self {
    Self {
      cwd: None,
      env_clear: false,
      env_remove: Vec::new(),
      env: BTreeMap::new(),
      receive_stdout: Vec::new(),
      receive_stderr: Vec::new(),
      enforce_code: None,
      capture: false,
    }
  }

  pub fn cwd(&mut self, cwd: &str) -> &mut Self {
    self.cwd = Some(cwd.to_owned());
    self
  }

  pub fn env_clear(&mut self) -> &mut Self {
    self.env_clear = true;
    self
  }

  pub fn env_remove<S: AsRef<OsStr>>(&mut self, name: S) {
    self.env_remove.push(name.as_ref().to_os_string());
  }

  pub fn env<I, K, V>(&mut self, vars: I) -> &mut Self
  where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<OsStr>,
    V: AsRef<OsStr>,
  {
    for (ref key, ref val) in vars {
      self.env.insert(key.as_ref().to_os_string(), val.as_ref().to_os_string());
    }
    self
  }

  pub fn receive_stdout(&mut self, callback: fn(&str)) -> &mut Self {
    self.receive_stdout.push(callback);
    self
  }

  pub fn receive_stderr(&mut self, callback: fn(&str)) -> &mut Self {
    self.receive_stderr.push(callback);
    self
  }

  pub fn enforce_code(&mut self, code: i32) -> &mut Self {
    self.enforce_code = Some(code);
    self
  }

  pub fn enforce(&mut self) -> &mut Self {
    self.enforce_code = Some(0i32);
    self
  }

  pub fn capture(&mut self) -> &mut Self {
    self.capture = true;
    self
  }

  pub fn run<I, S>(&self, bin: &str, args: I) -> Result<Output>
  where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
  {
    let mut command = std::process::Command::new(bin);
    if let Some(cwd) = &self.cwd { command.current_dir(cwd); }
    if self.env_clear { command.env_clear(); }
    for var in &self.env_remove { command.env_remove(var); }
    let mut child = command
      .args(args)
      .envs(&self.env)
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()?;
    let stdout_fd = child.stdout.take().ok_or(Error::StdioHandleMissing("stdout"))?;
    let stderr_fd = child.stderr.take().ok_or(Error::StdioHandleMissing("stderr"))?;
    let (out_tx, out_rx) = mpsc::channel();
    let (err_tx, err_rx) = mpsc::channel();
    let (receive_stdout, receive_stderr) = (self.receive_stdout.clone(), self.receive_stderr.clone());
    let capture = self.capture;
    let stdout_thread = thread::spawn(move || -> std::result::Result<(), anyhow::Error> {
      Ok(out_tx.send(read_stdout_thread(stdout_fd, receive_stdout, capture)?)?)
    });
    let stderr_thread = thread::spawn(move || -> std::result::Result<(), anyhow::Error> {
      Ok(err_tx.send(read_stderr_thread(stderr_fd, receive_stderr, capture)?)?)
    });
    stdout_thread.join().map_err(|_| Error::StdioJoin("stdout"))??;
    stderr_thread.join().map_err(|_| Error::StdioJoin("stderr"))??;
    let stdout = out_rx.recv()?;
    let stderr = err_rx.recv()?;
    let code = child.wait()?.code().ok_or(Error::ExitCodeMissing)?;
    match self.enforce_code {
      Some(enforce_code) if enforce_code != code => Err(Error::UnexpectedExitCode(code)),
      _ => Ok(Output::new(code, stdout, stderr)),
    }  
  }

  #[cfg(feature = "async")]
  pub async fn run_async<I, S>(&self, bin: &str, args: I) -> Result<Output>
  where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
  {
    let mut command = tokio::process::Command::new(bin);
    if let Some(cwd) = &self.cwd { command.current_dir(cwd); }
    if self.env_clear { command.env_clear(); }
    for var in &self.env_remove { command.env_remove(var); }
    let mut child = command
      .args(args)
      .envs(&self.env)
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()?;
    let stdout_fd = child.stdout.take().ok_or(Error::StdioHandleMissing("stdout"))?;
    let stderr_fd = child.stderr.take().ok_or(Error::StdioHandleMissing("stderr"))?;
    let stdout_future = tokio::task::spawn(read_stdout_async(stdout_fd, self.receive_stdout.clone(), self.capture));
    let stderr_future = tokio::task::spawn(read_stderr_async(stderr_fd, self.receive_stderr.clone(), self.capture));
    let stdout = stdout_future.await??;
    let stderr = stderr_future.await??;
    let code = child.await?.code().ok_or(Error::ExitCodeMissing)?;
    match self.enforce_code {
      Some(enforce_code) if enforce_code != code => Err(Error::UnexpectedExitCode(code)),
      _ => Ok(Output::new(code, stdout, stderr)),
    }
  }
}

/////
// stdio readers

fn read_stdout_thread(
  fd: std::process::ChildStdout,
  receivers: Vec<fn(&str)>,
  capture: bool,
) -> Result<Option<String>>
{
  let mut lines = BufReader::new(fd).lines();
  if capture {
    let mut buf = String::new();
    while let Some(line) = lines.next() {
      if let Ok(line) = &line {
        buf.push_str(line);
        buf.push_str("\n");
        receivers.iter().for_each(|receiver| receiver(line));
      }
    }
    Ok(Some(buf))
  } else {
    while let Some(line) = lines.next() {
      if let Ok(line) = &line {
        receivers.iter().for_each(|receiver| receiver(line));
      }
    }
    Ok(None)
  }
}

fn read_stderr_thread(
  fd: std::process::ChildStderr,
  receivers: Vec<fn(&str)>,
  capture: bool
) -> Result<Option<String>>
{
  let mut lines = BufReader::new(fd).lines();
  if capture {
    let mut buf = String::new();
    while let Some(line) = lines.next() {
      if let Ok(line) = &line {
        buf.push_str(line);
        buf.push_str("\n");
        receivers.iter().for_each(|receiver| receiver(line));
      }
    }
    Ok(Some(buf))
  } else {
    while let Some(line) = lines.next() {
      if let Ok(line) = &line {
        receivers.iter().for_each(|receiver| receiver(line));
      }
    }
    Ok(None)
  }
}

#[cfg(feature = "async")]
async fn read_stdout_async(
  fd: tokio::process::ChildStdout,
  receivers: Vec<fn(&str)>,
  capture: bool
) -> Result<Option<String>>
{
  let mut lines = AsyncBufReader::new(fd).lines();
  if capture {
    let mut buf = String::new();
    while let Some(line) = lines.next_line().await? {
      buf.push_str(&line);
      buf.push_str("\n");
      receivers.iter().for_each(|receiver| receiver(&line));
    }
    Ok(Some(buf))
  } else {
    while let Some(line) = lines.next_line().await? {
      receivers.iter().for_each(|receiver| receiver(&line));
    }
    Ok(None)
  }
}

#[cfg(feature = "async")]
async fn read_stderr_async(
  fd: tokio::process::ChildStderr,
  receivers: Vec<fn(&str)>,
  capture: bool
) -> Result<Option<String>>
{
  let mut lines = AsyncBufReader::new(fd).lines();
  if capture {
    let mut buf = String::new();
    while let Some(line) = lines.next_line().await? {
      buf.push_str(&line);
      buf.push_str("\n");
      receivers.iter().for_each(|receiver| receiver(&line));
    }
    Ok(Some(buf))
  } else {
    while let Some(line) = lines.next_line().await? {
      receivers.iter().for_each(|receiver| receiver(&line));
    }
    Ok(None)
  }
}

/////
// utility functions

pub fn runner() -> Runner {

  fn log_stdout(line: &str) { info!(target: "lura::run [out]", "{}", line); }
  fn log_stderr(line: &str) { info!(target: "lura::run [err]", "{}", line); }

  let mut runner = Runner::new();
  runner
    .enforce()
    .receive_stdout(log_stdout)
    .receive_stderr(log_stderr);
  runner
}

pub fn run<I, S>(bin: &str, args: I) -> Result<Output>
where
  I: IntoIterator<Item = S>,
  S: AsRef<OsStr>,
{
  Ok(runner().capture().run(bin, args)?)
}

#[cfg(feature = "async")]
pub async fn run_async<I, S>(bin: &str, args: I) -> Result<Output>
where
  I: IntoIterator<Item = S>,
  S: AsRef<OsStr>,
{
  Ok(runner().capture().run_async(bin, args).await?)
}

pub fn sh(contents: &str) -> Result<Output> {

  for shell in ["bash", "sh"].iter() { // FIXME
    if let Ok(_) = which(shell) {
      return Ok(runner().capture().run(shell, ["-c", contents].iter())?);
    }
  }
  Err(Error::ShellMissing)
}

#[cfg(feature = "async")]
pub async fn sh_async(contents: &str) -> Result<Output> {

  for shell in ["bash", "sh"].iter() { // FIXME
    if let Ok(_) = which(shell) {
      return Ok(runner().capture().run_async(shell, ["-c", contents].iter()).await?);
    }
  }
  Err(Error::ShellMissing)
}

/////
// lua support

#[cfg(feature = "lua")]
use {
  rlua::{ Context, Error as LuaError, Result as LuaResult, UserData, Table },
  std::sync::Arc,
};

#[cfg(feature = "lua")]
impl From<Error> for LuaError {
  fn from(err: Error) -> LuaError {
    LuaError::ExternalError(Arc::new(err))
  }
}

#[cfg(feature = "lua")]
impl UserData for Output {
  fn add_methods<'lua, T: rlua::UserDataMethods<'lua, Self>>(methods: &mut T) {
    methods.add_method("code", |_, this, _: ()| { Ok(this.code()) });
    methods.add_method("zero", |_, this, _: ()| { Ok(this.zero()) });
    methods.add_method("stdout", |_, this, _: () | { Ok(this.stdout().unwrap_or("").to_owned()) });
    methods.add_method("stderr", |_, this, _: () | { Ok(this.stderr().unwrap_or("").to_owned()) });
  }
}

#[cfg(feature = "lua")]
pub(crate) fn lua_init(ctx: &Context) -> LuaResult<()> {
 
  debug!(target: MOD, "Lua init");

  let run_ = ctx.create_table()?;

  run_.set("run", ctx.create_function(|_, args: (String, Vec<String>)| {
    Ok(run(&args.0, args.1.iter())?)
  })?)?;
  run_.set("sh", ctx.create_function(|_, args: (String,)| {
    Ok(sh(&args.0)?)
  })?)?;

  ctx
    .globals()
    .get::<_, Table>("lura")?
    .set("run", run_)?;

  Ok(())
}
