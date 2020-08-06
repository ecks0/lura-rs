// tokio utilities
//
// this module provides two ways to synchronously call a future using tokio

use {
  std::future::Future,
  thiserror,
};

#[derive(thiserror::Error, Debug)]
pub enum Error {

  #[error(transparent)]
  IOError(#[from] std::io::Error),
}

pub fn block_on<F>(future: F) -> Result<F::Output, Error>
where
  F: Future,
{
  // run a future to completion on another thread

  Ok(tokio::runtime::Runtime::new()?.block_on(future))
}

pub fn block_on_local<F>(future: F) -> Result<F::Output, Error>
where
  F: Future,
{
  // run a future to completion on the current thread

  Ok(tokio_compat::runtime::current_thread::Runtime::new()?.block_on_std(future))
}
