use mcp_attr::{
    Result,
    client::McpClient,
    server::{McpServer, mcp_server},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() -> Result<()> {
    let client = McpClient::with_server(ExampleServer).await?;
    let tools = client.tools_list(None).await?;
    let tool = &tools.tools[0];
    dbg!(tool);
    Ok(())
}

struct ExampleServer;

#[mcp_server]
impl McpServer for ExampleServer {
    #[tool]
    async fn example_tool(&self, a: A) -> Result<String> {
        Ok(format!("{a:?}"))
    }
}

#[derive(Debug, JsonSchema, Serialize, Deserialize)]
struct A {
    b: B,
}

#[derive(Debug, JsonSchema, Serialize, Deserialize)]
struct B {
    x: String,
    y: i32,
}
