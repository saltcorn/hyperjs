mod task;

use std::{path::Path, sync::Arc};

use napi::{
  bindgen_prelude::*,
  threadsafe_function::{ThreadsafeCallContext, ThreadsafeFunction},
};
use napi_derive::napi;

use crate::{request::Request, response::Response, utilities::FileSendOptions};
use task::StaticMiddlewareTask;

type SetHeadersFnParams = FnArgs<(Response, String, FileStat)>;

type ThreadsafeSetHeadersFn =
  ThreadsafeFunction<SetHeadersFnParams, (), SetHeadersFnParams, Status, false, false, 0>;

type JsSetHeadersFn<'a> = Function<'a, SetHeadersFnParams, ()>;

#[napi(object)]
pub struct FileStat {}

#[napi(object)]
pub struct JsStaticOptions<'a> {
  /// Determines how dotfiles (files or directories that begin with a dot “.”)
  /// are treated.
  ///
  /// Possible values for this option are:
  ///
  /// - “allow” - No special treatment for dotfiles.
  /// - “deny” - Deny a request for a dotfile, respond with `403`, then call
  /// `next()`.
  /// - “ignore” - Act as if the dotfile does not exist, respond with `404`,
  /// then call `next()`.
  ///
  /// Default: "ignore"
  pub dotfiles: Option<String>,

  /// Enable or disable etag generation
  ///
  /// Default: true
  pub etag: Option<bool>,

  /// Sets file extension fallbacks: If a file is not found, search for files
  /// with the specified extensions and serve the first one found. Example:
  /// `['html', 'htm']`.
  ///
  /// Default: false
  pub extensions: Option<Either3<String, Vec<String>, bool>>,

  /// Let client errors fall-through as unhandled requests, otherwise forward a
  /// client error.
  ///
  /// When this option is `true`, client errors such as a bad request or a
  /// request to a non-existent file will cause this middleware to simply call
  /// `next()` to invoke the next middleware in the stack. When `false`, these
  /// errors (even `404`s), will invoke `next(err)`.
  ///
  /// Set this option to `true` so you can map multiple physical directories to
  /// the same web address or for routes to fill in non-existent files.
  ///
  /// Use `false` if you have mounted this middleware at a path designed to be
  /// strictly a single file system directory, which allows for
  /// short-circuiting `404`s for less overhead. This middleware will also
  /// reply to all methods.
  ///
  /// Default: true
  pub fallthrough: Option<bool>,

  /// Enable or disable the `immutable` directive in the `Cache-Control`
  /// response header. If enabled, the `maxAge` option should also be specified
  /// to enable caching. The `immutable` directive will prevent supported
  /// clients from making conditional requests during the life of the `maxAge`
  /// option to check if the file has changed.
  ///
  /// Default: false
  pub immutable: Option<bool>,

  /// Sends the specified directory index file. Set to `false` to disable
  /// directory indexing.
  ///
  /// Default: "index.html"
  pub index: Option<Either3<String, Vec<String>, bool>>,

  /// Set the `Last-Modified` header to the last modified date of the file on
  /// the OS.
  ///
  /// Default: true
  pub last_modified: Option<bool>,

  /// Set the max-age property of the Cache-Control header in milliseconds or a
  /// string in [jackdauer](https://docs.rs/jackdauer/0.1.2/jackdauer/) format.
  ///
  /// Default: 0
  pub max_age: Option<u32>,

  /// Redirect to trailing “/” when the pathname is a directory.
  ///
  /// Default: true
  pub redirect: Option<bool>,

  /// Function for setting HTTP headers to serve with the file.
  ///
  /// For this option, specify a function to set custom response headers.
  /// Alterations to the headers must occur synchronously.
  ///
  /// The signature of the function is:
  ///
  /// ```javascript
  /// fn(res, path, stat)
  /// ```
  ///
  /// Arguments:
  ///
  /// - `res`, the response object.
  /// - `path`, the file path that is being sent.
  /// - `stat`, the `stat` object of the file that is being sent.
  ///
  /// Default: undefined
  pub set_headers: Option<JsSetHeadersFn<'a>>,

  /// Enable or disable accepting ranged requests. Disabling this will not send
  /// the `Accept-Ranges` header and will ignore the contents of the `Range`
  /// request header.
  ///
  /// Default: true
  pub accept_ranges: Option<bool>,

  /// Enable or disable setting the `Cache-Control` response header. Disabling
  /// this will ignore the `immutable` and `maxAge` options.
  ///
  /// Default: true
  pub cache_control: Option<bool>,
}

impl<'a> TryFrom<&JsStaticOptions<'a>> for StaticOptions {
  type Error = Error;

  fn try_from(value: &JsStaticOptions<'a>) -> Result<Self> {
    let mut text_options = StaticOptions::default();

    if let Some(dotfiles) = &value.dotfiles {
      match dotfiles.as_str() {
        "allow" => text_options.dotfiles = "allow".to_owned(),
        "deny" => text_options.dotfiles = "deny".to_owned(),
        "ignore" => text_options.dotfiles = "ignore".to_owned(),
        _ => {
          return Err(Error::new(
            Status::InvalidArg,
            "Invalid value for dotfiles. Valid values: allow, deny, ignore",
          ));
        }
      }
    }

    if let Some(etag) = value.etag {
      text_options.etag = etag;
    }

    if let Some(extensions) = &value.extensions {
      match extensions {
        Either3::A(extension) => text_options.extensions = Some(vec![extension.to_owned()]),
        Either3::B(extensions) => text_options.extensions = Some(extensions.to_owned()),
        Either3::C(_) => text_options.extensions = None,
      }
    }

    if let Some(fallthrough) = value.fallthrough {
      text_options.fallthrough = fallthrough;
    }

    if let Some(immutable) = value.immutable {
      text_options.immutable = immutable;
    }

    if let Some(index) = &value.index {
      match index {
        Either3::A(index) => text_options.index = Some(vec![index.to_owned()]),
        Either3::B(indices) => text_options.index = Some(indices.to_owned()),
        Either3::C(_) => text_options.index = None,
      }
    }

    if let Some(last_modified) = value.last_modified {
      text_options.last_modified = last_modified;
    }

    if let Some(max_age) = value.max_age {
      text_options.max_age = max_age as u64;
    }

    if let Some(redirect) = value.redirect {
      text_options.redirect = redirect;
    }

    if let Some(set_headers_fn) = &value.set_headers {
      let tsfn = set_headers_fn
        .build_threadsafe_function()
        .build_callback(|ctx: ThreadsafeCallContext<SetHeadersFnParams>| Ok(ctx.value))?;
      text_options.set_headers = Some(Arc::new(tsfn));
    }

    if let Some(accept_ranges) = value.accept_ranges {
      text_options.accept_ranges = accept_ranges;
    }

    if let Some(cache_control) = value.cache_control {
      text_options.cache_control = cache_control;
    }

    Ok(text_options)
  }
}

#[derive(Clone)]
pub struct StaticOptions {
  pub dotfiles: String,
  pub etag: bool,
  pub extensions: Option<Vec<String>>,
  pub fallthrough: bool,
  pub immutable: bool,
  pub index: Option<Vec<String>>,
  pub last_modified: bool,
  pub max_age: u64,
  pub redirect: bool,
  pub set_headers: Option<Arc<ThreadsafeSetHeadersFn>>,
  pub accept_ranges: bool,
  pub cache_control: bool,
}

impl Default for StaticOptions {
  fn default() -> Self {
    Self {
      dotfiles: "ignore".to_owned(),
      etag: true,
      extensions: None,
      fallthrough: true,
      immutable: false,
      index: Some(vec!["index.html".to_owned()]),
      last_modified: true,
      max_age: 0,
      redirect: true,
      set_headers: None,
      accept_ranges: true,
      cache_control: true,
    }
  }
}

impl From<&StaticOptions> for FileSendOptions {
  fn from(value: &StaticOptions) -> Self {
    FileSendOptions {
      max_age: value.max_age,
      cache_control: value.cache_control,
      etag: value.etag,
      last_modified: value.last_modified,
      accept_ranges: value.accept_ranges,
      immutable: value.immutable,
      index: value.index.to_owned(),
      extensions: value.extensions.to_owned(),
      dotfiles: value.dotfiles.to_owned(),
      ..FileSendOptions::default()
    }
  }
}

/// This is a built-in middleware function. It serves static files and is based
/// on [hyper-staticfile](https://docs.rs/hyper-staticfile/latest/hyper_staticfile/).
///
/// > NOTE: For best results,
/// > [use a reverse proxy](https://expressjs.com/en/advanced/best-practice-performance.html#use-a-reverse-proxy)
/// > cache to improve performance of serving static assets.
///
/// The `root` argument specifies the root directory from which to serve static
/// assets. The function determines the file to serve by combining `req.url`
/// with the provided `root` directory. When a file is not found, instead of
/// sending a 404 response, it instead calls `next()` to move on to the next
/// middleware, allowing for stacking and fall-backs.
///
/// Example of using the StaticMiddleware
/// Here is an example of using the Static middleware with an elaborate options object:
///
/// ```javascript
/// const options = {
///   dotfiles: 'ignore',
///   etag: false,
///   extensions: ['htm', 'html'],
///   index: false,
///   maxAge: '1d',
///   redirect: false,
///   setHeaders (res, path, stat) {
///     res.set('x-timestamp', Date.now())
///   }
/// }
///
/// app.use(new StaticMiddleware('public', options))
/// ```
#[napi]
pub struct StaticMiddleware {
  root: String,
  options: StaticOptions,
}

#[napi]
impl StaticMiddleware {
  #[napi(constructor)]
  pub fn new(root: String, options: Option<JsStaticOptions>) -> Result<Self> {
    Ok(StaticMiddleware {
      root,
      options: match &options {
        Some(options) => StaticOptions::try_from(options)?,
        None => StaticOptions::default(),
      },
    })
  }

  #[napi]
  pub fn run(
    &self,
    request: &Request,
    response: &mut Response,
  ) -> Result<AsyncTask<StaticMiddlewareTask>> {
    println!("Static Middleware | Called!");

    Ok(AsyncTask::new(StaticMiddlewareTask {
      response: response.to_owned(),
      request: request.to_owned(),
      root: Path::new(&self.root).to_owned(),
      options: self.options.to_owned(),
    }))
  }
}
