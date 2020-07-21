pub mod tokio;

#[derive(Debug, thiserror::Error)]
pub enum Error {

  #[error(transparent)]
  IOError(#[from] std::io::Error),
}
