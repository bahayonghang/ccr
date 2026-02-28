// 🔐 Codex Auth 服务层
// 管理 Codex CLI 的多账号登录状态
//
// 核心职责:
// - 📋 检测登录状态
// - 💾 保存/切换/删除账号
// - 🔍 解析 JWT 提取账号信息
// - ⏰ 计算 Token 新鲜度
// - 🔄 进程检测与备份管理

use crate::core::error::{CcrError, Result};
use crate::models::{
    CodexAuthAccount, CodexAuthExport, CodexAuthExportAccount, CodexAuthItem, CodexAuthJson,
    CodexAuthRegistry, CurrentAuthInfo, ImportMode, ImportResult, LoginState, TokenFreshness,
};
use chrono::{DateTime, Duration, Utc};
use std::path::PathBuf;
use std::{env, fs};
use tracing::{debug, warn};

/// 备份保留数量
const MAX_BACKUPS: usize = 10;

/// Codex Auth 服务
///
/// 提供 Codex 多账号管理的所有业务逻辑
pub struct CodexAuthService {
    /// CCR 平台数据目录 (~/.ccr/platforms/codex/)
    ccr_codex_dir: PathBuf,
    /// Codex CLI 配置目录 (~/.codex/)
    codex_dir: PathBuf,
}

impl CodexAuthService {
    /// 创建新的 CodexAuthService 实例
    pub fn new() -> Result<Self> {
        let home =
            dirs::home_dir().ok_or_else(|| CcrError::ConfigError("无法获取用户主目录".into()))?;

        let ccr_codex_dir = if let Ok(custom) = env::var("CCR_DATA_DIR") {
            PathBuf::from(custom).join("platforms/codex")
        } else {
            home.join(".ccr/platforms/codex")
        };

        let codex_dir = if let Ok(custom) = env::var("CCR_CODEX_DIR") {
            PathBuf::from(custom)
        } else {
            home.join(".codex")
        };

        Ok(Self {
            ccr_codex_dir,
            codex_dir,
        })
    }

    // ==================== 路径辅助方法 ====================

    /// 获取 Codex auth.json 路径
    fn auth_json_path(&self) -> PathBuf {
        self.codex_dir.join("auth.json")
    }

    /// 获取 CCR auth 存储目录
    fn auth_storage_dir(&self) -> PathBuf {
        self.ccr_codex_dir.join("auth")
    }

    /// 获取 auth_registry.toml 路径
    fn registry_path(&self) -> PathBuf {
        self.ccr_codex_dir.join("auth_registry.toml")
    }

    /// 获取备份目录
    fn backup_dir(&self) -> PathBuf {
        self.auth_storage_dir().join("backups")
    }

    /// 获取指定账号的 auth 文件路径
    fn account_auth_path(&self, name: &str) -> PathBuf {
        self.auth_storage_dir().join(format!("{}.json", name))
    }

    // ==================== 登录状态检测 ====================

    /// 检查用户是否已登录 Codex
    pub fn is_logged_in(&self) -> bool {
        let auth_path = self.auth_json_path();
        if !auth_path.exists() {
            return false;
        }

        // 尝试解析 auth.json 验证其有效性
        match fs::read_to_string(&auth_path) {
            Ok(content) => serde_json::from_str::<CodexAuthJson>(&content).is_ok(),
            Err(_) => false,
        }
    }

    /// 获取当前登录状态
    pub fn get_login_state(&self) -> Result<LoginState> {
        if !self.is_logged_in() {
            return Ok(LoginState::NotLoggedIn);
        }

        // 检查当前登录是否已保存
        let current_info = self.get_current_auth_info()?;
        let registry = self.load_registry()?;

        // 查找匹配的已保存账号
        for (name, account) in &registry.accounts {
            if account.account_id == current_info.account_id {
                return Ok(LoginState::LoggedInSaved(name.clone()));
            }
        }

        Ok(LoginState::LoggedInUnsaved)
    }

    /// 获取当前 auth.json 的解析信息
    pub fn get_current_auth_info(&self) -> Result<CurrentAuthInfo> {
        let auth_path = self.auth_json_path();
        if !auth_path.exists() {
            return Err(CcrError::ConfigError(
                "未登录 Codex，请先运行 `codex login`".into(),
            ));
        }

        let content = fs::read_to_string(&auth_path)
            .map_err(|e| CcrError::ConfigError(format!("读取 auth.json 失败: {}", e)))?;

        let auth: CodexAuthJson = serde_json::from_str(&content)
            .map_err(|e| CcrError::ConfigError(format!("解析 auth.json 失败: {}", e)))?;

        // 提取 account_id
        let account_id = auth
            .tokens
            .as_ref()
            .and_then(|t| t.account_id.clone())
            .unwrap_or_else(|| "unknown".to_string());

        // 从 JWT 提取邮箱
        let email = self.extract_email_from_jwt(&auth);

        // 解析 last_refresh
        let last_refresh = auth
            .last_refresh
            .as_ref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        // 计算新鲜度
        let freshness = self.calculate_freshness(last_refresh);

        Ok(CurrentAuthInfo {
            account_id,
            email,
            last_refresh,
            freshness,
        })
    }

    // ==================== 账号管理操作 ====================

    /// 保存当前登录到指定名称
    pub fn save_current(
        &self,
        name: &str,
        description: Option<String>,
        expires_at: Option<DateTime<Utc>>,
        force: bool,
    ) -> Result<()> {
        // 检查是否已登录
        if !self.is_logged_in() {
            return Err(CcrError::ConfigError(
                "未登录 Codex，请先运行 `codex login`".into(),
            ));
        }

        // 验证名称
        self.validate_account_name(name)?;

        // 检查是否已存在
        let mut registry = self.load_registry()?;
        if registry.accounts.contains_key(name) && !force {
            return Err(CcrError::ConfigError(format!(
                "账号 '{}' 已存在，使用 --force 覆盖",
                name
            )));
        }

        // 确保目录存在
        let auth_storage = self.auth_storage_dir();
        fs::create_dir_all(&auth_storage)
            .map_err(|e| CcrError::ConfigError(format!("创建存储目录失败: {}", e)))?;

        // 复制 auth.json
        let src = self.auth_json_path();
        let dst = self.account_auth_path(name);
        fs::copy(&src, &dst)
            .map_err(|e| CcrError::ConfigError(format!("复制 auth.json 失败: {}", e)))?;

        // 设置文件权限 (Unix)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            let _ = fs::set_permissions(&dst, perms);
        }

        // 获取当前账号信息
        let current_info = self.get_current_auth_info()?;

        // 更新注册表
        let account = CodexAuthAccount {
            description,
            account_id: current_info.account_id,
            email: current_info.email.map(|e| self.mask_email(&e)),
            saved_at: Utc::now(),
            last_used: Some(Utc::now()),
            last_refresh: current_info.last_refresh,
            expires_at,
        };

        registry.accounts.insert(name.to_string(), account);
        registry.current_auth = Some(name.to_string());
        self.save_registry(&registry)?;

        debug!("已保存账号: {}", name);
        Ok(())
    }

    /// 列出所有账号
    pub fn list_accounts(&self) -> Result<Vec<CodexAuthItem>> {
        let registry = self.load_registry()?;
        let mut items = Vec::new();

        // 检查当前登录状态
        let login_state = self.get_login_state()?;
        let current_info = if self.is_logged_in() {
            self.get_current_auth_info().ok()
        } else {
            None
        };

        // 如果已登录但未保存，添加虚拟 "default" 项
        if let LoginState::LoggedInUnsaved = login_state
            && let Some(info) = &current_info
        {
            items.push(CodexAuthItem {
                name: "default".to_string(),
                description: Some("(未保存的当前登录)".to_string()),
                email: info.email.clone().map(|e| self.mask_email(&e)),
                is_current: true,
                is_virtual: true,
                saved_at: None,
                last_used: None,
                last_refresh: info.last_refresh,
                freshness: info.freshness,
                expires_at: None, // 虚拟项没有过期时间
            });
        }

        // 添加所有已保存的账号
        for (name, account) in &registry.accounts {
            let is_current = match &login_state {
                LoginState::LoggedInSaved(current_name) => current_name == name,
                _ => false,
            };

            // 计算新鲜度 (从保存的 auth 文件读取)
            let freshness = self.get_account_freshness(name);

            items.push(CodexAuthItem {
                name: name.clone(),
                description: account.description.clone(),
                email: account.email.clone(),
                is_current,
                is_virtual: false,
                saved_at: Some(account.saved_at),
                last_used: account.last_used,
                last_refresh: account.last_refresh,
                freshness,
                expires_at: account.expires_at,
            });
        }

        Ok(items)
    }

    /// 切换到指定账号
    pub fn switch_account(&self, name: &str) -> Result<()> {
        // 检查账号是否存在
        let registry = self.load_registry()?;
        if !registry.accounts.contains_key(name) {
            let available: Vec<_> = registry.accounts.keys().collect();
            return Err(CcrError::ConfigError(format!(
                "账号 '{}' 不存在。可用账号: {:?}",
                name, available
            )));
        }

        // 检查是否过期
        if let Some(account) = registry.accounts.get(name)
            && Self::is_expired(account.expires_at)
        {
            return Err(CcrError::ValidationError(format!(
                "账号 '{}' 已过期，无法切换。请更新或保存新的登录。",
                name
            )));
        }

        // 备份当前 auth.json (如果存在)
        if self.is_logged_in() {
            self.backup_current_auth()?;
        }

        // 复制保存的 auth 到 ~/.codex/auth.json
        let src = self.account_auth_path(name);
        let dst = self.auth_json_path();

        // 确保目标目录存在
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| CcrError::ConfigError(format!("创建 Codex 目录失败: {}", e)))?;
        }

        fs::copy(&src, &dst).map_err(|e| CcrError::ConfigError(format!("切换账号失败: {}", e)))?;

        // 更新注册表
        let mut registry = self.load_registry()?;
        registry.current_auth = Some(name.to_string());
        if let Some(account) = registry.accounts.get_mut(name) {
            account.last_used = Some(Utc::now());
        }
        self.save_registry(&registry)?;

        debug!("已切换到账号: {}", name);
        Ok(())
    }

    /// 删除指定账号
    pub fn delete_account(&self, name: &str) -> Result<()> {
        let mut registry = self.load_registry()?;

        // 检查账号是否存在
        if !registry.accounts.contains_key(name) {
            return Err(CcrError::ConfigError(format!("账号 '{}' 不存在", name)));
        }

        // 删除 auth 文件
        let auth_path = self.account_auth_path(name);
        if auth_path.exists() {
            fs::remove_file(&auth_path)
                .map_err(|e| CcrError::ConfigError(format!("删除 auth 文件失败: {}", e)))?;
        }

        // 从注册表移除
        registry.accounts.shift_remove(name);

        // 如果删除的是当前账号，清除 current_auth
        if registry.current_auth.as_deref() == Some(name) {
            registry.current_auth = None;
        }

        self.save_registry(&registry)?;

        debug!("已删除账号: {}", name);
        Ok(())
    }

    // ==================== 备份管理 ====================

    /// 备份当前 auth.json
    pub fn backup_current_auth(&self) -> Result<PathBuf> {
        let auth_path = self.auth_json_path();
        if !auth_path.exists() {
            return Err(CcrError::ConfigError("没有可备份的 auth.json".into()));
        }

        // 确保备份目录存在
        let backup_dir = self.backup_dir();
        fs::create_dir_all(&backup_dir)
            .map_err(|e| CcrError::ConfigError(format!("创建备份目录失败: {}", e)))?;

        // 生成带时间戳的备份文件名
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_name = format!("auth_{}.json", timestamp);
        let backup_path = backup_dir.join(&backup_name);

        // 复制文件
        fs::copy(&auth_path, &backup_path)
            .map_err(|e| CcrError::ConfigError(format!("备份失败: {}", e)))?;

        // 清理旧备份
        self.cleanup_old_backups()?;

        debug!("已备份到: {}", backup_path.display());
        Ok(backup_path)
    }

    fn backup_registry(&self) -> Result<Option<PathBuf>> {
        let registry_path = self.registry_path();
        if !registry_path.exists() {
            return Ok(None);
        }

        let backup_dir = self.backup_dir();
        fs::create_dir_all(&backup_dir)
            .map_err(|e| CcrError::ConfigError(format!("创建备份目录失败: {}", e)))?;

        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_name = format!("auth_registry_{}.toml", timestamp);
        let backup_path = backup_dir.join(&backup_name);

        fs::copy(&registry_path, &backup_path)
            .map_err(|e| CcrError::ConfigError(format!("备份注册表失败: {}", e)))?;

        Ok(Some(backup_path))
    }

    fn backup_account_auth(&self, name: &str) -> Result<Option<PathBuf>> {
        let auth_path = self.account_auth_path(name);
        if !auth_path.exists() {
            return Ok(None);
        }

        let backup_dir = self.backup_dir();
        fs::create_dir_all(&backup_dir)
            .map_err(|e| CcrError::ConfigError(format!("创建备份目录失败: {}", e)))?;

        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_name = format!("auth_account_{}_{}.json", name, timestamp);
        let backup_path = backup_dir.join(&backup_name);

        fs::copy(&auth_path, &backup_path)
            .map_err(|e| CcrError::ConfigError(format!("备份 auth 文件失败: {}", e)))?;

        Ok(Some(backup_path))
    }

    /// 清理旧备份，保留最新的 MAX_BACKUPS 个
    fn cleanup_old_backups(&self) -> Result<()> {
        let backup_dir = self.backup_dir();
        if !backup_dir.exists() {
            return Ok(());
        }

        let mut backups: Vec<_> = fs::read_dir(&backup_dir)
            .map_err(|e| CcrError::ConfigError(format!("读取备份目录失败: {}", e)))?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with("auth_") && n.ends_with(".json"))
            })
            .collect();

        // 按修改时间排序 (最新的在前)
        backups.sort_by(|a, b| {
            let time_a = a.metadata().and_then(|m| m.modified()).ok();
            let time_b = b.metadata().and_then(|m| m.modified()).ok();
            time_b.cmp(&time_a)
        });

        // 删除超出限制的旧备份
        for backup in backups.iter().skip(MAX_BACKUPS) {
            if let Err(e) = fs::remove_file(backup.path()) {
                warn!("删除旧备份失败: {}", e);
            }
        }

        Ok(())
    }

    // ==================== 进程检测 ====================

    /// 检测是否有 Codex 进程正在运行
    #[cfg(feature = "web")]
    pub fn detect_codex_process(&self) -> Vec<u32> {
        use sysinfo::System;

        let mut sys = System::new();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

        sys.processes()
            .iter()
            .filter(|(_, process)| {
                let name = process.name().to_string_lossy().to_lowercase();
                name.contains("codex") && !name.contains("ccr")
            })
            .map(|(pid, _)| pid.as_u32())
            .collect()
    }

    /// 检测是否有 Codex 进程正在运行 (无 sysinfo 时的 fallback)
    #[cfg(not(feature = "web"))]
    pub fn detect_codex_process(&self) -> Vec<u32> {
        // 无 sysinfo 依赖时返回空
        Vec::new()
    }

    // ==================== Token 新鲜度 ====================

    /// 计算 Token 新鲜度
    pub fn calculate_freshness(&self, last_refresh: Option<DateTime<Utc>>) -> TokenFreshness {
        let Some(refresh_time) = last_refresh else {
            return TokenFreshness::Unknown;
        };

        let now = Utc::now();
        let age = now.signed_duration_since(refresh_time);

        if age < Duration::days(1) {
            TokenFreshness::Fresh
        } else if age < Duration::days(7) {
            TokenFreshness::Stale
        } else {
            TokenFreshness::Old
        }
    }

    /// 获取指定账号的 Token 新鲜度
    fn get_account_freshness(&self, name: &str) -> TokenFreshness {
        let auth_path = self.account_auth_path(name);
        if !auth_path.exists() {
            return TokenFreshness::Unknown;
        }

        let content = match fs::read_to_string(&auth_path) {
            Ok(c) => c,
            Err(_) => return TokenFreshness::Unknown,
        };

        let auth: CodexAuthJson = match serde_json::from_str(&content) {
            Ok(a) => a,
            Err(_) => return TokenFreshness::Unknown,
        };

        let last_refresh = auth
            .last_refresh
            .as_ref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        self.calculate_freshness(last_refresh)
    }

    // ==================== JWT 解析 ====================

    /// 从 JWT 中提取邮箱
    fn extract_email_from_jwt(&self, auth: &CodexAuthJson) -> Option<String> {
        let id_token = auth.tokens.as_ref()?.id_token.as_ref()?;

        // JWT 格式: header.payload.signature
        let parts: Vec<&str> = id_token.split('.').collect();
        if parts.len() != 3 {
            return None;
        }

        // 解码 payload (base64url)
        let payload = parts[1];
        let decoded = self.base64url_decode(payload)?;
        let payload_str = String::from_utf8(decoded).ok()?;

        // 解析 JSON
        let payload_json: serde_json::Value = serde_json::from_str(&payload_str).ok()?;

        // 提取 email 字段
        payload_json
            .get("email")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    /// Base64URL 解码
    fn base64url_decode(&self, input: &str) -> Option<Vec<u8>> {
        use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};

        // 添加 padding
        let padded = match input.len() % 4 {
            2 => format!("{}==", input),
            3 => format!("{}=", input),
            _ => input.to_string(),
        };

        URL_SAFE_NO_PAD.decode(&padded).ok().or_else(|| {
            // 尝试标准 base64
            use base64::engine::general_purpose::STANDARD;
            STANDARD.decode(&padded).ok()
        })
    }

    // ==================== 注册表管理 ====================

    /// 加载注册表
    pub fn load_registry(&self) -> Result<CodexAuthRegistry> {
        let path = self.registry_path();
        if !path.exists() {
            return Ok(CodexAuthRegistry::default());
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| CcrError::ConfigError(format!("读取注册表失败: {}", e)))?;

        toml::from_str(&content)
            .map_err(|e| CcrError::ConfigError(format!("解析注册表失败: {}", e)))
    }

    /// 保存注册表
    fn save_registry(&self, registry: &CodexAuthRegistry) -> Result<()> {
        let path = self.registry_path();

        // 确保目录存在
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| CcrError::ConfigError(format!("创建目录失败: {}", e)))?;
        }

        let content = toml::to_string_pretty(registry)
            .map_err(|e| CcrError::ConfigError(format!("序列化注册表失败: {}", e)))?;

        fs::write(&path, content)
            .map_err(|e| CcrError::ConfigError(format!("写入注册表失败: {}", e)))?;

        Ok(())
    }

    // ==================== 辅助方法 ====================

    /// 验证账号名称
    fn validate_account_name(&self, name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(CcrError::ValidationError("账号名称不能为空".into()));
        }

        if name == "default" {
            return Err(CcrError::ValidationError(
                "'default' 是保留名称，请使用其他名称".into(),
            ));
        }

        // 只允许字母、数字、下划线、连字符
        if !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Err(CcrError::ValidationError(
                "账号名称只能包含字母、数字、下划线和连字符".into(),
            ));
        }

        if name.len() > 32 {
            return Err(CcrError::ValidationError(
                "账号名称不能超过 32 个字符".into(),
            ));
        }

        Ok(())
    }

    /// 邮箱脱敏
    pub fn mask_email(&self, email: &str) -> String {
        if let Some(at_pos) = email.find('@') {
            let local = &email[..at_pos];
            let domain = &email[at_pos..];

            if local.len() <= 3 {
                format!("{}***{}", local, domain)
            } else {
                let visible = &local[..3];
                format!("{}***{}", visible, domain)
            }
        } else {
            // 不是有效邮箱格式，直接返回
            email.to_string()
        }
    }

    // ==================== 导入/导出 ====================

    /// 导出所有账号到 JSON
    ///
    /// # 参数
    ///
    /// * `include_secrets` - 是否包含完整的 auth.json 数据（Token 等敏感信息）
    ///
    /// # 返回
    ///
    /// * `Ok(String)` - JSON 格式的导出数据
    /// * `Err(CcrError)` - 导出失败
    pub fn export_accounts(&self, include_secrets: bool) -> Result<String> {
        let registry = self.load_registry()?;

        let mut export_accounts = indexmap::IndexMap::new();

        for (name, account) in &registry.accounts {
            let auth_data = if include_secrets {
                // 读取完整的 auth.json
                let auth_path = self.account_auth_path(name);
                if auth_path.exists() {
                    let content = fs::read_to_string(&auth_path)
                        .map_err(|e| CcrError::ConfigError(format!("读取账号文件失败: {}", e)))?;
                    Some(
                        serde_json::from_str(&content).map_err(|e| {
                            CcrError::ConfigError(format!("解析账号文件失败: {}", e))
                        })?,
                    )
                } else {
                    warn!("账号 {} 的 auth 文件不存在", name);
                    None
                }
            } else {
                None
            };

            export_accounts.insert(
                name.clone(),
                CodexAuthExportAccount {
                    description: account.description.clone(),
                    account_id: account.account_id.clone(),
                    email: account.email.clone(),
                    saved_at: account.saved_at,
                    last_used: account.last_used,
                    last_refresh: account.last_refresh,
                    expires_at: account.expires_at,
                    auth_data,
                },
            );
        }

        let export = CodexAuthExport {
            version: "1.0".to_string(),
            exported_at: Utc::now(),
            accounts: export_accounts,
        };

        serde_json::to_string_pretty(&export)
            .map_err(|e| CcrError::ConfigError(format!("序列化导出数据失败: {}", e)))
    }

    /// 导入账号数据
    ///
    /// # 参数
    ///
    /// * `content` - JSON 格式的导入数据
    /// * `mode` - 导入模式 (Merge/Replace)
    /// * `force` - 是否强制覆盖（仅在 Merge 模式下有效）
    ///
    /// # 返回
    ///
    /// * `Ok(ImportResult)` - 导入结果统计
    /// * `Err(CcrError)` - 导入失败
    pub fn import_accounts(
        &self,
        content: &str,
        mode: ImportMode,
        force: bool,
    ) -> Result<ImportResult> {
        // 解析导入数据
        let import_data: CodexAuthExport = serde_json::from_str(content)
            .map_err(|e| CcrError::ConfigError(format!("解析导入数据失败: {}", e)))?;

        let mut registry = self.load_registry()?;
        let mut result = ImportResult::default();

        // 确保存储目录存在
        let auth_storage = self.auth_storage_dir();
        fs::create_dir_all(&auth_storage)
            .map_err(|e| CcrError::ConfigError(format!("创建存储目录失败: {}", e)))?;

        let mut registry_backed_up = false;

        for (name, import_account) in import_data.accounts {
            // 验证账号名称
            self.validate_account_name(&name)?;

            let exists = registry.accounts.contains_key(&name);

            if force && exists && !registry_backed_up {
                if let Some(path) = self.backup_registry()? {
                    debug!("已备份注册表: {}", path.display());
                }
                registry_backed_up = true;
            }

            match mode {
                ImportMode::Merge => {
                    if exists && !force {
                        debug!("跳过已存在的账号: {}", name);
                        result.skipped += 1;
                        continue;
                    }
                    if exists && force {
                        debug!("强制覆盖已存在的账号: {}", name);
                        result.overwritten.push(name.clone());
                    }
                }
                ImportMode::Replace => {
                    if exists {
                        debug!("替换模式覆盖账号: {}", name);
                        result.overwritten.push(name.clone());
                    }
                }
            }

            if force && exists {
                if let Some(path) = self.backup_account_auth(&name)? {
                    debug!("已备份账号 {} 的 auth 文件: {}", name, path.display());
                }

                let auth_path = self.account_auth_path(&name);
                if auth_path.exists() {
                    let metadata = fs::metadata(&auth_path)
                        .map_err(|e| CcrError::ConfigError(format!("无法读取文件元数据: {}", e)))?;
                    if metadata.permissions().readonly() {
                        return Err(CcrError::ConfigError(format!(
                            "无法覆盖账号 '{}': 文件为只读",
                            name
                        )));
                    }

                    fs::remove_file(&auth_path)
                        .map_err(|e| CcrError::ConfigError(format!("删除 auth 文件失败: {}", e)))?;
                }

                registry.accounts.shift_remove(&name);
            }

            // 保存 auth 文件（如果有）
            if let Some(auth_data) = &import_account.auth_data {
                let auth_path = self.account_auth_path(&name);

                // 检查文件写入权限
                if auth_path.exists() {
                    let metadata = fs::metadata(&auth_path)
                        .map_err(|e| CcrError::ConfigError(format!("无法读取文件元数据: {}", e)))?;
                    if metadata.permissions().readonly() {
                        return Err(CcrError::ConfigError(format!(
                            "无法覆盖账号 '{}': 文件为只读",
                            name
                        )));
                    }
                }

                let auth_content = serde_json::to_string_pretty(auth_data)
                    .map_err(|e| CcrError::ConfigError(format!("序列化 auth 数据失败: {}", e)))?;
                fs::write(&auth_path, auth_content).map_err(|e| {
                    CcrError::ConfigError(format!("写入 auth 文件失败 (账号: {}): {}", name, e))
                })?;

                // 设置文件权限 (Unix)
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let perms = std::fs::Permissions::from_mode(0o600);
                    let _ = fs::set_permissions(&auth_path, perms);
                }

                debug!("已写入账号 {} 的 auth 文件", name);
            }

            // 更新注册表
            let account = CodexAuthAccount {
                description: import_account.description,
                account_id: import_account.account_id,
                email: import_account.email,
                saved_at: import_account.saved_at,
                last_used: import_account.last_used,
                last_refresh: import_account.last_refresh,
                expires_at: import_account.expires_at,
            };

            registry.accounts.insert(name.clone(), account);

            if exists {
                debug!("已更新账号: {}", name);
                result.updated += 1;
            } else {
                debug!("已添加账号: {}", name);
                result.added += 1;
            }
        }

        self.save_registry(&registry)?;

        Ok(result)
    }

    /// 判断账号是否过期
    pub fn is_expired(expires_at: Option<DateTime<Utc>>) -> bool {
        if let Some(ts) = expires_at {
            ts <= Utc::now()
        } else {
            false
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::models::CodexAuthTokens;
    use tempfile::TempDir;

    /// 创建测试用的 service 实例
    fn create_test_service() -> (CodexAuthService, TempDir, TempDir) {
        let ccr_dir = TempDir::new().unwrap();
        let codex_dir = TempDir::new().unwrap();

        let service = CodexAuthService {
            ccr_codex_dir: ccr_dir.path().to_path_buf(),
            codex_dir: codex_dir.path().to_path_buf(),
        };

        (service, ccr_dir, codex_dir)
    }

    /// 创建测试用的 auth.json 内容
    fn create_test_auth_json(account_id: &str, last_refresh: &str) -> String {
        format!(
            r#"{{
                "OPENAI_API_KEY": null,
                "tokens": {{
                    "id_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJlbWFpbCI6InRlc3RAZXhhbXBsZS5jb20iLCJzdWIiOiIxMjM0NTY3ODkwIn0.signature",
                    "access_token": "eyJ...",
                    "refresh_token": "rt_test",
                    "account_id": "{}"
                }},
                "last_refresh": "{}"
            }}"#,
            account_id, last_refresh
        )
    }

    // ==================== 邮箱脱敏测试 ====================

    #[test]
    fn test_mask_email() {
        let (service, _ccr, _codex) = create_test_service();

        assert_eq!(service.mask_email("user@example.com"), "use***@example.com");
        assert_eq!(service.mask_email("ab@example.com"), "ab***@example.com");
        assert_eq!(service.mask_email("a@example.com"), "a***@example.com");
        assert_eq!(service.mask_email("invalid"), "invalid");
    }

    #[test]
    fn test_mask_email_edge_cases() {
        let (service, _ccr, _codex) = create_test_service();

        // 空邮箱
        assert_eq!(service.mask_email(""), "");
        // 只有 @
        assert_eq!(service.mask_email("@domain.com"), "***@domain.com");
        // 多个 @
        assert_eq!(
            service.mask_email("user@sub@domain.com"),
            "use***@sub@domain.com"
        );
    }

    // ==================== 账号名称验证测试 ====================

    #[test]
    fn test_validate_account_name() {
        let (service, _ccr, _codex) = create_test_service();

        // 有效名称
        assert!(service.validate_account_name("my-account").is_ok());
        assert!(service.validate_account_name("account_1").is_ok());
        assert!(service.validate_account_name("Account123").is_ok());
        assert!(service.validate_account_name("a").is_ok());
        assert!(service.validate_account_name("A1_b2-c3").is_ok());

        // 无效名称
        assert!(service.validate_account_name("").is_err());
        assert!(service.validate_account_name("default").is_err());
        assert!(service.validate_account_name("invalid name").is_err());
        assert!(service.validate_account_name("名称").is_err());
        assert!(service.validate_account_name("user@email").is_err());
        assert!(service.validate_account_name("path/name").is_err());
    }

    #[test]
    fn test_validate_account_name_length() {
        let (service, _ccr, _codex) = create_test_service();

        // 32 字符 - 有效
        let valid_name = "a".repeat(32);
        assert!(service.validate_account_name(&valid_name).is_ok());

        // 33 字符 - 无效
        let invalid_name = "a".repeat(33);
        assert!(service.validate_account_name(&invalid_name).is_err());
    }

    // ==================== Token 新鲜度测试 ====================

    #[test]
    fn test_calculate_freshness() {
        let (service, _ccr, _codex) = create_test_service();

        // Fresh: < 1 day
        let fresh_time = Utc::now() - Duration::hours(12);
        assert_eq!(
            service.calculate_freshness(Some(fresh_time)),
            TokenFreshness::Fresh
        );

        // Stale: 1-7 days
        let stale_time = Utc::now() - Duration::days(3);
        assert_eq!(
            service.calculate_freshness(Some(stale_time)),
            TokenFreshness::Stale
        );

        // Old: > 7 days
        let old_time = Utc::now() - Duration::days(10);
        assert_eq!(
            service.calculate_freshness(Some(old_time)),
            TokenFreshness::Old
        );

        // Unknown
        assert_eq!(service.calculate_freshness(None), TokenFreshness::Unknown);
    }

    #[test]
    fn test_calculate_freshness_boundary() {
        let (service, _ccr, _codex) = create_test_service();

        // 刚好 1 天 - 应该是 Stale
        let one_day = Utc::now() - Duration::days(1);
        assert_eq!(
            service.calculate_freshness(Some(one_day)),
            TokenFreshness::Stale
        );

        // 刚好 7 天 - 应该是 Old
        let seven_days = Utc::now() - Duration::days(7);
        assert_eq!(
            service.calculate_freshness(Some(seven_days)),
            TokenFreshness::Old
        );

        // 刚刚刷新 - 应该是 Fresh
        let just_now = Utc::now();
        assert_eq!(
            service.calculate_freshness(Some(just_now)),
            TokenFreshness::Fresh
        );
    }

    // ==================== 注册表测试 ====================

    #[test]
    fn test_registry_default() {
        let registry = CodexAuthRegistry::default();
        assert_eq!(registry.version, "1.0");
        assert!(registry.current_auth.is_none());
        assert!(registry.accounts.is_empty());
    }

    #[test]
    fn test_registry_serialization() {
        let mut registry = CodexAuthRegistry {
            current_auth: Some("test-account".to_string()),
            ..Default::default()
        };
        registry.accounts.insert(
            "test-account".to_string(),
            CodexAuthAccount {
                description: Some("Test".to_string()),
                account_id: "acc-123".to_string(),
                email: Some("tes***@example.com".to_string()),
                saved_at: Utc::now(),
                last_used: None,
                last_refresh: None,
                expires_at: None,
            },
        );

        // 序列化
        let toml_str = toml::to_string_pretty(&registry).unwrap();
        assert!(toml_str.contains("test-account"));
        assert!(toml_str.contains("acc-123"));

        // 反序列化
        let parsed: CodexAuthRegistry = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.current_auth, Some("test-account".to_string()));
        assert!(parsed.accounts.contains_key("test-account"));
    }

    // ==================== 登录状态测试 ====================

    #[test]
    fn test_is_logged_in_no_file() {
        let (service, _ccr, _codex) = create_test_service();
        assert!(!service.is_logged_in());
    }

    #[test]
    fn test_is_logged_in_with_valid_auth() {
        let (service, _ccr, codex) = create_test_service();

        // 创建有效的 auth.json
        let auth_path = codex.path().join("auth.json");
        let auth_content = create_test_auth_json("test-id", "2026-01-08T03:09:53.894843900Z");
        fs::write(&auth_path, auth_content).unwrap();

        assert!(service.is_logged_in());
    }

    #[test]
    fn test_is_logged_in_with_invalid_json() {
        let (service, _ccr, codex) = create_test_service();

        // 创建无效的 auth.json
        let auth_path = codex.path().join("auth.json");
        fs::write(&auth_path, "invalid json content").unwrap();

        assert!(!service.is_logged_in());
    }

    #[test]
    fn test_get_login_state_not_logged_in() {
        let (service, _ccr, _codex) = create_test_service();
        let state = service.get_login_state().unwrap();
        assert_eq!(state, LoginState::NotLoggedIn);
    }

    #[test]
    fn test_get_login_state_logged_in_unsaved() {
        let (service, _ccr, codex) = create_test_service();

        // 创建 auth.json 但不保存到注册表
        let auth_path = codex.path().join("auth.json");
        let auth_content = create_test_auth_json("test-id", "2026-01-08T03:09:53.894843900Z");
        fs::write(&auth_path, auth_content).unwrap();

        let state = service.get_login_state().unwrap();
        assert_eq!(state, LoginState::LoggedInUnsaved);
    }

    // ==================== 账号管理工作流测试 ====================

    #[test]
    fn test_save_switch_delete_workflow() {
        let (service, _ccr, codex) = create_test_service();

        // 1. 创建 auth.json
        let auth_path = codex.path().join("auth.json");
        let auth_content = create_test_auth_json("test-id-1", "2026-01-08T03:09:53.894843900Z");
        fs::write(&auth_path, auth_content).unwrap();

        // 2. 保存账号
        service
            .save_current("account1", Some("First account".to_string()), None, false)
            .unwrap();

        // 验证保存成功
        let accounts = service.list_accounts().unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].name, "account1");
        assert!(!accounts[0].is_virtual);

        // 3. 创建第二个 auth.json 并保存
        let auth_content2 = create_test_auth_json("test-id-2", "2026-01-09T03:09:53.894843900Z");
        fs::write(&auth_path, auth_content2).unwrap();

        service
            .save_current("account2", Some("Second account".to_string()), None, false)
            .unwrap();

        // 验证两个账号
        let accounts = service.list_accounts().unwrap();
        assert_eq!(accounts.len(), 2);

        // 4. 切换到 account1
        service.switch_account("account1").unwrap();

        // 验证切换成功
        let state = service.get_login_state().unwrap();
        assert_eq!(state, LoginState::LoggedInSaved("account1".to_string()));

        // 5. 删除 account2
        service.delete_account("account2").unwrap();

        // 验证删除成功
        let accounts = service.list_accounts().unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].name, "account1");
    }

    #[test]
    fn test_save_duplicate_without_force() {
        let (service, _ccr, codex) = create_test_service();

        // 创建 auth.json
        let auth_path = codex.path().join("auth.json");
        let auth_content = create_test_auth_json("test-id", "2026-01-08T03:09:53.894843900Z");
        fs::write(&auth_path, auth_content).unwrap();

        // 第一次保存
        service
            .save_current("myaccount", None, None, false)
            .unwrap();

        // 第二次保存同名 - 应该失败
        let result = service.save_current("myaccount", None, None, false);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("已存在"));
    }

    #[test]
    fn test_save_duplicate_with_force() {
        let (service, _ccr, codex) = create_test_service();

        // 创建 auth.json
        let auth_path = codex.path().join("auth.json");
        let auth_content = create_test_auth_json("test-id", "2026-01-08T03:09:53.894843900Z");
        fs::write(&auth_path, auth_content).unwrap();

        // 第一次保存
        service
            .save_current("myaccount", None, None, false)
            .unwrap();

        // 第二次保存同名 with force - 应该成功
        let result = service.save_current("myaccount", Some("Updated".to_string()), None, true);
        assert!(result.is_ok());

        // 验证描述已更新
        let accounts = service.list_accounts().unwrap();
        assert_eq!(accounts[0].description, Some("Updated".to_string()));
    }

    #[test]
    fn test_switch_nonexistent_account() {
        let (service, _ccr, _codex) = create_test_service();

        let result = service.switch_account("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("不存在"));
    }

    #[test]
    fn test_delete_nonexistent_account() {
        let (service, _ccr, _codex) = create_test_service();

        let result = service.delete_account("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("不存在"));
    }

    // ==================== 虚拟 default 账号测试 ====================

    #[test]
    fn test_virtual_default_account() {
        let (service, _ccr, codex) = create_test_service();

        // 创建 auth.json 但不保存
        let auth_path = codex.path().join("auth.json");
        let auth_content = create_test_auth_json("test-id", "2026-01-08T03:09:53.894843900Z");
        fs::write(&auth_path, auth_content).unwrap();

        // 列出账号 - 应该有虚拟 default
        let accounts = service.list_accounts().unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].name, "default");
        assert!(accounts[0].is_virtual);
        assert!(accounts[0].is_current);
    }

    #[test]
    fn test_no_virtual_default_when_saved() {
        let (service, _ccr, codex) = create_test_service();

        // 创建 auth.json
        let auth_path = codex.path().join("auth.json");
        let auth_content = create_test_auth_json("test-id", "2026-01-08T03:09:53.894843900Z");
        fs::write(&auth_path, auth_content).unwrap();

        // 保存账号
        service
            .save_current("myaccount", None, None, false)
            .unwrap();

        // 列出账号 - 不应该有虚拟 default
        let accounts = service.list_accounts().unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].name, "myaccount");
        assert!(!accounts[0].is_virtual);
    }

    // ==================== 备份测试 ====================

    #[test]
    fn test_backup_current_auth() {
        let (service, _ccr, codex) = create_test_service();

        // 创建 auth.json
        let auth_path = codex.path().join("auth.json");
        let auth_content = create_test_auth_json("test-id", "2026-01-08T03:09:53.894843900Z");
        fs::write(&auth_path, auth_content).unwrap();

        // 备份
        let backup_path = service.backup_current_auth().unwrap();
        assert!(backup_path.exists());
        assert!(backup_path.to_string_lossy().contains("auth_"));
    }

    #[test]
    fn test_backup_rotation() {
        let (service, ccr, codex) = create_test_service();

        // 创建 auth.json
        let auth_path = codex.path().join("auth.json");
        let auth_content = create_test_auth_json("test-id", "2026-01-08T03:09:53.894843900Z");
        fs::write(&auth_path, auth_content).unwrap();

        // 创建备份目录和 15 个旧备份
        let backup_dir = ccr.path().join("auth/backups");
        fs::create_dir_all(&backup_dir).unwrap();

        for i in 0..15 {
            let backup_name = format!("auth_20260101_{:06}.json", i);
            fs::write(backup_dir.join(&backup_name), "{}").unwrap();
        }

        // 执行新备份 (会触发清理)
        service.backup_current_auth().unwrap();

        // 验证只保留 MAX_BACKUPS 个
        let backups: Vec<_> = fs::read_dir(&backup_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert!(backups.len() <= MAX_BACKUPS + 1); // +1 for the new backup
    }

    // ==================== JWT 解析测试 ====================

    #[test]
    fn test_base64url_decode() {
        let (service, _ccr, _codex) = create_test_service();

        // 标准 base64url 编码的 "test"
        let decoded = service.base64url_decode("dGVzdA").unwrap();
        assert_eq!(decoded, b"test");

        // 带 padding 的情况
        let decoded2 = service.base64url_decode("dGVzdA==").unwrap();
        assert_eq!(decoded2, b"test");
    }

    #[test]
    fn test_extract_email_from_jwt() {
        let (service, _ccr, _codex) = create_test_service();

        // 创建包含 email 的 JWT payload
        // {"email":"test@example.com","sub":"1234567890"}
        // Base64URL: eyJlbWFpbCI6InRlc3RAZXhhbXBsZS5jb20iLCJzdWIiOiIxMjM0NTY3ODkwIn0
        let auth = CodexAuthJson {
            openai_api_key: None,
            tokens: Some(CodexAuthTokens {
                id_token: Some("eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJlbWFpbCI6InRlc3RAZXhhbXBsZS5jb20iLCJzdWIiOiIxMjM0NTY3ODkwIn0.signature".to_string()),
                access_token: None,
                refresh_token: None,
                account_id: Some("test-id".to_string()),
            }),
            last_refresh: None,
        };

        let email = service.extract_email_from_jwt(&auth);
        assert_eq!(email, Some("test@example.com".to_string()));
    }

    #[test]
    fn test_extract_email_no_token() {
        let (service, _ccr, _codex) = create_test_service();

        let auth = CodexAuthJson {
            openai_api_key: None,
            tokens: None,
            last_refresh: None,
        };

        let email = service.extract_email_from_jwt(&auth);
        assert!(email.is_none());
    }

    // ==================== 进程检测测试 ====================

    #[test]
    fn test_detect_codex_process() {
        let (service, _ccr, _codex) = create_test_service();

        // 这个测试主要验证函数不会 panic
        // 实际检测结果取决于系统状态
        let pids = service.detect_codex_process();
        // 返回类型正确即可
        assert!(pids.is_empty() || !pids.is_empty());
    }

    // ==================== 账号过期测试 ====================

    #[test]
    fn test_is_expired_none() {
        // None 不视为过期
        assert!(!CodexAuthService::is_expired(None));
    }

    #[test]
    fn test_is_expired_future() {
        // 未来时间不过期
        let future = Utc::now() + Duration::days(30);
        assert!(!CodexAuthService::is_expired(Some(future)));
    }

    #[test]
    fn test_is_expired_past() {
        // 过去时间已过期
        let past = Utc::now() - Duration::days(1);
        assert!(CodexAuthService::is_expired(Some(past)));
    }

    #[test]
    fn test_is_expired_boundary() {
        // 刚好现在 - 应该视为过期
        let now = Utc::now();
        assert!(CodexAuthService::is_expired(Some(now)));
    }

    #[test]
    fn test_registry_with_expiry_serialization() {
        let mut registry = CodexAuthRegistry {
            current_auth: Some("test-account".to_string()),
            ..Default::default()
        };

        let expires_at = Utc::now() + Duration::days(30);
        registry.accounts.insert(
            "test-account".to_string(),
            CodexAuthAccount {
                description: Some("Test".to_string()),
                account_id: "acc-123".to_string(),
                email: Some("tes***@example.com".to_string()),
                saved_at: Utc::now(),
                last_used: None,
                last_refresh: None,
                expires_at: Some(expires_at),
            },
        );

        // 序列化
        let toml_str = toml::to_string_pretty(&registry).unwrap();
        assert!(toml_str.contains("expires_at"));

        // 反序列化
        let parsed: CodexAuthRegistry = toml::from_str(&toml_str).unwrap();
        let account = parsed.accounts.get("test-account").unwrap();
        assert!(account.expires_at.is_some());
    }

    #[test]
    fn test_registry_without_expiry_serialization() {
        let mut registry = CodexAuthRegistry {
            current_auth: Some("test-account".to_string()),
            ..Default::default()
        };

        registry.accounts.insert(
            "test-account".to_string(),
            CodexAuthAccount {
                description: Some("Test".to_string()),
                account_id: "acc-123".to_string(),
                email: Some("tes***@example.com".to_string()),
                saved_at: Utc::now(),
                last_used: None,
                last_refresh: None,
                expires_at: None,
            },
        );

        // 序列化时 None 应该被跳过
        let toml_str = toml::to_string_pretty(&registry).unwrap();
        assert!(!toml_str.contains("expires_at"));

        // 反序列化
        let parsed: CodexAuthRegistry = toml::from_str(&toml_str).unwrap();
        let account = parsed.accounts.get("test-account").unwrap();
        assert!(account.expires_at.is_none());
    }

    #[test]
    fn test_switch_to_expired_account_blocked() {
        let (service, _ccr, codex) = create_test_service();

        // 创建 auth.json
        let auth_path = codex.path().join("auth.json");
        let auth_content = create_test_auth_json("test-id", "2026-01-08T03:09:53.894843900Z");
        fs::write(&auth_path, auth_content).unwrap();

        // 保存账号并设置过期时间为过去
        let past = Utc::now() - Duration::days(1);
        service
            .save_current(
                "expired-account",
                Some("Expired".to_string()),
                Some(past),
                false,
            )
            .unwrap();

        // 创建另一个 auth.json 以便切换
        let auth_content2 = create_test_auth_json("test-id-2", "2026-01-09T03:09:53.894843900Z");
        fs::write(&auth_path, auth_content2).unwrap();

        // 尝试切换到过期账号 - 应该失败
        let result = service.switch_account("expired-account");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("已过期"));
    }

    #[test]
    fn test_switch_to_non_expired_account_allowed() {
        let (service, _ccr, codex) = create_test_service();

        // 创建 auth.json
        let auth_path = codex.path().join("auth.json");
        let auth_content = create_test_auth_json("test-id", "2026-01-08T03:09:53.894843900Z");
        fs::write(&auth_path, auth_content).unwrap();

        // 保存账号并设置过期时间为未来
        let future = Utc::now() + Duration::days(30);
        service
            .save_current(
                "valid-account",
                Some("Valid".to_string()),
                Some(future),
                false,
            )
            .unwrap();

        // 创建另一个 auth.json 以便切换
        let auth_content2 = create_test_auth_json("test-id-2", "2026-01-09T03:09:53.894843900Z");
        fs::write(&auth_path, auth_content2).unwrap();

        // 切换到未过期账号 - 应该成功
        let result = service.switch_account("valid-account");
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_accounts_includes_expiry() {
        let (service, _ccr, codex) = create_test_service();

        // 创建 auth.json
        let auth_path = codex.path().join("auth.json");
        let auth_content = create_test_auth_json("test-id", "2026-01-08T03:09:53.894843900Z");
        fs::write(&auth_path, auth_content).unwrap();

        // 保存账号并设置过期时间
        let future = Utc::now() + Duration::days(30);
        service
            .save_current("with-expiry", None, Some(future), false)
            .unwrap();

        // 列出账号
        let accounts = service.list_accounts().unwrap();
        let account = accounts.iter().find(|a| a.name == "with-expiry").unwrap();
        assert!(account.expires_at.is_some());
    }

    // ==================== 导入账号测试 ====================

    #[test]
    fn test_import_accounts_merge_without_force() {
        let (service, _ccr, _codex) = create_test_service();

        // 先保存一个账号
        let auth_path = service.codex_dir.join("auth.json");
        let auth_content = create_test_auth_json("existing-id", "2026-01-08T03:09:53.894843900Z");
        fs::write(&auth_path, auth_content).unwrap();
        service
            .save_current(
                "existing",
                Some("Existing account".to_string()),
                None,
                false,
            )
            .unwrap();

        // 准备导入数据（包含同名账号和新账号）
        let import_json = r#"{
            "version": "1.0",
            "exported_at": "2026-01-22T00:00:00Z",
            "accounts": {
                "existing": {
                    "description": "Updated description",
                    "account_id": "new-id",
                    "email": "new***@example.com",
                    "saved_at": "2026-01-22T00:00:00Z",
                    "auth_data": {
                        "tokens": {
                            "id_token": "new_token",
                            "access_token": "new_access",
                            "refresh_token": "new_refresh",
                            "account_id": "new-id"
                        },
                        "last_refresh": "2026-01-22T00:00:00Z"
                    }
                },
                "new-account": {
                    "description": "New account",
                    "account_id": "new-account-id",
                    "email": "new***@example.com",
                    "saved_at": "2026-01-22T00:00:00Z"
                }
            }
        }"#;

        // 合并模式，不强制覆盖
        let result = service
            .import_accounts(import_json, ImportMode::Merge, false)
            .unwrap();

        // 验证结果
        assert_eq!(result.added, 1); // new-account
        assert_eq!(result.updated, 0);
        assert_eq!(result.skipped, 1); // existing
        assert_eq!(result.overwritten.len(), 0);

        // 验证 registry 中的账号数量
        let registry = service.load_registry().unwrap();
        assert_eq!(registry.accounts.len(), 2);

        // 验证 existing 账号没有被更新
        let existing = registry.accounts.get("existing").unwrap();
        assert_eq!(existing.account_id, "existing-id");
        assert_eq!(existing.description, Some("Existing account".to_string()));
    }

    #[test]
    fn test_import_accounts_merge_with_force() {
        let (service, _ccr, _codex) = create_test_service();

        // 先保存一个账号
        let auth_path = service.codex_dir.join("auth.json");
        let auth_content = create_test_auth_json("existing-id", "2026-01-08T03:09:53.894843900Z");
        fs::write(&auth_path, auth_content).unwrap();
        service
            .save_current(
                "existing",
                Some("Existing account".to_string()),
                None,
                false,
            )
            .unwrap();

        // 准备导入数据
        let import_json = r#"{
            "version": "1.0",
            "exported_at": "2026-01-22T00:00:00Z",
            "accounts": {
                "existing": {
                    "description": "Updated description",
                    "account_id": "new-id",
                    "email": "new***@example.com",
                    "saved_at": "2026-01-22T00:00:00Z",
                    "auth_data": {
                        "tokens": {
                            "id_token": "new_token",
                            "access_token": "new_access",
                            "refresh_token": "new_refresh",
                            "account_id": "new-id"
                        },
                        "last_refresh": "2026-01-22T00:00:00Z"
                    }
                },
                "new-account": {
                    "description": "New account",
                    "account_id": "new-account-id",
                    "email": "new***@example.com",
                    "saved_at": "2026-01-22T00:00:00Z"
                }
            }
        }"#;

        // 合并模式，强制覆盖
        let result = service
            .import_accounts(import_json, ImportMode::Merge, true)
            .unwrap();

        // 验证结果
        assert_eq!(result.added, 1); // new-account
        assert_eq!(result.updated, 1); // existing 被更新
        assert_eq!(result.skipped, 0);
        assert_eq!(result.overwritten.len(), 1);
        assert_eq!(result.overwritten[0], "existing");

        // 验证 registry 中的账号数量
        let registry = service.load_registry().unwrap();
        assert_eq!(registry.accounts.len(), 2);

        // 验证 existing 账号已被更新
        let existing = registry.accounts.get("existing").unwrap();
        assert_eq!(existing.account_id, "new-id");
        assert_eq!(
            existing.description,
            Some("Updated description".to_string())
        );
    }

    #[test]
    fn test_import_accounts_force_creates_backups() {
        let (service, _ccr, _codex) = create_test_service();

        let auth_path = service.codex_dir.join("auth.json");
        let auth_content = create_test_auth_json("existing-id", "2026-01-08T03:09:53.894843900Z");
        fs::write(&auth_path, auth_content).unwrap();
        service
            .save_current(
                "existing",
                Some("Existing account".to_string()),
                None,
                false,
            )
            .unwrap();

        let import_json = r#"{
            "version": "1.0",
            "exported_at": "2026-01-22T00:00:00Z",
            "accounts": {
                "existing": {
                    "description": "Updated description",
                    "account_id": "new-id",
                    "email": "new***@example.com",
                    "saved_at": "2026-01-22T00:00:00Z",
                    "auth_data": {
                        "tokens": {
                            "id_token": "new_token",
                            "access_token": "new_access",
                            "refresh_token": "new_refresh",
                            "account_id": "new-id"
                        },
                        "last_refresh": "2026-01-22T00:00:00Z"
                    }
                }
            }
        }"#;

        service
            .import_accounts(import_json, ImportMode::Merge, true)
            .unwrap();

        let stored_auth_path = service.account_auth_path("existing");
        let stored_auth = fs::read_to_string(&stored_auth_path).unwrap();
        assert!(stored_auth.contains("new_access"));

        let backup_dir = service.backup_dir();
        let backups: Vec<_> = fs::read_dir(&backup_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();

        let has_registry_backup = backups.iter().any(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("auth_registry_")
        });
        let has_account_backup = backups.iter().any(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("auth_account_existing_")
        });

        assert!(has_registry_backup);
        assert!(has_account_backup);
    }

    #[test]
    fn test_import_accounts_replace_mode() {
        let (service, _ccr, _codex) = create_test_service();

        // 先保存一个账号
        let auth_path = service.codex_dir.join("auth.json");
        let auth_content = create_test_auth_json("existing-id", "2026-01-08T03:09:53.894843900Z");
        fs::write(&auth_path, auth_content).unwrap();
        service
            .save_current(
                "existing",
                Some("Existing account".to_string()),
                None,
                false,
            )
            .unwrap();

        // 准备导入数据
        let import_json = r#"{
            "version": "1.0",
            "exported_at": "2026-01-22T00:00:00Z",
            "accounts": {
                "existing": {
                    "description": "Replaced description",
                    "account_id": "replaced-id",
                    "email": "rep***@example.com",
                    "saved_at": "2026-01-22T00:00:00Z"
                }
            }
        }"#;

        // 替换模式（force 参数在 Replace 模式下被忽略）
        let result = service
            .import_accounts(import_json, ImportMode::Replace, false)
            .unwrap();

        // 验证结果
        assert_eq!(result.added, 0);
        assert_eq!(result.updated, 1);
        assert_eq!(result.skipped, 0);
        assert_eq!(result.overwritten.len(), 1);
        assert_eq!(result.overwritten[0], "existing");

        // 验证账号已被替换 - 从 registry 读取
        let registry = service.load_registry().unwrap();
        let existing = registry.accounts.get("existing").unwrap();
        assert_eq!(existing.account_id, "replaced-id");
        assert_eq!(
            existing.description,
            Some("Replaced description".to_string())
        );
    }

    #[test]
    fn test_import_accounts_invalid_name() {
        let (service, _ccr, _codex) = create_test_service();

        // 准备包含无效账号名的导入数据
        let import_json = r#"{
            "version": "1.0",
            "exported_at": "2026-01-22T00:00:00Z",
            "accounts": {
                "invalid name": {
                    "description": "Invalid",
                    "account_id": "test-id",
                    "email": "test***@example.com",
                    "saved_at": "2026-01-22T00:00:00Z"
                }
            }
        }"#;

        // 应该返回错误
        let result = service.import_accounts(import_json, ImportMode::Merge, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_import_accounts_invalid_json() {
        let (service, _ccr, _codex) = create_test_service();

        // 无效的 JSON
        let invalid_json = "{ invalid json }";

        // 应该返回错误
        let result = service.import_accounts(invalid_json, ImportMode::Merge, false);
        assert!(result.is_err());
    }
}
