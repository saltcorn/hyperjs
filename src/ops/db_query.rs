use deno_core::op2;
use serde_json::{Value, json};
use sqlx::{Column, Row, SqlitePool};
use std::sync::Arc;

use crate::DB_POOL;

// Database query op - executes a SQL query and returns results as JSON
#[op2(async)]
#[string]
pub async fn op_db_query(#[string] query: String, #[serde] params: serde_json::Value) -> String {
    // Get pool from global static
    let pool = match DB_POOL.get() {
        Some(pool) => pool.clone(),
        None => {
            return json!({"error": "Database not initialized", "success": false}).to_string();
        }
    };

    // Now execute the query with the cloned pool
    execute_db_query(pool, query, params).await
}

pub async fn execute_db_query(
    pool: Arc<SqlitePool>,
    query: String,
    params: serde_json::Value,
) -> String {
    println!("[DB] Executing query: {} with params: {:?}", query, params);

    // Helper function to handle errors and return JSON error string
    let handle_error =
        |msg: &str| -> String { json!({"error": msg, "success": false}).to_string() };

    // Parse parameters array
    let param_array = match params.as_array() {
        Some(arr) => arr,
        None => return handle_error("params must be an array"),
    };

    // Execute query based on type (SELECT vs other)
    if query.trim().to_uppercase().starts_with("SELECT") {
        // For SELECT queries, fetch all rows and return as JSON array
        let mut sql_query = sqlx::query(&query);

        // Bind parameters
        for param in param_array {
            sql_query = match param {
                Value::String(s) => sql_query.bind(s.clone()),
                Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        sql_query.bind(i)
                    } else if let Some(f) = n.as_f64() {
                        sql_query.bind(f)
                    } else {
                        return handle_error("Invalid number type");
                    }
                }
                Value::Bool(b) => sql_query.bind(b),
                Value::Null => sql_query.bind(Option::<String>::None),
                _ => return handle_error("Unsupported parameter type"),
            };
        }

        let rows = match sql_query.fetch_all(&*pool).await {
            Ok(rows) => rows,
            Err(e) => return handle_error(&format!("Database query failed: {}", e)),
        };

        // Convert rows to JSON
        let mut results = Vec::new();
        for row in rows {
            let mut obj = serde_json::Map::new();

            // Dynamically extract columns
            for (idx, column) in row.columns().iter().enumerate() {
                let column_name = column.name();

                // Try to get value as different types
                let value: Value = if let Ok(val) = row.try_get::<String, _>(idx) {
                    Value::String(val)
                } else if let Ok(val) = row.try_get::<i64, _>(idx) {
                    Value::Number(val.into())
                } else if let Ok(val) = row.try_get::<f64, _>(idx) {
                    Value::Number(serde_json::Number::from_f64(val).unwrap_or(0.into()))
                } else if let Ok(val) = row.try_get::<bool, _>(idx) {
                    Value::Bool(val)
                } else if let Ok(val) = row.try_get::<Option<String>, _>(idx) {
                    match val {
                        Some(val) => Value::String(val),
                        None => Value::Null,
                    }
                } else {
                    Value::Null
                };

                obj.insert(column_name.to_string(), value);
            }

            results.push(Value::Object(obj));
        }

        match serde_json::to_string(&results) {
            Ok(json_str) => json_str,
            Err(e) => handle_error(&format!("JSON serialization failed: {}", e)),
        }
    } else {
        // For INSERT, UPDATE, DELETE queries, execute and return rows affected
        let mut sql_query = sqlx::query(&query);

        // Bind parameters
        for param in param_array {
            sql_query = match param {
                Value::String(s) => sql_query.bind(s.clone()),
                Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        sql_query.bind(i)
                    } else if let Some(f) = n.as_f64() {
                        sql_query.bind(f)
                    } else {
                        return handle_error("Invalid number type");
                    }
                }
                Value::Bool(b) => sql_query.bind(b),
                Value::Null => sql_query.bind(Option::<String>::None),
                _ => return handle_error("Unsupported parameter type"),
            };
        }

        let result = match sql_query.execute(&*pool).await {
            Ok(result) => result,
            Err(e) => return handle_error(&format!("Database query failed: {}", e)),
        };

        let rows_affected = result.rows_affected();

        json!({
            "rowsAffected": rows_affected,
            "success": true
        })
        .to_string()
    }
}
