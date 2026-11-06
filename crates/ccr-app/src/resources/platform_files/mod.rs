//! Captured directory resources for OpenCode and Antigravity.
//! File schema stays platform-owned; all writes reuse guarded CAS and backups.
pub mod antigravity;
pub mod opencode;
use crate::platform_settings::{self, Document, Format, SaveOutcome, Snapshot};
use ccr_core::core::{
    BackupPolicy, LockManager, VersionedWriteOutcome, backup_guarded,
    guarded_write::delete_guarded_versioned,
};
use std::{
    fs,
    path::{Component, Path, PathBuf},
};
const MAX_BYTES: u64 = 1024 * 1024;
const MAX_FILES: usize = 1000;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    OpenCodeAgent,
    OpenCodeCommand,
    OpenCodePlugin,
    AntigravityCommand,
    Extension,
}
impl Kind {
    pub fn extension(self) -> &'static str {
        match self {
            Self::OpenCodeAgent | Self::OpenCodeCommand => "md",
            Self::AntigravityCommand => "toml",
            Self::Extension => "json",
            Self::OpenCodePlugin => "js",
        }
    }
    pub fn writable(self) -> bool {
        !matches!(self, Self::Extension | Self::OpenCodePlugin)
    }
    pub fn initial_text(self) -> &'static str {
        match self {
            Self::OpenCodeAgent => {
                "---\ndescription: Describe the agent\nmode: subagent\n---\n\nWrite agent instructions here.\n"
            }
            Self::OpenCodeCommand => {
                "---\ndescription: Describe the command\n---\n\nWrite the command template here.\n"
            }
            Self::AntigravityCommand => {
                "description = \"Describe the command\"\nprompt = \"Write the prompt here.\"\n"
            }
            Self::Extension | Self::OpenCodePlugin => "",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidName,
    UnsafePath,
    Storage,
    TooLarge,
    Conflict,
    Missing,
    InvalidContent,
    ReadOnly,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InvalidName => "resource_invalid_name",
            Self::UnsafePath => "resource_unsafe_path",
            Self::Storage => "resource_storage_failed",
            Self::TooLarge => "resource_limit_exceeded",
            Self::Conflict => "resource_conflict",
            Self::Missing => "resource_missing",
            Self::InvalidContent => "resource_invalid_content",
            Self::ReadOnly => "resource_read_only",
        })
    }
}
impl std::error::Error for Error {}
#[derive(Clone)]
pub struct Row {
    pub key: String,
    pub bytes: u64,
    pub description: Option<String>,
    pub version: Option<String>,
    pub malformed: bool,
}
#[derive(Clone)]
pub struct Store {
    root: PathBuf,
    backups: PathBuf,
    locks: PathBuf,
    kind: Kind,
}
/// Raw source and original revision never enter retained operation snapshots.
pub struct Session {
    store: Store,
    snapshot: Snapshot,
    key: String,
}
impl Session {
    pub fn content(&self) -> &str {
        &self.snapshot.content
    }
    pub fn exists(&self) -> bool {
        self.snapshot.exists
    }
    pub fn key(&self) -> &str {
        &self.key
    }
    pub fn path(&self) -> &Path {
        &self.snapshot.source
    }
    pub fn kind(&self) -> Kind {
        self.store.kind
    }
    pub fn save(&self, content: &str) -> Result<(), Error> {
        if !self.kind().writable() {
            return Err(Error::ReadOnly);
        }
        if content.len() as u64 > MAX_BYTES {
            return Err(Error::TooLarge);
        }
        match self.kind() {
            Kind::OpenCodeAgent | Kind::OpenCodeCommand => {
                let doc = opencode::parse_markdown_document(content)
                    .map_err(|_| Error::InvalidContent)?;
                opencode::validate(&doc).map_err(|_| Error::InvalidContent)?;
            }
            Kind::AntigravityCommand => {
                antigravity::validate(content).map_err(|_| Error::InvalidContent)?;
            }
            Kind::Extension | Kind::OpenCodePlugin => return Err(Error::ReadOnly),
        }
        let path = self.store.path(&self.key)?;
        check_path(&self.store.backups)?;
        check_path(&self.store.locks)?;
        match self
            .store
            .document(path)
            .save_validated(content, &self.snapshot)
            .map_err(|_| Error::Storage)?
        {
            SaveOutcome::Saved { .. } => Ok(()),
            SaveOutcome::Conflict => Err(Error::Conflict),
        }
    }
    pub fn delete(&self) -> Result<(), Error> {
        if !self.kind().writable() {
            return Err(Error::ReadOnly);
        }
        if !self.exists() {
            return Err(Error::Missing);
        }
        let path = self.store.path(&self.key)?;
        check_path(&self.store.backups)?;
        check_path(&self.store.locks)?;
        LockManager::with_scoped_directory(&self.store.locks, || {
            backup_guarded(&path, &self.store.backup())?;
            delete_guarded_versioned(&path, &self.snapshot.token)
        })
        .map_err(|_| Error::Storage)
        .and_then(|outcome| match outcome {
            VersionedWriteOutcome::Written => Ok(()),
            VersionedWriteOutcome::Conflict => Err(Error::Conflict),
        })
    }
}
impl Store {
    /// Caller captures the exact directory once; construction has no I/O or globals.
    pub fn new(root: PathBuf, backups: PathBuf, locks: PathBuf, kind: Kind) -> Self {
        Self {
            root,
            backups,
            locks,
            kind,
        }
    }
    pub fn with_default_storage(root: PathBuf, kind: Kind) -> Result<Self, Error> {
        let paths = ccr_config::PlatformPaths::new(ccr_config::Platform::Claude)
            .map_err(|_| Error::Storage)?;
        let locks = LockManager::with_default_path().map_err(|_| Error::Storage)?;
        Ok(Self::new(
            root,
            paths.root.join("backups/platform-resources"),
            locks.lock_dir().to_path_buf(),
            kind,
        ))
    }
    pub fn kind(&self) -> Kind {
        self.kind
    }
    fn backup(&self) -> BackupPolicy {
        BackupPolicy::Dir {
            dir: self.backups.clone(),
            prefix: "platform-resource".into(),
        }
    }
    fn document(&self, path: PathBuf) -> Document {
        Document::new(
            path,
            None,
            self.backup(),
            Some(self.locks.clone()),
            Format::Json,
        )
    }
    pub fn directory(&self) -> &Path {
        &self.root
    }
    fn path(&self, key: &str) -> Result<PathBuf, Error> {
        validate_key(self.kind, key)?;
        check_path(&self.root)?;
        let path = self.root.join(key);
        check_path(&path)?;
        Ok(path)
    }
    pub fn open(&self, key: &str) -> Result<Session, Error> {
        if self.kind == Kind::OpenCodePlugin {
            return Err(Error::ReadOnly);
        }
        let path = self.path(key)?;
        match fs::metadata(&path) {
            Ok(meta) if !meta.is_file() => return Err(Error::UnsafePath),
            Ok(meta) if meta.len() > MAX_BYTES => return Err(Error::TooLarge),
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err(Error::Storage),
        }
        let snapshot = platform_settings::read_text(&path).map_err(|_| Error::Storage)?;
        if snapshot.content.len() as u64 > MAX_BYTES {
            return Err(Error::TooLarge);
        }
        Ok(Session {
            store: self.clone(),
            snapshot,
            key: key.into(),
        })
    }
    pub fn list(&self) -> Result<Vec<Row>, Error> {
        check_path(&self.root)?;
        if self.kind == Kind::Extension {
            return self.extensions();
        }
        if self.kind == Kind::OpenCodePlugin {
            return opencode::list_plugins(&self.root).map(|files| {
                files
                    .into_iter()
                    .map(|file| Row {
                        key: file.name,
                        bytes: file.size,
                        description: None,
                        version: None,
                        malformed: false,
                    })
                    .collect()
            });
        }
        let mut keys = Vec::new();
        let mut visited = 0;
        self.collect(&self.root, 0, &mut visited, &mut keys)?;
        keys.sort();
        keys.into_iter()
            .map(|key| {
                let session = self.open(&key)?;
                let metadata = match self.kind {
                    Kind::OpenCodeAgent | Kind::OpenCodeCommand => {
                        opencode::parse_markdown_document(session.content())
                            .map(|doc| opencode::yaml_string(&doc.frontmatter, "description"))
                    }
                    Kind::AntigravityCommand => {
                        antigravity::record(&key, session.content()).map(|value| {
                            value
                                .get("description")
                                .and_then(serde_json::Value::as_str)
                                .map(str::to_owned)
                        })
                    }
                    Kind::Extension | Kind::OpenCodePlugin => return Err(Error::ReadOnly),
                };
                Ok(Row {
                    key,
                    bytes: session.content().len() as u64,
                    description: metadata.as_ref().ok().cloned().flatten(),
                    version: None,
                    malformed: metadata.is_err(),
                })
            })
            .collect()
    }
    fn collect(
        &self,
        dir: &Path,
        depth: usize,
        visited: &mut usize,
        keys: &mut Vec<String>,
    ) -> Result<(), Error> {
        if depth > 16 {
            return Err(Error::TooLarge);
        }
        let entries = match fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(_) => return Err(Error::Storage),
        };
        for entry in entries {
            *visited += 1;
            if *visited > 4096 {
                return Err(Error::TooLarge);
            }
            let entry = entry.map_err(|_| Error::Storage)?;
            let path = entry.path();
            check_path(&path)?;
            let meta = entry.metadata().map_err(|_| Error::Storage)?;
            if meta.is_dir() && self.kind == Kind::AntigravityCommand {
                self.collect(&path, depth + 1, visited, keys)?;
            } else if meta.is_file()
                && path
                    .extension()
                    .is_some_and(|ext| ext == self.kind.extension())
            {
                if keys.len() >= MAX_FILES {
                    return Err(Error::TooLarge);
                }
                let key = path
                    .strip_prefix(&self.root)
                    .map_err(|_| Error::UnsafePath)?
                    .to_str()
                    .ok_or(Error::InvalidName)?
                    .replace('\\', "/");
                validate_key(self.kind, &key)?;
                keys.push(key);
            }
        }
        Ok(())
    }
    fn extensions(&self) -> Result<Vec<Row>, Error> {
        let entries = match fs::read_dir(&self.root) {
            Ok(entries) => entries,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(_) => return Err(Error::Storage),
        };
        let mut rows = Vec::new();
        for (index, entry) in entries.enumerate() {
            if index >= 4096 || rows.len() >= MAX_FILES {
                return Err(Error::TooLarge);
            }
            let entry = entry.map_err(|_| Error::Storage)?;
            check_path(&entry.path())?;
            if !entry.metadata().map_err(|_| Error::Storage)?.is_dir() {
                continue;
            }
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| Error::InvalidName)?;
            let preferred = entry.path().join("gemini-extension.json");
            let alternate = entry.path().join("package.json");
            check_path(&preferred)?;
            check_path(&alternate)?;
            let manifest = if preferred.exists() || !alternate.exists() {
                "gemini-extension.json"
            } else {
                "package.json"
            };
            let key = format!("{name}/{manifest}");
            let session = self.open(&key)?;
            let value = serde_json::from_str::<serde_json::Value>(session.content());
            rows.push(Row {
                key,
                bytes: session.content().len() as u64,
                description: value
                    .as_ref()
                    .ok()
                    .and_then(|v| v.get("description"))
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_owned),
                version: value
                    .as_ref()
                    .ok()
                    .and_then(|v| v.get("version"))
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_owned),
                malformed: value.is_err(),
            });
        }
        rows.sort_by(|a, b| a.key.cmp(&b.key));
        Ok(rows)
    }
}
pub fn validate_key(kind: Kind, key: &str) -> Result<(), Error> {
    if key.is_empty()
        || key != key.trim()
        || key.len() > 240
        || key.contains(['\\', ':', '\0', '<', '>', '"', '|', '?', '*'])
        || key.chars().any(char::is_control)
        || Path::new(key)
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
        || key
            .split('/')
            .any(|part| part.is_empty() || part.ends_with(['.', ' ']))
    {
        return Err(Error::InvalidName);
    }
    let segments: Vec<_> = key.split('/').collect();
    for segment in &segments {
        let stem = segment.split('.').next().unwrap_or("").to_ascii_uppercase();
        if matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
            || ["COM", "LPT"].iter().any(|prefix| {
                stem.strip_prefix(*prefix).is_some_and(|suffix| {
                    matches!(suffix, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9")
                })
            })
        {
            return Err(Error::InvalidName);
        }
    }
    if segments
        .last()
        .is_some_and(|name| *name == format!(".{}", kind.extension()))
    {
        return Err(Error::InvalidName);
    }
    if !key.ends_with(&format!(".{}", kind.extension()))
        || matches!(kind, Kind::OpenCodeAgent | Kind::OpenCodeCommand) && segments.len() != 1
        || kind == Kind::Extension
            && (segments.len() != 2
                || !matches!(segments[1], "gemini-extension.json" | "package.json"))
    {
        return Err(Error::InvalidName);
    }
    Ok(())
}
fn check_path(path: &Path) -> Result<(), Error> {
    if !path.is_absolute() {
        return Err(Error::UnsafePath);
    }
    for path in path.ancestors() {
        match fs::symlink_metadata(path) {
            Ok(meta) => {
                if meta.file_type().is_symlink() {
                    return Err(Error::UnsafePath);
                }
                #[cfg(windows)]
                {
                    use std::os::windows::fs::MetadataExt;
                    if meta.file_attributes() & 0x400 != 0 {
                        return Err(Error::UnsafePath);
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err(Error::Storage),
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests;
