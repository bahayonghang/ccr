//! Native file transfer keeps the legacy schema and captured database/crypto owner.
use super::*;
use ccr_core::core::LockManager;
use ccr_core::core::guarded_write::{self, BackupPolicy, WriteOptions};
use std::sync::Arc;

pub struct ImportSession {
    path: PathBuf,
    version: String,
    root: PathBuf,
    database_version: String,
    pub preview: ImportPreviewResponse,
}
impl ImportSession {
    pub fn path(&self) -> &Path {
        &self.path
    }
}
fn safe_file(path: &Path) -> Result<()> {
    if !path.is_absolute() {
        return Err(ExportError::ImportFailed(
            "An absolute file path is required".into(),
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
    fn validated_native_data(&self, source: &ExportData) -> Result<ExportData> {
        let crypto = if source
            .accounts
            .iter()
            .any(|account| account.cookies_json_encrypted)
        {
            Some(
                CryptoManager::load_existing(&self.checkin_dir).map_err(|_| {
                    ExportError::CryptoError(
                        "Encrypted configuration requires the matching existing crypto key".into(),
                    )
                })?,
            )
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
        Ok(data)
    }

    fn fingerprint(&self) -> Result<String> {
        let providers = ProviderManager::with_db(self.db.clone())
            .load_all()
            .map_err(|e| ExportError::ProviderError(e.to_string()))?;
        let accounts = AccountManager::with_db(&self.checkin_dir, self.db.clone())
            .load_all()
            .map_err(|e| ExportError::AccountError(e.to_string()))?;
        let value = serde_json::to_vec(&(providers, accounts))
            .map_err(|e| ExportError::ExportFailed(e.to_string()))?;
        Ok(guarded_write::content_version_token(&value))
    }
    pub fn export_file(&self, path: &Path) -> Result<()> {
        let data = self.export(&ExportOptions::default())?;
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
        if path.exists() {
            return Err(ExportError::ExportFailed(
                "Choose a new file to avoid replacing existing data".into(),
            ));
        }
        let result = LockManager::with_scoped_directory(&self.checkin_dir.join("locks"), || {
            guarded_write::write_guarded_versioned(
                path,
                bytes,
                "",
                &WriteOptions {
                    secret: true,
                    backup: BackupPolicy::None,
                    ..Default::default()
                },
            )
        })
        .map_err(|e| ExportError::ExportFailed(e.to_string()))?;
        if result == guarded_write::VersionedWriteOutcome::Conflict {
            return Err(ExportError::ExportFailed(
                "Export destination changed".into(),
            ));
        }
        Ok(())
    }
    pub fn preview_file(&self, path: &Path) -> Result<ImportSession> {
        safe_file(path)?;
        let bytes = read_source(path)?;
        let raw: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|e| ExportError::ImportFailed(e.to_string()))?;
        let data: ExportData = serde_json::from_value(raw.clone())
            .map_err(|e| ExportError::ImportFailed(e.to_string()))?;
        let known =
            serde_json::to_value(&data).map_err(|e| ExportError::ImportFailed(e.to_string()))?;
        reject_unknown(&raw, &known)?;
        self.validated_native_data(&data)?;
        let preview = self.preview_import(&data)?;
        if !preview.version_compatible {
            return Err(ExportError::VersionIncompatible(data.version));
        }
        Ok(ImportSession {
            path: path.to_owned(),
            version: guarded_write::content_version_token(&bytes),
            root: self.checkin_dir.clone(),
            database_version: self.fingerprint()?,
            preview,
        })
    }
    pub fn import_preview(
        &self,
        session: &Arc<ImportSession>,
        overwrite: bool,
    ) -> Result<ImportResult> {
        safe_file(&session.path)?;
        let current = read_source(&session.path)?;
        if session.root != self.checkin_dir
            || guarded_write::content_version_token(&current) != session.version
            || self.fingerprint()? != session.database_version
        {
            return Err(ExportError::Conflict);
        }
        // Retain only preview metadata between actions. Parse this exact hash-checked
        // source at commit time so plaintext cookies never live in an app-owned result.
        let data: ExportData = serde_json::from_slice(&current)
            .map_err(|e| ExportError::ImportFailed(e.to_string()))?;
        let validated = self.validated_native_data(&data)?;
        let backups = self.checkin_dir.join("backups");
        safe_file(&backups)?;
        std::fs::create_dir_all(&backups).map_err(|e| ExportError::ImportFailed(e.to_string()))?;
        let backup = backups.join(format!("pre-import-{}.db", uuid::Uuid::new_v4()));
        self.db
            .with_connection(|conn| {
                Ok(ccr_db::database::backup::create_verified_backup(
                    conn, &backup,
                ))
            })
            .map_err(|e| ExportError::ImportFailed(e.to_string()))?
            .map_err(|e| ExportError::ImportFailed(e.to_string()))?;
        self.import(
            validated,
            &ImportOptions {
                conflict_strategy: if overwrite {
                    ImportConflictStrategy::Overwrite
                } else {
                    ImportConflictStrategy::Skip
                },
                ..Default::default()
            },
        )
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use ccr_core::Secret;
    use ccr_db::{
        database::{self, DbAccess},
        models::checkin::{CreateAccountRequest, CreateProviderRequest, UpdateAccountRequest},
    };
    #[test]
    fn transfer_uses_scoped_database_preserves_extensions_and_refuses_stale_preview() {
        let root = tempfile::tempdir().unwrap();
        let crypto = root.path().join("crypto");
        let pool = database::create_pool(&root.path().join("source.db"), None).unwrap();
        {
            let conn = pool.get().unwrap();
            conn.execute_batch(database::schema::CREATE_TABLES_SQL)
                .unwrap();
        }
        let db = DbAccess::Pool(pool.clone());
        let provider = ProviderManager::with_db(db.clone())
            .create(CreateProviderRequest {
                name: "Fixture".into(),
                base_url: "https://fixture.invalid".into(),
                checkin_path: None,
                balance_path: None,
                user_info_path: None,
                auth_header: None,
                auth_prefix: None,
                builtin_id: None,
            })
            .unwrap();
        let accounts = AccountManager::with_db(&crypto, db.clone());
        let account = accounts
            .create(CreateAccountRequest {
                provider_id: provider.id,
                name: "Fixture account".into(),
                cookies_json: Secret::new(r#"{"session":"fixture-sensitive"}"#),
                api_user: "7".into(),
                extra_config: r#"{"future":"keep"}"#.into(),
            })
            .unwrap();
        let manager = ExportManager::with_db(&crypto, db);
        let export = root.path().join("config.json");
        manager.export_file(&export).unwrap();
        assert!(
            !std::fs::read_to_string(&export)
                .unwrap()
                .contains("fixture-sensitive")
        );
        assert!(manager.export_file(&export).is_err());
        let preview = Arc::new(manager.preview_file(&export).unwrap());
        accounts
            .update(
                &account.id,
                UpdateAccountRequest {
                    name: Some("external winner".into()),
                    cookies_json: None,
                    api_user: None,
                    enabled: None,
                    extra_config: None,
                },
            )
            .unwrap();
        assert!(manager.import_preview(&preview, true).is_err());
        let preview = Arc::new(manager.preview_file(&export).unwrap());
        let result = manager.import_preview(&preview, true).unwrap();
        assert_eq!(result.accounts_imported, 1);
        assert_eq!(
            accounts.get(&account.id).unwrap().extra_config,
            r#"{"future":"keep"}"#
        );
        let backups: Vec<_> = std::fs::read_dir(crypto.join("backups")).unwrap().collect();
        assert_eq!(backups.len(), 1);
        let mut raw: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&export).unwrap()).unwrap();
        raw["unknown"] = serde_json::Value::Null;
        std::fs::write(&export, serde_json::to_vec(&raw).unwrap()).unwrap();
        assert!(manager.preview_file(&export).is_err());
        // A source readable at preview must still be decryptable before commit.
        let fresh = root.path().join("fresh.json");
        manager.export_file(&fresh).unwrap();
        let preview = Arc::new(manager.preview_file(&fresh).unwrap());
        let original_account = accounts.get(&account.id).unwrap();
        let foreign = root.path().join("foreign-crypto");
        CryptoManager::new(&foreign).unwrap();
        std::fs::copy(foreign.join("crypto.key"), crypto.join("crypto.key")).unwrap();
        assert!(manager.import_preview(&preview, true).is_err());
        assert!(manager.preview_file(&fresh).is_err());
        assert_eq!(
            accounts.get(&account.id).unwrap().cookies_json_encrypted,
            original_account.cookies_json_encrypted
        );
        assert_eq!(
            std::fs::read_dir(crypto.join("backups")).unwrap().count(),
            1
        );
    }
}
