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

    #[tool(destructive)]
    async fn destructive_tool(&self) -> Result<&'static str> {
        Ok("destructive")
    }

    #[tool(destructive = false)]
    async fn non_destructive_tool(&self) -> Result<&'static str> {
        Ok("non_destructive")
    }

    #[tool(destructive, idempotent, read_only = false, open_world = true)]
    async fn multi_annotation_tool(&self) -> Result<&'static str> {
        Ok("multi")
    }

    #[tool(destructive = false, idempotent = true, read_only, open_world = false)]
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
    let expected = ToolAnnotations {
        destructive_hint: Some(true),
        ..ToolAnnotations::default()
    };
    assert_eq!(destructive_tool.annotations, Some(expected));

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

    let multi_tool = result
        .tools
        .iter()
        .find(|t| t.name == "multi_annotation_tool")
        .unwrap();
    let expected = ToolAnnotations {
        destructive_hint: Some(true),
        idempotent_hint: Some(true),
        read_only_hint: Some(false),
        open_world_hint: Some(true),
        ..ToolAnnotations::default()
    };
    assert_eq!(multi_tool.annotations, Some(expected));

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
