mod execute_handler;
mod handle_route;
mod js_handler_manager;
mod js_worker_sync;
mod load_handler;
mod ops;
mod worker_message;

use sqlx::SqlitePool;
use std::sync::{Arc, Mutex, OnceLock};

use handle_route::handle_route;
use js_handler_manager::JsHandlerManager;

// Include the auto-generated handlers and macro from build.rs
include!(concat!(env!("OUT_DIR"), "/handlers_generated.rs"));

// Shared state for passing results from JS to Rust
static RESULT_STORAGE: Mutex<Option<(u16, String)>> = Mutex::new(None);

// Global database pool with lazy initialization
static DB_POOL: OnceLock<Arc<SqlitePool>> = OnceLock::new();

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize SQLite database
    println!("📊 Initializing SQLite database...");
    let pool = SqlitePool::connect("sqlite::memory:").await?;

    // Create users table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            email TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    // Insert sample data
    sqlx::query("INSERT INTO users (name, email) VALUES (?, ?)")
        .bind("Alice")
        .bind("alice@example.com")
        .execute(&pool)
        .await?;

    sqlx::query("INSERT INTO users (name, email) VALUES (?, ?)")
        .bind("Bob")
        .bind("bob@example.com")
        .execute(&pool)
        .await?;

    sqlx::query("INSERT INTO users (name, email) VALUES (?, ?)")
        .bind("Charlie")
        .bind("charlie@example.com")
        .execute(&pool)
        .await?;

    println!("✓ Database initialized with sample data");

    // Store the pool in global static
    DB_POOL
        .set(Arc::new(pool))
        .expect("Failed to initialize global database pool");
    println!("✓ Database pool stored in global static");

    // Initialize handler manager (spawns worker)
    let manager = JsHandlerManager::new();

    // Give worker time to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Use the macro to load all handlers and register routes automatically
    let app = load_and_register_handlers!(manager, Router::new())?;

    println!("🚀 Rust server starting on 0.0.0.0:8080");
    println!("   All routes automatically loaded from handlers/ directory");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
