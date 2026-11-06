//! Claude rule and output-style files on explicitly captured directories.
//! Markdown/frontmatter remain opaque text; edits never reconstruct their schema.
use crate::platform_settings::{self, Document, Format, SaveOutcome, Snapshot};
use ccr_core::core::{
    BackupPolicy, LockManager, VersionedWriteOutcome, backup_guarded,
    guarded_write::delete_guarded_versioned,
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Component, Path, PathBuf},
};

const MAX_FILE_BYTES: u64 = 1024 * 1024;
const MAX_FILES: usize = 1000;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Rule,
    OutputStyle,
}
impl Kind {
    pub fn directory(self) -> &'static str {
        match self {
            Self::Rule => "rules",
            Self::OutputStyle => "output-styles",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Error {
    InvalidName,
    UnsafePath,
    Storage,
    TooLarge,
    Conflict,
    Missing,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InvalidName => "claude_resource_invalid_name",
            Self::UnsafePath => "claude_resource_unsafe_path",
            Self::Storage => "claude_resource_storage_failed",
            Self::TooLarge => "claude_resource_limit_exceeded",
            Self::Conflict => "claude_resource_conflict",
            Self::Missing => "claude_resource_missing",
        })
    }
}
impl std::error::Error for Error {}

#[derive(Clone)]
pub struct Row {
    pub name: String,
    pub bytes: u64,
}

#[derive(Clone)]
pub struct Store {
    root: PathBuf,
    backups: PathBuf,
    locks: PathBuf,
}

/// Contains private source text and intentionally has no Debug/Serialize.
pub struct Session {
    store: Store,
    kind: Kind,
    name: String,
    snapshot: Snapshot,
}
impl Session {
    pub fn content(&self) -> &str {
        &self.snapshot.content
    }
    pub fn exists(&self) -> bool {
        self.snapshot.exists
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn kind(&self) -> Kind {
        self.kind
    }
    pub fn path(&self) -> &Path {
        &self.snapshot.source
    }
    pub fn save(&self, content: &str) -> Result<(), Error> {
        if content.len() as u64 > MAX_FILE_BYTES {
            return Err(Error::TooLarge);
        }
        let path = self.store.path(self.kind, &self.name)?;
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
        if !self.snapshot.exists {
            return Err(Error::Missing);
        }
        let path = self.store.path(self.kind, &self.name)?;
        check_path(&self.store.backups)?;
        check_path(&self.store.locks)?;
        // Both operations use the existing guarded per-path lock. A competing
        // writer after backup is still caught by the original delete token.
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
    /// Construction performs no I/O and does not consult HOME or process globals.
    pub fn new(claude_root: PathBuf, backups: PathBuf, locks: PathBuf) -> Self {
        Self {
            root: claude_root,
            backups,
            locks,
        }
    }
    fn backup(&self) -> BackupPolicy {
        BackupPolicy::Dir {
            dir: self.backups.clone(),
            prefix: "claude-resource".into(),
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
    fn path(&self, kind: Kind, name: &str) -> Result<PathBuf, Error> {
        validate_name(name)?;
        let directory = self.root.join(kind.directory());
        check_path(&self.root)?;
        check_path(&directory)?;
        let mut path = directory;
        for component in Path::new(name).components() {
            path.push(component);
            check_path(&path)?;
        }
        Ok(path)
    }
    pub fn list(&self, kind: Kind) -> Result<Vec<Row>, Error> {
        check_path(&self.root)?;
        let directory = self.root.join(kind.directory());
        check_path(&directory)?;
        let mut rows = Vec::new();
        collect(&directory, &directory, &mut rows, &mut 0)?;
        rows.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(rows)
    }
    pub fn open(&self, kind: Kind, name: &str) -> Result<Session, Error> {
        let path = self.path(kind, name)?;
        match fs::metadata(&path) {
            Ok(meta) if meta.len() > MAX_FILE_BYTES => return Err(Error::TooLarge),
            Ok(meta) if !meta.is_file() => return Err(Error::UnsafePath),
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err(Error::Storage),
        }
        let snapshot = platform_settings::read_text(&path).map_err(|_| Error::Storage)?;
        if snapshot.content.len() as u64 > MAX_FILE_BYTES {
            return Err(Error::TooLarge);
        }
        Ok(Session {
            store: self.clone(),
            kind,
            name: name.into(),
            snapshot,
        })
    }
}

pub fn validate_name(name: &str) -> Result<(), Error> {
    if name.is_empty()
        || name != name.trim()
        || name.len() > 240
        || !name.ends_with(".md")
        || name.contains(['\\', ':', '\0'])
        || name.chars().any(char::is_control)
        || Path::new(name)
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
        || name
            .split('/')
            .any(|part| part.is_empty() || part.ends_with(['.', ' ']) || part == ".md")
    {
        return Err(Error::InvalidName);
    }
    Ok(())
}

fn check_path(path: &Path) -> Result<(), Error> {
    if !path.is_absolute() {
        return Err(Error::UnsafePath);
    }
    for ancestor in path.ancestors() {
        match fs::symlink_metadata(ancestor) {
            Ok(meta) => {
                #[cfg(windows)]
                {
                    use std::os::windows::fs::MetadataExt;
                    if meta.file_attributes() & 0x400 != 0 {
                        return Err(Error::UnsafePath);
                    }
                }
                if meta.file_type().is_symlink() {
                    return Err(Error::UnsafePath);
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err(Error::Storage),
        }
    }
    Ok(())
}

fn collect(
    root: &Path,
    directory: &Path,
    rows: &mut Vec<Row>,
    visited: &mut usize,
) -> Result<(), Error> {
    if directory
        .strip_prefix(root)
        .map_err(|_| Error::UnsafePath)?
        .components()
        .count()
        > 16
    {
        return Err(Error::TooLarge);
    }
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
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
        let metadata = entry.metadata().map_err(|_| Error::Storage)?;
        if metadata.is_dir() {
            collect(root, &path, rows, visited)?;
        } else if metadata.is_file() && path.extension().is_some_and(|ext| ext == "md") {
            if rows.len() >= MAX_FILES {
                return Err(Error::TooLarge);
            }
            let name = path
                .strip_prefix(root)
                .map_err(|_| Error::UnsafePath)?
                .to_str()
                .ok_or(Error::InvalidName)?
                .replace('\\', "/");
            validate_name(&name)?;
            rows.push(Row {
                name,
                bytes: metadata.len(),
            });
        }
    }
    Ok(())
}

/// Existing Tauri output-style wire shape; no UI framework dependency.
#[derive(Serialize, Deserialize)]
pub struct OutputStyle {
    pub name: String,
    pub content: String,
}

pub fn list_output_styles(store: &Store) -> Result<Vec<OutputStyle>, Error> {
    store
        .list(Kind::OutputStyle)?
        .into_iter()
        .filter(|row| !row.name.contains('/'))
        .map(|row| {
            let session = store.open(Kind::OutputStyle, &row.name)?;
            Ok(OutputStyle {
                name: row.name.trim_end_matches(".md").to_owned(),
                content: session.content().to_owned(),
            })
        })
        .collect()
}

/// Existing batch-upsert API: omitted styles are retained, never deleted.
pub fn save_output_styles(store: &Store, styles: &[OutputStyle]) -> Result<(), Error> {
    let sessions = styles
        .iter()
        .map(|style| {
            if style.name.contains('/') {
                return Err(Error::InvalidName);
            }
            store.open(Kind::OutputStyle, &format!("{}.md", style.name))
        })
        .collect::<Result<Vec<_>, _>>()?;
    for (session, style) in sessions.into_iter().zip(styles) {
        session.save(&style.content)?;
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    #[test]
    fn isolated_files_preserve_text_original_revision_and_delete_backup() {
        let root = tempfile::tempdir().unwrap();
        let store = Store::new(
            root.path().join(".claude"),
            root.path().join("backups"),
            root.path().join("locks"),
        );
        for kind in [Kind::Rule, Kind::OutputStyle] {
            assert!(store.list(kind).unwrap().is_empty());
            let missing = store.open(kind, "nested/example.md").unwrap();
            assert!(!missing.exists());
            assert!(!root.path().join(".claude").join(kind.directory()).exists());
            missing
                .save("---\npaths: ['src/**']\nunknown: keep\n---\nPrivate instruction\n")
                .unwrap();
            assert_eq!(missing.save("stale"), Err(Error::Conflict));
            let opened = store.open(kind, "nested/example.md").unwrap();
            assert!(opened.content().contains("unknown: keep"));
            opened.save("updated").unwrap();
            assert_eq!(opened.delete(), Err(Error::Conflict));
            store
                .open(kind, "nested/example.md")
                .unwrap()
                .delete()
                .unwrap();
            assert!(store.list(kind).unwrap().is_empty());
        }
        assert!(
            fs::read_dir(root.path().join("backups"))
                .unwrap()
                .next()
                .is_some()
        );
    }
    #[test]
    fn relative_escape_and_links_are_refused() {
        for name in [
            "../outside.md",
            "a/../outside.md",
            "/outside.md",
            "a\\b.md",
            "a:secret.md",
            "x.txt",
            " .md",
        ] {
            assert_eq!(validate_name(name), Err(Error::InvalidName));
        }
        #[cfg(unix)]
        {
            let root = tempfile::tempdir().unwrap();
            fs::create_dir_all(root.path().join(".claude/rules")).unwrap();
            fs::write(root.path().join("outside.md"), "untouched").unwrap();
            std::os::unix::fs::symlink(
                root.path().join("outside.md"),
                root.path().join(".claude/rules/link.md"),
            )
            .unwrap();
            let store = Store::new(
                root.path().join(".claude"),
                root.path().join("backups"),
                root.path().join("locks"),
            );
            assert!(matches!(
                store.open(Kind::Rule, "link.md"),
                Err(Error::UnsafePath)
            ));
            std::os::unix::fs::symlink(root.path(), root.path().join("redirected-home")).unwrap();
            let redirected = Store::new(
                root.path().join("redirected-home/.claude"),
                root.path().join("backups"),
                root.path().join("locks"),
            );
            assert!(matches!(
                redirected.open(Kind::Rule, "new.md"),
                Err(Error::UnsafePath)
            ));
        }
    }
}
