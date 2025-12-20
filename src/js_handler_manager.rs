use serde_json::Value;
use std::collections::HashMap;
use tokio::sync::{mpsc, oneshot};

use crate::{js_worker_sync::js_worker_sync, worker_message::WorkerMessage};

// Handler manager that communicates with the worker
#[derive(Clone)]
pub struct JsHandlerManager {
    tx: mpsc::UnboundedSender<WorkerMessage>,
}

impl JsHandlerManager {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        // Spawn worker in a dedicated OS thread (not tokio task)
        std::thread::spawn(move || {
            js_worker_sync(rx);
        });

        Self { tx }
    }

    pub async fn load_handler(&self, handler_name: &str, code: &str) -> Result<(), String> {
        let (response_tx, response_rx) = oneshot::channel();

        self.tx
            .send(WorkerMessage::LoadHandler {
                handler_name: handler_name.to_string(),
                code: code.to_string(),
                response_tx,
            })
            .map_err(|e| e.to_string())?;

        response_rx.await.map_err(|e| e.to_string())?
    }

    pub async fn execute_handler(
        &self,
        handler_name: &str,
        params: HashMap<String, String>,
        query: HashMap<String, String>,
        body: Option<Value>,
    ) -> Result<(u16, String), String> {
        let (response_tx, response_rx) = oneshot::channel();

        self.tx
            .send(WorkerMessage::Execute {
                handler_name: handler_name.to_string(),
                params,
                query,
                body,
                response_tx,
            })
            .map_err(|e| e.to_string())?;

        response_rx.await.map_err(|e| e.to_string())?
    }
}
