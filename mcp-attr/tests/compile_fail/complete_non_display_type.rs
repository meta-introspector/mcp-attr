use mcp_attr::{server::{mcp_server, McpServer, RequestContext}, Result};

struct TestServer;

#[derive(Debug)]
struct NonDisplayType {
    value: i32,
}

async fn complete_non_display(_value: &str, _cx: &RequestContext) -> Result<Vec<NonDisplayType>> {
    Ok(vec![NonDisplayType { value: 42 }])
}

#[mcp_server]
impl McpServer for TestServer {
    #[prompt]
    async fn test_prompt(&self, #[complete(complete_non_display)] msg: String) -> Result<String> {
        Ok(msg)
    }
}

fn main() {}