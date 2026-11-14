use super::fingerprint::WorkflowInitMode;
use ccr_core::core::error::{CcrError, Result};
use clap::ValueEnum;
use std::fs;
use std::path::Path;

pub(crate) const TRELLIS_MANAGED_SIGNATURE: &str = "Managed by Trellis.";
pub(crate) const MINI_TRELLIS_MANAGED_SIGNATURE: &str = "Managed by mini-trellis.";
const MANAGED_BLOCK_START: &str = "<!-- TRELLIS:START -->";
const MANAGED_BLOCK_END: &str = "<!-- TRELLIS:END -->";

/// Workflow engine initialized by `ccr project init`.
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkflowEngine {
    /// Full task-driven multi-stage Trellis workflow.
    #[value(
        name = "trellis",
        help = "任务驱动的完整多阶段工作流（支持 5 项平台，含 Grok；-y 默认兼容）"
    )]
    Trellis,
    /// Lightweight spec, research, and cross-session memory layer.
    #[value(
        name = "mini-trellis",
        help = "轻量规格、研究笔记与跨会话记忆层（支持 4 项平台，不含 Grok；交互默认推荐）"
    )]
    MiniTrellis,
}

impl WorkflowEngine {
    // 交互态单选菜单顺序：默认第 1 项（索引 0）推荐 mini-trellis，第 2 项为 trellis。
    pub(crate) const INTERACTIVE_MENU_ORDER: [Self; 2] = [Self::MiniTrellis, Self::Trellis];

    // 解析 -y 非交互模式下的工作流引擎；省略 --workflow 时默认保持 trellis 兼容。
    pub(crate) fn resolve_non_interactive(explicit: Option<Self>) -> Self {
        explicit.unwrap_or(Self::Trellis)
    }

    pub(crate) fn binary_name(self) -> &'static str {
        match self {
            Self::Trellis => "trellis",
            Self::MiniTrellis => "mini-trellis",
        }
    }

    pub(crate) fn display_name(self) -> &'static str {
        match self {
            Self::Trellis => "Trellis",
            Self::MiniTrellis => "mini-trellis",
        }
    }

    pub(crate) fn menu_label_for(self, is_dumb: bool) -> &'static str {
        match (self, is_dumb) {
            (Self::MiniTrellis, false) => "mini-trellis — 轻量规格、研究笔记与跨会话记忆层 (推荐)",
            (Self::MiniTrellis, true) => "mini-trellis - 轻量规格、研究笔记与跨会话记忆层 (推荐)",
            (Self::Trellis, false) => "trellis — 任务驱动的完整多阶段工作流",
            (Self::Trellis, true) => "trellis - 任务驱动的完整多阶段工作流",
        }
    }

    pub(crate) fn supported_platforms(self) -> &'static [WorkflowPlatform] {
        match self {
            Self::Trellis => &WorkflowPlatform::TRELLIS_PLATFORMS,
            Self::MiniTrellis => &WorkflowPlatform::MINI_TRELLIS_PLATFORMS,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WorkflowPlatform {
    Claude,
    Codex,
    Cursor,
    Omp,
    Grok,
}

impl WorkflowPlatform {
    pub(crate) const TRELLIS_PLATFORMS: [Self; 5] = [
        Self::Claude,
        Self::Codex,
        Self::Cursor,
        Self::Omp,
        Self::Grok,
    ];

    pub(crate) const MINI_TRELLIS_PLATFORMS: [Self; 4] =
        [Self::Claude, Self::Codex, Self::Cursor, Self::Omp];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Claude => "Claude Code",
            Self::Codex => "Codex",
            Self::Cursor => "Cursor",
            Self::Omp => "Oh My Pi",
            Self::Grok => "Grok",
        }
    }

    pub(crate) fn flag(self) -> &'static str {
        match self {
            Self::Claude => "--claude",
            Self::Codex => "--codex",
            Self::Cursor => "--cursor",
            Self::Omp => "--omp",
            Self::Grok => "--grok",
        }
    }

    pub(crate) fn menu_label(self) -> &'static str {
        match self {
            Self::Claude => "Claude Code (--claude)",
            Self::Codex => "Codex (--codex)",
            Self::Cursor => "Cursor (--cursor)",
            Self::Omp => "Oh My Pi (--omp)",
            Self::Grok => "Grok (--grok)",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct WorkflowInitRequest {
    pub(crate) workflow: WorkflowEngine,
    pub(crate) mode: WorkflowInitMode,
    pub(crate) developer: String,
    pub(crate) platforms: Vec<WorkflowPlatform>,
    pub(crate) auto_yes: bool,
}

impl WorkflowInitRequest {
    // 按目标引擎定义的固定菜单顺序返回已启用平台的显示名称。
    pub(crate) fn ordered_platform_labels(&self) -> Vec<&'static str> {
        self.workflow
            .supported_platforms()
            .iter()
            .filter(|platform| self.platforms.contains(platform))
            .map(|platform| platform.label())
            .collect()
    }
}

pub(crate) fn build_workflow_init_args(request: &WorkflowInitRequest) -> Result<Vec<String>> {
    let stage = request.workflow.display_name();
    let developer = request.developer.trim();
    if developer.is_empty() {
        return Err(CcrError::ValidationError(format!(
            "{stage} 阶段：开发者用户名不能为空"
        )));
    }
    if request.platforms.is_empty() {
        return Err(CcrError::ValidationError(format!(
            "{stage} 阶段：至少选择一个平台"
        )));
    }

    let supported = request.workflow.supported_platforms();
    for platform in &request.platforms {
        if !supported.contains(platform) {
            return Err(CcrError::ValidationError(format!(
                "{stage} 阶段：{} 不支持平台 {} ({})",
                request.workflow.binary_name(),
                platform.label(),
                platform.flag()
            )));
        }
    }

    let mut args = vec!["init".to_string(), "-u".to_string(), developer.to_string()];
    for platform in supported {
        if request.platforms.contains(platform) {
            args.push(platform.flag().to_string());
        }
    }
    if request.auto_yes {
        args.push("--yes".to_string());
    }
    Ok(args)
}

pub(crate) fn validate_workflow_outputs(root: &Path, engine: WorkflowEngine) -> Result<()> {
    let mut missing = Vec::new();

    match engine {
        WorkflowEngine::Trellis => {
            for rel in [".trellis/workflow.md", ".trellis/scripts/task.py"] {
                let path = root.join(rel);
                if !path.is_file() {
                    missing.push(path.display().to_string());
                }
            }
        }
        WorkflowEngine::MiniTrellis => {
            for rel in [
                ".trellis/.mini-trellis-manifest.json",
                ".trellis/scripts/add_session.py",
            ] {
                let path = root.join(rel);
                if !path.is_file() {
                    missing.push(path.display().to_string());
                }
            }
            let agents_md = root.join("AGENTS.md");
            if !agents_md.is_file() {
                missing.push(agents_md.display().to_string());
            } else {
                match fs::read_to_string(&agents_md) {
                    Ok(content)
                        if managed_block_contains_marker(
                            &content,
                            MINI_TRELLIS_MANAGED_SIGNATURE,
                        ) => {}
                    Ok(_) => {
                        missing.push(format!(
                            "{} (缺少闭合受管块签名 {MINI_TRELLIS_MANAGED_SIGNATURE})",
                            agents_md.display()
                        ));
                    }
                    Err(error) => {
                        missing.push(format!(
                            "{} (无法作为有效 UTF-8 读取: {error})",
                            agents_md.display()
                        ));
                    }
                }
            }
        }
    }

    if missing.is_empty() {
        Ok(())
    } else {
        Err(CcrError::ValidationError(format!(
            "{} 阶段：{} init 成功退出，但缺少最低工作流文件: {}",
            engine.display_name(),
            engine.binary_name(),
            missing.join(", ")
        )))
    }
}

// 严格在闭合的 `<!-- TRELLIS:START -->...<!-- TRELLIS:END -->` 受管块内部检测标记。
pub(crate) fn managed_block_contains_marker(content: &str, marker: &str) -> bool {
    let mut remainder = content;
    while let Some(start_idx) = remainder.find(MANAGED_BLOCK_START) {
        let after_start = &remainder[start_idx + MANAGED_BLOCK_START.len()..];
        let Some(end_idx) = after_start.find(MANAGED_BLOCK_END) else {
            break;
        };
        let block = &after_start[..end_idx];
        let inner = block.rsplit(MANAGED_BLOCK_START).next().unwrap_or(block);
        if inner.contains(marker) {
            return true;
        }
        remainder = &after_start[end_idx + MANAGED_BLOCK_END.len()..];
    }
    false
}

pub(crate) fn agents_md_contains_marker(root: &Path, marker: &str) -> Result<bool> {
    let agents_path = root.join("AGENTS.md");
    if !agents_path.is_file() {
        return Ok(false);
    }

    let content = fs::read_to_string(&agents_path).map_err(|error| {
        CcrError::FileIoError(format!(
            "工作流检测阶段：读取 {} 失败: {error}",
            agents_path.display()
        ))
    })?;

    Ok(managed_block_contains_marker(&content, marker))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn workflow_defaults_preserve_auto_yes_trellis_and_interactive_mini_trellis_first() {
        assert_eq!(
            WorkflowEngine::resolve_non_interactive(None),
            WorkflowEngine::Trellis
        );
        assert_eq!(
            WorkflowEngine::resolve_non_interactive(Some(WorkflowEngine::MiniTrellis)),
            WorkflowEngine::MiniTrellis
        );
        assert_eq!(
            WorkflowEngine::INTERACTIVE_MENU_ORDER,
            [WorkflowEngine::MiniTrellis, WorkflowEngine::Trellis]
        );
    }

    #[test]
    fn mini_trellis_args_include_four_platforms_without_grok_or_skip_existing() {
        let request = WorkflowInitRequest {
            workflow: WorkflowEngine::MiniTrellis,
            mode: WorkflowInitMode::Fresh,
            developer: "  Ada Lovelace  ".to_string(),
            platforms: WorkflowEngine::MiniTrellis.supported_platforms().to_vec(),
            auto_yes: true,
        };

        let args = build_workflow_init_args(&request).unwrap();
        assert_eq!(
            args,
            vec![
                "init",
                "-u",
                "Ada Lovelace",
                "--claude",
                "--codex",
                "--cursor",
                "--omp",
                "--yes",
            ]
        );
        assert!(
            !args
                .iter()
                .any(|arg| arg == "--grok" || arg == "-s" || arg == "--skip-existing")
        );
    }

    #[test]
    fn mini_trellis_args_reject_unsupported_grok_platform() {
        let request = WorkflowInitRequest {
            workflow: WorkflowEngine::MiniTrellis,
            mode: WorkflowInitMode::Fresh,
            developer: "Ada".to_string(),
            platforms: vec![WorkflowPlatform::Claude, WorkflowPlatform::Grok],
            auto_yes: false,
        };

        let error = build_workflow_init_args(&request).unwrap_err();
        assert!(matches!(error, CcrError::ValidationError(_)));
        assert_eq!(error.exit_code(), 90);
    }

    #[test]
    fn mini_trellis_validation_requires_manifest_add_session_and_managed_agents_md() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        std::fs::create_dir_all(root.join(".trellis/scripts")).unwrap();

        // 1. 仅存在清单与 add_session.py（缺少 AGENTS.md）-> ValidationError (90)
        std::fs::write(root.join(".trellis/.mini-trellis-manifest.json"), "{}").unwrap();
        std::fs::write(root.join(".trellis/scripts/add_session.py"), "# script").unwrap();
        let err_no_agents =
            validate_workflow_outputs(root, WorkflowEngine::MiniTrellis).unwrap_err();
        assert_eq!(err_no_agents.exit_code(), 90);
        assert!(err_no_agents.to_string().contains("AGENTS.md"));

        // 2. AGENTS.md 仅包含 Trellis 受管签名（缺少 mini-trellis 签名）-> ValidationError (90)
        std::fs::write(
            root.join("AGENTS.md"),
            "<!-- TRELLIS:START -->\nManaged by Trellis.\n<!-- TRELLIS:END -->\n",
        )
        .unwrap();
        let err_wrong_agents =
            validate_workflow_outputs(root, WorkflowEngine::MiniTrellis).unwrap_err();
        assert_eq!(err_wrong_agents.exit_code(), 90);
        assert!(err_wrong_agents.to_string().contains("AGENTS.md"));

        // 3. AGENTS.md 受管块合法，但缺少 .mini-trellis-manifest.json -> ValidationError (90)
        std::fs::write(
            root.join("AGENTS.md"),
            "<!-- TRELLIS:START -->\n# mini-trellis\nManaged by mini-trellis.\n<!-- TRELLIS:END -->\n",
        )
        .unwrap();
        std::fs::remove_file(root.join(".trellis/.mini-trellis-manifest.json")).unwrap();
        let err_no_manifest =
            validate_workflow_outputs(root, WorkflowEngine::MiniTrellis).unwrap_err();
        assert_eq!(err_no_manifest.exit_code(), 90);
        assert!(
            err_no_manifest
                .to_string()
                .contains(".mini-trellis-manifest.json")
        );

        // 4. 恢复清单文件但移除 add_session.py -> ValidationError (90)
        std::fs::write(root.join(".trellis/.mini-trellis-manifest.json"), "{}").unwrap();
        std::fs::remove_file(root.join(".trellis/scripts/add_session.py")).unwrap();
        let err_no_script =
            validate_workflow_outputs(root, WorkflowEngine::MiniTrellis).unwrap_err();
        assert_eq!(err_no_script.exit_code(), 90);
        assert!(err_no_script.to_string().contains("add_session.py"));

        // 5. 三项最低工作流产物全部齐备 -> 校验通过
        std::fs::write(root.join(".trellis/scripts/add_session.py"), "# script").unwrap();
        validate_workflow_outputs(root, WorkflowEngine::MiniTrellis).unwrap();
    }

    #[test]
    fn mini_trellis_validation_rejects_unmanaged_unclosed_and_non_utf8_agents_md_with_exit_90() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        std::fs::create_dir_all(root.join(".trellis/scripts")).unwrap();
        std::fs::write(root.join(".trellis/.mini-trellis-manifest.json"), "{}").unwrap();
        std::fs::write(root.join(".trellis/scripts/add_session.py"), "# script").unwrap();

        // 无受管块或受管块未闭合的 AGENTS.md 均应按最低产物校验失败返回 90
        for invalid_text in [
            "# Notes\nManaged by mini-trellis.\n",
            "<!-- TRELLIS:START -->\n# Unclosed\nManaged by mini-trellis.\n",
        ] {
            std::fs::write(root.join("AGENTS.md"), invalid_text).unwrap();
            let err = validate_workflow_outputs(root, WorkflowEngine::MiniTrellis).unwrap_err();
            assert!(matches!(err, CcrError::ValidationError(_)), "err: {err}");
            assert_eq!(err.exit_code(), 90);
            assert!(err.to_string().contains("mini-trellis 阶段"), "err: {err}");
            assert!(err.to_string().contains("AGENTS.md"), "err: {err}");
        }

        // 非 UTF-8 损坏的 AGENTS.md 在第 3 阶段同样统一返回 ValidationError (90)
        std::fs::write(root.join("AGENTS.md"), [0xff, 0xfe, 0x00]).unwrap();
        let non_utf8_err =
            validate_workflow_outputs(root, WorkflowEngine::MiniTrellis).unwrap_err();
        assert!(
            matches!(non_utf8_err, CcrError::ValidationError(_)),
            "err: {non_utf8_err}"
        );
        assert_eq!(non_utf8_err.exit_code(), 90);
        assert!(
            non_utf8_err.to_string().contains("mini-trellis 阶段"),
            "err: {non_utf8_err}"
        );
        assert!(
            non_utf8_err.to_string().contains("AGENTS.md"),
            "err: {non_utf8_err}"
        );
    }
}
