use std::future::Future;
use crate::runtime::Error;

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
