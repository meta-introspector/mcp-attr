//! Module for implementing MCP server

use std::{future::Future, sync::Arc};

use jsoncall::{
    ErrorCode, Handler, Hook, NotificationContext, Params, RequestContextAs, RequestId, Response,
    Result, Session, SessionContext, SessionOptions, SessionResult, bail_public,
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Map;

use crate::{
    PROTOCOL_VERSION,
    common::McpCancellationHook,
    schema::{
        CallToolRequestParams, CallToolResult, CancelledNotificationParams, ClientCapabilities,
        CompleteRequestParams, CompleteResult, CreateMessageRequestParams, CreateMessageResult,
        GetPromptRequestParams, GetPromptResult, Implementation, InitializeRequestParams,
        InitializeResult, InitializedNotificationParams, ListPromptsRequestParams,
        ListPromptsResult, ListResourceTemplatesRequestParams, ListResourceTemplatesResult,
        ListResourcesRequestParams, ListResourcesResult, ListRootsRequestParams, ListRootsResult,
        ListToolsRequestParams, ListToolsResult, PingRequestParams, ProgressNotificationParams,
        ReadResourceRequestParams, ReadResourceResult, Root, ServerCapabilities,
        ServerCapabilitiesPrompts, ServerCapabilitiesResources, ServerCapabilitiesTools,
    },
    server::errors::{prompt_not_found, tool_not_found},
    utils::Empty,
};

pub mod errors;
mod mcp_server_attr;

pub use mcp_server_attr::mcp_server;

struct McpServerHandler {
    server: Arc<dyn DynMcpServer>,
    initialize: Option<Arc<InitializeRequestParams>>,
    is_initizlized: bool,
}
impl Handler for McpServerHandler {
    fn hook(&self) -> Arc<dyn Hook> {
        Arc::new(McpCancellationHook)
    }
    fn request(
        &mut self,
        method: &str,
        params: Params,
        cx: jsoncall::RequestContext,
    ) -> Result<Response> {
        match method {
            "initialize" => return cx.handle(self.initialize(params.to()?)),
            "ping" => return cx.handle(self.ping(params.to_opt()?)),
            _ => {}
        }
        let (Some(initialize), true) = (&self.initialize, self.is_initizlized) else {
            bail_public!(_, "Server not initialized");
        };
        let i = initialize.clone();
        match method {
            "prompts/list" => self.call_opt(params, cx, |s, p, cx| s.dyn_prompts_list(p, cx, i)),
            "prompts/get" => self.call(params, cx, |s, p, cx| s.dyn_prompts_get(p, cx, i)),
            "resources/list" => {
                self.call_opt(params, cx, |s, p, cx| s.dyn_resources_list(p, cx, i))
            }
            "resources/templates/list" => self.call_opt(params, cx, |s, p, cx| {
                s.dyn_resources_templates_list(p, cx, i)
            }),
            "resources/read" => self.call(params, cx, |s, p, cx| s.dyn_resources_read(p, cx, i)),
            "tools/list" => self.call_opt(params, cx, |s, p, cx| s.dyn_tools_list(p, cx, i)),
            "tools/call" => self.call(params, cx, |s, p, cx| s.dyn_tools_call(p, cx, i)),
            "completion/complete" => {
                self.call(params, cx, |s, p, cx| s.dyn_completion_complete(p, cx, i))
            }
            _ => cx.method_not_found(),
        }
    }
    fn notification(
        &mut self,
        method: &str,
        params: Params,
        cx: NotificationContext,
    ) -> Result<Response> {
        match method {
            "notifications/initialized" => cx.handle(self.initialized(params.to_opt()?)),
            "notifications/cancelled" => self.notifications_cancelled(params.to()?, cx),
            _ => cx.method_not_found(),
        }
    }
}
impl McpServerHandler {
    pub fn new(server: impl McpServer) -> Self {
        Self {
            server: Arc::new(server),
            initialize: None,
            is_initizlized: false,
        }
    }
}
impl McpServerHandler {
    fn initialize(&mut self, p: InitializeRequestParams) -> Result<InitializeResult> {
        if p.protocol_version != PROTOCOL_VERSION {
            bail_public!(ErrorCode::INVALID_PARAMS, "Unsupported protocol version");
        }
        self.initialize = Some(Arc::new(p));
        Ok(self.server.initialize_result())
    }
    fn initialized(&mut self, _p: Option<InitializedNotificationParams>) -> Result<()> {
        if self.initialize.is_none() {
            bail_public!(
                _,
                "`initialize` request must be called before `initialized` notification"
            );
        }
        self.is_initizlized = true;
        Ok(())
    }
    fn ping(&self, _p: Option<PingRequestParams>) -> Result<Empty> {
        Ok(Empty::default())
    }
    fn notifications_cancelled(
        &self,
        p: CancelledNotificationParams,
        cx: NotificationContext,
    ) -> Result<Response> {
        cx.session().cancel_incoming_request(&p.request_id, None);
        cx.handle(Ok(()))
    }

    // fn logging_set_level(&self, p: SetLevelRequestParams) -> Result<()> {
    //     todo!()
    // }

    // fn resources_subscribe(&self, p: SubscribeRequestParams) -> Result<()> {
    //     todo!()
    // }

    // fn resources_unsubscribe(&self, p: UnsubscribeRequestParams) -> Result<()> {
    //     todo!()
    // }

    fn call<P, R>(
        &self,
        p: Params,
        cx: jsoncall::RequestContext,
        f: impl FnOnce(Arc<dyn DynMcpServer>, P, RequestContextAs<R>) -> Result<Response>,
    ) -> Result<Response>
    where
        P: DeserializeOwned,
        R: Serialize,
    {
        f(self.server.clone(), p.to()?, cx.to())
    }
    fn call_opt<P, R>(
        &self,
        p: Params,
        cx: jsoncall::RequestContext,
        f: impl FnOnce(Arc<dyn DynMcpServer>, P, RequestContextAs<R>) -> Result<Response>,
    ) -> Result<Response>
    where
        P: DeserializeOwned + Default,
        R: Serialize,
    {
        f(
            self.server.clone(),
            p.to_opt()?.unwrap_or_default(),
            cx.to(),
        )
    }
}

trait DynMcpServer: Send + Sync + 'static {
    fn initialize_result(&self) -> InitializeResult;

    fn dyn_prompts_list(
        self: Arc<Self>,
        p: ListPromptsRequestParams,
        cx: RequestContextAs<ListPromptsResult>,
        initialize: Arc<InitializeRequestParams>,
    ) -> Result<Response>;

    fn dyn_prompts_get(
        self: Arc<Self>,
        p: GetPromptRequestParams,
        cx: RequestContextAs<GetPromptResult>,
        initialize: Arc<InitializeRequestParams>,
    ) -> Result<Response>;

    fn dyn_resources_list(
        self: Arc<Self>,
        p: ListResourcesRequestParams,
        cx: RequestContextAs<ListResourcesResult>,
        initialize: Arc<InitializeRequestParams>,
    ) -> Result<Response>;

    fn dyn_resources_read(
        self: Arc<Self>,
        p: ReadResourceRequestParams,
        cx: RequestContextAs<ReadResourceResult>,
        initialize: Arc<InitializeRequestParams>,
    ) -> Result<Response>;

    fn dyn_resources_templates_list(
        self: Arc<Self>,
        p: ListResourceTemplatesRequestParams,
        cx: RequestContextAs<ListResourceTemplatesResult>,
        initialize: Arc<InitializeRequestParams>,
    ) -> Result<Response>;

    fn dyn_tools_list(
        self: Arc<Self>,
        p: ListToolsRequestParams,
        cx: RequestContextAs<ListToolsResult>,
        initialize: Arc<InitializeRequestParams>,
    ) -> Result<Response>;

    fn dyn_tools_call(
        self: Arc<Self>,
        p: CallToolRequestParams,
        cx: RequestContextAs<CallToolResult>,
        initialize: Arc<InitializeRequestParams>,
    ) -> Result<Response>;

    fn dyn_completion_complete(
        self: Arc<Self>,
        p: CompleteRequestParams,
        cx: RequestContextAs<CompleteResult>,
        initialize: Arc<InitializeRequestParams>,
    ) -> Result<Response>;
}
impl<T: McpServer> DynMcpServer for T {
    fn initialize_result(&self) -> InitializeResult {
        InitializeResult {
            capabilities: self.capabilities(),
            instructions: self.instructions(),
            meta: Map::new(),
            protocol_version: PROTOCOL_VERSION.to_string(),
            server_info: self.server_info(),
        }
    }
    fn dyn_prompts_list(
        self: Arc<Self>,
        p: ListPromptsRequestParams,
        cx: RequestContextAs<ListPromptsResult>,
        initialize: Arc<InitializeRequestParams>,
    ) -> Result<Response> {
        let mut mcp_cx = RequestContext::new(&cx, initialize);
        cx.handle_async(async move { self.prompts_list(p, &mut mcp_cx).await })
    }

    fn dyn_prompts_get(
        self: Arc<Self>,
        p: GetPromptRequestParams,
        cx: RequestContextAs<GetPromptResult>,
        initialize: Arc<InitializeRequestParams>,
    ) -> Result<Response> {
        let mut mcp_cx = RequestContext::new(&cx, initialize);
        cx.handle_async(async move { self.prompts_get(p, &mut mcp_cx).await })
    }

    fn dyn_resources_list(
        self: Arc<Self>,
        p: ListResourcesRequestParams,
        cx: RequestContextAs<ListResourcesResult>,
        initialize: Arc<InitializeRequestParams>,
    ) -> Result<Response> {
        let mut mcp_cx = RequestContext::new(&cx, initialize);
        cx.handle_async(async move { self.resources_list(p, &mut mcp_cx).await })
    }

    fn dyn_resources_templates_list(
        self: Arc<Self>,
        p: ListResourceTemplatesRequestParams,
        cx: RequestContextAs<ListResourceTemplatesResult>,
        initialize: Arc<InitializeRequestParams>,
    ) -> Result<Response> {
        let mut mcp_cx = RequestContext::new(&cx, initialize);
        cx.handle_async(async move { self.resources_templates_list(p, &mut mcp_cx).await })
    }

    fn dyn_resources_read(
        self: Arc<Self>,
        p: ReadResourceRequestParams,
        cx: RequestContextAs<ReadResourceResult>,
        initialize: Arc<InitializeRequestParams>,
    ) -> Result<Response> {
        let mut mcp_cx = RequestContext::new(&cx, initialize);
        cx.handle_async(async move { self.resources_read(p, &mut mcp_cx).await })
    }

    fn dyn_tools_list(
        self: Arc<Self>,
        p: ListToolsRequestParams,
        cx: RequestContextAs<ListToolsResult>,
        initialize: Arc<InitializeRequestParams>,
    ) -> Result<Response> {
        let mut mcp_cx = RequestContext::new(&cx, initialize);
        cx.handle_async(async move { self.tools_list(p, &mut mcp_cx).await })
    }

    fn dyn_tools_call(
        self: Arc<Self>,
        p: CallToolRequestParams,
        cx: RequestContextAs<CallToolResult>,
        initialize: Arc<InitializeRequestParams>,
    ) -> Result<Response> {
        let mut mcp_cx = RequestContext::new(&cx, initialize);
        cx.handle_async(async move { self.tools_call(p, &mut mcp_cx).await })
    }

    fn dyn_completion_complete(
        self: Arc<Self>,
        p: CompleteRequestParams,
        cx: RequestContextAs<CompleteResult>,
        initialize: Arc<InitializeRequestParams>,
    ) -> Result<Response> {
        let mut mcp_cx = RequestContext::new(&cx, initialize);
        cx.handle_async(async move { self.completion_complete(p, &mut mcp_cx).await })
    }
}

/// Trait for implementing MCP server
pub trait McpServer: Send + Sync + 'static {
    /// Returns `server_info` used in the [`initialize`] request response
    ///
    /// [`initialize`]: https://spec.modelcontextprotocol.io/specification/2024-11-05/basic/lifecycle/#initialization
    fn server_info(&self) -> Implementation {
        Implementation::from_compile_time_env()
    }

    /// Returns `instructions` used in the [`initialize`] request response
    ///
    /// [`initialize`]: https://spec.modelcontextprotocol.io/specification/2024-11-05/basic/lifecycle/#initialization
    fn instructions(&self) -> Option<String> {
        None
    }

    /// Returns `capabilities` used in the [`initialize`] request response
    ///
    /// [`initialize`]: https://spec.modelcontextprotocol.io/specification/2024-11-05/basic/lifecycle/#initialization
    fn capabilities(&self) -> ServerCapabilities {
        ServerCapabilities {
            prompts: Some(ServerCapabilitiesPrompts {
                ..Default::default()
            }),
            resources: Some(ServerCapabilitiesResources {
                ..Default::default()
            }),
            tools: Some(ServerCapabilitiesTools {
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    /// Handles [`prompts/list`]
    ///
    /// [`prompts/list`]: https://spec.modelcontextprotocol.io/specification/2024-11-05/server/prompts/#listing-prompts
    #[allow(unused_variables)]
    fn prompts_list(
        &self,
        p: ListPromptsRequestParams,
        cx: &mut RequestContext,
    ) -> impl Future<Output = Result<ListPromptsResult>> + Send {
        async { Ok(ListPromptsResult::default()) }
    }

    /// Handles [`prompts/get`]
    ///
    /// [`prompts/get`]: https://spec.modelcontextprotocol.io/specification/2024-11-05/server/prompts/#getting-a-prompt
    #[allow(unused_variables)]
    fn prompts_get(
        &self,
        p: GetPromptRequestParams,
        cx: &mut RequestContext,
    ) -> impl Future<Output = Result<GetPromptResult>> + Send {
        async move { Err(prompt_not_found(&p.name)) }
    }

    /// Handles [`resources/list`]
    ///
    /// [`resources/list`]: https://spec.modelcontextprotocol.io/specification/2024-11-05/server/resources/#listing-resources
    #[allow(unused_variables)]
    fn resources_list(
        &self,
        p: ListResourcesRequestParams,
        cx: &mut RequestContext,
    ) -> impl Future<Output = Result<ListResourcesResult>> + Send {
        async { Ok(ListResourcesResult::default()) }
    }

    /// Handles [`resources/templates/list`]
    ///
    /// [`resources/templates/list`]: https://spec.modelcontextprotocol.io/specification/2024-11-05/server/resources/#resource-templates
    #[allow(unused_variables)]
    fn resources_templates_list(
        &self,
        p: ListResourceTemplatesRequestParams,
        cx: &mut RequestContext,
    ) -> impl Future<Output = Result<ListResourceTemplatesResult>> + Send {
        async { Ok(ListResourceTemplatesResult::default()) }
    }

    /// Handles [`resources/read`]
    ///
    /// [`resources/read`]: https://spec.modelcontextprotocol.io/specification/2024-11-05/server/resources/#reading-resources
    #[allow(unused_variables)]
    fn resources_read(
        &self,
        p: ReadResourceRequestParams,
        cx: &mut RequestContext,
    ) -> impl Future<Output = Result<ReadResourceResult>> + Send {
        async move { bail_public!(ErrorCode::INVALID_PARAMS, "Resource `{}` not found", p.uri) }
    }

    /// Handles [`tools/list`]
    ///
    /// [`tools/list`]: https://spec.modelcontextprotocol.io/specification/2024-11-05/server/tools/#listing-tools
    #[allow(unused_variables)]
    fn tools_list(
        &self,
        p: ListToolsRequestParams,
        cx: &mut RequestContext,
    ) -> impl Future<Output = Result<ListToolsResult>> + Send {
        async { Ok(ListToolsResult::default()) }
    }

    /// Handles [`tools/call`]
    ///
    /// [`tools/call`]: https://spec.modelcontextprotocol.io/specification/2024-11-05/server/tools/#calling-a-tool
    #[allow(unused_variables)]
    fn tools_call(
        &self,
        p: CallToolRequestParams,
        cx: &mut RequestContext,
    ) -> impl Future<Output = Result<CallToolResult>> + Send {
        async move { Err(tool_not_found(&p.name)) }
    }

    /// Handles [`completion/complete`]
    ///
    /// [`completion/complete`]: https://spec.modelcontextprotocol.io/specification/2024-11-05/server/utilities/completion/#completing-a-prompt
    #[allow(unused_variables)]
    fn completion_complete(
        &self,
        p: CompleteRequestParams,
        cx: &mut RequestContext,
    ) -> impl Future<Output = Result<CompleteResult>> + Send {
        async { Ok(CompleteResult::default()) }
    }

    /// Gets the JSON RPC `Handler`
    fn into_handler(self) -> impl Handler + Send + Sync + 'static
    where
        Self: Sized + Send + Sync + 'static,
    {
        McpServerHandler::new(self)
    }
}

/// Context for retrieving request-related information and calling client features
pub struct RequestContext {
    session: SessionContext,
    id: RequestId,
    initialize: Arc<InitializeRequestParams>,
}

impl RequestContext {
    fn new(
        cx: &RequestContextAs<impl Serialize>,
        initialize: Arc<InitializeRequestParams>,
    ) -> Self {
        Self {
            session: cx.session(),
            id: cx.id().clone(),
            initialize,
        }
    }

    /// Gets client information
    pub fn client_info(&self) -> &Implementation {
        &self.initialize.client_info
    }

    /// Gets client capabilities
    pub fn client_capabilities(&self) -> &ClientCapabilities {
        &self.initialize.capabilities
    }

    /// Notifies progress of the request associated with this context
    ///
    /// See [`notifications/progress`]
    ///
    /// [`notifications/progress`]: https://spec.modelcontextprotocol.io/specification/2024-11-05/server/notifications/#progress-notification
    pub fn progress(&self, progress: f64, total: Option<f64>) {
        self.session
            .notification(
                "notifications/progress",
                Some(&ProgressNotificationParams {
                    progress,
                    total,
                    progress_token: self.id.clone(),
                }),
            )
            .unwrap();
    }

    /// Calls [`sampling/createMessage`]
    pub async fn sampling_create_message(
        &self,
        p: CreateMessageRequestParams,
    ) -> SessionResult<CreateMessageResult> {
        self.session
            .request("sampling/createMessage", Some(&p))
            .await
    }

    /// Calls [`roots/list`]
    pub async fn roots_list(&self) -> SessionResult<Vec<Root>> {
        let res: ListRootsResult = self
            .session
            .request("roots/list", Some(&ListRootsRequestParams::default()))
            .await?;
        Ok(res.roots)
    }
}

/// Runs an MCP server using stdio transport
pub async fn serve_stdio(server: impl McpServer) -> SessionResult<()> {
    Session::from_stdio(McpServerHandler::new(server), &SessionOptions::default())
        .wait()
        .await
}

/// Runs an MCP server using stdio transport with specified options
pub async fn serve_stdio_with(
    server: impl McpServer,
    options: &SessionOptions,
) -> SessionResult<()> {
    Session::from_stdio(McpServerHandler::new(server), options)
        .wait()
        .await
}
