use super::WorkflowEngine;
use super::fingerprint::WorkflowInitMode;
use super::workflow::WorkflowInitRequest;
use crate::commands::common::{new_table, new_utf8_table};
use ccr_core::core::logging::ColorOutput;
use colored::Colorize;
use comfy_table::{Attribute, Cell, Color as TableColor, ContentArrangement, Row, Table};
use std::io::IsTerminal;
use std::path::{Path, PathBuf};

const HEADER_TITLE: &str = "项目工作流初始化";
pub(crate) const STAGE_1_GIT_MESSAGE: &str = "[1/4] 检查 Git 仓库";
pub(crate) const STAGE_2_CONFIG_MESSAGE: &str = "[2/4] 配置工作流与宿主平台";
pub(crate) const STAGE_4_WORKSPACE_FILES_MESSAGE: &str =
    "[4/4] 准备 ref/ 目录与 .gitignore 忽略规则";

// 仅负责 `project init` 专属控件（dialoguer 主题、子进程边界线、摘要表样式与边框）的流级能力判定。
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

    pub(crate) fn from_env(is_tty: bool) -> Self {
        let term = std::env::var("TERM").ok();
        let no_color = std::env::var_os("NO_COLOR");
        let clicolor = std::env::var_os("CLICOLOR");
        let clicolor_force = std::env::var_os("CLICOLOR_FORCE");
        let no_color_str = match no_color.as_deref() {
            None => None,
            Some(v) if v.is_empty() => Some(""),
            Some(v) => Some(v.to_str().unwrap_or("1")),
        };
        let clicolor_str = match clicolor.as_deref() {
            None => None,
            Some(v) if v.is_empty() => Some(""),
            Some(v) if v == "0" => Some("0"),
            Some(v) => Some(v.to_str().unwrap_or("1")),
        };
        let clicolor_force_str = match clicolor_force.as_deref() {
            None => None,
            Some(v) if v.is_empty() => Some(""),
            Some(v) if v == "0" => Some("0"),
            Some(v) => Some(v.to_str().unwrap_or("1")),
        };
        Self::resolve(
            is_tty,
            term.as_deref(),
            no_color_str,
            clicolor_str,
            clicolor_force_str,
        )
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
        let color_disabled =
            no_color.is_some_and(|value| !value.is_empty()) || clicolor == Some("0");
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

pub(crate) fn header_context_fields(
    root: &Path,
    auto_yes: bool,
    explicit_workflow: Option<WorkflowEngine>,
) -> Vec<(&'static str, String)> {
    let mode_label = if auto_yes {
        "非交互自动模式 (-y)"
    } else {
        "交互式向导"
    };

    let mut fields = vec![
        ("目标目录", root.display().to_string()),
        ("运行模式", mode_label.to_string()),
    ];

    if let Some(engine) = explicit_workflow {
        fields.push(("工作流预设", engine.binary_name().to_string()));
    }

    fields
}

pub(crate) fn print_header(root: &Path, auto_yes: bool, explicit_workflow: Option<WorkflowEngine>) {
    ColorOutput::title(HEADER_TITLE);
    for (key, value) in header_context_fields(root, auto_yes, explicit_workflow) {
        ColorOutput::key_value(key, &value, 2);
    }
}

pub(crate) fn stage_3_engine_message(engine: WorkflowEngine) -> String {
    format!("[3/4] 执行 {} 初始化", engine.binary_name())
}

pub(crate) fn config_confirmation_fields(
    request: &WorkflowInitRequest,
) -> [(&'static str, String); 3] {
    let platforms = request.ordered_platform_labels().join(", ");
    [
        ("工作流引擎", request.workflow.binary_name().to_string()),
        ("宿主平台", platforms),
        ("开发者", request.developer.trim().to_string()),
    ]
}

pub(crate) fn print_config_confirmation(request: &WorkflowInitRequest) {
    for (key, value) in config_confirmation_fields(request) {
        ColorOutput::key_value(key, &value, 2);
    }
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
    table.set_content_arrangement(ContentArrangement::Dynamic);

    if caps.is_tty
        && let Some(tty_width) = table.width()
    {
        table.set_width(tty_width);
    }
    if !caps.style {
        table.force_no_tty();
    }

    let mut header_row = Row::from(
        ["阶段项", "状态", "结果详情"]
            .map(|title| summary_cell(title, caps.style, Some(TableColor::Cyan), true)),
    );
    header_row.max_height(1);
    table.set_header(header_row);

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

    let platform_labels = summary.request.ordered_platform_labels();
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
        let mut row = Row::from(vec![
            summary_cell(item, caps.style, None, caps.style),
            summary_cell(status, caps.style, Some(status_color), true),
            summary_cell(detail, caps.style, None, false),
        ]);
        row.max_height(2);
        table.add_row(row);
    }

    table
}

pub(crate) fn render_summary_table(
    summary: &ProjectInitSummary<'_>,
    caps: WizardCapabilities,
) -> String {
    build_summary_table(summary, caps).to_string()
}

pub(crate) fn final_ready_message(engine: WorkflowEngine) -> String {
    format!(
        "项目工作流初始化完成（Git、{}、Agent 忽略规则和 ref/ 均已就绪）",
        engine.display_name()
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
    use ccr_core::core::logging::OutputStatus;
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
    fn empty_no_color_and_redirected_stdout_keep_tty_stream_styled() {
        use std::ffi::OsStr;

        let empty_no_color =
            WizardCapabilities::resolve(true, Some("xterm-256color"), Some(""), None, None);
        assert!(empty_no_color.style);
        assert!(empty_no_color.symbols());

        let empty_clicolor =
            WizardCapabilities::resolve(true, Some("xterm-256color"), None, Some(""), None);
        assert!(empty_clicolor.style);

        let clicolor_zero =
            WizardCapabilities::resolve(true, Some("xterm-256color"), None, Some("0"), None);
        assert!(!clicolor_zero.style);

        for non_forcing in [Some(""), Some("0")] {
            let unforced = WizardCapabilities::resolve(
                true,
                Some("xterm-256color"),
                Some("1"),
                None,
                non_forcing,
            );
            assert!(!unforced.style);
        }

        let forced_over_no_color = WizardCapabilities::resolve(
            true,
            Some("xterm-256color"),
            Some("1"),
            Some("0"),
            Some("1"),
        );
        assert!(forced_over_no_color.style);

        let mut env = crate::test_support::TestHostEnv::new();
        env.set_env("TERM", OsStr::new("xterm-256color"));
        env.set_env("NO_COLOR", OsStr::new(""));
        env.remove_env("CLICOLOR");
        env.remove_env("CLICOLOR_FORCE");

        let redirected_stdout = WizardCapabilities::from_env(false);
        let interactive_stderr = WizardCapabilities::from_env(true);
        assert!(!redirected_stdout.style);
        assert!(interactive_stderr.style);
    }

    #[test]
    fn header_context_fields_reflect_mode_and_optional_workflow_preset() {
        let interactive = header_context_fields(Path::new("/tmp/demo-app"), false, None);
        assert_eq!(
            interactive,
            vec![
                ("目标目录", "/tmp/demo-app".to_string()),
                ("运行模式", "交互式向导".to_string()),
            ]
        );

        let auto_with_preset = header_context_fields(
            Path::new("/tmp/demo-app"),
            true,
            Some(WorkflowEngine::MiniTrellis),
        );
        assert_eq!(
            auto_with_preset,
            vec![
                ("目标目录", "/tmp/demo-app".to_string()),
                ("运行模式", "非交互自动模式 (-y)".to_string()),
                ("工作流预设", "mini-trellis".to_string()),
            ]
        );
    }

    #[test]
    fn stages_confirmation_and_boundaries_adapt_between_tty_no_color_captured_and_dumb() {
        use crate::commands::project::fingerprint::WorkflowInitMode;
        use crate::commands::project::workflow::WorkflowPlatform;
        use std::ffi::OsStr;

        let mut env = crate::test_support::TestHostEnv::new();
        env.set_env("TERM", OsStr::new("xterm-256color"));
        env.set_env("NO_COLOR", OsStr::new("1"));
        env.remove_env("CLICOLOR");
        env.remove_env("CLICOLOR_FORCE");

        let no_color_tty =
            WizardCapabilities::resolve(true, Some("xterm-256color"), Some("1"), None, None);
        let captured = WizardCapabilities::resolve(false, Some("xterm-256color"), None, None, None);
        let dumb = WizardCapabilities::resolve(true, Some("dumb"), None, None, Some("1"));

        // 状态行复用 ccr-core::ColorOutput::format_status
        assert_eq!(
            ColorOutput::format_status(OutputStatus::Step, STAGE_1_GIT_MESSAGE, true),
            "→ [1/4] 检查 Git 仓库"
        );
        assert_eq!(
            ColorOutput::format_status(OutputStatus::Step, STAGE_2_CONFIG_MESSAGE, true),
            "→ [2/4] 配置工作流与宿主平台"
        );
        assert_eq!(
            ColorOutput::format_status(
                OutputStatus::Step,
                &stage_3_engine_message(WorkflowEngine::MiniTrellis),
                true,
            ),
            "→ [3/4] 执行 mini-trellis 初始化"
        );
        assert_eq!(
            ColorOutput::format_status(OutputStatus::Step, STAGE_4_WORKSPACE_FILES_MESSAGE, true),
            "→ [4/4] 准备 ref/ 目录与 .gitignore 忽略规则"
        );
        assert_eq!(
            ColorOutput::format_status(OutputStatus::Step, STAGE_1_GIT_MESSAGE, false),
            "进度: [1/4] 检查 Git 仓库"
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
            config_confirmation_fields(&req),
            [
                ("工作流引擎", "mini-trellis".to_string()),
                (
                    "宿主平台",
                    "Claude Code, Codex, Cursor, Oh My Pi".to_string()
                ),
                ("开发者", "Ada Lovelace".to_string()),
            ]
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
        }

        assert_eq!(
            ColorOutput::format_status(
                OutputStatus::Success,
                "mini-trellis 工作流产物校验通过",
                false,
            ),
            "成功: mini-trellis 工作流产物校验通过"
        );
    }

    #[test]
    fn summary_table_ready_line_and_next_steps_reflect_outcomes_truncation_and_dumb_ascii() {
        use crate::commands::project::gitignore::PROJECT_IGNORE_RULES;
        use crate::commands::project::workflow::WorkflowPlatform;
        use std::ffi::OsStr;

        let mut env = crate::test_support::TestHostEnv::new();
        env.set_env("TERM", OsStr::new("xterm-256color"));
        env.set_env("NO_COLOR", OsStr::new("1"));
        env.remove_env("CLICOLOR");
        env.remove_env("CLICOLOR_FORCE");

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

        // 在父仓库子目录幂等重跑且处于 TERM=dumb 时，应使用纯 ASCII 边框且不含任何装饰性 Unicode
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

        // 直接使用 build_summary_table 验证 80 列不超宽、120 列单行完整显示、36 列触发 `...` 截断
        let mut table_80 = build_summary_table(&fresh_summary, no_color_tty);
        table_80.set_width(80);
        let rendered_80 = table_80.to_string();
        for line in rendered_80.lines() {
            let width = dialoguer::console::measure_text_width(line);
            assert!(
                width <= 80,
                "line width {width} > 80: {line}\n{rendered_80}"
            );
        }
        for rule in PROJECT_IGNORE_RULES {
            assert!(
                rendered_80.contains(rule),
                "missing {rule} in 80-column table:\n{rendered_80}"
            );
        }

        let mut table_120 = build_summary_table(&fresh_summary, no_color_tty);
        table_120.set_width(120);
        let rendered_120 = table_120.to_string();
        for line in rendered_120.lines() {
            let width = dialoguer::console::measure_text_width(line);
            assert!(width <= 120, "line width {width} > 120: {line}");
        }
        assert!(
            rendered_120.contains(".agents/、.claude/、.codex/、.cursor/、.omp/、.grok/、ref/"),
            "{rendered_120}"
        );

        let mut narrow = build_summary_table(&fresh_summary, no_color_tty);
        narrow.set_width(36);
        let narrow_rendered = narrow.to_string();
        for line in narrow_rendered.lines() {
            let width = dialoguer::console::measure_text_width(line);
            assert!(
                width <= 36,
                "line width {width} > 36: {line}\n{narrow_rendered}"
            );
        }
        assert!(narrow_rendered.contains("..."), "{narrow_rendered}");
        assert!(!narrow_rendered.contains('…'), "{narrow_rendered}");
        assert!(!narrow_rendered.contains("\u{1b}["));

        // 收尾就绪文案与引擎专属下一步引导
        assert_eq!(
            ColorOutput::format_status(
                OutputStatus::Success,
                &final_ready_message(WorkflowEngine::MiniTrellis),
                true,
            ),
            "✓ 项目工作流初始化完成（Git、mini-trellis、Agent 忽略规则和 ref/ 均已就绪）"
        );
        assert_eq!(
            ColorOutput::format_status(
                OutputStatus::Success,
                &final_ready_message(WorkflowEngine::Trellis),
                false,
            ),
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
