//! Codex CLI agent locator。
//! 约定路径：`~/.codex/skills` (全局)。Codex 不约定项目级目录。

use std::path::{Path, PathBuf};

use super::{AgentId, AgentLocator};

pub(super) struct CodexLocator;

impl AgentLocator for CodexLocator {
    fn id(&self) -> AgentId {
        AgentId::Codex
    }

    fn display_name(&self) -> &'static str {
        "Codex"
    }

    fn icon(&self) -> &'static str {
        "💻"
    }

    fn global_paths(&self, home: &Path) -> Vec<PathBuf> {
        vec![home.join(".codex").join("skills")]
    }

    fn project_paths(&self, _project_root: &Path) -> Vec<PathBuf> {
        Vec::new()
    }
}
