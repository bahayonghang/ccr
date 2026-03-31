// 📦 CCS 配置文件结构
// 对应 ~/.ccs_config.toml 或 profiles.toml

use crate::managers::config::types::{ConfigSection, GlobalSettings, ProviderType};
use ccr_core::core::error::{CcrError, Result};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// 📦 CCS 配置文件总体结构
///
/// 对应 ~/.ccs_config.toml 的完整结构
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

#[allow(dead_code)]
impl CcsConfig {
    /// 🔍 获取指定配置节
    pub fn get_section(&self, name: &str) -> Result<&ConfigSection> {
        self.sections
            .get(name)
            .ok_or_else(|| CcrError::ConfigSectionNotFound(name.to_string()))
    }

    /// 🔧 获取指定配置节的可变引用
    pub fn get_section_mut(&mut self, name: &str) -> Result<&mut ConfigSection> {
        self.sections
            .get_mut(name)
            .ok_or_else(|| CcrError::ConfigSectionNotFound(name.to_string()))
    }

    /// ▶️ 获取当前配置节
    pub fn get_current_section(&self) -> Result<&ConfigSection> {
        self.get_section(&self.current_config)
    }

    /// 🔄 设置当前配置
    #[allow(dead_code)]
    pub fn set_current(&mut self, name: &str) -> Result<()> {
        if !self.sections.contains_key(name) {
            return Err(CcrError::ConfigSectionNotFound(name.to_string()));
        }
        self.current_config = name.to_string();
        Ok(())
    }

    /// ➕ 添加或更新配置节
    pub fn set_section(&mut self, name: String, section: ConfigSection) {
        self.sections.insert(name, section);
    }

    /// ➖ 删除配置节
    pub fn remove_section(&mut self, name: &str) -> Result<ConfigSection> {
        self.sections
            .shift_remove(name)
            .ok_or_else(|| CcrError::ConfigSectionNotFound(name.to_string()))
    }

    /// 📜 列出所有配置节名称(已排序)
    pub fn list_sections(&self) -> impl Iterator<Item = &String> {
        let mut names: Vec<&String> = self.sections.keys().collect();
        names.sort();
        names.into_iter()
    }

    /// 🔄 按配置节名称排序
    pub fn sort_sections(&mut self) {
        self.sections.sort_by(|k1, _, k2, _| k1.cmp(k2));
    }

    // === 分类和筛选方法 ===

    /// 🏢 按提供商分组获取配置
    pub fn group_by_provider(&self) -> IndexMap<String, Vec<String>> {
        let mut groups: IndexMap<String, Vec<String>> = IndexMap::new();

        for (name, section) in &self.sections {
            let provider = section.provider_display().to_string();
            groups.entry(provider).or_default().push(name.clone());
        }

        for configs in groups.values_mut() {
            configs.sort();
        }

        groups
    }

    /// 🏷️ 按提供商类型分组获取配置
    pub fn group_by_provider_type(&self) -> IndexMap<String, Vec<String>> {
        let mut groups: IndexMap<String, Vec<String>> = IndexMap::new();

        for (name, section) in &self.sections {
            let provider_type = section.provider_type_display().to_string();
            groups.entry(provider_type).or_default().push(name.clone());
        }

        for configs in groups.values_mut() {
            configs.sort();
        }

        groups
    }

    /// 🔍 按标签筛选配置
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
