// 🎯 CCR 同步内容选择器
// 提供交互式界面让用户选择要同步的内容类型

use crate::core::error::{CcrError, Result};
use crate::core::logging::ColorOutput;
use colored::*;
use std::collections::HashMap;
use std::io::{self, Write};

/// 同步内容类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SyncContentType {
    Config,
    Claude,
    Gemini,
    Qwen,
    IFlow,
}

impl SyncContentType {
    /// 获取显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            SyncContentType::Config => "CCR 配置 (config.toml)",
            SyncContentType::Claude => "Claude 配置 (.claude/)",
            SyncContentType::Gemini => "Gemini 配置 (.gemini/)",
            SyncContentType::Qwen => "Qwen 配置 (.qwen/)",
            SyncContentType::IFlow => "iFlow 配置 (.iflow/)",
        }
    }

    /// 获取简短名称
    #[allow(dead_code)]
    pub fn short_name(&self) -> &'static str {
        match self {
            SyncContentType::Config => "config",
            SyncContentType::Claude => "claude",
            SyncContentType::Gemini => "gemini",
            SyncContentType::Qwen => "qwen",
            SyncContentType::IFlow => "iflow",
        }
    }

    /// 获取所有可用类型
    pub fn all_types() -> Vec<SyncContentType> {
        vec![
            SyncContentType::Config,
            SyncContentType::Claude,
            SyncContentType::Gemini,
            SyncContentType::Qwen,
            SyncContentType::IFlow,
        ]
    }

    /// 检查内容是否存在
    pub fn exists(&self) -> bool {
        use std::path::PathBuf;
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let ccr_root = if let Ok(root) = std::env::var("CCR_ROOT") {
            PathBuf::from(root)
        } else {
            home.join(".ccr")
        };

        match self {
            SyncContentType::Config => ccr_root.join("config.toml").exists(),
            SyncContentType::Claude => {
                home.join(".claude").exists() || ccr_root.join("platforms").join("claude").exists()
            }
            SyncContentType::Gemini => {
                home.join(".gemini").exists() || ccr_root.join("platforms").join("gemini").exists()
            }
            SyncContentType::Qwen => {
                home.join(".qwen").exists() || ccr_root.join("platforms").join("qwen").exists()
            }
            SyncContentType::IFlow => {
                home.join(".iflow").exists() || ccr_root.join("platforms").join("iflow").exists()
            }
        }
    }
}

/// 同步内容选择结果
#[derive(Debug, Clone)]
pub struct SyncContentSelection {
    pub selected_types: Vec<SyncContentType>,
    #[allow(dead_code)]
    pub use_default: bool,
}

impl Default for SyncContentSelection {
    fn default() -> Self {
        Self {
            selected_types: vec![SyncContentType::Config],
            use_default: true,
        }
    }
}

impl SyncContentSelection {
    /// 创建自定义选择
    pub fn custom(selected_types: Vec<SyncContentType>) -> Self {
        Self {
            selected_types,
            use_default: false,
        }
    }

    /// 检查是否选择了指定类型
    #[allow(dead_code)]
    pub fn contains(&self, content_type: &SyncContentType) -> bool {
        self.selected_types.contains(content_type)
    }

    /// 获取选择的内容数量
    pub fn count(&self) -> usize {
        self.selected_types.len()
    }

    /// 转换为路径列表（用于同步过滤）
    pub fn to_paths(&self) -> Vec<String> {
        let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        let ccr_root = if let Ok(root) = std::env::var("CCR_ROOT") {
            std::path::PathBuf::from(root)
        } else {
            home.join(".ccr")
        };

        let mut paths = Vec::new();

        for content_type in &self.selected_types {
            match content_type {
                SyncContentType::Config => {
                    paths.push("config.toml".to_string());
                }
                SyncContentType::Claude => {
                    if home.join(".claude").exists() {
                        paths.push(".claude".to_string());
                    } else if ccr_root.join("platforms").join("claude").exists() {
                        paths.push("platforms/claude".to_string());
                    }
                }
                SyncContentType::Gemini => {
                    if home.join(".gemini").exists() {
                        paths.push(".gemini".to_string());
                    } else if ccr_root.join("platforms").join("gemini").exists() {
                        paths.push("platforms/gemini".to_string());
                    }
                }
                SyncContentType::Qwen => {
                    if home.join(".qwen").exists() {
                        paths.push(".qwen".to_string());
                    } else if ccr_root.join("platforms").join("qwen").exists() {
                        paths.push("platforms/qwen".to_string());
                    }
                }
                SyncContentType::IFlow => {
                    if home.join(".iflow").exists() {
                        paths.push(".iflow".to_string());
                    } else if ccr_root.join("platforms").join("iflow").exists() {
                        paths.push("platforms/iflow".to_string());
                    }
                }
            }
        }

        paths
    }
}

/// 交互式内容选择面板
pub struct SyncContentSelector {
    available_types: Vec<SyncContentType>,
    selected: HashMap<SyncContentType, bool>,
}

impl Default for SyncContentSelector {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncContentSelector {
    /// 创建新的选择器
    pub fn new() -> Self {
        let available_types = SyncContentType::all_types()
            .into_iter()
            .filter(|t| t.exists())
            .collect();

        let mut selected = HashMap::new();
        // 默认选中config
        selected.insert(SyncContentType::Config, true);

        Self {
            available_types,
            selected,
        }
    }

    /// 获取可用类型列表（用于测试）
    #[allow(dead_code)]
    pub fn get_available_types(&self) -> &[SyncContentType] {
        &self.available_types
    }

    /// 显示选择面板并获取用户选择
    pub fn select_content(&mut self) -> Result<SyncContentSelection> {
        ColorOutput::title("选择同步内容");
        println!();

        if self.available_types.is_empty() {
            ColorOutput::warning("未找到可同步的内容");
            return Ok(SyncContentSelection::default());
        }

        loop {
            self.display_options();
            println!();
            ColorOutput::info("操作选项:");
            println!("  1-{}: 切换对应内容的选择状态", self.available_types.len());
            println!("  a: 全选");
            println!("  n: 取消全选");
            println!("  c: 确认选择");
            println!("  q: 取消操作");
            println!();

            print!("请选择操作: ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();

            match input {
                "a" => self.select_all(),
                "n" => self.deselect_all(),
                "c" => {
                    let selected_types = self.get_selected_types();
                    if selected_types.is_empty() {
                        ColorOutput::warning("请至少选择一项内容");
                        continue;
                    }
                    return Ok(SyncContentSelection::custom(selected_types));
                }
                "q" => {
                    return Err(CcrError::ConfigError("用户取消操作".into()));
                }
                num if num.chars().all(|c| c.is_ascii_digit()) => {
                    if let Ok(idx) = num.parse::<usize>() {
                        if idx >= 1 && idx <= self.available_types.len() {
                            self.toggle_selection(idx - 1);
                        } else {
                            ColorOutput::error("无效的选择编号");
                        }
                    }
                }
                _ => {
                    ColorOutput::error("无效的输入");
                }
            }
        }
    }

    /// 显示当前选项
    fn display_options(&self) {
        ColorOutput::info("可选内容:");
        for (i, content_type) in self.available_types.iter().enumerate() {
            let selected = self.selected.get(content_type).unwrap_or(&false);
            let checkbox = if *selected { "[✓]" } else { "[ ]" };
            let name = content_type.display_name();

            println!(
                "  {} {} {}",
                (i + 1).to_string().cyan(),
                checkbox.green(),
                name
            );
        }
    }

    /// 切换选择状态
    fn toggle_selection(&mut self, index: usize) {
        if let Some(content_type) = self.available_types.get(index) {
            let current = self.selected.get(content_type).unwrap_or(&false);
            self.selected.insert(content_type.clone(), !*current);
        }
    }

    /// 全选
    fn select_all(&mut self) {
        for content_type in &self.available_types {
            self.selected.insert(content_type.clone(), true);
        }
    }

    /// 取消全选
    fn deselect_all(&mut self) {
        for content_type in &self.available_types {
            self.selected.insert(content_type.clone(), false);
        }
    }

    /// 获取选中的类型
    fn get_selected_types(&self) -> Vec<SyncContentType> {
        self.available_types
            .iter()
            .filter(|t| *self.selected.get(*t).unwrap_or(&false))
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_sync_content_type_display() {
        assert_eq!(
            SyncContentType::Config.display_name(),
            "CCR 配置 (config.toml)"
        );
        assert_eq!(
            SyncContentType::Claude.display_name(),
            "Claude 配置 (.claude/)"
        );
        assert_eq!(SyncContentType::Config.short_name(), "config");
        assert_eq!(SyncContentType::Claude.short_name(), "claude");
    }

    #[test]
    fn test_sync_content_selection() {
        let selection = SyncContentSelection::default();
        assert!(selection.contains(&SyncContentType::Config));
        assert_eq!(selection.count(), 1);
        assert!(selection.use_default);

        let custom =
            SyncContentSelection::custom(vec![SyncContentType::Config, SyncContentType::Claude]);
        assert!(custom.contains(&SyncContentType::Config));
        assert!(custom.contains(&SyncContentType::Claude));
        assert_eq!(custom.count(), 2);
        assert!(!custom.use_default);
    }

    #[test]
    fn test_sync_content_selection_to_paths() {
        let temp_dir = tempdir().unwrap();
        let ccr_root = temp_dir.path().join(".ccr");
        unsafe {
            std::env::set_var("CCR_ROOT", ccr_root.to_str().unwrap());
        }

        // 创建测试文件
        fs::create_dir_all(&ccr_root).unwrap();
        fs::write(ccr_root.join("config.toml"), "test").unwrap();

        let selection =
            SyncContentSelection::custom(vec![SyncContentType::Config, SyncContentType::Claude]);

        let paths = selection.to_paths();
        assert!(paths.contains(&"config.toml".to_string()));

        // 清理环境变量
        unsafe {
            std::env::remove_var("CCR_ROOT");
        }
    }

    #[test]
    fn test_sync_content_type_exists() {
        let temp_dir = tempdir().unwrap();
        let ccr_root = temp_dir.path().join(".ccr");
        unsafe {
            std::env::set_var("CCR_ROOT", ccr_root.to_str().unwrap());
        }

        fs::create_dir_all(&ccr_root).unwrap();
        fs::write(ccr_root.join("config.toml"), "test").unwrap();

        assert!(SyncContentType::Config.exists());
        // Claude 默认不存在，因为我们没有创建对应的目录或文件

        // 清理环境变量
        unsafe {
            std::env::remove_var("CCR_ROOT");
        }
    }

    #[test]
    fn test_sync_content_selector_new() {
        let temp_dir = tempdir().unwrap();
        let ccr_root = temp_dir.path().join(".ccr");
        unsafe {
            std::env::set_var("CCR_ROOT", ccr_root.to_str().unwrap());
        }

        fs::create_dir_all(&ccr_root).unwrap();
        fs::write(ccr_root.join("config.toml"), "test").unwrap();

        let selector = SyncContentSelector::new();
        assert!(!selector.available_types.is_empty());
        assert!(selector.selected.contains_key(&SyncContentType::Config));

        // 清理环境变量
        unsafe {
            std::env::remove_var("CCR_ROOT");
        }
    }
}
