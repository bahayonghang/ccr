use serde::{Deserialize, Serialize};

/// 技能元数据
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillMetadata {
    /// 作者
    pub author: Option<String>,
    /// 版本
    pub version: Option<String>,
    /// 许可证
    pub license: Option<String>,
    /// 分类
    pub category: Option<String>,
    /// 标签列表
    #[serde(default)]
    pub tags: Vec<String>,
    /// 最后更新时间
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub description: Option<String>,
    pub path: String,
    pub instruction: String,
    /// 元数据信息
    #[serde(default)]
    pub metadata: SkillMetadata,
    /// 是否为远程技能
    #[serde(default)]
    pub is_remote: bool,
    /// 远程仓库名称
    pub repository: Option<String>,
}

impl Skill {
    /// Parse frontmatter and inline metadata from SKILL.md content.
    /// Returns (metadata, description_from_frontmatter)
    #[allow(dead_code)] // Used by ccr-ui-backend crate
    pub fn parse_frontmatter(content: &str) -> (SkillMetadata, Option<String>) {
        let mut metadata = SkillMetadata::default();
        let mut description = None;

        // Try YAML frontmatter (--- delimited)
        let trimmed = content.trim();
        if let Some(after_prefix) = trimmed.strip_prefix("---")
            && let Some(end_idx) = after_prefix.find("---")
        {
            let frontmatter = &after_prefix[..end_idx];
            for line in frontmatter.lines() {
                let line = line.trim();
                if let Some((key, value)) = line.split_once(':') {
                    let key = key.trim().to_lowercase();
                    let value = value.trim().to_string();
                    match key.as_str() {
                        "name" => {} // name is set separately
                        "description" => description = Some(value),
                        "category" => metadata.category = Some(value),
                        "author" => metadata.author = Some(value),
                        "version" => metadata.version = Some(value),
                        "license" => metadata.license = Some(value),
                        "tags" => {
                            // Parse YAML array: [tag1, tag2] or comma separated
                            let stripped = value.trim_start_matches('[').trim_end_matches(']');
                            metadata.tags = stripped
                                .split(',')
                                .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
                                .filter(|s| !s.is_empty())
                                .collect();
                        }
                        _ => {}
                    }
                }
            }
            return (metadata, description);
        }

        // Fallback: parse inline key: value format
        metadata = Self::parse_metadata(content);
        (metadata, None)
    }

    /// 从 SKILL.md 内容解析元数据 (legacy inline format)
    pub fn parse_metadata(content: &str) -> SkillMetadata {
        let mut metadata = SkillMetadata::default();

        for line in content.lines() {
            let line = line.trim();
            // 解析常见的元数据格式：`key: value` 或 `**key**: value`
            if let Some(rest) = line
                .strip_prefix("Author:")
                .or_else(|| line.strip_prefix("**Author**:"))
            {
                metadata.author = Some(rest.trim().to_string());
            } else if let Some(rest) = line
                .strip_prefix("Version:")
                .or_else(|| line.strip_prefix("**Version**:"))
            {
                metadata.version = Some(rest.trim().to_string());
            } else if let Some(rest) = line
                .strip_prefix("License:")
                .or_else(|| line.strip_prefix("**License**:"))
            {
                metadata.license = Some(rest.trim().to_string());
            } else if let Some(rest) = line
                .strip_prefix("Category:")
                .or_else(|| line.strip_prefix("**Category**:"))
            {
                metadata.category = Some(rest.trim().to_string());
            } else if let Some(rest) = line
                .strip_prefix("Tags:")
                .or_else(|| line.strip_prefix("**Tags**:"))
            {
                metadata.tags = rest
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
        }

        metadata
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillRepository {
    pub name: String,
    pub url: String,
    pub branch: String,
    pub description: Option<String>,
    /// 技能数量
    #[serde(default)]
    pub skill_count: u32,
    /// 上次同步时间
    pub last_synced: Option<String>,
    /// 是否为官方仓库
    #[serde(default)]
    pub is_official: bool,
}

impl Default for SkillRepository {
    fn default() -> Self {
        Self {
            name: "official".to_string(),
            url: "https://github.com/anthropics/anthropic-quickstarts".to_string(),
            branch: "main".to_string(),
            description: Some("Anthropic Official Quickstarts".to_string()),
            skill_count: 0,
            last_synced: None,
            is_official: true,
        }
    }
}
