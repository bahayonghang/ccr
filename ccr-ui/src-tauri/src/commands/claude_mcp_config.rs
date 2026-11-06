//! Compatibility adapter: policy is shared with the native desktop.
use ccr_app::claude_mcp::{self, ClaudeMcpContext, ClaudeMcpList, ClaudeMcpScope};
pub(crate) use ccr_app::claude_mcp::{ClaudeMcpDiagnostic, parse_scope};
use serde_json::Value;

fn context() -> Result<ClaudeMcpContext, String> {
    match crate::fixture::current() {
        Some(layout) => Ok(ClaudeMcpContext {
            user_state_path: layout.roots.claude.state_file.clone(),
            project_root: layout.path("project")?,
            lock_directory: Some(layout.roots.locks.clone()),
        }),
        None => ClaudeMcpContext::detect(),
    }
}

pub(crate) fn list_claude_mcp_default() -> Result<ClaudeMcpList, String> {
    claude_mcp::list_claude_mcp(&context()?)
}
pub(crate) fn add_claude_mcp_server_default(
    name: String,
    config: Value,
    scope: Option<ClaudeMcpScope>,
) -> Result<Value, String> {
    claude_mcp::add_claude_mcp_server(&context()?, name, config, scope)
}
pub(crate) fn update_claude_mcp_server_default(
    name: String,
    patch: Value,
    scope: Option<ClaudeMcpScope>,
) -> Result<Value, String> {
    claude_mcp::update_claude_mcp_server(&context()?, name, patch, scope)
}
pub(crate) fn delete_claude_mcp_server_default(
    name: String,
    scope: Option<ClaudeMcpScope>,
) -> Result<String, String> {
    claude_mcp::delete_claude_mcp_server(&context()?, name, scope)
}
