mod db_query;

use deno_core::{extension, op2};

use crate::RESULT_STORAGE;
use db_query::op_db_query;

// Define custom ops
#[op2(fast)]
pub fn op_log(#[string] message: String) {
    println!("[JS Log] {}", message);
}

// Op to store the handler result
#[op2(fast)]
pub fn op_store_result(#[smi] status: i32, #[string] body: String) {
    let mut storage = RESULT_STORAGE.lock().unwrap();
    *storage = Some((status as u16, body));
}

// Async op for simulating delays (like setTimeout)
#[op2(async)]
pub async fn op_sleep(#[smi] ms: i32) {
    tokio::time::sleep(tokio::time::Duration::from_millis(ms as u64)).await;
}

// Create extension with ops
extension!(
    custom_ops,
    ops = [op_log, op_store_result, op_sleep, op_db_query]
);
