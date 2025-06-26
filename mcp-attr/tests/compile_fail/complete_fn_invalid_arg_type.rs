use mcp_attr::{server::{McpServer, mcp_server}, Result};

struct TestServer;

#[mcp_server]
impl McpServer for TestServer {
    #[prompt]
    async fn test_prompt(&self, #[complete(.complete_invalid)] msg: String) -> Result<String> {
        Ok(msg)
    }
    
    #[complete_fn]
    async fn complete_invalid(&self, _value: &str, _invalid_arg: Vec<String>) -> Result<Vec<String>> {
        //~^ ERROR: unsupported argument type for completion function
        Ok(vec!["test".to_string()])
    }
}

fn main() {}