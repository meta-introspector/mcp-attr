use mcp_attr::{server::{McpServer, mcp_server}, Result};

struct TestServer;

#[mcp_server]
impl McpServer for TestServer {
    #[prompt]
    async fn test_prompt(&self, #[complete(.complete_wrong)] msg: String) -> Result<String> {
        Ok(msg)
    }
    
    #[complete_fn]
    async fn complete_wrong(&self) -> Result<Vec<String>> {
        //~^ ERROR: completion function must have `value: &str` parameter
        Ok(vec!["test".to_string()])
    }
}

fn main() {}