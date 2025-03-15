pub mod client;
mod common;
#[doc(hidden)]
pub mod helpers;

/// Types defined in the [MCP protocol schema]
///
/// This module was automatically generated from the [MCP protocol schema].
///
/// [MCP protocol schema]: https://github.com/modelcontextprotocol/specification/blob/main/schema/2024-11-05/schema.json
pub mod schema;
mod schema_ext;
pub mod server;
mod transitivity;
pub mod utils;

pub use jsoncall;
pub use jsoncall::{Error, ErrorCode, Result, bail, bail_public};

const PROTOCOL_VERSION: &str = "2024-11-05";

#[cfg(doctest)]
mod tests_readme;
