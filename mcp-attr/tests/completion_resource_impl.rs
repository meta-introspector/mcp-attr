use mcp_attr::{
    Result,
    client::McpClient,
    schema::{
        CompleteRequestParams, CompleteRequestParamsArgument, CompleteRequestParamsContext,
        ResourceTemplateReference,
    },
    server::{McpServer, RequestContext, mcp_server},
};
use std::collections::BTreeMap;

struct TestServer;

#[mcp_server]
impl McpServer for TestServer {
    #[resource("test://{name}")]
    async fn resource_simple(&self, #[complete(.complete_simple)] name: String) -> Result<String> {
        Ok(format!("Resource: {name}"))
    }

    #[resource("test_context://{name}")]
    async fn resource_with_context(
        &self,
        #[complete(.complete_with_context)] name: String,
    ) -> Result<String> {
        Ok(format!("Context resource: {name}"))
    }

    #[resource("test_str://{name}")]
    async fn resource_with_args_str(
        &self,
        #[complete(.complete_with_args_str)] name: String,
    ) -> Result<String> {
        Ok(format!("Str resource: {name}"))
    }

    #[resource("test_opt_str://{name}")]
    async fn resource_with_args_optional_str(
        &self,
        #[complete(.complete_with_args_optional_str)] name: String,
    ) -> Result<String> {
        Ok(format!("Optional str resource: {name}"))
    }

    #[resource("test_fromstr://{name}")]
    async fn resource_with_args_fromstr(
        &self,
        #[complete(.complete_with_args_fromstr)] name: String,
    ) -> Result<String> {
        Ok(format!("FromStr resource: {name}"))
    }

    #[resource("test_opt_fromstr://{name}")]
    async fn resource_with_args_optional_fromstr(
        &self,
        #[complete(.complete_with_args_optional_fromstr)] name: String,
    ) -> Result<String> {
        Ok(format!("Optional FromStr resource: {name}"))
    }

    #[resource("test_mixed://{name}")]
    async fn resource_with_args_mixed(
        &self,
        #[complete(.complete_with_args_mixed)] name: String,
    ) -> Result<String> {
        Ok(format!("Mixed resource: {name}"))
    }

    #[resource("files://{path}/{file}")]
    async fn resource_multi_param(
        &self,
        #[complete(.complete_path)] path: String,
        #[complete(.complete_file)] file: String,
    ) -> Result<String> {
        Ok(format!("Multi param: {path}/{file}"))
    }

    #[resource("static://{name}")]
    async fn resource_return_static_str(
        &self,
        #[complete(.complete_return_static_str)] name: String,
    ) -> Result<String> {
        Ok(format!("Static resource: {name}"))
    }

    #[resource("string://{name}")]
    async fn resource_return_string(
        &self,
        #[complete(.complete_return_string)] name: String,
    ) -> Result<String> {
        Ok(format!("String resource: {name}"))
    }

    #[resource("display://{name}")]
    async fn resource_return_display(
        &self,
        #[complete(.complete_return_display)] name: String,
    ) -> Result<String> {
        Ok(format!("Display resource: {name}"))
    }

    #[resource("plain://resource")]
    async fn resource_no_completion_defined(&self) -> Result<String> {
        Ok("Plain resource".to_string())
    }

    // Complete functions
    #[complete_fn]
    async fn complete_simple(&self, _value: &str) -> Result<Vec<&'static str>> {
        Ok(vec!["res1", "res2"])
    }

    #[complete_fn]
    async fn complete_with_context(
        &self,
        _value: &str,
        _cx: &RequestContext,
    ) -> Result<Vec<&'static str>> {
        Ok(vec!["context_res1", "context_res2"])
    }

    #[complete_fn]
    async fn complete_with_args_str(&self, _value: &str, category: &str) -> Result<Vec<String>> {
        Ok(vec![
            format!("{}_resource_1", category),
            format!("{}_resource_2", category),
        ])
    }

    #[complete_fn]
    async fn complete_with_args_optional_str(
        &self,
        _value: &str,
        prefix: Option<&str>,
    ) -> Result<Vec<String>> {
        let prefix = prefix.unwrap_or("default");
        Ok(vec![
            format!("{}_res_1", prefix),
            format!("{}_res_2", prefix),
        ])
    }

    #[complete_fn]
    async fn complete_with_args_fromstr(&self, _value: &str, count: u32) -> Result<Vec<String>> {
        Ok((1..=count).map(|i| format!("resource_{i}")).collect())
    }

    #[complete_fn]
    async fn complete_with_args_optional_fromstr(
        &self,
        _value: &str,
        count: Option<u32>,
    ) -> Result<Vec<String>> {
        let count = count.unwrap_or(3);
        Ok((1..=count).map(|i| format!("resource_{i}")).collect())
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
            format!("{}_{}_{}_{}", required_str, num, opt_str, "res1"),
            format!("{}_{}_{}_{}", required_str, num, opt_str, "res2"),
        ])
    }

    #[complete_fn]
    async fn complete_path(&self, _value: &str, _cx: &RequestContext) -> Result<Vec<&'static str>> {
        Ok(vec!["home", "usr", "var"])
    }

    #[complete_fn]
    async fn complete_file(&self, _value: &str, _cx: &RequestContext) -> Result<Vec<&'static str>> {
        Ok(vec!["config.txt", "data.json"])
    }

    #[complete_fn]
    async fn complete_return_static_str(
        &self,
        _value: &str,
        _cx: &RequestContext,
    ) -> Result<Vec<&'static str>> {
        Ok(vec!["static_res1", "static_res2"])
    }

    #[complete_fn]
    async fn complete_return_string(
        &self,
        _value: &str,
        _cx: &RequestContext,
    ) -> Result<Vec<String>> {
        Ok(vec!["string_res1".to_string(), "string_res2".to_string()])
    }

    #[complete_fn]
    async fn complete_return_display(
        &self,
        _value: &str,
        _cx: &RequestContext,
    ) -> Result<Vec<u32>> {
        Ok(vec![1000, 2000, 3000])
    }
}

// Tests
#[tokio::test]
async fn test_simple() -> Result<()> {
    let client = McpClient::with_server(TestServer).await?;
    let ret = client
        .completion_complete(CompleteRequestParams::new(
            ResourceTemplateReference::new("test://{name}"),
            CompleteRequestParamsArgument::new("name", ""),
        ))
        .await?;
    assert_eq!(
        ret.completion.values,
        vec!["res1".to_string(), "res2".to_string()]
    );
    assert_eq!(ret.completion.total, Some(2));
    Ok(())
}

#[tokio::test]
async fn test_with_context() -> Result<()> {
    let client = McpClient::with_server(TestServer).await?;
    let ret = client
        .completion_complete(CompleteRequestParams::new(
            ResourceTemplateReference::new("test_context://{name}"),
            CompleteRequestParamsArgument::new("name", "c"),
        ))
        .await?;
    assert_eq!(
        ret.completion.values,
        vec!["context_res1".to_string(), "context_res2".to_string()]
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
        ResourceTemplateReference::new("test_str://{name}"),
        CompleteRequestParamsArgument::new("name", ""),
    );
    params.context = Some(CompleteRequestParamsContext { arguments });

    let ret = client.completion_complete(params).await?;
    assert_eq!(
        ret.completion.values,
        vec!["test_resource_1".to_string(), "test_resource_2".to_string()]
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
        ResourceTemplateReference::new("test_opt_str://{name}"),
        CompleteRequestParamsArgument::new("name", ""),
    );
    params.context = Some(CompleteRequestParamsContext { arguments });

    let ret = client.completion_complete(params).await?;
    assert_eq!(
        ret.completion.values,
        vec!["custom_res_1".to_string(), "custom_res_2".to_string()]
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
        ResourceTemplateReference::new("test_fromstr://{name}"),
        CompleteRequestParamsArgument::new("name", ""),
    );
    params.context = Some(CompleteRequestParamsContext { arguments });

    let ret = client.completion_complete(params).await?;
    assert_eq!(
        ret.completion.values,
        vec!["resource_1".to_string(), "resource_2".to_string()]
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
        ResourceTemplateReference::new("test_opt_fromstr://{name}"),
        CompleteRequestParamsArgument::new("name", ""),
    );
    params.context = Some(CompleteRequestParamsContext { arguments });

    let ret = client.completion_complete(params).await?;
    assert_eq!(
        ret.completion.values,
        vec![
            "resource_1".to_string(),
            "resource_2".to_string(),
            "resource_3".to_string()
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
        ResourceTemplateReference::new("test_mixed://{name}"),
        CompleteRequestParamsArgument::new("name", ""),
    );
    params.context = Some(CompleteRequestParamsContext { arguments });

    let ret = client.completion_complete(params).await?;
    assert_eq!(
        ret.completion.values,
        vec![
            "base_99_custom_res1".to_string(),
            "base_99_custom_res2".to_string()
        ]
    );
    assert_eq!(ret.completion.total, Some(2));
    Ok(())
}

#[tokio::test]
async fn test_multi_arg() -> Result<()> {
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
async fn test_return_types_static_str() -> Result<()> {
    let client = McpClient::with_server(TestServer).await?;
    let ret = client
        .completion_complete(CompleteRequestParams::new(
            ResourceTemplateReference::new("static://{name}"),
            CompleteRequestParamsArgument::new("name", "s"),
        ))
        .await?;
    assert_eq!(
        ret.completion.values,
        vec!["static_res1".to_string(), "static_res2".to_string()]
    );
    assert_eq!(ret.completion.total, Some(2));
    Ok(())
}

#[tokio::test]
async fn test_return_types_string() -> Result<()> {
    let client = McpClient::with_server(TestServer).await?;
    let ret = client
        .completion_complete(CompleteRequestParams::new(
            ResourceTemplateReference::new("string://{name}"),
            CompleteRequestParamsArgument::new("name", "s"),
        ))
        .await?;
    assert_eq!(
        ret.completion.values,
        vec!["string_res1".to_string(), "string_res2".to_string()]
    );
    assert_eq!(ret.completion.total, Some(2));
    Ok(())
}

#[tokio::test]
async fn test_return_types_display() -> Result<()> {
    let client = McpClient::with_server(TestServer).await?;
    let ret = client
        .completion_complete(CompleteRequestParams::new(
            ResourceTemplateReference::new("display://{name}"),
            CompleteRequestParamsArgument::new("name", "d"),
        ))
        .await?;
    assert_eq!(
        ret.completion.values,
        vec!["1000".to_string(), "2000".to_string(), "3000".to_string()]
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
        ResourceTemplateReference::new("test_str://{name}"),
        CompleteRequestParamsArgument::new("name", ""),
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
        ResourceTemplateReference::new("test_fromstr://{name}"),
        CompleteRequestParamsArgument::new("name", ""),
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
            ResourceTemplateReference::new("plain://resource"),
            CompleteRequestParamsArgument::new("name", "value"),
        ))
        .await?;
    // Should return empty completion when no completion is defined
    assert_eq!(ret.completion.values, Vec::<String>::new());
    Ok(())
}
