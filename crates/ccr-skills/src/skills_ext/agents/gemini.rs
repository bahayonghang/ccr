//! Antigravity CLI agent locator。
//!
//! Internal agent id remains `gemini` for CCR compatibility. Antigravity uses
//! `.agents/skills` in workspaces and `~/.gemini/antigravity-cli/skills`
//! globally, while older `~/.gemini/skills` remains readable as a legacy/shared
//! source.

use std::path::{Path, PathBuf};

use super::{AgentId, AgentLocator};

pub(super) struct GeminiLocator;

impl AgentLocator for GeminiLocator {
    fn id(&self) -> AgentId {
        AgentId::Gemini
    }

    fn display_name(&self) -> &'static str {
        "Antigravity CLI"
    }

    fn icon(&self) -> &'static str {
        "✨"
    }

    fn global_paths(&self, home: &Path) -> Vec<PathBuf> {
        vec![
            home.join(".gemini").join("antigravity-cli").join("skills"),
            home.join(".gemini").join("skills"),
        ]
    }

    fn project_paths(&self, project_root: &Path) -> Vec<PathBuf> {
        vec![project_root.join(".agents").join("skills")]
    }
}
