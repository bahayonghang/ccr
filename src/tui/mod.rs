// 🖥️ TUI 模块 - 终端用户界面
// 基于 ratatui 实现的双Tab配置选择器

mod app;
pub mod codex_auth;
mod event;
mod theme;
mod ui;

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
pub fn run_tui() -> Result<()> {
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
    let final_app = run_app(&mut terminal, app, event_handler)?;

    // 🧹 恢复终端
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // 📢 打印最后的切换结果
    if let Some((platform, profile, success, error)) = final_app.last_applied {
        if success {
            println!("✅ [{}] 已切换到配置: {}", platform, profile);
        } else if let Some(err) = error {
            eprintln!("❌ [{}] 切换配置 {} 失败: {}", platform, profile, err);
        }
    }

    Ok(())
}

/// 🔄 主事件循环
fn run_app<B>(
    terminal: &mut Terminal<B>,
    mut app: App,
    mut event_handler: EventHandler,
) -> Result<App>
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
            Event::Tick => {
                // ⏱️ 周期性刷新（可选）
            }
        }
    }
}

fn draw_frame<B>(terminal: &mut Terminal<B>, app: &App) -> Result<()>
where
    B: ratatui::backend::Backend,
    B::Error: std::error::Error + Send + Sync + 'static,
{
    terminal
        .draw(|f| ui::draw(f, app))
        .map_err(|err| crate::core::error::CcrError::IoError(io::Error::other(err)))?;
    Ok(())
}
