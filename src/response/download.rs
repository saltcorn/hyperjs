use std::path::Path;

use hyper::{HeaderMap, header::CONTENT_DISPOSITION};
use napi::bindgen_prelude::*;
use napi_derive::napi;

use super::Response;
use crate::utilities::{self, FileSendOptions, FileSendTask};

#[napi(object)]
pub struct DownloadOptions<'a> {
  pub max_age: Option<u32>,
  pub root: Option<String>,
  pub last_modified: Option<bool>,
  pub headers: Option<Object<'a>>,
  pub dotfiles: Option<String>,
  pub accept_ranges: Option<bool>,
  pub cache_control: Option<bool>,
  pub immutable: Option<bool>,
  pub index: Option<Either3<String, Vec<String>, bool>>,
}

impl<'a> TryFrom<&DownloadOptions<'a>> for FileSendOptions {
  type Error = Error;

  fn try_from(value: &DownloadOptions<'a>) -> Result<Self> {
    Ok(Self {
      max_age: 0,
      last_modified: value.last_modified.unwrap_or(true),
      headers: match &value.headers {
        Some(headers) => Some(utilities::object_to_header_map(headers)?),
        None => None,
      },
      dotfiles: value.dotfiles.to_owned().unwrap_or("ignore".to_owned()),
      accept_ranges: value.accept_ranges.unwrap_or(true),
      cache_control: value.cache_control.unwrap_or(true),
      immutable: value.immutable.unwrap_or(false),
      index: match &value.index {
        Some(index) => match index {
          Either3::A(index) => Some(vec![index.to_owned()]),
          Either3::B(index_list) => Some(index_list.to_owned()),
          Either3::C(index) => match index {
            true => Some(vec!["index.html".to_owned()]),
            false => None,
          },
        },
        None => Some(vec!["index.html".to_owned()]),
      },
    })
  }
}

#[napi]
impl Response {
  /// Transfers the file at `path` as an "attachment". Typically, browsers will
  /// prompt the user for download. By default, the `Content-Disposition`
  /// header "filename=" parameter is derived from the `path` argument, but can
  /// be overridden with the `filename` parameter. If `path` is relative, then
  /// it will be based on the current working directory of the process or the
  /// `root` option, if provided.
  ///
  /// > This API provides access to data on the running file system. Ensure
  /// > that either (a) the way in which the `path` argument was constructed
  /// > into an absolute path is secure if it contains user input or (b) set
  /// > the `root` option to the absolute path of a directory to contain access
  /// > within.
  /// >
  /// > When the `root` option is provided, the `path` argument is allowed to
  /// > be a relative path, including containing `..`. Express will validate
  /// > that the relative path provided as `path` will resolve within the given
  /// > `root` option.
  ///
  /// The following table provides details on the options parameter.
  ///
  /// | Property | Description | Default |
  /// | --- | --- | --- |
  /// | `maxAge` | Sets the max-age property of the `Cache-Control` header in milliseconds or a string in [ms format](https://www.npmjs.org/package/ms) | 0 |
  /// | `root` | Root directory for relative filenames. | |
  /// | `lastModified` | Sets the `Last-Modified` header to the last modified date of the file on the OS. Set `false` to disable it. | Enabled |
  /// | `headers` | Object containing HTTP headers to serve with the file. | |
  /// | `dotfiles` | Option for serving dotfiles. Possible values are "allow", "deny", "ignore". | "ignore" |
  /// | `acceptRanges` | Enable or disable accepting ranged requests. | true |
  /// | `cacheControl` | Enable or disable setting `Cache-Control` response header. | true |
  /// | `immutable` | Enable or disable the `immutable` directive in the `Cache-Control` response header. If enabled, the `maxAge` option should also be specified to enable caching. The `immutable` directive will prevent supported clients from making conditional requests during the life of the `maxAge` option to check if the file has changed. | false |
  ///
  /// The method invokes the callback function `fn(err)` when the transfer is
  /// complete or when an error occurs. If the callback function is specified
  /// and an error occurs, the callback function must explicitly handle the
  /// response process either by ending the request-response cycle, or by
  /// passing control to the next route.
  ///
  /// Here is an example of using `res.sendFile` with all its arguments.
  ///
  /// ```javascript
  /// res.download('/report-12345.pdf')
  ///
  /// res.download('/report-12345.pdf', 'report.pdf')
  ///
  /// res.download('/report-12345.pdf', 'report.pdf', (err) => {
  ///   if (err) {
  ///     // Handle error, but keep in mind the response may be partially-sent
  ///     // so check res.headersSent
  ///   } else {
  ///     // decrement a download credit, etc.
  ///   }
  /// })
  ///
  /// ```
  ///
  #[napi]
  pub fn download(
    &self,
    path: String,
    options: Option<DownloadOptions>,
  ) -> Result<AsyncTask<FileSendTask>> {
    let mut file_send_options: FileSendOptions = match &options {
      Some(options) => options.try_into()?,
      None => FileSendOptions::default(),
    };

    // set Content-Disposition when file is sent
    let disposition = match utilities::content_disposition(&path) {
      Ok(disposition) => disposition,
      Err(e) => return Err(Error::new(Status::InvalidArg, e)),
    };

    file_send_options
      .headers
      .get_or_insert(HeaderMap::new())
      .insert(CONTENT_DISPOSITION, disposition);

    let root = match options.and_then(|options| options.root) {
      Some(root) => Path::new(&root).to_owned(),
      None => match std::env::current_dir() {
        Ok(cwd) => cwd,
        Err(e) => {
          return Err(Error::new(
            Status::GenericFailure,
            format!("Error getting current directory. {e}"),
          ));
        }
      },
    };

    // TODO: Wire application etag option to send
    Ok(AsyncTask::new(FileSendTask {
      response: self.clone(),
      root,
      path,
      options: file_send_options,
      //   TODO: Set etag based on server configuration
      etag: true,
    }))

    // self.with_inner(|response| response.send_file(body, env))
  }
}
