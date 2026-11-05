//! Explicit-path settings documents shared by desktop adapters.
//!
//! Snapshots may contain credentials and deliberately have no `Debug` implementation.
//! The existing guarded leaf writer owns locking, backups and private replacement.

pub mod codex;
pub mod json;

use std::path::{Path, PathBuf};

use ccr_core::core::{
    BackupPolicy, VersionedWriteOutcome, WriteOptions, content_version_token, lock::LockManager,
    write_guarded_versioned,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Json,
    Toml,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Io,
    Utf8,
    Invalid {
        line: Option<usize>,
        column: Option<usize>,
    },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Io => "settings_io_failed",
            Self::Utf8 => "settings_invalid_utf8",
            Self::Invalid { .. } => "settings_invalid_document",
        })
    }
}

impl std::error::Error for Error {}

#[derive(Clone)]
pub struct Snapshot {
    pub content: String,
    pub token: String,
    pub exists: bool,
    pub source: PathBuf,
    primary_token: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SaveOutcome {
    Saved { token: String },
    Conflict,
}

#[derive(Clone)]
pub struct Document {
    path: PathBuf,
    fallback: Option<PathBuf>,
    backup: BackupPolicy,
    lock_directory: Option<PathBuf>,
    format: Format,
}

impl Document {
    /// Construction only records paths. It performs no I/O or directory creation.
    pub fn new(
        path: PathBuf,
        fallback: Option<PathBuf>,
        backup: BackupPolicy,
        lock_directory: Option<PathBuf>,
        format: Format,
    ) -> Self {
        Self {
            path,
            fallback,
            backup,
            lock_directory,
            format,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn read(&self) -> Result<Snapshot, Error> {
        let primary = read_text(&self.path)?;
        if primary.exists {
            return Ok(primary);
        }
        if let Some(fallback) = &self.fallback {
            let mut source = read_text(fallback)?;
            source.primary_token = primary.token;
            return Ok(source);
        }
        Ok(primary)
    }

    /// Raw editors keep the original source token, including a missing file's empty token.
    pub fn save_text(&self, content: &str, expected: &Snapshot) -> Result<SaveOutcome, Error> {
        validate(self.format, content)?;
        self.save_validated(content, expected)
    }

    /// The platform owner performs schema validation before calling this method.
    pub fn save_validated(&self, content: &str, expected: &Snapshot) -> Result<SaveOutcome, Error> {
        let current = self.read()?;
        if current.source != expected.source
            || current.token != expected.token
            || current.primary_token != expected.primary_token
        {
            return Ok(SaveOutcome::Conflict);
        }
        write_versioned(
            &self.path,
            content,
            &expected.primary_token,
            self.backup.clone(),
            true,
            self.lock_directory.as_deref(),
        )
    }
}

pub fn write_versioned(
    path: &Path,
    content: &str,
    expected_token: &str,
    backup: BackupPolicy,
    secret: bool,
    lock_directory: Option<&Path>,
) -> Result<SaveOutcome, Error> {
    let options = WriteOptions {
        backup,
        secret,
        ..Default::default()
    };
    let write = || write_guarded_versioned(path, content.as_bytes(), expected_token, &options);
    let outcome = match lock_directory {
        Some(directory) => LockManager::with_scoped_directory(directory, write),
        None => write(),
    }
    .map_err(|_| Error::Io)?;
    Ok(match outcome {
        VersionedWriteOutcome::Written => SaveOutcome::Saved {
            token: content_version_token(content.as_bytes()),
        },
        VersionedWriteOutcome::Conflict => SaveOutcome::Conflict,
    })
}

pub fn read_text(path: &Path) -> Result<Snapshot, Error> {
    match std::fs::read(path) {
        Ok(bytes) => {
            let token = content_version_token(&bytes);
            Ok(Snapshot {
                content: String::from_utf8(bytes).map_err(|_| Error::Utf8)?,
                primary_token: token.clone(),
                token,
                exists: true,
                source: path.to_path_buf(),
            })
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Snapshot {
            content: String::new(),
            token: String::new(),
            primary_token: String::new(),
            exists: false,
            source: path.to_path_buf(),
        }),
        Err(_) => Err(Error::Io),
    }
}

pub fn validate(format: Format, content: &str) -> Result<(), Error> {
    match format {
        Format::Json => {
            let value: serde_json::Value =
                serde_json::from_str(content).map_err(|error| Error::Invalid {
                    line: Some(error.line()),
                    column: Some(error.column()),
                })?;
            if !value.is_object() {
                return Err(Error::Invalid {
                    line: None,
                    column: None,
                });
            }
        }
        Format::Toml => {
            toml::from_str::<toml::Value>(content).map_err(|error| {
                let position = error.span().and_then(|span| {
                    let prefix = content.get(..span.start.min(content.len()))?;
                    Some((
                        prefix.bytes().filter(|byte| *byte == b'\n').count() + 1,
                        prefix
                            .rsplit_once('\n')
                            .map_or(prefix.chars().count() + 1, |(_, tail)| {
                                tail.chars().count() + 1
                            }),
                    ))
                });
                Error::Invalid {
                    line: position.map(|value| value.0),
                    column: position.map(|value| value.1),
                }
            })?;
        }
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::fs;

    fn document(root: &Path, fallback: Option<PathBuf>) -> Document {
        Document::new(
            root.join("settings.json"),
            fallback,
            BackupPolicy::Dir {
                dir: root.join("backups"),
                prefix: "settings".into(),
            },
            Some(root.join("locks")),
            Format::Json,
        )
    }

    #[test]
    fn pure_missing_read_and_invalid_source_leave_no_files_or_secret_diagnostic() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("absent");
        let doc = document(&root, None);
        let before = doc.read().unwrap();
        assert!(!before.exists);
        assert!(!root.exists());
        let error = doc
            .save_text("{\"credential\":\"secret-sentinel\",", &before)
            .unwrap_err();
        assert!(!format!("{error:?} {error}").contains("sentinel"));
        assert!(!root.exists());
    }

    #[test]
    fn two_original_revision_clients_have_one_winner_and_no_conflict_backup() {
        let temp = tempfile::tempdir().unwrap();
        let doc = document(temp.path(), None);
        fs::write(doc.path(), r#"{"unknown":{"kept":true},"model":"old"}"#).unwrap();
        let a = doc.read().unwrap();
        let b = doc.read().unwrap();
        assert!(matches!(
            doc.save_text(r#"{"unknown":{"kept":true},"model":"a"}"#, &a),
            Ok(SaveOutcome::Saved { .. })
        ));
        let backups = fs::read_dir(temp.path().join("backups")).unwrap().count();
        assert_eq!(
            doc.save_text(r#"{"model":"b"}"#, &b).unwrap(),
            SaveOutcome::Conflict
        );
        assert_eq!(
            fs::read_dir(temp.path().join("backups")).unwrap().count(),
            backups
        );
        let saved: serde_json::Value = serde_json::from_str(&doc.read().unwrap().content).unwrap();
        assert_eq!(saved["model"], "a");
        assert_eq!(saved["unknown"]["kept"], true);
    }

    #[test]
    fn existing_empty_file_is_distinct_from_absence() {
        let temp = tempfile::tempdir().unwrap();
        let doc = document(temp.path(), None);
        let absent = doc.read().unwrap();
        fs::write(doc.path(), "").unwrap();
        assert_eq!(doc.save_text("{}", &absent).unwrap(), SaveOutcome::Conflict);
        let empty = doc.read().unwrap();
        assert!(empty.exists && !empty.token.is_empty());
        assert!(matches!(
            doc.save_text("{}", &empty),
            Ok(SaveOutcome::Saved { .. })
        ));
    }

    #[test]
    fn fallback_change_and_new_primary_conflict_without_overwrite() {
        let temp = tempfile::tempdir().unwrap();
        let fallback = temp.path().join("legacy.json");
        fs::write(&fallback, r#"{"legacy":true}"#).unwrap();
        let doc = document(temp.path(), Some(fallback.clone()));
        let initial = doc.read().unwrap();
        assert_eq!(initial.source, fallback);
        fs::write(&fallback, r#"{"legacy":"external"}"#).unwrap();
        assert_eq!(
            doc.save_text("{}", &initial).unwrap(),
            SaveOutcome::Conflict
        );
        assert!(!doc.path().exists());
        let fresh = doc.read().unwrap();
        fs::write(doc.path(), r#"{"primary":"external"}"#).unwrap();
        assert_eq!(doc.save_text("{}", &fresh).unwrap(), SaveOutcome::Conflict);
        assert_eq!(
            fs::read_to_string(doc.path()).unwrap(),
            r#"{"primary":"external"}"#
        );
    }
}
