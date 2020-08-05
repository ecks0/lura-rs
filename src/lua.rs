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
};

pub use rlua::{Error, Result};

const MOD: &str = std::module_path!();

pub fn new() -> Result<Lua> {
  // initialize and return a new lua instance

  let lua = Lua::new();
  lua.context(|ctx| -> Result<()> { init(&ctx) })?;
  Ok(lua)
}

pub fn init(ctx: &Context) -> Result<()> {
  // initialize a lua context

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
  // run a lua repl over stdin/stdout

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

pub fn call<'lua, A, R>(ctx: Context<'lua>, modu: &str, fun: &str, args: A) -> Result<R>
where
  A: ToLuaMulti<'lua>,
  R: FromLuaMulti<'lua>,
{
  // load the given lua function from the specified module path, call it, and return it result

  debug!(target: MOD, "Lua call: {}.{}()", modu, fun);
  let table = match modu {
    "" | "global" | "globals" => ctx.globals(),
    _ => modu
      .split('.')
      .try_fold(ctx.globals(), |tbl, modu| tbl.get::<_, Table>(modu))?,
  };
  Ok(table
    .get::<_, Function>(fun)?
    .call::<_, R>(args)?)
}
