use deno_core::{JsRuntime, error::AnyError};
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::RESULT_STORAGE;

// Execute a handler
pub async fn execute_handler(
    runtime: &mut JsRuntime,
    handler_name: &str,
    params: HashMap<String, String>,
    query: HashMap<String, String>,
    body: Option<Value>,
) -> Result<(u16, String), AnyError> {
    println!("[DEBUG] Executing handler: {}", handler_name);

    let req_json = json!({
        "params": params,
        "query": query,
        "body": body,
    });

    let code = format!(
        r#"
        (async () => {{
            console.log('[JS] Starting handler execution');
            const req = {};
            const res = {{
                statusCode: 200,
                body: null,
                headers: {{}},
                send(data) {{
                    console.log('[JS] send() called with:', data);
                    this.body = typeof data === 'string' ? data : JSON.stringify(data);
                }},
                json(data) {{
                    console.log('[JS] json() called with:', data);
                    this.headers['Content-Type'] = 'application/json';
                    this.body = JSON.stringify(data);
                }},
                status(code) {{
                    console.log('[JS] status() called with:', code);
                    this.statusCode = code;
                    return this;
                }}
            }};

            try {{
                console.log('[JS] Calling handler');
                await globalThis.{}(req, res);
                console.log('[JS] Handler completed, status:', res.statusCode, 'body:', res.body);
                // Use our custom op to store the result
                Deno.core.ops.op_store_result(res.statusCode, res.body || '');
                console.log('[JS] Result stored');
                return true;
            }} catch (err) {{
                console.error('[JS] Handler error:', err.message);
                Deno.core.ops.op_store_result(500, 'Internal Server Error: ' + err.message);
                return false;
            }}
        }})()
        "#,
        req_json, handler_name
    );

    println!("[DEBUG] Executing script");
    let _result_val = runtime.execute_script("<execute>", code)?;

    println!("[DEBUG] Running event loop to completion");
    // Run the event loop to completion - this will execute the promise and call our op
    // We don't need to explicitly resolve the promise since we're capturing results via op_store_result
    runtime
        .run_event_loop(deno_core::PollEventLoopOptions::default())
        .await?;

    println!("[DEBUG] Retrieving result from storage");
    // Retrieve the result from our static storage
    let result = RESULT_STORAGE
        .lock()
        .unwrap()
        .take()
        .ok_or_else(|| AnyError::msg("No result stored"))?;

    println!("[DEBUG] Result retrieved: {:?}", result);
    Ok(result)
}
