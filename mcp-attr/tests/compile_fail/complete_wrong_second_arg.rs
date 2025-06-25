use mcp_attr::{server::{mcp_server, McpServer, RequestContext}, Result};

struct TestServer;

async fn complete_wrong_second_arg(_value: &str, cx: RequestContext) -> Result<Vec<String>> {
    Ok(vec!["test".to_string()])
}

#[mcp_server]
impl McpServer for TestServer {
    #[prompt]
    async fn test_prompt(&self, #[complete(complete_wrong_second_arg)] msg: String) -> Result<String> {
        Ok(msg)
    }
}

fn main() {}