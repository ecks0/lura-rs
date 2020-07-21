use thiserror;
use crate::{
  run,
  runtime::tokio::block_on_local,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
  
  #[error(transparent)]
  RunError(#[from] crate::run::Error),

  #[error(transparent)]
  RuntimeTokioError(#[from] crate::runtime::Error),
}

/////
// build

pub async fn build_async(
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
  runner.run_async("docker", args).await?;
  Ok(())
}

pub fn build(
  runner: &run::Runner,
  target: &str,
  tag: Option<&str>,
) -> Result<(), Error>
{
  block_on_local(build_async(runner, target, tag))?
}

/////
// tag

pub async fn tag_async(
  runner: &run::Runner,
  src: &str,
  dst: &str,
) -> Result<(), Error>
{
  runner.run_async("docker", vec!["tag", src, dst]).await?;
  Ok(())
}

pub fn tag(
  runner: &run::Runner,
  src: &str,
  dst: &str,
) -> Result<(), Error>
{
  block_on_local(tag_async(runner, src, dst))?
}

/////
// push

pub async fn push_async(
  runner: &run::Runner,
  target: &str,
) -> Result<(), Error>
{
  runner.run_async("docker", vec!["push", target]).await?;
  Ok(())
}

pub fn push(
  runner: &run::Runner,
  target: &str,
) -> Result<(), Error>
{
  block_on_local(push_async(runner, target))?
}
