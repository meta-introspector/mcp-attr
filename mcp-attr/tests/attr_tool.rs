use jsoncall::{ErrorCode, SessionResult};
use mcp_attr::Result;
use mcp_attr::client::McpClient;
use mcp_attr::schema::{
    CallToolRequestParams, CallToolResult, ListToolsRequestParams, ListToolsResult, Tool,
    ToolInputSchema,
};
use mcp_attr::server::{McpServer, McpServerBuilder};
use pretty_assertions::assert_eq;
use serde_json::Value;
use tokio::test;

use mcp_attr::server::{route, tool};

#[tool]
async fn no_arg() -> Result<&'static str> {
    Ok("abc")
}

#[tool("xxx")]
async fn with_name() -> Result<&'static str> {
    Ok("def")
}

#[tool]
async fn arg_1(arg_1: String) -> Result<String> {
    Ok(arg_1)
}

#[tool]
async fn arg_2(arg_1: String, arg_2: String) -> Result<String> {
    Ok(format!("{arg_1} {arg_2}"))
}

#[tool]
async fn arg_opt(arg_1: Option<String>) -> Result<String> {
    if let Some(arg_1) = arg_1 {
        Ok(arg_1)
    } else {
        Ok("---".to_string())
    }
}

#[tool]
async fn arg_name_underscore(_arg: String) -> Result<String> {
    Ok(_arg)
}

#[tool]
async fn arg_name_underscore_2(__arg: String) -> Result<String> {
    Ok(__arg)
}

#[tool]
async fn arg_attr_name(#[arg("xxx")] arg: String) -> Result<String> {
    Ok(arg)
}

#[tool]
async fn arg_attr_name_underscore(#[arg("_xxx")] arg: String) -> Result<String> {
    Ok(arg)
}

/// Tool description
#[tool]
async fn tool_description() -> Result<()> {
    Ok(())
}

#[tool]
async fn arg_description(
    /// Arg description
    arg: String,
) -> Result<String> {
    Ok(format!("hello {arg}"))
}

#[tool(description = "Tool with attribute description")]
async fn tool_attr_description() -> Result<()> {
    Ok(())
}

/// This doc comment should be ignored
#[tool(description = "Attribute wins")]
async fn tool_priority_test() -> Result<()> {
    Ok(())
}

#[tool("custom_tool_name", description = "Named tool with description")]
async fn tool_name_with_description() -> Result<()> {
    Ok(())
}

fn build_server() -> Result<impl McpServer> {
    Ok(McpServerBuilder::new()
        .route(route![
            no_arg,
            with_name,
            arg_1,
            arg_2,
            arg_opt,
            arg_name_underscore,
            arg_name_underscore_2,
            arg_attr_name,
            arg_attr_name_underscore,
            tool_description,
            arg_description,
            tool_attr_description,
            tool_priority_test,
            tool_name_with_description
        ])
        .build())
}
async fn build_client() -> Result<McpClient> {
    Ok(McpClient::with_server(build_server()?).await?)
}

#[test]
async fn list_some() -> Result<()> {
    let client = build_client().await?;
    let a = client
        .tools_list(Some(ListToolsRequestParams::default()))
        .await?;
    let e = tools_expected()?;
    assert_eq!(a, e);
    Ok(())
}

#[test]
async fn list_none() -> Result<()> {
    let client = build_client().await?;
    let a = client.tools_list(None).await?;
    let e = tools_expected()?;
    assert_eq!(a, e);
    Ok(())
}

fn tools_expected() -> Result<ListToolsResult> {
    Ok(vec![
        Tool::new("no_arg", ToolInputSchema::new()),
        Tool::new("xxx", ToolInputSchema::new()),
        Tool::new(
            "arg_1",
            ToolInputSchema::new().with_property::<String>("arg_1", "", true)?,
        ),
        Tool::new(
            "arg_2",
            ToolInputSchema::new()
                .with_property::<String>("arg_1", "", true)?
                .with_property::<String>("arg_2", "", true)?,
        ),
        Tool::new(
            "arg_opt",
            ToolInputSchema::new().with_property::<String>("arg_1", "", false)?,
        ),
        Tool::new(
            "arg_name_underscore",
            ToolInputSchema::new().with_property::<String>("arg", "", true)?,
        ),
        Tool::new(
            "arg_name_underscore_2",
            ToolInputSchema::new().with_property::<String>("_arg", "", true)?,
        ),
        Tool::new(
            "arg_attr_name",
            ToolInputSchema::new().with_property::<String>("xxx", "", true)?,
        ),
        Tool::new(
            "arg_attr_name_underscore",
            ToolInputSchema::new().with_property::<String>("_xxx", "", true)?,
        ),
        Tool::new("tool_description", ToolInputSchema::new()).with_description("Tool description"),
        Tool::new(
            "arg_description",
            ToolInputSchema::new().with_property::<String>("arg", "Arg description", true)?,
        ),
        Tool::new("tool_attr_description", ToolInputSchema::new())
            .with_description("Tool with attribute description"),
        Tool::new("tool_priority_test", ToolInputSchema::new()).with_description("Attribute wins"),
        Tool::new("custom_tool_name", ToolInputSchema::new())
            .with_description("Named tool with description"),
    ]
    .into())
}

#[test]
async fn call() -> Result<()> {
    let server = McpServerBuilder::new().route(route![no_arg]).build();
    let client = McpClient::with_server(server).await?;
    let a = client
        .tools_call(CallToolRequestParams::new("no_arg"))
        .await?;
    let e: CallToolResult = "abc".into();
    assert_eq!(a, e);
    Ok(())
}

#[test]
async fn call_name_mismatch() -> Result<()> {
    let server = McpServerBuilder::new().route(route![no_arg]).build();
    let client = McpClient::with_server(server).await?;
    let a = client
        .tools_call(CallToolRequestParams::new("unknown"))
        .await;
    assert_error(a, ErrorCode::METHOD_NOT_FOUND);
    Ok(())
}

fn assert_error<T: std::fmt::Debug>(a: SessionResult<T>, code: ErrorCode) {
    match a {
        Ok(_) => panic!("expected error.\n{a:#?}"),
        Err(e) => {
            if let Some(e) = e.error_object() {
                assert_eq!(e.code, code, "{e:#?}");
            } else {
                panic!("no error object\n{e:#?}");
            }
        }
    }
}

#[test]
async fn call_with_name() -> Result<()> {
    let server = McpServerBuilder::new().route(route![with_name]).build();
    let client = McpClient::with_server(server).await?;
    let a = client
        .tools_call(CallToolRequestParams::new("xxx").with_argument("xxx", "abc")?)
        .await?;
    let e: CallToolResult = "def".into();
    assert_eq!(a, e);
    Ok(())
}

#[test]
async fn call_with_arg_1() -> Result<()> {
    let server = McpServerBuilder::new().route(route![arg_1]).build();
    let client = McpClient::with_server(server).await?;
    let a = client
        .tools_call(CallToolRequestParams::new("arg_1").with_argument("arg_1", "abc")?)
        .await?;
    let e: CallToolResult = "abc".into();
    assert_eq!(a, e);
    Ok(())
}

#[test]
async fn call_with_arg_2() -> Result<()> {
    let server = McpServerBuilder::new().route(route![arg_2]).build();
    let client = McpClient::with_server(server).await?;
    let a = client
        .tools_call(
            CallToolRequestParams::new("arg_2")
                .with_argument("arg_1", "abc")?
                .with_argument("arg_2", "def")?,
        )
        .await?;
    let e: CallToolResult = "abc def".into();
    assert_eq!(a, e);
    Ok(())
}

#[test]
async fn call_with_arg_opt_some() -> Result<()> {
    let server = McpServerBuilder::new().route(route![arg_opt]).build();
    let client = McpClient::with_server(server).await?;
    let a = client
        .tools_call(CallToolRequestParams::new("arg_opt").with_argument("arg_1", "abc")?)
        .await?;
    let e: CallToolResult = "abc".into();
    assert_eq!(a, e);
    Ok(())
}

#[test]
async fn call_with_arg_opt_none() -> Result<()> {
    let server = McpServerBuilder::new().route(route![arg_opt]).build();
    let client = McpClient::with_server(server).await?;
    let a = client
        .tools_call(CallToolRequestParams::new("arg_opt"))
        .await?;
    let e: CallToolResult = "---".into();
    assert_eq!(a, e);
    Ok(())
}

#[test]
async fn call_with_arg_opt_null() -> Result<()> {
    let server = McpServerBuilder::new().route(route![arg_opt]).build();
    let client = McpClient::with_server(server).await?;
    let a = client
        .tools_call(CallToolRequestParams::new("arg_opt").with_argument("arg_1", Value::Null)?)
        .await?;
    let e: CallToolResult = "---".into();
    assert_eq!(a, e);
    Ok(())
}

#[test]
async fn call_missing_arg() -> Result<()> {
    let server = McpServerBuilder::new().route(route![arg_1]).build();
    let client = McpClient::with_server(server).await?;
    let a = client.tools_call(CallToolRequestParams::new("arg_1")).await;
    assert_error(a, ErrorCode::INVALID_PARAMS);
    Ok(())
}
