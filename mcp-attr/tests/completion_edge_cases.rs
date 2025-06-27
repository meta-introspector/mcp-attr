use mcp_attr::{
    Result,
    client::McpClient,
    schema::{
        CompleteRequestParams, CompleteRequestParamsArgument, CompleteRequestParamsContext,
        CompleteResult, CompleteResultCompletion, PromptReference, ResourceTemplateReference,
    },
    server::{McpServer, RequestContext, complete_fn, mcp_server},
};
use std::collections::BTreeMap;

// Test server with manual completion_complete implementation (should override auto-generation)
struct ManualCompletionServer;

#[mcp_server]
impl McpServer for ManualCompletionServer {
    #[prompt]
    async fn test_prompt(&self, #[complete(complete_hello)] msg: String) -> Result<String> {
        Ok(format!("Test: {msg}"))
    }

    // Manual implementation should override auto-generated one
    async fn completion_complete(
        &self,
        _p: CompleteRequestParams,
        _cx: &mut RequestContext,
    ) -> Result<CompleteResult> {
        let completion = CompleteResultCompletion {
            values: vec!["manual".to_string(), "override".to_string()],
            total: Some(2),
            has_more: None,
        };
        Ok(CompleteResult {
            completion,
            meta: Default::default(),
        })
    }
}

// Test server with global static method completion
struct StaticMethodServer;

#[mcp_server]
impl McpServer for StaticMethodServer {
    #[prompt]
    async fn static_prompt(&self, #[complete(complete_static)] value: String) -> Result<String> {
        Ok(format!("Static: {value}!"))
    }
}

#[complete_fn]
async fn complete_static(_value: &str, _cx: &RequestContext) -> Result<Vec<&'static str>> {
    Ok(vec!["static1", "static2", "static3"])
}

#[complete_fn]
#[allow(dead_code)] // Used in #[complete] attribute but overridden by manual implementation
async fn complete_hello(_value: &str, _cx: &RequestContext) -> Result<Vec<&'static str>> {
    Ok(vec!["hello1", "hello2"])
}

// Test server for Iterator completion functionality
struct IteratorCompletionServer;

#[mcp_server]
impl McpServer for IteratorCompletionServer {
    #[prompt]
    async fn numbers_prompt(
        &self,
        #[complete(.complete_from_iterator)] range: String,
    ) -> Result<String> {
        Ok(format!("Range: {range}"))
    }

    #[complete_fn]
    async fn complete_from_iterator(&self, _value: &str, _cx: &RequestContext) -> Result<Vec<u32>> {
        Ok((1..=5).map(|i| i * 10).collect())
    }
}

// Test server for simple additional arguments
struct SimpleArgsServer;

#[mcp_server]
impl McpServer for SimpleArgsServer {
    #[prompt]
    async fn test_prompt(
        &self,
        #[complete(.complete_with_simple_arg)] msg: String,
    ) -> Result<String> {
        Ok(format!("Message: {msg}"))
    }

    #[complete_fn]
    async fn complete_with_simple_arg(&self, _value: &str, msg: &str) -> Result<Vec<String>> {
        Ok(vec![format!("{}_option1", msg), format!("{}_option2", msg)])
    }
}

// Tests
#[tokio::test]
async fn test_manual_override() -> Result<()> {
    let client = McpClient::with_server(ManualCompletionServer).await?;
    let ret = client
        .completion_complete(CompleteRequestParams::new(
            PromptReference::new("test_prompt"),
            CompleteRequestParamsArgument::new("msg", "any"),
        ))
        .await?;
    // Should use manual implementation, not auto-generated one
    assert_eq!(
        ret.completion.values,
        vec!["manual".to_string(), "override".to_string()]
    );
    assert_eq!(ret.completion.total, Some(2));
    Ok(())
}

#[tokio::test]
async fn test_undefined_prompt_name() -> Result<()> {
    let client = McpClient::with_server(ManualCompletionServer).await?;
    let ret = client
        .completion_complete(CompleteRequestParams::new(
            PromptReference::new("unknown_prompt"),
            CompleteRequestParamsArgument::new("arg", "value"),
        ))
        .await?;
    // Should return manual override result even for unknown prompts
    assert_eq!(
        ret.completion.values,
        vec!["manual".to_string(), "override".to_string()]
    );
    Ok(())
}

#[tokio::test]
async fn test_undefined_resource_template() -> Result<()> {
    let client = McpClient::with_server(ManualCompletionServer).await?;
    let ret = client
        .completion_complete(CompleteRequestParams::new(
            ResourceTemplateReference::new("unknown://resource"),
            CompleteRequestParamsArgument::new("arg", "value"),
        ))
        .await?;
    // Should return manual override result even for unknown resources
    assert_eq!(
        ret.completion.values,
        vec!["manual".to_string(), "override".to_string()]
    );
    Ok(())
}

#[tokio::test]
async fn test_undefined_argument_name() -> Result<()> {
    let client = McpClient::with_server(ManualCompletionServer).await?;
    let ret = client
        .completion_complete(CompleteRequestParams::new(
            PromptReference::new("test_prompt"),
            CompleteRequestParamsArgument::new("unknown_arg", "value"),
        ))
        .await?;
    // Should return manual override result even for unknown arguments
    assert_eq!(
        ret.completion.values,
        vec!["manual".to_string(), "override".to_string()]
    );
    Ok(())
}

#[tokio::test]
async fn test_static_method_completion() -> Result<()> {
    let client = McpClient::with_server(StaticMethodServer).await?;
    let ret = client
        .completion_complete(CompleteRequestParams::new(
            PromptReference::new("static_prompt"),
            CompleteRequestParamsArgument::new("value", "s"),
        ))
        .await?;
    assert_eq!(
        ret.completion.values,
        vec![
            "static1".to_string(),
            "static2".to_string(),
            "static3".to_string()
        ]
    );
    assert_eq!(ret.completion.total, Some(3));
    Ok(())
}

#[tokio::test]
async fn test_from_iterator_completion() -> Result<()> {
    let client = McpClient::with_server(IteratorCompletionServer).await?;
    let ret = client
        .completion_complete(CompleteRequestParams::new(
            PromptReference::new("numbers_prompt"),
            CompleteRequestParamsArgument::new("range", "1"),
        ))
        .await?;
    assert_eq!(
        ret.completion.values,
        vec![
            "10".to_string(),
            "20".to_string(),
            "30".to_string(),
            "40".to_string(),
            "50".to_string()
        ]
    );
    assert_eq!(ret.completion.total, Some(5));
    Ok(())
}

#[tokio::test]
async fn test_simple_args_completion() -> Result<()> {
    let client = McpClient::with_server(SimpleArgsServer).await?;

    // Create context with arguments
    let mut arguments = BTreeMap::new();
    arguments.insert("msg".to_string(), "hello".to_string());

    let mut params = CompleteRequestParams::new(
        PromptReference::new("test_prompt"),
        CompleteRequestParamsArgument::new("msg", "h"),
    );
    params.context = Some(CompleteRequestParamsContext { arguments });

    let ret = client.completion_complete(params).await?;
    assert_eq!(
        ret.completion.values,
        vec!["hello_option1".to_string(), "hello_option2".to_string()]
    );
    assert_eq!(ret.completion.total, Some(2));
    Ok(())
}
