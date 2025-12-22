// 📦 内置中转站提供商配置
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
    /// 签到功能是否有 bug (如 AgentRouter 自动签到)
    pub checkin_bugged: bool,
    /// 图标 (emoji 或 URL)
    pub icon: String,
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

/// 获取所有内置提供商
pub fn get_builtin_providers() -> Vec<BuiltinProvider> {
    vec![
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
            checkin_bugged: false,
            icon: "🌐".to_string(),
        },
        // AgentRouter - 无需 WAF，但签到功能有 bug
        BuiltinProvider {
            id: "builtin-agentrouter".to_string(),
            name: "AgentRouter".to_string(),
            description: "AI 代理路由站，查询用户信息时自动签到".to_string(),
            domain: "agentrouter.org".to_string(),
            base_url: "https://agentrouter.org".to_string(),
            checkin_path: None, // AgentRouter 没有专门的签到接口
            balance_path: "/api/user/self".to_string(),
            user_info_path: "/api/user/self".to_string(),
            auth_header: "Authorization".to_string(),
            auth_prefix: "Bearer".to_string(),
            supports_checkin: true, // 通过 user_info 自动签到
            requires_waf_bypass: false,
            checkin_bugged: true, // 自动签到机制
            icon: "🤖".to_string(),
        },
        // CodeRouter - 代码镜像中转站，不支持签到
        BuiltinProvider {
            id: "builtin-coderouter".to_string(),
            name: "CodeRouter".to_string(),
            description: "代码镜像 AI 中转站，无签到功能".to_string(),
            domain: "api.codemirror.codes".to_string(),
            base_url: "https://api.codemirror.codes".to_string(),
            checkin_path: None,
            balance_path: "/api/user/self".to_string(),
            user_info_path: "/api/user/self".to_string(),
            auth_header: "Authorization".to_string(),
            auth_prefix: "Bearer".to_string(),
            supports_checkin: false,
            requires_waf_bypass: false,
            checkin_bugged: false,
            icon: "💻".to_string(),
        },
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
        assert_eq!(providers.len(), 3);

        // 验证 AnyRouter
        let anyrouter = providers.iter().find(|p| p.name == "AnyRouter").unwrap();
        assert!(anyrouter.supports_checkin);
        assert!(anyrouter.requires_waf_bypass);

        // 验证 AgentRouter
        let agentrouter = providers.iter().find(|p| p.name == "AgentRouter").unwrap();
        assert!(agentrouter.checkin_bugged);

        // 验证 CodeRouter
        let coderouter = providers.iter().find(|p| p.name == "CodeRouter").unwrap();
        assert!(!coderouter.supports_checkin);
    }

    #[test]
    fn test_to_checkin_provider() {
        let builtin = get_builtin_providers().into_iter().next().unwrap();
        let provider = builtin.to_checkin_provider();

        assert_eq!(provider.name, builtin.name);
        assert_eq!(provider.base_url, builtin.base_url);
        assert!(provider.enabled);
    }

    #[test]
    fn test_is_builtin_provider_id() {
        assert!(is_builtin_provider_id("builtin-anyrouter"));
        assert!(is_builtin_provider_id("builtin-agentrouter"));
        assert!(!is_builtin_provider_id("custom-provider-123"));
    }
}
