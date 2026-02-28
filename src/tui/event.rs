// ⚡ TUI 事件处理
// 处理键盘输入和定时刷新事件

use crate::core::error::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, KeyEventKind, MouseEvent};
use std::time::Duration;

/// 🎯 事件类型
#[derive(Debug, Clone, Copy)]
pub enum Event {
    /// ⌨️ 键盘事件
    Key(KeyEvent),
    /// 🖱️ 鼠标事件
    Mouse(MouseEvent),
    /// 📐 窗口大小变化
    Resize,
    /// ⏱️ 定时刷新
    Tick,
}

/// 📡 事件处理器
pub struct EventHandler {
    /// ⏱️ 刷新间隔 (毫秒)
    tick_rate: Duration,
}

impl EventHandler {
    /// 🏗️ 创建新的事件处理器
    ///
    /// 参数:
    /// - tick_rate_ms: 刷新间隔 (毫秒)
    pub fn new(tick_rate_ms: u64) -> Self {
        Self {
            tick_rate: Duration::from_millis(tick_rate_ms),
        }
    }

    /// 获取下一个事件
    ///
    /// 阻塞直到有事件发生或超时
    ///
    /// 注意: 在 Windows 上，crossterm 会为每次按键发送 Press 和 Release 两个事件
    /// 必须过滤只处理 Press 事件，否则会导致按键被处理两次
    pub fn poll_event(&mut self) -> Result<Event> {
        if event::poll(self.tick_rate)? {
            match event::read()? {
                CrosstermEvent::Key(key) => {
                    // 🔑 关键修复: 只处理 Press 事件，忽略 Release 和 Repeat
                    // Windows 上每次按键会触发 Press + Release 两个事件
                    // 如果不过滤，Tab 切换会执行两次导致看起来没有效果
                    if key.kind == KeyEventKind::Press {
                        Ok(Event::Key(key))
                    } else {
                        Ok(Event::Tick)
                    }
                }
                CrosstermEvent::Resize(_, _) => Ok(Event::Resize),
                CrosstermEvent::Mouse(mouse) => Ok(Event::Mouse(mouse)),
                _ => Ok(Event::Tick),
            }
        } else {
            Ok(Event::Tick)
        }
    }
}
