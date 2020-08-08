// thin wrapper for ansible cli

use {
  std::collections::BTreeMap,
  thiserror,
  crate::run_async::{Error as RunError, Runner},
};

#[derive(thiserror::Error, Debug)]
pub enum Error {

  #[error(transparent)]
  LuraRun(#[from] crate::run_async::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Ansible {
  user: Option<String>,
  password: Option<String>,
  escalate: Option<bool>,
  escalate_password: Option<String>,
  inventory: Option<String>,
  extra_vars: BTreeMap<String, String>,
  timeout: Option<usize>,
}

impl Ansible {

  pub fn new() -> Self {
    Self {
      user: None,
      password: None,
      escalate: None,
      escalate_password: None,
      inventory: None,
      extra_vars: BTreeMap::new(),
      timeout: None,
    }
  }

  pub fn user(&mut self, user: &str) -> &mut Self {
    self.user = Some(user.to_owned());
    self
  }

  pub fn password(&mut self, password: &str) -> &mut Self {
    self.password = Some(password.to_owned());
    self
  }

  pub fn escalate(&mut self) -> &mut Self {
    self.escalate = Some(true);
    self
  }

  pub fn escalate_password(&mut self, escalate_password: &str) -> &mut Self {
    self.escalate_password = Some(escalate_password.to_owned());
    self
  }

  pub fn inventory(&mut self, path: &str) -> &mut Self {
    self.inventory = Some(path.to_owned());
    self
  }

  pub fn extra_var(&mut self, k: &str, v: &str) -> &mut Self {

    self.extra_vars.insert(k.to_owned(), v.to_owned());
    self
  }

  pub fn timeout(&mut self, timeout: usize) -> &mut Self {
    self.timeout = Some(timeout);
    self
  }

  fn get_args(&self) -> Vec<String> {
    let mut args = vec![];
    if let Some(user) = &self.user {
      args.push("-u".to_owned());
      args.push(user.to_owned());
    }
    if let Some(true) = &self.escalate {
      args.push("-b".to_owned());
    }
    if let Some(inventory) = &self.inventory {
      args.push("-i".to_owned());
      args.push(inventory.to_owned());
    }
    if let Some(timeout) = self.timeout {
      args.push("-T".to_owned());
      args.push(timeout.to_string())
    }
    if let Some(password) = &self.password {
      args.push("-e".to_owned());
      args.push(format!("ansible_password={}", password)); // FIXME
    }
    if let Some(escalate_password) = &self.escalate_password {
      args.push("-e".to_owned());
      args.push(format!("ansible_become_password={}", escalate_password)); // FIXME
    }
    for (k, v) in &self.extra_vars {
      args.push("-e".to_owned());
      args.push(format!("{}={}", k, v));
    }
    args
  }

  pub async fn playbook(&self, runner: Runner, playbook_path: &str) -> Result<()> {
    let mut args = self.get_args();
    args.insert(0, playbook_path.to_owned());
    let code = &runner.run("ansible-playbook", args.iter()).await?.code();
    if 0.eq(code) {
      Ok(())
    } else {
      Err(Error::from(RunError::UnexpectedExitCode(*code)))
    }
  }

  pub async fn module<I>(&self, runner: Runner, module: &str, module_args: &str) -> Result<()>
  {
    let mut args = self.get_args();
    args.insert(0, "ansible".to_owned());
    args.insert(1, "-m".to_owned());
    args.insert(2, module.to_owned());
    args.insert(3, "-a".to_owned());
    args.insert(4, module_args.to_owned());
    let code = &runner.run("ansible", args.iter()).await?.code();
    if 0.eq(code) {
      Ok(())
    } else {
      Err(Error::from(RunError::UnexpectedExitCode(*code)))
    }
  }
}

