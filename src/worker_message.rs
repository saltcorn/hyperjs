use serde_json::Value;
use std::collections::HashMap;
use tokio::sync::oneshot;

// Message types for worker communication
#[derive(Debug)]
pub enum WorkerMessage {
    Execute {
        handler_name: String,
        params: HashMap<String, String>,
        query: HashMap<String, String>,
        body: Option<Value>,
        response_tx: oneshot::Sender<Result<(u16, String), String>>,
    },
    LoadHandler {
        handler_name: String,
        code: String,
        response_tx: oneshot::Sender<Result<(), String>>,
    },
}
