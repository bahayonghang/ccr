// 内置中转站提供商配置
// 提供预设的知名中转站配置，用户可以快速添加

use crate::models::checkin::CheckinProvider;
use chrono::Utc;
use serde::{Deserialize, Serialize};

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
            created_at: Utc::now(),
            updated_at: None,
        }
    }
}

/// 构造 OAuth 配置（仅在存在 OAuth 元数据时返回）
fn oauth_provider_config(
    github_client_id: Option<&str>,
    linuxdo_client_id: Option<&str>,
    oauth_state_path: &str,
) -> Option<OAuthProviderConfig> {
    if github_client_id.is_none() && linuxdo_client_id.is_none() {
        return None;
    }

    Some(OAuthProviderConfig {
        github_client_id: github_client_id.map(str::to_string),
        linuxdo_client_id: linuxdo_client_id.map(str::to_string),
        oauth_state_path: oauth_state_path.to_string(),
    })
}

/// 创建标准 NewAPI 公益站提供商
fn standard_provider(
    id: &str,
    name: &str,
    domain: &str,
    base_url: &str,
    icon: &str,
    oauth_config: Option<OAuthProviderConfig>,
) -> BuiltinProvider {
    BuiltinProvider {
        id: id.to_string(),
        name: name.to_string(),
        description: format!("{name} 公益 AI 中转站"),
        domain: domain.to_string(),
        base_url: base_url.to_string(),
        checkin_path: Some("/api/user/checkin".to_string()),
        balance_path: "/api/user/self".to_string(),
        user_info_path: "/api/user/self".to_string(),
        auth_header: "Authorization".to_string(),
        auth_prefix: "Bearer".to_string(),
        supports_checkin: true,
        requires_waf_bypass: false,
        requires_cf_clearance: false,
        checkin_bugged: false,
        icon: icon.to_string(),
        category: "standard".to_string(),
        cdk_config: None,
        oauth_config,
    }
}

/// 获取所有内置提供商
pub fn get_builtin_providers() -> Vec<BuiltinProvider> {
    vec![
        // ═══════════════════════════════════════════════════════════
        // 特殊机制提供商 (Special)
        // ═══════════════════════════════════════════════════════════

        // AnyRouter - 主流中转站，需要 WAF 绕过
        BuiltinProvider {
            id: "builtin-anyrouter".to_string(),
            name: "AnyRouter".to_string(),
            description: "主流 AI 中转站，支持多模型，需要 WAF 绕过".to_string(),
            domain: "anyrouter.top".to_string(),
            base_url: "https://anyrouter.top".to_string(),
            checkin_path: Some("/api/user/sign_in".to_string()),
            balance_path: "/api/user/self".to_string(),
            user_info_path: "/api/user/self".to_string(),
            auth_header: "Authorization".to_string(),
            auth_prefix: "Bearer".to_string(),
            supports_checkin: true,
            requires_waf_bypass: true,
            requires_cf_clearance: false,
            checkin_bugged: false,
            icon: "🌐".to_string(),
            category: "waf_required".to_string(),
            cdk_config: None,
            oauth_config: Some(OAuthProviderConfig {
                github_client_id: None,
                linuxdo_client_id: None,
                oauth_state_path: "/api/oauth/state".to_string(),
            }),
        },
        // AgentRouter - 无需 WAF，但签到功能有 bug（查询用户信息时自动签到）
        BuiltinProvider {
            id: "builtin-agentrouter".to_string(),
            name: "AgentRouter".to_string(),
            description: "AI 代理路由站，查询用户信息时自动签到".to_string(),
            domain: "agentrouter.org".to_string(),
            base_url: "https://agentrouter.org".to_string(),
            checkin_path: None,
            balance_path: "/api/user/self".to_string(),
            user_info_path: "/api/user/self".to_string(),
            auth_header: "Authorization".to_string(),
            auth_prefix: "Bearer".to_string(),
            supports_checkin: true,
            requires_waf_bypass: false,
            requires_cf_clearance: false,
            checkin_bugged: true,
            icon: "🤖".to_string(),
            category: "special".to_string(),
            cdk_config: None,
            oauth_config: Some(OAuthProviderConfig {
                github_client_id: None,
                linuxdo_client_id: None,
                oauth_state_path: "/api/oauth/state".to_string(),
            }),
        },
        // CodeRouter - 代码镜像中转站，仅支持余额查询
        BuiltinProvider {
            id: "builtin-coderouter".to_string(),
            name: "CodeRouter".to_string(),
            description: "代码镜像 AI 中转站，仅支持余额查询".to_string(),
            domain: "api.codemirror.codes".to_string(),
            base_url: "https://api.codemirror.codes".to_string(),
            checkin_path: None,
            balance_path: "/api/user/self".to_string(),
            user_info_path: "/api/user/self".to_string(),
            auth_header: "Authorization".to_string(),
            auth_prefix: "Bearer".to_string(),
            supports_checkin: false,
            requires_waf_bypass: false,
            requires_cf_clearance: false,
            checkin_bugged: false,
            icon: "💻".to_string(),
            category: "balance_only".to_string(),
            cdk_config: None,
            oauth_config: None,
        },
        // ═══════════════════════════════════════════════════════════
        // 需要 CF Clearance 绕过的提供商
        // ═══════════════════════════════════════════════════════════

        // RunAnytime - 需要 CF Clearance + CDK 充值 (fuli.hxi.me)
        BuiltinProvider {
            id: "builtin-runawaytime".to_string(),
            name: "RunAnytime".to_string(),
            description: "公益 AI 中转站，需要 Cloudflare 绕过，支持 CDK 充值".to_string(),
            domain: "runanytime.hxi.me".to_string(),
            base_url: "https://runanytime.hxi.me".to_string(),
            checkin_path: None,
            balance_path: "/api/user/self".to_string(),
            user_info_path: "/api/user/self".to_string(),
            auth_header: "Authorization".to_string(),
            auth_prefix: "Bearer".to_string(),
            supports_checkin: false,
            requires_waf_bypass: false,
            requires_cf_clearance: true,
            checkin_bugged: false,
            icon: "⏱️".to_string(),
            category: "cf_required".to_string(),
            cdk_config: Some(CdkProviderConfig {
                cdk_type: "runawaytime".to_string(),
                cdk_source_url: "https://fuli.hxi.me".to_string(),
                topup_path: Some("/api/user/topup".to_string()),
                requires_cdk_cookies: true,
                requires_access_token: false,
            }),
            oauth_config: None,
        },
        // Elysiver - 需要 CF Clearance
        BuiltinProvider {
            id: "builtin-elysiver".to_string(),
            name: "Elysiver".to_string(),
            description: "公益 AI 中转站，需要 Cloudflare 绕过".to_string(),
            domain: "elysiver.h-e.top".to_string(),
            base_url: "https://elysiver.h-e.top".to_string(),
            checkin_path: Some("/api/user/checkin".to_string()),
            balance_path: "/api/user/self".to_string(),
            user_info_path: "/api/user/self".to_string(),
            auth_header: "Authorization".to_string(),
            auth_prefix: "Bearer".to_string(),
            supports_checkin: true,
            requires_waf_bypass: false,
            requires_cf_clearance: true,
            checkin_bugged: false,
            icon: "🌸".to_string(),
            category: "cf_required".to_string(),
            cdk_config: None,
            oauth_config: None,
        },
        // Hotaru - 需要 CF Clearance
        BuiltinProvider {
            id: "builtin-hotaru".to_string(),
            name: "Hotaru".to_string(),
            description: "公益 AI 中转站，需要 Cloudflare 绕过".to_string(),
            domain: "hotaruapi.com".to_string(),
            base_url: "https://hotaruapi.com".to_string(),
            checkin_path: Some("/api/user/checkin".to_string()),
            balance_path: "/api/user/self".to_string(),
            user_info_path: "/api/user/self".to_string(),
            auth_header: "Authorization".to_string(),
            auth_prefix: "Bearer".to_string(),
            supports_checkin: true,
            requires_waf_bypass: false,
            requires_cf_clearance: true,
            checkin_bugged: false,
            icon: "🔥".to_string(),
            category: "cf_required".to_string(),
            cdk_config: None,
            oauth_config: None,
        },
        // B4U - 需要 CF Clearance + CDK 充值 (tw.b4u.qzz.io luckydraw)
        BuiltinProvider {
            id: "builtin-b4u".to_string(),
            name: "B4U".to_string(),
            description: "公益 AI 中转站，需要 Cloudflare 绕过，支持 CDK 充值".to_string(),
            domain: "b4u.qzz.io".to_string(),
            base_url: "https://b4u.qzz.io".to_string(),
            checkin_path: None,
            balance_path: "/api/user/self".to_string(),
            user_info_path: "/api/user/self".to_string(),
            auth_header: "Authorization".to_string(),
            auth_prefix: "Bearer".to_string(),
            supports_checkin: false,
            requires_waf_bypass: false,
            requires_cf_clearance: true,
            checkin_bugged: false,
            icon: "🎲".to_string(),
            category: "cf_required".to_string(),
            cdk_config: Some(CdkProviderConfig {
                cdk_type: "b4u".to_string(),
                cdk_source_url: "https://tw.b4u.qzz.io".to_string(),
                topup_path: Some("/api/user/topup".to_string()),
                requires_cdk_cookies: true,
                requires_access_token: false,
            }),
            oauth_config: None,
        },
        // ═══════════════════════════════════════════════════════════
        // CDK 充值站点 (通过外部站点获取充值码)
        // ═══════════════════════════════════════════════════════════

        // x666 - 通过 up.x666.me 抽奖，奖励直接充值到账户
        BuiltinProvider {
            id: "builtin-x666".to_string(),
            name: "x666".to_string(),
            description: "公益 AI 中转站，通过 up.x666.me 抽奖获取额度".to_string(),
            domain: "x666.me".to_string(),
            base_url: "https://x666.me".to_string(),
            checkin_path: None,
            balance_path: "/api/user/self".to_string(),
            user_info_path: "/api/user/self".to_string(),
            auth_header: "Authorization".to_string(),
            auth_prefix: "Bearer".to_string(),
            supports_checkin: false,
            requires_waf_bypass: false,
            requires_cf_clearance: false,
            checkin_bugged: false,
            icon: "🎰".to_string(),
            category: "cdk".to_string(),
            cdk_config: Some(CdkProviderConfig {
                cdk_type: "x666".to_string(),
                cdk_source_url: "https://up.x666.me".to_string(),
                topup_path: None,
                requires_cdk_cookies: false,
                requires_access_token: true,
            }),
            oauth_config: None,
        },
        // ═══════════════════════════════════════════════════════════
        // 标准 NewAPI 公益站 (同步自 PROVIDERS.json)
        // ═══════════════════════════════════════════════════════════
        standard_provider(
            "builtin-codex-cab",
            "Codex.cab",
            "codex.cab",
            "https://codex.cab",
            "📝",
            oauth_provider_config(
                None,
                Some("nYxyCYi7VDrfjNn2rBM8VkPaKNKxWEx1"),
                "/api/oauth/state",
            ),
        ),
        standard_provider(
            "builtin-clove",
            "Clove",
            "clove.cc.cd",
            "https://clove.cc.cd",
            "🍀",
            oauth_provider_config(
                None,
                Some("Lr8C2Ny7JPr7c4YqysaDtVEqkO1a9eL7"),
                "/api/oauth/state",
            ),
        ),
        standard_provider(
            "builtin-npcodex",
            "NPCodex",
            "npcodex.kiroxubei.tech",
            "https://npcodex.kiroxubei.tech",
            "🔮",
            oauth_provider_config(
                None,
                Some("APUcB3LChvSGi3FmkODZx6Ij2038mkHY"),
                "/api/oauth/state",
            ),
        ),
        standard_provider(
            "builtin-muapi",
            "MuAPI",
            "ai.muapi.cn",
            "https://ai.muapi.cn",
            "🎵",
            oauth_provider_config(
                None,
                Some("WKD07GTaaAQcPi15BAfmIHMMCg1BG95t"),
                "/api/oauth/state",
            ),
        ),
        standard_provider(
            "builtin-feisakura",
            "Feisakura",
            "api.feisakura.fun",
            "https://api.feisakura.fun",
            "🌸",
            oauth_provider_config(
                None,
                Some("XPXmWksr3NcH2aiz0MgqK5jtEmfdfZ0Q"),
                "/api/oauth/state",
            ),
        ),
        standard_provider(
            "builtin-xionger",
            "Xionger",
            "api.xionger.ccwu.cc",
            "https://api.xionger.ccwu.cc",
            "🐻",
            oauth_provider_config(
                None,
                Some("SYU8YV8Dd0PHmBnNCcjmGIhYfDnmPtBc"),
                "/api/oauth/state",
            ),
        ),
        standard_provider(
            "builtin-einzieg",
            "Einzieg",
            "api.einzieg.site",
            "https://api.einzieg.site",
            "⚡",
            oauth_provider_config(
                None,
                Some("aBambSqvDqCgTW8fCarJBeQji8M5RATf"),
                "/api/oauth/state",
            ),
        ),
        standard_provider(
            "builtin-2020111",
            "2020111",
            "api.2020111.xyz",
            "https://api.2020111.xyz",
            "🔢",
            oauth_provider_config(
                None,
                Some("gnyvfmAfXrnYrt9ierq3Onj1ADvdVmmm"),
                "/api/oauth/state",
            ),
        ),
        standard_provider(
            "builtin-361888",
            "361888",
            "api.361888.xyz",
            "https://api.361888.xyz",
            "🎰",
            oauth_provider_config(
                None,
                Some("ze9QLEoERDgCdFnlBJeB0uASPwOTVyfM"),
                "/api/oauth/state",
            ),
        ),
        standard_provider(
            "builtin-yyds",
            "YYDS",
            "yyds.215.im",
            "https://yyds.215.im",
            "🏆",
            oauth_provider_config(
                None,
                Some("BvCzH7KoNBVpQIfdWCgUMIGaPMOpgbwI"),
                "/api/oauth/state",
            ),
        ),
        standard_provider(
            "builtin-anthorpic",
            "Anthorpic",
            "anthorpic.us.ci",
            "https://anthorpic.us.ci",
            "🤖",
            oauth_provider_config(
                None,
                Some("nNzrggkmAew2bJYxCgC2iaU6IYcaWt8S"),
                "/api/oauth/state",
            ),
        ),
        standard_provider(
            "builtin-nanohajimi",
            "Nanohajimi",
            "free.nanohajimi.mom",
            "https://free.nanohajimi.mom",
            "🌟",
            oauth_provider_config(
                None,
                Some("svkUqtRyhOJMULQ1Zfnfhvv9ALSnANhf"),
                "/api/oauth/state",
            ),
        ),
        standard_provider(
            "builtin-zapi-aicc0",
            "ZAPI",
            "zapi.aicc0.com",
            "https://zapi.aicc0.com",
            "🧩",
            oauth_provider_config(
                Some("Ov23linrAuNoCCMoztG7"),
                Some("Tm30iXRcGTM9oyVreW3edvCNGO5kPEWX"),
                "/api/oauth/state",
            ),
        ),
        standard_provider(
            "builtin-llmapi-vhuds",
            "LLMAPI",
            "llmapi.vhuds.com",
            "https://llmapi.vhuds.com",
            "💡",
            oauth_provider_config(
                None,
                Some("aiepRPewQgwTPbUIq8Z4muSYw76NFSUZ"),
                "/api/oauth/state",
            ),
        ),
    ]
}

/// 根据 ID 获取内置提供商
pub fn get_builtin_provider_by_id(id: &str) -> Option<BuiltinProvider> {
    get_builtin_providers().into_iter().find(|p| p.id == id)
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

    #[test]
    fn test_get_builtin_providers() {
        let providers = get_builtin_providers();
        assert_eq!(providers.len(), 22);
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
}
