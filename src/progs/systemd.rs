use {
  thiserror,
  crate::run::{Runner, runner},
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
    runner().run("systemctl", ["start", &self.0].iter())?;
    Ok(())
  }

  pub fn stop(&self) -> Result<()> {
    runner().run("systemctl", ["stop", &self.0].iter())?;
    Ok(())
  }

  pub fn restart(&self) -> Result<()> {
   runner().run("systemctl", ["restart", &self.0].iter())?;
   Ok(())
  }

  pub fn reload(&self) -> Result<()> {
    runner().run("systemctl", ["reload", &self.0].iter())?;
    Ok(())
  }

  pub fn started(&self) -> Result<bool> {
    Ok(runner()
      .run("systemctl", ["status", &self.0].iter())?
      .code() == 0)
  }

  pub fn stopped(&self) -> Result<bool> {
    Ok(runner()
      .run("systemctl", ["status", &self.0].iter())?
      .code() != 0)
  }

  pub fn journal(&self, lines: usize) -> Result<String> {
    Runner
      ::new()
      .capture(true)
      .enforce(true)
      .run("journalctl", ["-u", &self.0, "-n", &lines.to_string()].iter())?
      .stdout()
      .ok_or(Error::StdoutMissing)
      .map(String::from)
  }
}
