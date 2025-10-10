// CCR Web 服务器模块
// 提供配置管理的 Web 界面和 RESTful API

use crate::config::{ConfigManager, ConfigSection};
use crate::error::{CcrError, Result};
use crate::history::HistoryManager;
use crate::logging::ColorOutput;
use crate::settings::SettingsManager;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tiny_http::{Header, Method, Request, Response, Server, StatusCode};

/// API 响应结构
#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    message: Option<String>,
}

impl<T> ApiResponse<T> {
    fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
        }
    }

    fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            message: Some(message),
        }
    }
}

/// 配置列表响应
#[derive(Debug, Serialize, Deserialize)]
struct ConfigListResponse {
    current_config: String,
    default_config: String,
    configs: Vec<ConfigItem>,
}

/// 配置项
#[derive(Debug, Serialize, Deserialize)]
struct ConfigItem {
    name: String,
    description: String,
    base_url: String,
    auth_token: String,
    model: Option<String>,
    small_fast_model: Option<String>,
    is_current: bool,
    is_default: bool,
}

/// 切换配置请求
#[derive(Debug, Serialize, Deserialize)]
struct SwitchRequest {
    config_name: String,
}

/// 清理备份请求
#[derive(Debug, Serialize, Deserialize)]
struct CleanRequest {
    #[serde(default = "default_days")]
    days: u64,
    #[serde(default)]
    dry_run: bool,
}

fn default_days() -> u64 {
    7
}

/// 清理备份响应
#[derive(Debug, Serialize, Deserialize)]
struct CleanResponse {
    deleted_count: usize,
    skipped_count: usize,
    total_size_mb: f64,
    dry_run: bool,
}

/// 更新配置请求
#[derive(Debug, Serialize, Deserialize)]
struct UpdateConfigRequest {
    name: String,
    description: Option<String>,
    base_url: String,
    auth_token: String,
    model: Option<String>,
    small_fast_model: Option<String>,
}

/// 历史记录响应
#[derive(Debug, Serialize, Deserialize)]
struct HistoryResponse {
    entries: Vec<HistoryEntryJson>,
    total: usize,
}

/// 历史记录条目 JSON 格式
#[derive(Debug, Serialize, Deserialize)]
struct HistoryEntryJson {
    id: String,
    timestamp: String,
    operation: String,
    actor: String,
    from_config: Option<String>,
    to_config: Option<String>,
    changes: Vec<EnvChangeJson>,
}

/// 环境变量更改 JSON 格式
#[derive(Debug, Serialize, Deserialize)]
struct EnvChangeJson {
    key: String,
    old_value: Option<String>,
    new_value: Option<String>,
}

/// Settings 响应
#[derive(Debug, Serialize, Deserialize)]
struct SettingsResponse {
    settings: serde_json::Value,
}

/// Settings 备份响应
#[derive(Debug, Serialize, Deserialize)]
struct SettingsBackupsResponse {
    backups: Vec<BackupItem>,
}

/// 备份项
#[derive(Debug, Serialize, Deserialize)]
struct BackupItem {
    filename: String,
    path: String,
    created_at: String,
    size_bytes: u64,
}

/// 恢复 Settings 请求
#[derive(Debug, Serialize, Deserialize)]
struct RestoreSettingsRequest {
    backup_path: String,
}

/// Web 服务器
pub struct WebServer {
    config_manager: Arc<ConfigManager>,
    settings_manager: Arc<SettingsManager>,
    history_manager: Arc<HistoryManager>,
    port: u16,
}

impl WebServer {
    /// 创建新的 Web 服务器
    pub fn new(port: u16) -> Result<Self> {
        let config_manager = Arc::new(ConfigManager::default()?);
        let settings_manager = Arc::new(SettingsManager::default()?);
        let history_manager = Arc::new(HistoryManager::default()?);

        Ok(Self {
            config_manager,
            settings_manager,
            history_manager,
            port,
        })
    }

    /// 启动服务器
    pub fn start(&self) -> Result<()> {
        let addr = format!("0.0.0.0:{}", self.port);
        let server = Server::http(&addr).map_err(|e| {
            CcrError::ConfigError(format!("无法启动 HTTP 服务器: {}", e))
        })?;

        ColorOutput::success(&format!("CCR Web 服务器已启动"));
        ColorOutput::info(&format!("地址: http://localhost:{}", self.port));
        ColorOutput::info("按 Ctrl+C 停止服务器");
        println!();

        // 尝试自动打开浏览器
        if let Err(e) = open::that(format!("http://localhost:{}", self.port)) {
            ColorOutput::warning(&format!("无法自动打开浏览器: {}", e));
            ColorOutput::info(&format!("请手动访问 http://localhost:{}", self.port));
        }

        // 处理请求
        for request in server.incoming_requests() {
            if let Err(e) = self.handle_request(request) {
                log::error!("处理请求失败: {}", e);
            }
        }

        Ok(())
    }

    /// 处理 HTTP 请求
    fn handle_request(&self, mut request: Request) -> Result<()> {
        let url = request.url().to_string();
        let method = request.method().clone();

        log::debug!("{} {}", method, url);

        // 路由处理
        let response = match (method, url.as_str()) {
            // 静态文件
            (Method::Get, "/") => self.serve_html(),

            // API 路由 - 配置管理
            (Method::Get, "/api/configs") => self.handle_list_configs(),
            (Method::Post, "/api/switch") => self.handle_switch_config(&mut request),
            (Method::Post, "/api/config") => self.handle_add_config(&mut request),
            (Method::Put, path) if path.starts_with("/api/config/") => {
                let name = path.trim_start_matches("/api/config/");
                self.handle_update_config(name, &mut request)
            }
            (Method::Delete, path) if path.starts_with("/api/config/") => {
                let name = path.trim_start_matches("/api/config/");
                self.handle_delete_config(name)
            }
            (Method::Get, "/api/history") => self.handle_get_history(&request),
            (Method::Post, "/api/validate") => self.handle_validate(),
            (Method::Post, "/api/clean") => self.handle_clean(&mut request),

            // API 路由 - Settings 管理
            (Method::Get, "/api/settings") => self.handle_get_settings(),
            (Method::Get, "/api/settings/backups") => self.handle_get_settings_backups(),
            (Method::Post, "/api/settings/restore") => self.handle_restore_settings(&mut request),

            // 404
            _ => self.serve_404(),
        };

        request.respond(response).map_err(|e| {
            CcrError::ConfigError(format!("发送响应失败: {}", e))
        })?;

        Ok(())
    }

    /// 提供 HTML 页面
    fn serve_html(&self) -> Response<std::io::Cursor<Vec<u8>>> {
        let html = include_str!("../web/index.html");
        let content = html.as_bytes().to_vec();

        Response::from_data(content)
            .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap())
            .with_status_code(StatusCode(200))
    }

    /// 提供 404 页面
    fn serve_404(&self) -> Response<std::io::Cursor<Vec<u8>>> {
        let html = r#"
<!DOCTYPE html>
<html>
<head><title>404 Not Found</title></head>
<body><h1>404 Not Found</h1></body>
</html>
        "#;

        Response::from_data(html.as_bytes().to_vec())
            .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap())
            .with_status_code(StatusCode(404))
    }

    /// 处理列出配置
    fn handle_list_configs(&self) -> Response<std::io::Cursor<Vec<u8>>> {
        match self.list_configs_impl() {
            Ok(data) => self.json_response(ApiResponse::success(data), 200),
            Err(e) => self.json_response(
                ApiResponse::<()>::error(e.user_message()),
                500,
            ),
        }
    }

    fn list_configs_impl(&self) -> Result<ConfigListResponse> {
        let config = self.config_manager.load()?;

        let configs: Vec<ConfigItem> = config
            .list_sections()
            .into_iter()
            .filter_map(|name| {
                config.get_section(&name).ok().map(|section| ConfigItem {
                    name: name.clone(),
                    description: section.display_description(),
                    base_url: section.base_url.clone().unwrap_or_default(),
                    auth_token: ColorOutput::mask_sensitive(
                        &section.auth_token.clone().unwrap_or_default()
                    ),
                    model: section.model.clone(),
                    small_fast_model: section.small_fast_model.clone(),
                    is_current: name == config.current_config,
                    is_default: name == config.default_config,
                })
            })
            .collect();

        Ok(ConfigListResponse {
            current_config: config.current_config.clone(),
            default_config: config.default_config.clone(),
            configs,
        })
    }

    /// 处理切换配置
    fn handle_switch_config(&self, request: &mut Request) -> Response<std::io::Cursor<Vec<u8>>> {
        match self.switch_config_impl(request) {
            Ok(_) => self.json_response(
                ApiResponse::success("配置切换成功"),
                200,
            ),
            Err(e) => self.json_response(
                ApiResponse::<()>::error(e.user_message()),
                500,
            ),
        }
    }

    fn switch_config_impl(&self, request: &mut Request) -> Result<()> {
        let mut body = String::new();
        request.as_reader().read_to_string(&mut body).map_err(|e| {
            CcrError::ConfigError(format!("读取请求体失败: {}", e))
        })?;

        let req: SwitchRequest = serde_json::from_str(&body).map_err(|e| {
            CcrError::ConfigError(format!("解析请求失败: {}", e))
        })?;

        // 执行切换（使用现有的 switch 命令逻辑）
        crate::commands::switch_command(&req.config_name)?;

        Ok(())
    }

    /// 处理添加配置
    fn handle_add_config(&self, request: &mut Request) -> Response<std::io::Cursor<Vec<u8>>> {
        match self.add_config_impl(request) {
            Ok(_) => self.json_response(
                ApiResponse::success("配置添加成功"),
                200,
            ),
            Err(e) => self.json_response(
                ApiResponse::<()>::error(e.user_message()),
                500,
            ),
        }
    }

    fn add_config_impl(&self, request: &mut Request) -> Result<()> {
        let mut body = String::new();
        request.as_reader().read_to_string(&mut body).map_err(|e| {
            CcrError::ConfigError(format!("读取请求体失败: {}", e))
        })?;

        let req: UpdateConfigRequest = serde_json::from_str(&body).map_err(|e| {
            CcrError::ConfigError(format!("解析请求失败: {}", e))
        })?;

        let mut config = self.config_manager.load()?;

        let section = ConfigSection {
            description: req.description,
            base_url: Some(req.base_url),
            auth_token: Some(req.auth_token),
            model: req.model,
            small_fast_model: req.small_fast_model,
        };

        section.validate()?;
        config.set_section(req.name, section);
        self.config_manager.save(&config)?;

        Ok(())
    }

    /// 处理更新配置
    fn handle_update_config(&self, name: &str, request: &mut Request) -> Response<std::io::Cursor<Vec<u8>>> {
        match self.update_config_impl(name, request) {
            Ok(_) => self.json_response(
                ApiResponse::success("配置更新成功"),
                200,
            ),
            Err(e) => self.json_response(
                ApiResponse::<()>::error(e.user_message()),
                500,
            ),
        }
    }

    fn update_config_impl(&self, old_name: &str, request: &mut Request) -> Result<()> {
        let mut body = String::new();
        request.as_reader().read_to_string(&mut body).map_err(|e| {
            CcrError::ConfigError(format!("读取请求体失败: {}", e))
        })?;

        let req: UpdateConfigRequest = serde_json::from_str(&body).map_err(|e| {
            CcrError::ConfigError(format!("解析请求失败: {}", e))
        })?;

        let mut config = self.config_manager.load()?;

        // 如果名称变更，删除旧配置
        if old_name != req.name {
            config.remove_section(old_name)?;

            // 更新引用
            if config.current_config == old_name {
                config.current_config = req.name.clone();
            }
            if config.default_config == old_name {
                config.default_config = req.name.clone();
            }
        }

        let section = ConfigSection {
            description: req.description,
            base_url: Some(req.base_url),
            auth_token: Some(req.auth_token),
            model: req.model,
            small_fast_model: req.small_fast_model,
        };

        section.validate()?;
        config.set_section(req.name, section);
        self.config_manager.save(&config)?;

        Ok(())
    }

    /// 处理删除配置
    fn handle_delete_config(&self, name: &str) -> Response<std::io::Cursor<Vec<u8>>> {
        match self.delete_config_impl(name) {
            Ok(_) => self.json_response(
                ApiResponse::success("配置删除成功"),
                200,
            ),
            Err(e) => self.json_response(
                ApiResponse::<()>::error(e.user_message()),
                500,
            ),
        }
    }

    fn delete_config_impl(&self, name: &str) -> Result<()> {
        let mut config = self.config_manager.load()?;

        // 不允许删除当前或默认配置
        if name == config.current_config {
            return Err(CcrError::ValidationError("不能删除当前配置".into()));
        }
        if name == config.default_config {
            return Err(CcrError::ValidationError("不能删除默认配置".into()));
        }

        config.remove_section(name)?;
        self.config_manager.save(&config)?;

        Ok(())
    }

    /// 处理获取历史记录
    fn handle_get_history(&self, request: &Request) -> Response<std::io::Cursor<Vec<u8>>> {
        match self.get_history_impl(request) {
            Ok(data) => self.json_response(ApiResponse::success(data), 200),
            Err(e) => self.json_response(
                ApiResponse::<()>::error(e.user_message()),
                500,
            ),
        }
    }

    fn get_history_impl(&self, _request: &Request) -> Result<HistoryResponse> {
        let entries = self.history_manager.load()?;
        let json_entries: Vec<HistoryEntryJson> = entries
            .iter()
            .take(50) // 限制返回最近 50 条
            .map(|entry| HistoryEntryJson {
                id: entry.id.clone(),
                timestamp: entry.timestamp.to_rfc3339(),
                operation: entry.operation.as_str().to_string(),
                actor: entry.actor.clone(),
                from_config: entry.details.from_config.clone(),
                to_config: entry.details.to_config.clone(),
                changes: entry
                    .env_changes
                    .iter()
                    .map(|change| EnvChangeJson {
                        key: change.var_name.clone(),
                        old_value: change.old_value.as_ref().map(|v| ColorOutput::mask_sensitive(v)),
                        new_value: change.new_value.as_ref().map(|v| ColorOutput::mask_sensitive(v)),
                    })
                    .collect(),
            })
            .collect();

        Ok(HistoryResponse {
            total: json_entries.len(),
            entries: json_entries,
        })
    }

    /// 处理验证配置
    fn handle_validate(&self) -> Response<std::io::Cursor<Vec<u8>>> {
        match crate::commands::validate_command() {
            Ok(_) => self.json_response(
                ApiResponse::success("验证通过"),
                200,
            ),
            Err(e) => self.json_response(
                ApiResponse::<()>::error(e.user_message()),
                500,
            ),
        }
    }

    /// 处理清理备份
    fn handle_clean(&self, request: &mut Request) -> Response<std::io::Cursor<Vec<u8>>> {
        match self.clean_impl(request) {
            Ok(data) => self.json_response(ApiResponse::success(data), 200),
            Err(e) => self.json_response(
                ApiResponse::<()>::error(e.user_message()),
                500,
            ),
        }
    }

    fn clean_impl(&self, request: &mut Request) -> Result<CleanResponse> {
        let mut body = String::new();
        request.as_reader().read_to_string(&mut body).map_err(|e| {
            CcrError::ConfigError(format!("读取请求体失败: {}", e))
        })?;

        let req: CleanRequest = serde_json::from_str(&body).unwrap_or(CleanRequest {
            days: 7,
            dry_run: false,
        });

        // 调用 clean 功能实现
        use std::fs;
        use std::time::{Duration, SystemTime};

        let home = dirs::home_dir()
            .ok_or_else(|| CcrError::ConfigError("无法获取用户主目录".into()))?;
        let backup_dir = home.join(".claude").join("backups");

        if !backup_dir.exists() {
            return Ok(CleanResponse {
                deleted_count: 0,
                skipped_count: 0,
                total_size_mb: 0.0,
                dry_run: req.dry_run,
            });
        }

        let cutoff_time = SystemTime::now() - Duration::from_secs(req.days * 24 * 60 * 60);
        let mut deleted_count = 0;
        let mut skipped_count = 0;
        let mut total_size = 0u64;

        let entries = fs::read_dir(&backup_dir)
            .map_err(|e| CcrError::ConfigError(format!("读取备份目录失败: {}", e)))?;

        for entry in entries {
            let entry = entry.map_err(|e| CcrError::ConfigError(format!("读取目录项失败: {}", e)))?;
            let path = entry.path();

            if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("bak") {
                continue;
            }

            let metadata = fs::metadata(&path)
                .map_err(|e| CcrError::ConfigError(format!("读取文件元数据失败: {}", e)))?;

            let modified_time = metadata.modified()
                .map_err(|e| CcrError::ConfigError(format!("获取文件修改时间失败: {}", e)))?;

            if modified_time < cutoff_time {
                let file_size = metadata.len();
                total_size += file_size;
                deleted_count += 1;

                if !req.dry_run {
                    fs::remove_file(&path)
                        .map_err(|e| CcrError::ConfigError(format!("删除文件失败: {}", e)))?;
                }
            } else {
                skipped_count += 1;
            }
        }

        Ok(CleanResponse {
            deleted_count,
            skipped_count,
            total_size_mb: total_size as f64 / 1024.0 / 1024.0,
            dry_run: req.dry_run,
        })
    }

    /// 处理获取 Settings
    fn handle_get_settings(&self) -> Response<std::io::Cursor<Vec<u8>>> {
        match self.get_settings_impl() {
            Ok(data) => self.json_response(ApiResponse::success(data), 200),
            Err(e) => self.json_response(
                ApiResponse::<()>::error(e.user_message()),
                500,
            ),
        }
    }

    fn get_settings_impl(&self) -> Result<SettingsResponse> {
        let settings = self.settings_manager.load()?;
        Ok(SettingsResponse {
            settings: serde_json::to_value(&settings)
                .map_err(|e| CcrError::ConfigError(format!("序列化设置失败: {}", e)))?,
        })
    }

    /// 处理获取 Settings 备份列表
    fn handle_get_settings_backups(&self) -> Response<std::io::Cursor<Vec<u8>>> {
        match self.get_settings_backups_impl() {
            Ok(data) => self.json_response(ApiResponse::success(data), 200),
            Err(e) => self.json_response(
                ApiResponse::<()>::error(e.user_message()),
                500,
            ),
        }
    }

    fn get_settings_backups_impl(&self) -> Result<SettingsBackupsResponse> {
        let backup_paths = self.settings_manager.list_backups()?;

        let backups: Vec<BackupItem> = backup_paths
            .iter()
            .filter_map(|path| {
                let metadata = std::fs::metadata(path).ok()?;
                let created_at = metadata.modified().ok()?
                    .duration_since(std::time::UNIX_EPOCH).ok()?;

                Some(BackupItem {
                    filename: path.file_name()?.to_string_lossy().to_string(),
                    path: path.to_string_lossy().to_string(),
                    created_at: chrono::DateTime::<chrono::Utc>::from(
                        std::time::UNIX_EPOCH + std::time::Duration::from_secs(created_at.as_secs())
                    ).to_rfc3339(),
                    size_bytes: metadata.len(),
                })
            })
            .collect();

        Ok(SettingsBackupsResponse { backups })
    }

    /// 处理恢复 Settings
    fn handle_restore_settings(&self, request: &mut Request) -> Response<std::io::Cursor<Vec<u8>>> {
        match self.restore_settings_impl(request) {
            Ok(_) => self.json_response(
                ApiResponse::success("Settings 恢复成功"),
                200,
            ),
            Err(e) => self.json_response(
                ApiResponse::<()>::error(e.user_message()),
                500,
            ),
        }
    }

    fn restore_settings_impl(&self, request: &mut Request) -> Result<()> {
        let mut body = String::new();
        request.as_reader().read_to_string(&mut body).map_err(|e| {
            CcrError::ConfigError(format!("读取请求体失败: {}", e))
        })?;

        let req: RestoreSettingsRequest = serde_json::from_str(&body).map_err(|e| {
            CcrError::ConfigError(format!("解析请求失败: {}", e))
        })?;

        self.settings_manager.restore(&req.backup_path)?;
        Ok(())
    }

    /// 创建 JSON 响应
    fn json_response<T: Serialize>(&self, data: T, status: u16) -> Response<std::io::Cursor<Vec<u8>>> {
        let json = serde_json::to_string(&data).unwrap_or_else(|_| {
            r#"{"success":false,"message":"序列化失败"}"#.to_string()
        });

        Response::from_data(json.into_bytes())
            .with_header(
                Header::from_bytes(&b"Content-Type"[..], &b"application/json; charset=utf-8"[..])
                    .unwrap(),
            )
            .with_status_code(StatusCode(status))
    }
}

/// Web 命令入口
pub fn web_command(port: Option<u16>) -> Result<()> {
    let port = port.unwrap_or(8080);
    let server = WebServer::new(port)?;
    server.start()
}
