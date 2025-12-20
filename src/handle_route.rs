use axum::{
    body::Body,
    extract::{Path, Query},
    http::StatusCode,
    response::Response,
};
use std::collections::HashMap;

use crate::js_handler_manager::JsHandlerManager;

// Route handler that delegates to JS worker
pub async fn handle_route(
    manager: JsHandlerManager,
    handler_name: &'static str,
    Path(params): Path<HashMap<String, String>>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Response, StatusCode> {
    match manager
        .execute_handler(handler_name, params, query, None)
        .await
    {
        Ok((status, body)) => Ok(Response::builder()
            .status(status)
            .body(Body::from(body))
            .unwrap()),
        Err(e) => {
            eprintln!("Error executing {}: {}", handler_name, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
