// 🖥️ TUI 模块 - 终端用户界面
// 基于 ratatui 实现的交互式配置管理界面

mod app;
mod event;
mod tabs;
mod theme;
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
pub fn run_tui(auto_yes: bool) -> Result<()> {
    // 🔧 设置终端
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 🎯 创建应用实例
    let mut app = App::new()?;
    // 根据命令行参数设置会话级自动确认模式（不持久化）
    if auto_yes {
        app.auto_confirm_mode = true;
    }
    let event_handler = EventHandler::new(250);

    // 🔄 首次渲染前刷新必要缓存，避免在渲染阶段进行 I/O
    // 将读取配置与历史的磁盘操作移出渲染闭包，提升绘制性能与流畅度
    if let Err(e) = app.refresh_caches() {
        // 缓存刷新失败不影响 TUI 启动，但在状态栏给出提示
        app.set_status(format!("初始化数据加载失败: {}", e), true);
    }

    // 🎨 运行主循环
    // 首帧绘制
    terminal.draw(|f| ui::draw(f, &mut app))?;

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
    // 📈 增量渲染策略：仅在事件或需要刷新时绘制
    // - Key 事件：总是重绘（状态或焦点可能变化）
    // - Tick 事件：仅在需要动画/消息帧计数变化时重绘
    let mut should_redraw = false;
    loop {
        match event_handler.poll_event()? {
            Event::Key(key) => {
                // ⌨️ 处理按键事件
                if app.handle_key(key)? {
                    // 用户请求退出
                    return Ok(());
                }
                should_redraw = true;
            }
            Event::Tick => {
                // ⏱️ 处理周期性刷新（状态消息帧计数、微动画）
                app.tick();
                if app.should_redraw_on_tick() {
                    should_redraw = true;
                }
            }
        }

        if should_redraw {
            terminal.draw(|f| ui::draw(f, &mut app))?;
            should_redraw = false;
        }
    }
}
