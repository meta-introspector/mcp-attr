use tokio::test;

use mcp_attr::{
    Result,
    client::McpClient,
    server::{McpServer, mcp_server},
};

struct MyMcpServer;

#[mcp_server]
impl McpServer for MyMcpServer {}

#[test]
async fn ping_client_to_server() -> Result<()> {
    let server = MyMcpServer;
    let client = McpClient::with_server(server).await?;
    client.ping().await?;
    Ok(())
}
