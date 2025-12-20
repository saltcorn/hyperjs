# Handler Macro System Guide

## Overview

This project now includes an automatic handler loading system that reads JavaScript handler files from a `handlers/` directory at compile time and automatically:

1. Loads the handler functions into the JavaScript worker
2. Registers the routes with the Axum router

This eliminates the need to manually add each handler to `src/handlers.rs` and manually register routes in `src/main.rs`.

## How It Works

### Build Script (`build.rs`)

The build script runs at compile time and:

1. Scans the `handlers/` directory for `.js` files
2. Reads each file and extracts:
    - The **route pattern** from the first line comment
    - The **handler code** from the rest of the file
    - The **handler name** from the filename (without extension)
3. Generates Rust code including:
    - Constants for each handler's code
    - A macro `load_and_register_handlers!` that loads handlers and registers routes

### File Format Convention

Each `.js` file in the `handlers/` directory must follow this format:

```javascript
// METHOD /route/path/{param}
async (req, res) => {
    // Your handler code here
};
```

**Important rules:**

-   **First line MUST be a comment** containing the HTTP method (optional) and route path
-   Supported comment styles: `//`, `///`, or `/* ... */`
-   HTTP method can be: `GET`, `POST`, `PUT`, `DELETE`, `PATCH` (defaults to `GET` if not specified)
-   The route path uses Axum's path syntax (e.g., `{name}` for path parameters)
-   The filename (without `.js` extension) becomes the handler name

### Example Handler Files

#### `handlers/helloHandler.js`

```javascript
// /hello/{name}
async (req, res) => {
    const { name } = req.params;
    await Promise.resolve();
    res.send(`Hello ${name}!`);
};
```

#### `handlers/userHandler.js` (GET method, implicit)

```javascript
// /users/{userId}
async (req, res) => {
    const { userId } = req.params;
    const { include } = req.query;

    res.json({
        userId,
        name: "John Doe",
        included: include || "none",
    });
};
```

#### `handlers/createUserHandler.js` (POST method)

```javascript
// POST /users
async (req, res) => {
    const { name, email } = req.query;

    if (!name || !email) {
        res.status(400).json({ error: "Missing required fields" });
        return;
    }

    const result = await Deno.core.ops.op_db_query(
        "INSERT INTO users (name, email) VALUES (?, ?)",
        [name, email]
    );

    res.status(201).json({ message: "User created", result });
};
```

#### `handlers/updateUserHandler.js` (PUT method with /// comment)

```javascript
/// PUT /users/{userId}
async (req, res) => {
    const { userId } = req.params;
    const { name, email } = req.query;

    const result = await Deno.core.ops.op_db_query(
        "UPDATE users SET name = ?, email = ? WHERE id = ?",
        [name, email, userId]
    );

    res.json({ message: "User updated", result });
};
```

#### `handlers/deleteUserHandler.js` (DELETE method with /\* \*/ comment)

```javascript
/* DELETE /users/{userId} */
async (req, res) => {
    const { userId } = req.params;

    const result = await Deno.core.ops.op_db_query(
        "DELETE FROM users WHERE id = ?",
        [userId]
    );

    res.json({ message: "User deleted", result });
};
```

## Using the Macro in `main.rs`

The generated macro is used in `main.rs` like this:

```rust
// Include the auto-generated handlers and macro from build.rs
include!(concat!(env!("OUT_DIR"), "/handlers_generated.rs"));

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ... database setup ...

    // Initialize handler manager
    let manager = JsHandlerManager::new();

    // Give worker time to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Use the macro to load all handlers and register routes automatically
    let app = load_and_register_handlers!(manager, Router::new())?;

    // ... start server ...
}
```

The macro expands to code that:

1. Calls `manager.load_handler()` for each handler file
2. Chains `.route()` calls on the router for each route
3. Returns the configured router

## Adding a New Handler

To add a new handler, simply create a new `.js` file in the `handlers/` directory:

1. Create the file: `handlers/productHandler.js`
2. Add the route comment and handler code:
    ```javascript
    // /products/{productId}
    async (req, res) => {
        const { productId } = req.params;
        res.json({
            productId,
            name: "Sample Product",
            price: 99.99,
        });
    };
    ```
3. Rebuild the project: `cargo build`
4. The handler is automatically loaded and the route is registered!

**No changes to `main.rs` or `handlers.rs` needed!**

## Generated Code

The build script generates code in `$OUT_DIR/handlers_generated.rs` that looks like:

```rust
// Auto-generated handler constants
#[allow(dead_code)]
const HELLOHANDLER_HANDLER: &str = r#"// /hello/{name}
async (req, res) => {
    const { name } = req.params;
    res.send(`Hello ${name}!`);
};
"#;

// ... more constants ...

/// Macro to load all handlers and register their routes
macro_rules! load_and_register_handlers {
    ($manager:expr, $router:expr) => {{
        use axum::{routing::get, Router};
        use axum::{extract::{Path, Query}};
        use std::collections::HashMap;

        // Load all handlers
        $manager.load_handler("helloHandler", HELLOHANDLER_HANDLER).await?;
        // ... more handlers ...

        println!("✓ Loaded 5 JavaScript handlers");

        // Register all routes
        let router = $router
            .route(
                "/hello/{name}",
                get({
                    let mgr = $manager.clone();
                    move |path: Path<HashMap<String, String>>,
                          query: Query<HashMap<String, String>>| async move {
                        handle_route(mgr, "helloHandler", path, query).await
                    }
                }),
            )
            // ... more routes ...
            ;

        Ok::<_, Box<dyn std::error::Error>>(router)
    }};
}
```

## Build Process

The build script is triggered:

-   When you run `cargo build`
-   When files in the `handlers/` directory change
-   When `build.rs` itself changes

The `println!("cargo:rerun-if-changed=handlers");` line tells Cargo to rebuild whenever the `handlers/` directory changes.

## Benefits

1. **No Manual Registration**: Add new handlers without touching Rust code
2. **Single Source of Truth**: Route paths are defined in the handler files
3. **Compile-Time Safety**: Invalid syntax in handler files is caught at compile time
4. **Zero Runtime Overhead**: All code generation happens at compile time
5. **Clean Separation**: JavaScript handlers are in their own files

## Troubleshooting

### "No handlers directory found"

-   Make sure the `handlers/` directory exists in the project root
-   Create it with: `mkdir handlers`

### Handler not loading

-   Check that the file has a `.js` extension
-   Verify the first line is a comment with a route path
-   Run `cargo clean && cargo build` to force a rebuild

### Route not working

-   Check that the route path uses correct Axum syntax
-   Parameter names in `{}` should match what the handler expects
-   Test with `curl http://localhost:8080/your/route`

## Architecture

```
Project Root
├── build.rs                    # Scans handlers/ at compile time
├── handlers/                   # Your JavaScript handler files
│   ├── helloHandler.js
│   ├── dataHandler.js
│   └── ...
├── src/
│   ├── main.rs                 # Uses the generated macro
│   └── ...
└── target/
    └── debug/
        └── build/
            └── hyperjs-xxx/
                └── out/
                    └── handlers_generated.rs  # Generated code
```

## Supported HTTP Methods

The macro system currently supports the following HTTP methods:

-   ✅ **GET** - Read operations
-   ✅ **POST** - Create operations
-   ✅ **PUT** - Update/replace operations
-   ✅ **DELETE** - Delete operations
-   ✅ **PATCH** - Partial update operations

Simply specify the method in the first line comment, or omit it to default to GET.

## Future Enhancements

Possible improvements to the macro system:

-   Support for middleware configuration per-handler
-   Support for handler metadata (description, tags, etc.)
-   Automatic API documentation generation from handler files
-   Hot reloading in development mode
-   Request body parsing configuration
-   Response type hints for TypeScript definitions
