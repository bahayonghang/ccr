// 🎯 临时配置覆盖管理模块
// 📝 负责管理临时的配置覆盖,不修改永久配置文件
//
// 核心职责:
// - 🔄 临时覆盖 auth_token, base_url, model 等字段
// - 💾 存储到独立的 JSON 文件 (~/.claude/temp_override.json)
// - ⏰ 支持过期时间管理
// - 🧹 自动清理过期配置

use ccr_core::core::error::{CcrError, Result};
use ccr_core::{Secret, expose_plaintext_option};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tokio::fs as async_fs;

/// 🎯 临时配置覆盖结构
///
/// 用于临时覆盖当前配置的某些字段,而不修改 toml 配置文件
///
/// 使用场景:
/// - 临时使用免费 token 测试（一次性使用，用完自动清除）
/// - 临时切换 API 端点
/// - 临时使用不同模型
///
/// 设计原则:
/// - **一次性使用**：应用后自动清除，不需要手动清理
/// - **简单直接**：设置后下次 switch 时自动应用并清除
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TempOverride {
    /// 🔑 临时认证令牌(最常用的覆盖字段)：Debug/日志/默认序列化恒掩码；
    /// 落盘原文走 expose_plaintext_option 注解
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "expose_plaintext_option"
    )]
    pub auth_token: Option<Secret>,

    /// 🌐 临时 API 基础 URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,

    /// 🤖 临时默认模型
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// ⚡ 临时快速小模型
    #[serde(skip_serializing_if = "Option::is_none")]
    pub small_fast_model: Option<String>,

    /// 📅 创建时间
    pub created_at: DateTime<Utc>,

    /// 📝 备注说明(可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl TempOverride {
    /// 🏗️ 创建新的临时配置覆盖
    ///
    /// 参数:
    /// - auth_token: 临时 token
    pub fn new(auth_token: String) -> Self {
        Self {
            auth_token: Some(Secret::new(auth_token)),
            base_url: None,
            model: None,
            small_fast_model: None,
            created_at: Utc::now(),
            note: None,
        }
    }

    /// 🔍 获取覆盖字段的数量
    pub fn override_count(&self) -> usize {
        let mut count = 0;
        if self.auth_token.is_some() {
            count += 1;
        }
        if self.base_url.is_some() {
            count += 1;
        }
        if self.model.is_some() {
            count += 1;
        }
        if self.small_fast_model.is_some() {
            count += 1;
        }
        count
    }

    /// 📊 检查是否有任何覆盖字段
    #[allow(dead_code)]
    pub fn has_any_override(&self) -> bool {
        self.auth_token.is_some()
            || self.base_url.is_some()
            || self.model.is_some()
            || self.small_fast_model.is_some()
    }
}

/// 🔧 临时配置覆盖管理器
///
/// 负责临时配置的加载、保存和管理
pub struct TempOverrideManager {
    override_path: PathBuf,
}

#[allow(dead_code)]
impl TempOverrideManager {
    /// 🏗️ 创建新的临时配置管理器
    pub fn new<P: AsRef<Path>>(override_path: P) -> Self {
        Self {
            override_path: override_path.as_ref().to_path_buf(),
        }
    }

    /// 🏠 使用默认路径创建管理器
    ///
    /// 默认路径: ~/.claude/temp_override.json
    ///
    /// ⚙️ **开发者注意**:
    /// 可以通过环境变量 `CCR_TEMP_OVERRIDE_PATH` 覆盖默认路径
    pub fn with_default() -> Result<Self> {
        let override_path = if let Ok(custom_path) = std::env::var("CCR_TEMP_OVERRIDE_PATH") {
            PathBuf::from(custom_path)
        } else {
            let home = dirs::home_dir()
                .ok_or_else(|| CcrError::ConfigError("无法获取用户主目录".into()))?;
            home.join(".claude").join("temp_override.json")
        };

        tracing::debug!("使用临时配置路径: {:?}", override_path);
        Ok(Self::new(override_path))
    }

    /// 📁 获取临时配置文件路径
    #[allow(dead_code)]
    pub fn override_path(&self) -> &Path {
        &self.override_path
    }

    /// 📖 加载临时配置
    ///
    /// 返回:
    /// - Some(TempOverride): 存在临时配置
    /// - None: 不存在
    pub fn load(&self) -> Result<Option<TempOverride>> {
        // 文件不存在时返回 None
        if !self.override_path.exists() {
            return Ok(None);
        }

        // 读取文件内容
        let content = fs::read_to_string(&self.override_path)
            .map_err(|e| CcrError::ConfigError(format!("读取临时配置文件失败: {}", e)))?;

        // 解析 JSON
        let temp_override: TempOverride = serde_json::from_str(&content)
            .map_err(|e| CcrError::ConfigError(format!("解析临时配置文件失败: {}", e)))?;

        tracing::debug!(
            "✅ 成功加载临时配置: {} 个字段覆盖",
            temp_override.override_count()
        );
        Ok(Some(temp_override))
    }

    /// 📖 异步加载临时配置
    pub async fn load_async(&self) -> Result<Option<TempOverride>> {
        let exists = async_fs::try_exists(&self.override_path)
            .await
            .map_err(|e| CcrError::ConfigError(format!("检查临时配置文件失败: {}", e)))?;
        if !exists {
            return Ok(None);
        }

        let content = async_fs::read_to_string(&self.override_path)
            .await
            .map_err(|e| CcrError::ConfigError(format!("读取临时配置文件失败: {}", e)))?;

        let temp_override: TempOverride = serde_json::from_str(&content)
            .map_err(|e| CcrError::ConfigError(format!("解析临时配置文件失败: {}", e)))?;

        tracing::debug!(
            "✅ 成功加载临时配置: {} 个字段覆盖",
            temp_override.override_count()
        );
        Ok(Some(temp_override))
    }

    /// 💾 保存临时配置
    pub fn save(&self, temp_override: &TempOverride) -> Result<()> {
        // 📁 确保目录存在
        if let Some(parent) = self.override_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| CcrError::ConfigError(format!("创建临时配置目录失败: {}", e)))?;
        }

        // 📝 序列化为 JSON(美化格式)
        let content = serde_json::to_string_pretty(temp_override)
            .map_err(|e| CcrError::ConfigError(format!("序列化临时配置失败: {}", e)))?;

        // 💾 写入文件
        fs::write(&self.override_path, content)
            .map_err(|e| CcrError::ConfigError(format!("写入临时配置文件失败: {}", e)))?;

        tracing::info!("✅ 临时配置已保存: {:?}", self.override_path);
        Ok(())
    }

    /// 💾 异步保存临时配置
    pub async fn save_async(&self, temp_override: &TempOverride) -> Result<()> {
        if let Some(parent) = self.override_path.parent() {
            async_fs::create_dir_all(parent)
                .await
                .map_err(|e| CcrError::ConfigError(format!("创建临时配置目录失败: {}", e)))?;
        }

        let content = serde_json::to_string_pretty(temp_override)
            .map_err(|e| CcrError::ConfigError(format!("序列化临时配置失败: {}", e)))?;

        async_fs::write(&self.override_path, content)
            .await
            .map_err(|e| CcrError::ConfigError(format!("写入临时配置文件失败: {}", e)))?;

        tracing::info!("✅ 临时配置已保存: {:?}", self.override_path);
        Ok(())
    }

    /// 🧹 清除临时配置
    pub fn clear(&self) -> Result<()> {
        if self.override_path.exists() {
            fs::remove_file(&self.override_path)
                .map_err(|e| CcrError::ConfigError(format!("删除临时配置文件失败: {}", e)))?;
            tracing::info!("✅ 临时配置已清除");
        } else {
            tracing::debug!("临时配置文件不存在,无需清除");
        }
        Ok(())
    }

    /// 🧹 异步清除临时配置
    pub async fn clear_async(&self) -> Result<()> {
        let exists = async_fs::try_exists(&self.override_path)
            .await
            .map_err(|e| CcrError::ConfigError(format!("检查临时配置文件失败: {}", e)))?;
        if exists {
            async_fs::remove_file(&self.override_path)
                .await
                .map_err(|e| CcrError::ConfigError(format!("删除临时配置文件失败: {}", e)))?;
            tracing::info!("✅ 临时配置已清除");
        } else {
            tracing::debug!("临时配置文件不存在,无需清除");
        }
        Ok(())
    }

    /// 🔍 检查是否存在临时配置
    pub fn exists(&self) -> bool {
        self.load().ok().flatten().is_some()
    }

    /// 🔍 异步检查是否存在临时配置
    pub async fn exists_async(&self) -> Result<bool> {
        async_fs::try_exists(&self.override_path)
            .await
            .map_err(|e| CcrError::ConfigError(format!("检查临时配置文件失败: {}", e)))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_temp_override_creation() {
        let temp = TempOverride::new("sk-test-token".to_string());
        assert_eq!(temp.auth_token, Some(Secret::from("sk-test-token")));
    }

    #[test]
    fn test_temp_override_manager_save_load() {
        let temp_dir = tempfile::tempdir().unwrap();
        let override_path = temp_dir.path().join("temp_override.json");

        let manager = TempOverrideManager::new(&override_path);

        // 创建并保存
        let temp = TempOverride::new("sk-test-token".to_string());
        manager.save(&temp).unwrap();
        assert!(override_path.exists());

        // 加载并验证
        let loaded = manager.load().unwrap();
        assert!(loaded.is_some());
        let loaded_temp = loaded.unwrap();
        assert_eq!(loaded_temp.auth_token, Some(Secret::from("sk-test-token")));

        // 落盘为明文原文（expose_plaintext_option 注解），旧版本可无损读回
        let on_disk = fs::read_to_string(&override_path).unwrap();
        assert!(on_disk.contains("sk-test-token"));
    }

    #[test]
    fn test_temp_override_manager_clear() {
        let temp_dir = tempfile::tempdir().unwrap();
        let override_path = temp_dir.path().join("temp_override.json");

        let manager = TempOverrideManager::new(&override_path);

        // 创建并保存
        let temp = TempOverride::new("sk-test-token".to_string());
        manager.save(&temp).unwrap();
        assert!(override_path.exists());

        // 清除
        manager.clear().unwrap();
        assert!(!override_path.exists());
    }

    #[test]
    fn test_temp_override_fields() {
        let mut temp = TempOverride::new("sk-test".to_string());
        assert_eq!(temp.override_count(), 1);

        temp.base_url = Some("https://test.com".to_string());
        assert_eq!(temp.override_count(), 2);

        temp.model = Some("test-model".to_string());
        assert_eq!(temp.override_count(), 3);

        assert!(temp.has_any_override());
    }
}
