use mcp_attr_macros::complete_fn;

struct TestServer {
    name: String,
}

impl TestServer {
    #[complete_fn]
    async fn complete_with_self(&self, value: &str) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(vec![format!("completion for {}", value)])
    }
}

fn main() {}