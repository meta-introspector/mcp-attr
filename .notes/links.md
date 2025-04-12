# External Links

<!-- markdownlint-disable MD053 -->

## Badges

- [Crates.io](https://crates.io/crates/mcp-attr)
- [Docs.rs](https://docs.rs/mcp-attr/)
- [GitHub Actions](https://github.com/frozenlib/mcp-attr/actions)

## GitHub Repository

- [frozenlib/mcp-attr](https://github.com/frozenlib/mcp-attr)

## Documentation Links

- [GetPromptResult](https://docs.rs/mcp-attr/latest/mcp_attr/schema/struct.GetPromptResult.html)
- [ReadResourceResult](https://docs.rs/mcp-attr/latest/mcp_attr/schema/struct.ReadResourceResult.html)
- [CallToolResult](https://docs.rs/mcp-attr/latest/mcp_attr/schema/struct.CallToolResult.html)
- [mcp_attr::Error](https://docs.rs/mcp-attr/latest/mcp_attr/struct.Error.html)
- [mcp_attr::Result](https://docs.rs/mcp-attr/latest/mcp_attr/type.Result.html)
- [anyhow::Error](https://docs.rs/anyhow/latest/anyhow/struct.Error.html)
- [std::error::Error](https://doc.rust-lang.org/std/error/trait.Error.html)
- [anyhow::bail!](https://docs.rs/anyhow/latest/anyhow/macro.bail.html)
- [bail!](https://docs.rs/mcp-attr/latest/mcp_attr/macro.bail.html)
- [bail_public!](https://docs.rs/mcp-attr/latest/mcp_attr/macro.bail_public.html)
- [server_info](https://docs.rs/mcp-attr/latest/mcp_attr/server/trait.McpServer.html#method.server_info)
- [instructions](https://docs.rs/mcp-attr/latest/mcp_attr/server/trait.McpServer.html#method.instructions)
- [completion_complete](https://docs.rs/mcp-attr/latest/mcp_attr/server/trait.McpServer.html#method.completion_complete)
- [mcp_attr::server::McpServer](https://docs.rs/mcp-attr/latest/mcp_attr/server/trait.McpServer.html)
- [mcp_attr::client::McpClient](https://docs.rs/mcp-attr/latest/mcp_attr/client/struct.McpClient.html)
- [serde::DeserializeOwned](https://docs.rs/serde/latest/serde/de/trait.DeserializeOwned.html)
- [mcp_attr::server::RequestContext](https://docs.rs/mcp-attr/latest/mcp_attr/server/struct.RequestContext.html)

## MCP Specification Links

- [prompts/list](https://modelcontextprotocol.io/specification/2025-03-26/server/prompts/#listing-prompts)
- [prompts/get](https://modelcontextprotocol.io/specification/2025-03-26/server/prompts/#getting-a-prompt)
- [resources/list](https://modelcontextprotocol.io/specification/2025-03-26/server/resources/#listing-resources)
- [resources/read](https://modelcontextprotocol.io/specification/2025-03-26/server/resources/#reading-resources)
- [resources/templates/list](https://modelcontextprotocol.io/specification/2025-03-26/server/resources/#resource-templates)
- [tools/list](https://modelcontextprotocol.io/specification/2025-03-26/server/tools/#listing-tools)
- [tools/call](https://modelcontextprotocol.io/specification/2025-03-26/server/tools/#calling-tools)
- [roots/list](https://modelcontextprotocol.io/specification/2025-03-26/client/roots/#listing-roots)
- [sampling/createMessage](https://modelcontextprotocol.io/specification/2025-03-26/client/sampling/#creating-messages)
- [completion/complete](https://modelcontextprotocol.io/specification/2025-03-26/server/utilities/completion/#requesting-completions)
- [initialize](https://modelcontextprotocol.io/specification/2025-03-26/basic/lifecycle/#initialization)
- [ping](https://modelcontextprotocol.io/specification/2025-03-26/basic/utilities/ping/)

## Standards

- [RFC 6570](https://datatracker.ietf.org/doc/html/rfc6570) - URI Template

## Reference Links

[`prompts_list`]: crate::server::McpServer::prompts_list
[`prompts_get`]: crate::server::McpServer::prompts_get
[`resources_list`]: crate::server::McpServer::resources_list
[`resources_read`]: crate::server::McpServer::resources_read
[`resources_templates_list`]: crate::server::McpServer::resources_templates_list
[`tools_list`]: crate::server::McpServer::tools_list
[`tools_call`]: crate::server::McpServer::tools_call
[`FromStr`]: std::str::FromStr
[`JsonSchema`]: schemars::JsonSchema
[`UriTemplate`]: uri_template_ex::UriTemplate
[`Model Context Protocol`]: https://spec.modelcontextprotocol.io/
