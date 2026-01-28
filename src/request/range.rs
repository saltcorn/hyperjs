use hyper::header::RANGE;
use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::utilities::{self, parse_range::RangeParseError};

use super::Request;

/// Represents a single byte range with start and end positions
#[napi(object)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Range {
  pub start: u32,
  pub end: u32,
}

impl From<utilities::parse_range::Range> for Range {
  fn from(value: utilities::parse_range::Range) -> Self {
    Self {
      start: value.start as u32,
      end: value.end as u32,
    }
  }
}

/// Represents a collection of ranges with a type identifier
#[napi(object)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ranges {
  pub ranges: Vec<Range>,
  pub range_type: String,
}

impl From<utilities::parse_range::Ranges> for Ranges {
  fn from(value: utilities::parse_range::Ranges) -> Self {
    Self {
      ranges: value.ranges.into_iter().map(|r| r.into()).collect(),
      range_type: value.range_type,
    }
  }
}

/// Options for range parsing
#[napi(object)]
#[derive(Debug, Clone, Default)]
pub struct RangeOptions {
  pub combine: bool,
}

impl From<RangeOptions> for utilities::parse_range::RangeOptions {
  fn from(value: RangeOptions) -> Self {
    Self {
      combine: value.combine,
    }
  }
}

#[napi]
impl Request {
  #[napi(getter)]
  pub fn range(
    &self,
    size: u32,
    options: Option<RangeOptions>,
  ) -> Result<Option<Either<i8, Ranges>>> {
    let Some(range_header_value) =
      self.with_inner_mut(|request| Ok(request.inner()?.headers().get(RANGE).cloned()))?
    else {
      return Ok(None);
    };
    let Ok(range_header_value) = range_header_value.to_str() else {
      return Err(Error::new(
        Status::GenericFailure,
        "Expected value of RANGE header to be a string",
      ));
    };
    let size = size as usize;
    let options: Option<utilities::parse_range::RangeOptions> = options.map(|o| o.into());
    let parse_result = match utilities::parse_range(size, range_header_value, options) {
      Ok(ranges) => Either::B(Ranges::from(ranges)),
      Err(RangeParseError::InvalidFormat) => Either::A(-2),
      Err(RangeParseError::Unsatisfiable) => Either::A(-1),
    };
    Ok(Some(parse_result))
  }
}
