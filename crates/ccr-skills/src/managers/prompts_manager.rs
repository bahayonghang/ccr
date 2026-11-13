use crate::models::Platform;
use crate::models::prompt::{PromptPreset, PromptTarget};
use ccr_core::core::error::{CcrError, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
struct PromptsConfig {
    #[serde(default)]
    presets: Vec<PromptPreset>,
}

pub struct PromptsManager {
    config_path: PathBuf,
    platform_dir: PathBuf,
}

fn platform_prompt_dir(home: &std::path::Path, platform: Platform) -> PathBuf {
    match platform {
        Platform::Claude => home.join(".claude"),
        Platform::Codex => home.join(".codex"),
        Platform::Gemini => home.join(".gemini").join("antigravity-cli"),
        p => home.join(".ccr").join("platforms").join(p.short_name()),
    }
}

impl PromptsManager {
    pub fn new(platform: Platform) -> Result<Self> {
        let home = dirs::home_dir()
            .ok_or_else(|| CcrError::ConfigError("Cannot find home directory".into()))?;
        let manager = Self::for_home(platform, &home);
        if platform == Platform::Claude {
            // Claude Code 设置 CLAUDE_CONFIG_DIR 后只读该目录下的 CLAUDE.md，应用与读取跟随同一配置根。
            let config_dir = ccr_config::ClaudeRuntimePaths::from_env()?.config_dir;
            return Ok(manager.with_platform_dir(config_dir));
        }
        Ok(manager)
    }

    /// Replace the target-file directory with an explicitly captured platform root; no I/O.
    pub fn with_platform_dir(mut self, platform_dir: PathBuf) -> Self {
        self.platform_dir = platform_dir;
        self
    }

    /// Explicit roots for embedding clients and isolated fixtures; no I/O.
    pub fn for_home(platform: Platform, home: &std::path::Path) -> Self {
        let platform_dir = platform_prompt_dir(home, platform);
        let config_path = home
            .join(".ccr")
            .join("platforms")
            .join(platform.short_name())
            .join("prompts.toml");

        Self {
            config_path,
            platform_dir,
        }
    }

    pub fn config_path(&self) -> &std::path::Path {
        &self.config_path
    }

    /// Parse the established schema from a caller's versioned snapshot.
    pub fn parse_presets(content: &str) -> Result<Vec<PromptPreset>> {
        if content.trim().is_empty() {
            return Ok(Vec::new());
        }
        toml::from_str::<PromptsConfig>(content)
            .map(|config| config.presets)
            .map_err(|error| CcrError::ConfigFormatInvalid(error.to_string()))
    }

    fn ensure_config_dir(&self) -> Result<()> {
        if let Some(parent) = self.config_path.parent()
            && !parent.exists()
        {
            fs::create_dir_all(parent).map_err(CcrError::IoError)?;
        }
        Ok(())
    }

    fn load_config(&self) -> Result<PromptsConfig> {
        if !self.config_path.exists() {
            return Ok(PromptsConfig::default());
        }

        let content = fs::read_to_string(&self.config_path).map_err(CcrError::IoError)?;
        toml::from_str(&content).map_err(|e| CcrError::ConfigFormatInvalid(e.to_string()))
    }

    fn save_config(&self, config: &PromptsConfig) -> Result<()> {
        self.ensure_config_dir()?;
        let content =
            toml::to_string_pretty(config).map_err(|e| CcrError::ConfigError(e.to_string()))?;
        fs::write(&self.config_path, content).map_err(CcrError::IoError)?;
        Ok(())
    }

    pub fn list_presets(&self) -> Result<Vec<PromptPreset>> {
        let config = self.load_config()?;
        Ok(config.presets)
    }

    pub fn add_preset(&self, preset: PromptPreset) -> Result<()> {
        let mut config = self.load_config()?;
        if config.presets.iter().any(|p| p.name == preset.name) {
            return Err(CcrError::ResourceAlreadyExists(format!(
                "Preset '{}' already exists",
                preset.name
            )));
        }
        config.presets.push(preset);
        self.save_config(&config)
    }

    pub fn remove_preset(&self, name: &str) -> Result<()> {
        let mut config = self.load_config()?;
        let initial_len = config.presets.len();
        config.presets.retain(|p| p.name != name);

        if config.presets.len() == initial_len {
            return Err(CcrError::ResourceNotFound(format!(
                "Preset '{}' not found",
                name
            )));
        }

        self.save_config(&config)
    }

    pub fn get_preset(&self, name: &str) -> Result<PromptPreset> {
        let config = self.load_config()?;
        config
            .presets
            .into_iter()
            .find(|p| p.name == name)
            .ok_or_else(|| CcrError::ResourceNotFound(format!("Preset '{}' not found", name)))
    }

    pub fn apply_preset(&self, name: &str) -> Result<()> {
        let preset = self.get_preset(name)?;
        let target_file = self.platform_dir.join(preset.target_file.filename());

        // Backup existing file if it exists
        if target_file.exists() {
            let backup_path = target_file.with_extension("md.backup");
            fs::copy(&target_file, &backup_path).map_err(CcrError::IoError)?;
            tracing::info!("Backed up existing file to {:?}", backup_path);
        }

        // Write preset content
        fs::write(&target_file, &preset.content).map_err(CcrError::IoError)?;
        tracing::info!("Applied preset '{}' to {:?}", name, target_file);

        Ok(())
    }

    pub fn get_current_content(&self, target: PromptTarget) -> Result<String> {
        let target_file = self.platform_dir.join(target.filename());

        if !target_file.exists() {
            return Ok(String::new());
        }

        fs::read_to_string(&target_file).map_err(CcrError::IoError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn gemini_prompt_dir_uses_antigravity_cli() {
        let dir = platform_prompt_dir(Path::new("/home/test"), Platform::Gemini)
            .to_string_lossy()
            .replace('\\', "/");

        assert_eq!(dir, "/home/test/.gemini/antigravity-cli");
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn captured_claude_root_receives_applied_memory_and_default_home_is_untouched() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let custom = root.path().join("custom-claude");
        let default_memory = home.join(".claude/CLAUDE.md");
        fs::create_dir_all(default_memory.parent().unwrap()).unwrap();
        fs::write(&default_memory, "default home").unwrap();
        fs::create_dir_all(&custom).unwrap();
        let manager =
            PromptsManager::for_home(Platform::Claude, &home).with_platform_dir(custom.clone());
        manager
            .add_preset(PromptPreset {
                name: "memory".into(),
                description: None,
                target_file: PromptTarget::Claude,
                content: "# Applied\n".into(),
                tags: vec![],
            })
            .unwrap();
        manager.apply_preset("memory").unwrap();
        assert_eq!(
            fs::read_to_string(custom.join("CLAUDE.md")).unwrap(),
            "# Applied\n"
        );
        assert_eq!(
            manager.get_current_content(PromptTarget::Claude).unwrap(),
            "# Applied\n"
        );
        assert_eq!(fs::read_to_string(&default_memory).unwrap(), "default home");
        assert!(!home.join(".claude/CLAUDE.md.backup").exists());
    }
}
