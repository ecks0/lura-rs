use {
  thiserror,
  crate::run::{Runner, drunner},
};

#[derive(thiserror::Error, Debug)]
pub enum Error {

  #[error("Runner did not capture stdout")]
  StdoutMissing,

  #[error(transparent)]
  LuraRun(#[from] crate::run::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Service(String);

impl Service {

  pub fn new(name: &str) -> Self {
    Self(name.to_owned())
  }

  pub fn start(&self) -> Result<()> {
    drunner().run("systemctl", ["start", &self.0].iter())?;
    Ok(())
  }

  pub fn stop(&self) -> Result<()> {
    drunner().run("systemctl", ["stop", &self.0].iter())?;
    Ok(())
  }

  pub fn restart(&self) -> Result<()> {
   drunner().run("systemctl", ["restart", &self.0].iter())?;
   Ok(())
  }

  pub fn reload(&self) -> Result<()> {
    drunner().run("systemctl", ["reload", &self.0].iter())?;
    Ok(())
  }

  pub fn started(&self) -> Result<bool> {
    Ok(drunner()
      .run("systemctl", ["status", &self.0].iter())?
      .code() == 0)
  }

  pub fn stopped(&self) -> Result<bool> {
    Ok(drunner()
      .run("systemctl", ["status", &self.0].iter())?
      .code() != 0)
  }

  pub fn journal(&self, lines: usize) -> Result<String> {
    Runner
      ::new()
      .capture()
      .enforce()
      .run("journalctl", ["-u", &self.0, "-n", &lines.to_string()].iter())?
      .stdout()
      .ok_or(Error::StdoutMissing)
      .map(String::from)
  }
}

#[cfg(feature = "async")]
#[derive(Debug)]
pub struct AsyncService(String);

#[cfg(feature = "async")]
impl AsyncService {

  pub fn new(name: &str) -> Self {
    Self(name.to_owned())
  }

  pub async fn start(&self) -> Result<()> {
    drunner().run_async("systemctl", ["start", &self.0].iter()).await?;
    Ok(())
  }

  pub async fn stop(&self) -> Result<()> {
    drunner().run_async("systemctl", ["stop", &self.0].iter()).await?;
    Ok(())
  }

  pub async fn restart(&self) -> Result<()> {
   drunner().run_async("systemctl", ["restart", &self.0].iter()).await?;
   Ok(())
  }

  pub async fn reload(&self) -> Result<()> {
    drunner().run_async("systemctl", ["reload", &self.0].iter()).await?;
    Ok(())
  }

  pub async fn started(&self) -> Result<bool> {
    Ok(drunner()
      .run_async("systemctl", ["status", &self.0].iter()).await?
      .code() == 0)
  }

  pub async fn stopped(&self) -> Result<bool> {
    Ok(drunner()
      .run_async("systemctl", ["status", &self.0].iter()).await?
      .code() != 0)
  }

  pub async fn journal(&self, lines: usize) -> Result<String> {
    Runner
      ::new()
      .capture()
      .enforce()
      .run_async("journalctl", ["-u", &self.0, "-n", &lines.to_string()].iter()).await?
      .stdout()
      .ok_or(Error::StdoutMissing)
      .map(String::from)
  }
}
