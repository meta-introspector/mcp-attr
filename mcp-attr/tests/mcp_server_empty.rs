use mcp_attr::server::{McpServer, mcp_server};

struct MyMcpServer;

#[mcp_server]
impl McpServer for MyMcpServer {}

#[test]
fn test() {
    let server = MyMcpServer;
    fn f(_: impl McpServer) {}
    f(server);
}
