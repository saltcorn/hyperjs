use std::path::Path;

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
}

impl<'a> TryFrom<&SendFileOptions<'a>> for FileSendOptions {
  type Error = Error;

  fn try_from(value: &SendFileOptions<'a>) -> Result<Self> {
    let mut options = FileSendOptions::default();

    if let Some(root) = &value.root {
      options.root = Some(Path::new(root).to_path_buf())
    }

    if let Some(max_age) = value.max_age {
      options.max_age = max_age as u64;
    }

    if let Some(last_modified) = value.last_modified {
      options.last_modified = last_modified;
    }

    if let Some(headers) = &value.headers {
      options.headers = Some(utilities::object_to_header_map(headers)?);
    }

    if let Some(dotfiles) = &value.dotfiles {
      options.dotfiles = dotfiles.to_owned();
    }

    if let Some(accept_ranges) = value.accept_ranges {
      options.accept_ranges = accept_ranges;
    }

    if let Some(cache_control) = value.cache_control {
      options.cache_control = cache_control;
    }

    if let Some(immutable) = value.immutable {
      options.immutable = immutable;
    }

    Ok(options)
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

    Ok(AsyncTask::new(FileSendTask {
      response: self.clone(),
      path,
      options: file_send_options,
    }))
  }
}
