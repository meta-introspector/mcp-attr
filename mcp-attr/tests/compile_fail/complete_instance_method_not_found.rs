use mcp_attr::{server::{mcp_server, McpServer}, Result};

struct TestServer;

#[mcp_server]
impl McpServer for TestServer {
    #[prompt]
    async fn test_prompt(&self, #[complete(.non_existent_method)] msg: String) -> Result<String> {
        Ok(msg)
    }
}

fn main() {}