use std::path::{Path, PathBuf};

use hyper::{HeaderMap, Method, Response as HyperResponse, StatusCode, header};
use hyper_staticfile::{AcceptEncoding, ResolveResult, Resolver};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use tokio::runtime::Runtime;

use super::Response;
use crate::{response::CrateBody, utilities};

#[napi(object)]
pub struct SendFileOptions<'a> {
  pub max_age: Option<u32>,
  pub root: Option<String>,
  pub headers: Option<Object<'a>>,
  pub dotfiles: Option<String>,
  pub cache_control: Option<bool>,
  pub immutable: Option<bool>,
  pub index: Option<Either3<String, Vec<String>, bool>>,
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
    options: SendFileOptions,
  ) -> Result<AsyncTask<FileSendTask>> {
    if options.root.is_none() && !Path::new(&path).is_absolute() {
      return Err(Error::new(
        Status::InvalidArg,
        "path must be absolute or specify root to res.sendFile",
      ));
    }

    let wrapped_path = Path::new(&path);

    let root = match &options.root {
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

    // create file stream
    // let pathname = utilities::encode_url(&path);

    // TODO: Wire application etag option to send
    Ok(AsyncTask::new(FileSendTask {
      response: self.clone(),
      root,
      path,
      options: options.into(),
    }))

    // self.with_inner(|response| response.send_file(body, env))
  }
}

// TODO: Enhance this with all options from https://www.npmjs.com/package/send
// TODO: Use unused fields
pub struct FileSendOptions {
  pub max_age: u32,
  pub headers: Option<HeaderMap>,
  pub dotfiles: String,
  pub cache_control: bool,
  pub immutable: bool,
  pub index: Option<Vec<String>>,
}

impl<'a> From<SendFileOptions<'a>> for FileSendOptions {
  fn from(value: SendFileOptions<'a>) -> Self {
    Self {
      max_age: 0,
      headers: None,
      dotfiles: value.dotfiles.unwrap_or("ignore".to_owned()),
      cache_control: value.cache_control.unwrap_or(true),
      immutable: value.immutable.unwrap_or(false),
      index: match value.index {
        Some(index) => match index {
          Either3::A(index) => Some(vec![index]),
          Either3::B(index_list) => Some(index_list),
          Either3::C(index) => match index {
            true => Some(vec!["index.html".to_owned()]),
            false => None,
          },
        },
        None => Some(vec!["index.html".to_owned()]),
      },
    }
  }
}

pub struct FileSendTask {
  response: Response,
  root: PathBuf,
  path: String,
  options: FileSendOptions,
}

impl Task for FileSendTask {
  type Output = HyperResponse<CrateBody>;
  type JsValue = ();

  fn compute(&mut self) -> Result<Self::Output> {
    // dotfile handling
    if utilities::contains_dot_file(Path::new(&self.path)) {
      match self.options.dotfiles.as_str() {
        "allow" => {}
        "deny" => {
          // return 403 response
          let mut response = HyperResponse::new(CrateBody::Empty);
          *response.status_mut() = StatusCode::FORBIDDEN;
          return Ok(response);
        }
        _ => {
          // return 404 response
          let mut response = HyperResponse::new(CrateBody::Empty);
          *response.status_mut() = StatusCode::NOT_FOUND;
          return Ok(response);
        }
      }
    }

    // support the index option
    // TODO: RequestedPath::resolve
    //       Check if is_dir_request && index.is_none => 404 - serve index file disabled
    //       Check if is_dir_request && index.is_some => set path to first index file for which Path.exists

    // Create the runtime
    let rt = Runtime::new()?;

    // Get a handle from this runtime
    let handle = rt.handle();

    let request = self
      .response
      .req()
      .with_inner_mut(|w_req| w_req.take_inner())?;

    // Handle only `GET`/`HEAD` and absolute paths.
    let result = match *request.method() {
      Method::HEAD | Method::GET => {
        let resolver = Resolver::new(&self.root);

        // Parse `Accept-Encoding` header.
        let accept_encoding = resolver.allowed_encodings
          & request
            .headers()
            .get(header::ACCEPT_ENCODING)
            .map(AcceptEncoding::from_header_value)
            .unwrap_or(AcceptEncoding::none());

        println!("RS: Resolving path '{}' in {:?}", self.path, self.root);

        handle.block_on(async { resolver.resolve_path(&self.path, accept_encoding).await })?
      }
      _ => ResolveResult::MethodNotMatched,
    };

    println!("RS: Result: {result:?}");

    let response = hyper_staticfile::ResponseBuilder::new()
      .request(&request)
      .build(result)
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?
      .map(|b| b.into());

    self.response.req().with_inner_mut(|w_req| {
      w_req.set_inner(request);
      Ok(())
    })?;

    Ok(response)
  }

  fn resolve(&mut self, _env: Env, response: Self::Output) -> Result<Self::JsValue> {
    self.response.with_inner(|w_res| {
      w_res.set_inner(response);
      Ok(())
    })
  }
}
