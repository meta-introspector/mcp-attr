use std::collections::BTreeMap;
use mcp_attr::{
    Result,
    client::McpClient,
    schema::{
        CompleteRequestParams, CompleteRequestParamsArgument, CompleteRequestParamsContext,
        PromptReference,
    },
    server::{McpServer, RequestContext, mcp_server},
};

struct TestServer;

#[mcp_server]
impl McpServer for TestServer {
    #[prompt]
    async fn prompt_simple(&self, #[complete(.complete_simple)] msg: String) -> Result<String> {
        Ok(format!("Simple: {}", msg))
    }

    #[prompt]
    async fn prompt_with_context(&self, #[complete(.complete_with_context)] msg: String) -> Result<String> {
        Ok(format!("With context: {}", msg))
    }

    #[prompt]
    async fn prompt_with_args_str(&self, #[complete(.complete_with_args_str)] msg: String) -> Result<String> {
        Ok(format!("Str args: {}", msg))
    }

    #[prompt]
    async fn prompt_with_args_optional_str(&self, #[complete(.complete_with_args_optional_str)] msg: String) -> Result<String> {
        Ok(format!("Optional str: {}", msg))
    }

    #[prompt]
    async fn prompt_with_args_fromstr(&self, #[complete(.complete_with_args_fromstr)] msg: String) -> Result<String> {
        Ok(format!("FromStr: {}", msg))
    }

    #[prompt]
    async fn prompt_with_args_optional_fromstr(&self, #[complete(.complete_with_args_optional_fromstr)] msg: String) -> Result<String> {
        Ok(format!("Optional FromStr: {}", msg))
    }

    #[prompt]
    async fn prompt_with_args_mixed(&self, #[complete(.complete_with_args_mixed)] msg: String) -> Result<String> {
        Ok(format!("Mixed args: {}", msg))
    }

    #[prompt]
    async fn prompt_multi_arg(&self, #[complete(.complete_multi_arg)] msg: String) -> Result<String> {
        Ok(format!("Multi arg: {}", msg))
    }

    #[prompt]
    async fn prompt_return_static_str(&self, #[complete(.complete_return_static_str)] msg: String) -> Result<String> {
        Ok(format!("Static str: {}", msg))
    }

    #[prompt]
    async fn prompt_return_string(&self, #[complete(.complete_return_string)] msg: String) -> Result<String> {
        Ok(format!("String: {}", msg))
    }

    #[prompt]
    async fn prompt_return_display(&self, #[complete(.complete_return_display)] msg: String) -> Result<String> {
        Ok(format!("Display: {}", msg))
    }

    #[prompt]
    async fn prompt_no_completion_defined(&self, msg: String) -> Result<String> {
        Ok(format!("No completion: {}", msg))
    }

    #[prompt]
    async fn prompt_with_arg_for_completion_func(&self, #[complete(.complete_with_custom_arg_source)] #[arg("source_type")] msg: String) -> Result<String> {
        Ok(format!("Prompt with arg for completion func: {}", msg))
    }

    // Complete functions
    #[complete_fn]
    async fn complete_simple(&self, _value: &str) -> Result<Vec<&'static str>> {
        Ok(vec!["hello", "world"])
    }

    #[complete_fn]
    async fn complete_with_context(&self, _value: &str, _cx: &RequestContext) -> Result<Vec<&'static str>> {
        Ok(vec!["context1", "context2"])
    }

    #[complete_fn]
    async fn complete_with_args_str(&self, _value: &str, category: &str) -> Result<Vec<String>> {
        Ok(vec![format!("{}_item_1", category), format!("{}_item_2", category)])
    }

    #[complete_fn]
    async fn complete_with_args_optional_str(&self, _value: &str, prefix: Option<&str>) -> Result<Vec<String>> {
        let prefix = prefix.unwrap_or("default");
        Ok(vec![
            format!("{}_option_1", prefix),
            format!("{}_option_2", prefix),
        ])
    }

    #[complete_fn]
    async fn complete_with_args_fromstr(&self, _value: &str, count: u32) -> Result<Vec<String>> {
        Ok((1..=count)
            .map(|i| format!("item_{}", i))
            .collect())
    }

    #[complete_fn]
    async fn complete_with_args_optional_fromstr(&self, _value: &str, count: Option<u32>) -> Result<Vec<String>> {
        let count = count.unwrap_or(3);
        Ok((1..=count)
            .map(|i| format!("item_{}", i))
            .collect())
    }

    #[complete_fn]
    async fn complete_with_args_mixed(
        &self,
        _value: &str,
        required_str: &str,
        optional_num: Option<i32>,
        optional_str: Option<&str>,
    ) -> Result<Vec<String>> {
        let num = optional_num.unwrap_or(42);
        let opt_str = optional_str.unwrap_or("default");
        Ok(vec![
            format!("{}_{}_{}_{}", required_str, num, opt_str, "result1"),
            format!("{}_{}_{}_{}", required_str, num, opt_str, "result2"),
        ])
    }

    #[complete_fn]
    async fn complete_multi_arg(
        &self,
        _value: &str,
        category: &str,
        count: Option<u32>,
        prefix: Option<&str>,
    ) -> Result<Vec<String>> {
        let base_count = count.unwrap_or(3);
        let prefix = prefix.unwrap_or("item");
        Ok((1..=base_count)
            .map(|i| format!("{}_{}_{}", category, prefix, i))
            .collect())
    }

    #[complete_fn]
    async fn complete_return_static_str(&self, _value: &str, _cx: &RequestContext) -> Result<Vec<&'static str>> {
        Ok(vec!["static1", "static2"])
    }

    #[complete_fn]
    async fn complete_return_string(&self, _value: &str, _cx: &RequestContext) -> Result<Vec<String>> {
        Ok(vec!["string1".to_string(), "string2".to_string()])
    }

    #[complete_fn]
    async fn complete_return_display(&self, _value: &str, _cx: &RequestContext) -> Result<Vec<u32>> {
        Ok(vec![100, 200, 300])
    }

    #[complete_fn]
    async fn complete_with_custom_arg_source(&self, _value: &str, source_type: &str) -> Result<Vec<String>> {
        Ok(vec![
            format!("{}_option_1", source_type),
            format!("{}_option_2", source_type),
        ])
    }
}

// Tests
#[tokio::test]
async fn test_simple() -> Result<()> {
    let client = McpClient::with_server(TestServer).await?;
    let ret = client
        .completion_complete(CompleteRequestParams::new(
            PromptReference::new("prompt_simple"),
            CompleteRequestParamsArgument::new("msg", "h"),
        ))
        .await?;
    assert_eq!(
        ret.completion.values,
        vec!["hello".to_string(), "world".to_string()]
    );
    assert_eq!(ret.completion.total, Some(2));
    Ok(())
}

#[tokio::test]
async fn test_with_context() -> Result<()> {
    let client = McpClient::with_server(TestServer).await?;
    let ret = client
        .completion_complete(CompleteRequestParams::new(
            PromptReference::new("prompt_with_context"),
            CompleteRequestParamsArgument::new("msg", "c"),
        ))
        .await?;
    assert_eq!(
        ret.completion.values,
        vec!["context1".to_string(), "context2".to_string()]
    );
    assert_eq!(ret.completion.total, Some(2));
    Ok(())
}

#[tokio::test]
async fn test_with_args_str() -> Result<()> {
    let client = McpClient::with_server(TestServer).await?;

    let mut arguments = BTreeMap::new();
    arguments.insert("category".to_string(), "test".to_string());

    let mut params = CompleteRequestParams::new(
        PromptReference::new("prompt_with_args_str"),
        CompleteRequestParamsArgument::new("msg", ""),
    );
    params.context = Some(CompleteRequestParamsContext { arguments });

    let ret = client.completion_complete(params).await?;
    assert_eq!(
        ret.completion.values,
        vec!["test_item_1".to_string(), "test_item_2".to_string()]
    );
    assert_eq!(ret.completion.total, Some(2));
    Ok(())
}

#[tokio::test]
async fn test_with_args_optional_str() -> Result<()> {
    let client = McpClient::with_server(TestServer).await?;

    // Test with argument provided
    let mut arguments = BTreeMap::new();
    arguments.insert("prefix".to_string(), "custom".to_string());

    let mut params = CompleteRequestParams::new(
        PromptReference::new("prompt_with_args_optional_str"),
        CompleteRequestParamsArgument::new("msg", ""),
    );
    params.context = Some(CompleteRequestParamsContext { arguments });

    let ret = client.completion_complete(params).await?;
    assert_eq!(
        ret.completion.values,
        vec!["custom_option_1".to_string(), "custom_option_2".to_string()]
    );
    assert_eq!(ret.completion.total, Some(2));
    Ok(())
}

#[tokio::test]
async fn test_with_args_fromstr() -> Result<()> {
    let client = McpClient::with_server(TestServer).await?;

    let mut arguments = BTreeMap::new();
    arguments.insert("count".to_string(), "2".to_string());

    let mut params = CompleteRequestParams::new(
        PromptReference::new("prompt_with_args_fromstr"),
        CompleteRequestParamsArgument::new("msg", ""),
    );
    params.context = Some(CompleteRequestParamsContext { arguments });

    let ret = client.completion_complete(params).await?;
    assert_eq!(
        ret.completion.values,
        vec!["item_1".to_string(), "item_2".to_string()]
    );
    assert_eq!(ret.completion.total, Some(2));
    Ok(())
}

#[tokio::test]
async fn test_with_args_optional_fromstr() -> Result<()> {
    let client = McpClient::with_server(TestServer).await?;

    // Test without argument (should use default)
    let arguments = BTreeMap::new();

    let mut params = CompleteRequestParams::new(
        PromptReference::new("prompt_with_args_optional_fromstr"),
        CompleteRequestParamsArgument::new("msg", ""),
    );
    params.context = Some(CompleteRequestParamsContext { arguments });

    let ret = client.completion_complete(params).await?;
    assert_eq!(
        ret.completion.values,
        vec![
            "item_1".to_string(),
            "item_2".to_string(),
            "item_3".to_string()
        ]
    );
    assert_eq!(ret.completion.total, Some(3));
    Ok(())
}

#[tokio::test]
async fn test_with_args_mixed() -> Result<()> {
    let client = McpClient::with_server(TestServer).await?;

    let mut arguments = BTreeMap::new();
    arguments.insert("required_str".to_string(), "base".to_string());
    arguments.insert("optional_num".to_string(), "99".to_string());
    arguments.insert("optional_str".to_string(), "custom".to_string());

    let mut params = CompleteRequestParams::new(
        PromptReference::new("prompt_with_args_mixed"),
        CompleteRequestParamsArgument::new("msg", ""),
    );
    params.context = Some(CompleteRequestParamsContext { arguments });

    let ret = client.completion_complete(params).await?;
    assert_eq!(
        ret.completion.values,
        vec![
            "base_99_custom_result1".to_string(),
            "base_99_custom_result2".to_string()
        ]
    );
    assert_eq!(ret.completion.total, Some(2));
    Ok(())
}

#[tokio::test]
async fn test_multi_arg() -> Result<()> {
    let client = McpClient::with_server(TestServer).await?;

    let mut arguments = BTreeMap::new();
    arguments.insert("category".to_string(), "test".to_string());
    arguments.insert("count".to_string(), "2".to_string());
    arguments.insert("prefix".to_string(), "opt".to_string());

    let mut params = CompleteRequestParams::new(
        PromptReference::new("prompt_multi_arg"),
        CompleteRequestParamsArgument::new("msg", ""),
    );
    params.context = Some(CompleteRequestParamsContext { arguments });

    let ret = client.completion_complete(params).await?;
    assert_eq!(
        ret.completion.values,
        vec!["test_opt_1".to_string(), "test_opt_2".to_string()]
    );
    assert_eq!(ret.completion.total, Some(2));
    Ok(())
}

#[tokio::test]
async fn test_return_types_static_str() -> Result<()> {
    let client = McpClient::with_server(TestServer).await?;
    let ret = client
        .completion_complete(CompleteRequestParams::new(
            PromptReference::new("prompt_return_static_str"),
            CompleteRequestParamsArgument::new("msg", "s"),
        ))
        .await?;
    assert_eq!(
        ret.completion.values,
        vec!["static1".to_string(), "static2".to_string()]
    );
    assert_eq!(ret.completion.total, Some(2));
    Ok(())
}

#[tokio::test]
async fn test_return_types_string() -> Result<()> {
    let client = McpClient::with_server(TestServer).await?;
    let ret = client
        .completion_complete(CompleteRequestParams::new(
            PromptReference::new("prompt_return_string"),
            CompleteRequestParamsArgument::new("msg", "s"),
        ))
        .await?;
    assert_eq!(
        ret.completion.values,
        vec!["string1".to_string(), "string2".to_string()]
    );
    assert_eq!(ret.completion.total, Some(2));
    Ok(())
}

#[tokio::test]
async fn test_return_types_display() -> Result<()> {
    let client = McpClient::with_server(TestServer).await?;
    let ret = client
        .completion_complete(CompleteRequestParams::new(
            PromptReference::new("prompt_return_display"),
            CompleteRequestParamsArgument::new("msg", "d"),
        ))
        .await?;
    assert_eq!(
        ret.completion.values,
        vec!["100".to_string(), "200".to_string(), "300".to_string()]
    );
    assert_eq!(ret.completion.total, Some(3));
    Ok(())
}

#[tokio::test]
async fn test_missing_required_args() -> Result<()> {
    let client = McpClient::with_server(TestServer).await?;

    // Test without required argument
    let arguments = BTreeMap::new();

    let mut params = CompleteRequestParams::new(
        PromptReference::new("prompt_with_args_str"),
        CompleteRequestParamsArgument::new("msg", ""),
    );
    params.context = Some(CompleteRequestParamsContext { arguments });

    let ret = client.completion_complete(params).await?;
    // Should return empty completion when required argument is missing
    assert_eq!(ret.completion.values, Vec::<String>::new());
    Ok(())
}

#[tokio::test]
async fn test_type_conversion_error() -> Result<()> {
    let client = McpClient::with_server(TestServer).await?;

    // Test with invalid type conversion
    let mut arguments = BTreeMap::new();
    arguments.insert("count".to_string(), "invalid_number".to_string());

    let mut params = CompleteRequestParams::new(
        PromptReference::new("prompt_with_args_fromstr"),
        CompleteRequestParamsArgument::new("msg", ""),
    );
    params.context = Some(CompleteRequestParamsContext { arguments });

    let result = client.completion_complete(params).await;
    // Should return an error for invalid type conversion
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn test_no_completion_defined() -> Result<()> {
    let client = McpClient::with_server(TestServer).await?;
    let ret = client
        .completion_complete(CompleteRequestParams::new(
            PromptReference::new("prompt_no_completion_defined"),
            CompleteRequestParamsArgument::new("msg", "value"),
        ))
        .await?;
    // Should return empty completion when no completion is defined
    assert_eq!(ret.completion.values, Vec::<String>::new());
    Ok(())
}

#[tokio::test]
async fn test_arg_name_for_completion_function_args() -> Result<()> {
    let client = McpClient::with_server(TestServer).await?;

    // Test that #[arg("source_type")] provides the argument name for completion function
    let mut arguments = BTreeMap::new();
    arguments.insert("source_type".to_string(), "database".to_string());

    let mut params = CompleteRequestParams::new(
        PromptReference::new("prompt_with_arg_for_completion_func"),
        CompleteRequestParamsArgument::new("source_type", ""), // Using the arg name
    );
    params.context = Some(CompleteRequestParamsContext { arguments });

    let completion_result = client.completion_complete(params).await?;

    assert_eq!(
        completion_result.completion.values,
        vec![
            "database_option_1".to_string(),
            "database_option_2".to_string()
        ]
    );

    Ok(())
}