use {
  reqwest::{
    Url,
    StatusCode,
    blocking::{
      Client,
      RequestBuilder,
      Response,
    },
  },
  thiserror,
  crate::fs::dump,
};

#[derive(thiserror::Error, Debug)]
pub enum Error {

  #[error("External error: {0}")]
  External(anyhow::Error),

  #[error("Request failed with status {0}")]
  Fail(StatusCode),

  #[error(transparent)]
  LuraFs(#[from] crate::fs::Error),

  #[error(transparent)]
  Reqwest(#[from] reqwest::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn request<T, N, O, E>(url: Url, new: N, ok: O, err: E) -> Result<T>
where
  N: Fn(Url) -> RequestBuilder,
  O: Fn(Response) -> Result<T>,
  E: Fn(Response) -> Result<T>,
{
  // composable generalization of the request process
  //
  // new - returns a `RequestBuilder` from the provided `Url`
  // ok  - handles 2xx response from server and returns `Result<T>`
  // err - handles non-2xx response from server and returns `Result<T>`

  let builder = new(url);
  let result = builder.send().map_err(Error::from)?;
  if result.status().is_success() {
    ok(result)
  } else {
    err(result)
  }
}

pub fn fail<T>(response: Response) -> Result<T> {
  
  Err(Error::Fail(response.status()))
}

pub fn get<T, O, E>(url: Url, ok: O, err: E) -> Result<T>
where
  O: Fn(Response) -> Result<T>,
  E: Fn(Response) -> Result<T>,
{ 
  request(url, |u| Client::new().get(u), ok, err)
}

pub fn get_str(url: Url) -> Result<String> {
  request(
    url,
    |u| Client::new().get(u),
    |r| Ok(r.text()?),
    fail)
}

pub fn get_bytes(url: Url) -> Result<Vec<u8>> {
  request(
    url,
    |u| Client::new().get(u),
    |r| Ok(r.bytes()?.to_vec()),
    fail)
}

pub fn get_file(url: Url, path: &str) -> Result<()> {
  // FIXME
  Ok(dump(path, get_bytes(url)?)?)
}

pub fn post<T, O, E>(url: Url, ok: O, err: E) -> Result<T>
where
  O: Fn(Response) -> Result<T>,
  E: Fn(Response) -> Result<T>,
{
  request(url, |u| Client::new().post(u), ok, err)
}

pub fn put<T, O, E>(url: Url, ok: O, err: E) -> Result<T>
where
  O: Fn(Response) -> Result<T>,
  E: Fn(Response) -> Result<T>,
{
  request(url, |u| Client::new().put(u), ok, err)
}

pub fn delete<T, O, E>(url: Url, ok: O, err: E) -> Result<T>
where
  O: Fn(Response) -> Result<T>,
  E: Fn(Response) -> Result<T>,
{
  request(url, |u| Client::new().delete(u), ok, err)
}
