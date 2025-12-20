use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("handlers_generated.rs");
    let mut f = fs::File::create(&dest_path).unwrap();

    // Read handlers directory
    let handlers_dir = "handlers";

    // Check if handlers directory exists
    if !Path::new(handlers_dir).exists() {
        // If directory doesn't exist, generate empty code
        writeln!(f, "// No handlers directory found").unwrap();
        writeln!(f, "macro_rules! load_and_register_handlers {{").unwrap();
        writeln!(f, "    ($manager:expr, $router:expr) => {{{{").unwrap();
        writeln!(f, "        Ok::<_, Box<dyn std::error::Error>>($router)").unwrap();
        writeln!(f, "    }}}};").unwrap();
        writeln!(f, "}}").unwrap();
        return;
    }

    // Read all .js files from the handlers directory
    let mut handlers = Vec::new();

    if let Ok(entries) = fs::read_dir(handlers_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("js") {
                let filename = path.file_stem().unwrap().to_str().unwrap();
                let content = fs::read_to_string(&path).unwrap();

                // Extract route and method from first line comment
                let (method, route) = extract_route_from_content(&content);

                handlers.push((filename.to_string(), content, method, route));
            }
        }
    }

    // Generate handler constants
    writeln!(f, "// Auto-generated handler constants").unwrap();
    for (name, content, _, _) in &handlers {
        let const_name = format!("{}_HANDLER", name.to_uppercase());
        writeln!(f, "#[allow(dead_code)]").unwrap();
        writeln!(f, "const {}: &str = r#\"{}\"#;", const_name, content).unwrap();
        writeln!(f).unwrap();
    }

    // Generate the macro
    writeln!(
        f,
        "/// Macro to load all handlers and register their routes"
    )
    .unwrap();
    writeln!(f, "macro_rules! load_and_register_handlers {{").unwrap();
    writeln!(f, "    ($manager:expr, $router:expr) => {{{{").unwrap();
    writeln!(
        f,
        "        use axum::{{routing::{{get, post, put, delete}}, Router}};"
    )
    .unwrap();
    writeln!(f, "        use axum::{{extract::{{Path, Query}}}};").unwrap();
    writeln!(f, "        use std::collections::HashMap;").unwrap();
    writeln!(f).unwrap();

    // Generate handler loading code
    writeln!(f, "        // Load all handlers").unwrap();
    for (name, _, _, _) in &handlers {
        let const_name = format!("{}_HANDLER", name.to_uppercase());
        writeln!(
            f,
            "        $manager.load_handler(\"{}\", {}).await?;",
            name, const_name
        )
        .unwrap();
    }
    writeln!(f).unwrap();

    writeln!(
        f,
        "        println!(\"✓ Loaded {} JavaScript handlers\");",
        handlers.len()
    )
    .unwrap();
    writeln!(f).unwrap();

    // Generate route registration code
    writeln!(f, "        // Register all routes").unwrap();
    writeln!(f, "        let router = $router").unwrap();

    for (name, _, method, route) in &handlers {
        if let Some(route_path) = route {
            let method_fn = method.as_deref().unwrap_or("get");
            writeln!(f, "            .route(").unwrap();
            writeln!(f, "                \"{}\",", route_path).unwrap();
            writeln!(f, "                {}({{", method_fn).unwrap();
            writeln!(f, "                    let mgr = $manager.clone();").unwrap();
            writeln!(
                f,
                "                    move |path: Path<HashMap<String, String>>,"
            )
            .unwrap();
            writeln!(
                f,
                "                          query: Query<HashMap<String, String>>| async move {{"
            )
            .unwrap();
            writeln!(
                f,
                "                        handle_route(mgr, \"{}\", path, query).await",
                name
            )
            .unwrap();
            writeln!(f, "                    }}").unwrap();
            writeln!(f, "                }}),").unwrap();
            writeln!(f, "            )").unwrap();
        }
    }

    writeln!(f, "            ;").unwrap();
    writeln!(f).unwrap();
    writeln!(f, "        Ok::<_, Box<dyn std::error::Error>>(router)").unwrap();
    writeln!(f, "    }}}};").unwrap();
    writeln!(f, "}}").unwrap();

    println!("cargo:rerun-if-changed=handlers");
}

fn extract_route_from_content(content: &str) -> (Option<String>, Option<String>) {
    // Look for route in first line comment
    // Supports formats:
    // - // GET /route/path
    // - /// POST /route/path
    // - /* PUT /route/path */
    // - // /route/path (defaults to GET)

    let first_line = content.lines().next();
    if first_line.is_none() {
        return (None, None);
    }

    let trimmed = first_line.unwrap().trim();
    let mut comment_content = None;

    // Handle // or /// style comments
    if let Some(content) = trimmed.strip_prefix("///") {
        comment_content = Some(content.trim());
    } else if let Some(content) = trimmed.strip_prefix("//") {
        comment_content = Some(content.trim());
    }
    // Handle /* */ style comments
    else if trimmed.starts_with("/*") && trimmed.ends_with("*/") {
        let content = trimmed
            .trim_start_matches("/*")
            .trim_end_matches("*/")
            .trim();
        comment_content = Some(content);
    }

    if let Some(content) = comment_content {
        // Parse method and route
        // Format: "METHOD /path" or just "/path"
        let parts: Vec<&str> = content.split_whitespace().collect();

        if parts.is_empty() {
            return (None, None);
        }

        // Check if first part is an HTTP method
        let method = parts[0].to_uppercase();
        if method == "GET"
            || method == "POST"
            || method == "PUT"
            || method == "DELETE"
            || method == "PATCH"
        {
            // Method specified
            if parts.len() > 1 {
                let route = parts[1..].join(" ");
                return (Some(method.to_lowercase()), Some(route));
            } else {
                return (None, None);
            }
        } else if parts[0].starts_with('/') {
            // No method specified, route starts with /
            let route = parts.join(" ");
            return (None, Some(route));
        }
    }

    (None, None)
}
