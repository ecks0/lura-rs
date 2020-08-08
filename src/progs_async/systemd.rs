use {
  thiserror,
  crate::run_async::{Runner, runner},
};

#[derive(thiserror::Error, Debug)]
pub enum Error {

  #[error("Runner did not capture stdout")]
  StdoutMissing,

  #[error(transparent)]
  LuraRunAsync(#[from] crate::run_async::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Service(String);

impl Service {

  pub fn new(name: &str) -> Self {
    Self(name.to_owned())
  }

  pub async fn start(&self) -> Result<()> {
    runner().run("systemctl", ["start", &self.0].iter()).await?;
    Ok(())
  }

  pub async fn stop(&self) -> Result<()> {
    runner().run("systemctl", ["stop", &self.0].iter()).await?;
    Ok(())
  }

  pub async fn restart(&self) -> Result<()> {
   runner().run("systemctl", ["restart", &self.0].iter()).await?;
   Ok(())
  }

  pub async fn reload(&self) -> Result<()> {
    runner().run("systemctl", ["reload", &self.0].iter()).await?;
    Ok(())
  }

  pub async fn started(&self) -> Result<bool> {
    Ok(runner()
      .run("systemctl", ["status", &self.0].iter()).await?
      .code() == 0)
  }

  pub async fn stopped(&self) -> Result<bool> {
    Ok(runner()
      .run("systemctl", ["status", &self.0].iter()).await?
      .code() != 0)
  }

  pub async fn journal(&self, lines: usize) -> Result<String> {
    Runner
      ::new()
      .capture()
      .enforce()
      .run("journalctl", ["-u", &self.0, "-n", &lines.to_string()].iter()).await?
      .stdout()
      .ok_or(Error::StdoutMissing)
      .map(String::from)
  }
}
