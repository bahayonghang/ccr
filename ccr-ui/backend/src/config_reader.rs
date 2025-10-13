// Config File Reader
// Directly read and parse ~/.ccs_config.toml

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct CcsConfig {
    pub default_config: String,
    pub current_config: String,
    #[serde(flatten)]
    pub sections: HashMap<String, ConfigSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSection {
    pub description: Option<String>,
    pub base_url: Option<String>,
    pub auth_token: Option<String>,
    pub model: Option<String>,
    pub small_fast_model: Option<String>,
    pub provider: Option<String>,
    pub provider_type: Option<String>,
    pub account: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// Get config file path
pub fn get_config_path() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or_else(|| "Cannot get home directory".to_string())?;
    Ok(home.join(".ccs_config.toml"))
}

/// Read and parse config file
pub fn read_config() -> Result<CcsConfig, String> {
    let config_path = get_config_path()?;
    
    if !config_path.exists() {
        return Err(format!(
            "Config file not found: {}. Run 'ccr init' to create it.",
            config_path.display()
        ));
    }

    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let config: CcsConfig = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse config file: {}", e))?;

    Ok(config)
}

/// Mask sensitive token for display
pub fn mask_token(token: &str) -> String {
    if token.len() <= 10 {
        "*".repeat(token.len())
    } else {
        let prefix = &token[..4];
        let suffix = &token[token.len() - 4..];
        format!("{}...{}", prefix, suffix)
    }
}

