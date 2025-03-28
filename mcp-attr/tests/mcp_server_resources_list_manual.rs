use mcp_attr::{
    Result,
    schema::{ListResourcesRequestParams, ListResourcesResult, Resource},
    server::{McpServer, RequestContext, mcp_server},
};

struct MyMcpServer;

#[mcp_server]
impl McpServer for MyMcpServer {
    async fn resources_list(
        &self,
        _p: ListResourcesRequestParams,
        _cx: &mut RequestContext,
    ) -> Result<ListResourcesResult> {
        Ok(vec![Resource::new("http://a", "a")].into())
    }
}

#[test]
fn test() {
    let server = MyMcpServer;
    fn f(_: impl McpServer) {}
    f(server);
}
