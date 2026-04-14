// 🔐 OpenCode Auth 服务层
// 管理 OpenCode 的 openai provider 多账号手动切换

use crate::models::{
    OpenCodeAuthAccount, OpenCodeAuthItem, OpenCodeAuthRegistry, OpenCodeCurrentAuthInfo,
    OpenCodeLoginState, OpenCodeOpenAiAuth, OpenCodeReadSnapshot, TokenFreshness,
};
use crate::utils::{OpenCodePaths, decode_base64url, ensure_private_permissions};
use ccr_core::core::atomic_writer::AtomicWriter;
use ccr_core::core::error::{CcrError, Result};
use chrono::{DateTime, Duration, TimeZone, Utc};
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::fs;
use std::path::{Path, PathBuf};

/// OpenCode Auth 服务
pub struct OpenCodeAuthService {
    /// CCR 平台数据目录 (~/.ccr/platforms/opencode/)
    ccr_opencode_dir: PathBuf,
    /// OpenCode 数据目录（官方默认：`$HOME/.local/share/opencode/`）
    opencode_dir: PathBuf,
}

impl OpenCodeAuthService {
    /// 创建新的 OpenCodeAuthService 实例
    pub fn new() -> Result<Self> {
        let paths = OpenCodePaths::resolve()?;
        Ok(Self {
            ccr_opencode_dir: paths.ccr_opencode_dir,
            opencode_dir: paths.opencode_dir,
        })
    }

    pub fn from_dirs(ccr_opencode_dir: PathBuf, opencode_dir: PathBuf) -> Self {
        Self {
            ccr_opencode_dir,
            opencode_dir,
        }
    }

    pub fn opencode_dir(&self) -> &Path {
        &self.opencode_dir
    }

    fn auth_json_path(&self) -> PathBuf {
        self.opencode_dir.join("auth.json")
    }

    fn auth_storage_dir(&self) -> PathBuf {
        self.ccr_opencode_dir.join("auth")
    }

    fn registry_path(&self) -> PathBuf {
        self.ccr_opencode_dir.join("auth_registry.toml")
    }

    fn account_auth_path(&self, name: &str) -> PathBuf {
        self.auth_storage_dir().join(format!("{name}.json"))
    }

    fn ensure_storage_dirs(&self) -> Result<()> {
        fs::create_dir_all(&self.ccr_opencode_dir)
            .map_err(|e| CcrError::ConfigError(format!("创建 OpenCode 平台目录失败: {e}")))?;
        fs::create_dir_all(self.auth_storage_dir())
            .map_err(|e| CcrError::ConfigError(format!("创建 OpenCode auth 目录失败: {e}")))?;
        Ok(())
    }

    fn load_auth_root_map(&self) -> Result<JsonMap<String, JsonValue>> {
        let auth_path = self.auth_json_path();
        let content = fs::read_to_string(&auth_path)
            .map_err(|e| CcrError::ConfigError(format!("读取 OpenCode auth.json 失败: {e}")))?;
        serde_json::from_str::<JsonMap<String, JsonValue>>(&content)
            .map_err(|e| CcrError::ConfigError(format!("解析 OpenCode auth.json 失败: {e}")))
    }

    fn load_auth_root_map_or_default(&self) -> Result<JsonMap<String, JsonValue>> {
        let auth_path = self.auth_json_path();
        if !auth_path.exists() {
            return Ok(JsonMap::new());
        }
        self.load_auth_root_map()
    }

    fn current_openai_value(&self) -> Result<Option<JsonValue>> {
        let auth_path = self.auth_json_path();
        if !auth_path.exists() {
            return Ok(None);
        }

        let root = self.load_auth_root_map()?;
        Ok(root.get("openai").cloned().filter(|value| !value.is_null()))
    }

    /// 读取当前 OpenCode 登录快照
    pub fn read_auth_snapshot(&self) -> Result<OpenCodeReadSnapshot> {
        let registry = self.load_registry()?;
        let current_value = self.current_openai_value()?;
        let current_info = match current_value {
            Some(ref openai) => Some(self.extract_current_auth_info(openai)?),
            None => None,
        };
        let current_account_name =
            Self::matched_saved_account_name(&registry, current_info.as_ref());
        let current_expires_at = current_info.as_ref().and_then(|info| info.expires_at);
        let login_state = match (current_info.as_ref(), current_account_name.as_ref()) {
            (Some(_), Some(name)) => OpenCodeLoginState::LoggedInSaved(name.clone()),
            (Some(_), None) => OpenCodeLoginState::LoggedInUnsaved,
            (None, _) => OpenCodeLoginState::NotLoggedIn,
        };

        Ok(OpenCodeReadSnapshot {
            login_state,
            current_info,
            registry,
            current_account_name,
            current_expires_at,
        })
    }

    /// 构建 TUI 展示列表
    pub fn build_account_items(
        &self,
        snapshot: &OpenCodeReadSnapshot,
    ) -> Result<Vec<OpenCodeAuthItem>> {
        let mut items = Vec::new();

        if let Some(info) = snapshot.current_info.as_ref()
            && snapshot.current_account_name.is_none()
        {
            items.push(OpenCodeAuthItem {
                name: "current-login".to_string(),
                account_id: Some(info.account_id.clone()),
                email: info.email.as_ref().map(|email| self.mask_email(email)),
                plan_type: info.plan_type.clone(),
                is_current: true,
                is_virtual: true,
                saved_at: None,
                last_used: None,
                freshness: info.freshness.clone(),
                expires_at: info.expires_at,
            });
        }

        let mut saved_items: Vec<_> = snapshot
            .registry
            .accounts
            .iter()
            .map(|(name, account)| {
                let is_current = snapshot.current_account_name.as_deref() == Some(name.as_str());
                OpenCodeAuthItem {
                    name: name.clone(),
                    account_id: Some(account.account_id.clone()),
                    email: account.email.clone(),
                    plan_type: account.plan_type.clone(),
                    is_current,
                    is_virtual: false,
                    saved_at: Some(account.saved_at),
                    last_used: account.last_used,
                    freshness: if is_current {
                        snapshot
                            .current_info
                            .as_ref()
                            .map(|info| info.freshness.clone())
                            .unwrap_or_else(|| self.calculate_freshness(account.expires_at))
                    } else {
                        self.calculate_freshness(account.expires_at)
                    },
                    expires_at: account.expires_at,
                }
            })
            .collect();

        saved_items.sort_by(|left, right| {
            right
                .is_current
                .cmp(&left.is_current)
                .then_with(|| left.name.cmp(&right.name))
        });

        items.extend(saved_items);
        Ok(items)
    }

    /// 保存当前 openai 登录为一个可切换账号
    pub fn save_current(&self, name: &str, force: bool) -> Result<()> {
        self.validate_account_name(name)?;
        self.ensure_storage_dirs()?;

        let current_value = self
            .current_openai_value()?
            .ok_or_else(|| CcrError::ConfigError("当前 OpenCode 未检测到 openai 登录".into()))?;
        let current_info = self.extract_current_auth_info(&current_value)?;

        let mut registry = self.load_registry()?;
        if registry.accounts.contains_key(name) && !force {
            return Err(CcrError::ResourceAlreadyExists(format!(
                "OpenCode auth account '{name}'"
            )));
        }

        let now = Utc::now();
        let saved_at = registry
            .accounts
            .get(name)
            .map(|account| account.saved_at)
            .unwrap_or(now);

        registry.accounts.insert(
            name.to_string(),
            OpenCodeAuthAccount {
                account_id: current_info.account_id.clone(),
                email: current_info
                    .email
                    .as_ref()
                    .map(|email| self.mask_email(email)),
                plan_type: current_info.plan_type.clone(),
                saved_at,
                last_used: Some(now),
                expires_at: current_info.expires_at,
            },
        );
        registry.current_auth = Some(name.to_string());

        self.write_account_snapshot(name, &current_value)?;
        self.save_registry(&registry)
    }

    /// 切换到指定已保存账号
    pub fn switch_account(&self, name: &str) -> Result<()> {
        let mut registry = self.load_registry()?;
        let account =
            registry.accounts.get(name).cloned().ok_or_else(|| {
                CcrError::ResourceNotFound(format!("OpenCode auth account '{name}'"))
            })?;

        if Self::is_expired(account.expires_at) {
            return Err(CcrError::ValidationError(format!(
                "账号 '{name}' 已过期，请重新登录 OpenCode 后再保存"
            )));
        }

        let snapshot_path = self.account_auth_path(name);
        if !snapshot_path.exists() {
            return Err(CcrError::ResourceNotFound(format!(
                "OpenCode auth snapshot '{}'",
                snapshot_path.display()
            )));
        }

        let snapshot_content = fs::read_to_string(&snapshot_path)
            .map_err(|e| CcrError::ConfigError(format!("读取 OpenCode 账号快照失败: {e}")))?;
        let saved_openai: JsonValue = serde_json::from_str(&snapshot_content)
            .map_err(|e| CcrError::ConfigError(format!("解析 OpenCode 账号快照失败: {e}")))?;

        let mut root = self.load_auth_root_map_or_default()?;
        root.insert("openai".to_string(), saved_openai);
        self.write_auth_root_map(&root)?;

        registry.current_auth = Some(name.to_string());
        if let Some(account) = registry.accounts.get_mut(name) {
            account.last_used = Some(Utc::now());
        }
        self.save_registry(&registry)
    }

    /// 删除指定已保存账号
    pub fn delete_account(&self, name: &str) -> Result<()> {
        let mut registry = self.load_registry()?;
        if registry.accounts.shift_remove(name).is_none() {
            return Err(CcrError::ResourceNotFound(format!(
                "OpenCode auth account '{name}'"
            )));
        }

        if registry.current_auth.as_deref() == Some(name) {
            registry.current_auth = None;
        }

        let snapshot_path = self.account_auth_path(name);
        if snapshot_path.exists() {
            fs::remove_file(&snapshot_path)
                .map_err(|e| CcrError::ConfigError(format!("删除 OpenCode 账号快照失败: {e}")))?;
        }

        self.save_registry(&registry)
    }

    /// 加载注册表
    pub fn load_registry(&self) -> Result<OpenCodeAuthRegistry> {
        let registry_path = self.registry_path();
        if !registry_path.exists() {
            return Ok(OpenCodeAuthRegistry::default());
        }

        let content = fs::read_to_string(&registry_path)
            .map_err(|e| CcrError::ConfigError(format!("读取 OpenCode 注册表失败: {e}")))?;
        toml::from_str(&content)
            .map_err(|e| CcrError::ConfigError(format!("解析 OpenCode 注册表失败: {e}")))
    }

    fn save_registry(&self, registry: &OpenCodeAuthRegistry) -> Result<()> {
        self.ensure_storage_dirs()?;
        let content = toml::to_string_pretty(registry)
            .map_err(|e| CcrError::ConfigError(format!("序列化 OpenCode 注册表失败: {e}")))?;
        let path = self.registry_path();
        AtomicWriter::new(&path)
            .write_string(&content)
            .map_err(|e| CcrError::ConfigError(format!("写入 OpenCode 注册表失败: {e}")))?;
        ensure_private_permissions(&path);
        Ok(())
    }

    fn write_account_snapshot(&self, name: &str, openai: &JsonValue) -> Result<()> {
        self.ensure_storage_dirs()?;
        let content = serde_json::to_string_pretty(openai)
            .map_err(|e| CcrError::ConfigError(format!("序列化 OpenCode 账号快照失败: {e}")))?;
        let path = self.account_auth_path(name);
        AtomicWriter::new(&path)
            .write_string(&content)
            .map_err(|e| CcrError::ConfigError(format!("写入 OpenCode 账号快照失败: {e}")))?;
        ensure_private_permissions(&path);
        Ok(())
    }

    fn write_auth_root_map(&self, root: &JsonMap<String, JsonValue>) -> Result<()> {
        if let Some(parent) = self.auth_json_path().parent() {
            fs::create_dir_all(parent)
                .map_err(|e| CcrError::ConfigError(format!("创建 OpenCode 配置目录失败: {e}")))?;
        }

        let content = serde_json::to_string_pretty(root)
            .map_err(|e| CcrError::ConfigError(format!("序列化 OpenCode auth.json 失败: {e}")))?;
        let path = self.auth_json_path();
        AtomicWriter::new(&path)
            .write_string(&content)
            .map_err(|e| CcrError::ConfigError(format!("写入 OpenCode auth.json 失败: {e}")))?;
        ensure_private_permissions(&path);
        Ok(())
    }

    fn extract_current_auth_info(&self, openai: &JsonValue) -> Result<OpenCodeCurrentAuthInfo> {
        let auth: OpenCodeOpenAiAuth = serde_json::from_value(openai.clone()).map_err(|e| {
            CcrError::ConfigError(format!("解析 OpenCode openai provider 失败: {e}"))
        })?;

        let claims = auth.access.as_deref().and_then(Self::decode_jwt_claims);
        let account_id = auth
            .account_id
            .clone()
            .or_else(|| Self::claim_string(claims.as_ref(), "chatgpt_account_id"))
            .or_else(|| Self::claim_string(claims.as_ref(), "account_id"))
            .or_else(|| Self::claim_string(claims.as_ref(), "sub"))
            .ok_or_else(|| CcrError::ConfigError("当前 OpenCode 登录缺少 account_id".into()))?;

        let email = auth
            .extra
            .get("email")
            .and_then(JsonValue::as_str)
            .map(str::to_string)
            .or_else(|| Self::claim_string(claims.as_ref(), "email"));

        let plan_type = auth
            .extra
            .get("plan")
            .and_then(JsonValue::as_str)
            .map(Self::normalize_plan)
            .or_else(|| {
                auth.extra
                    .get("planType")
                    .and_then(JsonValue::as_str)
                    .map(Self::normalize_plan)
            })
            .or_else(|| {
                Self::claim_string(claims.as_ref(), "chatgpt_plan_type")
                    .map(|value| Self::normalize_plan(value.as_str()))
            })
            .or_else(|| {
                Self::claim_string(claims.as_ref(), "plan")
                    .map(|value| Self::normalize_plan(value.as_str()))
            });

        let expires_at = auth.expires.and_then(Self::unix_millis_to_datetime);
        let freshness = self.calculate_freshness(expires_at);

        Ok(OpenCodeCurrentAuthInfo {
            account_id,
            email,
            plan_type,
            freshness,
            expires_at,
        })
    }

    fn matched_saved_account_name(
        registry: &OpenCodeAuthRegistry,
        current_info: Option<&OpenCodeCurrentAuthInfo>,
    ) -> Option<String> {
        let info = current_info?;
        registry.accounts.iter().find_map(|(name, account)| {
            (account.account_id == info.account_id).then(|| name.clone())
        })
    }

    fn decode_jwt_claims(token: &str) -> Option<JsonValue> {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return None;
        }

        let decoded = decode_base64url(parts[1])?;
        let payload = String::from_utf8(decoded).ok()?;
        serde_json::from_str(&payload).ok()
    }

    fn claim_string(claims: Option<&JsonValue>, key: &str) -> Option<String> {
        claims?
            .get(key)
            .and_then(JsonValue::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    }

    fn unix_millis_to_datetime(value: i64) -> Option<DateTime<Utc>> {
        Utc.timestamp_millis_opt(value).single()
    }

    fn normalize_plan(plan: &str) -> String {
        plan.trim().to_ascii_uppercase()
    }

    /// 计算 token 新鲜度（基于 expires）
    pub fn calculate_freshness(&self, expires_at: Option<DateTime<Utc>>) -> TokenFreshness {
        match expires_at {
            None => TokenFreshness::unknown(),
            Some(expires_at) => {
                let remaining = expires_at - Utc::now();
                if remaining <= Duration::zero() {
                    TokenFreshness::Old
                } else if remaining <= Duration::days(1) {
                    TokenFreshness::Stale
                } else {
                    TokenFreshness::Fresh
                }
            }
        }
    }

    /// 验证账号名称
    fn validate_account_name(&self, name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(CcrError::ValidationError("账号名称不能为空".into()));
        }

        if matches!(name, "default" | "current-login") {
            return Err(CcrError::ValidationError(format!(
                "'{name}' 是保留名称，请使用其他名称"
            )));
        }

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
                format!("{local}***{domain}")
            } else {
                format!("{}***{domain}", &local[..3])
            }
        } else {
            email.to_string()
        }
    }

    /// 判断账号是否已过期
    pub fn is_expired(expires_at: Option<DateTime<Utc>>) -> bool {
        expires_at.is_some_and(|expires_at| expires_at <= Utc::now())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use base64::Engine;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use chrono::Duration;
    use serde_json::json;
    use tempfile::tempdir;

    fn create_test_service() -> (OpenCodeAuthService, tempfile::TempDir, tempfile::TempDir) {
        let ccr = tempdir().unwrap();
        let opencode = tempdir().unwrap();
        (
            OpenCodeAuthService::from_dirs(
                ccr.path().join("platforms").join("opencode"),
                opencode.path().to_path_buf(),
            ),
            ccr,
            opencode,
        )
    }

    fn fake_access_token(email: &str, account_id: &str, plan: &str) -> String {
        let header = URL_SAFE_NO_PAD.encode(r#"{"alg":"none","typ":"JWT"}"#);
        let payload = URL_SAFE_NO_PAD.encode(
            json!({
                "email": email,
                "chatgpt_account_id": account_id,
                "chatgpt_plan_type": plan
            })
            .to_string(),
        );
        format!("{header}.{payload}.signature")
    }

    fn sample_openai_provider(
        email: &str,
        account_id: &str,
        plan: &str,
        expires: i64,
    ) -> JsonValue {
        json!({
            "type": "oauth",
            "access": fake_access_token(email, account_id, plan),
            "refresh": format!("rt_{account_id}"),
            "expires": expires,
            "accountId": account_id,
            "plan": plan.to_ascii_lowercase()
        })
    }

    fn write_auth_json(service: &OpenCodeAuthService, openai: JsonValue, extra: JsonValue) {
        let mut root = JsonMap::new();
        root.insert("openai".to_string(), openai);
        if let Some(extra_map) = extra.as_object() {
            for (key, value) in extra_map {
                root.insert(key.clone(), value.clone());
            }
        }
        service.write_auth_root_map(&root).unwrap();
    }

    #[test]
    fn read_auth_snapshot_extracts_email_and_plan_from_access_token() {
        let (service, _ccr, _opencode) = create_test_service();
        let expires = (Utc::now() + Duration::days(7)).timestamp_millis();
        write_auth_json(
            &service,
            sample_openai_provider("user@example.com", "acc-1", "plus", expires),
            json!({}),
        );

        let snapshot = service.read_auth_snapshot().unwrap();
        let info = snapshot.current_info.unwrap();
        assert_eq!(info.account_id, "acc-1");
        assert_eq!(info.email.as_deref(), Some("user@example.com"));
        assert_eq!(info.plan_type.as_deref(), Some("PLUS"));
        assert_eq!(snapshot.login_state, OpenCodeLoginState::LoggedInUnsaved);
    }

    #[test]
    fn save_current_persists_registry_and_snapshot() {
        let (service, _ccr, _opencode) = create_test_service();
        let expires = (Utc::now() + Duration::days(7)).timestamp_millis();
        write_auth_json(
            &service,
            sample_openai_provider("user@example.com", "acc-1", "plus", expires),
            json!({}),
        );

        service.save_current("work", false).unwrap();

        let registry = service.load_registry().unwrap();
        assert_eq!(registry.current_auth.as_deref(), Some("work"));
        assert!(registry.accounts.contains_key("work"));
        assert_eq!(
            registry.accounts["work"].email.as_deref(),
            Some("use***@example.com")
        );
        assert_eq!(registry.accounts["work"].plan_type.as_deref(), Some("PLUS"));
        assert!(service.account_auth_path("work").exists());

        let snapshot = service.read_auth_snapshot().unwrap();
        assert_eq!(
            snapshot.login_state,
            OpenCodeLoginState::LoggedInSaved("work".to_string())
        );
    }

    #[test]
    fn switch_account_keeps_other_providers_intact() {
        let (service, _ccr, _opencode) = create_test_service();
        let expires = (Utc::now() + Duration::days(7)).timestamp_millis();
        write_auth_json(
            &service,
            sample_openai_provider("one@example.com", "acc-1", "plus", expires),
            json!({
                "github": {
                    "type": "oauth",
                    "token": "gh-token"
                }
            }),
        );
        service.save_current("primary", false).unwrap();

        write_auth_json(
            &service,
            sample_openai_provider("two@example.com", "acc-2", "pro", expires),
            json!({
                "github": {
                    "type": "oauth",
                    "token": "gh-token"
                }
            }),
        );

        service.switch_account("primary").unwrap();

        let root = service.load_auth_root_map().unwrap();
        assert_eq!(
            root.get("github")
                .and_then(|value| value.get("token"))
                .and_then(JsonValue::as_str),
            Some("gh-token")
        );
        assert_eq!(
            root.get("openai")
                .and_then(|value| value.get("accountId"))
                .and_then(JsonValue::as_str),
            Some("acc-1")
        );
    }

    #[test]
    fn switch_account_blocks_expired_account() {
        let (service, _ccr, _opencode) = create_test_service();
        let future = (Utc::now() + Duration::days(7)).timestamp_millis();
        write_auth_json(
            &service,
            sample_openai_provider("one@example.com", "acc-1", "plus", future),
            json!({}),
        );
        service.save_current("expired", false).unwrap();

        let mut registry = service.load_registry().unwrap();
        registry.accounts.get_mut("expired").unwrap().expires_at =
            Some(Utc::now() - Duration::hours(1));
        service.save_registry(&registry).unwrap();

        let err = service.switch_account("expired").unwrap_err();
        assert!(err.to_string().contains("已过期"));
    }

    #[test]
    fn delete_account_removes_registry_and_snapshot() {
        let (service, _ccr, _opencode) = create_test_service();
        let expires = (Utc::now() + Duration::days(7)).timestamp_millis();
        write_auth_json(
            &service,
            sample_openai_provider("user@example.com", "acc-1", "plus", expires),
            json!({}),
        );
        service.save_current("work", false).unwrap();

        service.delete_account("work").unwrap();

        let registry = service.load_registry().unwrap();
        assert!(!registry.accounts.contains_key("work"));
        assert!(!service.account_auth_path("work").exists());
    }

    #[test]
    fn calculate_freshness_uses_expiry_window() {
        let (service, _ccr, _opencode) = create_test_service();
        let fresh = Utc::now() + Duration::days(2);
        let stale = Utc::now() + Duration::hours(12);
        let old = Utc::now() - Duration::minutes(1);

        assert_eq!(
            service.calculate_freshness(Some(fresh)),
            TokenFreshness::Fresh
        );
        assert_eq!(
            service.calculate_freshness(Some(stale)),
            TokenFreshness::Stale
        );
        assert_eq!(service.calculate_freshness(Some(old)), TokenFreshness::Old);
        assert_eq!(service.calculate_freshness(None), TokenFreshness::unknown());
    }
}
