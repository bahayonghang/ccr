use super::WorkflowEngine;
use super::fingerprint::WorkflowInitMode;
use super::workflow::WorkflowInitRequest;
use crate::commands::common::{new_table, new_utf8_table};
use ccr_core::core::logging::OutputStatus;
use colored::Colorize;
use comfy_table::{Attribute, Cell, Color as TableColor, Table};
use std::io::IsTerminal;
use std::path::{Path, PathBuf};

const HEADER_TITLE: &str = "项目工作流初始化";
const HEADER_UNDERLINE: &str = "================";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct WizardCapabilities {
    pub(crate) is_tty: bool,
    pub(crate) is_dumb: bool,
    pub(crate) style: bool,
}

impl WizardCapabilities {
    pub(crate) fn for_stdout() -> Self {
        Self::from_env(std::io::stdout().is_terminal())
    }

    pub(crate) fn for_stderr() -> Self {
        Self::from_env(std::io::stderr().is_terminal())
    }

    fn from_env(is_tty: bool) -> Self {
        let term = std::env::var("TERM").ok();
        let no_color = std::env::var_os("NO_COLOR").map(|_| "1");
        let clicolor = std::env::var("CLICOLOR").ok();
        let clicolor_force = std::env::var("CLICOLOR_FORCE").ok();
        let mut caps = Self::resolve(
            is_tty,
            term.as_deref(),
            no_color,
            clicolor.as_deref(),
            clicolor_force.as_deref(),
        );
        if !colored::control::SHOULD_COLORIZE.should_colorize() {
            caps.style = false;
        }
        caps
    }

    pub(crate) fn resolve(
        is_tty: bool,
        term: Option<&str>,
        no_color: Option<&str>,
        clicolor: Option<&str>,
        clicolor_force: Option<&str>,
    ) -> Self {
        let is_dumb = term == Some("dumb");
        let force_color = clicolor_force.is_some_and(|value| !value.is_empty() && value != "0");
        let color_disabled = no_color.is_some() || clicolor == Some("0");
        let style = is_tty && !is_dumb && (force_color || !color_disabled);
        Self {
            is_tty,
            is_dumb,
            style,
        }
    }

    pub(crate) fn symbols(self) -> bool {
        self.is_tty && !self.is_dumb
    }

    pub(crate) fn utf8_table(self) -> bool {
        !self.is_dumb
    }
}

pub(crate) fn format_kv_line(key: &str, value: &str, indent: usize, style: bool) -> String {
    let padding = " ".repeat(indent);
    let styled_key = if style {
        key.bold().to_string()
    } else {
        key.to_string()
    };
    let mut lines = value.split('\n');
    let mut output = format!(
        "{padding}{styled_key}: {}",
        lines.next().unwrap_or_default()
    );
    for line in lines {
        output.push_str(&format!("\n{padding}  {line}"));
    }
    output
}

pub(crate) fn format_header(
    root: &Path,
    auto_yes: bool,
    explicit_workflow: Option<WorkflowEngine>,
    caps: WizardCapabilities,
) -> String {
    let (title, underline) = if caps.style {
        (
            HEADER_TITLE.blue().bold().to_string(),
            HEADER_UNDERLINE.blue().to_string(),
        )
    } else {
        (HEADER_TITLE.to_string(), HEADER_UNDERLINE.to_string())
    };

    let mode_label = if auto_yes {
        "非交互自动模式 (-y)"
    } else {
        "交互式向导"
    };

    let mut lines = vec![
        title,
        underline,
        format_kv_line("目标目录", &root.display().to_string(), 2, caps.style),
        format_kv_line("运行模式", mode_label, 2, caps.style),
    ];

    if let Some(engine) = explicit_workflow {
        lines.push(format_kv_line(
            "工作流预设",
            engine.binary_name(),
            2,
            caps.style,
        ));
    }

    lines.join("\n")
}

pub(crate) fn format_wizard_status(
    status: OutputStatus,
    msg: &str,
    caps: WizardCapabilities,
) -> String {
    let (marker, label) = match (status, caps.symbols()) {
        (OutputStatus::Success, true) => ("✓", ""),
        (OutputStatus::Warning, true) => ("!", " 警告:"),
        (OutputStatus::Error, true) => ("×", " 错误:"),
        (OutputStatus::Step, true) => ("→", ""),
        (OutputStatus::Skipped, true) => ("-", " 跳过:"),
        (OutputStatus::Success, false) => ("成功:", ""),
        (OutputStatus::Warning, false) => ("警告:", ""),
        (OutputStatus::Error, false) => ("错误:", ""),
        (OutputStatus::Step, false) => ("进度:", ""),
        (OutputStatus::Skipped, false) => ("跳过:", ""),
    };

    let marker = if caps.style {
        match status {
            OutputStatus::Success => marker.green().bold().to_string(),
            OutputStatus::Warning => marker.yellow().bold().to_string(),
            OutputStatus::Error => marker.red().bold().to_string(),
            OutputStatus::Step => marker.cyan().bold().to_string(),
            OutputStatus::Skipped => marker.to_string(),
        }
    } else {
        marker.to_string()
    };

    format!("{marker}{label} {msg}")
}

pub(crate) fn format_stage_1_git(caps: WizardCapabilities) -> String {
    format_wizard_status(OutputStatus::Step, "[1/4] 检查 Git 仓库", caps)
}

pub(crate) fn format_stage_2_config(caps: WizardCapabilities) -> String {
    format_wizard_status(OutputStatus::Step, "[2/4] 配置工作流与宿主平台", caps)
}

pub(crate) fn format_stage_3_engine(engine: WorkflowEngine, caps: WizardCapabilities) -> String {
    format_wizard_status(
        OutputStatus::Step,
        &format!("[3/4] 执行 {} 初始化", engine.binary_name()),
        caps,
    )
}

pub(crate) fn format_stage_4_workspace_files(caps: WizardCapabilities) -> String {
    format_wizard_status(
        OutputStatus::Step,
        "[4/4] 准备 ref/ 目录与 .gitignore 忽略规则",
        caps,
    )
}

pub(crate) fn ordered_platform_labels(request: &WorkflowInitRequest) -> Vec<&'static str> {
    request
        .workflow
        .supported_platforms()
        .iter()
        .filter(|platform| request.platforms.contains(platform))
        .map(|platform| platform.label())
        .collect()
}

pub(crate) fn format_config_confirmation(
    request: &WorkflowInitRequest,
    caps: WizardCapabilities,
) -> String {
    let platforms = ordered_platform_labels(request).join(", ");
    [
        format_kv_line("工作流引擎", request.workflow.binary_name(), 2, caps.style),
        format_kv_line("宿主平台", &platforms, 2, caps.style),
        format_kv_line("开发者", request.developer.trim(), 2, caps.style),
    ]
    .join("\n")
}

pub(crate) fn format_subprocess_start(engine: WorkflowEngine, caps: WizardCapabilities) -> String {
    let binary = engine.binary_name();
    if !caps.symbols() {
        return format!("  --- 开始执行 {binary} init ---");
    }

    let trailing_dashes = match engine {
        WorkflowEngine::MiniTrellis => "─".repeat(26),
        WorkflowEngine::Trellis => "─".repeat(31),
    };
    let line = format!("  ┌─ 外部工具输出 ({binary} init) {trailing_dashes}");
    if caps.style {
        line.dimmed().to_string()
    } else {
        line
    }
}

pub(crate) fn format_subprocess_end(engine: WorkflowEngine, caps: WizardCapabilities) -> String {
    let binary = engine.binary_name();
    if !caps.symbols() {
        return format!("  --- {binary} init 执行结束 ---");
    }

    let line = format!("  └─ 外部工具执行完毕 {}", "─".repeat(42));
    if caps.style {
        line.dimmed().to_string()
    } else {
        line
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum GitRepoOutcome {
    Initialized,
    ExistingRoot,
    ParentWorktree { repository_root: PathBuf },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RefDirOutcome {
    Created,
    Existing,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum GitignoreOutcome {
    Updated { added_rules: Vec<&'static str> },
    Unchanged,
}

pub(crate) struct ProjectInitSummary<'a> {
    pub(crate) git: &'a GitRepoOutcome,
    pub(crate) request: &'a WorkflowInitRequest,
    pub(crate) ref_dir: RefDirOutcome,
    pub(crate) gitignore: &'a GitignoreOutcome,
}

const MINI_TRELLIS_NEXT_STEPS: [(&str, &str); 2] = [
    (
        "检查 mini-trellis 宿主接线与健康状态",
        "mini-trellis doctor",
    ),
    (
        "检索项目规格与历史记忆",
        "mini-trellis mem recall \"project init\"",
    ),
];

const TRELLIS_NEXT_STEPS: [(&str, &str); 2] = [
    ("检查 Trellis 工作流配置与环境状态", "trellis doctor"),
    ("查看项目初始化命令与引擎说明", "ccr project init --help"),
];

#[derive(Clone, Copy)]
enum SummaryStatusTone {
    Active,
    Reused,
}

fn summary_cell(
    text: impl Into<String>,
    style: bool,
    color: Option<TableColor>,
    bold: bool,
) -> Cell {
    let mut cell = Cell::new(text.into());
    if style {
        if let Some(fg) = color {
            cell = cell.fg(fg);
        }
        if bold {
            cell = cell.add_attribute(Attribute::Bold);
        }
    }
    cell
}

pub(crate) fn build_summary_table(
    summary: &ProjectInitSummary<'_>,
    caps: WizardCapabilities,
) -> Table {
    let mut table = if caps.utf8_table() {
        new_utf8_table()
    } else {
        new_table()
    };

    if !caps.style {
        table.force_no_tty();
    }

    let headers = ["阶段项", "状态", "结果详情"]
        .map(|title| summary_cell(title, caps.style, Some(TableColor::Cyan), true));
    table.set_header(headers);

    let (git_status, git_tone, git_detail) = match summary.git {
        GitRepoOutcome::Initialized => (
            "已新建",
            SummaryStatusTone::Active,
            "已执行 git init 初始化仓库".to_string(),
        ),
        GitRepoOutcome::ExistingRoot => (
            "复用",
            SummaryStatusTone::Reused,
            "当前目录已是仓库根".to_string(),
        ),
        GitRepoOutcome::ParentWorktree { repository_root } => (
            "复用",
            SummaryStatusTone::Reused,
            format!(
                "复用父仓库 {}（跳过嵌套 git init）",
                repository_root.display()
            ),
        ),
    };

    let binary = summary.request.workflow.binary_name();
    let (workflow_status, workflow_detail) = match summary.request.mode {
        WorkflowInitMode::Fresh => ("已初始化", format!("{binary}（首次初始化）")),
        WorkflowInitMode::Reinit => ("已更新", format!("{binary}（幂等更新）")),
    };

    let platform_labels = ordered_platform_labels(summary.request);
    let platform_status = format!("{} 项启用", platform_labels.len());
    let platform_detail = platform_labels.join(", ");

    let (ref_status, ref_tone, ref_detail) = match summary.ref_dir {
        RefDirOutcome::Created => ("已新建", SummaryStatusTone::Active, "ref/".to_string()),
        RefDirOutcome::Existing => (
            "复用",
            SummaryStatusTone::Reused,
            "保留现有 ref/ 目录内容".to_string(),
        ),
    };

    let (gitignore_status, gitignore_tone, gitignore_detail) = match summary.gitignore {
        GitignoreOutcome::Updated { added_rules } => {
            ("已更新", SummaryStatusTone::Active, added_rules.join("、"))
        }
        GitignoreOutcome::Unchanged => (
            "跳过",
            SummaryStatusTone::Reused,
            "已包含全部项目忽略规则，跳过写入".to_string(),
        ),
    };

    let rows: [(&str, &str, SummaryStatusTone, String); 6] = [
        ("Git 仓库", git_status, git_tone, git_detail),
        (
            "工作流引擎",
            workflow_status,
            SummaryStatusTone::Active,
            workflow_detail,
        ),
        (
            "宿主平台",
            &platform_status,
            SummaryStatusTone::Active,
            platform_detail,
        ),
        (
            "开发者",
            "已设置",
            SummaryStatusTone::Active,
            summary.request.developer.trim().to_string(),
        ),
        ("ref/ 目录", ref_status, ref_tone, ref_detail),
        (
            ".gitignore",
            gitignore_status,
            gitignore_tone,
            gitignore_detail,
        ),
    ];

    for (item, status, tone, detail) in rows {
        let status_color = match tone {
            SummaryStatusTone::Active => TableColor::Green,
            SummaryStatusTone::Reused => TableColor::Yellow,
        };
        table.add_row(vec![
            summary_cell(item, caps.style, None, caps.style),
            summary_cell(status, caps.style, Some(status_color), true),
            summary_cell(detail, caps.style, None, false),
        ]);
    }

    table
}

pub(crate) fn render_summary_table(
    summary: &ProjectInitSummary<'_>,
    caps: WizardCapabilities,
) -> String {
    build_summary_table(summary, caps).to_string()
}

pub(crate) fn format_final_ready_line(engine: WorkflowEngine, caps: WizardCapabilities) -> String {
    format_wizard_status(
        OutputStatus::Success,
        &format!(
            "项目工作流初始化完成（Git、{}、Agent 忽略规则和 ref/ 均已就绪）",
            engine.stage_name()
        ),
        caps,
    )
}

pub(crate) fn next_steps_for_engine(
    engine: WorkflowEngine,
) -> &'static [(&'static str, &'static str)] {
    match engine {
        WorkflowEngine::MiniTrellis => &MINI_TRELLIS_NEXT_STEPS,
        WorkflowEngine::Trellis => &TRELLIS_NEXT_STEPS,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn capabilities_distinguish_styled_tty_no_color_captured_and_dumb_modes() {
        let styled_tty =
            WizardCapabilities::resolve(true, Some("xterm-256color"), None, None, None);
        assert!(styled_tty.is_tty);
        assert!(!styled_tty.is_dumb);
        assert!(styled_tty.style);
        assert!(styled_tty.symbols());
        assert!(styled_tty.utf8_table());

        let no_color_tty =
            WizardCapabilities::resolve(true, Some("xterm-256color"), Some("1"), None, None);
        assert!(no_color_tty.is_tty);
        assert!(!no_color_tty.is_dumb);
        assert!(!no_color_tty.style);
        assert!(no_color_tty.symbols());
        assert!(no_color_tty.utf8_table());

        let captured =
            WizardCapabilities::resolve(false, Some("xterm-256color"), None, None, Some("1"));
        assert!(!captured.is_tty);
        assert!(!captured.is_dumb);
        assert!(!captured.style);
        assert!(!captured.symbols());
        assert!(captured.utf8_table());

        let dumb = WizardCapabilities::resolve(true, Some("dumb"), None, None, Some("1"));
        assert!(dumb.is_tty);
        assert!(dumb.is_dumb);
        assert!(!dumb.style);
        assert!(!dumb.symbols());
        assert!(!dumb.utf8_table());
    }

    #[test]
    fn header_renders_context_fields_and_optional_workflow_preset_without_ansi_when_unstyled() {
        let plain = WizardCapabilities::resolve(false, Some("xterm-256color"), None, None, None);
        let interactive = format_header(Path::new("/tmp/demo-app"), false, None, plain);
        assert_eq!(
            interactive,
            concat!(
                "项目工作流初始化\n",
                "================\n",
                "  目标目录: /tmp/demo-app\n",
                "  运行模式: 交互式向导"
            )
        );
        assert!(!interactive.contains("\u{1b}["));

        let auto_with_preset = format_header(
            Path::new("/tmp/demo-app"),
            true,
            Some(WorkflowEngine::MiniTrellis),
            plain,
        );
        assert_eq!(
            auto_with_preset,
            concat!(
                "项目工作流初始化\n",
                "================\n",
                "  目标目录: /tmp/demo-app\n",
                "  运行模式: 非交互自动模式 (-y)\n",
                "  工作流预设: mini-trellis"
            )
        );
        assert!(!auto_with_preset.contains("\u{1b}["));
    }

    #[test]
    fn stages_confirmation_and_boundaries_adapt_between_tty_no_color_captured_and_dumb() {
        use crate::commands::project::fingerprint::WorkflowInitMode;
        use crate::commands::project::workflow::WorkflowPlatform;

        let no_color_tty =
            WizardCapabilities::resolve(true, Some("xterm-256color"), Some("1"), None, None);
        let captured = WizardCapabilities::resolve(false, Some("xterm-256color"), None, None, None);
        let dumb = WizardCapabilities::resolve(true, Some("dumb"), None, None, Some("1"));

        assert_eq!(format_stage_1_git(no_color_tty), "→ [1/4] 检查 Git 仓库");
        assert_eq!(
            format_stage_2_config(no_color_tty),
            "→ [2/4] 配置工作流与宿主平台"
        );
        assert_eq!(
            format_stage_3_engine(WorkflowEngine::MiniTrellis, no_color_tty),
            "→ [3/4] 执行 mini-trellis 初始化"
        );
        assert_eq!(
            format_stage_4_workspace_files(no_color_tty),
            "→ [4/4] 准备 ref/ 目录与 .gitignore 忽略规则"
        );

        assert_eq!(format_stage_1_git(captured), "进度: [1/4] 检查 Git 仓库");
        assert_eq!(
            format_stage_3_engine(WorkflowEngine::Trellis, dumb),
            "进度: [3/4] 执行 trellis 初始化"
        );

        let req = WorkflowInitRequest {
            workflow: WorkflowEngine::MiniTrellis,
            mode: WorkflowInitMode::Fresh,
            developer: "  Ada Lovelace  ".to_string(),
            platforms: vec![
                WorkflowPlatform::Omp,
                WorkflowPlatform::Claude,
                WorkflowPlatform::Codex,
                WorkflowPlatform::Cursor,
            ],
            auto_yes: false,
        };
        assert_eq!(
            format_config_confirmation(&req, captured),
            concat!(
                "  工作流引擎: mini-trellis\n",
                "  宿主平台: Claude Code, Codex, Cursor, Oh My Pi\n",
                "  开发者: Ada Lovelace"
            )
        );

        let tty_start = format_subprocess_start(WorkflowEngine::MiniTrellis, no_color_tty);
        let tty_end = format_subprocess_end(WorkflowEngine::MiniTrellis, no_color_tty);
        assert!(tty_start.starts_with("  ┌─ 外部工具输出 (mini-trellis init) ──"));
        assert!(tty_end.starts_with("  └─ 外部工具执行完毕 ──"));
        assert!(!tty_start.contains("\u{1b}["));
        assert!(!tty_end.contains("\u{1b}["));

        for plain_caps in [captured, dumb] {
            assert_eq!(
                format_subprocess_start(WorkflowEngine::MiniTrellis, plain_caps),
                "  --- 开始执行 mini-trellis init ---"
            );
            assert_eq!(
                format_subprocess_end(WorkflowEngine::MiniTrellis, plain_caps),
                "  --- mini-trellis init 执行结束 ---"
            );
            assert_eq!(
                format_wizard_status(
                    OutputStatus::Success,
                    "mini-trellis 工作流产物校验通过",
                    plain_caps
                ),
                "成功: mini-trellis 工作流产物校验通过"
            );
        }
    }

    #[test]
    fn summary_table_ready_line_and_next_steps_reflect_outcomes_truncation_and_dumb_ascii() {
        use crate::commands::project::gitignore::PROJECT_IGNORE_RULES;
        use crate::commands::project::workflow::WorkflowPlatform;
        use comfy_table::ContentArrangement;

        let no_color_tty =
            WizardCapabilities::resolve(true, Some("xterm-256color"), Some("1"), None, None);
        let captured = WizardCapabilities::resolve(false, Some("xterm-256color"), None, None, None);
        let dumb = WizardCapabilities::resolve(true, Some("dumb"), None, None, Some("1"));

        let fresh_request = WorkflowInitRequest {
            workflow: WorkflowEngine::MiniTrellis,
            mode: WorkflowInitMode::Fresh,
            developer: "Ada Lovelace".to_string(),
            platforms: WorkflowPlatform::MINI_TRELLIS_PLATFORMS.to_vec(),
            auto_yes: false,
        };
        let fresh_git = GitRepoOutcome::ExistingRoot;
        let fresh_gitignore = GitignoreOutcome::Updated {
            added_rules: PROJECT_IGNORE_RULES.to_vec(),
        };
        let fresh_summary = ProjectInitSummary {
            git: &fresh_git,
            request: &fresh_request,
            ref_dir: RefDirOutcome::Created,
            gitignore: &fresh_gitignore,
        };

        let new_repo_git = GitRepoOutcome::Initialized;
        let new_repo_summary = ProjectInitSummary {
            git: &new_repo_git,
            request: &fresh_request,
            ref_dir: RefDirOutcome::Created,
            gitignore: &fresh_gitignore,
        };
        let new_repo_table = render_summary_table(&new_repo_summary, captured);
        assert!(new_repo_table.contains("已执行 git init 初始化仓库"));

        let utf8_output = render_summary_table(&fresh_summary, captured);
        assert!(!utf8_output.contains("\u{1b}["));
        assert!(utf8_output.starts_with('┌'), "{utf8_output}");
        for expected in [
            "阶段项",
            "状态",
            "结果详情",
            "Git 仓库",
            "复用",
            "当前目录已是仓库根",
            "工作流引擎",
            "已初始化",
            "mini-trellis（首次初始化）",
            "宿主平台",
            "4 项启用",
            "Claude Code, Codex, Cursor, Oh My Pi",
            "开发者",
            "已设置",
            "Ada Lovelace",
            "ref/ 目录",
            "已新建",
            ".gitignore",
            "已更新",
            ".agents/、.claude/、.codex/、.cursor/、.omp/、.grok/、ref/",
        ] {
            assert!(
                utf8_output.contains(expected),
                "missing {expected} in:\n{utf8_output}"
            );
        }

        // Idempotent rerun in parent worktree under TERM=dumb -> ASCII borders + zero decorative Unicode
        let reinit_request = WorkflowInitRequest {
            workflow: WorkflowEngine::Trellis,
            mode: WorkflowInitMode::Reinit,
            developer: "Grace Hopper".to_string(),
            platforms: WorkflowPlatform::TRELLIS_PLATFORMS.to_vec(),
            auto_yes: true,
        };
        let parent_git = GitRepoOutcome::ParentWorktree {
            repository_root: PathBuf::from("/repo/parent"),
        };
        let skipped_gitignore = GitignoreOutcome::Unchanged;
        let reinit_summary = ProjectInitSummary {
            git: &parent_git,
            request: &reinit_request,
            ref_dir: RefDirOutcome::Existing,
            gitignore: &skipped_gitignore,
        };

        let dumb_table = render_summary_table(&reinit_summary, dumb);
        assert!(!dumb_table.contains("\u{1b}["));
        assert!(dumb_table.starts_with('+'), "{dumb_table}");
        for forbidden in ['┌', '─', '│', '┆', '╞', '╪', '╡', '└', '→', '✓', '⚠', '═']
        {
            assert!(
                !dumb_table.contains(forbidden),
                "unexpected {forbidden:?} in dumb table:\n{dumb_table}"
            );
        }
        for expected in [
            "复用父仓库 /repo/parent（跳过嵌套 git init）",
            "trellis（幂等更新）",
            "5 项启用",
            "保留现有 ref/ 目录内容",
            "跳过",
            "已包含全部项目忽略规则，跳过写入",
        ] {
            assert!(
                dumb_table.contains(expected),
                "missing {expected} in:\n{dumb_table}"
            );
        }

        // Narrow width truncation preserves `...` (never `…`)
        let mut narrow = build_summary_table(&fresh_summary, no_color_tty);
        for row in narrow.row_iter_mut() {
            row.max_height(1);
        }
        narrow
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_width(36);
        let narrow_rendered = narrow.to_string();
        assert!(narrow_rendered.contains("..."), "{narrow_rendered}");
        assert!(!narrow_rendered.contains('…'), "{narrow_rendered}");
        assert!(!narrow_rendered.contains("\u{1b}["));

        // Ready line & engine-specific next steps
        assert_eq!(
            format_final_ready_line(WorkflowEngine::MiniTrellis, no_color_tty),
            "✓ 项目工作流初始化完成（Git、mini-trellis、Agent 忽略规则和 ref/ 均已就绪）"
        );
        assert_eq!(
            format_final_ready_line(WorkflowEngine::Trellis, dumb),
            "成功: 项目工作流初始化完成（Git、Trellis、Agent 忽略规则和 ref/ 均已就绪）"
        );

        let mini_steps = next_steps_for_engine(WorkflowEngine::MiniTrellis);
        assert_eq!(mini_steps.len(), 2);
        assert_eq!(mini_steps[0].1, "mini-trellis doctor");
        assert_eq!(mini_steps[1].1, "mini-trellis mem recall \"project init\"");

        let trellis_steps = next_steps_for_engine(WorkflowEngine::Trellis);
        assert_eq!(trellis_steps.len(), 2);
        assert_eq!(trellis_steps[0].1, "trellis doctor");
        assert_eq!(trellis_steps[1].1, "ccr project init --help");
    }
}
