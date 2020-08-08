use {
  thiserror,
  std::path::Path,
};

#[derive(thiserror::Error, Debug)]
pub enum Error {

  #[error(transparent)]
  TokioIo(#[from] tokio::io::Error),

  #[error(transparent)]
  LuraRunAsync(#[from] crate::run_async::Error),

  #[error(transparent)]
  LuraFs(#[from] crate::fs::Error),
}

type Result<T> = std::result::Result<T, Error>;

pub async fn mv<P: AsRef<Path>>(src: P, dst: P) -> Result<()> {
  // move a file or directory recursively

  let src = &crate::fs::path_to_string(src.as_ref())?;
  let dst = &crate::fs::path_to_string(dst.as_ref())?;
  crate::run_async::run("mv", ["-f", src, dst].iter()).await?; // FIXME
  Ok(())
}

pub async fn load<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
  // load data from a file as bytes

  Ok(tokio::fs::read(path).await?)
}

