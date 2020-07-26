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

type Result<T> = std::result::Result<T, Error>;

/////
// clone

#[derive(Debug)]
pub struct Clone {
  url: String,
  branch: Option<String>,
  single_branch: Option<bool>,
  path: Option<String>,
}

impl Clone {

  pub fn new(repo_url: &str) -> Self {
    Self {
      url: repo_url.to_owned(),
      branch: None,
      single_branch: None,
      path: None,
    }
  }

  pub fn branch(&mut self, branch: &str) -> &mut Self {
    self.branch = Some(branch.to_owned());
    self
  }

  pub fn single_branch(&mut self, single_branch: bool) -> &mut Self {
    self.single_branch = Some(single_branch);
    self
  }

  pub fn path(&mut self, path: &str) -> &mut Self {
    self.path = Some(path.to_owned());
    self
  }

  pub async fn run_async(&self, runner: &run::Runner) -> Result<()> {
    let mut args = vec!["clone"];
    if let Some(branch) = &self.branch {
      args.push("-b");
      args.push(branch);
    }
    if let Some(single_branch) = self.single_branch {
      match single_branch {
        true => args.push("--single-branch"),
        false => args.push("--no-single-branch"),
      };
    }
    args.push(&self.url);
    if let Some(path) = &self.path {
      args.push(path);
    }
    runner.run("git", args)?;
    Ok(())
  }

  pub fn run(&mut self, runner: &run::Runner) -> Result<()> {
    block_on_local(self.run_async(runner))??;
    Ok(())
  }
}