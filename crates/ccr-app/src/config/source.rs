//! Canonical profile-source policy shared by desktop adapters. All I/O is blocking.

use ccr_config::{ConfigManager, PlatformPaths, parse_profiles_from_str};
use ccr_core::{CcrError, Result};
use std::path::{Path, PathBuf};

/// Plaintext is scoped to an explicitly opened source editor; never log this value.
pub struct SourceSnapshot {
    pub content: String,
    pub token: String,
    pub path: PathBuf,
    pub exists: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceInvalidKind {
    Syntax,
    Semantic,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceSaveOutcome {
    Invalid {
        kind: SourceInvalidKind,
        message: &'static str,
        position: Option<(usize, usize)>,
    },
    ActivationConflict {
        current: String,
    },
    Conflict,
    Saved {
        token: String,
        profiles_count: usize,
    },
}

pub fn read(paths: &PlatformPaths) -> Result<SourceSnapshot> {
    let manager = ConfigManager::new(&paths.profiles_file);
    let document = manager.canonical_toml()?;
    Ok(match document {
        Some((content, token)) => SourceSnapshot {
            content,
            token,
            path: manager.profiles_db().to_path_buf(),
            exists: true,
        },
        None => SourceSnapshot {
            content: String::new(),
            token: String::new(),
            path: manager.profiles_db().to_path_buf(),
            exists: false,
        },
    })
}

/// Dedicated profile export retains its historical secret-only contract.
/// Generic `ConfigService::export_config` separately supports redacted export.
pub fn export(profiles_file: &Path, include_secrets: bool) -> Result<String> {
    if !include_secrets {
        return Err(CcrError::ValidationError(
            "Redacted profiles export is not supported".into(),
        ));
    }
    ConfigManager::new(profiles_file)
        .canonical_toml()?
        .map(|(content, _)| content)
        .ok_or_else(|| CcrError::ConfigError("Failed to read profiles catalog".into()))
}

/// `force` acknowledges removal of the active profile; it never bypasses CAS.
/// Call inside the platform's synchronous operation context when one is needed.
pub fn save(
    paths: &PlatformPaths,
    current_profile: Option<&str>,
    content: &str,
    token: &str,
    force: bool,
) -> Result<SourceSaveOutcome> {
    if let Err(error) = toml::from_str::<toml::Value>(content) {
        let position = error.span().and_then(|span| {
            let prefix = content.get(..span.start.min(content.len()))?;
            Some((
                prefix.bytes().filter(|byte| *byte == b'\n').count() + 1,
                prefix.rsplit('\n').next()?.chars().count() + 1,
            ))
        });
        return Ok(SourceSaveOutcome::Invalid {
            kind: SourceInvalidKind::Syntax,
            message: "Invalid profiles TOML syntax",
            position,
        });
    }
    let profiles = match parse_profiles_from_str(content) {
        Ok(profiles) if !profiles.is_empty() => profiles,
        Ok(_) => {
            return Ok(SourceSaveOutcome::Invalid {
                kind: SourceInvalidKind::Semantic,
                message: "Profiles TOML must contain at least one profile",
                position: None,
            });
        }
        Err(_) => {
            return Ok(SourceSaveOutcome::Invalid {
                kind: SourceInvalidKind::Semantic,
                message: "Profiles TOML does not match the supported profile structure",
                position: None,
            });
        }
    };
    if let Some(current) = current_profile
        && !profiles.contains_key(current)
        && !force
    {
        return Ok(SourceSaveOutcome::ActivationConflict {
            current: current.into(),
        });
    }
    match ConfigManager::new(&paths.profiles_file).replace_toml(content, token) {
        Ok(token) => Ok(SourceSaveOutcome::Saved {
            token,
            profiles_count: profiles.len(),
        }),
        Err(CcrError::ValidationError(message)) if message == "配置版本冲突，请重新读取后重试" => {
            Ok(SourceSaveOutcome::Conflict)
        }
        Err(error) => Err(error),
    }
}
