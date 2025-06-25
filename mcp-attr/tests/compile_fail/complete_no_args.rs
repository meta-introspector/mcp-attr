use mcp_attr::{server::{mcp_server, McpServer}, Result};

struct TestServer;

async fn complete_no_args() -> Result<Vec<String>> {
    Ok(vec!["test".to_string()])
}

#[mcp_server]
impl McpServer for TestServer {
    #[prompt]
    async fn test_prompt(&self, #[complete(complete_no_args)] msg: String) -> Result<String> {
        Ok(msg)
    }
}

fn main() {}