// TUI module — terminal user interface
// Built on ratatui with unified runtime infrastructure

pub mod action;
mod app;
mod auth_refresh;
pub mod claude_auth;
pub mod codex_auth;
mod event;
pub mod i18n;
pub mod opencode_auth;
pub mod overlay;
mod pagination;
pub mod runtime;
mod selection;
pub mod theme;
pub mod toast;
mod ui;
pub mod usage;

pub use app::App;
pub use event::EventHandler;

use ccr_core::core::error::Result;
use runtime::{AsyncTaskExecutor, TerminalGuard, run_loop};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletedAction {
    Delete,
    Import,
    Rename,
    Save,
    Switch,
}

impl CompletedAction {
    fn success_label(self) -> &'static str {
        match self {
            Self::Delete => crate::tui_text!("Deleted", "已删除"),
            Self::Import => crate::tui_text!("Imported", "已导入"),
            Self::Rename => crate::tui_text!("Renamed", "已重命名"),
            Self::Save => crate::tui_text!("Saved", "已保存"),
            Self::Switch => crate::tui_text!("Switched to", "已切换到"),
        }
    }

    fn failure_label(self) -> &'static str {
        match self {
            Self::Delete => crate::tui_text!("delete", "删除"),
            Self::Import => crate::tui_text!("import", "导入"),
            Self::Rename => crate::tui_text!("rename", "重命名"),
            Self::Save => crate::tui_text!("save", "保存"),
            Self::Switch => crate::tui_text!("switch to", "切换"),
        }
    }
}

/// Print exit info for both profile and codex auth actions.
fn print_exit_info(app: &App) {
    // Profile switch result
    if let Some((platform, profile, success, error)) = &app.last_applied {
        if *success {
            println!(
                "{}",
                crate::tui_format!(
                    "[{}] Switched to profile: {}",
                    "[{}] 已切换到配置：{}",
                    platform,
                    profile
                )
            );
        } else if let Some(err) = error {
            eprintln!(
                "{}",
                crate::tui_format!(
                    "[{}] Failed to switch profile {}: {}",
                    "[{}] 切换配置 {} 失败：{}",
                    platform,
                    profile,
                    err
                )
            );
        }
    }

    // Claude auth action result
    if let Some((action, name, success, error)) = &app.last_claude_action {
        if *success {
            println!(
                "{}",
                crate::tui_format!(
                    "{} Claude official account: {}",
                    "{} Claude 官方账号：{}",
                    action.success_label(),
                    name
                )
            );
        } else if let Some(err) = error {
            eprintln!(
                "{}",
                crate::tui_format!(
                    "Failed to {} Claude official account {}: {}",
                    "Claude 官方账号 {1} {0}失败：{2}",
                    action.failure_label(),
                    name,
                    err
                )
            );
        }
    }

    // Codex auth action result
    if let Some((action, name, success, error)) = &app.last_codex_action {
        if *success {
            println!(
                "{}",
                crate::tui_format!("{} account: {}", "{}账号：{}", action.success_label(), name)
            );
        } else if let Some(err) = error {
            eprintln!(
                "{}",
                crate::tui_format!(
                    "Failed to {} account {}: {}",
                    "账号 {1} {0}失败：{2}",
                    action.failure_label(),
                    name,
                    err
                )
            );
        }
    }

    // OpenCode auth action result
    if let Some((action, name, success, error)) = &app.last_opencode_action {
        if *success {
            if *action == CompletedAction::Import {
                println!(
                    "{}",
                    crate::tui_format!(
                        "Imported OpenCode accounts: {}",
                        "已导入 OpenCode 账号：{} 个",
                        name
                    )
                );
            } else {
                println!(
                    "{}",
                    crate::tui_format!(
                        "{} OpenCode account: {}",
                        "{} OpenCode 账号：{}",
                        action.success_label(),
                        name
                    )
                );
            }
        } else if let Some(err) = error {
            if *action == CompletedAction::Import {
                eprintln!(
                    "{}",
                    crate::tui_format!(
                        "Failed to import Codex accounts into OpenCode: {}",
                        "Codex 账号导入 OpenCode 失败：{}",
                        err
                    )
                );
            } else {
                eprintln!(
                    "{}",
                    crate::tui_format!(
                        "Failed to {} OpenCode account {}: {}",
                        "OpenCode 账号 {1} {0}失败：{2}",
                        action.failure_label(),
                        name,
                        err
                    )
                );
            }
        }
    }
}

/// Run the main profile-switching TUI.
pub fn run_tui() -> Result<()> {
    i18n::initialize_from_config();
    let mut guard = TerminalGuard::new()?;
    let task_executor = AsyncTaskExecutor::from_current_or_test();
    let mut app = App::with_task_executor(task_executor)?;
    let mut events = EventHandler::new(250);

    run_loop(&mut guard, &mut app, &mut events)?;

    // Must drop guard BEFORE printing so terminal leaves alternate screen first
    drop(guard);
    print_exit_info(&app);

    Ok(())
}

/// Run the main TUI pre-selected to the Codex tab.
pub fn run_tui_with_codex_auth() -> Result<()> {
    i18n::initialize_from_config();
    let mut guard = TerminalGuard::new()?;
    let task_executor = AsyncTaskExecutor::from_current_or_test();
    let mut app = App::with_task_executor(task_executor)?.with_codex_tab();
    let mut events = EventHandler::new(250);

    run_loop(&mut guard, &mut app, &mut events)?;

    // Must drop guard BEFORE printing so terminal leaves alternate screen first
    drop(guard);
    print_exit_info(&app);

    Ok(())
}

/// Run the main TUI pre-selected to the Claude Auth tab.
pub fn run_tui_with_claude_auth() -> Result<()> {
    i18n::initialize_from_config();
    let mut guard = TerminalGuard::new()?;
    let task_executor = AsyncTaskExecutor::from_current_or_test();
    let mut app = App::with_task_executor(task_executor)?.with_claude_auth_tab();
    let mut events = EventHandler::new(250);

    run_loop(&mut guard, &mut app, &mut events)?;

    drop(guard);
    print_exit_info(&app);

    Ok(())
}

/// Run the main TUI pre-selected to the OpenCode Auth tab.
pub fn run_tui_with_opencode_auth() -> Result<()> {
    i18n::initialize_from_config();
    let mut guard = TerminalGuard::new()?;
    let task_executor = AsyncTaskExecutor::from_current_or_test();
    let mut app = App::with_task_executor(task_executor)?.with_opencode_auth_tab();
    let mut events = EventHandler::new(250);

    run_loop(&mut guard, &mut app, &mut events)?;

    drop(guard);
    print_exit_info(&app);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::CompletedAction;
    use ccr_cli::managers::TuiLanguage;

    #[test]
    fn completed_action_labels_follow_the_current_language() {
        crate::tui::i18n::set_language(TuiLanguage::English);
        assert_eq!(CompletedAction::Switch.success_label(), "Switched to");
        assert_eq!(CompletedAction::Delete.failure_label(), "delete");

        crate::tui::i18n::set_language(TuiLanguage::SimplifiedChinese);
        assert_eq!(CompletedAction::Switch.success_label(), "已切换到");
        assert_eq!(CompletedAction::Delete.failure_label(), "删除");

        crate::tui::i18n::set_language(TuiLanguage::English);
    }
}
