use crate::run;
use tokio::runtime::Runtime;

/////
// build

pub async fn build_async(
  runner: &run::Runner,
  docker: &str,
  target: &str,
  tag: Option<&str>,
) -> Result<run::Output, run::RunError>
{
  let mut args = vec!["build"];
  if let Some(tag) = &tag {
    args.push("-t");
    args.push(tag);
  }
  args.push(target);
  runner.run_async(docker, args).await
}

pub fn build(
  runner: &run::Runner,
  docker: &str,
  target: &str,
  tag: Option<&str>,
) -> Result<run::Output, run::RunError>
{
  Runtime::new()?.block_on(build_async(runner, docker, target, tag))
}

/////
// tag

pub async fn tag_async(
  runner: &run::Runner,
  docker: &str,
  src: &str,
  dst: &str,
) -> Result<run::Output, run::RunError>
{
  runner.run_async(docker, vec!["tag", src, dst]).await
}

pub fn tag(
  runner: &run::Runner,
  docker: &str,
  src: &str,
  dst: &str,
) -> Result<run::Output, run::RunError>
{
  Runtime::new()?.block_on(tag_async(runner, docker, src, dst))
}

/////
// push

pub async fn push_async(
  runner: &run::Runner,
  docker: &str,
  target: &str,
) -> Result<run::Output, run::RunError>
{
  runner.run_async(docker, vec!["push", target]).await
}

pub fn push(
  runner: &run::Runner,
  docker: &str,
  target: &str,
) -> Result<run::Output, run::RunError>
{
  Runtime::new()?.block_on(push_async(runner, docker, target))
}
