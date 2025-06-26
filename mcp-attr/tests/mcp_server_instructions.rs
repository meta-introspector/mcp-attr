use tokio::test;

use mcp_attr::{
    Result,
    server::{McpServer, mcp_server},
};

struct ExampleServer;

/// This is a test server for checking instructions functionality.
/// It provides various tools and resources for testing purposes.
#[mcp_server]
impl McpServer for ExampleServer {
    #[tool]
    async fn hello(&self) -> Result<String> {
        Ok("Hello, world!".to_string())
    }
}

#[test]
async fn test_instructions_from_doc_comment() -> Result<()> {
    let server = ExampleServer;
    let instructions = server.instructions();

    assert!(instructions.is_some());
    let instructions_text = instructions.unwrap();
    assert!(
        instructions_text
            .contains("This is a test server for checking instructions functionality.")
    );
    assert!(
        instructions_text.contains("It provides various tools and resources for testing purposes.")
    );

    Ok(())
}

struct ManualInstructionsServer;

/// This doc comment should be ignored because instructions is manually implemented.
#[mcp_server]
impl McpServer for ManualInstructionsServer {
    fn instructions(&self) -> Option<String> {
        Some("Manual implementation takes priority".to_string())
    }

    #[tool]
    async fn hello(&self) -> Result<String> {
        Ok("Hello, manual!".to_string())
    }
}

#[test]
async fn test_manual_instructions_priority() -> Result<()> {
    let server = ManualInstructionsServer;
    let instructions = server.instructions();

    assert!(instructions.is_some());
    let instructions_text = instructions.unwrap();
    assert_eq!(instructions_text, "Manual implementation takes priority");

    // Doc commentの内容は含まれていないことを確認
    assert!(!instructions_text.contains("This doc comment should be ignored"));

    Ok(())
}

struct NoDocCommentServer;

#[mcp_server]
impl McpServer for NoDocCommentServer {
    #[tool]
    async fn hello(&self) -> Result<String> {
        Ok("Hello, no doc!".to_string())
    }
}

#[test]
async fn test_no_doc_comment_returns_none() -> Result<()> {
    let server = NoDocCommentServer;
    let instructions = server.instructions();

    assert!(instructions.is_none());

    Ok(())
}

struct MultiLineDocServer;

/// This is a multi-line documentation comment.
///
/// It spans multiple lines and contains various information:
/// - Feature description
/// - Usage examples
/// - Additional notes
///
/// This should all be captured in the instructions.
#[mcp_server]
impl McpServer for MultiLineDocServer {
    #[tool]
    async fn hello(&self) -> Result<String> {
        Ok("Hello, multi-line!".to_string())
    }
}

#[test]
async fn test_multiline_doc_comment() -> Result<()> {
    let server = MultiLineDocServer;
    let instructions = server.instructions();

    assert!(instructions.is_some());
    let instructions_text = instructions.unwrap();

    // 複数行の内容がすべて含まれていることを確認
    assert!(instructions_text.contains("This is a multi-line documentation comment."));
    assert!(instructions_text.contains("It spans multiple lines"));
    assert!(instructions_text.contains("Feature description"));
    assert!(instructions_text.contains("Usage examples"));
    assert!(instructions_text.contains("Additional notes"));
    assert!(instructions_text.contains("This should all be captured in the instructions."));

    Ok(())
}
