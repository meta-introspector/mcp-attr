use mcp_attr::{Result, server::RequestContext};
use mcp_attr_macros::prompt;

#[prompt]
async fn test_prompt(#[complete(.method_name)] msg: String) -> Result<String> {
    Ok(msg)
}

fn main() {}