// 🎨 ui 命令实现 - 启动 CCR UI Web 应用
// 提供图形化的配置管理界面

#![allow(clippy::unused_async)]

use crate::services::ui_service::UiService;
use ccr_core::core::error::{CcrError, Result};

/// 🎨 启动 CCR UI
///
/// 根据环境自动选择启动方式:
/// - 开发环境: 使用源码启动 (just dev)
/// - 生产环境: 启动预构建版本 (未来支持)
///
/// # 参数
/// - `port`: 前端端口 (默认 15173)
/// - `backend_port`: 后端端口 (默认 38081)
pub async fn ui_command(port: u16, backend_port: u16, auto_yes: bool) -> Result<()> {
    let ui_service = UiService::new()?;
    tokio::task::spawn_blocking(move || ui_service.start(port, backend_port, auto_yes))
        .await
        .map_err(|e| CcrError::UiError(format!("启动 UI 任务失败: {}", e)))??;
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
