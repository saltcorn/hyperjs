use deno_core::{JsRuntime, error::AnyError};

// Load a handler function
pub fn load_handler(
    runtime: &mut JsRuntime,
    handler_name: &str,
    js_code: &str,
) -> Result<(), AnyError> {
    let code = format!(
        r#"
        globalThis.{} = {};
        "#,
        handler_name, js_code
    );

    runtime.execute_script(format!("<handler:{}>", handler_name), code)?;
    Ok(())
}
