// Droids (Subagents) 文件管理器
// 管理 .factory/droids/ 目录下的 Markdown 文件

use anyhow::{Context, Result};
use serde_json;
use serde_yaml;
use std::fs;
use std::path::PathBuf;

use crate::models::platforms::droid::{Droid, DroidRequest};

/// Droid frontmatter (不含 system_prompt，因为它在 markdown body 中)
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct DroidFrontmatter {
    pub name: String,
    pub description: Option<String>,
    pub model: String,
    pub reasoning_effort: Option<String>,
    pub tools: Option<serde_json::Value>,
}

/// Droids 文件管理器
pub struct DroidsManager {
    droids_dir: PathBuf,
}

impl DroidsManager {
    /// 创建新的 DroidsManager
    pub fn new(base_dir: PathBuf) -> Result<Self> {
        let droids_dir = base_dir.join("droids");

        // 确保目录存在
        if !droids_dir.exists() {
            fs::create_dir_all(&droids_dir).context("创建 droids 目录失败")?;
        }

        Ok(Self { droids_dir })
    }

    /// 默认构造函数 (使用 .factory/droids/)
    pub fn default() -> Result<Self> {
        let base_dir = std::env::current_dir()
            .context("获取当前目录失败")?
            .join(".factory");
        Self::new(base_dir)
    }

    /// 列出所有 Droids
    pub fn list_droids(&self) -> Result<Vec<Droid>> {
        let mut droids = Vec::new();

        if !self.droids_dir.exists() {
            return Ok(droids);
        }

        for entry in fs::read_dir(&self.droids_dir).context("读取 droids 目录失败")? {
            let entry = entry.context("读取目录项失败")?;
            let path = entry.path();

            // 只处理 .md 文件
            if path.extension().and_then(|s| s.to_str()) == Some("md")
                && let Ok(droid) = self.read_droid_file(&path)
            {
                droids.push(droid);
            }
        }

        // 按名称排序
        droids.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(droids)
    }

    /// 获取单个 Droid
    pub fn get_droid(&self, name: &str) -> Result<Droid> {
        let file_path = self.droids_dir.join(format!("{}.md", name));

        if !file_path.exists() {
            anyhow::bail!("Droid '{}' 不存在", name);
        }

        self.read_droid_file(&file_path)
    }

    /// 创建新 Droid
    pub fn create_droid(&self, request: DroidRequest) -> Result<()> {
        // 验证名称格式
        self.validate_droid_name(&request.name)?;

        let file_path = self.droids_dir.join(format!("{}.md", request.name));

        // 检查是否已存在
        if file_path.exists() {
            anyhow::bail!("Droid '{}' 已存在", request.name);
        }

        // 构建 Droid
        let droid = Droid {
            name: request.name.clone(),
            description: request.description,
            model: request.model,
            reasoning_effort: request.reasoning_effort,
            tools: request.tools,
            system_prompt: request.system_prompt,
        };

        // 写入文件
        self.write_droid_file(&file_path, &droid)?;

        Ok(())
    }

    /// 更新 Droid
    pub fn update_droid(&self, name: &str, request: DroidRequest) -> Result<()> {
        let file_path = self.droids_dir.join(format!("{}.md", name));

        // 检查是否存在
        if !file_path.exists() {
            anyhow::bail!("Droid '{}' 不存在", name);
        }

        // 如果名称改变，需要重命名文件
        if name != request.name {
            self.validate_droid_name(&request.name)?;
            let new_file_path = self.droids_dir.join(format!("{}.md", request.name));

            if new_file_path.exists() {
                anyhow::bail!("Droid '{}' 已存在", request.name);
            }

            // 删除旧文件
            fs::remove_file(&file_path).context("删除旧 Droid 文件失败")?;
        }

        // 构建 Droid
        let droid = Droid {
            name: request.name.clone(),
            description: request.description,
            model: request.model,
            reasoning_effort: request.reasoning_effort,
            tools: request.tools,
            system_prompt: request.system_prompt,
        };

        // 写入文件
        let target_path = self.droids_dir.join(format!("{}.md", request.name));
        self.write_droid_file(&target_path, &droid)?;

        Ok(())
    }

    /// 删除 Droid
    pub fn delete_droid(&self, name: &str) -> Result<()> {
        let file_path = self.droids_dir.join(format!("{}.md", name));

        if !file_path.exists() {
            anyhow::bail!("Droid '{}' 不存在", name);
        }

        fs::remove_file(&file_path).context("删除 Droid 文件失败")?;

        Ok(())
    }

    /// 读取 Droid 文件
    fn read_droid_file(&self, path: &PathBuf) -> Result<Droid> {
        let content = fs::read_to_string(path).context("读取 Droid 文件失败")?;

        // 解析 Markdown frontmatter
        self.parse_droid_markdown(&content)
    }

    /// 写入 Droid 文件
    fn write_droid_file(&self, path: &PathBuf, droid: &Droid) -> Result<()> {
        let content = self.format_droid_markdown(droid)?;

        fs::write(path, content).context("写入 Droid 文件失败")?;

        Ok(())
    }

    /// 解析 Markdown frontmatter
    fn parse_droid_markdown(&self, content: &str) -> Result<Droid> {
        // 查找 frontmatter 边界
        let lines: Vec<&str> = content.lines().collect();

        if lines.is_empty() || lines[0] != "---" {
            anyhow::bail!("无效的 Droid 文件格式：缺少 frontmatter");
        }

        // 找到第二个 ---
        let end_index = lines
            .iter()
            .skip(1)
            .position(|&line| line == "---")
            .context("无效的 Droid 文件格式：frontmatter 未闭合")?;

        // 提取 frontmatter YAML
        let frontmatter_lines = &lines[1..=end_index];
        let frontmatter_yaml = frontmatter_lines.join("\n");

        // 提取系统提示 (frontmatter 之后的所有内容)
        let system_prompt_lines = &lines[end_index + 2..];
        let system_prompt = system_prompt_lines.join("\n").trim().to_string();

        // 解析 YAML 为 DroidFrontmatter（不包含 system_prompt）
        let frontmatter: DroidFrontmatter =
            serde_yaml::from_str(&frontmatter_yaml).context("解析 Droid frontmatter 失败")?;

        // 构建完整的 Droid 对象
        let droid = Droid {
            name: frontmatter.name,
            description: frontmatter.description,
            model: frontmatter.model,
            reasoning_effort: frontmatter.reasoning_effort,
            tools: frontmatter.tools,
            system_prompt,
        };

        Ok(droid)
    }

    /// 格式化为 Markdown
    fn format_droid_markdown(&self, droid: &Droid) -> Result<String> {
        // 构建 frontmatter 对象
        let frontmatter_obj = serde_json::json!({
            "name": droid.name,
            "description": droid.description,
            "model": droid.model,
            "reasoningEffort": droid.reasoning_effort,
            "tools": droid.tools,
        });

        // 序列化为 YAML
        let mut frontmatter =
            serde_yaml::to_string(&frontmatter_obj).context("序列化 Droid frontmatter 失败")?;

        // 移除 null 值
        frontmatter = frontmatter
            .lines()
            .filter(|line| !line.contains(": null") && !line.contains(": ~"))
            .collect::<Vec<_>>()
            .join("\n");

        // 构建完整内容
        let content = format!(
            "---\n{}\n---\n\n{}",
            frontmatter.trim(),
            droid.system_prompt.trim()
        );

        Ok(content)
    }

    /// 验证 Droid 名称格式
    fn validate_droid_name(&self, name: &str) -> Result<()> {
        if name.is_empty() {
            anyhow::bail!("Droid 名称不能为空");
        }

        // 只允许小写字母、数字、- 和 _
        if !name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
        {
            anyhow::bail!("Droid 名称只能包含小写字母、数字、- 和 _");
        }

        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_and_read_droid() {
        let temp_dir = TempDir::new().unwrap();
        let manager = DroidsManager::new(temp_dir.path().to_path_buf()).unwrap();

        let request = DroidRequest {
            name: "test-droid".to_string(),
            description: Some("Test droid".to_string()),
            model: "inherit".to_string(),
            reasoning_effort: Some("medium".to_string()),
            tools: Some(serde_json::json!(["Read", "Write"])),
            system_prompt: "You are a test droid.".to_string(),
        };

        manager.create_droid(request).unwrap();

        let droid = manager.get_droid("test-droid").unwrap();
        assert_eq!(droid.name, "test-droid");
        assert_eq!(droid.description, Some("Test droid".to_string()));
        assert_eq!(droid.system_prompt, "You are a test droid.");
    }

    #[test]
    fn test_list_droids() {
        let temp_dir = TempDir::new().unwrap();
        let manager = DroidsManager::new(temp_dir.path().to_path_buf()).unwrap();

        // 创建多个 droids
        for i in 1..=3 {
            let request = DroidRequest {
                name: format!("droid-{}", i),
                description: None,
                model: "inherit".to_string(),
                reasoning_effort: None,
                tools: None,
                system_prompt: format!("Droid {}", i),
            };
            manager.create_droid(request).unwrap();
        }

        let droids = manager.list_droids().unwrap();
        assert_eq!(droids.len(), 3);
    }

    #[test]
    fn test_validate_droid_name() {
        let temp_dir = TempDir::new().unwrap();
        let manager = DroidsManager::new(temp_dir.path().to_path_buf()).unwrap();

        assert!(manager.validate_droid_name("valid-name").is_ok());
        assert!(manager.validate_droid_name("valid_name").is_ok());
        assert!(manager.validate_droid_name("valid123").is_ok());

        assert!(manager.validate_droid_name("Invalid-Name").is_err());
        assert!(manager.validate_droid_name("invalid name").is_err());
        assert!(manager.validate_droid_name("").is_err());
    }
}
