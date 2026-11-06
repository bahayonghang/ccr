//! Compatibility adapter: policy is shared with the native desktop.
pub(crate) use ccr_app::claude_mcp::{
    ClaudeMcpDiagnostic, add_claude_mcp_server_default, delete_claude_mcp_server_default,
    list_claude_mcp_default, parse_scope, update_claude_mcp_server_default,
};
