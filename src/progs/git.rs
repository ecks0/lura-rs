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
// clone

pub async fn clone_async(
  runner: &run::Runner,
  repo: &str,
  branch: Option<&str>,
  dst: Option<&str>,
) -> Result<run::Output, Error>
{
  let mut args = vec!["clone"];
  if let Some(branch) = branch {
    args.push("-b");
    args.push(branch);
  }
  args.push(repo);
  if let Some(dst) = dst {
    args.push(dst);
  }
  Ok(runner.run_async("git", args).await?)
}

pub fn clone(
  runner: &run::Runner,
  repo: &str,
  branch: Option<&str>,
  dst: Option<&str>,
) -> Result<run::Output, Error>
{
  block_on_local(clone_async(runner, repo, branch, dst))?
}
