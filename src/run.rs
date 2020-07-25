use std::{
  collections::BTreeMap,
  ffi::{OsStr, OsString},
  process::Stdio,
};
use thiserror;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process;
use tokio::task;
use crate::runtime::tokio::block_on_local;

const _NOARGS: Vec<OsString> = Vec::new();
pub const NOARGS: &Vec<OsString> = &_NOARGS;

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

  #[error(transparent)]
  IOError(#[from] std::io::Error),

  #[error(transparent)]
  TokioTaskJoinError(#[from] tokio::task::JoinError),

  #[error(transparent)]
  RuntimeError(#[from] crate::runtime::Error),
}

/////
// Output, the result of running a command

#[derive(Debug)]
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

  pub async fn run_async<I, S>(&self, bin: &str, args: I) -> Result<Output, Error>
  where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
  {
    let mut command = process::Command::new(bin);
    if let Some(cwd) = &self.cwd {
      command.current_dir(cwd);
    }
    if self.env_clear {
      command.env_clear();
    }
    for var in &self.env_remove {
      command.env_remove(var);
    }
    let mut child = command
      .args(args)
      .envs(&self.env)
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()?;
    let stdout_fd = child.stdout.take().ok_or(Error::StdioHandleMissing("stdout"))?;
    let stderr_fd = child.stderr.take().ok_or(Error::StdioHandleMissing("stderr"))?;
    let stdout_future = task::spawn(read_stdout(stdout_fd, self.receive_stdout.clone(), self.capture));
    let stderr_future = task::spawn(read_stderr(stderr_fd, self.receive_stderr.clone(), self.capture));
    let code = child.await?.code().ok_or(Error::ExitCodeMissing)?;
    let stdout = stdout_future.await??;
    let stderr = stderr_future.await??;
    match self.enforce_code {
      Some(enforce_code) if enforce_code != code => Err(Error::UnexpectedExitCode(code)),
      _ => Ok(Output::new(code, stdout, stderr)),
    }
  }

  pub fn run<I, S>(&self, bin: &str, args: I) -> Result<Output, Error>
  where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
  {
    block_on_local(self.run_async(bin, args))?
  }
}

async fn read_stdout(
  fd: process::ChildStdout,
  receivers: Vec<fn(&str)>,
  capture: bool
) -> Result<Option<String>, Error>
{
  let mut lines = BufReader::new(fd).lines();
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
  fd: process::ChildStderr,
  receivers: Vec<fn(&str)>,
  capture: bool
) -> Result<Option<String>, Error>
{
  let mut lines = BufReader::new(fd).lines();
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

#[cfg(test)]
mod tests {

  use crate::run::{Runner, NOARGS};

  #[test]
  fn test_stdout() {

    fn receive_stdout(line: &str) {
      assert_eq!(line, "hello test");
    }
    
    fn receive_stderr(_line: &str) {
      assert!(false);
    }

    let mut runner = Runner::new();
    let output = runner
      .receive_stdout(receive_stdout)
      .receive_stderr(receive_stderr)
      .capture()
      .enforce()
      .run("echo", vec!["hello", "test"])
      .unwrap();
    assert_eq!(output.code, 0);
    assert_eq!(output.stdout.unwrap(), "hello test\n");
    assert_eq!(output.stderr.unwrap(), "");
  }

  #[test]
  fn test_stderr() {

    fn receive_stdout(_line: &str) {
      assert!(false);
    }
    
    fn receive_stderr(line: &str) {
      assert_eq!(line, "hello test");
    }

    let mut runner = Runner::new();
    let output = runner
      .receive_stdout(receive_stdout)
      .receive_stderr(receive_stderr)
      .capture()
      .enforce()
      .run("sh", vec!["-c", "echo hello test >&2"])
      .unwrap();
    assert_eq!(output.code, 0);
    assert_eq!(output.stdout.unwrap(), "");
    assert_eq!(output.stderr.unwrap(), "hello test\n");
  }

  #[test]
  #[should_panic]
  fn test_enforce() {
    let mut runner = Runner::new();
    runner
      .enforce()
      .run("/bin/false", NOARGS)
      .unwrap();
  }

  #[test]
  fn test_reuse() {
    let mut runner = Runner::new();
    runner.run("/bin/false", NOARGS).unwrap();
    runner.enforce();
    runner.run("/bin/true", NOARGS).unwrap();
  }
}
