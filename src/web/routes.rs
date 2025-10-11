// 🚦 路由定义
// 定义 URL 路由规则(未来扩展)

#![allow(dead_code)]

/// 路由枚举
///
/// 表示所有可能的 API 路由
#[derive(Debug, Clone)]
pub enum Route {
    /// GET / - 主页
    Home,

    /// GET /api/configs - 列出所有配置
    ListConfigs,

    /// POST /api/switch - 切换配置
    SwitchConfig,

    /// POST /api/config - 添加配置
    AddConfig,

    /// PUT /api/config/:name - 更新配置
    UpdateConfig(String),

    /// DELETE /api/config/:name - 删除配置
    DeleteConfig(String),

    /// GET /api/history - 获取历史记录
    GetHistory,

    /// POST /api/validate - 验证配置
    Validate,

    /// POST /api/clean - 清理备份
    Clean,

    /// GET /api/settings - 获取设置
    GetSettings,

    /// GET /api/settings/backups - 获取备份列表
    GetSettingsBackups,

    /// POST /api/settings/restore - 恢复设置
    RestoreSettings,

    /// POST /api/export - 导出配置
    ExportConfig,

    /// POST /api/import - 导入配置
    ImportConfig,

    /// 404 - 未找到
    NotFound,
}

impl Route {
    /// 从 HTTP 方法和路径匹配路由
    pub fn from_request(method: &str, path: &str) -> Self {
        match (method, path) {
            ("GET", "/") => Route::Home,
            ("GET", "/api/configs") => Route::ListConfigs,
            ("POST", "/api/switch") => Route::SwitchConfig,
            ("POST", "/api/config") => Route::AddConfig,
            ("PUT", path) if path.starts_with("/api/config/") => {
                let name = path.trim_start_matches("/api/config/");
                Route::UpdateConfig(name.to_string())
            }
            ("DELETE", path) if path.starts_with("/api/config/") => {
                let name = path.trim_start_matches("/api/config/");
                Route::DeleteConfig(name.to_string())
            }
            ("GET", "/api/history") => Route::GetHistory,
            ("POST", "/api/validate") => Route::Validate,
            ("POST", "/api/clean") => Route::Clean,
            ("GET", "/api/settings") => Route::GetSettings,
            ("GET", "/api/settings/backups") => Route::GetSettingsBackups,
            ("POST", "/api/settings/restore") => Route::RestoreSettings,
            ("POST", "/api/export") => Route::ExportConfig,
            ("POST", "/api/import") => Route::ImportConfig,
            _ => Route::NotFound,
        }
    }
}
