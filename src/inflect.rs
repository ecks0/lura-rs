use log::debug;
use rlua::{ Context, Result as LuaResult };

const MOD: &str = std::module_path!();

pub use inflector::cases::{
  camelcase::to_camel_case,
  classcase::to_class_case,
  kebabcase::to_kebab_case,
  screamingsnakecase::to_screaming_snake_case as to_env_case,
  snakecase::to_snake_case,
};


#[cfg(feature = "lua")]
pub(crate) fn lua_init(ctx: &Context) -> LuaResult<()> {
 
  debug!(target: MOD, "Lua init");
  
  let inflect = ctx.create_table()?;

  inflect.set("to_camel_case", ctx.create_function(|_, args: (String,)| {
    Ok(to_camel_case(&args.0))
  })?)?;
  inflect.set("to_class_case", ctx.create_function(|_, args: (String,)| {
    Ok(to_class_case(&args.0))
  })?)?;
  inflect.set("to_env_case", ctx.create_function(|_, args: (String,)| {
    Ok(to_env_case(&args.0))
  })?)?;
  inflect.set("to_kebab_case", ctx.create_function(|_, args: (String,)| {
    Ok(to_kebab_case(&args.0))
  })?)?;
  inflect.set("to_snake_case", ctx.create_function(|_, args: (String,)| {
    Ok(to_snake_case(&args.0))
  })?)?;

  ctx.globals().set("inflect", inflect)?;

  Ok(())
}
