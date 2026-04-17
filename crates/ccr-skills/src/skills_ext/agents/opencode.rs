//! OpenCode agent locator。
//! 约定路径：`~/.config/opencode/skills` (全局，XDG 风格)。项目级无约定。

use std::path::{Path, PathBuf};

use super::{AgentId, AgentLocator};

pub(super) struct OpenCodeLocator;

impl AgentLocator for OpenCodeLocator {
    fn id(&self) -> AgentId {
        AgentId::OpenCode
    }

    fn display_name(&self) -> &'static str {
        "OpenCode"
    }

    fn icon(&self) -> &'static str {
        "📖"
    }

    fn global_paths(&self, home: &Path) -> Vec<PathBuf> {
        vec![home.join(".config").join("opencode").join("skills")]
    }

    fn project_paths(&self, _project_root: &Path) -> Vec<PathBuf> {
        Vec::new()
    }
}
