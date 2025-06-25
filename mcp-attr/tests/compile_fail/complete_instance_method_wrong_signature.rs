use mcp_attr::{server::{mcp_server, McpServer}, Result};

struct TestServer;

impl TestServer {
    async fn wrong_signature_method(&self, wrong_param: i32) -> Result<Vec<String>> {
        Ok(vec![wrong_param.to_string()])
    }
}

#[mcp_server]
impl McpServer for TestServer {
    #[prompt]
    async fn test_prompt(&self, #[complete(.wrong_signature_method)] msg: String) -> Result<String> {
        Ok(msg)
    }
}

fn main() {}