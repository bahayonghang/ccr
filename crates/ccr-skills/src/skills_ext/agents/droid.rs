//! Droid agent locator。
//! 约定路径：`~/.factory/skills` (全局) + `<project>/.factory/skills` (项目)。
//! 参考 skill-hub 的 agents.ts 注册表。

use std::path::{Path, PathBuf};

use super::{AgentId, AgentLocator};

pub(super) struct DroidLocator;

impl AgentLocator for DroidLocator {
    fn id(&self) -> AgentId {
        AgentId::Droid
    }

    fn display_name(&self) -> &'static str {
        "Droid"
    }

    fn icon(&self) -> &'static str {
        "🦾"
    }

    fn global_paths(&self, home: &Path) -> Vec<PathBuf> {
        vec![home.join(".factory").join("skills")]
    }

    fn project_paths(&self, project_root: &Path) -> Vec<PathBuf> {
        vec![project_root.join(".factory").join("skills")]
    }
}
