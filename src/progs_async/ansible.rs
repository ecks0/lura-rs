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

pub struct Ansible {
  runner: Runner,
  user: Option<String>,
  password: Option<String>,
  escalate: Option<bool>,
  escalate_password: Option<String>,
  inventory: Option<String>,
  extra_vars: BTreeMap<String, String>,
  timeout: Option<usize>,
  check: Option<bool>,
  diff: Option<bool>,
}

impl Ansible {

  pub fn new(runner: Runner) -> Self {
    Self {
      runner: runner,
      user: None,
      password: None,
      escalate: None,
      escalate_password: None,
      inventory: None,
      extra_vars: BTreeMap::new(),
      timeout: None,
      check: None,
      diff: None,
    }
  }

  pub fn user(&mut self, user: Option<String>) -> &mut Self {
    self.user = user;
    self
  }

  pub fn password(&mut self, password: Option<String>) -> &mut Self {
    self.password = password;
    self
  }

  pub fn escalate(&mut self, value: bool) -> &mut Self {
    self.escalate = Some(value);
    self
  }

  pub fn escalate_password(&mut self, escalate_password: Option<String>) -> &mut Self {
    self.escalate_password = escalate_password;
    self
  }

  pub fn inventory(&mut self, path: Option<String>) -> &mut Self {
    self.inventory = path;
    self
  }

  pub fn clear_extra_vars(&mut self) -> &mut Self {
    self.extra_vars.clear();
    self
  }

  pub fn extra_var(&mut self, k: &str, v: &str) -> &mut Self {
    self.extra_vars.insert(k.to_owned(), v.to_owned());
    self
  }

  pub fn timeout(&mut self, timeout: Option<usize>) -> &mut Self {
    self.timeout = timeout;
    self
  }

  pub fn check(&mut self, value: bool) -> &mut Self {
    self.check = Some(value);
    self
  }

  pub fn diff(&mut self, value: bool) -> &mut Self {
    self.diff = Some(value);
    self
  }

  fn get_args(&self) -> Vec<String> {
    let mut args = vec![];
    if let Some(check) = self.check {
      if check == true { args.push("--check".to_owned()); }
    }
    if let Some(diff) = self.diff {
      if diff == true { args.push("--diff".to_owned()); }
    }
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
      args.push(format!("ansible_password={}", password));
    }
    if let Some(escalate_password) = &self.escalate_password {
      args.push("-e".to_owned());
      args.push(format!("ansible_become_password={}", escalate_password));
    }
    for (k, v) in &self.extra_vars {
      args.push("-e".to_owned());
      args.push(format!("{}={}", k, v));
    }
    args
  }

  pub async fn playbook(&self, playbook_path: &str) -> Result<()> {
    let mut args = self.get_args();
    args.insert(0, playbook_path.to_owned());
    let code = &self.runner.run("ansible-playbook", args.iter()).await?.code();
    if 0.eq(code) {
      Ok(())
    } else {
      Err(Error::from(RunError::UnexpectedExitCode(*code)))
    }
  }

  pub async fn module<I>(&self, module: &str, module_args: &str) -> Result<()>
  {
    let mut args = self.get_args();
    args.insert(0, "ansible".to_owned());
    args.insert(1, "-m".to_owned());
    args.insert(2, module.to_owned());
    args.insert(3, "-a".to_owned());
    args.insert(4, module_args.to_owned());
    let code = &self.runner.run("ansible", args.iter()).await?.code();
    if 0.eq(code) {
      Ok(())
    } else {
      Err(Error::from(RunError::UnexpectedExitCode(*code)))
    }
  }
}

