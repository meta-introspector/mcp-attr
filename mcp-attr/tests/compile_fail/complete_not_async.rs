use mcp_attr::{server::{mcp_server, McpServer, RequestContext, complete_fn}, Result};

struct TestServer;

#[complete_fn]
fn complete_not_async(_value: &str, _cx: &RequestContext) -> Result<Vec<String>> {
    Ok(vec!["test".to_string()])
}

#[mcp_server]
impl McpServer for TestServer {
    #[prompt]
    async fn test_prompt(&self, #[complete(complete_not_async)] msg: String) -> Result<String> {
        Ok(msg)
    }
}

fn main() {}