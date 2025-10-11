// 🔌 Web 请求处理器
// 处理所有 HTTP 请求并调用相应的 Service

use crate::config::ConfigSection;
use crate::error::{CcrError, Result};
use crate::logging::ColorOutput;
use crate::services::{BackupService, ConfigService, HistoryService, SettingsService};
use crate::web::models::*;
use serde::Serialize;
use std::sync::Arc;
use tiny_http::{Header, Method, Request, Response, StatusCode};

/// 🔌 请求处理器
///
/// 持有所有 Service 的引用,处理 HTTP 请求
pub struct Handlers {
    config_service: Arc<ConfigService>,
    settings_service: Arc<SettingsService>,
    history_service: Arc<HistoryService>,
    backup_service: Arc<BackupService>,
}

impl Handlers {
    /// 🏗️ 创建新的处理器
    pub fn new(
        config_service: Arc<ConfigService>,
        settings_service: Arc<SettingsService>,
        history_service: Arc<HistoryService>,
        backup_service: Arc<BackupService>,
    ) -> Self {
        Self {
            config_service,
            settings_service,
            history_service,
            backup_service,
        }
    }

    /// 🔄 处理请求
    pub fn handle_request(&self, mut request: Request) -> Result<()> {
        let url = request.url().to_string();
        let method = request.method().clone();

        log::debug!("{} {}", method, url);

        // 路由分发
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
            (Method::Get, "/api/history") => self.handle_get_history(),
            (Method::Post, "/api/validate") => self.handle_validate(),
            (Method::Post, "/api/clean") => self.handle_clean(&mut request),
            (Method::Get, "/api/settings") => self.handle_get_settings(),
            (Method::Get, "/api/settings/backups") => self.handle_get_settings_backups(),
            (Method::Post, "/api/settings/restore") => self.handle_restore_settings(&mut request),

            // 404
            _ => self.serve_404(),
        };

        // 发送响应
        request
            .respond(response)
            .map_err(|e| CcrError::ConfigError(format!("发送响应失败: {}", e)))?;

        Ok(())
    }

    /// 提供 HTML 页面
    fn serve_html(&self) -> Response<std::io::Cursor<Vec<u8>>> {
        let html = include_str!("../../web/index.html");
        let content = html.as_bytes().to_vec();

        Response::from_data(content)
            .with_header(
                Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap(),
            )
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
            .with_header(
                Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap(),
            )
            .with_status_code(StatusCode(404))
    }

    /// 处理列出配置
    fn handle_list_configs(&self) -> Response<std::io::Cursor<Vec<u8>>> {
        match self.config_service.list_configs() {
            Ok(list) => {
                let configs: Vec<ConfigItem> = list
                    .configs
                    .into_iter()
                    .map(|info| ConfigItem {
                        name: info.name.clone(),
                        description: info.description.clone(),
                        base_url: info.base_url.clone().unwrap_or_default(),
                        auth_token: ColorOutput::mask_sensitive(
                            &info.auth_token.clone().unwrap_or_default(),
                        ),
                        model: info.model,
                        small_fast_model: info.small_fast_model,
                        is_current: info.is_current,
                        is_default: info.is_default,
                    })
                    .collect();

                let response_data = ConfigListResponse {
                    current_config: list.current_config,
                    default_config: list.default_config,
                    configs,
                };

                self.json_response(ApiResponse::success(response_data), 200)
            }
            Err(e) => {
                let error_response: ApiResponse<()> =
                    ApiResponse::error_without_data(e.user_message());
                self.json_response(error_response, 500)
            }
        }
    }

    /// 处理切换配置
    fn handle_switch_config(&self, request: &mut Request) -> Response<std::io::Cursor<Vec<u8>>> {
        let mut body = String::new();
        if let Err(e) = request.as_reader().read_to_string(&mut body) {
            let error_response: ApiResponse<()> =
                ApiResponse::error_without_data(format!("读取请求体失败: {}", e));
            return self.json_response(error_response, 400);
        }

        let req: SwitchRequest = match serde_json::from_str(&body) {
            Ok(r) => r,
            Err(e) => {
                let error_response: ApiResponse<()> =
                    ApiResponse::error_without_data(format!("解析请求失败: {}", e));
                return self.json_response(error_response, 400);
            }
        };

        // 执行切换
        match crate::commands::switch_command(&req.config_name) {
            Ok(_) => self.json_response(ApiResponse::success("配置切换成功"), 200),
            Err(e) => {
                let error_response: ApiResponse<()> =
                    ApiResponse::error_without_data(e.user_message());
                self.json_response(error_response, 500)
            }
        }
    }

    /// 处理添加配置
    fn handle_add_config(&self, request: &mut Request) -> Response<std::io::Cursor<Vec<u8>>> {
        let mut body = String::new();
        if let Err(e) = request.as_reader().read_to_string(&mut body) {
            let error_response: ApiResponse<()> =
                ApiResponse::error_without_data(format!("读取请求体失败: {}", e));
            return self.json_response(error_response, 400);
        }

        let req: UpdateConfigRequest = match serde_json::from_str(&body) {
            Ok(r) => r,
            Err(e) => {
                let error_response: ApiResponse<()> =
                    ApiResponse::error_without_data(format!("解析请求失败: {}", e));
                return self.json_response(error_response, 400);
            }
        };

        let section = ConfigSection {
            description: req.description,
            base_url: Some(req.base_url),
            auth_token: Some(req.auth_token),
            model: req.model,
            small_fast_model: req.small_fast_model,
        };

        match self.config_service.add_config(req.name, section) {
            Ok(_) => self.json_response(ApiResponse::success("配置添加成功"), 200),
            Err(e) => {
                let error_response: ApiResponse<()> =
                    ApiResponse::error_without_data(e.user_message());
                self.json_response(error_response, 500)
            }
        }
    }

    /// 处理更新配置
    fn handle_update_config(
        &self,
        old_name: &str,
        request: &mut Request,
    ) -> Response<std::io::Cursor<Vec<u8>>> {
        let mut body = String::new();
        if let Err(e) = request.as_reader().read_to_string(&mut body) {
            let error_response: ApiResponse<()> =
                ApiResponse::error_without_data(format!("读取请求体失败: {}", e));
            return self.json_response(error_response, 400);
        }

        let req: UpdateConfigRequest = match serde_json::from_str(&body) {
            Ok(r) => r,
            Err(e) => {
                let error_response: ApiResponse<()> =
                    ApiResponse::error_without_data(format!("解析请求失败: {}", e));
                return self.json_response(error_response, 400);
            }
        };

        let section = ConfigSection {
            description: req.description,
            base_url: Some(req.base_url),
            auth_token: Some(req.auth_token),
            model: req.model,
            small_fast_model: req.small_fast_model,
        };

        match self
            .config_service
            .update_config(old_name, req.name, section)
        {
            Ok(_) => self.json_response(ApiResponse::success("配置更新成功"), 200),
            Err(e) => {
                let error_response: ApiResponse<()> =
                    ApiResponse::error_without_data(e.user_message());
                self.json_response(error_response, 500)
            }
        }
    }

    /// 处理删除配置
    fn handle_delete_config(&self, name: &str) -> Response<std::io::Cursor<Vec<u8>>> {
        match self.config_service.delete_config(name) {
            Ok(_) => self.json_response(ApiResponse::success("配置删除成功"), 200),
            Err(e) => {
                let error_response: ApiResponse<()> =
                    ApiResponse::error_without_data(e.user_message());
                self.json_response(error_response, 500)
            }
        }
    }

    /// 处理获取历史记录
    fn handle_get_history(&self) -> Response<std::io::Cursor<Vec<u8>>> {
        match self.history_service.get_recent(50) {
            Ok(entries) => {
                let json_entries: Vec<HistoryEntryJson> = entries
                    .iter()
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
                                old_value: change.old_value.clone(),
                                new_value: change.new_value.clone(),
                            })
                            .collect(),
                    })
                    .collect();

                let response_data = HistoryResponse {
                    entries: json_entries.clone(),
                    total: json_entries.len(),
                };

                self.json_response(ApiResponse::success(response_data), 200)
            }
            Err(e) => {
                let error_response: ApiResponse<()> =
                    ApiResponse::error_without_data(e.user_message());
                self.json_response(error_response, 500)
            }
        }
    }

    /// 处理验证配置
    fn handle_validate(&self) -> Response<std::io::Cursor<Vec<u8>>> {
        match crate::commands::validate_command() {
            Ok(_) => self.json_response(ApiResponse::success("验证通过"), 200),
            Err(e) => {
                let error_response: ApiResponse<()> =
                    ApiResponse::error_without_data(e.user_message());
                self.json_response(error_response, 500)
            }
        }
    }

    /// 处理清理备份
    fn handle_clean(&self, request: &mut Request) -> Response<std::io::Cursor<Vec<u8>>> {
        let mut body = String::new();
        if let Err(e) = request.as_reader().read_to_string(&mut body) {
            let error_response: ApiResponse<()> =
                ApiResponse::error_without_data(format!("读取请求体失败: {}", e));
            return self.json_response(error_response, 400);
        }

        let req: CleanRequest = serde_json::from_str(&body).unwrap_or(CleanRequest {
            days: 7,
            dry_run: false,
        });

        match self.backup_service.clean_old_backups(req.days, req.dry_run) {
            Ok(result) => {
                let response_data = CleanResponse {
                    deleted_count: result.deleted_count,
                    skipped_count: result.skipped_count,
                    total_size_mb: result.total_size as f64 / 1024.0 / 1024.0,
                    dry_run: req.dry_run,
                };
                self.json_response(ApiResponse::success(response_data), 200)
            }
            Err(e) => {
                let error_response: ApiResponse<()> =
                    ApiResponse::error_without_data(e.user_message());
                self.json_response(error_response, 500)
            }
        }
    }

    /// 处理获取 Settings
    fn handle_get_settings(&self) -> Response<std::io::Cursor<Vec<u8>>> {
        match self.settings_service.get_current_settings() {
            Ok(settings) => {
                let settings_value = match serde_json::to_value(&settings) {
                    Ok(v) => v,
                    Err(e) => {
                        let error_response: ApiResponse<()> =
                            ApiResponse::error_without_data(format!("序列化设置失败: {}", e));
                        return self.json_response(error_response, 500);
                    }
                };

                let response_data = SettingsResponse {
                    settings: settings_value,
                };

                self.json_response(ApiResponse::success(response_data), 200)
            }
            Err(e) => {
                let error_response: ApiResponse<()> =
                    ApiResponse::error_without_data(e.user_message());
                self.json_response(error_response, 500)
            }
        }
    }

    /// 处理获取 Settings 备份列表
    fn handle_get_settings_backups(&self) -> Response<std::io::Cursor<Vec<u8>>> {
        match self.settings_service.list_backups() {
            Ok(backups) => {
                let backup_items: Vec<BackupItem> = backups
                    .iter()
                    .map(|backup| BackupItem {
                        filename: backup.filename.clone(),
                        path: backup.path.to_string_lossy().to_string(),
                        created_at: chrono::DateTime::<chrono::Utc>::from(backup.created_at)
                            .to_rfc3339(),
                        size_bytes: backup.size_bytes,
                    })
                    .collect();

                let response_data = SettingsBackupsResponse {
                    backups: backup_items,
                };

                self.json_response(ApiResponse::success(response_data), 200)
            }
            Err(e) => {
                let error_response: ApiResponse<()> =
                    ApiResponse::error_without_data(e.user_message());
                self.json_response(error_response, 500)
            }
        }
    }

    /// 处理恢复 Settings
    fn handle_restore_settings(&self, request: &mut Request) -> Response<std::io::Cursor<Vec<u8>>> {
        let mut body = String::new();
        if let Err(e) = request.as_reader().read_to_string(&mut body) {
            let error_response: ApiResponse<()> =
                ApiResponse::error_without_data(format!("读取请求体失败: {}", e));
            return self.json_response(error_response, 400);
        }

        let req: RestoreSettingsRequest = match serde_json::from_str(&body) {
            Ok(r) => r,
            Err(e) => {
                let error_response: ApiResponse<()> =
                    ApiResponse::error_without_data(format!("解析请求失败: {}", e));
                return self.json_response(error_response, 400);
            }
        };

        match self
            .settings_service
            .restore_settings(std::path::Path::new(&req.backup_path))
        {
            Ok(_) => self.json_response(ApiResponse::success("Settings 恢复成功"), 200),
            Err(e) => {
                let error_response: ApiResponse<()> =
                    ApiResponse::error_without_data(e.user_message());
                self.json_response(error_response, 500)
            }
        }
    }

    /// 创建 JSON 响应
    fn json_response<T: Serialize>(
        &self,
        data: T,
        status: u16,
    ) -> Response<std::io::Cursor<Vec<u8>>> {
        let json = serde_json::to_string(&data)
            .unwrap_or_else(|_| r#"{"success":false,"message":"序列化失败"}"#.to_string());

        Response::from_data(json.into_bytes())
            .with_header(
                Header::from_bytes(
                    &b"Content-Type"[..],
                    &b"application/json; charset=utf-8"[..],
                )
                .unwrap(),
            )
            .with_status_code(StatusCode(status))
    }
}
