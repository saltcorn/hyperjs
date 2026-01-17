use napi::bindgen_prelude::*;
use serde_json::Value as JsonValue;

fn json_array_to_string_vec(value: Vec<JsonValue>) -> Result<Vec<String>> {
  let mut values = Vec::with_capacity(value.len());
  for json_value in value {
    values.push(json_scalar_to_string(json_value)?);
  }
  Ok(values)
}

fn json_scalar_to_string(value: JsonValue) -> Result<String> {
  Ok(match value {
    JsonValue::Null => "null".to_owned(),
    JsonValue::Bool(val) => val.to_string(),
    JsonValue::Number(number) => number.to_string(),
    JsonValue::String(val) => val,
    JsonValue::Array(_) | JsonValue::Object(_) => {
      return Err(Error::new(
        Status::GenericFailure,
        "Supplied value is not a JSON value scalar.",
      ));
    }
  })
}

pub fn json_value_as_string(value: JsonValue) -> Result<Either<String, Vec<String>>> {
  Ok(match value {
    JsonValue::Null | JsonValue::Bool(_) | JsonValue::Number(_) | JsonValue::String(_) => {
      Either::A(json_scalar_to_string(value)?)
    }
    JsonValue::Array(values) => Either::B(json_array_to_string_vec(values)?),
    JsonValue::Object(_map) => todo!(),
  })
}
