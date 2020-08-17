use {
  rand::{thread_rng, Rng},
  rand::distributions::Alphanumeric,
  regex::Regex,
  thiserror,
  std::{
    env,
    path::Path,
  },
  crate::fs::{chmod, path_buf_to_string},
};

#[derive(thiserror::Error, Debug)]
pub enum Error {

  #[error(transparent)] LuraRunAsync(#[from] crate::run_async::Error),
  #[error(transparent)] LuraFs(#[from] crate::fs::Error),
  #[error(transparent)] Regex(#[from] regex::Error),
  #[error(transparent)] TokioIo(#[from] tokio::io::Error),
}

type Result<T> = std::result::Result<T, Error>;

pub async fn tempdir(prefix: &str) -> Result<String> {
  // create and return a temporary directory in `std::env::temp_dir()` which begins with
  // `prefix`. the directory IS NOT automatically deleted. initial permissions of the directory
  // will be 0o700

  fn randstr() -> String {
    thread_rng()
      .sample_iter(&Alphanumeric)
      .take(12)
      .collect()
  }

  loop {
    let temp_dir = format!("{0}/{1}.{2}", path_buf_to_string(env::temp_dir())?, prefix, randstr());
    match mkdir(&temp_dir).await {
      Ok(()) => {
        chmod(&temp_dir, 0o700)?;
        return Ok(temp_dir);
      },
      Err(err) => {
        if let Error::TokioIo(err) = &err {
          if let tokio::io::ErrorKind::AlreadyExists = err.kind() { continue; }
        }
        return Err(err);
      },
    }
  }
}

pub async fn mkdir(path: &str) -> Result<()> {
  // create a directory

  Ok(tokio::fs::create_dir(path).await?)
}

pub async fn cp(src: &str, dst: &str) -> Result<()> {
  // move a file or directory recursively

  crate::run_async::run("cp", ["-R", src, dst].iter()).await?;
  Ok(())
}

pub async fn mv(src: &str, dst: &str) -> Result<()> {
  // move a file or directory recursively

  crate::run_async::run("mv", ["-f", src, dst].iter()).await?;
  Ok(())
}

pub async fn rm(path: &str) -> Result<()> {
  // remove a path. directories are removed recursively - be careful

  let path = Path::new(path);
  Ok(if path.is_dir() {
    tokio::fs::remove_dir_all(path).await?
  } else {
     tokio::fs::remove_file(path).await?
  })
}

pub async fn load(path: &str) -> Result<Vec<u8>> {
  // load data from a file as bytes

  Ok(tokio::fs::read(path).await?)
}

pub async fn loads(path: &str) -> Result<String> {
  // load data from a file as `String`

  Ok(std::fs::read_to_string(path)?)
}

pub async fn dump<D: AsRef<[u8]>>(path: &str, data: D) -> Result<()> {
  // write data to a file

  Ok(tokio::fs::write(path, data.as_ref()).await?)
}

pub async fn replace_line(path: &str, regexp: &str, replace: &str) -> Result<usize> {
  // replace the pattern `regexp` with `replace` in file at `path`. named back-references may be
  // used. returns the number of lines that were relpaced. data will be written to `path` only if
  // at least one match is found

  let re = Regex::new(regexp)?;
  let mut matched = 0usize;
  let mut output = String::new();
  for line in loads(&path).await?.split("\n") { // FIXME
    if re.is_match(line) {
      matched += 1;
      output.push_str(&re.replace_all(line, replace).into_owned());
    } else {
      output.push_str(line);
    };
    output.push_str("\n");
  }
  if matched > 0 { dump(&path, output).await?; }
  Ok(matched)
}
