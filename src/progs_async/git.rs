// thin wrapper for git cli

use {
  thiserror,
  crate::run_async,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
  
  #[error(transparent)]
  LuraRunAsync(#[from] crate::run_async::Error),
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

  pub fn new(url: &str) -> Self {
    Self {
      url: url.to_owned(),
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

  fn run_args(&self) -> Vec<&str> {
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
    args
  }

  pub async fn run(&self, runner: &run_async::Runner) -> Result<()> {
    runner.run("git", self.run_args()).await?;
    Ok(())
  }
}
