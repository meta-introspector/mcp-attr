use mcp_attr::Result;
use mcp_attr::client::McpClient;
use mcp_attr::schema::{CallToolResult, GetPromptResult, ReadResourceResult};
use mcp_attr::server::{McpServer, mcp_server};
use tokio::test;

struct MyMcpServer;

#[mcp_server]
impl McpServer for MyMcpServer {
    #[prompt]
    async fn prompt(&self) -> Result<GetPromptResult> {
        todo!()
    }

    #[resource("http://a/b")]
    async fn resource(&self) -> Result<ReadResourceResult> {
        todo!()
    }
    #[resource]
    async fn resource_no_url(&self) -> Result<ReadResourceResult> {
        todo!()
    }
    #[tool]
    async fn tool(&self) -> Result<CallToolResult> {
        todo!()
    }
}

#[test]
async fn test() -> Result<()> {
    let server = MyMcpServer;
    let _client = McpClient::with_server(server).await?;
    Ok(())
}
