use mcp_attr::{server::{mcp_server, McpServer, RequestContext}};

struct TestServer;

async fn complete_not_result(_value: &str, _cx: &RequestContext) -> Vec<String> {
    vec!["test".to_string()]
}

#[mcp_server]
impl McpServer for TestServer {
    #[prompt]
    async fn test_prompt(&self, #[complete(complete_not_result)] msg: String) -> mcp_attr::Result<String> {
        Ok(msg)
    }
}

fn main() {}