use std::{future::Future, pin::Pin};

use derive_ex::Ex;
pub use mcp_attr_macros::{prompt, resource, route, tool};
use uri_template_ex::{Captures, UriTemplate};

use crate::{
    Result,
    schema::{
        CallToolRequestParams, CallToolResult, CompleteRequestParams, CompleteResult,
        GetPromptRequestParams, GetPromptResult, Implementation, ListPromptsRequestParams,
        ListPromptsResult, ListResourceTemplatesRequestParams, ListResourcesRequestParams,
        ListResourcesResult, ListToolsRequestParams, ListToolsResult, Prompt,
        ReadResourceRequestParams, ReadResourceResult, Resource, ResourceTemplate, Tool,
    },
    server::errors::{prompt_not_found, resource_not_found, tool_not_found},
};

use super::{McpServer, RequestContext};

/// Completion function information for prompts and resources
#[derive(Debug, Clone)]
pub struct CompletionInfo {
    /// Name of the prompt or resource URI
    pub name: String,
    /// Argument name for completion
    pub argument: String,
    /// Completion function
    pub complete_fn: CompleteFn,
}

/// Completion function type
#[derive(Clone)]
pub struct CompleteFn {
    /// Function that handles completion
    #[allow(clippy::type_complexity)]
    pub f: std::sync::Arc<
        dyn for<'a> Fn(&'a CompleteRequestParams, &'a RequestContext) -> CompleteFuture<'a> + Send + Sync,
    >,
}

impl std::fmt::Debug for CompleteFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompleteFn").finish_non_exhaustive()
    }
}

type CompleteFuture<'a> = Pin<Box<dyn Future<Output = Result<crate::schema::CompleteResult>> + Send + Sync + 'a>>;

struct CustomServer {
    route: Route,
    instructions: Option<String>,
    server_info: Implementation,
}
impl McpServer for CustomServer {
    fn capabilities(&self) -> crate::schema::ServerCapabilities {
        let mut c = crate::schema::ServerCapabilities::default();
        if !self.route.tools.is_empty() {
            c.tools = Some(crate::schema::ServerCapabilitiesTools {
                ..Default::default()
            });
        }
        if !self.route.prompts.is_empty() {
            c.prompts = Some(crate::schema::ServerCapabilitiesPrompts {
                ..Default::default()
            });
        }
        if !self.route.resources.is_empty() {
            c.resources = Some(crate::schema::ServerCapabilitiesResources {
                ..Default::default()
            });
        }
        c
    }
    fn server_info(&self) -> Implementation {
        self.server_info.clone()
    }
    fn instructions(&self) -> Option<String> {
        self.instructions.clone()
    }
    async fn prompts_list(
        &self,
        _p: ListPromptsRequestParams,
        _cx: &mut RequestContext,
    ) -> Result<ListPromptsResult> {
        let prompts: Vec<Prompt> = self
            .route
            .prompts
            .iter()
            .map(|p| p.prompt.clone())
            .collect();
        Ok(prompts.into())
    }
    async fn prompts_get(
        &self,
        p: GetPromptRequestParams,
        cx: &mut RequestContext,
    ) -> Result<GetPromptResult> {
        for prompt in &self.route.prompts {
            if prompt.prompt.name == p.name {
                return (prompt.f)(&p, cx).await;
            }
        }
        Err(prompt_not_found(&p.name))
    }
    async fn resources_list(
        &self,
        _p: ListResourcesRequestParams,
        _cx: &mut RequestContext,
    ) -> Result<ListResourcesResult> {
        let resources: Vec<Resource> = self
            .route
            .resources
            .iter()
            .filter_map(|r| r.to_resource())
            .collect();
        Ok(resources.into())
    }
    async fn resources_templates_list(
        &self,
        _p: ListResourceTemplatesRequestParams,
        _cx: &mut RequestContext,
    ) -> Result<crate::schema::ListResourceTemplatesResult> {
        let templates: Vec<ResourceTemplate> = self
            .route
            .resources
            .iter()
            .filter_map(|r| r.to_resource_template())
            .collect();
        Ok(templates.into())
    }

    async fn resources_read(
        &self,
        p: ReadResourceRequestParams,
        cx: &mut RequestContext,
    ) -> Result<ReadResourceResult> {
        for resource in &self.route.resources {
            if let Some(c) = resource.captures(&p.uri) {
                return (resource.f)(&p, &c, cx).await;
            }
        }
        Err(resource_not_found(&p.uri))
    }
    async fn tools_list(
        &self,
        _p: ListToolsRequestParams,
        _cx: &mut RequestContext,
    ) -> Result<ListToolsResult> {
        let tools: Vec<Tool> = self.route.tools.iter().map(|t| t.tool.clone()).collect();
        Ok(tools.into())
    }
    async fn tools_call(
        &self,
        p: CallToolRequestParams,
        cx: &mut RequestContext,
    ) -> Result<CallToolResult> {
        for tool in &self.route.tools {
            if tool.tool.name == p.name {
                return (tool.f)(&p, cx).await;
            }
        }
        Err(tool_not_found(&p.name))
    }

    async fn completion_complete(
        &self,
        p: CompleteRequestParams,
        cx: &mut RequestContext,
    ) -> Result<CompleteResult> {
        use crate::schema::{CompleteRequestParamsRef, CompleteResultCompletion};

        match &p.ref_ {
            CompleteRequestParamsRef::PromptReference(prompt_ref) => {
                // Find matching completion for prompt
                for completion in &self.route.completions {
                    if completion.name == prompt_ref.name && completion.argument == p.argument.name {
                        return (completion.complete_fn.f)(&p, cx).await;
                    }
                }
                // Return empty completion if no match found
                Ok(CompleteResult {
                    completion: CompleteResultCompletion::default(),
                    meta: Default::default(),
                })
            }
            CompleteRequestParamsRef::ResourceTemplateReference(resource_ref) => {
                // Find matching completion for resource
                for completion in &self.route.completions {
                    if completion.name == resource_ref.uri && completion.argument == p.argument.name {
                        return (completion.complete_fn.f)(&p, cx).await;
                    }
                }
                // Return empty completion if no match found
                Ok(CompleteResult {
                    completion: CompleteResultCompletion::default(),
                    meta: Default::default(),
                })
            }
        }
    }
}

#[derive(Ex)]
#[derive_ex(Default)]
#[default(Self::new())]
pub struct McpServerBuilder {
    route: Route,
    instructions: Option<String>,
    server_info: Implementation,
}
impl McpServerBuilder {
    pub fn new() -> Self {
        Self {
            route: Route::default(),
            instructions: None,
            server_info: Implementation::from_compile_time_env(),
        }
    }
    pub fn route(mut self, route: impl Into<Route>) -> Self {
        self.route.extend(route);
        self
    }
    pub fn instructions(mut self, instructions: &str) -> Self {
        self.instructions = Some(instructions.to_string());
        self
    }
    pub fn server_info(mut self, server_info: Implementation) -> Self {
        self.server_info = server_info;
        self
    }
    pub fn build(self) -> impl McpServer {
        CustomServer {
            route: self.route,
            instructions: self.instructions,
            server_info: self.server_info,
        }
    }
}

#[derive(Default)]
pub struct Route {
    tools: Vec<ToolDefinition>,
    prompts: Vec<PromptDefinition>,
    resources: Vec<ResourceDefinition>,
    completions: Vec<CompletionInfo>,
}
impl Route {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn extend(&mut self, route: impl Into<Route>) {
        let route = route.into();
        self.tools.extend(route.tools);
        self.prompts.extend(route.prompts);
        self.resources.extend(route.resources);
        self.completions.extend(route.completions);
    }
}
impl<T> FromIterator<T> for Route
where
    T: Into<Route>,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut route = Route::new();
        for r in iter {
            let r = r.into();
            route.tools.extend(r.tools);
            route.prompts.extend(r.prompts);
            route.resources.extend(r.resources);
            route.completions.extend(r.completions);
        }
        route
    }
}

impl<T> From<Vec<T>> for Route
where
    T: Into<Route>,
{
    fn from(value: Vec<T>) -> Self {
        value.into_iter().collect()
    }
}
impl<T, const N: usize> From<[T; N]> for Route
where
    T: Into<Route>,
{
    fn from(value: [T; N]) -> Self {
        value.into_iter().collect()
    }
}

type PromptResultFuture<'a> =
    Pin<Box<dyn Future<Output = Result<GetPromptResult>> + Send + Sync + 'a>>;

#[doc(hidden)]
pub struct PromptDefinition {
    prompt: Prompt,
    #[allow(clippy::type_complexity)]
    f: Box<
        dyn for<'a> Fn(&'a GetPromptRequestParams, &'a RequestContext) -> PromptResultFuture<'a>
            + Send
            + Sync,
    >,
    completions: Vec<CompletionInfo>,
}
impl PromptDefinition {
    pub fn new(
        prompt: Prompt,
        f: impl for<'a> Fn(&'a GetPromptRequestParams, &'a RequestContext) -> PromptResultFuture<'a>
        + Send
        + Sync
        + 'static,
    ) -> Self {
        let f = Box::new(f);
        Self { prompt, f, completions: Vec::new() }
    }
    
    pub fn with_completions(mut self, completions: Vec<CompletionInfo>) -> Self {
        self.completions = completions;
        self
    }
}
impl From<PromptDefinition> for Route {
    fn from(value: PromptDefinition) -> Self {
        let completions = value.completions.clone();
        Route {
            prompts: vec![value],
            completions,
            ..Default::default()
        }
    }
}

type ResourceResultFuture<'a> =
    Pin<Box<dyn Future<Output = Result<ReadResourceResult>> + Send + Sync + 'a>>;

#[doc(hidden)]
pub struct ResourceDefinition {
    uri: Option<UriTemplate>,
    #[allow(clippy::type_complexity)]
    f: Box<
        dyn for<'a> Fn(
                &'a ReadResourceRequestParams,
                &'a Captures<'a>,
                &'a RequestContext,
            ) -> ResourceResultFuture<'a>
            + Send
            + Sync
            + 'static,
    >,
    name: String,
    description: Option<String>,
    mime_type: Option<String>,
    title: Option<String>,
    completions: Vec<CompletionInfo>,
}
impl ResourceDefinition {
    pub fn new(
        name: &str,
        uri: Option<&str>,
        f: impl for<'a> Fn(
            &'a ReadResourceRequestParams,
            &'a Captures<'a>,
            &'a RequestContext,
        ) -> ResourceResultFuture<'a>
        + Send
        + Sync
        + 'static,
    ) -> Result<Self> {
        let f = Box::new(f);
        Ok(Self {
            uri: uri.map(UriTemplate::new).transpose()?,
            f,
            name: name.to_string(),
            description: None,
            mime_type: None,
            title: None,
            completions: Vec::new(),
        })
    }
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }
    pub fn with_mime_type(mut self, mime_type: &str) -> Self {
        self.mime_type = Some(mime_type.to_string());
        self
    }
    pub fn with_title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }
    pub fn with_completions(mut self, completions: Vec<CompletionInfo>) -> Self {
        self.completions = completions;
        self
    }
    fn to_resource(&self) -> Option<Resource> {
        let uri = self.uri.as_ref()?;
        if uri.var_names().count() != 0 {
            return None;
        }
        Some(Resource {
            name: self.name.clone(),
            description: self.description.clone(),
            mime_type: self.mime_type.clone(),
            uri: uri.to_string(),
            size: None,
            annotations: None,
            meta: Default::default(),
            title: self.title.clone(),
        })
    }
    fn to_resource_template(&self) -> Option<ResourceTemplate> {
        let uri = self.uri.as_ref()?;
        if uri.var_names().count() == 0 {
            return None;
        }
        Some(ResourceTemplate {
            name: self.name.clone(),
            uri_template: uri.to_string(),
            description: self.description.clone(),
            mime_type: self.mime_type.clone(),
            annotations: None,
            meta: Default::default(),
            title: self.title.clone(),
        })
    }
    fn captures<'a>(&'a self, input: &'a str) -> Option<Captures<'a>> {
        if let Some(uri) = self.uri.as_ref() {
            uri.captures(input)
        } else {
            Some(Captures::empty())
        }
    }
}
impl From<ResourceDefinition> for Route {
    fn from(value: ResourceDefinition) -> Self {
        let completions = value.completions.clone();
        Route {
            resources: vec![value],
            completions,
            ..Default::default()
        }
    }
}

type ToolResultFuture<'a> =
    Pin<Box<dyn Future<Output = Result<CallToolResult>> + Send + Sync + 'a>>;

#[doc(hidden)]
pub struct ToolDefinition {
    tool: Tool,
    #[allow(clippy::type_complexity)]
    f: Box<
        dyn for<'a> Fn(&'a CallToolRequestParams, &'a RequestContext) -> ToolResultFuture<'a>
            + Send
            + Sync,
    >,
}
impl ToolDefinition {
    pub fn new(
        tool: Tool,
        f: impl for<'a> Fn(&'a CallToolRequestParams, &'a RequestContext) -> ToolResultFuture<'a>
        + Send
        + Sync
        + 'static,
    ) -> Self {
        let f = Box::new(f);
        Self { tool, f }
    }
}
impl From<ToolDefinition> for Route {
    fn from(value: ToolDefinition) -> Self {
        Route {
            tools: vec![value],
            ..Default::default()
        }
    }
}
