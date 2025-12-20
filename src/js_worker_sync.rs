use deno_core::{JsRuntime, RuntimeOptions};
use tokio::sync::mpsc;

use crate::{
    execute_handler::execute_handler, load_handler::load_handler, ops::custom_ops,
    worker_message::WorkerMessage,
};

// JavaScript worker that owns the JsRuntime
pub fn js_worker_sync(mut rx: mpsc::UnboundedReceiver<WorkerMessage>) {
    // Create a Tokio runtime for this thread to handle async ops
    let tokio_runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime");

    // Create JS runtime in this thread - it's !Send so stays here
    let mut runtime = JsRuntime::new(RuntimeOptions {
        extensions: vec![custom_ops::init()],
        ..Default::default()
    });

    println!("✓ JavaScript worker started");

    // Use blocking recv since we're in a sync context
    while let Some(msg) = rx.blocking_recv() {
        match msg {
            WorkerMessage::LoadHandler {
                handler_name,
                code,
                response_tx,
            } => {
                let result = load_handler(&mut runtime, &handler_name, &code);
                let _ = response_tx.send(result.map_err(|e| e.to_string()));
            }
            WorkerMessage::Execute {
                handler_name,
                params,
                query,
                body,
                response_tx,
            } => {
                // Use the Tokio runtime to run the async function
                let result = tokio_runtime.block_on(execute_handler(
                    &mut runtime,
                    &handler_name,
                    params,
                    query,
                    body,
                ));
                let _ = response_tx.send(result.map_err(|e| e.to_string()));
            }
        }
    }
}
