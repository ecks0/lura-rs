// functional http api

use {
  thiserror,
  crate::fs::dump,
};

pub use {
  reqwest::{
    Method,
    StatusCode,
    Url,
    blocking::{Client, Request, RequestBuilder, Response},
  },
  url::ParseError,
};

#[derive(thiserror::Error, Debug)]
pub enum Error {

  #[error("Request or response has a body which cannot be cloned")]
  Clone,

  #[error("External error: {0}")]
  External(anyhow::Error),

  #[error("{1} failed with code {2}: {0}")]
  Fail(Url, Method, StatusCode),

  #[error(transparent)]
  LuraFs(#[from] crate::fs::Error),

  #[error(transparent)]
  Reqwest(#[from] reqwest::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn request<T, N, O, E>(url: Url, new: N, ok: O, err: E) -> Result<T>
where
  N: Fn(Url, Client) -> RequestBuilder,
  O: Fn(Request, Response) -> Result<T>,
  E: Fn(Request, Response) -> Result<T>,
{
  // new - returns a `RequestBuilder` from the provided `Url` and `Client`
  // ok  - handles 2xx response from server and returns `Result<T>`
  // err - handles non-2xx response from server and returns `Result<T>`
  //
  // callers may use `http::ok` and `http::err` as defaultimplementations
  // of `ok` and `err`

  let client = Client::new();
  let builder = new(url, client);
  let request = builder.try_clone().ok_or(Error::Clone)?.build()?;
  let response = builder.send().map_err(Error::from)?;
  if response.status().is_success() {
    ok(request, response)
  } else {
    err(request, response)
  }
}

pub fn ok(_: Request, _: Response) -> Result<()> {
  Ok(())
}

pub fn err<T>(request: Request, response: Response) -> Result<T> {
  Err(Error::Fail(request.url().clone(), request.method().clone(), response.status()))
}

pub fn get<T, O, E>(url: Url, ok: O, err: E) -> Result<T>
where
  O: Fn(Request, Response) -> Result<T>,
  E: Fn(Request, Response) -> Result<T>,
{ 
  request(url, |u, c| c.get(u), ok, err)
}

pub fn get_url(url: Url) -> Result<()> {
  request(
    url,
    |u, c| c.get(u),
    ok,
    err,
  )
}

pub fn get_and<T, O>(url: Url, ok: O) -> Result<T> 
where
  O: Fn(Request, Response) -> Result<T>,
{
  request(
    url,
    |u, c| c.get(u),
    ok,
    err)
}

pub fn get_or<T, E>(url: Url, err: E) -> Result<()> 
where
  E: Fn(Request, Response) -> Result<()>,
{
  request(
    url,
    |u, c| c.get(u),
    ok,
    err)
}

pub fn get_str(url: Url) -> Result<String> {
  request(
    url,
    |u, c| c.get(u),
    |_, res| Ok(res.text()?),
    err)
}

pub fn get_bytes(url: Url) -> Result<Vec<u8>> {
  request(
    url,
    |u, c| c.get(u),
    |_, res| Ok(res.bytes()?.to_vec()),
    err)
}

pub fn get_file(url: Url, path: &str) -> Result<()> {
  Ok(dump(path, get_bytes(url)?)?)
}

pub fn post<T, O, E>(url: Url, ok: O, err: E) -> Result<T>
where
  O: Fn(Request, Response) -> Result<T>,
  E: Fn(Request, Response) -> Result<T>,
{
  request(
    url,
    |u, c| c.post(u),
    ok,
    err)
}

pub fn post_url(url: Url) -> Result<()> {
  request(
    url,
    |u, c| c.post(u),
    ok,
    err,
  )
}

pub fn post_and<T, O>(url: Url, ok: O) -> Result<T> 
where
  O: Fn(Request, Response) -> Result<T>,
{
  request(
    url,
    |u, c| c.post(u),
    ok,
    err)
}

pub fn post_or<T, E>(url: Url, err: E) -> Result<()> 
where
  E: Fn(Request, Response) -> Result<()>,
{
  request(
    url,
    |u, c| c.post(u),
    ok,
    err)
}

pub fn put<T, O, E>(url: Url, ok: O, err: E) -> Result<T>
where
  O: Fn(Request, Response) -> Result<T>,
  E: Fn(Request, Response) -> Result<T>,
{
  request(url, |u, c| c.put(u), ok, err)
}

pub fn put_url(url: Url) -> Result<()> {
  request(
    url,
    |u, c| c.put(u),
    ok,
    err,
  )
}

pub fn put_and<T, O>(url: Url, ok: O) -> Result<T> 
where
  O: Fn(Request, Response) -> Result<T>,
{
  request(
    url,
    |u, c| c.put(u),
    ok,
    err)
}

pub fn put_or<T, E>(url: Url, err: E) -> Result<()> 
where
  E: Fn(Request, Response) -> Result<()>,
{
  request(
    url,
    |u, c| c.put(u),
    ok,
    err)
}

pub fn delete<T, O, E>(url: Url, ok: O, err: E) -> Result<T>
where
  O: Fn(Request, Response) -> Result<T>,
  E: Fn(Request, Response) -> Result<T>,
{
  request(url, |u, c| c.delete(u), ok, err)
}

pub fn delete_url(url: Url) -> Result<()> {
  request(
    url,
    |u, c| c.delete(u),
    ok,
    err,
  )
}

pub fn delete_and<T, O>(url: Url, ok: O) -> Result<T> 
where
  O: Fn(Request, Response) -> Result<T>,
{
  request(
    url,
    |u, c| c.delete(u),
    ok,
    err)
}

pub fn delete_or<T, E>(url: Url, err: E) -> Result<()> 
where
  E: Fn(Request, Response) -> Result<()>,
{
  request(
    url,
    |u, c| c.delete(u),
    ok,
    err)
}
