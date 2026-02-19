use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi]
pub fn serialize_napi_object(env: Env, obj: Object) -> Result<String> {
  let global = env.get_global()?;
  let json: Object = global.get_named_property("JSON")?;
  let stringify: Function = json.get_named_property("stringify")?;

  // Convert Object to Unknown
  let obj_unknown = unsafe { Unknown::from_napi_value(env.raw(), obj.raw())? };
  let result: Unknown = stringify.call(obj_unknown)?;

  result
    .coerce_to_string()?
    .into_utf8()?
    .as_str()
    .map(|s| s.to_owned())
}
