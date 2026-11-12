//! Reviewed catalog imports. Plaintext never leaves this opaque plan for display.
use super::source::{self, SourceSaveOutcome};
use ccr_config::{ConfigManager, PlatformPaths, platforms::base::parse_config_from_str};
use ccr_core::{CcrError, Result};
use std::collections::BTreeSet;

pub struct Preview {
    revision: String,
    content: String,
    input: String,
    paths: PlatformPaths,
    pub added: Vec<String>,
    pub conflicts: Vec<String>,
    /// 替换全部时会被移除的现有方案；其余导入方式为空。
    pub removed: Vec<String>,
    pub replace_all: bool,
}

impl Preview {
    pub fn matches_input(&self, input: &str) -> bool {
        self.input == input
    }
}

fn invalid() -> CcrError {
    CcrError::ValidationError("Invalid profile import".into())
}

/// Build a reviewable intent; matching names are retained unless explicitly selected.
/// A full replacement is previewed only: the preimage backup happens in `commit`, so an
/// uncommitted preview never leaves backup files behind.
pub fn preview(
    paths: &PlatformPaths,
    input: &str,
    overwrite: &BTreeSet<String>,
    replace_all: bool,
) -> Result<Preview> {
    let incoming = parse_config_from_str(input).map_err(|_| invalid())?;
    if incoming.sections.is_empty() {
        return Err(invalid());
    }
    let original = source::read(paths)?;
    let mut current = if original.exists {
        parse_config_from_str(&original.content).map_err(|_| invalid())?
    } else {
        incoming.clone()
    };
    if !original.exists {
        current.sections.clear();
    }
    let added = incoming
        .sections
        .keys()
        .filter(|name| !current.sections.contains_key(*name))
        .cloned()
        .collect();
    let conflicts = incoming
        .sections
        .keys()
        .filter(|name| current.sections.contains_key(*name))
        .cloned()
        .collect();
    let mut removed = Vec::new();
    if replace_all {
        removed = current
            .sections
            .keys()
            .filter(|name| !incoming.sections.contains_key(*name))
            .cloned()
            .collect();
        current.sections = incoming.sections;
        current.default_config = incoming.default_config;
    } else {
        for (name, section) in incoming.sections {
            if !current.sections.contains_key(&name) || overwrite.contains(&name) {
                current.sections.insert(name, section);
            }
        }
    }
    let content = toml::to_string_pretty(&current).map_err(|_| invalid())?;
    Ok(Preview {
        revision: original.token,
        content,
        input: input.into(),
        paths: paths.clone(),
        added,
        conflicts,
        removed,
        replace_all,
    })
}

/// Caller holds the platform application lock and resolves the actual current item again.
/// Revision validation is owned by the atomic catalog writer; no force bypass is offered.
pub fn commit(
    paths: &PlatformPaths,
    preview: &Preview,
    current: Option<&str>,
) -> Result<SourceSaveOutcome> {
    if paths.profiles_file != preview.paths.profiles_file {
        return Err(invalid());
    }
    let current_document = source::read(paths)?;
    if current_document.token != preview.revision {
        return Ok(SourceSaveOutcome::Conflict);
    }
    if current_document.exists {
        ConfigManager::new(&paths.profiles_file).backup(Some("reviewed_import"))?;
    }
    source::save_without_activation(paths, current, &preview.content, &preview.revision)
}
