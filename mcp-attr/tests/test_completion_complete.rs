use mcp_attr::{
    Result,
    client::McpClient,
    schema::{
        CompleteRequestParams, CompleteRequestParamsArgument, CompleteRequestParamsContext, CompleteResult,
        CompleteResultCompletion, ResourceTemplateReference, PromptReference,
    },
    server::{McpServer, RequestContext, mcp_server, complete_fn},
};
use std::collections::BTreeMap;

struct TestServer;

#[mcp_server]
impl McpServer for TestServer {
    #[prompt]
    async fn hello_prompt(&self, #[complete(.complete_hello)] msg: String) -> Result<String> {
        Ok(format!("Hello, {msg}!"))
    }

    #[resource("test://{name}")]
    async fn test_resource(&self, #[complete(.complete_name)] name: String) -> Result<String> {
        Ok(format!("Resource: {name}"))
    }

    #[resource("files://{path}/{file}")]
    async fn multi_arg_resource(
        &self,
        #[complete(.complete_path)] path: String,
        #[complete(.complete_file)] file: String,
    ) -> Result<String> {
        Ok(format!("File: {path}/{file}"))
    }

    #[prompt]
    async fn no_complete_prompt(&self, msg: String) -> Result<String> {
        Ok(format!("No completion: {msg}"))
    }

    #[resource("plain://resource")]
    async fn no_arg_resource(&self) -> Result<String> {
        Ok("Plain resource".to_string())
    }
}

impl TestServer {
    #[complete_fn]
    async fn complete_hello(&self, _value: &str, _cx: &RequestContext) -> Result<Vec<&'static str>> {
        Ok(vec!["world"])
    }

    #[complete_fn]
    async fn complete_name(&self, _value: &str, _cx: &RequestContext) -> Result<Vec<&'static str>> {
        Ok(vec!["test1", "test2"])
    }

    #[complete_fn]
    async fn complete_path(&self, _value: &str, _cx: &RequestContext) -> Result<Vec<&'static str>> {
        Ok(vec!["home", "usr", "var"])
    }

    #[complete_fn]
    async fn complete_file(&self, _value: &str, _cx: &RequestContext) -> Result<Vec<&'static str>> {
        Ok(vec!["config.txt", "data.json"])
    }
}

// Test server with #[complete_fn] inside #[mcp_server] impl block
struct InlineCompleteServer;

#[mcp_server]
impl McpServer for InlineCompleteServer {
    #[prompt]
    async fn test_prompt(&self, #[complete(.complete_inline)] msg: String) -> Result<String> {
        Ok(format!("Inline: {msg}"))
    }

    #[complete_fn]
    async fn complete_inline(&self, _value: &str, _cx: &RequestContext) -> Result<Vec<&'static str>> {
        Ok(vec!["inline1", "inline2", "inline3"])
    }
}

// Test server with RequestContext omitted in complete_fn
struct SimpleCompleteServer;

#[mcp_server]
impl McpServer for SimpleCompleteServer {
    #[prompt]
    async fn simple_prompt(&self, #[complete(.complete_simple)] msg: String) -> Result<String> {
        Ok(format!("Simple: {msg}"))
    }

    #[complete_fn]
    async fn complete_simple(&self, _value: &str) -> Result<Vec<&'static str>> {
        Ok(vec!["simple1", "simple2"])
    }
}

// Test server with simple additional arguments
struct SimpleArgsServer;

#[mcp_server]
impl McpServer for SimpleArgsServer {
    #[prompt]
    async fn test_prompt(&self, #[complete(.complete_with_simple_arg)] msg: String) -> Result<String> {
        Ok(format!("Message: {msg}"))
    }

    #[complete_fn]
    async fn complete_with_simple_arg(&self, _value: &str, msg: &str) -> Result<Vec<String>> {
        Ok(vec![format!("{}_option1", msg), format!("{}_option2", msg)])
    }
}


#[tokio::test]
async fn test_resource_completion() -> Result<()> {
    let client = McpClient::with_server(TestServer).await?;
    let ret = client
        .completion_complete(CompleteRequestParams::new(
            ResourceTemplateReference::new("test://{name}"),
            CompleteRequestParamsArgument::new("name", ""),
        ))
        .await?;
    assert_eq!(
        ret.completion.values,
        vec!["test1".to_string(), "test2".to_string()]
    );
    assert_eq!(ret.completion.total, Some(2));
    Ok(())
}

#[tokio::test]
async fn test_prompt_completion() -> Result<()> {
    let client = McpClient::with_server(TestServer).await?;
    let ret = client
        .completion_complete(CompleteRequestParams::new(
            PromptReference::new("hello_prompt"),
            CompleteRequestParamsArgument::new("msg", "w"),
        ))
        .await?;
    assert_eq!(
        ret.completion.values,
        vec!["world".to_string()]
    );
    assert_eq!(ret.completion.total, Some(1));
    Ok(())
}

#[tokio::test]
async fn test_multi_arg_resource_completion() -> Result<()> {
    let client = McpClient::with_server(TestServer).await?;
    
    // Test path completion
    let ret = client
        .completion_complete(CompleteRequestParams::new(
            ResourceTemplateReference::new("files://{path}/{file}"),
            CompleteRequestParamsArgument::new("path", ""),
        ))
        .await?;
    assert_eq!(
        ret.completion.values,
        vec!["home".to_string(), "usr".to_string(), "var".to_string()]
    );
    assert_eq!(ret.completion.total, Some(3));
    
    // Test file completion
    let ret = client
        .completion_complete(CompleteRequestParams::new(
            ResourceTemplateReference::new("files://{path}/{file}"),
            CompleteRequestParamsArgument::new("file", ""),
        ))
        .await?;
    assert_eq!(
        ret.completion.values,
        vec!["config.txt".to_string(), "data.json".to_string()]
    );
    assert_eq!(ret.completion.total, Some(2));
    
    Ok(())
}

#[tokio::test]
async fn test_unknown_prompt_completion() -> Result<()> {
    let client = McpClient::with_server(TestServer).await?;
    let ret = client
        .completion_complete(CompleteRequestParams::new(
            PromptReference::new("unknown_prompt"),
            CompleteRequestParamsArgument::new("arg", "value"),
        ))
        .await?;
    // Should return default empty completion
    assert_eq!(ret.completion.values, Vec::<String>::new());
    Ok(())
}

#[tokio::test]
async fn test_unknown_resource_completion() -> Result<()> {
    let client = McpClient::with_server(TestServer).await?;
    let ret = client
        .completion_complete(CompleteRequestParams::new(
            ResourceTemplateReference::new("unknown://resource"),
            CompleteRequestParamsArgument::new("arg", "value"),
        ))
        .await?;
    // Should return default empty completion
    assert_eq!(ret.completion.values, Vec::<String>::new());
    Ok(())
}

#[tokio::test]
async fn test_unknown_argument_completion() -> Result<()> {
    let client = McpClient::with_server(TestServer).await?;
    let ret = client
        .completion_complete(CompleteRequestParams::new(
            PromptReference::new("hello_prompt"),
            CompleteRequestParamsArgument::new("unknown_arg", "value"),
        ))
        .await?;
    // Should return default empty completion
    assert_eq!(ret.completion.values, Vec::<String>::new());
    Ok(())
}

#[tokio::test]
async fn test_no_completion_defined() -> Result<()> {
    let client = McpClient::with_server(TestServer).await?;
    let ret = client
        .completion_complete(CompleteRequestParams::new(
            PromptReference::new("no_complete_prompt"),
            CompleteRequestParamsArgument::new("msg", "value"),
        ))
        .await?;
    // Should return default empty completion when no completion is defined
    assert_eq!(ret.completion.values, Vec::<String>::new());
    Ok(())
}

// Test server with Self:: method completion
struct SelfMethodServer;

#[mcp_server]
impl McpServer for SelfMethodServer {
    #[prompt]
    async fn greet_prompt(&self, #[complete(.complete_greeting)] name: String) -> Result<String> {
        Ok(format!("Greetings, {name}!"))
    }
}

impl SelfMethodServer {
    #[complete_fn]
    async fn complete_greeting(&self, _value: &str, _cx: &RequestContext) -> Result<Vec<u32>> {
        Ok(vec![1, 2, 3])
    }
}

#[tokio::test]
async fn test_self_method_completion() -> Result<()> {
    let client = McpClient::with_server(SelfMethodServer).await?;
    let ret = client
        .completion_complete(CompleteRequestParams::new(
            PromptReference::new("greet_prompt"),
            CompleteRequestParamsArgument::new("name", "A"),
        ))
        .await?;
    assert_eq!(
        ret.completion.values,
        vec!["1".to_string(), "2".to_string(), "3".to_string()]
    );
    assert_eq!(ret.completion.total, Some(3));
    Ok(())
}

// Test server with Self:: method completion for resources
struct SelfResourceServer;

#[mcp_server]
impl McpServer for SelfResourceServer {
    #[resource("data://{id}")]
    async fn get_data(&self, #[complete(.complete_data_id)] id: String) -> Result<String> {
        Ok(format!("Data: {id}"))
    }
}

impl SelfResourceServer {
    #[complete_fn]
    async fn complete_data_id(&self, _value: &str, _cx: &RequestContext) -> Result<Vec<u32>> {
        Ok(vec![123, 456, 789])
    }
}

#[tokio::test]
async fn test_self_method_resource_completion() -> Result<()> {
    let client = McpClient::with_server(SelfResourceServer).await?;
    let ret = client
        .completion_complete(CompleteRequestParams::new(
            ResourceTemplateReference::new("data://{id}"),
            CompleteRequestParamsArgument::new("id", "1"),
        ))
        .await?;
    assert_eq!(
        ret.completion.values,
        vec!["123".to_string(), "456".to_string(), "789".to_string()]
    );
    assert_eq!(ret.completion.total, Some(3));
    Ok(())
}

// Test server with FromIterator completion functionality  
struct IteratorCompletionServer;

#[mcp_server]
impl McpServer for IteratorCompletionServer {
    #[prompt]
    async fn numbers_prompt(&self, #[complete(.complete_from_iterator)] range: String) -> Result<String> {
        Ok(format!("Range: {range}"))
    }
}

impl IteratorCompletionServer {
    #[complete_fn]
    async fn complete_from_iterator(&self, _value: &str, _cx: &RequestContext) -> Result<Vec<u32>> {
        Ok((1..=5).map(|i| i * 10).collect())
    }
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
        vec!["10".to_string(), "20".to_string(), "30".to_string(), "40".to_string(), "50".to_string()]
    );
    assert_eq!(ret.completion.total, Some(5));
    Ok(())
}

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

#[tokio::test]
async fn test_manual_completion_override() -> Result<()> {
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

// Test server with Self:: static method completion
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
        vec!["static1".to_string(), "static2".to_string(), "static3".to_string()]
    );
    assert_eq!(ret.completion.total, Some(3));
    Ok(())
}

#[tokio::test]
async fn test_inline_complete_fn() -> Result<()> {
    let client = McpClient::with_server(InlineCompleteServer).await?;
    let ret = client
        .completion_complete(CompleteRequestParams::new(
            PromptReference::new("test_prompt"),
            CompleteRequestParamsArgument::new("msg", "i"),
        ))
        .await?;
    assert_eq!(
        ret.completion.values,
        vec!["inline1".to_string(), "inline2".to_string(), "inline3".to_string()]
    );
    assert_eq!(ret.completion.total, Some(3));
    Ok(())
}

#[tokio::test]
async fn test_simple_complete_fn() -> Result<()> {
    let client = McpClient::with_server(SimpleCompleteServer).await?;
    let ret = client
        .completion_complete(CompleteRequestParams::new(
            PromptReference::new("simple_prompt"),
            CompleteRequestParamsArgument::new("msg", "s"),
        ))
        .await?;
    assert_eq!(
        ret.completion.values,
        vec!["simple1".to_string(), "simple2".to_string()]
    );
    assert_eq!(ret.completion.total, Some(2));
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
    params.context = Some(CompleteRequestParamsContext {
        arguments,
        ..Default::default()
    });
    
    let ret = client.completion_complete(params).await?;
    assert_eq!(
        ret.completion.values,
        vec!["hello_option1".to_string(), "hello_option2".to_string()]
    );
    assert_eq!(ret.completion.total, Some(2));
    Ok(())
}

// Test server with complex additional arguments
struct ComplexArgsServer;

#[mcp_server]
impl McpServer for ComplexArgsServer {
    #[prompt]
    async fn test_prompt(&self, #[complete(.complete_with_complex_args)] msg: String) -> Result<String> {
        Ok(format!("Message: {msg}"))
    }

    #[complete_fn]
    async fn complete_with_complex_args(&self, _value: &str, category: &str, count: Option<u32>, prefix: Option<&str>) -> Result<Vec<String>> {
        let base_count = count.unwrap_or(3);
        let prefix = prefix.unwrap_or("item");
        Ok((1..=base_count).map(|i| format!("{}_{}_{}", category, prefix, i)).collect())
    }
}

#[tokio::test]
async fn test_complex_args_completion() -> Result<()> {
    let client = McpClient::with_server(ComplexArgsServer).await?;
    
    // Create context with arguments
    let mut arguments = BTreeMap::new();
    arguments.insert("category".to_string(), "test".to_string());
    arguments.insert("count".to_string(), "2".to_string());
    arguments.insert("prefix".to_string(), "opt".to_string());
    
    let mut params = CompleteRequestParams::new(
        PromptReference::new("test_prompt"),
        CompleteRequestParamsArgument::new("msg", ""),
    );
    params.context = Some(CompleteRequestParamsContext {
        arguments,
        ..Default::default()
    });
    
    let ret = client.completion_complete(params).await?;
    assert_eq!(
        ret.completion.values,
        vec!["test_opt_1".to_string(), "test_opt_2".to_string()]
    );
    assert_eq!(ret.completion.total, Some(2));
    Ok(())
}

#[tokio::test]
async fn test_missing_optional_args_completion() -> Result<()> {
    let client = McpClient::with_server(ComplexArgsServer).await?;
    
    // Create context with only required arguments
    let mut arguments = BTreeMap::new();
    arguments.insert("category".to_string(), "prod".to_string());
    // count and prefix are optional, so not provided
    
    let mut params = CompleteRequestParams::new(
        PromptReference::new("test_prompt"),
        CompleteRequestParamsArgument::new("msg", ""),
    );
    params.context = Some(CompleteRequestParamsContext {
        arguments,
        ..Default::default()
    });
    
    let ret = client.completion_complete(params).await?;
    assert_eq!(
        ret.completion.values,
        vec!["prod_item_1".to_string(), "prod_item_2".to_string(), "prod_item_3".to_string()]
    );
    assert_eq!(ret.completion.total, Some(3));
    Ok(())
}

#[tokio::test]
async fn test_missing_required_args_completion() -> Result<()> {
    let client = McpClient::with_server(ComplexArgsServer).await?;
    
    // Create context without required argument
    let arguments = BTreeMap::new();
    // category is required but not provided
    
    let mut params = CompleteRequestParams::new(
        PromptReference::new("test_prompt"),
        CompleteRequestParamsArgument::new("msg", ""),
    );
    params.context = Some(CompleteRequestParamsContext {
        arguments,
        ..Default::default()
    });
    
    let ret = client.completion_complete(params).await?;
    // Should return empty completion when required argument is missing
    assert_eq!(ret.completion.values, Vec::<String>::new());
    Ok(())
}
