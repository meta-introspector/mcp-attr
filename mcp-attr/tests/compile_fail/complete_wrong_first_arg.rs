use mcp_attr::{server::{mcp_server, McpServer, RequestContext}, Result};

struct TestServer;

async fn complete_wrong_first_arg(value: i32, _cx: &RequestContext) -> Result<Vec<String>> {
    Ok(vec![value.to_string()])
}

#[mcp_server]
impl McpServer for TestServer {
    #[prompt]
    async fn test_prompt(&self, #[complete(complete_wrong_first_arg)] msg: String) -> Result<String> {
        Ok(msg)
    }
}

fn main() {}