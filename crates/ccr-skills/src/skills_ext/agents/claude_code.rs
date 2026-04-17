//! Claude Code agent locator。
//! 约定路径：`~/.claude/skills` (全局) + `<project>/.claude/skills` (项目)。

use std::path::{Path, PathBuf};

use super::{AgentId, AgentLocator};

pub(super) struct ClaudeCodeLocator;

impl AgentLocator for ClaudeCodeLocator {
    fn id(&self) -> AgentId {
        AgentId::ClaudeCode
    }

    fn display_name(&self) -> &'static str {
        "Claude Code"
    }

    fn icon(&self) -> &'static str {
        "🤖"
    }

    fn global_paths(&self, home: &Path) -> Vec<PathBuf> {
        vec![home.join(".claude").join("skills")]
    }

    fn project_paths(&self, project_root: &Path) -> Vec<PathBuf> {
        vec![project_root.join(".claude").join("skills")]
    }
}
