use {
  log,
  rlua::{ Context, Result as LuaResult },
};

const MOD: &str = std::module_path!();

pub(crate) fn lua_init(ctx: &Context) -> LuaResult<()> {

  log::debug!(target: MOD, "Lua init");

  let log = ctx.create_table()?;
  
  log.set("error", ctx.create_function(|_, args: (String, String)| {
    log::error!(target: &args.0, "{0}", args.1); Ok(()) 
  })?)?;
  log.set("warn", ctx.create_function(|_, args: (String, String)| {
    log::warn!(target: &args.0, "{0}", args.1); Ok(())
  })?)?;
  log.set("info", ctx.create_function(|_, args: (String, String)|{
     log::info!(target: &args.0, "{0}", args.1); Ok(())
  })?)?;
  log.set("debug", ctx.create_function(|_, args: (String, String)| {
    log::debug!(target: &args.0, "{0}", args.1); Ok(())
  })?)?;
  log.set("trace", ctx.create_function(|_, args: (String, String)| {
    log::trace!(target: &args.0, "{0}", args.1); Ok(())
  })?)?;

  ctx.globals().set("log", log)?;

  Ok(())
}
