// ⚙️ CCR 配置管理模块
// 📁 负责读写和管理 ~/.ccs_config.toml 配置文件
//
// 核心职责:
// - 🔍 解析 TOML 配置文件
// - 💾 保存配置到文件
// - ✅ 验证配置完整性
// - 📋 管理多个配置节

use crate::core::error::{CcrError, Result};
use crate::utils::Validatable;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

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
    #[allow(dead_code)]
    pub fn display_name(&self) -> &str {
        match self {
            ProviderType::OfficialRelay => "官方中转",
            ProviderType::ThirdPartyModel => "第三方模型",
        }
    }

    /// 获取类型的图标（用于 CLI 显示）
    #[allow(dead_code)]
    pub fn icon(&self) -> &str {
        match self {
            ProviderType::OfficialRelay => "🔄",
            ProviderType::ThirdPartyModel => "🤖",
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
///
/// 每个配置节包含:
/// - 🏷️ 描述信息
/// - 🌐 API 基础 URL
/// - 🔑 认证令牌
/// - 🤖 模型配置
/// - 🏢 提供商信息（新增分类字段）
#[derive(Debug, Clone, Serialize, Deserialize)]
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

    // === 🆕 分类字段 ===
    /// 🏢 提供商名称（如 "anyrouter", "glm", "moonshot"）
    /// 用于标识同一提供商的不同配置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,

    /// 🏷️ 提供商类型（官方中转/第三方模型）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_type: Option<ProviderType>,

    /// 👤 账号标识（用于区分同一提供商的不同账号）
    /// 如 "github_5953", "linuxdo_79797"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,

    /// 🏷️ 标签列表（用于灵活分类和筛选）
    /// 如 ["free", "stable", "high-speed"]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

impl Validatable for ConfigSection {
    /// ✅ 验证配置节的完整性
    ///
    /// 验证规则:
    /// 1. 🌐 base_url 必须存在且符合 URL 格式
    /// 2. 🔑 auth_token 必须存在且非空
    /// 3. 🤖 model 如果提供则不能为空字符串
    fn validate(&self) -> Result<()> {
        // 🌐 检查 base_url
        let base_url = self
            .base_url
            .as_ref()
            .ok_or_else(|| CcrError::ValidationError("base_url 不能为空".into()))?;

        if base_url.trim().is_empty() {
            return Err(CcrError::ValidationError("base_url 不能为空".into()));
        }

        // 🔍 简单的 URL 格式验证
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

impl ConfigSection {
    /// 📝 获取配置的人类可读描述
    /// 🎯 优化：返回 &str 避免克隆
    pub fn display_description(&self) -> &str {
        self.description.as_deref().unwrap_or("(无描述)")
    }

    /// 🏢 获取提供商显示名称
    #[allow(dead_code)]
    pub fn provider_display(&self) -> &str {
        self.provider.as_deref().unwrap_or("未分类")
    }

    /// 🏷️ 获取提供商类型显示名称
    #[allow(dead_code)]
    pub fn provider_type_display(&self) -> &str {
        self.provider_type
            .as_ref()
            .map(|t| t.display_name())
            .unwrap_or("未分类")
    }

    /// 🎨 获取提供商类型图标
    #[allow(dead_code)]
    pub fn provider_type_icon(&self) -> &str {
        self.provider_type
            .as_ref()
            .map(|t| t.icon())
            .unwrap_or("❓")
    }

    /// 👤 获取账号显示名称
    #[allow(dead_code)]
    pub fn account_display(&self) -> &str {
        self.account.as_deref().unwrap_or("")
    }

    /// 🏷️ 检查是否有指定标签
    #[allow(dead_code)]
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags
            .as_ref()
            .map(|tags| tags.iter().any(|t| t == tag))
            .unwrap_or(false)
    }

    /// 📋 获取所有标签
    #[allow(dead_code)]
    pub fn tags_display(&self) -> String {
        self.tags
            .as_ref()
            .map(|tags| tags.join(", "))
            .unwrap_or_default()
    }
}

/// ⚙️ 全局设置结构
///
/// 用于存储 CCR 的全局配置选项
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GlobalSettings {
    /// ⚡ 自动确认模式 - 跳过所有确认提示（便利功能）
    ///
    /// **功能说明**：
    /// 启用后，所有需要人工确认的操作将自动执行，无需手动输入 'y' 确认
    ///
    /// **启用后的行为**：
    /// - 删除配置：跳过 "确认删除?" 提示
    /// - 替换配置：跳过 "确认替换?" 提示
    /// - 覆盖文件：跳过 "确认覆盖?" 提示
    /// - 清理备份：跳过 "确认清理?" 提示
    ///
    /// ⚠️ **注意事项**：
    /// - 这是用户便利性功能，不影响安全机制
    /// - 所有操作仍会自动备份
    /// - 所有操作仍会记录到历史
    /// - 仍由人类手动执行命令
    ///
    /// **建议用法**：
    /// - ✅ CI/CD 管道中使用（避免交互阻塞）
    /// - ✅ 自动化脚本中使用
    /// - ✅ 批量操作时使用
    /// - ⚠️ 谨慎在生产环境使用
    ///
    /// **等效于**：
    /// 在每个命令后添加 `--yes` 或 `-y` 参数
    #[serde(default, alias = "yolo_mode")]
    pub skip_confirmation: bool,

    /// 🎨 TUI 主题名称 (预留字段)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tui_theme: Option<String>,

    /// ☁️ WebDAV 同步配置（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync: Option<SyncConfig>,
}

/// ☁️ WebDAV 同步配置结构
///
/// 用于配置文件的云端同步，默认支持坚果云
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// 🔌 是否启用同步功能
    #[serde(default)]
    pub enabled: bool,

    /// 🌐 WebDAV 服务器地址
    ///
    /// 坚果云默认地址: https://dav.jianguoyun.com/dav/
    /// 其他WebDAV服务器也支持
    pub webdav_url: String,

    /// 👤 用户名
    ///
    /// 对于坚果云，这是您的邮箱地址
    pub username: String,

    /// 🔑 密码/应用密码
    ///
    /// ⚠️ 对于坚果云，请使用"应用密码"而非账户密码
    /// 获取方式：账户信息 -> 安全选项 -> 添加应用 -> 生成密码
    pub password: String,

    /// 📁 远程文件路径
    ///
    /// 配置文件在WebDAV服务器上的路径
    /// 默认: /ccr/.ccs_config.toml
    #[serde(default = "default_remote_path")]
    pub remote_path: String,

    /// ⚡ 自动同步模式
    ///
    /// 启用后，每次配置操作后自动同步到云端
    #[serde(default)]
    pub auto_sync: bool,
}

/// 默认远程路径
fn default_remote_path() -> String {
    "/ccr/.ccs_config.toml".to_string()
}

/// 📦 CCS 配置文件总体结构
///
/// 对应 ~/.ccs_config.toml 的完整结构
///
/// 结构说明:
/// - 🎯 default_config: 默认配置名
/// - ▶️ current_config: 当前激活配置
/// - ⚙️ settings: 全局设置
/// - 📋 sections: 所有具体配置节的集合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CcsConfig {
    /// 🎯 默认配置名称
    pub default_config: String,

    /// ▶️ 当前活跃配置名称
    pub current_config: String,

    /// ⚙️ 全局设置
    #[serde(default)]
    pub settings: GlobalSettings,

    /// 📋 所有配置节(使用 flatten 序列化)
    #[serde(flatten)]
    pub sections: IndexMap<String, ConfigSection>,
}

impl CcsConfig {
    /// 🔍 获取指定配置节
    pub fn get_section(&self, name: &str) -> Result<&ConfigSection> {
        self.sections
            .get(name)
            .ok_or_else(|| CcrError::ConfigSectionNotFound(name.to_string()))
    }

    /// ▶️ 获取当前配置节
    pub fn get_current_section(&self) -> Result<&ConfigSection> {
        self.get_section(&self.current_config)
    }

    /// 🔄 设置当前配置
    ///
    /// 切换前会验证目标配置是否存在
    pub fn set_current(&mut self, name: &str) -> Result<()> {
        // ✅ 验证配置节存在
        if !self.sections.contains_key(name) {
            return Err(CcrError::ConfigSectionNotFound(name.to_string()));
        }
        self.current_config = name.to_string();
        Ok(())
    }

    /// ➕ 添加或更新配置节
    #[allow(dead_code)]
    pub fn set_section(&mut self, name: String, section: ConfigSection) {
        self.sections.insert(name, section);
    }

    /// ➖ 删除配置节
    #[allow(dead_code)]
    pub fn remove_section(&mut self, name: &str) -> Result<ConfigSection> {
        self.sections
            .shift_remove(name)
            .ok_or_else(|| CcrError::ConfigSectionNotFound(name.to_string()))
    }

    /// 📜 列出所有配置节名称(已排序)
    /// 🎯 优化：返回迭代器避免分配，由调用方决定是否需要 Vec
    pub fn list_sections(&self) -> impl Iterator<Item = &String> {
        let mut names: Vec<&String> = self.sections.keys().collect();
        names.sort();
        names.into_iter()
    }

    /// 🔄 按配置节名称排序
    ///
    /// 将所有配置节按照名称的字母顺序重新排列
    /// 这会直接修改内部的 IndexMap 顺序
    /// 🎯 优化：使用 IndexMap 原生的 sort_by 方法，避免重新分配
    pub fn sort_sections(&mut self) {
        self.sections.sort_by(|k1, _, k2, _| k1.cmp(k2));
    }

    // === 🆕 分类和筛选方法 ===

    /// 🏢 按提供商分组获取配置
    ///
    /// 返回 HashMap<提供商名称, Vec<配置名称>>
    #[allow(dead_code)]
    pub fn group_by_provider(&self) -> IndexMap<String, Vec<String>> {
        let mut groups: IndexMap<String, Vec<String>> = IndexMap::new();

        for (name, section) in &self.sections {
            let provider = section.provider_display().to_string();
            groups.entry(provider).or_default().push(name.clone());
        }

        // 排序每个组内的配置名称
        for configs in groups.values_mut() {
            configs.sort();
        }

        groups
    }

    /// 🏷️ 按提供商类型分组获取配置
    ///
    /// 返回 HashMap<提供商类型, Vec<配置名称>>
    #[allow(dead_code)]
    pub fn group_by_provider_type(&self) -> IndexMap<String, Vec<String>> {
        let mut groups: IndexMap<String, Vec<String>> = IndexMap::new();

        for (name, section) in &self.sections {
            let provider_type = section.provider_type_display().to_string();
            groups.entry(provider_type).or_default().push(name.clone());
        }

        // 排序每个组内的配置名称
        for configs in groups.values_mut() {
            configs.sort();
        }

        groups
    }

    /// 🔍 按标签筛选配置
    ///
    /// 返回包含指定标签的所有配置名称
    #[allow(dead_code)]
    pub fn filter_by_tag(&self, tag: &str) -> Vec<String> {
        let mut names: Vec<String> = self
            .sections
            .iter()
            .filter(|(_, section)| section.has_tag(tag))
            .map(|(name, _)| name.clone())
            .collect();

        names.sort();
        names
    }

    /// 🔍 按提供商筛选配置
    ///
    /// 返回属于指定提供商的所有配置名称
    #[allow(dead_code)]
    pub fn filter_by_provider(&self, provider: &str) -> Vec<String> {
        let mut names: Vec<String> = self
            .sections
            .iter()
            .filter(|(_, section)| section.provider.as_deref() == Some(provider))
            .map(|(name, _)| name.clone())
            .collect();

        names.sort();
        names
    }

    /// 🔍 按提供商类型筛选配置
    ///
    /// 返回属于指定提供商类型的所有配置名称
    #[allow(dead_code)]
    pub fn filter_by_provider_type(&self, provider_type: &ProviderType) -> Vec<String> {
        let mut names: Vec<String> = self
            .sections
            .iter()
            .filter(|(_, section)| section.provider_type.as_ref() == Some(provider_type))
            .map(|(name, _)| name.clone())
            .collect();

        names.sort();
        names
    }
}

/// 🔧 配置管理器
///
/// 负责配置文件的加载、保存和管理
///
/// 主要功能:
/// - 📖 从磁盘加载 TOML 配置
/// - 💾 保存配置到磁盘
/// - 🔍 解析和验证配置格式
pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    /// 🏗️ 创建新的配置管理器
    pub fn new<P: AsRef<Path>>(config_path: P) -> Self {
        Self {
            config_path: config_path.as_ref().to_path_buf(),
        }
    }

    /// 🏠 使用默认配置路径创建管理器
    ///
    /// 默认路径: ~/.ccs_config.toml
    ///
    /// ⚙️ **开发者注意**：
    /// 可以通过环境变量 `CCR_CONFIG_PATH` 覆盖默认路径
    /// 这样在开发时不会影响本地真实配置
    ///
    /// 示例：
    /// ```bash
    /// export CCR_CONFIG_PATH=/tmp/ccr_dev_config.toml
    /// cargo run -- init
    /// ```
    pub fn default() -> Result<Self> {
        // 🔍 首先检测是否为 Unified 模式
        let (is_unified, unified_config_path) = Self::detect_unified_mode();

        if is_unified {
            // 📦 Unified 模式：读取平台配置，获取当前平台的 profiles 路径
            if let Some(ref unified_path) = unified_config_path {
                let unified_root = unified_path.parent()
                    .ok_or_else(|| CcrError::ConfigError("无法获取 CCR 根目录".into()))?;

                // 读取统一配置文件以获取当前平台
                let platform_config_manager = crate::managers::PlatformConfigManager::new(unified_path.clone());
                if let Ok(unified_config) = platform_config_manager.load() {
                    let platform = &unified_config.current_platform;
                    let platform_profiles_path = unified_root
                        .join("platforms")
                        .join(platform)
                        .join("profiles.toml");

                    log::debug!("🔄 Unified 模式: 使用平台 {} 的配置路径: {:?}", platform, platform_profiles_path);
                    return Ok(Self::new(platform_profiles_path));
                }
            }
        }

        // 🔍 Legacy 模式或 Unified 配置加载失败：检查环境变量
        let config_path = if let Ok(custom_path) = std::env::var("CCR_CONFIG_PATH") {
            std::path::PathBuf::from(custom_path)
        } else {
            let home = dirs::home_dir()
                .ok_or_else(|| CcrError::ConfigError("无法获取用户主目录".into()))?;
            home.join(".ccs_config.toml")
        };

        log::debug!("📁 Legacy 模式: 使用配置路径: {:?}", config_path);
        Ok(Self::new(config_path))
    }

    /// 📁 获取配置文件路径
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    /// 📖 加载配置文件
    ///
    /// 执行步骤:
    /// 1. ✅ 检查文件是否存在
    /// 2. 📄 读取文件内容
    /// 3. 🔍 解析 TOML 格式
    pub fn load(&self) -> Result<CcsConfig> {
        // ✅ 检查文件是否存在
        if !self.config_path.exists() {
            return Err(CcrError::ConfigMissing(
                self.config_path.display().to_string(),
            ));
        }

        // 📄 读取文件内容
        let content = fs::read_to_string(&self.config_path)
            .map_err(|e| CcrError::ConfigError(format!("读取配置文件失败: {}", e)))?;

        // 🔍 解析 TOML
        let config: CcsConfig = toml::from_str(&content)
            .map_err(|e| CcrError::ConfigFormatInvalid(format!("TOML 解析失败: {}", e)))?;

        log::debug!(
            "✅ 成功加载配置文件: {:?}, 配置节数量: {}",
            self.config_path,
            config.sections.len()
        );

        Ok(config)
    }

    /// 💾 保存配置文件
    ///
    /// 执行步骤:
    /// 1. 📝 序列化为 TOML 格式
    /// 2. 💾 写入磁盘
    pub fn save(&self, config: &CcsConfig) -> Result<()> {
        // 📝 序列化为 TOML(美化格式)
        let content = toml::to_string_pretty(config)
            .map_err(|e| CcrError::ConfigError(format!("配置序列化失败: {}", e)))?;

        // 💾 写入文件
        fs::write(&self.config_path, content)
            .map_err(|e| CcrError::ConfigError(format!("写入配置文件失败: {}", e)))?;

        log::debug!("✅ 配置文件已保存: {:?}", self.config_path);
        Ok(())
    }

    /// 💾 备份配置文件
    ///
    /// 执行流程:
    /// 1. ✅ 验证源文件存在
    /// 2. 🏷️ 生成带时间戳的备份文件名
    /// 3. 📋 复制文件到备份位置
    /// 4. 🧹 自动清理旧备份(只保留最近10个)
    ///
    /// 文件名格式:
    /// - 有标签: .ccs_config.toml.{tag}_{timestamp}.bak
    /// - 无标签: .ccs_config.toml.{timestamp}.bak
    ///
    /// 备份位置: 与配置文件同目录
    pub fn backup(&self, tag: Option<&str>) -> Result<PathBuf> {
        // ✅ 验证源文件存在
        if !self.config_path.exists() {
            return Err(CcrError::ConfigMissing(
                self.config_path.display().to_string(),
            ));
        }

        // 🏷️ 生成备份文件名(带时间戳)
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let backup_path = if let Some(tag_str) = tag {
            self.config_path
                .with_extension(format!("toml.{}_{}.bak", tag_str, timestamp))
        } else {
            self.config_path
                .with_extension(format!("toml.{}.bak", timestamp))
        };

        // 📋 复制文件
        fs::copy(&self.config_path, &backup_path)
            .map_err(|e| CcrError::ConfigError(format!("备份配置文件失败: {}", e)))?;

        log::info!("💾 配置文件已备份: {:?}", backup_path);

        // 🧹 自动清理旧备份(只保留最近10个)
        const MAX_BACKUPS: usize = 10;
        if let Ok(backups) = self.list_backups()
            && backups.len() > MAX_BACKUPS
        {
            let to_delete = &backups[MAX_BACKUPS..];
            for old_backup in to_delete {
                if let Err(e) = fs::remove_file(old_backup) {
                    log::warn!("清理旧备份失败 {:?}: {}", old_backup, e);
                } else {
                    log::debug!("🗑️ 已删除旧备份: {:?}", old_backup);
                }
            }
            log::info!(
                "🧹 已自动清理 {} 个旧配置备份,保留最近 {} 个",
                to_delete.len(),
                MAX_BACKUPS
            );
        }

        Ok(backup_path)
    }

    /// 📋 列出所有配置备份文件
    ///
    /// 返回所有配置文件的 .bak 备份,按修改时间倒序排列(最新的在前)
    pub fn list_backups(&self) -> Result<Vec<PathBuf>> {
        let config_dir = self
            .config_path
            .parent()
            .ok_or_else(|| CcrError::ConfigError("无法获取配置目录".into()))?;

        if !config_dir.exists() {
            return Ok(vec![]);
        }

        let config_filename = self
            .config_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| CcrError::ConfigError("无效的配置文件名".into()))?;

        let mut backups = Vec::new();

        // 📂 遍历配置目录
        for entry in fs::read_dir(config_dir)
            .map_err(|e| CcrError::ConfigError(format!("读取配置目录失败: {}", e)))?
        {
            let entry =
                entry.map_err(|e| CcrError::ConfigError(format!("读取目录项失败: {}", e)))?;

            let path = entry.path();
            let filename = path.file_name().and_then(|n| n.to_str());

            // 🔍 只收集配置文件的 .bak 文件
            // 例如: .ccs_config.toml.20240101_120000.bak
            if let Some(name) = filename
                && path.is_file() && name.starts_with(config_filename) && name.ends_with(".bak")
            {
                backups.push(path);
            }
        }

        // 📅 按修改时间排序(最新的在前)
        backups.sort_by(|a, b| {
            let a_time = fs::metadata(a).and_then(|m| m.modified()).ok();
            let b_time = fs::metadata(b).and_then(|m| m.modified()).ok();
            b_time.cmp(&a_time)
        });

        Ok(backups)
    }

    // === 🆕 多平台支持和迁移检测方法 ===

    /// 🔍 检测是否启用了统一模式
    ///
    /// 统一模式特征:
    /// 1. 环境变量 CCR_ROOT 已设置
    /// 2. ~/.ccr/ 目录存在
    /// 3. ~/.ccr/config.toml 文件存在
    ///
    /// 返回 (is_unified_mode, unified_config_path)
    pub fn detect_unified_mode() -> (bool, Option<PathBuf>) {
        // 1. 检查环境变量
        if let Ok(ccr_root) = std::env::var("CCR_ROOT") {
            let root_path = PathBuf::from(ccr_root);
            let config_path = root_path.join("config.toml");
            return (true, Some(config_path));
        }

        // 2. 检查默认统一配置路径
        if let Some(home) = dirs::home_dir() {
            let unified_root = home.join(".ccr");
            let unified_config = unified_root.join("config.toml");

            if unified_root.exists() && unified_config.exists() {
                return (true, Some(unified_config));
            }
        }

        (false, None)
    }

    /// 🔄 检测是否应该迁移到统一模式
    ///
    /// 迁移条件:
    /// 1. Legacy 配置文件存在 (~/.ccs_config.toml)
    /// 2. 统一模式配置不存在
    /// 3. 配置中有多个配置节（值得迁移）
    pub fn should_migrate(&self) -> Result<bool> {
        // ✅ Legacy 配置必须存在
        if !self.config_path.exists() {
            return Ok(false);
        }

        // ✅ 如果统一模式已启用，不需要迁移
        let (is_unified, _) = Self::detect_unified_mode();
        if is_unified {
            return Ok(false);
        }

        // ✅ 加载配置检查配置节数量
        let config = self.load()?;

        // 如果有 2 个或更多配置节，建议迁移
        // (单个配置节迁移意义不大)
        Ok(config.sections.len() >= 2)
    }

    /// 📊 获取迁移状态信息
    ///
    /// 返回迁移相关的详细信息
    pub fn get_migration_status(&self) -> MigrationStatus {
        let (is_unified, unified_path) = Self::detect_unified_mode();
        let legacy_exists = self.config_path.exists();

        let legacy_section_count = if legacy_exists {
            self.load().ok().map(|c| c.sections.len()).unwrap_or(0)
        } else {
            0
        };

        MigrationStatus {
            is_unified_mode: is_unified,
            legacy_config_exists: legacy_exists,
            legacy_config_path: self.config_path.clone(),
            unified_config_path: unified_path,
            legacy_section_count,
            should_migrate: self.should_migrate().unwrap_or(false),
        }
    }

    /// 🎯 获取当前配置模式
    ///
    /// 返回 "Legacy" 或 "Unified"
    #[allow(dead_code)]
    pub fn get_current_mode() -> &'static str {
        let (is_unified, _) = Self::detect_unified_mode();
        if is_unified { "Unified" } else { "Legacy" }
    }
}

/// 📊 迁移状态信息
#[derive(Debug, Clone)]
pub struct MigrationStatus {
    /// 是否已启用统一模式
    pub is_unified_mode: bool,

    /// Legacy 配置是否存在
    pub legacy_config_exists: bool,

    /// Legacy 配置路径
    pub legacy_config_path: PathBuf,

    /// 统一配置路径(如果存在)
    pub unified_config_path: Option<PathBuf>,

    /// Legacy 配置节数量
    pub legacy_section_count: usize,

    /// 是否应该迁移
    pub should_migrate: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_section() -> ConfigSection {
        ConfigSection {
            description: Some("Test config".into()),
            base_url: Some("https://api.test.com".into()),
            auth_token: Some("sk-test-token".into()),
            model: Some("test-model".into()),
            small_fast_model: Some("test-small-model".into()),
            provider: None,
            provider_type: None,
            account: None,
            tags: None,
        }
    }

    #[test]
    fn test_config_section_validate() {
        let section = create_test_section();
        assert!(section.validate().is_ok());

        // 测试空 base_url
        let mut invalid = section.clone();
        invalid.base_url = Some("".into());
        assert!(invalid.validate().is_err());

        // 测试无效的 URL
        let mut invalid = section.clone();
        invalid.base_url = Some("not-a-url".into());
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_ccs_config() {
        let mut config = CcsConfig {
            default_config: "default".into(),
            current_config: "default".into(),
            settings: GlobalSettings::default(),
            sections: IndexMap::new(),
        };
        assert_eq!(config.default_config, "default");
        assert_eq!(config.current_config, "default");

        // 添加配置节
        config.set_section("test".into(), create_test_section());
        assert!(config.get_section("test").is_ok());
        assert!(config.get_section("nonexistent").is_err());

        // 设置当前配置
        assert!(config.set_current("test").is_ok());
        assert_eq!(config.current_config, "test");
        assert!(config.set_current("nonexistent").is_err());
    }

    #[test]
    fn test_config_manager_load_save() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");

        // 创建测试配置
        let mut config = CcsConfig {
            default_config: "test".into(),
            current_config: "test".into(),
            settings: GlobalSettings::default(),
            sections: IndexMap::new(),
        };
        config.set_section("test".into(), create_test_section());

        // 保存
        let manager = ConfigManager::new(&config_path);
        manager.save(&config).unwrap();
        assert!(config_path.exists());

        // 加载
        let loaded = manager.load().unwrap();
        assert_eq!(loaded.default_config, "test");
        assert!(loaded.get_section("test").is_ok());
    }

    #[test]
    fn test_config_manager_backup() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join(".ccs_config.toml");

        // 创建测试配置
        let mut config = CcsConfig {
            default_config: "test".into(),
            current_config: "test".into(),
            settings: GlobalSettings::default(),
            sections: IndexMap::new(),
        };
        config.set_section("test".into(), create_test_section());

        let manager = ConfigManager::new(&config_path);
        manager.save(&config).unwrap();

        // 测试备份
        let backup_path = manager.backup(Some("test")).unwrap();
        assert!(backup_path.exists());
        assert!(backup_path.to_string_lossy().contains("test_"));

        // 测试列出备份
        let backups = manager.list_backups().unwrap();
        assert_eq!(backups.len(), 1);
    }

    #[test]
    fn test_config_backup_auto_cleanup() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join(".ccs_config.toml");

        // 创建测试配置
        let mut config = CcsConfig {
            default_config: "test".into(),
            current_config: "test".into(),
            settings: GlobalSettings::default(),
            sections: IndexMap::new(),
        };
        config.set_section("test".into(), create_test_section());

        let manager = ConfigManager::new(&config_path);
        manager.save(&config).unwrap();

        // 创建15个备份
        for i in 0..15 {
            manager.backup(Some(&format!("tag{}", i))).unwrap();
            // 短暂延迟确保时间戳不同
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        // 验证只保留了最近10个备份
        let backups = manager.list_backups().unwrap();
        assert_eq!(
            backups.len(),
            10,
            "应该只保留10个配置备份,但实际有 {} 个",
            backups.len()
        );
    }

    #[test]
    fn test_global_settings() {
        // 测试默认设置
        let settings = GlobalSettings::default();
        assert!(!settings.skip_confirmation);
        assert_eq!(settings.tui_theme, None);

        // 测试序列化
        let toml_str = toml::to_string(&settings).unwrap();
        assert!(toml_str.contains("skip_confirmation = false"));

        // 测试反序列化
        let loaded: GlobalSettings = toml::from_str(&toml_str).unwrap();
        assert_eq!(loaded.skip_confirmation, settings.skip_confirmation);

        // 测试向后兼容（yolo_mode别名）
        let old_format = "yolo_mode = true";
        let loaded_old: GlobalSettings = toml::from_str(old_format).unwrap();
        assert!(loaded_old.skip_confirmation);
    }
}
