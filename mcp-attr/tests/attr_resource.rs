use pretty_assertions::assert_eq;
use tokio::test;

use mcp_attr::Result;
use mcp_attr::client::McpClient;
use mcp_attr::schema::{
    ListResourceTemplatesRequestParams, ListResourceTemplatesResult, ListResourcesRequestParams,
    ListResourcesResult, ReadResourceRequestParams, ReadResourceResult, Resource, ResourceTemplate,
};
use mcp_attr::server::{McpServer, McpServerBuilder, resource, route};

#[resource("http://localhost/a.txt")]
async fn no_arg() -> Result<&'static str> {
    Ok("abc")
}

#[resource("http://localhost/b.txt", name = "xxx")]
async fn no_arg_with_name() -> Result<&'static str> {
    Ok("def")
}

#[resource("http://localhost/b/{a}", name = "xxx2")]
async fn arg_with_name(a: String) -> Result<String> {
    Ok(format!("hello {a}"))
}

#[resource("http://localhost/c.txt", mime_type = "text/plain")]
async fn no_arg_with_mime_type() -> Result<&'static str> {
    Ok("def")
}

#[resource("http://localhost/c/{a}", mime_type = "text/plain")]
async fn arg_with_mime_type(a: String) -> Result<String> {
    Ok(format!("hello {a}"))
}

#[resource("http://localhost/se/{a}")]
async fn simple_expansion(a: String) -> Result<String> {
    Ok(format!("hello {a}"))
}

#[resource("http://localhost/re/{+a}")]
async fn reserved_expansion(a: String) -> Result<String> {
    Ok(format!("hello {a}"))
}

#[resource("http://localhost/ge/{#a}")]
async fn fragment_expansion(a: String) -> Result<String> {
    Ok(format!("hello {a}"))
}

#[resource("http://localhost/a2/{a}/{b}")]
async fn arg_2(a: String, b: String) -> Result<String> {
    Ok(format!("hello {a} {b}"))
}

#[resource("http://localhost/ao_se/{#a}")]
async fn arg_opt_simple_expansion(a: Option<String>) -> Result<String> {
    if let Some(a) = a {
        Ok(format!("hello {a}"))
    } else {
        Ok("---".to_string())
    }
}

#[resource("http://localhost/ao_re/{+a}")]
async fn arg_opt_reserved_expansion(a: Option<String>) -> Result<String> {
    if let Some(a) = a {
        Ok(format!("hello {a}"))
    } else {
        Ok("---".to_string())
    }
}

#[resource("http://localhost/ao_fe/{#a}")]
async fn arg_opt_fragment_expansion(a: Option<String>) -> Result<String> {
    if let Some(a) = a {
        Ok(format!("hello {a}"))
    } else {
        Ok("---".to_string())
    }
}

#[resource("http://localhost/au/{arg}")]
async fn arg_name_underscore(_arg: String) -> Result<String> {
    Ok("---".to_string())
}

#[resource("http://localhost/au2/{_arg}")]
async fn arg_name_underscore_2(__arg: String) -> Result<String> {
    Ok("---".to_string())
}

/// Resource Description
#[resource("http://localhost/rd")]
async fn resource_description() -> Result<&'static str> {
    Ok("resource_description")
}

/// Resource Template Description
#[resource("http://localhost/rtd/{a}")]
async fn resource_template_description(a: String) -> Result<String> {
    Ok(format!("resource_template_description {a}"))
}

#[resource]
async fn all_url(url: String) -> Result<String> {
    Ok(format!("--{url}---"))
}

#[resource("http://localhost/attr_desc", description = "Resource with attribute description")]
async fn resource_attr_description() -> Result<String> {
    Ok("test".into())
}

/// This doc comment should be ignored
#[resource("http://localhost/priority", description = "Attribute wins")]
async fn resource_priority_test() -> Result<String> {
    Ok("test".into())
}

#[resource("http://localhost/named", name = "custom_resource_name", description = "Named resource with description")]
async fn resource_name_with_description() -> Result<String> {
    Ok("test".into())
}

fn build_server() -> Result<impl McpServer> {
    Ok(McpServerBuilder::new()
        .route(route![
            no_arg,
            no_arg_with_name,
            arg_with_name,
            no_arg_with_mime_type,
            arg_with_mime_type,
            simple_expansion,
            reserved_expansion,
            fragment_expansion,
            arg_2,
            arg_opt_simple_expansion,
            arg_opt_reserved_expansion,
            arg_opt_fragment_expansion,
            arg_name_underscore,
            arg_name_underscore_2,
            resource_description,
            resource_template_description,
            all_url,
            resource_attr_description,
            resource_priority_test,
            resource_name_with_description
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
        .resources_list(Some(ListResourcesRequestParams::default()))
        .await?;
    let e = resources_expected();
    assert_eq!(a, e);
    Ok(())
}

#[test]
async fn list_none() -> Result<()> {
    let client = build_client().await?;
    let a = client.resources_list(None).await?;
    let e = resources_expected();
    assert_eq!(a, e);
    Ok(())
}

fn resources_expected() -> ListResourcesResult {
    vec![
        Resource::new("http://localhost/a.txt", "no_arg"),
        Resource::new("http://localhost/b.txt", "xxx"),
        Resource::new("http://localhost/c.txt", "no_arg_with_mime_type")
            .with_mime_type("text/plain"),
        Resource::new("http://localhost/rd", "resource_description")
            .with_description("Resource Description"),
        Resource::new("http://localhost/attr_desc", "resource_attr_description")
            .with_description("Resource with attribute description"),
        Resource::new("http://localhost/priority", "resource_priority_test")
            .with_description("Attribute wins"),
        Resource::new("http://localhost/named", "custom_resource_name")
            .with_description("Named resource with description"),
    ]
    .into()
}

#[test]
async fn templates_list_some() -> Result<()> {
    let client = build_client().await?;
    let a = client
        .resources_templates_list(Some(ListResourceTemplatesRequestParams::default()))
        .await?;
    let e = templates_list_expected();
    assert_eq!(a, e);
    Ok(())
}

#[test]
async fn templates_list_none() -> Result<()> {
    let client = build_client().await?;
    let a = client.resources_templates_list(None).await?;
    let e = templates_list_expected();
    assert_eq!(a, e);
    Ok(())
}

fn templates_list_expected() -> ListResourceTemplatesResult {
    vec![
        ResourceTemplate::new("http://localhost/b/{a}", "xxx2"),
        ResourceTemplate::new("http://localhost/c/{a}", "arg_with_mime_type")
            .with_mime_type("text/plain"),
        ResourceTemplate::new("http://localhost/se/{a}", "simple_expansion"),
        ResourceTemplate::new("http://localhost/re/{+a}", "reserved_expansion"),
        ResourceTemplate::new("http://localhost/ge/{#a}", "fragment_expansion"),
        ResourceTemplate::new("http://localhost/a2/{a}/{b}", "arg_2"),
        ResourceTemplate::new("http://localhost/ao_se/{#a}", "arg_opt_simple_expansion"),
        ResourceTemplate::new("http://localhost/ao_re/{+a}", "arg_opt_reserved_expansion"),
        ResourceTemplate::new("http://localhost/ao_fe/{#a}", "arg_opt_fragment_expansion"),
        ResourceTemplate::new("http://localhost/au/{arg}", "arg_name_underscore"),
        ResourceTemplate::new("http://localhost/au2/{_arg}", "arg_name_underscore_2"),
        ResourceTemplate::new("http://localhost/rtd/{a}", "resource_template_description")
            .with_description("Resource Template Description"),
    ]
    .into()
}

#[test]
async fn read_no_arg() -> Result<()> {
    let server = McpServerBuilder::new().route(route![no_arg]).build();
    let client = McpClient::with_server(server).await?;
    let a = client
        .resources_read(ReadResourceRequestParams::new("http://localhost/a.txt"))
        .await?;
    let e: ReadResourceResult = "abc".into();
    assert_eq!(a, e);
    Ok(())
}

#[test]
async fn read_no_arg_with_name() -> Result<()> {
    let server = McpServerBuilder::new()
        .route(route![no_arg_with_name])
        .build();
    let client = McpClient::with_server(server).await?;
    let a = client
        .resources_read(ReadResourceRequestParams::new("http://localhost/b.txt"))
        .await?;
    let e: ReadResourceResult = "def".into();
    assert_eq!(a, e);
    Ok(())
}

#[test]
async fn read_arg_with_name() -> Result<()> {
    let server = McpServerBuilder::new().route(route![arg_with_name]).build();
    let client = McpClient::with_server(server).await?;
    let a = client
        .resources_read(ReadResourceRequestParams::new("http://localhost/b/123"))
        .await?;
    let e: ReadResourceResult = "hello 123".into();
    assert_eq!(a, e);
    Ok(())
}

#[test]
async fn read_simple_expansion() -> Result<()> {
    let server = McpServerBuilder::new()
        .route(route![simple_expansion])
        .build();
    let client = McpClient::with_server(server).await?;
    let a = client
        .resources_read(ReadResourceRequestParams::new("http://localhost/se/123"))
        .await?;
    let e: ReadResourceResult = "hello 123".into();
    assert_eq!(a, e);
    Ok(())
}

#[test]
async fn read_simple_expansion_decode() -> Result<()> {
    let server = McpServerBuilder::new()
        .route(route![simple_expansion])
        .build();
    let client = McpClient::with_server(server).await?;
    let a = client
        .resources_read(ReadResourceRequestParams::new(
            "http://localhost/se/%E3%81%82",
        ))
        .await?;
    let e: ReadResourceResult = "hello あ".into();
    assert_eq!(a, e);
    Ok(())
}

#[test]
async fn read_reserved_expansion() -> Result<()> {
    let server = McpServerBuilder::new()
        .route(route![reserved_expansion])
        .build();
    let client = McpClient::with_server(server).await?;
    let a = client
        .resources_read(ReadResourceRequestParams::new(
            "http://localhost/re/123/456",
        ))
        .await?;
    let e: ReadResourceResult = "hello 123/456".into();
    assert_eq!(a, e);
    Ok(())
}

#[test]
async fn read_reserved_expansion_not_decode() -> Result<()> {
    let server = McpServerBuilder::new()
        .route(route![reserved_expansion])
        .build();
    let client = McpClient::with_server(server).await?;
    let a = client
        .resources_read(ReadResourceRequestParams::new(
            "http://localhost/re/%E3%81%82",
        ))
        .await?;
    let e: ReadResourceResult = "hello %E3%81%82".into();
    assert_eq!(a, e);
    Ok(())
}

#[test]
async fn read_fragment_expansion() -> Result<()> {
    let server = McpServerBuilder::new()
        .route(route![fragment_expansion])
        .build();
    let client = McpClient::with_server(server).await?;
    let a = client
        .resources_read(ReadResourceRequestParams::new("http://localhost/ge/#123"))
        .await?;
    let e: ReadResourceResult = "hello 123".into();
    assert_eq!(a, e);
    Ok(())
}
