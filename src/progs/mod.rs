pub mod docker;
pub mod git;

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