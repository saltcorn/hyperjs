use hyper::http::status::StatusCode as LibStatusCode;
use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi]
pub struct StatusCode {
  inner: LibStatusCode,
}

impl From<LibStatusCode> for StatusCode {
  fn from(value: LibStatusCode) -> Self {
    Self { inner: value }
  }
}

impl From<&StatusCode> for LibStatusCode {
  fn from(value: &StatusCode) -> Self {
    value.inner.to_owned()
  }
}

#[napi]
impl StatusCode {
  #[napi(factory)]
  pub fn from_u16(src: u16) -> Result<Self> {
    let status_code =
      LibStatusCode::from_u16(src).map_err(|e| Error::new(Status::InvalidArg, e.to_string()))?;
    Ok(Self::from(status_code))
  }

  #[napi(factory)]
  pub fn from_bytes(src: Uint8Array) -> Result<Self> {
    let status_code = LibStatusCode::from_bytes(src.as_ref())
      .map_err(|e| Error::new(Status::InvalidArg, e.to_string()))?;
    Ok(Self::from(status_code))
  }

  #[napi(js_name = "toString")]
  pub fn as_js_string(&mut self) -> String {
    self.inner.as_str().to_owned()
  }

  #[napi]
  pub fn canonical_reason(&mut self) -> Option<String> {
    self.inner.canonical_reason().map(String::from)
  }

  #[napi]
  pub fn is_informational(&mut self) -> bool {
    self.inner.is_informational()
  }

  #[napi]
  pub fn is_success(&mut self) -> bool {
    self.inner.is_success()
  }

  #[napi]
  pub fn is_redirection(&mut self) -> bool {
    self.inner.is_redirection()
  }

  #[napi]
  pub fn is_client_error(&mut self) -> bool {
    self.inner.is_client_error()
  }

  #[napi(factory, js_name = "continue")]
  pub fn cont() -> Self {
    Self::from(LibStatusCode::CONTINUE)
  }

  #[napi(factory)]
  pub fn switching_protocols() -> Self {
    Self::from(LibStatusCode::SWITCHING_PROTOCOLS)
  }

  #[napi(factory)]
  pub fn processing() -> Self {
    Self::from(LibStatusCode::PROCESSING)
  }

  #[napi(factory)]
  pub fn ok() -> Self {
    Self::from(LibStatusCode::OK)
  }

  #[napi(factory)]
  pub fn created() -> Self {
    Self::from(LibStatusCode::CREATED)
  }

  #[napi(factory)]
  pub fn accepted() -> Self {
    Self::from(LibStatusCode::ACCEPTED)
  }

  #[napi(factory)]
  pub fn non_authoritative_information() -> Self {
    Self::from(LibStatusCode::NON_AUTHORITATIVE_INFORMATION)
  }

  #[napi(factory)]
  pub fn no_content() -> Self {
    Self::from(LibStatusCode::NO_CONTENT)
  }

  #[napi(factory)]
  pub fn reset_content() -> Self {
    Self::from(LibStatusCode::RESET_CONTENT)
  }

  #[napi(factory)]
  pub fn partial_content() -> Self {
    Self::from(LibStatusCode::PARTIAL_CONTENT)
  }

  #[napi(factory)]
  pub fn multi_status() -> Self {
    Self::from(LibStatusCode::MULTI_STATUS)
  }

  #[napi(factory)]
  pub fn already_reported() -> Self {
    Self::from(LibStatusCode::ALREADY_REPORTED)
  }

  #[napi(factory)]
  pub fn im_used() -> Self {
    Self::from(LibStatusCode::IM_USED)
  }

  #[napi(factory)]
  pub fn multiple_choices() -> Self {
    Self::from(LibStatusCode::MULTIPLE_CHOICES)
  }

  #[napi(factory)]
  pub fn moved_permanently() -> Self {
    Self::from(LibStatusCode::MOVED_PERMANENTLY)
  }

  #[napi(factory)]
  pub fn found() -> Self {
    Self::from(LibStatusCode::FOUND)
  }

  #[napi(factory)]
  pub fn see_other() -> Self {
    Self::from(LibStatusCode::SEE_OTHER)
  }

  #[napi(factory)]
  pub fn not_modified() -> Self {
    Self::from(LibStatusCode::NOT_MODIFIED)
  }

  #[napi(factory)]
  pub fn use_proxy() -> Self {
    Self::from(LibStatusCode::USE_PROXY)
  }

  #[napi(factory)]
  pub fn temporary_redirect() -> Self {
    Self::from(LibStatusCode::TEMPORARY_REDIRECT)
  }

  #[napi(factory)]
  pub fn permanent_redirect() -> Self {
    Self::from(LibStatusCode::PERMANENT_REDIRECT)
  }

  #[napi(factory)]
  pub fn bad_request() -> Self {
    Self::from(LibStatusCode::BAD_REQUEST)
  }

  #[napi(factory)]
  pub fn unauthorized() -> Self {
    Self::from(LibStatusCode::UNAUTHORIZED)
  }

  #[napi(factory)]
  pub fn payment_required() -> Self {
    Self::from(LibStatusCode::PAYMENT_REQUIRED)
  }

  #[napi(factory)]
  pub fn forbidden() -> Self {
    Self::from(LibStatusCode::FORBIDDEN)
  }

  #[napi(factory)]
  pub fn not_found() -> Self {
    Self::from(LibStatusCode::NOT_FOUND)
  }

  #[napi(factory)]
  pub fn method_not_allowed() -> Self {
    Self::from(LibStatusCode::METHOD_NOT_ALLOWED)
  }

  #[napi(factory)]
  pub fn not_acceptable() -> Self {
    Self::from(LibStatusCode::NOT_ACCEPTABLE)
  }

  #[napi(factory)]
  pub fn proxy_authentication_required() -> Self {
    Self::from(LibStatusCode::PROXY_AUTHENTICATION_REQUIRED)
  }

  #[napi(factory)]
  pub fn request_timeout() -> Self {
    Self::from(LibStatusCode::REQUEST_TIMEOUT)
  }

  #[napi(factory)]
  pub fn conflict() -> Self {
    Self::from(LibStatusCode::CONFLICT)
  }

  #[napi(factory)]
  pub fn gone() -> Self {
    Self::from(LibStatusCode::GONE)
  }

  #[napi(factory)]
  pub fn length_required() -> Self {
    Self::from(LibStatusCode::LENGTH_REQUIRED)
  }

  #[napi(factory)]
  pub fn precondition_failed() -> Self {
    Self::from(LibStatusCode::PRECONDITION_FAILED)
  }

  #[napi(factory)]
  pub fn payload_too_large() -> Self {
    Self::from(LibStatusCode::PAYLOAD_TOO_LARGE)
  }

  #[napi(factory)]
  pub fn uri_too_long() -> Self {
    Self::from(LibStatusCode::URI_TOO_LONG)
  }

  #[napi(factory)]
  pub fn unsupported_media_type() -> Self {
    Self::from(LibStatusCode::UNSUPPORTED_MEDIA_TYPE)
  }

  #[napi(factory)]
  pub fn range_not_satisfiable() -> Self {
    Self::from(LibStatusCode::RANGE_NOT_SATISFIABLE)
  }

  #[napi(factory)]
  pub fn expectation_failed() -> Self {
    Self::from(LibStatusCode::EXPECTATION_FAILED)
  }

  #[napi(factory)]
  pub fn im_a_teapot() -> Self {
    Self::from(LibStatusCode::IM_A_TEAPOT)
  }

  #[napi(factory)]
  pub fn misdirected_request() -> Self {
    Self::from(LibStatusCode::MISDIRECTED_REQUEST)
  }

  #[napi(factory)]
  pub fn unprocessable_entity() -> Self {
    Self::from(LibStatusCode::UNPROCESSABLE_ENTITY)
  }

  #[napi(factory)]
  pub fn locked() -> Self {
    Self::from(LibStatusCode::LOCKED)
  }

  #[napi(factory)]
  pub fn failed_dependency() -> Self {
    Self::from(LibStatusCode::FAILED_DEPENDENCY)
  }

  #[napi(factory)]
  pub fn too_early() -> Self {
    Self::from(LibStatusCode::TOO_EARLY)
  }

  #[napi(factory)]
  pub fn upgrade_required() -> Self {
    Self::from(LibStatusCode::UPGRADE_REQUIRED)
  }

  #[napi(factory)]
  pub fn precondition_required() -> Self {
    Self::from(LibStatusCode::PRECONDITION_REQUIRED)
  }

  #[napi(factory)]
  pub fn too_many_requests() -> Self {
    Self::from(LibStatusCode::TOO_MANY_REQUESTS)
  }

  #[napi(factory)]
  pub fn request_header_fields_too_large() -> Self {
    Self::from(LibStatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE)
  }

  #[napi(factory)]
  pub fn unavailable_for_legal_reasons() -> Self {
    Self::from(LibStatusCode::UNAVAILABLE_FOR_LEGAL_REASONS)
  }

  #[napi(factory)]
  pub fn internal_server_error() -> Self {
    Self::from(LibStatusCode::INTERNAL_SERVER_ERROR)
  }

  #[napi(factory)]
  pub fn not_implemented() -> Self {
    Self::from(LibStatusCode::NOT_IMPLEMENTED)
  }

  #[napi(factory)]
  pub fn bad_gateway() -> Self {
    Self::from(LibStatusCode::BAD_GATEWAY)
  }

  #[napi(factory)]
  pub fn service_unavailable() -> Self {
    Self::from(LibStatusCode::SERVICE_UNAVAILABLE)
  }

  #[napi(factory)]
  pub fn gateway_timeout() -> Self {
    Self::from(LibStatusCode::GATEWAY_TIMEOUT)
  }

  #[napi(factory)]
  pub fn http_version_not_supported() -> Self {
    Self::from(LibStatusCode::HTTP_VERSION_NOT_SUPPORTED)
  }

  #[napi(factory)]
  pub fn variant_also_negotiates() -> Self {
    Self::from(LibStatusCode::VARIANT_ALSO_NEGOTIATES)
  }

  #[napi(factory)]
  pub fn insufficient_storage() -> Self {
    Self::from(LibStatusCode::INSUFFICIENT_STORAGE)
  }

  #[napi(factory)]
  pub fn loop_detected() -> Self {
    Self::from(LibStatusCode::LOOP_DETECTED)
  }

  #[napi(factory)]
  pub fn not_extended() -> Self {
    Self::from(LibStatusCode::NOT_EXTENDED)
  }

  #[napi(factory)]
  pub fn network_authentication_required() -> Self {
    Self::from(LibStatusCode::NETWORK_AUTHENTICATION_REQUIRED)
  }
}
