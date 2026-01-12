// 🔐 Codex Auth TUI 模块
// 提供 Codex 多账号管理的终端用户界面

mod app;
mod ui;

pub use app::CodexAuthApp;

use crate::core::error::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

use super::event::{Event, EventHandler};

/// 🚀 运行 Codex Auth TUI 应用
pub fn run_codex_auth_tui() -> Result<()> {
    // 🔧 设置终端
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    // 确保进入 TUI 时清空旧输出，避免残留内容覆盖
    terminal.clear()?;

    // 🎯 创建应用实例
    let app = CodexAuthApp::new()?;
    let event_handler = EventHandler::new(250);

    // 🎨 运行主循环
    let final_app = run_app(&mut terminal, app, event_handler)?;

    // 🧹 恢复终端
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // 📢 打印最后的操作结果
    if let Some((action, name, success, error)) = final_app.last_action {
        if success {
            println!("✅ {} 账号: {}", action, name);
        } else if let Some(err) = error {
            eprintln!("❌ {} 账号 {} 失败: {}", action, name, err);
        }
    }

    Ok(())
}

/// 🔄 主事件循环
fn run_app<B>(
    terminal: &mut Terminal<B>,
    mut app: CodexAuthApp,
    mut event_handler: EventHandler,
) -> Result<CodexAuthApp>
where
    B: ratatui::backend::Backend,
    B::Error: std::error::Error + Send + Sync + 'static,
{
    // 首次绘制
    draw_frame(terminal, &app)?;

    loop {
        match event_handler.poll_event()? {
            Event::Key(key) => {
                // ⌨️ 处理按键事件
                if app.handle_key(key)? {
                    // 用户请求退出
                    return Ok(app);
                }
                draw_frame(terminal, &app)?;
            }
            Event::Resize(_, _) => {
                // 窗口变更时清屏，避免残影
                terminal.clear()?;
                draw_frame(terminal, &app)?;
            }
            Event::Tick => {
                // ⏱️ 周期性刷新（可选）
            }
        }
    }
}

fn draw_frame<B>(terminal: &mut Terminal<B>, app: &CodexAuthApp) -> Result<()>
where
    B: ratatui::backend::Backend,
    B::Error: std::error::Error + Send + Sync + 'static,
{
    terminal
        .draw(|f| ui::draw(f, app))
        .map_err(|err| crate::core::error::CcrError::IoError(io::Error::other(err)))?;
    Ok(())
}
