// ⚡ TUI 事件处理
// 处理键盘输入和定时刷新事件

use crate::core::error::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent};
use std::time::Duration;

/// 🎯 事件类型
#[derive(Debug, Clone, Copy)]
pub enum Event {
    /// ⌨️ 键盘事件
    Key(KeyEvent),
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

    /// ⏭️ 获取下一个事件
    ///
    /// 阻塞直到有事件发生或超时
    pub fn next(&mut self) -> Result<Event> {
        if event::poll(self.tick_rate)? {
            match event::read()? {
                CrosstermEvent::Key(key) => Ok(Event::Key(key)),
                _ => Ok(Event::Tick),
            }
        } else {
            Ok(Event::Tick)
        }
    }
}
