//! Existing prompt-preset schema with original-version writes for embedding clients.
use crate::platform_settings::{Document, Format, SaveOutcome, Snapshot};
use ccr_config::Platform;
use ccr_core::core::BackupPolicy;
use ccr_skills::{PromptPreset, PromptsManager};
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Invalid,
    Storage,
    Conflict,
    Missing,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Invalid => "prompt_preset_invalid",
            Self::Storage => "prompt_preset_storage_failed",
            Self::Conflict => "prompt_preset_conflict",
            Self::Missing => "prompt_preset_missing",
        })
    }
}
impl std::error::Error for Error {}

pub enum Mutation {
    Upsert(PromptPreset),
    Delete(String),
    Replace(Vec<PromptPreset>),
}
pub struct Catalog {
    document: Document,
    snapshot: Snapshot,
    pub presets: Vec<PromptPreset>,
}
impl Catalog {
    pub fn read(
        platform: Platform,
        home: &Path,
        backups: &Path,
        locks: &Path,
    ) -> Result<Self, Error> {
        let manager = PromptsManager::for_home(platform, home);
        let document = Document::new(
            manager.config_path().to_owned(),
            None,
            BackupPolicy::Dir {
                dir: backups.to_owned(),
                prefix: "prompt-presets".into(),
            },
            Some(locks.to_owned()),
            Format::Toml,
        );
        let snapshot = document.read().map_err(|_| Error::Storage)?;
        let presets =
            PromptsManager::parse_presets(&snapshot.content).map_err(|_| Error::Invalid)?;
        Ok(Self {
            document,
            snapshot,
            presets,
        })
    }
    pub fn commit(&self, mutation: Mutation) -> Result<(), Error> {
        let mut root = if self.snapshot.exists {
            toml::from_str::<toml::Value>(&self.snapshot.content).map_err(|_| Error::Invalid)?
        } else {
            toml::Value::Table(Default::default())
        };
        let rows = root
            .as_table_mut()
            .ok_or(Error::Invalid)?
            .entry("presets")
            .or_insert_with(|| toml::Value::Array(Vec::new()))
            .as_array_mut()
            .ok_or(Error::Invalid)?;
        match mutation {
            Mutation::Upsert(preset) => {
                validate(&preset)?;
                let incoming = toml::Value::try_from(&preset).map_err(|_| Error::Invalid)?;
                let matches: Vec<_> = rows
                    .iter()
                    .enumerate()
                    .filter(|(_, row)| {
                        row.get("name").and_then(toml::Value::as_str) == Some(&preset.name)
                    })
                    .map(|(index, _)| index)
                    .collect();
                if matches.len() > 1 {
                    return Err(Error::Invalid);
                }
                if let Some(index) = matches.first() {
                    let target = rows[*index].as_table_mut().ok_or(Error::Invalid)?;
                    for (key, value) in incoming.as_table().ok_or(Error::Invalid)? {
                        target.insert(key.clone(), value.clone());
                    }
                    if preset.description.is_none() {
                        target.remove("description");
                    }
                } else {
                    rows.push(incoming);
                }
            }
            Mutation::Delete(name) => {
                let before = rows.len();
                rows.retain(|row| row.get("name").and_then(toml::Value::as_str) != Some(&name));
                if before == rows.len() {
                    return Err(Error::Missing);
                }
            }
            Mutation::Replace(presets) => {
                let mut names = std::collections::HashSet::new();
                for preset in &presets {
                    validate(preset)?;
                    if !names.insert(&preset.name) {
                        return Err(Error::Invalid);
                    }
                }
                *rows = presets
                    .iter()
                    .map(toml::Value::try_from)
                    .collect::<Result<_, _>>()
                    .map_err(|_| Error::Invalid)?;
            }
        }
        let content = toml::to_string_pretty(&root).map_err(|_| Error::Invalid)?;
        match self
            .document
            .save_validated(&content, &self.snapshot)
            .map_err(|_| Error::Storage)?
        {
            SaveOutcome::Saved { .. } => Ok(()),
            SaveOutcome::Conflict => Err(Error::Conflict),
        }
    }
}
fn validate(preset: &PromptPreset) -> Result<(), Error> {
    if preset.name.trim().is_empty()
        || preset.name != preset.name.trim()
        || preset.content.len() > 1024 * 1024
    {
        Err(Error::Invalid)
    } else {
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use ccr_skills::PromptTarget;
    #[test]
    fn original_preset_revision_and_extensions_survive_native_edits() {
        let root = tempfile::tempdir().unwrap();
        let path = PromptsManager::for_home(Platform::Claude, root.path())
            .config_path()
            .to_owned();
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "extension = 7\n[[presets]]\nname = 'fixture'\ntarget_file = 'claude'\ncontent = 'old'\nunknown = 'keep'\n").unwrap();
        let catalog = Catalog::read(
            Platform::Claude,
            root.path(),
            &root.path().join("backups"),
            &root.path().join("locks"),
        )
        .unwrap();
        catalog
            .commit(Mutation::Upsert(PromptPreset {
                name: "fixture".into(),
                description: None,
                target_file: PromptTarget::Claude,
                content: "changed".into(),
                tags: Vec::new(),
            }))
            .unwrap();
        assert_eq!(
            catalog.commit(Mutation::Delete("fixture".into())),
            Err(Error::Conflict)
        );
        let saved: toml::Value = toml::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(saved["extension"].as_integer(), Some(7));
        assert_eq!(saved["presets"][0]["unknown"].as_str(), Some("keep"));
    }
}
