//! State-free Claude settings collection policy shared by native and Tauri adapters.
use crate::platform_settings::{self, Document, Snapshot, json};
use serde_json::{Value, json as value};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Collection {
    Agents,
    SlashCommands,
    Plugins,
}
impl Collection {
    pub fn key(self) -> &'static str {
        match self {
            Self::Agents => "agents",
            Self::SlashCommands => "slashCommands",
            Self::Plugins => "plugins",
        }
    }
    pub fn validate(self, entry: &Value) -> Result<(), Error> {
        let valid = match self {
            Self::Agents => serde_json::from_value::<ccr_types::Agent>(entry.clone()).is_ok(),
            Self::SlashCommands => {
                serde_json::from_value::<ccr_types::SlashCommand>(entry.clone()).is_ok()
            }
            Self::Plugins => serde_json::from_value::<ccr_types::Plugin>(entry.clone()).is_ok(),
        };
        if valid { Ok(()) } else { Err(Error::Invalid) }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Error {
    Invalid,
    Missing,
    Duplicate,
    Conflict,
    Storage,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Invalid => "claude_collection_invalid",
            Self::Missing => "claude_collection_entry_missing",
            Self::Duplicate => "claude_collection_name_exists",
            Self::Conflict => "claude_collection_conflict",
            Self::Storage => "claude_collection_storage_failed",
        })
    }
}
impl std::error::Error for Error {}

#[derive(Clone)]
pub enum Mutation {
    Add(Value),
    Patch(Value),
    Replace(Value),
    Delete,
}

/// Existing Tauri settings transactions retain their environment and retry owner.
pub fn apply_typed(
    settings: &mut ccr_types::ClaudeSettings,
    collection: Collection,
    name: &str,
    mutation: Mutation,
) -> Result<Value, Error> {
    let mut value = serde_json::to_value(&*settings).map_err(|_| Error::Invalid)?;
    apply(&mut value, collection, name, mutation)?;
    let rows = value.get(collection.key()).cloned().ok_or(Error::Invalid)?;
    *settings = serde_json::from_value(value).map_err(|_| Error::Invalid)?;
    Ok(rows)
}

fn validate_name(name: &str) -> Result<(), Error> {
    if name.trim().is_empty() || name != name.trim() || name.chars().any(char::is_control) {
        Err(Error::Invalid)
    } else {
        Ok(())
    }
}

/// Mutates only the requested array; unrelated settings and extension fields survive.
/// Replace retains the legacy Tauri replacement semantics. Native editors submit
/// Patch so omitted credentials and unknown fields stay in their original entry.
pub fn apply(
    settings: &mut Value,
    collection: Collection,
    name: &str,
    mutation: Mutation,
) -> Result<(), Error> {
    validate_name(name)?;
    let root = settings.as_object_mut().ok_or(Error::Invalid)?;
    let rows = root
        .entry(collection.key())
        .or_insert_with(|| value!([]))
        .as_array_mut()
        .ok_or(Error::Invalid)?;
    let matches = rows
        .iter()
        .enumerate()
        .filter(|(_, item)| item.get("name").and_then(Value::as_str) == Some(name))
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    if matches.len() > 1 {
        return Err(Error::Duplicate);
    }
    let index = matches.first().copied();
    match mutation {
        Mutation::Add(mut entry) => {
            if index.is_some() {
                return Err(Error::Duplicate);
            }
            entry
                .as_object_mut()
                .ok_or(Error::Invalid)?
                .insert("name".into(), Value::String(name.into()));
            collection.validate(&entry)?;
            rows.push(entry);
        }
        Mutation::Patch(patch) => {
            let index = index.ok_or(Error::Missing)?;
            if patch
                .get("name")
                .is_some_and(|value| value.as_str() != Some(name))
            {
                return Err(Error::Invalid);
            }
            let mut entry = rows[index].clone();
            merge(&mut entry, patch)?;
            collection.validate(&entry)?;
            rows[index] = entry;
        }
        Mutation::Replace(entry) => {
            let index = index.ok_or(Error::Missing)?;
            collection.validate(&entry)?;
            let new_name = entry
                .get("name")
                .and_then(Value::as_str)
                .ok_or(Error::Invalid)?;
            if rows.iter().enumerate().any(|(other, row)| {
                other != index && row.get("name").and_then(Value::as_str) == Some(new_name)
            }) {
                return Err(Error::Duplicate);
            }
            rows[index] = entry;
        }
        Mutation::Delete => {
            rows.remove(index.ok_or(Error::Missing)?);
        }
    }
    Ok(())
}

fn merge(target: &mut Value, patch: Value) -> Result<(), Error> {
    let target = target.as_object_mut().ok_or(Error::Invalid)?;
    for (key, incoming) in patch.as_object().ok_or(Error::Invalid)? {
        if incoming.is_object() && target.get(key).is_some_and(Value::is_object) {
            merge(target.get_mut(key).ok_or(Error::Invalid)?, incoming.clone())?;
        } else {
            target.insert(key.clone(), incoming.clone());
        }
    }
    Ok(())
}

#[derive(Clone)]
pub struct Row {
    pub name: String,
    pub enabled: bool,
}

pub fn list(document: &Document, collection: Collection) -> Result<Vec<Row>, Error> {
    let snapshot = document.read().map_err(|_| Error::Storage)?;
    let settings = json::parse(&snapshot).map_err(|_| Error::Invalid)?;
    let Some(entries) = settings.get(collection.key()) else {
        return Ok(Vec::new());
    };
    let entries = entries.as_array().ok_or(Error::Invalid)?;
    if entries.len() > 1000 {
        return Err(Error::Invalid);
    }
    entries
        .iter()
        .map(|entry| {
            collection.validate(entry)?;
            let name = entry
                .get("name")
                .and_then(Value::as_str)
                .ok_or(Error::Invalid)?
                .to_owned();
            let enabled = match collection {
                Collection::Plugins => entry
                    .get("enabled")
                    .and_then(Value::as_bool)
                    .unwrap_or(true),
                _ => !entry
                    .get("disabled")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
            };
            Ok(Row { name, enabled })
        })
        .collect()
}

/// Source-bound original version; no credentials are exposed by list/open projections.
pub struct Session {
    document: Document,
    snapshot: Snapshot,
    collection: Collection,
    name: String,
    exists: bool,
}
impl Session {
    pub fn open(document: Document, collection: Collection, name: &str) -> Result<Self, Error> {
        validate_name(name)?;
        let snapshot = document.read().map_err(|_| Error::Storage)?;
        let settings = json::parse(&snapshot).map_err(|_| Error::Invalid)?;
        let entries = settings
            .get(collection.key())
            .map(|value| value.as_array().ok_or(Error::Invalid))
            .transpose()?;
        let matches = entries
            .into_iter()
            .flatten()
            .filter(|entry| entry.get("name").and_then(Value::as_str) == Some(name))
            .count();
        if matches > 1 {
            return Err(Error::Duplicate);
        }
        Ok(Self {
            document,
            snapshot,
            collection,
            name: name.into(),
            exists: matches == 1,
        })
    }
    pub fn exists(&self) -> bool {
        self.exists
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn collection(&self) -> Collection {
        self.collection
    }
    pub fn path(&self) -> &std::path::Path {
        self.document.path()
    }
    pub fn commit(&self, mutation: Mutation) -> Result<(), Error> {
        let mut settings = json::parse(&self.snapshot).map_err(|_| Error::Invalid)?;
        apply(&mut settings, self.collection, &self.name, mutation)?;
        let content = serde_json::to_string_pretty(&settings).map_err(|_| Error::Invalid)?;
        match self
            .document
            .save_validated(&content, &self.snapshot)
            .map_err(|_| Error::Storage)?
        {
            platform_settings::SaveOutcome::Saved { .. } => Ok(()),
            platform_settings::SaveOutcome::Conflict => Err(Error::Conflict),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use ccr_core::core::BackupPolicy;
    #[test]
    fn typed_collections_preserve_unknown_fields_secrets_and_original_versions() {
        let root = tempfile::tempdir().unwrap();
        let document = Document::new(
            root.path().join("settings.json"),
            None,
            BackupPolicy::Dir {
                dir: root.path().join("backups"),
                prefix: "claude".into(),
            },
            Some(root.path().join("locks")),
            platform_settings::Format::Json,
        );
        for (collection, payload) in [
            (
                Collection::Agents,
                value!({"model":"fixture","extension":{"keep":7}}),
            ),
            (
                Collection::SlashCommands,
                value!({"description":"fixture","command":"fixture-command","extension":{"keep":7}}),
            ),
            (
                Collection::Plugins,
                value!({"id":"fixture","version":"1","config":{"token":"secret"},"extension":{"keep":7}}),
            ),
        ] {
            let original = Session::open(document.clone(), collection, "example").unwrap();
            original.commit(Mutation::Add(payload)).unwrap();
            assert_eq!(original.commit(Mutation::Delete), Err(Error::Missing));
            let session = Session::open(document.clone(), collection, "example").unwrap();
            session
                .commit(Mutation::Patch(value!({"extension":{"new":8}})))
                .unwrap();
            assert_eq!(session.commit(Mutation::Delete), Err(Error::Conflict));
            let saved = json::parse(&document.read().unwrap()).unwrap();
            assert_eq!(
                saved[collection.key()][0]["extension"],
                value!({"keep":7,"new":8})
            );
            if collection == Collection::Plugins {
                assert_eq!(saved[collection.key()][0]["config"]["token"], "secret");
            }
            Session::open(document.clone(), collection, "example")
                .unwrap()
                .commit(Mutation::Delete)
                .unwrap();
        }
    }
}
