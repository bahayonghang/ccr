// ⚙️ 配置类型定义
// 📦 ProviderType, ConfigSection, GlobalSettings

use crate::core::error::{CcrError, Result};
use crate::managers::sync_config::SyncConfig;
use crate::utils::{AutoCompletable, Validatable};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// 🏢 提供商类型枚举
///
/// 用于分类不同类型的 API 服务提供商
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProviderType {
    /// 官方中转 - 提供官方 Claude 模型的中转服务
    OfficialRelay,
    /// 第三方模型 - 提供自己的模型服务（如 GLM、Kimi 等）
    ThirdPartyModel,
}

impl ProviderType {
    /// 获取类型的显示名称
    pub fn display_name(&self) -> &str {
        match self {
            ProviderType::OfficialRelay => "官方中转",
            ProviderType::ThirdPartyModel => "第三方模型",
        }
    }

    /// 🆕 获取序列化字符串值（用于 API）
    /// 返回 "official_relay" 或 "third_party_model"
    pub fn to_string_value(&self) -> &str {
        match self {
            ProviderType::OfficialRelay => "official_relay",
            ProviderType::ThirdPartyModel => "third_party_model",
        }
    }
}

/// 📝 配置节结构
///
/// 代表一个具体的 API 配置(如 anthropic、anyrouter 等)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfigSection {
    /// 📝 配置描述(可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// 🌐 API 基础 URL(切换时必需)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,

    /// 🔑 认证令牌(切换时必需)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_token: Option<String>,

    /// 🤖 默认模型名称(可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// ⚡ 快速小模型名称(可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub small_fast_model: Option<String>,

    /// 🏢 提供商名称（如 "anyrouter", "glm", "moonshot"）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,

    /// 🏷️ 提供商类型（官方中转/第三方模型）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_type: Option<ProviderType>,

    /// 👤 账号标识（用于区分同一提供商的不同账号）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,

    /// 🏷️ 标签列表（用于灵活分类和筛选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,

    /// 📊 使用次数统计
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage_count: Option<u32>,

    /// 🔘 启用/禁用状态
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// 📦 额外字段（平台特定/向前兼容）
    #[serde(default, flatten)]
    pub other: IndexMap<String, toml::Value>,
}

impl Validatable for ConfigSection {
    fn validate(&self) -> Result<()> {
        // 🌐 检查 base_url
        let base_url = self
            .base_url
            .as_ref()
            .ok_or_else(|| CcrError::ValidationError("base_url 不能为空".into()))?;

        if base_url.trim().is_empty() {
            return Err(CcrError::ValidationError("base_url 不能为空".into()));
        }

        if !base_url.starts_with("http://") && !base_url.starts_with("https://") {
            return Err(CcrError::ValidationError(
                "base_url 必须以 http:// 或 https:// 开头".into(),
            ));
        }

        // 🔑 检查 auth_token
        let auth_token = self
            .auth_token
            .as_ref()
            .ok_or_else(|| CcrError::ValidationError("auth_token 不能为空".into()))?;

        if auth_token.trim().is_empty() {
            return Err(CcrError::ValidationError("auth_token 不能为空".into()));
        }

        // 🤖 检查 model(可选,如果提供了则不能为空)
        if let Some(model) = &self.model
            && model.trim().is_empty()
        {
            return Err(CcrError::ValidationError("model 不能为空字符串".into()));
        }

        Ok(())
    }
}

#[allow(dead_code)]
impl ConfigSection {
    /// 📝 获取配置的人类可读描述
    pub fn display_description(&self) -> &str {
        self.description.as_deref().unwrap_or("(无描述)")
    }

    /// 🏢 获取提供商显示名称
    #[allow(dead_code)]
    pub fn provider_display(&self) -> &str {
        self.provider.as_deref().unwrap_or("未分类")
    }

    /// 🏷️ 获取提供商类型显示名称
    pub fn provider_type_display(&self) -> &str {
        self.provider_type
            .as_ref()
            .map(|t| t.display_name())
            .unwrap_or("未分类")
    }

    /// 👤 获取账号显示名称
    pub fn account_display(&self) -> &str {
        self.account.as_deref().unwrap_or("")
    }

    /// 🏷️ 检查是否有指定标签
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags
            .as_ref()
            .map(|tags| tags.iter().any(|t| t == tag))
            .unwrap_or(false)
    }

    /// 📋 获取所有标签
    pub fn tags_display(&self) -> String {
        self.tags
            .as_ref()
            .map(|tags| tags.join(", "))
            .unwrap_or_default()
    }

    /// 📊 获取使用次数
    pub fn usage_count(&self) -> u32 {
        self.usage_count.unwrap_or(0)
    }

    /// 🔘 检查是否启用
    pub fn is_enabled(&self) -> bool {
        self.enabled.unwrap_or(true)
    }

    /// 📈 递增使用次数
    pub fn increment_usage(&mut self) {
        let count = self.usage_count.unwrap_or(0);
        self.usage_count = Some(count + 1);
    }

    /// 📊 获取预期的 ANTHROPIC_* 环境变量状态（无副作用）
    ///
    /// 根据配置节的字段值，返回应用后的环境变量状态
    /// 用于在不实际修改 settings 的情况下预览环境变量变化
    pub fn to_anthropic_env_status(&self) -> std::collections::HashMap<String, Option<String>> {
        use std::collections::HashMap;

        let mut status = HashMap::new();

        // 映射关系与 ClaudeSettings::update_from_config 保持一致
        status.insert("ANTHROPIC_BASE_URL".to_string(), self.base_url.clone());
        status.insert("ANTHROPIC_AUTH_TOKEN".to_string(), self.auth_token.clone());
        status.insert("ANTHROPIC_MODEL".to_string(), self.model.clone());
        status.insert(
            "ANTHROPIC_SMALL_FAST_MODEL".to_string(),
            self.small_fast_model.clone(),
        );

        status
    }

    /// ✅ 启用配置
    pub fn enable(&mut self) {
        self.enabled = Some(true);
    }

    /// ❌ 禁用配置
    pub fn disable(&mut self) {
        self.enabled = Some(false);
    }
}

impl AutoCompletable for ConfigSection {
    fn auto_complete(&mut self) -> bool {
        let mut modified = false;

        if self.usage_count.is_none() {
            self.usage_count = Some(0);
            modified = true;
            tracing::debug!("Auto-completed usage_count field for config");
        }

        if self.enabled.is_none() {
            self.enabled = Some(true);
            modified = true;
            tracing::debug!("Auto-completed enabled field for config");
        }

        modified
    }
}

/// ⚙️ 全局设置结构
///
/// 用于存储 CCR 的全局配置选项
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GlobalSettings {
    /// ⚡ 自动确认模式 - 跳过所有确认提示
    #[serde(default, alias = "yolo_mode")]
    pub skip_confirmation: bool,

    /// 🎨 TUI 主题名称 (预留字段)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tui_theme: Option<String>,

    /// ☁️ WebDAV 同步配置（已废弃，保留向后兼容）
    #[serde(skip_serializing_if = "Option::is_none")]
    #[deprecated(note = "Use SyncConfigManager to manage sync configuration")]
    pub sync: Option<SyncConfig>,
}
