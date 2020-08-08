// thin wrapper for docker cli

use {
  thiserror,
  crate::run_async,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
  
  #[error(transparent)]
  LuraRunAsync(#[from] crate::run_async::Error),
}

pub async fn build(
  runner: &run_async::Runner,
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
  runner.run("docker", args).await?;
  Ok(())
}

pub async fn tag(
  runner: &run_async::Runner,
  src: &str,
  dst: &str,
) -> Result<(), Error>
{
  runner.run("docker", vec!["tag", src, dst]).await?;
  Ok(())
}

pub async fn push(
  runner: &run_async::Runner,
  target: &str,
) -> Result<(), Error>
{
  runner.run("docker", vec!["push", target]).await?;
  Ok(())
}
