use mcp_attr::Result;
use mcp_attr::client::McpClient;
use mcp_attr::schema::{ListToolsRequestParams, ToolAnnotations};
use mcp_attr::server::{McpServer, mcp_server};
use tokio::test;

struct TestServer;

#[mcp_server]
impl McpServer for TestServer {
    #[tool]
    async fn basic_tool(&self) -> Result<&'static str> {
        Ok("basic")
    }

    #[tool]
    async fn destructive_tool(&self) -> Result<&'static str> {
        Ok("destructive")
    }

    #[tool(non_destructive)]
    async fn non_destructive_tool(&self) -> Result<&'static str> {
        Ok("non_destructive")
    }

    #[tool(non_destructive, idempotent, read_only, closed_world)]
    async fn all_explicit_tool(&self) -> Result<&'static str> {
        Ok("all_explicit")
    }
}

#[test]
async fn test_mcp_server_tool_annotations() -> Result<()> {
    let client = McpClient::with_server(TestServer).await?;

    let result = client
        .tools_list(Some(ListToolsRequestParams::default()))
        .await?;
    let basic_tool = result
        .tools
        .iter()
        .find(|t| t.name == "basic_tool")
        .unwrap();
    assert!(basic_tool.annotations.is_none());

    let destructive_tool = result
        .tools
        .iter()
        .find(|t| t.name == "destructive_tool")
        .unwrap();
    assert!(destructive_tool.annotations.is_none());

    let non_destructive_tool = result
        .tools
        .iter()
        .find(|t| t.name == "non_destructive_tool")
        .unwrap();
    let expected = ToolAnnotations {
        destructive_hint: Some(false),
        ..ToolAnnotations::default()
    };
    assert_eq!(non_destructive_tool.annotations, Some(expected));

    let all_explicit_tool = result
        .tools
        .iter()
        .find(|t| t.name == "all_explicit_tool")
        .unwrap();
    let expected = ToolAnnotations {
        destructive_hint: Some(false),
        idempotent_hint: Some(true),
        read_only_hint: Some(true),
        open_world_hint: Some(false),
        ..ToolAnnotations::default()
    };
    assert_eq!(all_explicit_tool.annotations, Some(expected));

    Ok(())
}
