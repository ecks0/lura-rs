// thin wrapper for kubectl
//
// base kubectl commands are expressed as builders, e.g. `Apply`, `Delete`, `Get`
//
// `Manifest` stores yaml resource data in memory, and operates on it by dumping it to
// a temp file and passing the temp file path to kubectl.
//
// `Application` is an aggregate for `Manifest`

use {
  log::info,
  thiserror,
  crate::fs::{TempDir, dump},
  crate::run::Runner,
};

const MOD: &str = std::module_path!();

#[derive(thiserror::Error, Debug)]
pub enum Error {

  #[error("Output from stdout is missing from result")]
  StdoutMissing,

  #[error(transparent)]
  LuraFs(#[from] crate::fs::Error),

  #[error(transparent)]
  LuraRun(#[from] crate::run::Error),

  #[error(transparent)]
  SerdeJson(#[from] serde_json::error::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

/////
// Apply

#[derive(Clone, Debug)]
pub struct Apply {
  pub recursive: Option<bool>,
  pub wait: Option<bool>,
  pub timeout: Option<usize>,
  pub dry_run: Option<bool>,
}

impl Apply {

  pub fn new() -> Self {
    Apply {
      recursive: None,
      wait: None,
      timeout: None,
      dry_run: None,
    }
  }

  pub fn recursive(&mut self, value: bool) -> &mut Self {
    self.recursive = Some(value);
    self
  }

  pub fn wait(&mut self, value: bool) -> &mut Self {
    self.wait = Some(value);
    self
  }

  pub fn timeout(&mut self, value: Option<usize>) -> &mut Self {
    self.timeout = value;
    self
  }

  pub fn dry_run(&mut self, value: bool) -> &mut Self {
    self.dry_run = Some(value);
    self
  }

  pub fn apply(&self, runner: &Runner, path: &str) -> Result<()> {
    let mut args = vec!["apply", path];
    if let Some(recursive) = &self.recursive {
      if *recursive == true { args.push("--recursive=true"); }
      else { args.push("--recursive=false"); }
    }
    if let Some(wait) = &self.wait {
      if *wait == true { args.push("--wait=true"); }
      else { args.push("--wait=false"); }
    }
    if let Some(dry_run) = &self.dry_run {
      if *dry_run == true { args.push("--dry-run=true"); }
      else { args.push("--dry-run=false"); }
    }
    runner.run("kubectl", args.iter())?;
    Ok(())
  }
}

/////
// Delete

#[derive(Clone, Debug)]
pub struct Delete {
  pub recursive: Option<bool>,
  pub wait: Option<bool>,
  pub timeout: Option<usize>,
}

impl Delete {

  pub fn new() -> Self {
    Delete {
      recursive: None,
      wait: None,
      timeout: None,
    }
  }

  pub fn recursive(&mut self, value: bool) -> &mut Self {
    self.recursive = Some(value);
    self
  }

  pub fn wait(&mut self, value: bool) -> &mut Self {
    self.wait = Some(value);
    self
  }

  pub fn timeout(&mut self, value: Option<usize>) -> &mut Self {
    self.timeout = value;
    self
  }

  pub fn delete(&self, runner: &Runner, path: &str) -> Result<()> {
    let mut args = vec!["delete", path];
    if let Some(recursive) = &self.recursive {
      if *recursive == true { args.push("--recursive=true"); }
      else { args.push("--recursive=false"); }
    }
    if let Some(wait) = &self.wait {
      if *wait == true { args.push("--wait=true"); }
      else { args.push("--wait=false"); }
    }
    runner.run("kubectl", args.iter())?;
    Ok(())
  }
}

/////
// Get

#[derive(Clone, Debug)]
pub struct Get {
  pub resource: Option<String>,
  pub filename: Option<String>,
  pub recursive: Option<bool>,
  pub namespace: Option<String>,
  pub all_namespaces: Option<bool>,
  pub selector: Option<String>,
}

impl Get {

  pub fn new() -> Self {
    Get {
      resource: None,
      filename: None,
      recursive: None,
      namespace: None,
      all_namespaces: None,
      selector: None,
    }
  }

  pub fn resource(&mut self, resource: &str) -> &mut Self {
    self.resource = Some(resource.to_owned());
    self
  }

  pub fn filename(&mut self, filename: &str) -> &mut Self {
    self.filename = Some(filename.to_owned());
    self
  }

  pub fn recursive(&mut self, value: bool) -> &mut Self {
    self.recursive = Some(value);
    self
  }

  pub fn namespace(&mut self, namespace: &str) -> &mut Self {
    self.namespace = Some(namespace.to_owned());
    self
  }

  pub fn all_namespaces(&mut self, value: bool) -> &mut Self {
    self.all_namespaces = Some(value);
    self
  }

  pub fn selector(&mut self, selector: &str) -> &mut Self {
    self.selector = Some(selector.to_owned());
    self
  }

  pub fn get(&self, runner: &Runner) -> Result<Option<serde_json::Value>> {
    let mut args = vec!["get".to_owned(), "--output=json".to_owned()];
    if let Some(resource) = &self.resource {
      args.push(resource.to_owned());
    }
    if let Some(filename) = &self.filename {
      args.push(format!("--filename={}", filename));
    }
    if let Some(recursive) = &self.recursive {
      if *recursive == true { args.push("--recursive=true".to_owned()); }
      else { args.push("--recursive=false".to_owned()); }
    }
    if let Some(namespace) = &self.namespace {
      args.push(format!("--namespace={}", namespace));
    }
    if let Some(all_namespaces) = &self.all_namespaces {
      if *all_namespaces == true { args.push("--all-namespaces=true".to_owned()); }
      else { args.push("--all-namespaces=false".to_owned()); }
    }
    if let Some(selector) = &self.selector {
      args.push(format!("--selector={}", selector));
    }
    let output = runner.run("kubectl", args.iter())?;
    let stdout = output.stdout().ok_or(Error::StdoutMissing)?;
    Ok(serde_json::from_str(stdout)?)
  }
}

/////
// Manifest

#[derive(Clone, Debug)]
pub struct Manifest {
  pub name: String,
  pub body: String,
  pub wait: bool,
  pub timeout: Option<usize>,
}

impl Manifest {

  pub fn new(name: &str, body: &str) -> Self {
    Self {
      name: name.to_owned(),
      body: body.to_owned(),
      wait: false,
      timeout: None
    }
  }

  pub fn wait(&mut self, value: bool) -> &mut Self {
    self.wait = value;
    self
  }

  pub fn timeout(&mut self, value: Option<usize>) -> &mut Self {
    self.timeout = value;
    self
  }

  pub fn dump(&self, path: &str) -> Result<()> {
    Ok(dump(path, &self.body)?)
  }

  pub fn apply(&self, runner: &Runner) -> Result<()> {
    info!(target: MOD, "Applying: {}", self.name);
    let temp_dir = TempDir::new("lura.progs.kubectl")?;
    let path = format!("{}/manifest.yaml", temp_dir.as_str());
    self.dump(&path)?;
    Ok(Apply
      ::new()
      .wait(self.wait)
      .timeout(self.timeout)
      .apply(runner, &path)?)
  }
  
  pub fn delete(&self, runner: &Runner) -> Result<()> {
    info!(target: MOD, "Deleting: {}", self.name);
    let temp_dir = TempDir::new("lura.progs.kubectl")?;
    let path = format!("{}/manifest.yaml", temp_dir.as_str());
    self.dump(&path)?;
    Ok(Delete
      ::new()
      .wait(self.wait)
      .timeout(self.timeout)
      .delete(runner, &path)?)
  }

  pub fn applied(&self, runner: &Runner) -> Result<bool> {
    // returns true if any of the resources described by this manifest are applied

    let temp_dir = TempDir::new("lura.progs.kubectl")?;
    let path = format!("{}/manifest.yaml", temp_dir.as_str());
    self.dump(&path)?;
    let json = Get
      ::new()
      .filename(&path)
      .get(&runner)?;
    match json {
      Some(_) => { info!(target: MOD, "Applied: {}", self.name); Ok(true) }
      None => { info!(target: MOD, "Not applied: {}", self.name); Ok(false) }
    }
  }
}

/////
// Application

pub struct Application {
  name: String,
  runner: Runner,
  manifests: Vec<Manifest>,
}

impl Application {

  pub fn new(name: &str, runner: Runner, manifests: Vec<Manifest>) -> Self {
    Self {
      name: name.to_owned(),
      manifests: manifests,
      runner: runner,
    }
  }

  pub fn name(&self) -> &str {
    &self.name
  }

  pub fn wait(&mut self, value: bool) -> &mut Self {
    for manifest in self.manifests.iter_mut() {
      manifest.wait = value;
    }
    self
  }

  pub fn timeout(&mut self, value: Option<usize>) -> &mut Self {
    for manifest in self.manifests.iter_mut() {
      manifest.timeout = value;
    }
    self
  }

  pub fn apply(&self) -> Result<()> {
    for manifest in &self.manifests {
      manifest.apply(&self.runner)?;
    }
    Ok(())
  }
  
  pub fn delete(&self) -> Result<()> {
    for manifest in &self.manifests {
      manifest.apply(&self.runner)?;
    }
    Ok(())
  }

  pub fn applied(&self) -> Result<bool> {
    for manifest in &self.manifests {
      if ! manifest.applied(&self.runner)? {
        return Ok(false);
      }
    }
    Ok(true)
  }
}
