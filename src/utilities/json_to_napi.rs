use napi::{Env, Result, Unknown};
use serde_json::Value;

pub fn json_to_napi(env: &Env, json_value: Value) -> Result<Unknown<'static>> {
  env.to_js_value(&json_value)
}
