pub mod docker;
#[cfg(feature = "async")]
pub mod docker_async;

pub mod git;
#[cfg(feature = "async")]
pub mod git_async;

//pub mod kubectl;
//#[cfg(feature = "async")]
//pub mod kubectl_async;

pub mod systemd;
#[cfg(feature = "async")]
pub mod systemd_async;

#[cfg(feature = "lua")]
const MOD: &str = std::module_path!();

#[cfg(feature = "lua")]
use {
  log::debug,
  rlua::{ Context, Result as LuaResult, Table },
};

#[cfg(feature = "lua")]
pub(crate) fn lua_init(ctx: &Context) -> LuaResult<()> {
 
  debug!(target: MOD, "Lua init");

  ctx
    .globals()
    .get::<_, Table>("lura")?
    .set("progs", ctx.create_table()?)?;

  Ok(())
}