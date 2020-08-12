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

#[cfg(feature = "lua")]
use {
  log::debug,
  rlua::{ Context, Error as LuaError, Result as LuaResult, Table },
  std::sync::Arc,
};

#[cfg(feature = "lua")]
impl From<Error> for LuaError {
  fn from(err: Error) -> LuaError {
    LuaError::ExternalError(Arc::new(err))
  }
}

#[cfg(feature = "lua")]
pub(crate) fn lua_init(ctx: &Context) -> LuaResult<()> {
 
  debug!("Lua init");

  let docker = ctx.create_table()?;

  docker.set("build", ctx.create_function(|_, args: (String, String)| {
    Ok(build(&run::runner(), &args.0, Some(&args.1))?)
  })?)?;
  docker.set("tag", ctx.create_function(|_, args: (String, String)| {
    Ok(tag(&run::runner(), &args.0, &args.1)?)
  })?)?;
  docker.set("push", ctx.create_function(|_, args: (String,)| {
    Ok(push(&run::runner(), &args.0)?)
  })?)?;

  ctx
    .globals()
    .get::<_, Table>("lura")?
    .get::<_, Table>("progs")?
    .set("docker", docker)?;
  
  Ok(())
}
