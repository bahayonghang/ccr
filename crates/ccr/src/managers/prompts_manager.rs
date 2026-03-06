use crate::core::error::{CcrError, Result};
use crate::models::Platform;
use crate::models::prompt::{PromptPreset, PromptTarget};
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

impl PromptsManager {
    pub fn new(platform: Platform) -> Result<Self> {
        let home = dirs::home_dir()
            .ok_or_else(|| CcrError::ConfigError("Cannot find home directory".into()))?;

        let platform_dir = match platform {
            Platform::Claude => home.join(".claude"),
            Platform::Codex => home.join(".codex"),
            Platform::Gemini => home.join(".gemini"),
            p => home.join(".ccr").join("platforms").join(p.short_name()),
        };

        let config_path = home
            .join(".ccr")
            .join("platforms")
            .join(platform.short_name())
            .join("prompts.toml");

        Ok(Self {
            config_path,
            platform_dir,
        })
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
