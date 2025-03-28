use mcp_attr::server::builder::route;
use mcp_attr::{
    Result,
    server::{RequestContext, builder::McpServerBuilder, prompt, resource, tool},
};

#[prompt]
async fn get_prompt_a(_arg0: String, _cx: &RequestContext) -> Result<String> {
    todo!()
}

#[test]
fn register_prompt() -> Result<()> {
    let s = McpServerBuilder::new();
    s.route(route![get_prompt_a]);
    Ok(())
}

#[resource("file://{path}")]
async fn get_file(_path: String, _cx: &RequestContext) -> Result<String> {
    todo!()
}

#[test]
fn register_resource() -> Result<()> {
    let s = McpServerBuilder::new();
    s.route(route![get_file]);
    Ok(())
}

#[tool]
async fn make_file(_arg0: String, _cx: &RequestContext) -> Result<String> {
    todo!()
}

#[test]
fn register_tool() -> Result<()> {
    let s = McpServerBuilder::new();
    s.route(route![make_file]);
    Ok(())
}
