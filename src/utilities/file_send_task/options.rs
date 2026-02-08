use std::path::PathBuf;

use hyper::HeaderMap;

// TODO: Enhance this with all options from https://www.npmjs.com/package/send
// TODO: Use unused fields
pub struct FileSendOptions {
  pub max_age: u32,
  pub last_modified: bool,
  pub headers: Option<HeaderMap>,
  pub dotfiles: String,
  pub accept_ranges: bool,
  pub cache_control: bool,
  pub immutable: bool,
  pub index: Option<Vec<String>>,
  pub etag: bool,
  pub root: Option<PathBuf>,
}

impl Default for FileSendOptions {
  fn default() -> Self {
    Self {
      max_age: 0,
      last_modified: true,
      headers: None,
      dotfiles: "ignore".to_owned(),
      accept_ranges: true,
      cache_control: true,
      immutable: true,
      index: Some(vec!["index.html".to_owned()]),
      etag: true,
      root: None,
    }
  }
}
