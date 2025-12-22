// 🎨 ui 命令实现 - 启动 CCR UI Web 应用
// 提供图形化的配置管理界面

use crate::core::error::Result;
use crate::services::ui_service::UiService;

/// 🎨 启动 CCR UI
///
/// 根据环境自动选择启动方式:
/// - 开发环境: 使用源码启动 (just dev)
/// - 生产环境: 启动预构建版本 (未来支持)
///
/// # 参数
/// - `port`: 前端端口 (默认 3000)
/// - `backend_port`: 后端端口 (默认 38081)
pub fn ui_command(port: u16, backend_port: u16, auto_yes: bool) -> Result<()> {
    let ui_service = UiService::new()?;
    ui_service.start(port, backend_port, auto_yes)?;
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_service_creation() {
        // 测试服务创建
        let result = UiService::new();
        assert!(result.is_ok(), "UI 服务应该能正常创建");
    }
}
