use std::path::{Path, PathBuf};

use napi::bindgen_prelude::*;
use napi_derive::napi;

use super::Response;
use crate::utilities::{self, FileSendOptions, FileSendTask};

#[napi(object)]
pub struct SendFileOptions<'a> {
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

impl<'a> TryFrom<&SendFileOptions<'a>> for FileSendOptions {
  type Error = Error;

  fn try_from(value: &SendFileOptions<'a>) -> Result<Self> {
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
  /// Transfers the file at the given `path`. Sets the `Content-Type` response
  /// HTTP header field based on the filename’s extension. Unless the `root`
  /// option is set in the options object, `path` must be an absolute path to
  /// the file.
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
  /// app.get('/file/:name', (req, res, next) => {
  ///   const options = {
  ///     root: path.join(__dirname, 'public'),
  ///     dotfiles: 'deny',
  ///     headers: {
  ///       'x-timestamp': Date.now(),
  ///       'x-sent': true
  ///     }
  ///   }
  ///
  ///   const fileName = req.params.name
  ///   res.sendFile(fileName, options, (err) => {
  ///     if (err) {
  ///       next(err)
  ///     } else {
  ///       console.log('Sent:', fileName)
  ///     }
  ///   })
  /// })
  /// ```
  ///
  /// The following example illustrates using `res.sendFile` to provide
  /// fine-grained support for serving files:
  ///
  /// ```javascript
  /// app.get('/user/:uid/photos/:file', (req, res) => {
  ///   const uid = req.params.uid
  ///   const file = req.params.file
  ///
  ///   req.user.mayViewFilesFrom(uid, (yes) => {
  ///     if (yes) {
  ///       res.sendFile(`/uploads/${uid}/${file}`)
  ///     } else {
  ///       res.status(403).send("Sorry! You can't see that.")
  ///     }
  ///   })
  /// })
  /// ```
  ///
  /// For more information, or if you have issues or concerns, see
  /// [send](https://github.com/pillarjs/send).
  #[napi]
  pub fn send_file(
    &self,
    path: String,
    options: Option<SendFileOptions>,
  ) -> Result<AsyncTask<FileSendTask>> {
    let file_send_options: FileSendOptions = match &options {
      Some(options) => options.try_into()?,
      None => FileSendOptions::default(),
    };

    let root = options.and_then(|options| options.root);

    if root.is_none() && !Path::new(&path).is_absolute() {
      return Err(Error::new(
        Status::InvalidArg,
        "path must be absolute or specify root to res.sendFile",
      ));
    }

    let wrapped_path = Path::new(&path);

    let root = match &root {
      Some(root) => PathBuf::from(root),
      None => match wrapped_path.is_absolute() {
        true => match wrapped_path.extension().is_some() {
          true => wrapped_path.parent().unwrap(), // infallible
          false => wrapped_path,
        }
        .to_owned(),
        false => {
          return Err(Error::new(
            Status::InvalidArg,
            "path must be absolute or specify root to res.sendFile",
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
  }
}
