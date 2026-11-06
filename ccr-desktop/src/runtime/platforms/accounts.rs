//! Captured authentication targets; credentials never enter display projections.
use super::*;
use std::path::PathBuf;

#[derive(Default)]
pub struct Revision {
    files: Vec<(PathBuf, String)>,
    grok: Option<ccr_cli::services::grok_auth_service::GrokAuthRevision>,
}

pub fn capture(
    roots: &PlatformRoots,
    platform: Platform,
    names: &[AccountRow],
) -> Result<Revision, PlatformError> {
    let (store, runtime) = match platform {
        Platform::Claude => (
            roots.ccr.join("platforms/claude"),
            roots.claude.credentials_file.clone(),
        ),
        Platform::Codex => (
            roots.codex_store.clone(),
            roots.codex_runtime.join("auth.json"),
        ),
        Platform::Grok => (
            roots.ccr.join("platforms/grok"),
            roots.grok_runtime.join("auth.json"),
        ),
        _ => return Ok(Revision::default()),
    };
    if platform == Platform::Grok {
        let snapshot = ccr_cli::services::GrokAuthService::from_roots(roots.clone())
            .read_snapshot()
            .map_err(domain_error)?;
        return Ok(Revision {
            files: Vec::new(),
            grok: Some(snapshot.revision),
        });
    }
    let mut paths = vec![runtime, store.join("auth_registry.toml")];
    for account in names {
        if account.name.contains(['/', '\\']) || account.name == ".." {
            return Err(PlatformError::Invalid);
        }
        paths.push(store.join("auth").join(format!("{}.json", account.name)));
    }
    paths
        .into_iter()
        .map(|path| token(&path).map(|token| (path, token)))
        .collect::<Result<Vec<_>, _>>()
        .map(|files| Revision { files, grok: None })
}

fn token(path: &std::path::Path) -> Result<String, PlatformError> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(ccr_core::core::content_version_token(&bytes)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(_) => Err(PlatformError::Storage),
    }
}

impl Revision {
    pub fn unchanged(&self) -> Result<bool, PlatformError> {
        for (path, expected) in &self.files {
            if token(path)? != *expected {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

pub fn apply(
    session: &PlatformSession,
    name: &str,
    action: &AccountAction,
) -> Result<ResultValue, PlatformError> {
    if session.surface != Surface::Auth {
        return Err(PlatformError::Invalid);
    }
    if !session.auth_revision.unchanged()?
        || session.document.read().map_err(settings_error)?.token != session.snapshot.token
    {
        return Ok(ResultValue::Saved(SaveOutcome::Conflict));
    }
    if session.platform == Platform::Grok {
        let service = ccr_cli::services::GrokAuthService::from_roots(session.roots.clone());
        let revision = session
            .auth_revision
            .grok
            .as_ref()
            .ok_or(PlatformError::Invalid)?;
        let warning = match action {
            AccountAction::Off => !service
                .off_checked(revision)
                .map_err(domain_error)?
                .warnings
                .is_empty(),
            AccountAction::Switch => !service
                .switch_account(name, revision)
                .map_err(domain_error)?
                .warnings
                .is_empty(),
            AccountAction::Delete => !service
                .delete_account(name, revision)
                .map_err(domain_error)?
                .warnings
                .is_empty(),
            AccountAction::SaveFrom(scope) => !service
                .save_current(name, scope, revision, false)
                .map_err(domain_error)?
                .warnings
                .is_empty(),
            _ => return Err(PlatformError::Unsupported),
        };
        return Ok(ResultValue::AccountChanged { warning });
    }
    let mut warning = false;
    if matches!(action, AccountAction::Off) {
        let platform = match session.platform {
            Platform::Claude => ccr_config::Platform::Claude,
            Platform::Codex => ccr_config::Platform::Codex,
            // Grok's account preservation has a separate owner; do not bypass it.
            _ => return Err(PlatformError::Unsupported),
        };
        let expected = session
            .auth_revision
            .files
            .first()
            .ok_or(PlatformError::Invalid)?;
        let Some(result) = ccr_cli::application::auth_off::auth_off_captured_versioned(
            platform,
            &session.roots,
            &expected.1,
        )
        .map_err(domain_error)?
        else {
            return Ok(ResultValue::Saved(SaveOutcome::Conflict));
        };
        warning = !result.warnings.is_empty();
    } else {
        match session.platform {
            Platform::Claude => {
                let service = session.roots.claude_auth();
                match action {
                    AccountAction::Save => {
                        service
                            .save_current(name, None, false)
                            .map_err(domain_error)?;
                    }
                    AccountAction::Switch => {
                        let result = service.switch_account(name).map_err(domain_error)?;
                        warning =
                            !result.warnings.is_empty() || !result.remaining_suppressors.is_empty();
                    }
                    AccountAction::Delete => service.delete_account(name).map_err(domain_error)?,
                    _ => return Err(PlatformError::Unsupported),
                }
            }
            Platform::Codex => {
                let service = session.roots.codex_auth();
                match action {
                    AccountAction::Save => service
                        .save_current(name, None, false)
                        .map_err(domain_error)?,
                    AccountAction::Switch => service.switch_account(name).map_err(domain_error)?,
                    AccountAction::Delete => service.delete_account(name).map_err(domain_error)?,
                    AccountAction::Rename(new_name) => {
                        service
                            .rename_account(name, new_name, false)
                            .map_err(domain_error)?;
                    }
                    AccountAction::ApiKey { key, base_url } => {
                        if key.trim().is_empty() {
                            return Err(PlatformError::Invalid);
                        }
                        let auth = ccr_codex::CodexAuthJson {
                            openai_api_key: Some(key.trim().into()),
                            tokens: None,
                            last_refresh: None,
                        };
                        let export = service
                            .build_export_account_from_auth_json(
                                auth,
                                None,
                                None,
                                base_url.clone(),
                                None,
                            )
                            .map_err(domain_error)?;
                        ccr_cli::application::codex_oauth::commit_account(
                            &service,
                            export,
                            Some(name.into()),
                            false,
                        )
                        .map_err(|_| PlatformError::Invalid)?;
                    }
                    AccountAction::Import(content) => {
                        let value: Value =
                            serde_json::from_str(content).map_err(|_| PlatformError::Invalid)?;
                        if value.get("accounts").is_some() {
                            let result = service
                                .import_accounts(content, ccr_codex::ImportMode::Merge, false)
                                .map_err(domain_error)?;
                            warning = result.skipped > 0;
                        } else {
                            let auth: ccr_codex::CodexAuthJson = serde_json::from_value(value)
                                .map_err(|_| PlatformError::Invalid)?;
                            let export = service
                                .build_export_account_from_auth_json(auth, None, None, None, None)
                                .map_err(domain_error)?;
                            ccr_cli::application::codex_oauth::commit_account(
                                &service,
                                export,
                                Some(name.into()),
                                false,
                            )
                            .map_err(|_| PlatformError::Invalid)?;
                        }
                    }
                    AccountAction::Off | AccountAction::SaveFrom(_) => {
                        return Err(PlatformError::Invalid);
                    }
                }
            }
            _ => return Err(PlatformError::Unsupported),
        }
    }
    Ok(ResultValue::AccountChanged { warning })
}
