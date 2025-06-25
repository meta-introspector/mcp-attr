use jsoncall::{ErrorCode, SessionResult};
use pretty_assertions::assert_eq;
use tokio::test;

use mcp_attr::Result;
use mcp_attr::client::McpClient;
use mcp_attr::schema::{
    GetPromptRequestParams, GetPromptResult, ListPromptsRequestParams, ListPromptsResult, Prompt,
    PromptArgument,
};
use mcp_attr::server::{McpServer, McpServerBuilder, prompt, route};

#[prompt]
async fn no_arg() -> Result<&'static str> {
    Ok("value_no_arg")
}

#[prompt("xxx")]
async fn with_name() -> Result<&'static str> {
    Ok("value_with_name")
}

#[prompt]
async fn arg_1(arg_1: String) -> Result<String> {
    Ok(format!("hello {arg_1}"))
}

#[prompt]
async fn arg_2(arg_1: String, arg_2: String) -> Result<String> {
    Ok(format!("{arg_1} {arg_2}"))
}

#[prompt]
async fn arg_opt(arg_1: Option<String>) -> Result<String> {
    if let Some(arg_1) = arg_1 {
        Ok(format!("hello {arg_1}"))
    } else {
        Ok("---".to_string())
    }
}

#[prompt]
async fn arg_name_underscore(_arg: String) -> Result<String> {
    Ok(format!("hello {_arg}"))
}

#[prompt]
async fn arg_name_underscore_2(__arg: String) -> Result<String> {
    Ok(format!("hello {__arg}"))
}

#[prompt]
async fn arg_attr_name(#[arg("xxx")] arg: String) -> Result<String> {
    Ok(format!("hello {arg}"))
}

#[prompt]
async fn arg_attr_name_underscore(#[arg("_xxx")] arg: String) -> Result<String> {
    Ok(format!("hello {arg}"))
}

/// Prompt Description
#[prompt]
async fn prompt_description() -> Result<&'static str> {
    Ok("prompt_description")
}

#[prompt]
async fn arg_description(
    /// Arg description
    arg: String,
) -> Result<String> {
    Ok(format!("hello {arg}"))
}

#[prompt(description = "Prompt with attribute description")]
async fn prompt_attr_description() -> Result<String> {
    Ok("test".into())
}

/// This doc comment should be ignored
#[prompt(description = "Attribute wins")]
async fn prompt_priority_test() -> Result<String> {
    Ok("test".into())
}

#[prompt("custom_prompt_name", description = "Named prompt with description")]
async fn prompt_name_with_description() -> Result<String> {
    Ok("test".into())
}

#[prompt(title = "Prompt Title")]
async fn prompt_with_title() -> Result<String> {
    Ok("test".into())
}

#[prompt(description = "Prompt with description", title = "Prompt Title and Description")]
async fn prompt_with_description_and_title() -> Result<String> {
    Ok("test".into())
}

fn get_prompt_title() -> &'static str {
    "Prompt Expression Title"
}

#[prompt(title = get_prompt_title())]
async fn prompt_with_expression_title() -> Result<String> {
    Ok("test".into())
}

const PROMPT_TITLE: &str = "Prompt Constant Title";

#[prompt(title = PROMPT_TITLE)]
async fn prompt_with_constant_title() -> Result<String> {
    Ok("test".into())
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
            prompt_description,
            arg_description,
            prompt_attr_description,
            prompt_priority_test,
            prompt_name_with_description,
            prompt_with_title,
            prompt_with_description_and_title,
            prompt_with_expression_title,
            prompt_with_constant_title
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
        .prompts_list(Some(ListPromptsRequestParams::default()))
        .await?;
    let e = prompts_expected();
    assert_eq!(a, e);
    Ok(())
}

#[test]
async fn list_none() -> Result<()> {
    let client = build_client().await?;
    let a = client.prompts_list(None).await?;
    let e = prompts_expected();
    assert_eq!(a, e);
    Ok(())
}

fn prompts_expected() -> ListPromptsResult {
    vec![
        Prompt::new("no_arg"),
        Prompt::new("xxx"),
        Prompt::new("arg_1").with_arguments(vec![PromptArgument::new("arg_1", true)]),
        Prompt::new("arg_2").with_arguments(vec![
            PromptArgument::new("arg_1", true),
            PromptArgument::new("arg_2", true),
        ]),
        Prompt::new("arg_opt").with_arguments(vec![PromptArgument::new("arg_1", false)]),
        Prompt::new("arg_name_underscore").with_arguments(vec![PromptArgument::new("arg", true)]),
        Prompt::new("arg_name_underscore_2")
            .with_arguments(vec![PromptArgument::new("_arg", true)]),
        Prompt::new("arg_attr_name").with_arguments(vec![PromptArgument::new("xxx", true)]),
        Prompt::new("arg_attr_name_underscore")
            .with_arguments(vec![PromptArgument::new("_xxx", true)]),
        Prompt::new("prompt_description").with_description("Prompt Description"),
        Prompt::new("arg_description").with_arguments(vec![
            PromptArgument::new("arg", true).with_description("Arg description"),
        ]),
        Prompt::new("prompt_attr_description")
            .with_description("Prompt with attribute description"),
        Prompt::new("prompt_priority_test").with_description("Attribute wins"),
        Prompt::new("custom_prompt_name").with_description("Named prompt with description"),
        Prompt::new("prompt_with_title").with_title("Prompt Title"),
        Prompt::new("prompt_with_description_and_title")
            .with_description("Prompt with description")
            .with_title("Prompt Title and Description"),
        Prompt::new("prompt_with_expression_title")
            .with_title("Prompt Expression Title"),
        Prompt::new("prompt_with_constant_title")
            .with_title("Prompt Constant Title"),
    ]
    .into()
}

#[test]
async fn get_no_arg() -> Result<()> {
    let server = McpServerBuilder::new().route(route![no_arg]).build();
    let client = McpClient::with_server(server).await?;
    let a = client
        .prompts_get(GetPromptRequestParams::new("no_arg"))
        .await?;
    let e: GetPromptResult = "value_no_arg".into();
    assert_eq!(a, e);
    Ok(())
}

#[test]
async fn get_with_name() -> Result<()> {
    let server = McpServerBuilder::new().route(route![with_name]).build();
    let client = McpClient::with_server(server).await?;
    let a = client
        .prompts_get(GetPromptRequestParams::new("xxx"))
        .await?;
    let e: GetPromptResult = "value_with_name".into();
    assert_eq!(a, e);
    Ok(())
}

#[test]
async fn get_arg_1() -> Result<()> {
    let server = McpServerBuilder::new().route(route![arg_1]).build();
    let client = McpClient::with_server(server).await?;
    let a = client
        .prompts_get(GetPromptRequestParams::new("arg_1").with_arguments(vec![("arg_1", "world")]))
        .await?;
    let e: GetPromptResult = "hello world".into();
    assert_eq!(a, e);
    Ok(())
}

#[test]
async fn get_arg_2() -> Result<()> {
    let server = McpServerBuilder::new().route(route![arg_2]).build();
    let client = McpClient::with_server(server).await?;
    let a = client
        .prompts_get(
            GetPromptRequestParams::new("arg_2")
                .with_arguments(vec![("arg_1", "aaa"), ("arg_2", "bbb")]),
        )
        .await?;
    let e: GetPromptResult = "aaa bbb".into();
    assert_eq!(a, e);
    Ok(())
}

#[test]
async fn get_arg_opt_some() -> Result<()> {
    let server = McpServerBuilder::new().route(route![arg_opt]).build();
    let client = McpClient::with_server(server).await?;
    let a = client
        .prompts_get(GetPromptRequestParams::new("arg_opt").with_arguments(vec![("arg_1", "aaa")]))
        .await?;
    let e: GetPromptResult = "hello aaa".into();
    assert_eq!(a, e);
    Ok(())
}

#[test]
async fn get_arg_opt_none() -> Result<()> {
    let server = McpServerBuilder::new().route(route![arg_opt]).build();
    let client = McpClient::with_server(server).await?;
    let a = client
        .prompts_get(GetPromptRequestParams::new("arg_opt"))
        .await?;
    let e: GetPromptResult = "---".into();
    assert_eq!(a, e);
    Ok(())
}

#[test]
async fn get_missing_arg() -> Result<()> {
    let server = McpServerBuilder::new().route(route![arg_1]).build();
    let client = McpClient::with_server(server).await?;
    let a = client
        .prompts_get(GetPromptRequestParams::new("arg_1"))
        .await;
    assert_error(a, ErrorCode::INVALID_PARAMS);
    Ok(())
}

#[test]
async fn get_name_mismatch() -> Result<()> {
    let server = McpServerBuilder::new().route(route![no_arg]).build();
    let client = McpClient::with_server(server).await?;
    let a = client
        .prompts_get(GetPromptRequestParams::new("unknown"))
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
