// lua support

use {
  log::debug,
  rlua::{
    Context,
    Function,
    FromLuaMulti,
    Lua,
    MultiValue,
    Table,
    ToLuaMulti,
  },
  rustyline::Editor,
  crate::relics::Relics,
};

pub use rlua::{Error, Result};

const MOD: &str = std::module_path!();

pub fn new() -> Result<Lua> {
  // initialize a new lua instance with bindings from this crate, and return it

  let lua = Lua::new();
  lua.context(|ctx| -> Result<()> { init(&ctx) })?;
  Ok(lua)
}

pub fn init(ctx: &Context) -> Result<()> {
  // initialize a lua context with bindings from this crate

  debug!(target: MOD, "Lua init");
  ctx.globals().set("lura", ctx.create_table()?)?;
  crate::progs::lua_init(ctx)?;
  crate::progs::docker::lua_init(ctx)?;
  crate::config::lua_init(ctx)?;
  crate::fs::lua_init(ctx)?;
  crate::log::lua_init(ctx)?;
  crate::inflect::lua_init(ctx)?;
  crate::run::lua_init(ctx)?;
  crate::template::lua_init(ctx)?;
  Ok(())
}

pub fn repl(lua: &Lua, prompt1: &str, prompt2: &str) {
  // run a lua repl over stdin/stdout using custom prompts

  lua.context(|lua| {
    let mut editor = Editor::<()>::new();

    loop {
      let mut prompt = prompt1;
      let mut line = String::new();

      loop {
        match editor.readline(prompt) {
          Ok(input) => line.push_str(&input),
          Err(_) => return,
        }

        match lua.load(&line).eval::<MultiValue>() {
          Ok(values) => {
            editor.add_history_entry(line);
            println!(
              "{}",
              values
                .iter()
                .map(|value| format!("{:?}", value))
                .collect::<Vec<_>>()
                .join("\t")
            );
            break;
          }
          Err(Error::SyntaxError { incomplete_input: true, .. }) => {
            // continue reading input and append it to `line`
            line.push_str("\n"); // separate input lines
            prompt = prompt2;
          }
          Err(e) => {
            eprintln!("Error: {}", e);
            break;
          }
        }
      }
    }
  });
}

pub fn call<'lua, A, R>(ctx: Context<'lua>, path: &str, fun: &str, args: A) -> Result<R>
where
  A: ToLuaMulti<'lua>,
  R: FromLuaMulti<'lua>,
{
  // load a lua function from a table and call it. `path` is the path to the table,
  // e.g. `"some_table.other_table", or an empty string to use the globals table. `fun`
  // is function's key in the final table. the result of the function call is returned

  debug!(target: MOD, "Lua call: {}.{}()", path, fun);
  let table = match path {
    "" => ctx.globals(),
    _ => path
      .split('.')
      .try_fold(ctx.globals(), |tbl, path| tbl.get::<_, Table>(path))?,
  };
  Ok(table
    .get::<_, Function>(fun)?
    .call::<_, R>(args)?)
}

pub fn load<'a, I, S>(ctx: &Context, module: &str, relics: &Relics, sources: I) -> Result<()>
where
  I: IntoIterator<Item = S>,
  S: AsRef<str>
{
  // load lua source code from static relics and run in the given lua context

  Ok(sources
    .into_iter()
    .try_for_each(|source| -> Result<()> {
      let source = source.as_ref();
      debug!(target: module, "Lua load: {}", source);
      Ok(ctx
        .load(relics
          .as_str(source)
          .map_err(Error::from)?)
        .set_name(source)?
        .exec()?)
    })?)
}

