use super::WorkflowEngine;
use super::fingerprint::{ensure_not_mixed_workspace, validate_markers_for_target};
use super::render::WizardCapabilities;
use super::workflow::{WorkflowInitRequest, WorkflowPlatform};
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::logging::ColorOutput;
use dialoguer::console::{Style, style};
use dialoguer::theme::{ColorfulTheme, SimpleTheme, Theme};
use dialoguer::{Input, MultiSelect, Select};
use std::path::Path;
use std::process::Command;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WizardThemeKind {
    Colorful,
    Simple,
}

impl WizardThemeKind {
    pub(crate) fn for_capabilities(caps: WizardCapabilities) -> Self {
        if caps.style {
            Self::Colorful
        } else {
            Self::Simple
        }
    }

    fn into_theme(self) -> Box<dyn Theme> {
        match self {
            Self::Colorful => Box::new(build_colorful_theme()),
            Self::Simple => Box::new(SimpleTheme),
        }
    }
}

fn build_colorful_theme() -> ColorfulTheme {
    ColorfulTheme {
        prompt_prefix: style("?".to_string()).for_stderr().yellow().bold(),
        prompt_style: Style::new().for_stderr().bold(),
        prompt_suffix: style("›".to_string()).for_stderr().black().bright(),
        active_item_prefix: style("❯".to_string()).for_stderr().cyan().bold(),
        active_item_style: Style::new().for_stderr().cyan().bold(),
        checked_item_prefix: style("✓".to_string()).for_stderr().green().bold(),
        unchecked_item_prefix: style("·".to_string()).for_stderr().black().bright(),
        values_style: Style::new().for_stderr().green(),
        ..ColorfulTheme::default()
    }
}

const WORKFLOW_PROMPT_TTY: &str = "选择项目工作流引擎 (方向键移动 · 回车确认)";
const WORKFLOW_PROMPT_DUMB: &str = "选择项目工作流引擎 (方向键移动, 回车确认)";
const PLATFORM_PROMPT_TTY: &str = "选择要初始化的宿主平台 (空格勾选 · 回车确认)";
const PLATFORM_PROMPT_DUMB: &str = "选择要初始化的宿主平台 (空格勾选, 回车确认)";
const EMPTY_PLATFORM_WARNING: &str = "请至少选择一个平台";
const DEVELOPER_PROMPT: &str = "开发者用户名";
const EMPTY_DEVELOPER_WARNING: &str = "开发者用户名不能为空（用于工作流工作区目录）";
const MISSING_GIT_USER_MESSAGE: &str =
    "用户名阶段：需要设置 git config user.name，或去掉 -y 后手动输入开发者用户名";

pub(crate) trait ProjectWizardPrompter {
    fn is_dumb(&self) -> bool {
        false
    }
    fn select_workflow(&mut self, prompt: &str, items: &[&str], default: usize) -> Result<usize>;
    fn select_platforms(
        &mut self,
        prompt: &str,
        items: &[&str],
        defaults: &[bool],
    ) -> Result<Vec<usize>>;
    fn input_developer(&mut self, prompt: &str) -> Result<String>;
    fn warn(&mut self, message: &str);
}

struct DialoguerPrompter {
    theme: Box<dyn Theme>,
    stderr_caps: WizardCapabilities,
}

impl DialoguerPrompter {
    fn new(stderr_caps: WizardCapabilities) -> Self {
        let theme = WizardThemeKind::for_capabilities(stderr_caps).into_theme();
        Self { theme, stderr_caps }
    }
}

impl ProjectWizardPrompter for DialoguerPrompter {
    fn is_dumb(&self) -> bool {
        self.stderr_caps.is_dumb
    }

    fn select_workflow(&mut self, prompt: &str, items: &[&str], default: usize) -> Result<usize> {
        Select::with_theme(self.theme.as_ref())
            .with_prompt(prompt)
            .items(items)
            .default(default)
            .interact()
            .map_err(|error| CcrError::UiError(format!("工作流选择阶段：交互失败: {error}")))
    }

    fn select_platforms(
        &mut self,
        prompt: &str,
        items: &[&str],
        defaults: &[bool],
    ) -> Result<Vec<usize>> {
        // `defaults` 依赖已添加的选项列表，因此必须先调用 `items`。
        MultiSelect::with_theme(self.theme.as_ref())
            .with_prompt(prompt)
            .items(items)
            .defaults(defaults)
            .interact()
            .map_err(|error| CcrError::UiError(format!("平台选择阶段：交互失败: {error}")))
    }

    fn input_developer(&mut self, prompt: &str) -> Result<String> {
        Input::<String>::with_theme(self.theme.as_ref())
            .with_prompt(prompt)
            .allow_empty(true)
            .interact_text()
            .map_err(|error| CcrError::UiError(format!("用户名阶段：交互失败: {error}")))
    }

    fn warn(&mut self, message: &str) {
        ColorOutput::warning(message);
    }
}

pub(crate) fn resolve_workflow_request(
    auto_yes: bool,
    workflow: Option<WorkflowEngine>,
    git: &Path,
    root: &Path,
) -> Result<WorkflowInitRequest> {
    let stderr_caps = WizardCapabilities::for_stderr();
    let mut prompter = DialoguerPrompter::new(stderr_caps);
    resolve_workflow_request_with(auto_yes, workflow, git, root, &mut prompter)
}

pub(crate) fn resolve_workflow_request_with(
    auto_yes: bool,
    workflow: Option<WorkflowEngine>,
    git: &Path,
    root: &Path,
    prompter: &mut impl ProjectWizardPrompter,
) -> Result<WorkflowInitRequest> {
    // 双引擎混杂目录无论选哪套引擎都不可继续，前置立即阻断。
    let markers = ensure_not_mixed_workspace(root)?;

    if auto_yes {
        let engine = WorkflowEngine::resolve_non_interactive(workflow);
        let mode = validate_markers_for_target(&markers, engine)?;
        return Ok(WorkflowInitRequest {
            workflow: engine,
            mode,
            developer: read_git_user_name(git, root)?,
            platforms: engine.supported_platforms().to_vec(),
            auto_yes: true,
        });
    }

    let engine = match workflow {
        Some(explicit) => explicit,
        None => prompt_workflow_with(prompter)?,
    };
    let mode = validate_markers_for_target(&markers, engine)?;
    let platforms = prompt_platforms_with(engine, prompter)?;
    let developer = prompt_developer_with(prompter)?;

    Ok(WorkflowInitRequest {
        workflow: engine,
        mode,
        developer,
        platforms,
        auto_yes: false,
    })
}

fn prompt_workflow_with(prompter: &mut impl ProjectWizardPrompter) -> Result<WorkflowEngine> {
    let is_dumb = prompter.is_dumb();
    let labels =
        WorkflowEngine::INTERACTIVE_MENU_ORDER.map(|engine| engine.menu_label_for(is_dumb));
    let prompt = if is_dumb {
        WORKFLOW_PROMPT_DUMB
    } else {
        WORKFLOW_PROMPT_TTY
    };
    let index = prompter.select_workflow(prompt, &labels, 0)?;
    WorkflowEngine::INTERACTIVE_MENU_ORDER
        .get(index)
        .copied()
        .ok_or_else(|| {
            CcrError::ValidationError(format!("工作流选择阶段：无效的工作流下标 {index}"))
        })
}

fn prompt_platforms_with(
    engine: WorkflowEngine,
    prompter: &mut impl ProjectWizardPrompter,
) -> Result<Vec<WorkflowPlatform>> {
    let supported = engine.supported_platforms();
    let labels = supported
        .iter()
        .copied()
        .map(WorkflowPlatform::menu_label)
        .collect::<Vec<_>>();
    let defaults = vec![true; supported.len()];
    let prompt = if prompter.is_dumb() {
        PLATFORM_PROMPT_DUMB
    } else {
        PLATFORM_PROMPT_TTY
    };

    loop {
        let selected = prompter.select_platforms(prompt, &labels, &defaults)?;
        if selected.is_empty() {
            prompter.warn(EMPTY_PLATFORM_WARNING);
            continue;
        }
        return platforms_from_indices(engine, &selected);
    }
}

fn platforms_from_indices(
    engine: WorkflowEngine,
    indices: &[usize],
) -> Result<Vec<WorkflowPlatform>> {
    let supported = engine.supported_platforms();
    let mut platforms = Vec::with_capacity(indices.len());
    for index in indices {
        let platform = supported.get(*index).copied().ok_or_else(|| {
            CcrError::ValidationError(format!("平台选择阶段：无效的平台下标 {index}"))
        })?;
        if !platforms.contains(&platform) {
            platforms.push(platform);
        }
    }
    Ok(platforms)
}

fn prompt_developer_with(prompter: &mut impl ProjectWizardPrompter) -> Result<String> {
    loop {
        let raw = prompter.input_developer(DEVELOPER_PROMPT)?;
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
        prompter.warn(EMPTY_DEVELOPER_WARNING);
    }
}

fn read_git_user_name(git: &Path, root: &Path) -> Result<String> {
    let output = Command::new(git)
        .args(["config", "user.name"])
        .current_dir(root)
        .output()
        .map_err(|error| {
            CcrError::ExternalCommandError(format!(
                "用户名阶段：无法启动 git config user.name: {error}"
            ))
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    developer_name_from_git_config(output.status.success(), &stdout)
}

pub(crate) fn developer_name_from_git_config(command_ok: bool, stdout: &str) -> Result<String> {
    let name = stdout.trim();
    if !command_ok || name.is_empty() {
        return Err(CcrError::ValidationError(MISSING_GIT_USER_MESSAGE.into()));
    }
    Ok(name.to_string())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::fs;
    use tempfile::tempdir;

    #[derive(Default)]
    struct ScriptedPrompter {
        workflow_choices: VecDeque<usize>,
        platform_choices: VecDeque<Vec<usize>>,
        developer_inputs: VecDeque<String>,
        observed_workflow_prompts: Vec<String>,
        observed_workflow_items: Vec<Vec<String>>,
        observed_workflow_defaults: Vec<usize>,
        observed_platform_prompts: Vec<String>,
        observed_platform_items: Vec<Vec<String>>,
        observed_platform_defaults: Vec<Vec<bool>>,
        observed_developer_prompts: Vec<String>,
        warnings: Vec<String>,
    }

    impl ProjectWizardPrompter for ScriptedPrompter {
        fn select_workflow(
            &mut self,
            prompt: &str,
            items: &[&str],
            default: usize,
        ) -> Result<usize> {
            self.observed_workflow_prompts.push(prompt.to_string());
            self.observed_workflow_items
                .push(items.iter().map(|s| (*s).to_string()).collect());
            self.observed_workflow_defaults.push(default);
            Ok(self.workflow_choices.pop_front().expect("workflow choice"))
        }

        fn select_platforms(
            &mut self,
            prompt: &str,
            items: &[&str],
            defaults: &[bool],
        ) -> Result<Vec<usize>> {
            self.observed_platform_prompts.push(prompt.to_string());
            self.observed_platform_items
                .push(items.iter().map(|s| (*s).to_string()).collect());
            self.observed_platform_defaults.push(defaults.to_vec());
            Ok(self.platform_choices.pop_front().expect("platform choice"))
        }

        fn input_developer(&mut self, prompt: &str) -> Result<String> {
            self.observed_developer_prompts.push(prompt.to_string());
            Ok(self.developer_inputs.pop_front().expect("developer input"))
        }

        fn warn(&mut self, message: &str) {
            self.warnings.push(message.to_string());
        }
    }

    #[cfg(windows)]
    fn write_fake_git(dir: &Path, user_name: Option<&str>) -> std::path::PathBuf {
        let path = dir.join("fake-git.cmd");
        let script = match user_name {
            Some(name) => format!("@echo off\r\necho {name}\r\nexit /b 0\r\n"),
            None => "@echo off\r\nexit /b 1\r\n".to_string(),
        };
        fs::write(&path, script).unwrap();
        path
    }

    #[cfg(unix)]
    fn write_fake_git(dir: &Path, user_name: Option<&str>) -> std::path::PathBuf {
        use std::os::unix::fs::PermissionsExt;

        let path = dir.join("fake-git");
        let script = match user_name {
            Some(name) => format!("#!/bin/sh\nprintf '%s\\n' '{name}'\nexit 0\n"),
            None => "#!/bin/sh\nexit 1\n".to_string(),
        };
        fs::write(&path, script).unwrap();
        let mut perms = fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).unwrap();
        path
    }

    #[test]
    fn git_user_name_trims_surrounding_whitespace() {
        assert_eq!(
            developer_name_from_git_config(true, "  Ada Lovelace \r\n").unwrap(),
            "Ada Lovelace"
        );
    }

    #[test]
    fn git_user_name_rejects_failed_or_blank_output() {
        for (command_ok, stdout) in [(false, "Ada\n"), (true, " \n"), (true, "")] {
            let error = developer_name_from_git_config(command_ok, stdout).unwrap_err();
            let message = error.to_string();
            assert!(matches!(error, CcrError::ValidationError(_)));
            assert_eq!(error.exit_code(), 90);
            assert!(message.contains("git config user.name"), "{message}");
            assert!(message.contains("-y"), "{message}");
        }
    }

    #[test]
    fn interactive_without_workflow_flag_defaults_first_choice_to_mini_trellis_without_prefilling_git_user()
     {
        let temp = tempdir().unwrap();
        let git = write_fake_git(temp.path(), Some("  Ada Lovelace  "));
        let mut prompter = ScriptedPrompter {
            workflow_choices: VecDeque::from([0]),
            platform_choices: VecDeque::from([vec![0, 1, 2, 3]]),
            // 即使 git config user.name 存在，也不应预填，直接回车必须提示告警并要求手动输入
            developer_inputs: VecDeque::from([String::new(), "  lyh  ".to_string()]),
            ..Default::default()
        };

        let req =
            resolve_workflow_request_with(false, None, &git, temp.path(), &mut prompter).unwrap();

        assert_eq!(prompter.observed_workflow_defaults, vec![0]);
        assert_eq!(prompter.observed_workflow_items.len(), 1);
        assert!(prompter.observed_workflow_items[0][0].starts_with("mini-trellis"));
        assert!(prompter.observed_workflow_items[0][1].starts_with("trellis"));

        // mini-trellis 宿主平台菜单默认全选 4 项带开关标签的平台，不含 Grok
        assert!(prompter.observed_workflow_prompts[0].contains("方向键移动"));
        assert!(prompter.observed_workflow_prompts[0].contains("回车确认"));
        assert!(prompter.observed_platform_prompts[0].contains("空格勾选"));
        assert!(prompter.observed_platform_prompts[0].contains("回车确认"));
        assert_eq!(
            prompter.observed_platform_items,
            vec![vec![
                "Claude Code (--claude)",
                "Codex (--codex)",
                "Cursor (--cursor)",
                "Oh My Pi (--omp)",
            ]]
        );
        assert_eq!(
            prompter.observed_platform_defaults,
            vec![vec![true, true, true, true]]
        );
        assert_eq!(
            prompter.observed_developer_prompts,
            vec![DEVELOPER_PROMPT, DEVELOPER_PROMPT]
        );
        assert_eq!(prompter.warnings, vec![EMPTY_DEVELOPER_WARNING]);

        assert_eq!(req.workflow, WorkflowEngine::MiniTrellis);
        assert_eq!(req.developer, "lyh");
        assert_eq!(
            req.platforms,
            vec![
                WorkflowPlatform::Claude,
                WorkflowPlatform::Codex,
                WorkflowPlatform::Cursor,
                WorkflowPlatform::Omp,
            ]
        );
        assert!(!req.auto_yes);
    }

    #[test]
    fn auto_yes_reads_git_user_name_without_prompting() {
        let temp = tempdir().unwrap();
        let git = write_fake_git(temp.path(), Some("  Ada Lovelace  "));
        let mut prompter = ScriptedPrompter::default();

        let req = resolve_workflow_request_with(
            true,
            Some(WorkflowEngine::MiniTrellis),
            &git,
            temp.path(),
            &mut prompter,
        )
        .unwrap();

        assert_eq!(req.workflow, WorkflowEngine::MiniTrellis);
        assert_eq!(req.developer, "Ada Lovelace");
        assert_eq!(
            req.platforms,
            WorkflowPlatform::MINI_TRELLIS_PLATFORMS.to_vec()
        );
        assert!(req.auto_yes);
        assert!(prompter.observed_workflow_prompts.is_empty());
        assert!(prompter.observed_platform_prompts.is_empty());
        assert!(prompter.observed_developer_prompts.is_empty());
    }

    #[test]
    fn interactive_reprompts_on_empty_platforms_and_enforces_non_empty_developer() {
        let temp = tempdir().unwrap();
        let git = write_fake_git(temp.path(), None);
        let mut prompter = ScriptedPrompter {
            platform_choices: VecDeque::from([vec![], vec![4, 0]]),
            developer_inputs: VecDeque::from([
                String::new(),
                "   ".to_string(),
                "  Grace Hopper  ".to_string(),
            ]),
            ..Default::default()
        };

        // 显式传入 --workflow trellis 时跳过引擎单选菜单
        let req = resolve_workflow_request_with(
            false,
            Some(WorkflowEngine::Trellis),
            &git,
            temp.path(),
            &mut prompter,
        )
        .unwrap();

        assert!(prompter.observed_workflow_items.is_empty());
        assert_eq!(prompter.observed_platform_items.len(), 2);
        assert_eq!(
            prompter.observed_platform_items[0],
            vec![
                "Claude Code (--claude)",
                "Codex (--codex)",
                "Cursor (--cursor)",
                "Oh My Pi (--omp)",
                "Grok (--grok)",
            ]
        );
        assert_eq!(
            prompter.observed_developer_prompts,
            vec![DEVELOPER_PROMPT, DEVELOPER_PROMPT, DEVELOPER_PROMPT]
        );
        // 1 次空平台选择告警 + 2 次空白用户名告警
        assert_eq!(
            prompter.warnings.len(),
            3,
            "warnings: {:?}",
            prompter.warnings
        );

        assert_eq!(req.workflow, WorkflowEngine::Trellis);
        assert_eq!(req.developer, "Grace Hopper");
        assert_eq!(
            req.platforms,
            vec![WorkflowPlatform::Grok, WorkflowPlatform::Claude]
        );
    }

    #[test]
    fn interactive_conflict_stops_right_after_workflow_selection_before_platform_prompt() {
        let temp = tempdir().unwrap();
        fs::create_dir_all(temp.path().join(".trellis")).unwrap();
        fs::write(temp.path().join(".trellis/workflow.md"), "trellis").unwrap();
        let git = write_fake_git(temp.path(), Some("Ada"));
        let mut prompter = ScriptedPrompter {
            // 在已有 trellis 工作区中选择下标 0（mini-trellis）
            workflow_choices: VecDeque::from([0]),
            ..Default::default()
        };

        let err = resolve_workflow_request_with(false, None, &git, temp.path(), &mut prompter)
            .unwrap_err();

        assert_eq!(err.exit_code(), 90);
        assert!(err.to_string().contains("mini-trellis migrate"));
        // 冲突应立即终止，不进入后续平台复选与用户名输入提示
        assert!(prompter.observed_platform_items.is_empty());
        assert!(prompter.observed_developer_prompts.is_empty());
    }

    #[test]
    fn wizard_theme_uses_colorful_only_on_styled_non_dumb_tty_and_falls_back_to_simple() {
        let styled_tty =
            WizardCapabilities::resolve(true, Some("xterm-256color"), None, None, None);
        let no_color_tty =
            WizardCapabilities::resolve(true, Some("xterm-256color"), Some("1"), None, None);
        let captured =
            WizardCapabilities::resolve(false, Some("xterm-256color"), None, None, Some("1"));
        let dumb = WizardCapabilities::resolve(true, Some("dumb"), None, None, Some("1"));

        assert_eq!(
            WizardThemeKind::for_capabilities(styled_tty),
            WizardThemeKind::Colorful
        );
        for degraded in [no_color_tty, captured, dumb] {
            assert_eq!(
                WizardThemeKind::for_capabilities(degraded),
                WizardThemeKind::Simple
            );
        }

        let simple_theme = WizardThemeKind::Simple.into_theme();
        let mut rendered = String::new();
        simple_theme
            .format_multi_select_prompt_item(&mut rendered, "Claude Code (--claude)", true, true)
            .unwrap();
        assert!(!rendered.contains("\u{1b}["), "{rendered:?}");
        for forbidden in ['→', '✓', '⚠', '═', '┌', '─', '│', '❯'] {
            assert!(!rendered.contains(forbidden), "{rendered:?}");
        }
    }

    #[test]
    fn stderr_tty_keeps_colorful_theme_when_stdout_is_redirected_or_no_color_is_empty() {
        use std::ffi::OsStr;

        let mut env = crate::test_support::TestHostEnv::new();
        env.set_env("TERM", OsStr::new("xterm-256color"));
        env.remove_env("CLICOLOR");
        env.remove_env("CLICOLOR_FORCE");

        // 仅重定向 stdout 但 stderr 仍是交互 TTY 时，交互菜单应保留彩色主题
        env.remove_env("NO_COLOR");
        let redirected_stdout = WizardCapabilities::from_env(false);
        let tty_stderr = WizardCapabilities::from_env(true);
        assert!(!redirected_stdout.style);
        assert_eq!(
            WizardThemeKind::for_capabilities(tty_stderr),
            WizardThemeKind::Colorful
        );

        // 空值 NO_COLOR="" 对齐 colored 3.x 语义，不禁用 TTY 彩色主题
        env.set_env("NO_COLOR", OsStr::new(""));
        let tty_stderr_empty_no_color = WizardCapabilities::from_env(true);
        assert_eq!(
            WizardThemeKind::for_capabilities(tty_stderr_empty_no_color),
            WizardThemeKind::Colorful
        );

        env.set_env("NO_COLOR", OsStr::new("1"));
        let tty_stderr_disabled = WizardCapabilities::from_env(true);
        assert_eq!(
            WizardThemeKind::for_capabilities(tty_stderr_disabled),
            WizardThemeKind::Simple
        );
    }
}
