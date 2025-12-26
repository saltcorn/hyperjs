use napi_derive::napi;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// A simple, thread-safe router for path matching
///
/// This router only handles path matching. Handler management
/// and invocation should be done in JavaScript for simplicity.
///
/// # JavaScript Usage Example:
///
/// ```javascript
/// const { Router, Request, Response, Body, StatusCode } = require('./index');
///
/// // Create router
/// const router = new Router();
///
/// // Store handlers in JavaScript (simpler and more flexible)
/// const handlers = new Map();
///
/// // Register routes and handlers
/// function register(path, handler) {
///   router.addRoute(path);
///   handlers.set(path, handler);
/// }
///
/// register('/users', async (req) => {
///   return Response.builder()
///     .status(StatusCode.ok())
///     .body(Body.string('Users'));
/// });
///
/// register('/posts', (req) => {
///   return Response.builder()
///     .status(StatusCode.ok())
///     .body(Body.string('Posts'));
/// });
///
/// // Handle requests
/// async function handleRequest(request) {
///   const path = request.uri();
///   
///   if (router.hasRoute(path)) {
///     const handler = handlers.get(path);
///     if (handler) {
///       return await handler(request);
///     }
///   }
///   
///   return Response.builder()
///     .status(StatusCode.notFound())
///     .body(Body.string(`Not Found: ${path}`));
/// }
///
/// // Use with Node.js http server or in hyper service
/// ```
#[napi]
#[derive(Clone)]
pub struct Router {
  routes: Arc<RwLock<HashMap<String, RouteInfo>>>,
}

#[napi(object)]
#[derive(Clone)]
pub struct RouteInfo {
  pub path: String,
  pub method: Option<String>,
}

#[napi]
impl Router {
  /// Create a new router
  #[napi(constructor)]
  pub fn new() -> Self {
    Self {
      routes: Arc::new(RwLock::new(HashMap::new())),
    }
  }

  /// Add a route to the router
  ///
  /// @param path - The path to register (e.g., "/users")
  /// @param method - Optional HTTP method (e.g., "GET", "POST")
  #[napi]
  pub fn add_route(&self, path: String, method: Option<String>) {
    let mut routes = self.routes.write().unwrap();
    routes.insert(path.clone(), RouteInfo { path, method });
  }

  /// Remove a route from the router
  ///
  /// @returns true if the route existed and was removed
  #[napi]
  pub fn remove_route(&self, path: String) -> bool {
    let mut routes = self.routes.write().unwrap();
    routes.remove(&path).is_some()
  }

  /// Check if a route exists
  ///
  /// @param path - The path to check
  /// @returns true if the route is registered
  #[napi]
  pub fn has_route(&self, path: String) -> bool {
    let routes = self.routes.read().unwrap();
    routes.contains_key(&path)
  }

  /// Find a route and get its info
  ///
  /// @param path - The path to look up
  /// @returns RouteInfo if found, null otherwise
  #[napi]
  pub fn find_route(&self, path: String) -> Option<RouteInfo> {
    let routes = self.routes.read().unwrap();
    routes.get(&path).cloned()
  }

  /// Get all registered routes
  ///
  /// @returns Array of all route paths
  #[napi]
  pub fn get_routes(&self) -> Vec<String> {
    let routes = self.routes.read().unwrap();
    routes.keys().cloned().collect()
  }

  /// Get all route information
  ///
  /// @returns Array of RouteInfo objects
  #[napi]
  pub fn get_route_info(&self) -> Vec<RouteInfo> {
    let routes = self.routes.read().unwrap();
    routes.values().cloned().collect()
  }

  /// Clear all routes
  #[napi]
  pub fn clear(&self) {
    let mut routes = self.routes.write().unwrap();
    routes.clear();
  }

  /// Get the number of registered routes
  #[napi]
  pub fn count(&self) -> u32 {
    let routes = self.routes.read().unwrap();
    routes.len() as u32
  }
}

impl Default for Router {
  fn default() -> Self {
    Self::new()
  }
}
