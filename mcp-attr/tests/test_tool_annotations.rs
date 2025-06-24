use mcp_attr::Result;
use mcp_attr::client::McpClient;
use mcp_attr::schema::{ListToolsRequestParams, ToolAnnotations};
use mcp_attr::server::{McpServer, McpServerBuilder, route, tool};
use tokio::test;

#[tool]
async fn basic_tool() -> Result<&'static str> {
    Ok("basic")
}

#[tool(destructive)]
async fn destructive_tool() -> Result<&'static str> {
    Ok("destructive")
}

#[tool(destructive = false)]
async fn non_destructive_tool() -> Result<&'static str> {
    Ok("non_destructive")
}

#[tool(destructive, read_only = true, idempotent = false)]
async fn multi_annotation_tool() -> Result<&'static str> {
    Ok("multi")
}

#[tool(destructive = false, idempotent, read_only, open_world = false)]
async fn trailing_comma_tool() -> Result<&'static str> {
    Ok("trailing_comma")
}

fn build_server() -> Result<impl McpServer> {
    Ok(McpServerBuilder::new()
        .route(route![
            basic_tool,
            destructive_tool,
            non_destructive_tool,
            multi_annotation_tool,
            trailing_comma_tool,
        ])
        .build())
}

async fn build_client() -> Result<McpClient> {
    Ok(McpClient::with_server(build_server()?).await?)
}

#[test]
async fn test_tool_annotations() -> Result<()> {
    let client = build_client().await?;

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
        idempotent_hint: Some(false),
        read_only_hint: Some(true),
        ..ToolAnnotations::default()
    };
    assert_eq!(multi_tool.annotations, Some(expected));

    let trailing_comma_tool = result
        .tools
        .iter()
        .find(|t| t.name == "trailing_comma_tool")
        .unwrap();
    let expected = ToolAnnotations {
        destructive_hint: Some(false),
        idempotent_hint: Some(true),
        read_only_hint: Some(true),
        open_world_hint: Some(false),
        ..ToolAnnotations::default()
    };
    assert_eq!(trailing_comma_tool.annotations, Some(expected));

    Ok(())
}
