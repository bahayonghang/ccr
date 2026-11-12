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

/// Save an explicitly selected export to a new file with secret-file permissions.
/// Existing files and linked paths are never overwritten by temporary-output delivery.
pub fn save_export(destination: &Path, content: &str) -> Result<bool> {
    if !destination.is_absolute()
        || destination
            .components()
            .any(|part| matches!(part, std::path::Component::ParentDir))
    {
        return Err(CcrError::ValidationError(
            "Invalid export destination".into(),
        ));
    }
    for path in destination.ancestors() {
        match std::fs::symlink_metadata(path) {
            Ok(meta) => {
                #[cfg(windows)]
                {
                    use std::os::windows::fs::MetadataExt;
                    if meta.file_attributes() & 0x400 != 0 {
                        return Err(CcrError::ValidationError(
                            "Linked export destination".into(),
                        ));
                    }
                }
                if meta.file_type().is_symlink() {
                    return Err(CcrError::ValidationError(
                        "Linked export destination".into(),
                    ));
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(CcrError::IoError(error)),
        }
    }
    use ccr_core::core::{
        VersionedWriteOutcome,
        guarded_write::{WriteOptions, write_guarded_versioned},
    };
    Ok(matches!(
        write_guarded_versioned(
            destination,
            content.as_bytes(),
            "",
            &WriteOptions {
                secret: true,
                ..Default::default()
            }
        )?,
        VersionedWriteOutcome::Written
    ))
}

/// Save a catalog without changing activation or allowing removal of the current item.
/// The caller holds its platform application lock; unknown fields and CAS are retained.
pub fn save_without_activation(
    paths: &PlatformPaths,
    current: Option<&str>,
    content: &str,
    token: &str,
) -> Result<SourceSaveOutcome> {
    let Ok(mut document) = toml::from_str::<toml::Value>(content) else {
        return save(paths, current, content, token, false);
    };
    let Ok(parsed) = ccr_config::platforms::base::parse_config_from_str(content) else {
        return save(paths, current, content, token, false);
    };
    if let Some(table) = document.as_table_mut() {
        table.insert("current_config".into(), current.unwrap_or_default().into());
        table
            .entry("default_config")
            .or_insert(parsed.default_config.into());
    }
    let normalized = toml::to_string_pretty(&document)
        .map_err(|_| CcrError::ConfigError("Cannot serialize profiles catalog".into()))?;
    save(paths, current, &normalized, token, false)
}

/// Legacy callers may explicitly acknowledge activation removal; CAS still applies.
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
