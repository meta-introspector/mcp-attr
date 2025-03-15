//! Functions for creating errors for MCP server implementation

use jsoncall::{Error, ErrorCode, ErrorObject};
use serde_json::json;

/// Creates an error for when a prompt is not found
pub fn prompt_not_found(_name: &str) -> Error {
    Error::new(ErrorCode::METHOD_NOT_FOUND).with_message("Prompt not found", true)
}

/// Creates an error for when a tool is not found
pub fn tool_not_found(_name: &str) -> Error {
    Error::new(ErrorCode::METHOD_NOT_FOUND).with_message("Tool not found", true)
}

/// Creates an error for when a resource is not found
pub fn resource_not_found(uri: &str) -> Error {
    ErrorObject {
        code: ErrorCode::INVALID_PARAMS,
        message: "Resource not found".to_string(),
        data: Some(json!({ "uri": uri })),
    }
    .into()
}
