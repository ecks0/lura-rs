// subprocess executor
//
// `Runner` is a builder similar to `std::process::Command`
//
// - `Runner` can be configured once and used many times
// - `Runner` can automatically error on unexpected exit code
// - `Runner` can read stdio automatically using either threads or tasks
// - `Runner` can dispatch lines as they are read from stdout/stderr to callback functions
// - `Runner` can execute either blocking or async

use {
  log::{error, info},
  std::{
    collections::BTreeMap,
    ffi::{OsStr, OsString},
    process::Stdio,
  },
  thiserror,
  tokio,
  tokio::io::{AsyncBufReadExt, BufReader as AsyncBufReader},
  which::which,
};

/////
// Error

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

  #[error(transparent)]
  TokioRuntimeTaskJoin(#[from] tokio::task::JoinError),
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

/////
// Runner, subproceses executor

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

  pub fn cwd(&mut self, cwd: Option<&str>) -> &mut Self {
    // set the current working directory

    self.cwd = cwd.map(|i| i.to_owned());
    self
  }

  pub fn env_clear(&mut self) -> &mut Self {
    // clear environment variables

    self.env_clear = true;
    self
  }

  pub fn env_remove<S: AsRef<OsStr>>(&mut self, name: S) {
    // remove an environment variable

    self.env_remove.push(name.as_ref().to_os_string());
  }

  pub fn env<I, K, V>(&mut self, vars: I) -> &mut Self
  where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<OsStr>,
    V: AsRef<OsStr>,
  {
    // specify environment variables

    for (ref key, ref val) in vars {
      self.env.insert(key.as_ref().to_os_string(), val.as_ref().to_os_string());
    }
    self
  }

  pub fn receive_stdout(&mut self, callback: fn(&str)) -> &mut Self {
    // add a callback to receive stdout lines

    self.receive_stdout.push(callback);
    self
  }

  pub fn receive_stderr(&mut self, callback: fn(&str)) -> &mut Self {
    // add a callback to receive stderr lines

    self.receive_stderr.push(callback);
    self
  }

  pub fn enforce_code(&mut self, code: Option<i32>) -> &mut Self {
    // return an error if an exit code does not match `code`

    self.enforce_code = code;
    self
  }

  pub fn enforce(&mut self, value: bool) -> &mut Self {
    // return an error if an exit code is not 0

    self.enforce_code = if value == true { Some(0i32) } else { None };
    self
  }

  pub fn capture(&mut self, value: bool) -> &mut Self {
    // capture stdout and stderr and return them on the `Output` result

    self.capture = value;
    self
  }

  pub async fn run<I, S>(&self, bin: &str, args: I) -> Result<Output>
  where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
  {
    // run the command `bin` with arguments `args`

    let mut command = tokio::process::Command::new(bin);
    if let Some(cwd) = &self.cwd { command.current_dir(cwd); }
    if self.env_clear { command.env_clear(); }
    for var in &self.env_remove { command.env_remove(var); }
    let mut child = command
      .args(args)
      .envs(&self.env)
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .stdin(Stdio::null())
      .spawn()?;
    let stdout_fd = child.stdout.take().ok_or(Error::StdioHandleMissing("stdout"))?;
    let stderr_fd = child.stderr.take().ok_or(Error::StdioHandleMissing("stderr"))?;
    let stdout = tokio::task::spawn(read_stdout(stdout_fd, self.receive_stdout.clone(), self.capture)).await??;
    let stderr = tokio::task::spawn(read_stderr(stderr_fd, self.receive_stderr.clone(), self.capture)).await??;
    let code = child.await?.code().ok_or(Error::ExitCodeMissing)?;
    match self.enforce_code {
      Some(enforce_code) if enforce_code != code => Err(Error::UnexpectedExitCode(code)),
      _ => Ok(Output::new(code, stdout, stderr)),
    }
  }
}

/////
// stdio loops

async fn read_stdout(
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

async fn read_stderr(
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
  // return a new `Runner` with the following default configuration
  //
  // - enforce exit code 0
  // - send stdout and stderr lines to the log with level `info`

  fn log_stdout(line: &str) { info!(target: "lura::run [out]", "{}", line); }
  fn log_stderr(line: &str) { info!(target: "lura::run [err]", "{}", line); }

  let mut runner = Runner::new();
  runner
    .enforce(true)
    .receive_stdout(log_stdout)
    .receive_stderr(log_stderr);
  runner
}

pub async fn run<I, S>(bin: &str, args: I) -> Result<Output>
where
  I: IntoIterator<Item = S>,
  S: AsRef<OsStr>,
{
  // run a command using a `Runner` with the following configuration
  //
  // - enforce exit code 0
  // - send stdout and stderr lines to the log with level `info`

  Ok(runner().capture(true).run(bin, args).await?)
}

pub async fn sh(contents: &str) -> Result<Output> {
  // run a shell command using a `Runner` with the following configuration
  //
  // - enforce exit code 0
  // - send stdout and stderr lines to the log with level `info`

  for shell in ["bash", "sh"].iter() { // FIXME
    if let Ok(_) = which(shell) {
      return Ok(runner().capture(true).run(shell, ["-c", contents].iter()).await?);
    }
  }
  Err(Error::ShellMissing)
}
