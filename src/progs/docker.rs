// thin wrapper for docker cli

use {
  thiserror,
  crate::run,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
  
  #[error(transparent)]
  RunError(#[from] crate::run::Error),
}

pub fn build(
  runner: &run::Runner,
  target: &str,
  tag: Option<&str>,
) -> Result<(), Error>
{
  let mut args = vec!["build"];
  if let Some(tag) = &tag {
    args.push("-t");
    args.push(tag);
  }
  args.push(target);
  runner.run("docker", args)?;
  Ok(())
}

pub fn tag(
  runner: &run::Runner,
  src: &str,
  dst: &str,
) -> Result<(), Error>
{
  runner.run("docker", vec!["tag", src, dst])?;
  Ok(())
}

pub fn push(
  runner: &run::Runner,
  target: &str,
) -> Result<(), Error>
{
  runner.run("docker", vec!["push", target])?;
  Ok(())
}
