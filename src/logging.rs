use {chrono, fern};

#[derive(thiserror::Error, Debug)]
pub enum Error {

  #[error(transparent)] LogSetLogger(#[from] log::SetLoggerError),
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn setup<F, O>(level: log::LevelFilter, output: O, filter: F) -> Result<()>
where
  F: Fn(&log::Metadata) -> bool + Send + Sync + 'static,
  O: Into<fern::Output>,
{
  Ok(fern::Dispatch
    ::new()
    .format(|out, message, record| {
      out.finish(format_args!(
        "{0: <18} {1: >5} {2}",
        chrono::Local::now().format("%m/%d %H:%M:%S%.3f"),
        record.level(),
        message,
      ))        
    })
    .level(level)
    .filter(filter)
    .chain(output)
    .apply()?)
}
