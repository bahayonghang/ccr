//! Native file transfer keeps the legacy schema and captured database/crypto owner.
use super::*;
use ccr_core::core::guarded_write;
use std::io::Write;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct ImportReceipt {
    pub counts: ImportResult,
    /// Generated basename relative to the captured check-in backups directory.
    pub backup_file: String,
}

pub struct ImportSession {
    path: PathBuf,
    version: String,
    root: PathBuf,
    database_path: Option<PathBuf>,
    database_version: Option<String>,
    pub preview: ImportPreviewResponse,
}
impl ImportSession {
    fn new(
        path: &Path,
        version: String,
        root: &Path,
        database: Option<&Path>,
        database_version: Option<String>,
        mut preview: ImportPreviewResponse,
    ) -> Result<Self> {
        if !preview.version_compatible {
            return Err(ExportError::VersionIncompatible(
                "Unsupported configuration version".into(),
            ));
        }
        // Imported names and IDs are user-controlled. The native panel only needs counts.
        preview.items.clear();
        Ok(Self {
            path: path.to_owned(),
            version,
            root: root.to_owned(),
            database_path: database.map(Path::to_owned),
            database_version,
            preview,
        })
    }

    fn validated_source(
        &self,
        root: &Path,
        existing_accounts: bool,
    ) -> Result<(ExportData, Option<CryptoManager>)> {
        safe_file(&self.path)?;
        let bytes = read_source(&self.path).map_err(|_| ExportError::Conflict)?;
        if guarded_write::content_version_token(&bytes) != self.version {
            return Err(ExportError::Conflict);
        }
        let data = serde_json::from_slice(&bytes)
            .map_err(|_| ExportError::ImportFailed("Invalid configuration document".into()))?;
        ExportManager::validated_native_data(root, &data, existing_accounts)
    }

    pub fn target_missing(&self) -> bool {
        self.database_version.is_none()
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}
fn read_rows(
    conn: &rusqlite::Connection,
) -> rusqlite::Result<(Vec<CheckinProvider>, Vec<CheckinAccount>)> {
    use ccr_db::database::repositories::checkin_repo;
    let tx = conn.unchecked_transaction()?;
    let providers = checkin_repo::get_all_providers(&tx)?;
    let accounts = checkin_repo::get_all_accounts(&tx)?;
    tx.commit()?;
    Ok((providers, accounts))
}

/// Map foreign IDs to the reviewed local provider before importing associated accounts.
fn bind_providers(data: &mut ExportData, existing: &[CheckinProvider]) -> Result<()> {
    use std::collections::{HashMap, HashSet};
    let mut aliases = HashMap::new();
    let mut targets = HashSet::new();
    let mut names = HashSet::new();
    for provider in &mut data.providers {
        let mut matching = existing
            .iter()
            .filter(|p| p.id == provider.id || p.name == provider.name);
        let target = matching
            .next()
            .map(|p| p.id.clone())
            .unwrap_or_else(|| provider.id.clone());
        if matching.next().is_some()
            || !targets.insert(target.clone())
            || !names.insert(provider.name.clone())
            || aliases
                .insert(provider.id.clone(), target.clone())
                .is_some()
        {
            return Err(ExportError::ImportFailed(
                "Ambiguous provider identities in configuration".into(),
            ));
        }
        provider.id = target;
    }
    let mut accounts = HashSet::new();
    for account in &mut data.accounts {
        if !accounts.insert(account.id.clone()) {
            return Err(ExportError::ImportFailed(
                "Duplicate account identity in configuration".into(),
            ));
        }
        if let Some(target) = aliases.get(&account.provider_id) {
            account.provider_id.clone_from(target);
        } else if !existing.iter().any(|p| p.id == account.provider_id) {
            return Err(ExportError::ImportFailed(
                "Account provider is missing from configuration".into(),
            ));
        }
    }
    Ok(())
}

fn publish_initial_schema(database: &Path) -> Result<()> {
    let parent = database.parent().ok_or(ExportError::Conflict)?;
    std::fs::create_dir_all(parent).map_err(|e| ExportError::ImportFailed(e.to_string()))?;
    // Publish a readable first schema atomically. A failed later pool initialization
    // can be reviewed and retried without deleting another client's database.
    let staged = tempfile::NamedTempFile::new_in(parent)
        .map_err(|e| ExportError::ImportFailed(e.to_string()))?
        .into_temp_path();
    {
        let conn = rusqlite::Connection::open(&staged)
            .map_err(|e| ExportError::ImportFailed(e.to_string()))?;
        database::migrations::run_initial_migration(&conn)
            .map_err(|e| ExportError::ImportFailed(e.to_string()))?;
    }
    staged.persist_noclobber(database).map_err(|e| {
        if e.error.kind() == std::io::ErrorKind::AlreadyExists {
            ExportError::Conflict
        } else {
            ExportError::ImportFailed(e.to_string())
        }
    })?;
    Ok(())
}

fn row_version(providers: &[CheckinProvider], accounts: &[CheckinAccount]) -> Result<String> {
    let value = serde_json::to_vec(&(providers, accounts))
        .map_err(|e| ExportError::ImportFailed(e.to_string()))?;
    Ok(guarded_write::content_version_token(&value))
}

fn safe_file(path: &Path) -> Result<()> {
    if !path.is_absolute() {
        return Err(ExportError::ImportFailed(
            "An absolute file path is required".into(),
        ));
    }
    if path
        .components()
        .any(|part| part == std::path::Component::ParentDir)
    {
        return Err(ExportError::ImportFailed(
            "Parent-directory components are not supported".into(),
        ));
    }
    for component in path.ancestors() {
        if let Ok(meta) = std::fs::symlink_metadata(component) {
            let link = meta.file_type().is_symlink();
            #[cfg(windows)]
            let link = {
                use std::os::windows::fs::MetadataExt;
                link || meta.file_attributes() & 0x400 != 0
            };
            if link {
                return Err(ExportError::ImportFailed(
                    "Linked paths are not supported".into(),
                ));
            }
        }
    }
    Ok(())
}
fn read_source(path: &Path) -> Result<Vec<u8>> {
    use std::io::Read;
    let file = std::fs::File::open(path).map_err(|e| ExportError::ImportFailed(e.to_string()))?;
    if !file
        .metadata()
        .map_err(|e| ExportError::ImportFailed(e.to_string()))?
        .is_file()
    {
        return Err(ExportError::ImportFailed(
            "A regular configuration file is required".into(),
        ));
    }
    let mut bytes = Vec::new();
    file.take(10 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| ExportError::ImportFailed(e.to_string()))?;
    if bytes.len() > 10 * 1024 * 1024 {
        return Err(ExportError::ImportFailed("Import file is too large".into()));
    }
    Ok(bytes)
}
fn reject_unknown(raw: &serde_json::Value, known: &serde_json::Value) -> Result<()> {
    match (raw, known) {
        (serde_json::Value::Object(raw), serde_json::Value::Object(known)) => {
            for (key, value) in raw {
                if let Some(other) = known.get(key) {
                    reject_unknown(value, other)?;
                } else if !(value.is_null()
                    && known.contains_key("base_url")
                    && matches!(key.as_str(), "builtin_id" | "updated_at"))
                {
                    return Err(ExportError::ImportFailed(
                        "Unknown fields cannot be preserved by this configuration format".into(),
                    ));
                }
            }
        }
        (serde_json::Value::Array(raw), serde_json::Value::Array(known)) => {
            for (value, other) in raw.iter().zip(known) {
                reject_unknown(value, other)?;
            }
        }
        _ => {}
    }
    Ok(())
}
impl ExportManager {
    // Native admission validates every credential before the first database mutation.
    // Pass plaintext Secret values to legacy import only after validation, so its
    // historical foreign-key fallback can never replace a valid account with {}.
    fn validated_native_data(
        root: &Path,
        source: &ExportData,
        existing_accounts: bool,
    ) -> Result<(ExportData, Option<CryptoManager>)> {
        let crypto = if existing_accounts {
            Some(
                CryptoManager::load_existing(root)
                    .map_err(|_| ExportError::ExistingKeyUnavailable)?,
            )
        } else if source
            .accounts
            .iter()
            .any(|account| account.cookies_json_encrypted)
        {
            Some(CryptoManager::load_existing(root).map_err(|_| {
                ExportError::CryptoError(
                    "Encrypted configuration requires the matching existing crypto key".into(),
                )
            })?)
        } else {
            None
        };
        let mut data = source.clone();
        for account in &mut data.accounts {
            if account.cookies_json_encrypted {
                account.cookies_json = crypto
                    .as_ref()
                    .ok_or_else(|| ExportError::CryptoError("Matching key required".into()))?
                    .decrypt(account.cookies_json.expose())
                    .map_err(|_| {
                        ExportError::CryptoError(
                            "Configuration credentials cannot be decrypted with this device's key"
                                .into(),
                        )
                    })?;
                account.cookies_json_encrypted = false;
            }
            serde_json::from_str::<std::collections::HashMap<String, String>>(
                account.cookies_json.expose(),
            )
            .map_err(|_| {
                ExportError::ImportFailed("Each account requires a valid cookie JSON object".into())
            })?;
        }
        Ok((data, crypto))
    }

    fn fingerprint(&self) -> Result<String> {
        let (providers, accounts) = self
            .db
            .with_connection(read_rows)
            .map_err(|e| ExportError::ImportFailed(e.to_string()))?;
        row_version(&providers, &accounts)
    }
    pub fn export_file(&self, path: &Path) -> Result<()> {
        let (providers, accounts) = self
            .db
            .with_connection(read_rows)
            .map_err(|e| ExportError::ExportFailed(e.to_string()))?;
        let crypto = if accounts.is_empty() {
            CryptoManager::new(&self.checkin_dir)
                .map_err(|e| ExportError::CryptoError(e.to_string()))?
        } else {
            CryptoManager::load_existing(&self.checkin_dir)
                .map_err(|_| ExportError::ExistingKeyUnavailable)?
        };
        let data = ExportData::new(
            providers,
            Self::encode_accounts(&accounts, &ExportOptions::default(), &crypto)?,
        );
        let bytes = serde_json::to_vec_pretty(&data)
            .map_err(|e| ExportError::ExportFailed(e.to_string()))?;
        self.write_new_export(path, &bytes)
    }
    pub fn export_stats_file(&self, path: &Path) -> Result<()> {
        let stats = crate::CheckinService::with_db(self.checkin_dir.clone(), self.db.clone())
            .get_today_stats()
            .map_err(|e| ExportError::ExportFailed(e.to_string()))?;
        let bytes = serde_json::to_vec_pretty(&stats)
            .map_err(|e| ExportError::ExportFailed(e.to_string()))?;
        self.write_new_export(path, &bytes)
    }
    fn write_new_export(&self, path: &Path, bytes: &[u8]) -> Result<()> {
        safe_file(path)?;
        let parent = path
            .parent()
            .ok_or_else(|| ExportError::ExportFailed("Select a new file".into()))?;
        let mut staged = tempfile::NamedTempFile::new_in(parent)
            .map_err(|e| ExportError::ExportFailed(e.to_string()))?;
        guarded_write::restrict_owner_only(staged.path())
            .map_err(|e| ExportError::ExportFailed(e.to_string()))?;
        staged
            .write_all(bytes)
            .and_then(|_| staged.as_file().sync_all())
            .map_err(|e| ExportError::ExportFailed(e.to_string()))?;
        staged.persist_noclobber(path).map_err(|e| {
            if e.error.kind() == std::io::ErrorKind::AlreadyExists {
                ExportError::DestinationExists
            } else {
                ExportError::ExportFailed(e.error.to_string())
            }
        })?;
        Ok(())
    }
    fn read_import(path: &Path, root: &Path) -> Result<(ExportData, String)> {
        safe_file(path)?;
        let bytes = read_source(path)?;
        let raw: serde_json::Value = serde_json::from_slice(&bytes)
            .map_err(|_| ExportError::ImportFailed("Invalid configuration document".into()))?;
        let data: ExportData = serde_json::from_value(raw.clone())
            .map_err(|_| ExportError::ImportFailed("Invalid configuration fields".into()))?;
        let known =
            serde_json::to_value(&data).map_err(|e| ExportError::ImportFailed(e.to_string()))?;
        reject_unknown(&raw, &known)?;
        Self::validated_native_data(root, &data, false)?;
        Ok((data, guarded_write::content_version_token(&bytes)))
    }

    /// Preview against the captured destination without acquiring a write pool or migrating.
    pub fn preview_file_at(path: &Path, database: &Path, root: &Path) -> Result<ImportSession> {
        safe_file(database)?;
        safe_file(root)?;
        let (mut data, version) = Self::read_import(path, root)?;
        let (providers, accounts, database_version) = match std::fs::metadata(database) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => (vec![], vec![], None),
            Ok(meta) if meta.is_file() => {
                let conn = rusqlite::Connection::open_with_flags(
                    database,
                    rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
                )
                .map_err(|e| ExportError::ImportFailed(e.to_string()))?;
                let (providers, accounts) =
                    read_rows(&conn).map_err(|e| ExportError::ImportFailed(e.to_string()))?;
                let version = row_version(&providers, &accounts)?;
                (providers, accounts, Some(version))
            }
            _ => {
                return Err(ExportError::ImportFailed(
                    "Cannot read target database".into(),
                ));
            }
        };
        bind_providers(&mut data, &providers)?;
        let preview = Self::preview_against(&data, &providers, &accounts);
        ImportSession::new(
            path,
            version,
            root,
            Some(database),
            database_version,
            preview,
        )
    }

    pub fn preview_file(&self, path: &Path) -> Result<ImportSession> {
        let (mut data, version) = Self::read_import(path, &self.checkin_dir)?;
        let (providers, accounts) = self
            .db
            .with_connection(read_rows)
            .map_err(|e| ExportError::ImportFailed(e.to_string()))?;
        bind_providers(&mut data, &providers)?;
        let preview = Self::preview_against(&data, &providers, &accounts);
        ImportSession::new(
            path,
            version,
            &self.checkin_dir,
            None,
            Some(row_version(&providers, &accounts)?),
            preview,
        )
    }
    /// Admit the exact reviewed source and target before letting the caller initialize storage.
    pub fn apply_file_at(
        session: &ImportSession,
        database: &Path,
        root: &Path,
        overwrite: bool,
        open: impl FnOnce() -> Result<DbAccess>,
    ) -> Result<ImportReceipt> {
        if session.database_path.as_deref() != Some(database) || session.root != root {
            return Err(ExportError::Conflict);
        }
        safe_file(database)?;
        safe_file(root)?;
        let missing = !database
            .try_exists()
            .map_err(|e| ExportError::ImportFailed(e.to_string()))?;
        if missing != session.target_missing() {
            return Err(ExportError::Conflict);
        }
        let (expected, existing_accounts) = if missing {
            session.validated_source(root, false)?;
            publish_initial_schema(database)?;
            (row_version(&[], &[])?, false)
        } else {
            let conn = rusqlite::Connection::open_with_flags(
                database,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
            )
            .map_err(|e| ExportError::ImportFailed(e.to_string()))?;
            let (providers, accounts) =
                read_rows(&conn).map_err(|e| ExportError::ImportFailed(e.to_string()))?;
            let revision = row_version(&providers, &accounts)?;
            if !accounts.is_empty() {
                CryptoManager::load_existing(root)
                    .map_err(|_| ExportError::ExistingKeyUnavailable)?;
            }
            if Some(&revision) != session.database_version.as_ref() {
                return Err(ExportError::Conflict);
            }
            session.validated_source(root, !accounts.is_empty())?;
            (revision, !accounts.is_empty())
        };
        let manager = Self::with_db(root, open()?);
        let opened = manager
            .db
            .with_connection(|conn| Ok(conn.path().map(PathBuf::from)))
            .map_err(|e| ExportError::ImportFailed(e.to_string()))?;
        if opened.and_then(|p| p.canonicalize().ok())
            != Some(database.canonicalize().map_err(|_| ExportError::Conflict)?)
            || manager.fingerprint()? != expected
        {
            return Err(ExportError::Conflict);
        }
        let (validated, key) = session.validated_source(root, existing_accounts)?;
        manager.apply_validated(validated, key.as_ref(), overwrite)
    }

    pub fn import_preview(
        &self,
        session: &Arc<ImportSession>,
        overwrite: bool,
    ) -> Result<ImportResult> {
        if session.database_path.is_some()
            || session.root != self.checkin_dir
            || Some(self.fingerprint()?) != session.database_version
        {
            return Err(ExportError::Conflict);
        }
        let has_accounts = !AccountManager::with_db(&self.checkin_dir, self.db.clone())
            .load_all()
            .map_err(|e| ExportError::AccountError(e.to_string()))?
            .is_empty();
        let (validated, key) = session.validated_source(&self.checkin_dir, has_accounts)?;
        Ok(self
            .apply_validated(validated, key.as_ref(), overwrite)?
            .counts)
    }

    fn apply_validated(
        &self,
        mut validated: ExportData,
        key: Option<&CryptoManager>,
        overwrite: bool,
    ) -> Result<ImportReceipt> {
        let providers = ProviderManager::with_db(self.db.clone())
            .load_all()
            .map_err(|e| ExportError::ProviderError(e.to_string()))?;
        bind_providers(&mut validated, &providers)?;
        let backups = self.checkin_dir.join("backups");
        safe_file(&backups)?;
        std::fs::create_dir_all(&backups).map_err(|e| ExportError::ImportFailed(e.to_string()))?;
        let backup_file = format!("pre-import-{}.db", uuid::Uuid::new_v4());
        let backup = backups.join(&backup_file);
        self.db
            .with_connection(|conn| {
                Ok(ccr_db::database::backup::create_verified_backup(
                    conn, &backup,
                ))
            })
            .map_err(|e| ExportError::ImportFailed(e.to_string()))?
            .map_err(|e| ExportError::ImportFailed(e.to_string()))?;
        let mut counts = ImportResult::success("导入完成");
        let result = self.import_counted(
            validated,
            &ImportOptions {
                conflict_strategy: if overwrite {
                    ImportConflictStrategy::Overwrite
                } else {
                    ImportConflictStrategy::Skip
                },
                ..Default::default()
            },
            &mut counts,
            key,
        );
        let mut receipt = ImportReceipt {
            counts,
            backup_file,
        };
        if result.is_err() {
            receipt.counts.success = false;
            receipt.counts.message = "Import stopped".into();
            return Err(ExportError::PartialImport(Box::new(receipt)));
        }
        Ok(receipt)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[path = "export_transfer/tests.rs"]
mod tests;
