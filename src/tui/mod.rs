// 🖥️ TUI 模块 - 终端用户界面
// 基于 ratatui 实现的交互式配置管理界面

mod app;
mod event;
mod tabs;
mod ui;
mod widgets;

pub use app::App;
pub use event::{Event, EventHandler};

use crate::core::error::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

/// 🚀 运行 TUI 应用
///
/// 参数:
/// - auto_yes: 是否启动时启用自动确认模式（运行时可通过Y键切换）
pub fn run_tui(_auto_yes: bool) -> Result<()> {
    // 🔧 设置终端
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 🎯 创建应用实例
    let app = App::new()?;
    let event_handler = EventHandler::new(250);

    // 🎨 运行主循环
    let res = run_app(&mut terminal, app, event_handler);

    // 🧹 恢复终端
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    res
}

/// 🔄 主事件循环
fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    mut event_handler: EventHandler,
) -> Result<()> {
    loop {
        // 📉 递减消息帧计数器
        app.tick_message();

        // 🎨 渲染UI
        terminal.draw(|f| ui::draw(f, &mut app))?;

        // ⌨️ 处理事件
        if let Event::Key(key) = event_handler.next()? {
            if app.handle_key(key)? {
                // 用户请求退出
                return Ok(());
            }
        }
    }
}
