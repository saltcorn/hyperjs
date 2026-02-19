/// Represents a single byte range with start and end positions
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Range {
  pub start: usize,
  pub end: usize,
}

/// Represents a collection of ranges with a type identifier
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ranges {
  pub ranges: Vec<Range>,
  pub range_type: String,
}

/// Error codes for range parsing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangeParseError {
  /// Invalid range format (malformed)
  InvalidFormat,
  /// Unsatisfiable range
  Unsatisfiable,
}

/// Options for range parsing
#[derive(Debug, Clone, Default)]
pub struct RangeOptions {
  pub combine: bool,
}

/// Parse "Range" header string relative to the given file size.
///
/// # Arguments
///
/// * `size` - The total size of the resource
/// * `str` - The Range header value
/// * `options` - Optional parsing options
///
/// # Returns
///
/// * `Ok(Ranges)` - Successfully parsed ranges
/// * `Err(RangeParseError)` - Error during parsing
pub fn parse_range(
  size: usize,
  str: &str,
  options: Option<RangeOptions>,
) -> Result<Ranges, RangeParseError> {
  let index = str.find('=').ok_or(RangeParseError::InvalidFormat)?;

  // Split the range string
  let range_type = str[..index].to_string();
  let ranges_str = &str[index + 1..];
  let arr: Vec<&str> = ranges_str.split(',').collect();

  let mut ranges = Vec::new();
  let mut valid = false;

  // Parse all ranges
  for range_str in arr {
    let index_of = match range_str.find('-') {
      Some(idx) => idx,
      None => continue,
    };

    let start_str = range_str[..index_of].trim();
    let end_str = range_str[index_of + 1..].trim();

    let mut start = parse_pos(start_str);
    let mut end = parse_pos(end_str);

    if start_str.is_empty() {
      if let Some(end_val) = end {
        start = Some(size.saturating_sub(end_val));
        end = Some(size - 1);
      }
    } else if end_str.is_empty() {
      end = Some(size - 1);
    }

    // Limit last-byte-pos to current length
    if let Some(end_val) = end
      && end_val > size - 1
    {
      end = Some(size - 1);
    }

    // Invalid format range
    let (start_val, end_val) = match (start, end) {
      (Some(s), Some(e)) => (s, e),
      _ => continue,
    };

    // Skip unsatisfiable ranges
    if start_val > end_val || start_val >= size {
      valid = true;
      continue;
    }

    // Add range
    ranges.push(Range {
      start: start_val,
      end: end_val,
    });
  }

  if ranges.is_empty() {
    return if valid {
      Err(RangeParseError::Unsatisfiable)
    } else {
      Err(RangeParseError::InvalidFormat)
    };
  }

  let ranges = if options.map(|o| o.combine).unwrap_or(false) {
    combine_ranges(ranges)
  } else {
    ranges
  };

  Ok(Ranges { ranges, range_type })
}

/// Parse string to unsigned integer
fn parse_pos(s: &str) -> Option<usize> {
  if s.chars().all(|c| c.is_ascii_digit()) {
    s.parse().ok()
  } else {
    None
  }
}

/// Combine overlapping & adjacent ranges
fn combine_ranges(ranges: Vec<Range>) -> Vec<Range> {
  #[derive(Debug, Clone)]
  struct IndexedRange {
    start: usize,
    end: usize,
    index: usize,
  }

  let mut ordered: Vec<IndexedRange> = ranges
    .into_iter()
    .enumerate()
    .map(|(index, range)| IndexedRange {
      start: range.start,
      end: range.end,
      index,
    })
    .collect();

  // Sort by range start
  ordered.sort_by_key(|r| r.start);

  let mut j = 0;
  for i in 1..ordered.len() {
    let range_start = ordered[i].start;
    let range_end = ordered[i].end;
    let range_index = ordered[i].index;

    if range_start > ordered[j].end + 1 {
      // Next range
      j += 1;
      ordered[j] = IndexedRange {
        start: range_start,
        end: range_end,
        index: range_index,
      };
    } else if range_end > ordered[j].end {
      // Extend range
      ordered[j].end = range_end;
      ordered[j].index = ordered[j].index.min(range_index);
    }
  }

  // Trim ordered array
  ordered.truncate(j + 1);

  // Sort by original index
  ordered.sort_by_key(|r| r.index);

  // Generate combined range without index
  ordered
    .into_iter()
    .map(|r| Range {
      start: r.start,
      end: r.end,
    })
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_return_error_for_completely_empty_header() {
    let result = parse_range(200, "", None);
    assert_eq!(result, Err(RangeParseError::InvalidFormat));
  }

  #[test]
  fn test_return_error_for_range_missing_dash() {
    assert_eq!(
      parse_range(200, "bytes=100200", None),
      Err(RangeParseError::InvalidFormat)
    );
    assert_eq!(
      parse_range(200, "bytes=,100200", None),
      Err(RangeParseError::InvalidFormat)
    );
  }

  #[test]
  fn test_return_error_for_invalid_str() {
    assert_eq!(
      parse_range(200, "malformed", None),
      Err(RangeParseError::InvalidFormat)
    );
  }

  #[test]
  fn test_return_error_for_invalid_start_byte_position() {
    assert_eq!(
      parse_range(200, "bytes=x-100", None),
      Err(RangeParseError::InvalidFormat)
    );
  }

  #[test]
  fn test_return_error_for_invalid_end_byte_position() {
    assert_eq!(
      parse_range(200, "bytes=100-x", None),
      Err(RangeParseError::InvalidFormat)
    );
  }

  #[test]
  fn test_return_error_for_invalid_range_format() {
    assert_eq!(
      parse_range(200, "bytes=--100", None),
      Err(RangeParseError::InvalidFormat)
    );
    assert_eq!(
      parse_range(200, "bytes=100--200", None),
      Err(RangeParseError::InvalidFormat)
    );
    assert_eq!(
      parse_range(200, "bytes=-", None),
      Err(RangeParseError::InvalidFormat)
    );
    assert_eq!(
      parse_range(200, "bytes= - ", None),
      Err(RangeParseError::InvalidFormat)
    );
  }

  #[test]
  fn test_return_error_for_empty_range_value() {
    assert_eq!(
      parse_range(200, "bytes=", None),
      Err(RangeParseError::InvalidFormat)
    );
    assert_eq!(
      parse_range(200, "bytes=,", None),
      Err(RangeParseError::InvalidFormat)
    );
    assert_eq!(
      parse_range(200, "bytes= , , ", None),
      Err(RangeParseError::InvalidFormat)
    );
  }

  #[test]
  fn test_return_error_with_multiple_dashes_in_range() {
    assert_eq!(
      parse_range(200, "bytes=100-200-300", None),
      Err(RangeParseError::InvalidFormat)
    );
  }

  #[test]
  fn test_return_error_for_negative_start_byte_position() {
    assert_eq!(
      parse_range(200, "bytes=-100-150", None),
      Err(RangeParseError::InvalidFormat)
    );
  }

  #[test]
  fn test_return_error_for_invalid_number_format() {
    assert_eq!(
      parse_range(200, "bytes=01a-150", None),
      Err(RangeParseError::InvalidFormat)
    );
    assert_eq!(
      parse_range(200, "bytes=100-15b0", None),
      Err(RangeParseError::InvalidFormat)
    );
  }

  #[test]
  fn test_return_error_when_all_multiple_ranges_have_invalid_format() {
    assert_eq!(
      parse_range(200, "bytes=y-v,x-", None),
      Err(RangeParseError::InvalidFormat)
    );
    assert_eq!(
      parse_range(200, "bytes=abc-def,ghi-jkl", None),
      Err(RangeParseError::InvalidFormat)
    );
    assert_eq!(
      parse_range(200, "bytes=x-,y-,z-", None),
      Err(RangeParseError::InvalidFormat)
    );
  }

  #[test]
  fn test_return_error_for_unsatisfiable_range() {
    assert_eq!(
      parse_range(200, "bytes=500-600", None),
      Err(RangeParseError::Unsatisfiable)
    );
  }

  #[test]
  fn test_return_error_for_unsatisfiable_range_with_multiple_ranges() {
    assert_eq!(
      parse_range(200, "bytes=500-600,601-700", None),
      Err(RangeParseError::Unsatisfiable)
    );
  }

  #[test]
  fn test_return_error_if_all_specified_ranges_are_invalid() {
    assert_eq!(
      parse_range(200, "bytes=500-20", None),
      Err(RangeParseError::Unsatisfiable)
    );
    assert_eq!(
      parse_range(200, "bytes=500-999", None),
      Err(RangeParseError::Unsatisfiable)
    );
    assert_eq!(
      parse_range(200, "bytes=500-999,1000-1499", None),
      Err(RangeParseError::Unsatisfiable)
    );
  }

  #[test]
  fn test_return_error_for_mixed_invalid_and_unsatisfiable_ranges() {
    assert_eq!(
      parse_range(200, "bytes=abc-def,500-999", None),
      Err(RangeParseError::Unsatisfiable)
    );
    assert_eq!(
      parse_range(200, "bytes=500-999,xyz-uvw", None),
      Err(RangeParseError::Unsatisfiable)
    );
    assert_eq!(
      parse_range(200, "bytes=abc-def,500-999,xyz-uvw", None),
      Err(RangeParseError::Unsatisfiable)
    );
  }

  #[test]
  fn test_parse_str() {
    let result = parse_range(1000, "bytes=0-499", None).unwrap();
    assert_eq!(result.range_type, "bytes");
    assert_eq!(result.ranges.len(), 1);
    assert_eq!(result.ranges[0], Range { start: 0, end: 499 });
  }

  #[test]
  fn test_cap_end_at_size() {
    let result = parse_range(200, "bytes=0-499", None).unwrap();
    assert_eq!(result.range_type, "bytes");
    assert_eq!(result.ranges.len(), 1);
    assert_eq!(result.ranges[0], Range { start: 0, end: 199 });
  }

  #[test]
  fn test_parse_str_middle_range() {
    let result = parse_range(1000, "bytes=40-80", None).unwrap();
    assert_eq!(result.range_type, "bytes");
    assert_eq!(result.ranges.len(), 1);
    assert_eq!(result.ranges[0], Range { start: 40, end: 80 });
  }

  #[test]
  fn test_parse_str_asking_for_last_n_bytes() {
    let result = parse_range(1000, "bytes=-400", None).unwrap();
    assert_eq!(result.range_type, "bytes");
    assert_eq!(result.ranges.len(), 1);
    assert_eq!(
      result.ranges[0],
      Range {
        start: 600,
        end: 999
      }
    );
  }

  #[test]
  fn test_parse_str_with_only_start() {
    let result = parse_range(1000, "bytes=400-", None).unwrap();
    assert_eq!(result.range_type, "bytes");
    assert_eq!(result.ranges.len(), 1);
    assert_eq!(
      result.ranges[0],
      Range {
        start: 400,
        end: 999
      }
    );
  }

  #[test]
  fn test_parse_bytes_0_to_end() {
    let result = parse_range(1000, "bytes=0-", None).unwrap();
    assert_eq!(result.range_type, "bytes");
    assert_eq!(result.ranges.len(), 1);
    assert_eq!(result.ranges[0], Range { start: 0, end: 999 });
  }

  #[test]
  fn test_parse_str_with_no_bytes() {
    let result = parse_range(1000, "bytes=0-0", None).unwrap();
    assert_eq!(result.range_type, "bytes");
    assert_eq!(result.ranges.len(), 1);
    assert_eq!(result.ranges[0], Range { start: 0, end: 0 });
  }

  #[test]
  fn test_parse_str_asking_for_last_byte() {
    let result = parse_range(1000, "bytes=-1", None).unwrap();
    assert_eq!(result.range_type, "bytes");
    assert_eq!(result.ranges.len(), 1);
    assert_eq!(
      result.ranges[0],
      Range {
        start: 999,
        end: 999
      }
    );
  }

  #[test]
  fn test_ignore_invalid_format_range_when_valid_range_exists() {
    let result = parse_range(1000, "bytes=100-200,x-", None).unwrap();
    assert_eq!(result.range_type, "bytes");
    assert_eq!(result.ranges.len(), 1);
    assert_eq!(
      result.ranges[0],
      Range {
        start: 100,
        end: 200
      }
    );
  }

  #[test]
  fn test_ignore_invalid_format_ranges_when_some_are_valid() {
    let result = parse_range(1000, "bytes=x-,0-100,y-", None).unwrap();
    assert_eq!(result.range_type, "bytes");
    assert_eq!(result.ranges.len(), 1);
    assert_eq!(result.ranges[0], Range { start: 0, end: 100 });
  }

  #[test]
  fn test_ignore_invalid_format_ranges_at_different_positions() {
    let result = parse_range(1000, "bytes=0-50,abc-def,100-150", None).unwrap();
    assert_eq!(result.range_type, "bytes");
    assert_eq!(result.ranges.len(), 2);
    assert_eq!(result.ranges[0], Range { start: 0, end: 50 });
    assert_eq!(
      result.ranges[1],
      Range {
        start: 100,
        end: 150
      }
    );
  }

  #[test]
  fn test_parse_str_with_multiple_ranges() {
    let result = parse_range(1000, "bytes=40-80,81-90,-1", None).unwrap();
    assert_eq!(result.range_type, "bytes");
    assert_eq!(result.ranges.len(), 3);
    assert_eq!(result.ranges[0], Range { start: 40, end: 80 });
    assert_eq!(result.ranges[1], Range { start: 81, end: 90 });
    assert_eq!(
      result.ranges[2],
      Range {
        start: 999,
        end: 999
      }
    );
  }

  #[test]
  fn test_parse_str_with_some_invalid_ranges() {
    let result = parse_range(200, "bytes=0-499,1000-,500-999", None).unwrap();
    assert_eq!(result.range_type, "bytes");
    assert_eq!(result.ranges.len(), 1);
    assert_eq!(result.ranges[0], Range { start: 0, end: 199 });
  }

  #[test]
  fn test_parse_str_with_whitespace() {
    let result = parse_range(1000, "bytes=   40-80 , 81-90 , -1 ", None).unwrap();
    assert_eq!(result.range_type, "bytes");
    assert_eq!(result.ranges.len(), 3);
    assert_eq!(result.ranges[0], Range { start: 40, end: 80 });
    assert_eq!(result.ranges[1], Range { start: 81, end: 90 });
    assert_eq!(
      result.ranges[2],
      Range {
        start: 999,
        end: 999
      }
    );
  }

  #[test]
  fn test_parse_non_byte_range() {
    let result = parse_range(1000, "items=0-5", None).unwrap();
    assert_eq!(result.range_type, "items");
    assert_eq!(result.ranges.len(), 1);
    assert_eq!(result.ranges[0], Range { start: 0, end: 5 });
  }

  #[test]
  fn test_combine_overlapping_ranges() {
    let options = RangeOptions { combine: true };
    let result = parse_range(150, "bytes=0-4,90-99,5-75,100-199,101-102", Some(options)).unwrap();
    assert_eq!(result.range_type, "bytes");
    assert_eq!(result.ranges.len(), 2);
    assert_eq!(result.ranges[0], Range { start: 0, end: 75 });
    assert_eq!(
      result.ranges[1],
      Range {
        start: 90,
        end: 149
      }
    );
  }

  #[test]
  fn test_combine_retain_original_order() {
    let options = RangeOptions { combine: true };
    let result = parse_range(150, "bytes=-1,20-100,0-1,101-120", Some(options)).unwrap();
    assert_eq!(result.range_type, "bytes");
    assert_eq!(result.ranges.len(), 3);
    assert_eq!(
      result.ranges[0],
      Range {
        start: 149,
        end: 149
      }
    );
    assert_eq!(
      result.ranges[1],
      Range {
        start: 20,
        end: 120
      }
    );
    assert_eq!(result.ranges[2], Range { start: 0, end: 1 });
  }

  #[test]
  fn test_ignore_whitespace_only_invalid_ranges_when_valid_present() {
    let result = parse_range(1000, "bytes= , 0-10", None).unwrap();
    assert_eq!(result.range_type, "bytes");
    assert_eq!(result.ranges.len(), 1);
    assert_eq!(result.ranges[0], Range { start: 0, end: 10 });
  }
}
