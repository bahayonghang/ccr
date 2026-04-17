//! Gemini CLI agent locator。
//! 约定路径：`~/.gemini/skills` (全局)。

use std::path::{Path, PathBuf};

use super::{AgentId, AgentLocator};

pub(super) struct GeminiLocator;

impl AgentLocator for GeminiLocator {
    fn id(&self) -> AgentId {
        AgentId::Gemini
    }

    fn display_name(&self) -> &'static str {
        "Gemini CLI"
    }

    fn icon(&self) -> &'static str {
        "✨"
    }

    fn global_paths(&self, home: &Path) -> Vec<PathBuf> {
        vec![home.join(".gemini").join("skills")]
    }

    fn project_paths(&self, _project_root: &Path) -> Vec<PathBuf> {
        Vec::new()
    }
}
