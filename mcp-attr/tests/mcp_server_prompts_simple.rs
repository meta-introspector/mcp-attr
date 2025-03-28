use mcp_attr::Result;
use mcp_attr::server::{McpServer, mcp_server};

struct MyMcpServer;

#[mcp_server]
impl McpServer for MyMcpServer {
    #[prompt]
    async fn my_prompt(&self) -> Result<&str> {
        Ok("Hello, world!")
    }
}

#[test]
fn test() {
    let server = MyMcpServer;
    fn f(_: impl McpServer) {}
    f(server);
}
