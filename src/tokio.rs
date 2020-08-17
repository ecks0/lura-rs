// tokio utilities

use {
  bytes::Bytes,
  futures::{
    Future,
    stream::{self, Stream, StreamExt, TryStreamExt},
  },
  tokio::io::{AsyncRead, Result as IoResult},
  tokio_util::codec,
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

pub fn into_byte_stream<A>(async_read: A) -> impl Stream<Item=IoResult<u8>>
where
  A: AsyncRead,
{
  codec::FramedRead::new(async_read, codec::BytesCodec::new())
    .map_ok(|bytes| stream::iter(bytes).map(Ok))
    .try_flatten()
}

pub fn into_bytes_stream<A>(async_read: A) -> impl Stream<Item=IoResult<Bytes>>
where
  A: AsyncRead,
{
  codec::FramedRead::new(async_read, codec::BytesCodec::new())
    .map_ok(|bytes| bytes.freeze())
}
