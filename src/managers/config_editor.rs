//! 📝 配置编辑器模块
//!
//! 使用 toml_edit 进行配置文件编辑，保留格式和注释

use crate::core::error::{CcrError, Result};
use std::path::Path;
use toml_edit::{DocumentMut, Item, Table};

/// 配置编辑器
///
/// 使用 toml_edit 进行配置文件编辑，保留原有格式
#[allow(dead_code)]
pub struct ConfigEditor {
    /// 文档内容
    doc: DocumentMut,
    /// 文件路径
    path: std::path::PathBuf,
}

#[allow(dead_code)]
impl ConfigEditor {
    /// 从文件加载配置
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| CcrError::ConfigError(format!("无法读取配置文件: {}", e)))?;

        let doc = content
            .parse::<DocumentMut>()
            .map_err(|e| CcrError::ConfigError(format!("无法解析配置文件: {}", e)))?;

        Ok(Self {
            doc,
            path: path.to_path_buf(),
        })
    }

    /// 从字符串创建
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(content: &str) -> Result<Self> {
        let doc = content
            .parse::<DocumentMut>()
            .map_err(|e| CcrError::ConfigError(format!("无法解析配置: {}", e)))?;

        Ok(Self {
            doc,
            path: std::path::PathBuf::new(),
        })
    }

    /// 保存到文件
    pub fn save(&self) -> Result<()> {
        if self.path.as_os_str().is_empty() {
            return Err(CcrError::ConfigError("未指定保存路径".to_string()));
        }
        self.save_to(&self.path)
    }

    /// 保存到指定路径
    pub fn save_to(&self, path: &Path) -> Result<()> {
        std::fs::write(path, self.doc.to_string())
            .map_err(|e| CcrError::IoError(std::io::Error::other(e)))
    }

    /// 获取文档字符串
    pub fn as_str(&self) -> String {
        self.doc.to_string()
    }

    /// 获取配置节名称列表
    pub fn list_sections(&self) -> Vec<String> {
        self.doc
            .iter()
            .filter(|(_, v)| v.is_table())
            .map(|(k, _)| k.to_string())
            .collect()
    }

    /// 检查配置节是否存在
    pub fn has_section(&self, name: &str) -> bool {
        self.doc.get(name).map(|v| v.is_table()).unwrap_or(false)
    }

    /// 获取配置节值
    pub fn get_value(&self, section: &str, key: &str) -> Option<String> {
        self.doc
            .get(section)?
            .as_table()?
            .get(key)?
            .as_str()
            .map(|s| s.to_string())
    }

    /// 设置配置节值
    pub fn set_value(&mut self, section: &str, key: &str, value: &str) -> Result<()> {
        // 确保配置节存在
        if !self.has_section(section) {
            self.doc[section] = Item::Table(Table::new());
        }

        // 设置值
        if let Some(table) = self.doc[section].as_table_mut() {
            table[key] = toml_edit::value(value);
            Ok(())
        } else {
            Err(CcrError::ConfigError(format!(
                "配置节 {} 不是有效的表",
                section
            )))
        }
    }

    /// 设置布尔值
    pub fn set_bool(&mut self, section: &str, key: &str, value: bool) -> Result<()> {
        if !self.has_section(section) {
            self.doc[section] = Item::Table(Table::new());
        }

        if let Some(table) = self.doc[section].as_table_mut() {
            table[key] = toml_edit::value(value);
            Ok(())
        } else {
            Err(CcrError::ConfigError(format!(
                "配置节 {} 不是有效的表",
                section
            )))
        }
    }

    /// 设置整数值
    pub fn set_int(&mut self, section: &str, key: &str, value: i64) -> Result<()> {
        if !self.has_section(section) {
            self.doc[section] = Item::Table(Table::new());
        }

        if let Some(table) = self.doc[section].as_table_mut() {
            table[key] = toml_edit::value(value);
            Ok(())
        } else {
            Err(CcrError::ConfigError(format!(
                "配置节 {} 不是有效的表",
                section
            )))
        }
    }

    /// 删除配置节
    pub fn remove_section(&mut self, name: &str) -> bool {
        self.doc.remove(name).is_some()
    }

    /// 删除配置节中的键
    pub fn remove_key(&mut self, section: &str, key: &str) -> bool {
        self.doc
            .get_mut(section)
            .and_then(|s| s.as_table_mut())
            .map(|t| t.remove(key).is_some())
            .unwrap_or(false)
    }

    /// 重命名配置节
    pub fn rename_section(&mut self, old_name: &str, new_name: &str) -> Result<()> {
        if !self.has_section(old_name) {
            return Err(CcrError::ConfigError(format!("配置节 {} 不存在", old_name)));
        }
        if self.has_section(new_name) {
            return Err(CcrError::ConfigError(format!("配置节 {} 已存在", new_name)));
        }

        // 获取旧配置节的值
        if let Some(value) = self.doc.remove(old_name) {
            self.doc[new_name] = value;
            Ok(())
        } else {
            Err(CcrError::ConfigError(format!("配置节 {} 不存在", old_name)))
        }
    }

    /// 复制配置节
    pub fn copy_section(&mut self, source: &str, dest: &str) -> Result<()> {
        if !self.has_section(source) {
            return Err(CcrError::ConfigError(format!("配置节 {} 不存在", source)));
        }
        if self.has_section(dest) {
            return Err(CcrError::ConfigError(format!("配置节 {} 已存在", dest)));
        }

        // 获取源配置节的字符串表示并解析
        if let Some(table) = self.doc.get(source).and_then(|v| v.as_table()) {
            let mut new_table = Table::new();
            for (key, value) in table.iter() {
                new_table[key] = value.clone();
            }
            self.doc[dest] = Item::Table(new_table);
            Ok(())
        } else {
            Err(CcrError::ConfigError(format!("配置节 {} 不存在", source)))
        }
    }

    /// 获取配置节的所有键值对
    pub fn get_section_entries(&self, section: &str) -> Option<Vec<(String, String)>> {
        let table = self.doc.get(section)?.as_table()?;
        Some(
            table
                .iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.to_string(), s.to_string())))
                .collect(),
        )
    }

    /// 更新 settings 部分的 current_config
    pub fn set_current_config(&mut self, name: &str) -> Result<()> {
        self.set_value("settings", "current_config", name)
    }

    /// 获取当前配置名称
    pub fn get_current_config(&self) -> Option<String> {
        self.get_value("settings", "current_config")
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let content = r#"
[settings]
current_config = "anthropic"
skip_confirmation = false

[anthropic]
description = "Anthropic Official API"
base_url = "https://api.anthropic.com"
auth_token = "sk-ant-xxx"
model = "claude-sonnet-4-20250514"
"#;

        let mut editor = ConfigEditor::from_str(content).unwrap();

        // 测试获取值
        assert_eq!(
            editor.get_value("settings", "current_config"),
            Some("anthropic".to_string())
        );
        assert_eq!(
            editor.get_value("anthropic", "base_url"),
            Some("https://api.anthropic.com".to_string())
        );

        // 测试设置值
        editor
            .set_value("anthropic", "model", "claude-opus-4-0")
            .unwrap();
        assert_eq!(
            editor.get_value("anthropic", "model"),
            Some("claude-opus-4-0".to_string())
        );

        // 测试列出配置节
        let sections = editor.list_sections();
        assert!(sections.contains(&"settings".to_string()));
        assert!(sections.contains(&"anthropic".to_string()));

        // 测试删除配置节
        assert!(editor.remove_section("anthropic"));
        assert!(!editor.has_section("anthropic"));
    }

    #[test]
    fn test_format_preservation() {
        let content = r#"# 配置文件头部注释
[settings]
current_config = "test"

# API 配置
[test]
description = "Test Config"
base_url = "https://example.com"
"#;

        let mut editor = ConfigEditor::from_str(content)
            .expect("Failed to parse test TOML content");
        editor
            .set_value("test", "model", "gpt-4")
            .expect("Failed to set test value");

        let output = editor.as_str();
        // 验证注释被保留
        assert!(output.contains("# 配置文件头部注释"));
        assert!(output.contains("# API 配置"));
    }
}
