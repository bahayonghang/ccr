use crate::core::error::{CcrError, Result};
use crate::core::http::HTTP_CLIENT;
use crate::models::Platform;
use crate::models::skill::{Skill, SkillRepository};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
struct SkillsConfig {
    #[serde(default)]
    repositories: Vec<SkillRepository>,
}

pub struct SkillsManager {
    base_path: PathBuf,
    config_path: PathBuf,
}

impl SkillsManager {
    pub fn new(platform: Platform) -> Result<Self> {
        let home = dirs::home_dir()
            .ok_or_else(|| CcrError::ConfigError("Cannot find home directory".into()))?;

        // Determine paths based on platform
        // For Claude, we use the standard ~/.claude/skills
        // For others, we might use ~/.ccr/platforms/<platform>/skills
        let (base_path, config_dir) = match platform {
            Platform::Claude => (
                home.join(".claude").join("skills"),
                home.join(".ccr").join("platforms").join("claude"),
            ),
            p => (
                home.join(".ccr")
                    .join("platforms")
                    .join(p.short_name())
                    .join("skills"),
                home.join(".ccr").join("platforms").join(p.short_name()),
            ),
        };

        let config_path = config_dir.join("skills.toml");

        Ok(Self {
            base_path,
            config_path,
        })
    }

    fn ensure_base_dir(&self) -> Result<()> {
        if !self.base_path.exists() {
            fs::create_dir_all(&self.base_path).map_err(|e| {
                CcrError::IoError(std::io::Error::new(
                    e.kind(),
                    format!(
                        "Failed to create skills directory {:?}: {}",
                        self.base_path, e
                    ),
                ))
            })?;
        }
        Ok(())
    }

    fn ensure_config_dir(&self) -> Result<()> {
        if let Some(parent) = self.config_path.parent()
            && !parent.exists()
        {
            fs::create_dir_all(parent).map_err(|e| {
                CcrError::IoError(std::io::Error::new(
                    e.kind(),
                    format!("Failed to create config directory {:?}: {}", parent, e),
                ))
            })?;
        }
        Ok(())
    }

    // === Local Skills Management ===

    pub fn list_skills(&self) -> Result<Vec<Skill>> {
        self.ensure_base_dir()?;
        let mut skills = Vec::new();

        for entry in fs::read_dir(&self.base_path).map_err(CcrError::IoError)? {
            let entry = entry.map_err(CcrError::IoError)?;
            let path = entry.path();
            if path.is_dir() {
                let name = path
                    .file_name()
                    .expect("文件名应该存在")
                    .to_string_lossy()
                    .to_string();
                let skill_file = path.join("SKILL.md");

                if skill_file.exists() {
                    let instruction = fs::read_to_string(&skill_file).map_err(CcrError::IoError)?;
                    let description = instruction.lines().next().map(|s| s.to_string());
                    let metadata = Skill::parse_metadata(&instruction);

                    skills.push(Skill {
                        name,
                        description,
                        path: path.to_string_lossy().to_string(),
                        instruction,
                        metadata,
                        is_remote: false,
                        repository: None,
                    });
                }
            }
        }

        Ok(skills)
    }

    #[allow(dead_code)]
    pub fn get_skill(&self, name: &str) -> Result<Skill> {
        let skill_path = self.base_path.join(name);
        let skill_file = skill_path.join("SKILL.md");

        if !skill_file.exists() {
            return Err(CcrError::ResourceNotFound(format!(
                "Skill '{}' not found",
                name
            )));
        }

        let instruction = fs::read_to_string(&skill_file).map_err(CcrError::IoError)?;
        let description = instruction.lines().next().map(|s| s.to_string());
        let metadata = Skill::parse_metadata(&instruction);

        Ok(Skill {
            name: name.to_string(),
            description,
            path: skill_path.to_string_lossy().to_string(),
            instruction,
            metadata,
            is_remote: false,
            repository: None,
        })
    }

    pub fn install_skill(&self, name: &str, instruction: &str) -> Result<()> {
        self.ensure_base_dir()?;
        let skill_path = self.base_path.join(name);

        if skill_path.exists() {
            return Err(CcrError::ResourceAlreadyExists(format!(
                "Skill '{}' already exists",
                name
            )));
        }

        fs::create_dir_all(&skill_path).map_err(CcrError::IoError)?;
        fs::write(skill_path.join("SKILL.md"), instruction).map_err(CcrError::IoError)?;

        Ok(())
    }

    pub fn uninstall_skill(&self, name: &str) -> Result<()> {
        let skill_path = self.base_path.join(name);

        if !skill_path.exists() {
            return Err(CcrError::ResourceNotFound(format!(
                "Skill '{}' not found",
                name
            )));
        }

        fs::remove_dir_all(skill_path).map_err(CcrError::IoError)?;

        Ok(())
    }

    // === Repository Management ===

    fn load_config(&self) -> Result<SkillsConfig> {
        if !self.config_path.exists() {
            return Ok(SkillsConfig::default());
        }

        let content = fs::read_to_string(&self.config_path).map_err(CcrError::IoError)?;
        toml::from_str(&content).map_err(|e| CcrError::ConfigFormatInvalid(e.to_string()))
    }

    fn save_config(&self, config: &SkillsConfig) -> Result<()> {
        self.ensure_config_dir()?;
        let content =
            toml::to_string_pretty(config).map_err(|e| CcrError::ConfigError(e.to_string()))?;
        fs::write(&self.config_path, content).map_err(CcrError::IoError)?;
        Ok(())
    }

    pub fn list_repositories(&self) -> Result<Vec<SkillRepository>> {
        let config = self.load_config()?;
        Ok(config.repositories)
    }

    pub fn add_repository(&self, repo: SkillRepository) -> Result<()> {
        let mut config = self.load_config()?;
        if config.repositories.iter().any(|r| r.name == repo.name) {
            return Err(CcrError::ResourceAlreadyExists(format!(
                "Repository '{}' already exists",
                repo.name
            )));
        }
        config.repositories.push(repo);
        self.save_config(&config)
    }

    pub fn remove_repository(&self, name: &str) -> Result<()> {
        let mut config = self.load_config()?;
        let initial_len = config.repositories.len();
        config.repositories.retain(|r| r.name != name);

        if config.repositories.len() == initial_len {
            return Err(CcrError::ResourceNotFound(format!(
                "Repository '{}' not found",
                name
            )));
        }

        self.save_config(&config)
    }

    // === Remote Skills Discovery ===

    pub async fn fetch_remote_skills(&self, repo_name: &str) -> Result<Vec<Skill>> {
        let repos = self.list_repositories()?;
        let repo = repos.iter().find(|r| r.name == repo_name).ok_or_else(|| {
            CcrError::ResourceNotFound(format!("Repository '{}' not found", repo_name))
        })?;

        // Parse GitHub URL to get owner/repo
        // Expected format: https://github.com/owner/repo
        let url_parts: Vec<&str> = repo.url.trim_end_matches('/').split('/').collect();
        if url_parts.len() < 5 || url_parts[2] != "github.com" {
            return Err(CcrError::ConfigError(format!(
                "Invalid GitHub URL: {}",
                repo.url
            )));
        }
        let owner = url_parts[3];
        let repo_name_gh = url_parts[4];

        // Fetch contents of the repository root or a specific folder if we knew it
        // For now, assume skills are folders in root or we scan recursively?
        // Let's assume a simple structure: each folder in root is a skill if it has SKILL.md?
        // Or maybe look for a specific folder?
        // cc-switch says "custom directory scanning".
        // Let's try to list root contents first.

        let api_url = format!(
            "https://api.github.com/repos/{}/{}/contents",
            owner, repo_name_gh
        );

        let client = &*HTTP_CLIENT;
        let resp =
            client.get(&api_url).send().await.map_err(|e| {
                CcrError::NetworkError(format!("Failed to fetch repository: {}", e))
            })?;

        if !resp.status().is_success() {
            return Err(CcrError::NetworkError(format!(
                "GitHub API error: {}",
                resp.status()
            )));
        }

        let contents: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| CcrError::NetworkError(format!("Failed to parse JSON: {}", e)))?;

        let mut skills = Vec::new();

        if let Some(array) = contents.as_array() {
            for item in array {
                if let (Some("dir"), Some(name)) = (item["type"].as_str(), item["name"].as_str()) {
                    // Check if this directory has SKILL.md
                    // We can try to fetch contents/name/SKILL.md
                    // To avoid too many requests, maybe we can just assume it is a skill if we are scanning?
                    // Or we can fetch the raw SKILL.md

                    let raw_url = format!(
                        "https://raw.githubusercontent.com/{}/{}/{}/{}/SKILL.md",
                        owner, repo_name_gh, repo.branch, name
                    );

                    // We won't fetch content for ALL skills immediately to avoid rate limits
                    // We will return a "Remote Skill" which might need a separate struct or flag
                    // But `Skill` struct has `instruction`.
                    // Let's fetch it on demand? No, `fetch_remote_skills` implies getting the list.
                    // If we want to show description, we need to fetch it.
                    // Let's try to fetch SKILL.md for each dir. This is slow.
                    // Better approach: Fetch the repo tree recursively?
                    // Or maybe the repo has a manifest?

                    // For this implementation, let's just list the directories and set instruction to "Remote skill from ..."
                    // User can "install" which will fetch the real content.

                    skills.push(Skill {
                        name: name.to_string(),
                        description: Some(format!("Remote skill from {}", repo.name)),
                        path: raw_url,
                        instruction: String::new(),
                        metadata: crate::models::skill::SkillMetadata::default(),
                        is_remote: true,
                        repository: Some(repo.name.clone()),
                    });
                }
            }
        }

        Ok(skills)
    }

    pub async fn fetch_skill_content(&self, url: &str) -> Result<String> {
        let client = &*HTTP_CLIENT;
        let resp =
            client.get(url).send().await.map_err(|e| {
                CcrError::NetworkError(format!("Failed to fetch skill content: {}", e))
            })?;

        if !resp.status().is_success() {
            return Err(CcrError::NetworkError(format!(
                "Failed to download content: {}",
                resp.status()
            )));
        }

        resp.text()
            .await
            .map_err(|e| CcrError::NetworkError(format!("Failed to read content: {}", e)))
    }
}
