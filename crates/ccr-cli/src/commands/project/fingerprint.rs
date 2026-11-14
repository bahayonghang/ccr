use super::WorkflowEngine;
use super::workflow::{
    MINI_TRELLIS_MANAGED_SIGNATURE, TRELLIS_MANAGED_SIGNATURE, agents_md_contains_marker,
};
use ccr_core::core::error::{CcrError, Result};
use std::path::Path;

const TRELLIS_FILE_MARKERS: [&str; 4] = [
    ".trellis/workflow.md",
    ".trellis/scripts/task.py",
    ".trellis/.template-hashes.json",
    ".trellis/.version",
];

const MINI_TRELLIS_FILE_MARKERS: [&str; 5] = [
    ".trellis/.mini-trellis-manifest.json",
    ".trellis/spec/guides/mini-memory.md",
    ".claude/commands/mini-trellis/remember.md",
    ".agents/skills/mini-trellis-remember/SKILL.md",
    ".omp/extensions/mini-trellis/index.ts",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WorkflowInitMode {
    Fresh,
    Reinit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WorkspaceState {
    Uninitialized,
    Trellis,
    MiniTrellis,
    Mixed,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct WorkspaceMarkers {
    pub(crate) trellis: Vec<&'static str>,
    pub(crate) mini_trellis: Vec<&'static str>,
}

impl WorkspaceMarkers {
    pub(crate) fn state(&self) -> WorkspaceState {
        match (!self.trellis.is_empty(), !self.mini_trellis.is_empty()) {
            (false, false) => WorkspaceState::Uninitialized,
            (true, false) => WorkspaceState::Trellis,
            (false, true) => WorkspaceState::MiniTrellis,
            (true, true) => WorkspaceState::Mixed,
        }
    }
}

pub(crate) fn detect_workspace_markers(root: &Path) -> Result<WorkspaceMarkers> {
    let mut markers = WorkspaceMarkers::default();

    for rel in TRELLIS_FILE_MARKERS {
        if root.join(rel).is_file() {
            markers.trellis.push(rel);
        }
    }
    if agents_md_contains_marker(root, TRELLIS_MANAGED_SIGNATURE)? {
        markers.trellis.push("AGENTS.md (Managed by Trellis.)");
    }

    for rel in MINI_TRELLIS_FILE_MARKERS {
        if root.join(rel).is_file() {
            markers.mini_trellis.push(rel);
        }
    }
    if agents_md_contains_marker(root, MINI_TRELLIS_MANAGED_SIGNATURE)? {
        markers
            .mini_trellis
            .push("AGENTS.md (Managed by mini-trellis.)");
    }

    Ok(markers)
}

pub(crate) fn ensure_not_mixed_workspace(root: &Path) -> Result<WorkspaceMarkers> {
    let markers = detect_workspace_markers(root)?;
    if markers.state() == WorkspaceState::Mixed {
        return Err(mixed_workspace_error(&markers));
    }
    Ok(markers)
}

#[cfg(test)]
pub(crate) fn ensure_workflow_compatible(
    root: &Path,
    target: WorkflowEngine,
) -> Result<WorkflowInitMode> {
    let markers = detect_workspace_markers(root)?;
    validate_markers_for_target(&markers, target)
}

pub(crate) fn validate_markers_for_target(
    markers: &WorkspaceMarkers,
    target: WorkflowEngine,
) -> Result<WorkflowInitMode> {
    match (markers.state(), target) {
        (WorkspaceState::Uninitialized, _) => Ok(WorkflowInitMode::Fresh),
        (WorkspaceState::Trellis, WorkflowEngine::Trellis)
        | (WorkspaceState::MiniTrellis, WorkflowEngine::MiniTrellis) => {
            Ok(WorkflowInitMode::Reinit)
        }
        (WorkspaceState::Trellis, WorkflowEngine::MiniTrellis) => {
            Err(CcrError::ValidationError(format!(
                "工作流冲突：当前目录已存在 trellis 工作流资产（{}），不能直接初始化 mini-trellis；请先运行 `mini-trellis migrate --dry-run` 审阅迁移计划后再执行 `mini-trellis migrate`",
                markers.trellis.join(", ")
            )))
        }
        (WorkspaceState::MiniTrellis, WorkflowEngine::Trellis) => {
            Err(CcrError::ValidationError(format!(
                "工作流冲突：当前目录已存在 mini-trellis 工作流资产（{}），不能直接初始化 trellis；如需切换请先运行 `mini-trellis uninstall` 清理后再重试",
                markers.mini_trellis.join(", ")
            )))
        }
        (WorkspaceState::Mixed, _) => Err(mixed_workspace_error(markers)),
    }
}

fn mixed_workspace_error(markers: &WorkspaceMarkers) -> CcrError {
    CcrError::ValidationError(format!(
        "工作流冲突：当前目录存在双引擎混杂资产（trellis: {}；mini-trellis: {}），请先检查清理后再初始化",
        markers.trellis.join(", "),
        markers.mini_trellis.join(", ")
    ))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn write_file(root: &Path, rel: &str, content: &str) {
        let path = root.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
    }

    #[test]
    fn empty_or_bare_trellis_dir_is_uninitialized_and_fresh_for_both_engines() {
        let temp = tempdir().unwrap();
        fs::create_dir_all(temp.path().join(".trellis")).unwrap();

        let markers = detect_workspace_markers(temp.path()).unwrap();
        assert_eq!(markers.state(), WorkspaceState::Uninitialized);
        assert_eq!(
            ensure_workflow_compatible(temp.path(), WorkflowEngine::Trellis).unwrap(),
            WorkflowInitMode::Fresh
        );
        assert_eq!(
            ensure_workflow_compatible(temp.path(), WorkflowEngine::MiniTrellis).unwrap(),
            WorkflowInitMode::Fresh
        );
    }

    #[test]
    fn recognizes_every_trellis_marker_and_blocks_mini_trellis_with_migrate_hint() {
        for rel in [
            ".trellis/workflow.md",
            ".trellis/scripts/task.py",
            ".trellis/.template-hashes.json",
            ".trellis/.version",
        ] {
            let temp = tempdir().unwrap();
            write_file(temp.path(), rel, "marker");

            let markers = detect_workspace_markers(temp.path()).unwrap();
            assert_eq!(markers.state(), WorkspaceState::Trellis, "marker: {rel}");
            assert_eq!(
                ensure_workflow_compatible(temp.path(), WorkflowEngine::Trellis).unwrap(),
                WorkflowInitMode::Reinit
            );

            let err =
                ensure_workflow_compatible(temp.path(), WorkflowEngine::MiniTrellis).unwrap_err();
            assert!(matches!(err, CcrError::ValidationError(_)));
            assert_eq!(err.exit_code(), 90);
            assert!(
                err.to_string().contains("mini-trellis migrate"),
                "err: {err}"
            );
        }

        let temp = tempdir().unwrap();
        write_file(
            temp.path(),
            "AGENTS.md",
            "<!-- TRELLIS:START -->\n# Trellis\nManaged by Trellis.\n<!-- TRELLIS:END -->\n",
        );
        assert_eq!(
            detect_workspace_markers(temp.path()).unwrap().state(),
            WorkspaceState::Trellis
        );
    }

    #[test]
    fn recognizes_every_mini_trellis_marker_and_blocks_trellis_with_uninstall_hint() {
        for rel in [
            ".trellis/.mini-trellis-manifest.json",
            ".trellis/spec/guides/mini-memory.md",
            ".claude/commands/mini-trellis/remember.md",
            ".agents/skills/mini-trellis-remember/SKILL.md",
            ".omp/extensions/mini-trellis/index.ts",
        ] {
            let temp = tempdir().unwrap();
            write_file(temp.path(), rel, "marker");

            let markers = detect_workspace_markers(temp.path()).unwrap();
            assert_eq!(
                markers.state(),
                WorkspaceState::MiniTrellis,
                "marker: {rel}"
            );
            assert_eq!(
                ensure_workflow_compatible(temp.path(), WorkflowEngine::MiniTrellis).unwrap(),
                WorkflowInitMode::Reinit
            );

            let err = ensure_workflow_compatible(temp.path(), WorkflowEngine::Trellis).unwrap_err();
            assert!(matches!(err, CcrError::ValidationError(_)));
            assert_eq!(err.exit_code(), 90);
            assert!(
                err.to_string().contains("mini-trellis uninstall"),
                "err: {err}"
            );
        }

        let temp = tempdir().unwrap();
        write_file(
            temp.path(),
            "AGENTS.md",
            "<!-- TRELLIS:START -->\n# mini-trellis\nManaged by mini-trellis.\n<!-- TRELLIS:END -->\n",
        );
        assert_eq!(
            detect_workspace_markers(temp.path()).unwrap().state(),
            WorkspaceState::MiniTrellis
        );
    }

    #[test]
    fn mixed_markers_block_both_engines_with_mixed_asset_error() {
        let temp = tempdir().unwrap();
        write_file(temp.path(), ".trellis/workflow.md", "trellis");
        write_file(
            temp.path(),
            ".trellis/.mini-trellis-manifest.json",
            "mini-trellis",
        );

        let markers = detect_workspace_markers(temp.path()).unwrap();
        assert_eq!(markers.state(), WorkspaceState::Mixed);

        let mixed_err = ensure_not_mixed_workspace(temp.path()).unwrap_err();
        assert_eq!(mixed_err.exit_code(), 90);
        assert!(mixed_err.to_string().contains("混杂"));

        for engine in [WorkflowEngine::Trellis, WorkflowEngine::MiniTrellis] {
            let err = ensure_workflow_compatible(temp.path(), engine).unwrap_err();
            assert!(matches!(err, CcrError::ValidationError(_)));
            assert_eq!(err.exit_code(), 90);
            assert!(err.to_string().contains("混杂"), "err: {err}");
        }
    }
}
