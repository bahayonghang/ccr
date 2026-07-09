// 内置中转站提供商配置
// 站点目录数据来源于 data/providers-catalog.json（仓库内单一事实源，前后端共享同一文件），
// 本模块负责编译期内嵌 + 解析 catalog，并投影为 BuiltinProvider 供签到链路使用

use crate::models::checkin::CheckinProvider;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

/// 构建期内嵌的 catalog JSON 原文（与前端 import 的是同一份文件）
const PROVIDERS_CATALOG_JSON: &str = include_str!("../../../data/providers-catalog.json");

/// 当前后端支持的 catalog schema 版本
const PROVIDERS_CATALOG_SCHEMA_VERSION: u32 = 1;

// 解析失败属于打包错误（数据文件随二进制内嵌），此处用 expect 直接给出明确诊断信息
static PROVIDERS_CATALOG: LazyLock<ProvidersCatalog> = LazyLock::new(|| {
    parse_providers_catalog(PROVIDERS_CATALOG_JSON).expect(
        "providers-catalog.json is invalid: the bundled catalog must parse and match the \
         supported schemaVersion",
    )
});

/// Root structure of the bundled providers catalog.
///
/// The catalog file uses camelCase keys so the frontend can import the same
/// JSON file directly without a field-name adapter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProvidersCatalog {
    /// Schema version; parsing fails when it differs from the supported version.
    pub schema_version: u32,
    /// Catalog entries (one per site).
    pub providers: Vec<CatalogProviderEntry>,
}

/// A single site entry in the providers catalog.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogProviderEntry {
    /// Stable identifier (keeps the `builtin-` prefix semantics).
    pub id: String,
    /// Display name.
    pub name: String,
    /// Display description.
    pub description: String,
    /// Primary domain (used for display and URL matching).
    pub domain: String,
    /// Optional website URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub website_url: Option<String>,
    /// Icon (emoji or URL).
    pub icon: String,
    /// Business-axis category (community / commercial / official / aggregator / local).
    pub biz_category: String,
    /// Check-in mechanism category
    /// (standard / waf_required / cf_required / special / balance_only / cdk).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checkin_category: Option<String>,
    /// Optional search aliases.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aliases: Option<Vec<String>>,
    /// Optional tags.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    /// Optional check-in capability block (field set equivalent to BuiltinProvider).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checkin: Option<CatalogCheckinCapability>,
    /// Optional platform override block shaped like the frontend
    /// `ProviderTemplate.platforms`; the backend treats it as opaque data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platforms: Option<serde_json::Value>,
}

/// Check-in capability block of a catalog entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogCheckinCapability {
    /// API base URL.
    pub base_url: String,
    /// Check-in API path (`null` means querying user info performs the check-in,
    /// or the site does not support check-in when `supportsCheckin` is false).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checkin_path: Option<String>,
    /// Balance query API path.
    pub balance_path: String,
    /// User info API path.
    pub user_info_path: String,
    /// Auth header name.
    pub auth_header: String,
    /// Auth value prefix.
    pub auth_prefix: String,
    /// Whether the site supports check-in.
    pub supports_checkin: bool,
    /// Whether WAF bypass is required.
    pub requires_waf_bypass: bool,
    /// Whether Cloudflare cf_clearance is required.
    pub requires_cf_clearance: bool,
    /// Whether the check-in flow is bugged (for example auto check-in on query).
    pub checkin_bugged: bool,
    /// Required WAF cookie names (backend WAF policy data).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub waf_cookie_names: Option<Vec<String>>,
    /// Optional CDK top-up configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cdk: Option<CatalogCdkConfig>,
    /// Optional OAuth login metadata (public client ids only, not secrets).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oauth: Option<CatalogOauthConfig>,
}

/// CDK top-up configuration in the catalog (camelCase wire format).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogCdkConfig {
    /// CDK type: "runawaytime" | "b4u" | "x666".
    pub cdk_type: String,
    /// URL of the CDK source site.
    pub cdk_source_url: String,
    /// Top-up path; `null` means the reward is credited directly.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topup_path: Option<String>,
    /// Whether extra CDK-site cookies are required.
    pub requires_cdk_cookies: bool,
    /// Whether an access token (JWT) is required.
    pub requires_access_token: bool,
}

/// OAuth login metadata in the catalog (camelCase wire format).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogOauthConfig {
    /// GitHub OAuth client id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub github_client_id: Option<String>,
    /// LinuxDo OAuth client id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub linuxdo_client_id: Option<String>,
    /// OAuth state path (usually "/api/oauth/state").
    pub oauth_state_path: String,
}

/// 内置提供商定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuiltinProvider {
    /// 唯一标识符 (固定 ID，用于识别内置提供商)
    pub id: String,
    /// 显示名称
    pub name: String,
    /// 显示描述
    pub description: String,
    /// 域名 (主域名，用于显示)
    pub domain: String,
    /// 基础 URL (实际 API 地址)
    pub base_url: String,
    /// 签到 API 路径 (None 表示不支持签到)
    pub checkin_path: Option<String>,
    /// 余额查询 API 路径
    pub balance_path: String,
    /// 用户信息 API 路径
    pub user_info_path: String,
    /// 认证头名称
    pub auth_header: String,
    /// 认证前缀
    pub auth_prefix: String,
    /// 是否支持签到
    pub supports_checkin: bool,
    /// 是否需要 WAF 绕过
    pub requires_waf_bypass: bool,
    /// 是否需要 Cloudflare cf_clearance 绕过
    pub requires_cf_clearance: bool,
    /// 签到功能是否有 bug (如 AgentRouter 自动签到)
    pub checkin_bugged: bool,
    /// 图标 (emoji 或 URL)
    pub icon: String,
    /// 提供商分类 (standard / waf_required / cf_required / special / balance_only / cdk)
    pub category: String,
    /// CDK 充值配置 (可选，仅 CDK 站点需要)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cdk_config: Option<CdkProviderConfig>,
    /// OAuth 登录配置 (可选，支持 GitHub/LinuxDo OAuth 的站点)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth_config: Option<OAuthProviderConfig>,
}

/// CDK 充值配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdkProviderConfig {
    /// CDK 类型: "runawaytime" | "b4u" | "x666"
    pub cdk_type: String,
    /// CDK 来源站点 URL
    pub cdk_source_url: String,
    /// 充值路径 (如 "/api/user/topup")，x666 为 None (奖励直接到账)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topup_path: Option<String>,
    /// 是否需要额外的 CDK 站 cookies (runawaytime/b4u 需要)
    pub requires_cdk_cookies: bool,
    /// 是否需要 access_token (x666 的 JWT)
    pub requires_access_token: bool,
}

/// OAuth 登录配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthProviderConfig {
    /// GitHub OAuth client_id (可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub github_client_id: Option<String>,
    /// LinuxDo OAuth client_id (可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linuxdo_client_id: Option<String>,
    /// OAuth state 获取路径 (默认 "/api/oauth/state")
    pub oauth_state_path: String,
}

impl BuiltinProvider {
    /// 转换为 CheckinProvider
    #[allow(dead_code)]
    pub fn to_checkin_provider(&self) -> CheckinProvider {
        CheckinProvider {
            id: self.id.clone(),
            name: self.name.clone(),
            base_url: self.base_url.clone(),
            checkin_path: self
                .checkin_path
                .clone()
                .unwrap_or_else(|| "/api/user/sign_in".to_string()),
            balance_path: self.balance_path.clone(),
            user_info_path: self.user_info_path.clone(),
            auth_header: self.auth_header.clone(),
            auth_prefix: self.auth_prefix.clone(),
            enabled: true,
            builtin_id: Some(self.id.clone()),
            created_at: Utc::now(),
            updated_at: None,
        }
    }
}

impl CatalogProviderEntry {
    /// Project this catalog entry into the legacy `BuiltinProvider` shape.
    ///
    /// Returns `None` when the entry has no check-in capability block.
    pub fn to_builtin_provider(&self) -> Option<BuiltinProvider> {
        let checkin = self.checkin.as_ref()?;
        Some(BuiltinProvider {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            domain: self.domain.clone(),
            base_url: checkin.base_url.clone(),
            checkin_path: checkin.checkin_path.clone(),
            balance_path: checkin.balance_path.clone(),
            user_info_path: checkin.user_info_path.clone(),
            auth_header: checkin.auth_header.clone(),
            auth_prefix: checkin.auth_prefix.clone(),
            supports_checkin: checkin.supports_checkin,
            requires_waf_bypass: checkin.requires_waf_bypass,
            requires_cf_clearance: checkin.requires_cf_clearance,
            checkin_bugged: checkin.checkin_bugged,
            icon: self.icon.clone(),
            category: self
                .checkin_category
                .clone()
                .unwrap_or_else(|| "standard".to_string()),
            cdk_config: checkin.cdk.as_ref().map(|cdk| CdkProviderConfig {
                cdk_type: cdk.cdk_type.clone(),
                cdk_source_url: cdk.cdk_source_url.clone(),
                topup_path: cdk.topup_path.clone(),
                requires_cdk_cookies: cdk.requires_cdk_cookies,
                requires_access_token: cdk.requires_access_token,
            }),
            oauth_config: checkin.oauth.as_ref().map(|oauth| OAuthProviderConfig {
                github_client_id: oauth.github_client_id.clone(),
                linuxdo_client_id: oauth.linuxdo_client_id.clone(),
                oauth_state_path: oauth.oauth_state_path.clone(),
            }),
        })
    }
}

/// 解析 catalog JSON 并校验 schemaVersion（失败时返回带上下文的显式错误）
fn parse_providers_catalog(json: &str) -> Result<ProvidersCatalog, String> {
    let catalog: ProvidersCatalog =
        serde_json::from_str(json).map_err(|e| format!("providers-catalog.json 解析失败: {e}"))?;

    if catalog.schema_version != PROVIDERS_CATALOG_SCHEMA_VERSION {
        return Err(format!(
            "providers-catalog.json schemaVersion 不兼容: 期望 {PROVIDERS_CATALOG_SCHEMA_VERSION}, 实际 {}",
            catalog.schema_version
        ));
    }

    Ok(catalog)
}

/// Get the parsed bundled providers catalog (single source of truth).
pub fn get_providers_catalog() -> &'static ProvidersCatalog {
    &PROVIDERS_CATALOG
}

/// 获取所有内置提供商
pub fn get_builtin_providers() -> Vec<BuiltinProvider> {
    get_providers_catalog()
        .providers
        .iter()
        .filter_map(CatalogProviderEntry::to_builtin_provider)
        .collect()
}

/// 根据 ID 获取内置提供商
pub fn get_builtin_provider_by_id(id: &str) -> Option<BuiltinProvider> {
    get_providers_catalog()
        .providers
        .iter()
        .find(|entry| entry.id == id)
        .and_then(CatalogProviderEntry::to_builtin_provider)
}

/// Resolve the builtin provider behind a stored `CheckinProvider`.
///
/// Prefers the persisted `builtin_id` (rename-safe); falls back to the legacy
/// name-based match for rows created before `builtin_id` existed.
pub fn resolve_builtin_for_provider(provider: &CheckinProvider) -> Option<BuiltinProvider> {
    if let Some(builtin_id) = provider.builtin_id.as_deref()
        && let Some(found) = get_builtin_provider_by_id(builtin_id)
    {
        return Some(found);
    }

    // 旧数据回退路径：与历史行为保持一致（name 精确匹配或 builtin-{name} 推导）
    let derived_id = format!("builtin-{}", provider.name.to_lowercase());
    get_builtin_providers()
        .into_iter()
        .find(|bp| bp.name == provider.name || bp.id == derived_id)
}

/// 检查是否是内置提供商 ID
#[allow(dead_code)]
pub fn is_builtin_provider_id(id: &str) -> bool {
    id.starts_with("builtin-")
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// 改造前硬编码 vec 的关键字段对照表（迁移期 golden 数据，顺序与原 vec 一致）:
    /// (id, name, domain, base_url, icon, category, github_client_id, linuxdo_client_id)
    type GoldenRow = (
        &'static str,
        &'static str,
        &'static str,
        &'static str,
        &'static str,
        &'static str,
        Option<&'static str>,
        Option<&'static str>,
    );

    const GOLDEN_PROVIDERS: &[GoldenRow] = &[
        (
            "builtin-anyrouter",
            "AnyRouter",
            "anyrouter.top",
            "https://anyrouter.top",
            "🌐",
            "waf_required",
            None,
            None,
        ),
        (
            "builtin-agentrouter",
            "AgentRouter",
            "agentrouter.org",
            "https://agentrouter.org",
            "🤖",
            "special",
            None,
            None,
        ),
        (
            "builtin-coderouter",
            "CodeRouter",
            "api.codemirror.codes",
            "https://api.codemirror.codes",
            "💻",
            "balance_only",
            None,
            None,
        ),
        (
            "builtin-runawaytime",
            "RunAnytime",
            "runanytime.hxi.me",
            "https://runanytime.hxi.me",
            "⏱️",
            "cf_required",
            None,
            None,
        ),
        (
            "builtin-elysiver",
            "Elysiver",
            "elysiver.h-e.top",
            "https://elysiver.h-e.top",
            "🌸",
            "cf_required",
            None,
            None,
        ),
        (
            "builtin-hotaru",
            "Hotaru",
            "hotaruapi.com",
            "https://hotaruapi.com",
            "🔥",
            "cf_required",
            None,
            None,
        ),
        (
            "builtin-b4u",
            "B4U",
            "b4u.qzz.io",
            "https://b4u.qzz.io",
            "🎲",
            "cf_required",
            None,
            None,
        ),
        (
            "builtin-x666",
            "x666",
            "x666.me",
            "https://x666.me",
            "🎰",
            "cdk",
            None,
            None,
        ),
        (
            "builtin-codex-cab",
            "Codex.cab",
            "codex.cab",
            "https://codex.cab",
            "📝",
            "standard",
            None,
            Some("nYxyCYi7VDrfjNn2rBM8VkPaKNKxWEx1"),
        ),
        (
            "builtin-clove",
            "Clove",
            "clove.cc.cd",
            "https://clove.cc.cd",
            "🍀",
            "standard",
            None,
            Some("Lr8C2Ny7JPr7c4YqysaDtVEqkO1a9eL7"),
        ),
        (
            "builtin-npcodex",
            "NPCodex",
            "npcodex.kiroxubei.tech",
            "https://npcodex.kiroxubei.tech",
            "🔮",
            "standard",
            None,
            Some("APUcB3LChvSGi3FmkODZx6Ij2038mkHY"),
        ),
        (
            "builtin-muapi",
            "MuAPI",
            "ai.muapi.cn",
            "https://ai.muapi.cn",
            "🎵",
            "standard",
            None,
            Some("WKD07GTaaAQcPi15BAfmIHMMCg1BG95t"),
        ),
        (
            "builtin-feisakura",
            "Feisakura",
            "api.feisakura.fun",
            "https://api.feisakura.fun",
            "🌸",
            "standard",
            None,
            Some("XPXmWksr3NcH2aiz0MgqK5jtEmfdfZ0Q"),
        ),
        (
            "builtin-xionger",
            "Xionger",
            "api.xionger.ccwu.cc",
            "https://api.xionger.ccwu.cc",
            "🐻",
            "standard",
            None,
            Some("SYU8YV8Dd0PHmBnNCcjmGIhYfDnmPtBc"),
        ),
        (
            "builtin-einzieg",
            "Einzieg",
            "api.einzieg.site",
            "https://api.einzieg.site",
            "⚡",
            "standard",
            None,
            Some("aBambSqvDqCgTW8fCarJBeQji8M5RATf"),
        ),
        (
            "builtin-2020111",
            "2020111",
            "api.2020111.xyz",
            "https://api.2020111.xyz",
            "🔢",
            "standard",
            None,
            Some("gnyvfmAfXrnYrt9ierq3Onj1ADvdVmmm"),
        ),
        (
            "builtin-361888",
            "361888",
            "api.361888.xyz",
            "https://api.361888.xyz",
            "🎰",
            "standard",
            None,
            Some("ze9QLEoERDgCdFnlBJeB0uASPwOTVyfM"),
        ),
        (
            "builtin-yyds",
            "YYDS",
            "yyds.215.im",
            "https://yyds.215.im",
            "🏆",
            "standard",
            None,
            Some("BvCzH7KoNBVpQIfdWCgUMIGaPMOpgbwI"),
        ),
        (
            "builtin-anthorpic",
            "Anthorpic",
            "anthorpic.us.ci",
            "https://anthorpic.us.ci",
            "🤖",
            "standard",
            None,
            Some("nNzrggkmAew2bJYxCgC2iaU6IYcaWt8S"),
        ),
        (
            "builtin-nanohajimi",
            "Nanohajimi",
            "free.nanohajimi.mom",
            "https://free.nanohajimi.mom",
            "🌟",
            "standard",
            None,
            Some("svkUqtRyhOJMULQ1Zfnfhvv9ALSnANhf"),
        ),
        (
            "builtin-zapi-aicc0",
            "ZAPI",
            "zapi.aicc0.com",
            "https://zapi.aicc0.com",
            "🧩",
            "standard",
            Some("Ov23linrAuNoCCMoztG7"),
            Some("Tm30iXRcGTM9oyVreW3edvCNGO5kPEWX"),
        ),
        (
            "builtin-llmapi-vhuds",
            "LLMAPI",
            "llmapi.vhuds.com",
            "https://llmapi.vhuds.com",
            "💡",
            "standard",
            None,
            Some("aiepRPewQgwTPbUIq8Z4muSYw76NFSUZ"),
        ),
        (
            "builtin-muyuan",
            "Muyuan",
            "muyuan.do",
            "https://muyuan.do",
            "🎌",
            "waf_required",
            None,
            None,
        ),
    ];

    #[test]
    fn test_get_builtin_providers() {
        let providers = get_builtin_providers();
        assert_eq!(providers.len(), 23);
        assert_eq!(
            providers
                .iter()
                .filter(|p| p.category == "standard")
                .count(),
            14
        );

        let anyrouter = providers.iter().find(|p| p.name == "AnyRouter").unwrap();
        assert!(anyrouter.supports_checkin);
        assert!(anyrouter.requires_waf_bypass);
        assert!(!anyrouter.requires_cf_clearance);
        assert_eq!(anyrouter.category, "waf_required");

        let agentrouter = providers.iter().find(|p| p.name == "AgentRouter").unwrap();
        assert!(agentrouter.checkin_bugged);
        assert_eq!(agentrouter.category, "special");

        let coderouter = providers.iter().find(|p| p.name == "CodeRouter").unwrap();
        assert!(!coderouter.supports_checkin);
        assert_eq!(coderouter.category, "balance_only");

        let hotaru = providers.iter().find(|p| p.name == "Hotaru").unwrap();
        assert!(hotaru.requires_cf_clearance);
        assert!(!hotaru.requires_waf_bypass);
        assert_eq!(hotaru.category, "cf_required");

        let llmapi = providers.iter().find(|p| p.name == "LLMAPI").unwrap();
        assert!(llmapi.supports_checkin);
        assert!(!llmapi.requires_waf_bypass);
        assert_eq!(llmapi.category, "standard");
    }

    /// Golden test: catalog 投影结果与改造前硬编码 vec 的关键字段逐一等价（含顺序）
    #[test]
    fn test_golden_identity_matches_pre_migration_hardcode() {
        let providers = get_builtin_providers();
        assert_eq!(providers.len(), GOLDEN_PROVIDERS.len());

        for (provider, golden) in providers.iter().zip(GOLDEN_PROVIDERS.iter()) {
            let (id, name, domain, base_url, icon, category, github_id, linuxdo_id) = golden;
            assert_eq!(provider.id, *id, "id mismatch for {id}");
            assert_eq!(provider.name, *name, "name mismatch for {id}");
            assert_eq!(provider.domain, *domain, "domain mismatch for {id}");
            assert_eq!(provider.base_url, *base_url, "base_url mismatch for {id}");
            assert_eq!(provider.icon, *icon, "icon mismatch for {id}");
            assert_eq!(provider.category, *category, "category mismatch for {id}");
            assert_eq!(
                provider
                    .oauth_config
                    .as_ref()
                    .and_then(|o| o.github_client_id.as_deref()),
                *github_id,
                "github_client_id mismatch for {id}"
            );
            assert_eq!(
                provider
                    .oauth_config
                    .as_ref()
                    .and_then(|o| o.linuxdo_client_id.as_deref()),
                *linuxdo_id,
                "linuxdo_client_id mismatch for {id}"
            );
        }
    }

    /// Golden test: 14 个标准 NewAPI 公益站的共享字段与改造前 standard_provider() 等价
    #[test]
    fn test_golden_standard_provider_invariants() {
        let providers = get_builtin_providers();
        let standard: Vec<_> = providers
            .iter()
            .filter(|p| p.category == "standard")
            .collect();
        assert_eq!(standard.len(), 14);

        for p in standard {
            assert_eq!(
                p.description,
                format!("{} 公益 AI 中转站", p.name),
                "description mismatch for {}",
                p.id
            );
            assert_eq!(p.checkin_path.as_deref(), Some("/api/user/checkin"));
            assert_eq!(p.balance_path, "/api/user/self");
            assert_eq!(p.user_info_path, "/api/user/self");
            assert_eq!(p.auth_header, "Authorization");
            assert_eq!(p.auth_prefix, "Bearer");
            assert!(p.supports_checkin);
            assert!(!p.requires_waf_bypass);
            assert!(!p.requires_cf_clearance);
            assert!(!p.checkin_bugged);
            assert!(p.cdk_config.is_none());
            let oauth = p.oauth_config.as_ref().unwrap();
            assert_eq!(oauth.oauth_state_path, "/api/oauth/state");
        }
    }

    /// Golden test: 8 个特殊机制站的全部关键字段与改造前硬编码等价
    #[test]
    fn test_golden_special_providers_match_pre_migration_hardcode() {
        let anyrouter = get_builtin_provider_by_id("builtin-anyrouter").unwrap();
        assert_eq!(
            anyrouter.description,
            "主流 AI 中转站，支持多模型，需要 WAF 绕过"
        );
        assert_eq!(anyrouter.checkin_path.as_deref(), Some("/api/user/sign_in"));
        assert_eq!(anyrouter.balance_path, "/api/user/self");
        assert_eq!(anyrouter.user_info_path, "/api/user/self");
        assert_eq!(anyrouter.auth_header, "Authorization");
        assert_eq!(anyrouter.auth_prefix, "Bearer");
        assert!(anyrouter.supports_checkin);
        assert!(anyrouter.requires_waf_bypass);
        assert!(!anyrouter.requires_cf_clearance);
        assert!(!anyrouter.checkin_bugged);
        assert!(anyrouter.cdk_config.is_none());
        let anyrouter_oauth = anyrouter.oauth_config.as_ref().unwrap();
        assert_eq!(anyrouter_oauth.github_client_id, None);
        assert_eq!(anyrouter_oauth.linuxdo_client_id, None);
        assert_eq!(anyrouter_oauth.oauth_state_path, "/api/oauth/state");

        let agentrouter = get_builtin_provider_by_id("builtin-agentrouter").unwrap();
        assert_eq!(
            agentrouter.description,
            "AI 代理路由站，查询用户信息时自动签到"
        );
        assert_eq!(agentrouter.checkin_path, None);
        assert!(agentrouter.supports_checkin);
        assert!(!agentrouter.requires_waf_bypass);
        assert!(agentrouter.checkin_bugged);
        assert!(agentrouter.cdk_config.is_none());
        assert!(agentrouter.oauth_config.is_some());

        let coderouter = get_builtin_provider_by_id("builtin-coderouter").unwrap();
        assert_eq!(coderouter.description, "代码镜像 AI 中转站，仅支持余额查询");
        assert_eq!(coderouter.checkin_path, None);
        assert!(!coderouter.supports_checkin);
        assert!(coderouter.cdk_config.is_none());
        assert!(coderouter.oauth_config.is_none());

        let runawaytime = get_builtin_provider_by_id("builtin-runawaytime").unwrap();
        assert_eq!(
            runawaytime.description,
            "公益 AI 中转站，需要 Cloudflare 绕过，支持 CDK 充值"
        );
        assert_eq!(runawaytime.checkin_path, None);
        assert!(!runawaytime.supports_checkin);
        assert!(runawaytime.requires_cf_clearance);
        let runawaytime_cdk = runawaytime.cdk_config.as_ref().unwrap();
        assert_eq!(runawaytime_cdk.cdk_type, "runawaytime");
        assert_eq!(runawaytime_cdk.cdk_source_url, "https://fuli.hxi.me");
        assert_eq!(
            runawaytime_cdk.topup_path.as_deref(),
            Some("/api/user/topup")
        );
        assert!(runawaytime_cdk.requires_cdk_cookies);
        assert!(!runawaytime_cdk.requires_access_token);
        assert!(runawaytime.oauth_config.is_none());

        let elysiver = get_builtin_provider_by_id("builtin-elysiver").unwrap();
        assert_eq!(elysiver.description, "公益 AI 中转站，需要 Cloudflare 绕过");
        assert_eq!(elysiver.checkin_path.as_deref(), Some("/api/user/checkin"));
        assert!(elysiver.supports_checkin);
        assert!(elysiver.requires_cf_clearance);
        assert!(elysiver.cdk_config.is_none());

        let hotaru = get_builtin_provider_by_id("builtin-hotaru").unwrap();
        assert_eq!(hotaru.description, "公益 AI 中转站，需要 Cloudflare 绕过");
        assert_eq!(hotaru.checkin_path.as_deref(), Some("/api/user/checkin"));
        assert!(hotaru.supports_checkin);
        assert!(hotaru.requires_cf_clearance);

        let b4u = get_builtin_provider_by_id("builtin-b4u").unwrap();
        assert_eq!(
            b4u.description,
            "公益 AI 中转站，需要 Cloudflare 绕过，支持 CDK 充值"
        );
        assert_eq!(b4u.checkin_path, None);
        assert!(!b4u.supports_checkin);
        assert!(b4u.requires_cf_clearance);
        let b4u_cdk = b4u.cdk_config.as_ref().unwrap();
        assert_eq!(b4u_cdk.cdk_type, "b4u");
        assert_eq!(b4u_cdk.cdk_source_url, "https://tw.b4u.qzz.io");
        assert_eq!(b4u_cdk.topup_path.as_deref(), Some("/api/user/topup"));
        assert!(b4u_cdk.requires_cdk_cookies);
        assert!(!b4u_cdk.requires_access_token);

        let x666 = get_builtin_provider_by_id("builtin-x666").unwrap();
        assert_eq!(
            x666.description,
            "公益 AI 中转站，通过 up.x666.me 抽奖获取额度"
        );
        assert_eq!(x666.checkin_path, None);
        assert!(!x666.supports_checkin);
        assert!(!x666.requires_waf_bypass);
        assert!(!x666.requires_cf_clearance);
        let x666_cdk = x666.cdk_config.as_ref().unwrap();
        assert_eq!(x666_cdk.cdk_type, "x666");
        assert_eq!(x666_cdk.cdk_source_url, "https://up.x666.me");
        assert_eq!(x666_cdk.topup_path, None);
        assert!(!x666_cdk.requires_cdk_cookies);
        assert!(x666_cdk.requires_access_token);
    }

    #[test]
    fn test_standard_providers_include_oauth_metadata() {
        let providers = get_builtin_providers();
        assert!(
            providers
                .iter()
                .filter(|p| p.category == "standard")
                .all(|p| p.oauth_config.is_some())
        );

        let codex_cab = get_builtin_provider_by_id("builtin-codex-cab").unwrap();
        let codex_cab_oauth = codex_cab.oauth_config.as_ref().unwrap();
        assert_eq!(codex_cab_oauth.github_client_id, None);
        assert_eq!(
            codex_cab_oauth.linuxdo_client_id.as_deref(),
            Some("nYxyCYi7VDrfjNn2rBM8VkPaKNKxWEx1")
        );
        assert_eq!(codex_cab_oauth.oauth_state_path, "/api/oauth/state");

        let zapi = get_builtin_provider_by_id("builtin-zapi-aicc0").unwrap();
        let zapi_oauth = zapi.oauth_config.as_ref().unwrap();
        assert_eq!(
            zapi_oauth.github_client_id.as_deref(),
            Some("Ov23linrAuNoCCMoztG7")
        );
        assert_eq!(
            zapi_oauth.linuxdo_client_id.as_deref(),
            Some("Tm30iXRcGTM9oyVreW3edvCNGO5kPEWX")
        );
        assert_eq!(zapi_oauth.oauth_state_path, "/api/oauth/state");
    }

    #[test]
    fn test_all_providers_have_unique_ids() {
        let providers = get_builtin_providers();
        let mut ids: Vec<&str> = providers.iter().map(|p| p.id.as_str()).collect();
        let total = ids.len();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), total, "Duplicate provider IDs detected");
    }

    #[test]
    fn test_all_providers_have_valid_urls() {
        let providers = get_builtin_providers();
        for p in &providers {
            assert!(
                p.base_url.starts_with("https://"),
                "Provider {} base_url must start with https://",
                p.name
            );
            assert!(
                !p.base_url.ends_with('/'),
                "Provider {} base_url should not end with /",
                p.name
            );
        }
    }

    #[test]
    fn test_is_builtin_provider_id() {
        assert!(is_builtin_provider_id("builtin-anyrouter"));
        assert!(!is_builtin_provider_id("custom-provider-123"));
    }

    #[test]
    fn test_get_builtin_provider_by_id() {
        assert!(get_builtin_provider_by_id("builtin-anyrouter").is_some());
        assert!(get_builtin_provider_by_id("builtin-zapi-aicc0").is_some());
        assert!(get_builtin_provider_by_id("builtin-nonexistent").is_none());
    }

    #[test]
    fn test_to_checkin_provider_sets_builtin_id() {
        let anyrouter = get_builtin_provider_by_id("builtin-anyrouter").unwrap();
        let provider = anyrouter.to_checkin_provider();
        assert_eq!(provider.builtin_id.as_deref(), Some("builtin-anyrouter"));
    }

    #[test]
    fn test_resolve_builtin_prefers_builtin_id_after_rename() {
        // 改名后的 provider：name/base_url 都对不上，builtin_id 仍能精确反查
        let mut renamed =
            CheckinProvider::new("我的中转站".to_string(), "https://example.com".to_string());
        renamed.builtin_id = Some("builtin-runawaytime".to_string());

        let resolved = resolve_builtin_for_provider(&renamed).unwrap();
        assert_eq!(resolved.id, "builtin-runawaytime");
        assert!(resolved.cdk_config.is_some());

        // 旧数据（无 builtin_id）按 name 回退匹配
        let legacy =
            CheckinProvider::new("AnyRouter".to_string(), "https://anyrouter.top".to_string());
        let resolved = resolve_builtin_for_provider(&legacy).unwrap();
        assert_eq!(resolved.id, "builtin-anyrouter");

        // 既无 builtin_id 又对不上 name 的自定义站返回 None
        let custom = CheckinProvider::new(
            "My Custom".to_string(),
            "https://custom.example".to_string(),
        );
        assert!(resolve_builtin_for_provider(&custom).is_none());
    }

    #[test]
    fn test_catalog_rejects_wrong_schema_version() {
        let mut value: serde_json::Value = serde_json::from_str(PROVIDERS_CATALOG_JSON).unwrap();
        value["schemaVersion"] = serde_json::json!(999);
        let tampered = serde_json::to_string(&value).unwrap();

        let err = parse_providers_catalog(&tampered).unwrap_err();
        assert!(err.contains("schemaVersion"), "unexpected error: {err}");
        assert!(err.contains("999"), "unexpected error: {err}");
    }

    #[test]
    fn test_catalog_rejects_invalid_json() {
        let err = parse_providers_catalog("{ not json").unwrap_err();
        assert!(err.contains("解析失败"), "unexpected error: {err}");
    }

    #[test]
    fn test_catalog_serde_roundtrip() {
        let catalog = parse_providers_catalog(PROVIDERS_CATALOG_JSON).unwrap();
        let serialized = serde_json::to_string(&catalog).unwrap();
        let reparsed = parse_providers_catalog(&serialized).unwrap();
        assert_eq!(catalog, reparsed);
    }

    /// platforms 块只允许出现非敏感字段（绝不能携带任何密钥/凭证）
    #[test]
    fn test_platforms_blocks_contain_no_sensitive_fields() {
        fn assert_keys_safe(value: &serde_json::Value, path: &str) {
            const FORBIDDEN: &[&str] = &["token", "secret", "password", "cookie", "credential"];
            if let serde_json::Value::Object(map) = value {
                for (key, child) in map {
                    let lowered = key.to_lowercase();
                    for forbidden in FORBIDDEN {
                        assert!(
                            !lowered.contains(forbidden),
                            "sensitive key '{key}' found at {path}"
                        );
                    }
                    // apiKeyUrl（取 key 的文档地址）是模板契约允许的字段，其余含 key 的字段一律拒绝
                    if lowered.contains("key") {
                        assert_eq!(
                            key, "apiKeyUrl",
                            "unexpected key-like field '{key}' at {path}"
                        );
                    }
                    assert_keys_safe(child, &format!("{path}.{key}"));
                }
            }
        }

        for entry in &get_providers_catalog().providers {
            if let Some(platforms) = &entry.platforms {
                assert_keys_safe(platforms, &entry.id);
            }
        }
    }

    /// 标准 NewAPI 公益站必须带 claude/codex override，且 baseUrl 与签到 baseUrl 一致
    #[test]
    fn test_standard_providers_have_platform_overrides() {
        for entry in &get_providers_catalog().providers {
            if entry.checkin_category.as_deref() != Some("standard") {
                continue;
            }

            let checkin = entry.checkin.as_ref().unwrap();
            let platforms = entry
                .platforms
                .as_ref()
                .unwrap_or_else(|| panic!("standard provider {} missing platforms", entry.id));

            for platform in ["claude", "codex"] {
                let base_url = platforms
                    .get(platform)
                    .and_then(|p| p.get("baseUrl"))
                    .and_then(|v| v.as_str())
                    .unwrap_or_else(|| {
                        panic!("standard provider {} missing {platform}.baseUrl", entry.id)
                    });
                assert_eq!(
                    base_url, checkin.base_url,
                    "baseUrl mismatch for {}",
                    entry.id
                );
            }
        }
    }
}
