//! Credential-bearing internals; only explicit metadata DTOs cross this boundary.
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests;
use super::GrokAuthService;
use crate::application::AuthOffResult;
use ccr_core::Secret;
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::guarded_write::{
    VersionedWriteOutcome, WriteOptions, content_version_token, write_guarded_versioned,
};
use ccr_core::core::lock::FileLock;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[cfg(test)]
thread_local! {
    static STORE_DURABILITY_WARNING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static BEFORE_STORE_WRITE: std::cell::Cell<Option<fn(&Paths)>> = const { std::cell::Cell::new(None) };
    static WRITE_ERROR_AFTER_WRITE: std::cell::RefCell<Option<PathBuf>> = const { std::cell::RefCell::new(None) };
}

#[derive(Clone, Default, PartialEq, Eq)]
pub struct GrokAuthRevision {
    store: String,
    runtime: String,
}
impl std::fmt::Debug for GrokAuthRevision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("GrokAuthRevision([redacted])")
    }
}
#[derive(Debug, Clone, Default)]
pub struct GrokAuthAccount {
    pub name: String,
    pub scope: String,
    pub email: Option<String>,
    pub team: Option<String>,
    pub saved_at: String,
    pub expires_at: Option<String>,
    pub expired: bool,
    pub local_match: bool,
}
#[derive(Debug, Clone, Default)]
pub struct GrokAuthSource {
    pub scope: String,
    pub email: Option<String>,
    pub team: Option<String>,
    pub matched_account: Option<String>,
}
#[derive(Debug, Clone, Default)]
pub struct GrokAuthSnapshot {
    pub accounts: Vec<GrokAuthAccount>,
    pub sources: Vec<GrokAuthSource>,
    pub runtime_present: bool,
    pub runtime_error: Option<String>,
    pub revision: GrokAuthRevision,
}
#[derive(Debug, Clone, Default)]
pub struct GrokAuthMutation {
    pub warnings: Vec<String>,
    pub outgoing_saved: bool,
}
#[derive(Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Store {
    accounts: BTreeMap<String, Saved>,
}
#[derive(Clone, Serialize, Deserialize)]
struct Saved {
    saved_at: String,
    scope: String,
    #[serde(serialize_with = "ccr_core::expose_plaintext")]
    credential: Secret,
}
struct State {
    store: Store,
    runtime: Option<Vec<u8>>,
    revision: GrokAuthRevision,
}
struct Paths {
    store: PathBuf,
    runtime: PathBuf,
    operation: PathBuf,
}
fn error(message: &str) -> CcrError {
    CcrError::ValidationError(message.into())
}
fn read(path: &Path) -> Result<Option<Vec<u8>>> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(_) => Err(error(
            "Cannot read Grok authentication data; check file permissions",
        )),
    }
}
fn token(bytes: Option<&[u8]>) -> String {
    bytes.map(content_version_token).unwrap_or_default()
}
fn document(bytes: Option<&[u8]>) -> Result<Map<String, Value>> {
    match bytes {
        None => Ok(Map::new()),
        Some(bytes) => serde_json::from_slice::<Map<String, Value>>(bytes)
            .map_err(|_| error("Grok runtime JSON is damaged; refresh or explicitly log out")),
    }
}
fn string<'a>(v: &'a Value, field: &str) -> Option<&'a str> {
    v.get(field).and_then(Value::as_str)
}
fn credential(scope: &str, v: &Value) -> bool {
    let issuer = string(v, "oidc_issuer")
        .unwrap_or_default()
        .trim_end_matches('/');
    let client = string(v, "oidc_client_id").unwrap_or_default();
    v.is_object()
        && string(v, "auth_mode") == Some("oidc")
        && issuer == "https://auth.x.ai"
        && !client.is_empty()
        && scope == format!("{issuer}::{client}")
        && string(v, "key").is_some_and(|s| !s.is_empty())
        && string(v, "create_time").is_some_and(|s| chrono::DateTime::parse_from_rfc3339(s).is_ok())
        && v.get("expires_at").is_none_or(|x| {
            x.is_null()
                || x.as_str()
                    .is_some_and(|s| chrono::DateTime::parse_from_rfc3339(s).is_ok())
        })
        && v.get("user_id").is_none_or(Value::is_string)
        && [
            "email",
            "first_name",
            "last_name",
            "profile_image_asset_id",
            "principal_type",
            "principal_id",
            "team_id",
            "team_name",
            "team_role",
            "organization_id",
            "organization_name",
            "organization_role",
            "user_blocked_reason",
            "refresh_token",
        ]
        .iter()
        .all(|key| v.get(*key).is_none_or(|x| x.is_null() || x.is_string()))
        && v.get("coding_data_retention_opt_out")
            .is_none_or(Value::is_boolean)
        && v.get("has_grok_code_access")
            .is_none_or(|x| x.is_null() || x.is_boolean())
        && v.get("team_blocked_reasons").is_none_or(|x| {
            x.as_array()
                .is_some_and(|items| items.iter().all(Value::is_string))
        })
}
fn identity(v: &Value) -> Option<Vec<Option<&str>>> {
    let user = string(v, "user_id")?;
    if user.trim().is_empty() || user.eq_ignore_ascii_case("unknown") {
        return None;
    }
    let mut fields = vec![Some(user)];
    for key in ["principal_type", "principal_id", "team_id"] {
        match v.get(key) {
            None | Some(Value::Null) => fields.push(None),
            Some(Value::String(s))
                if !s.trim().is_empty() && !s.eq_ignore_ascii_case("unknown") =>
            {
                fields.push(Some(s))
            }
            _ => return None,
        }
    }
    if fields[1].is_some() != fields[2].is_some() {
        return None;
    }
    Some(fields)
}
fn saved_value(saved: &Saved) -> Result<Value> {
    let value: Value = serde_json::from_str(saved.credential.expose())
        .map_err(|_| error("Saved Grok credential is damaged"))?;
    if !credential(&saved.scope, &value) {
        return Err(error("Saved Grok credential is unsupported or damaged"));
    }
    Ok(value)
}
fn matching(store: &Store, scope: &str, value: &Value) -> Result<Option<String>> {
    let mut exact = Vec::new();
    let mut identified = Vec::new();
    for (name, saved) in &store.accounts {
        if saved.scope != scope {
            continue;
        }
        let other = saved_value(saved)?;
        if other == *value {
            exact.push(name.clone());
        }
        if identity(value).is_some() && identity(value) == identity(&other) {
            identified.push(name.clone());
        }
    }
    let candidates = if exact.is_empty() { identified } else { exact };
    Ok(if candidates.len() == 1 {
        candidates.into_iter().next()
    } else {
        None
    })
}
fn paths() -> Result<Paths> {
    let root = crate::application::auth_off::auth_off_backup_root()?;
    let runtime = std::path::absolute(crate::application::auth_off::grok_auth_json_path()?)
        .map_err(|_| error("Cannot resolve Grok runtime path"))?;
    let store = std::path::absolute(root.join("platforms/grok/auth/accounts.json"))
        .map_err(|_| error("Cannot resolve Grok account path"))?;
    // All operations on this store serialize, even with different GROK_HOME values.
    Ok(Paths {
        store,
        runtime,
        operation: root.join("locks/grok-auth-accounts.lock"),
    })
}
fn load(p: &Paths) -> Result<State> {
    let bytes = read(&p.store)?;
    let store: Store = match &bytes {
        None => Store::default(),
        Some(b) => serde_json::from_slice(b)
            .map_err(|_| error("Grok account store is damaged; no changes made"))?,
    };
    for saved in store.accounts.values() {
        saved_value(saved)?;
    }
    let runtime = read(&p.runtime)?;
    let revision = GrokAuthRevision {
        store: token(bytes.as_deref()),
        runtime: token(runtime.as_deref()),
    };
    Ok(State {
        store,
        runtime,
        revision,
    })
}
fn check(state: &State, expected: &GrokAuthRevision) -> Result<()> {
    if state.revision != *expected {
        return Err(error(
            "Grok authentication data changed; refresh before retrying",
        ));
    }
    Ok(())
}
fn persist(path: &Path, bytes: &[u8], expected: &str) -> Result<Option<String>> {
    let outcome = write_guarded_versioned(
        path,
        bytes,
        expected,
        &WriteOptions {
            secret: true,
            ..WriteOptions::default()
        },
    );
    #[cfg(test)]
    let outcome = if WRITE_ERROR_AFTER_WRITE.with(|slot| slot.borrow().as_deref() == Some(path)) {
        WRITE_ERROR_AFTER_WRITE.with(|slot| slot.borrow_mut().take());
        outcome.and_then(|_| Err(error("Injected write-after-rename error")))
    } else {
        outcome
    };
    match outcome {
        Ok(VersionedWriteOutcome::Written) => Ok(None),
        Ok(VersionedWriteOutcome::Conflict) => Err(error(
            "Grok authentication data changed; refresh before retrying",
        )),
        Err(_) => match read(path) {
            Ok(Some(actual)) if actual == bytes => Ok(Some(
                "Applied locally; durability is unconfirmed. Refresh before another operation"
                    .into(),
            )),
            Ok(_) => Err(error(
                "Grok write failed or was superseded; refresh before retrying",
            )),
            Err(_) => Err(error(
                "Grok write outcome is unknown; refresh before retrying",
            )),
        },
    }
}
fn save_store(p: &Paths, state: &State) -> Result<Option<String>> {
    #[cfg(test)]
    if let Some(hook) = BEFORE_STORE_WRITE.with(|slot| slot.take()) {
        hook(p);
    }
    let bytes = serde_json::to_vec_pretty(&state.store)
        .map_err(|_| error("Cannot encode Grok account store"))?;
    let outcome = persist(&p.store, &bytes, &state.revision.store)?;
    #[cfg(test)]
    if STORE_DURABILITY_WARNING.with(|flag| flag.replace(false)) {
        return Ok(Some("Injected durability warning".into()));
    }
    Ok(outcome)
}
fn captured(scope: &str, value: &Value) -> Saved {
    Saved {
        scope: scope.into(),
        saved_at: chrono::Utc::now().to_rfc3339(),
        credential: Secret::new(value.to_string()),
    }
}
impl GrokAuthService {
    pub fn read_snapshot(&self) -> Result<GrokAuthSnapshot> {
        let state = load(&paths()?)?;
        let mut result = GrokAuthSnapshot {
            runtime_present: state.runtime.is_some(),
            revision: state.revision.clone(),
            ..Default::default()
        };
        match document(state.runtime.as_deref()) {
            Ok(doc) => {
                for (scope, value) in doc.iter().filter(|(s, v)| credential(s, v)) {
                    result.sources.push(GrokAuthSource {
                        scope: scope.clone(),
                        email: string(value, "email").map(str::to_owned),
                        team: string(value, "team_id").map(str::to_owned),
                        matched_account: matching(&state.store, scope, value)?,
                    });
                }
            }
            Err(e) => result.runtime_error = Some(e.to_string()),
        }
        for (name, saved) in &state.store.accounts {
            let value = saved_value(saved)?;
            let expires_at = string(&value, "expires_at").map(str::to_owned);
            let expired = expires_at
                .as_deref()
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .is_some_and(|t| t < chrono::Utc::now());
            result.accounts.push(GrokAuthAccount {
                name: name.clone(),
                scope: saved.scope.clone(),
                email: string(&value, "email").map(str::to_owned),
                team: string(&value, "team_id").map(str::to_owned),
                saved_at: saved.saved_at.clone(),
                expires_at,
                expired,
                local_match: result
                    .sources
                    .iter()
                    .any(|s| s.matched_account.as_deref() == Some(name.as_str())),
            });
        }
        Ok(result)
    }
    pub fn save_current(
        &self,
        name: &str,
        source_scope: &str,
        revision: &GrokAuthRevision,
        replace: bool,
    ) -> Result<GrokAuthMutation> {
        if name.is_empty()
            || name == "default"
            || name.len() > 32
            || !name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Err(error(
                "Account name must be 1–32 letters, digits, underscores or hyphens; default is reserved",
            ));
        }
        let p = paths()?;
        let _operation = FileLock::new(&p.operation, Duration::from_secs(10))?;
        let mut state = load(&p)?;
        check(&state, revision)?;
        let doc = document(state.runtime.as_deref())?;
        let value = doc
            .get(source_scope)
            .filter(|v| credential(source_scope, v))
            .ok_or_else(|| {
                error("Select an official OAuth source; log in with Grok first if none exists")
            })?;
        if state.store.accounts.contains_key(name) && !replace {
            return Err(error(
                "Account already exists; explicit overwrite confirmation is required",
            ));
        }
        for (other, saved) in &state.store.accounts {
            let old = saved_value(saved)?;
            if other != name
                && saved.scope == source_scope
                && (old == *value
                    || (identity(value).is_some() && identity(value) == identity(&old)))
            {
                return Err(error(
                    "This account is already saved under another name; overwrite that saved account",
                ));
            }
        }
        state
            .store
            .accounts
            .insert(name.into(), captured(source_scope, value));
        Ok(GrokAuthMutation {
            warnings: save_store(&p, &state)?.into_iter().collect(),
            outgoing_saved: false,
        })
    }
    pub fn delete_account(
        &self,
        name: &str,
        revision: &GrokAuthRevision,
    ) -> Result<GrokAuthMutation> {
        let p = paths()?;
        let _operation = FileLock::new(&p.operation, Duration::from_secs(10))?;
        let mut state = load(&p)?;
        check(&state, revision)?;
        if state.store.accounts.remove(name).is_none() {
            return Err(error("Saved account no longer exists; refresh"));
        }
        Ok(GrokAuthMutation {
            warnings: save_store(&p, &state)?.into_iter().collect(),
            outgoing_saved: false,
        })
    }
    pub fn switch_account(
        &self,
        name: &str,
        revision: &GrokAuthRevision,
    ) -> Result<GrokAuthMutation> {
        let p = paths()?;
        let _operation = FileLock::new(&p.operation, Duration::from_secs(10))?;
        let _official = FileLock::new(
            p.runtime.with_extension("json.lock"),
            Duration::from_secs(10),
        )?;
        let mut state = load(&p)?;
        check(&state, revision)?;
        let scope = state
            .store
            .accounts
            .get(name)
            .ok_or_else(|| error("Saved account no longer exists; refresh"))?
            .scope
            .clone();
        let initial_target = saved_value(
            state
                .store
                .accounts
                .get(name)
                .ok_or_else(|| error("Saved target disappeared"))?,
        )?;
        if string(&initial_target, "user_id").is_none() {
            return Err(error(
                "Saved snapshot lacks the native user_id field; log in with Grok and save it again",
            ));
        }
        let mut doc = document(state.runtime.as_deref())?;
        let mut result = GrokAuthMutation::default();
        if let Some(outgoing) = doc.get(&scope) {
            if !credential(&scope, outgoing) {
                return Err(error(
                    "Current scope is unsupported or damaged; explicitly preserve it before switching",
                ));
            }
            let matched = matching(&state.store, &scope, outgoing)?.ok_or_else(|| {
                error("Current scope is unsaved or identity is uncertain; explicitly save it before switching")
            })?;
            state
                .store
                .accounts
                .insert(matched, captured(&scope, outgoing));
            if save_store(&p, &state)?.is_some() {
                return Err(error(
                    "Outgoing account was written but durability is unconfirmed; runtime was not changed. Refresh",
                ));
            }
            result.outgoing_saved = true;
        }
        let target = saved_value(
            state
                .store
                .accounts
                .get(name)
                .ok_or_else(|| error("Saved target disappeared"))?,
        )?;
        if doc.get(&scope) == Some(&target) {
            return Ok(result);
        }
        doc.insert(scope, target);
        let bytes =
            serde_json::to_vec_pretty(&doc).map_err(|_| error("Cannot encode Grok session"))?;
        match persist(&p.runtime, &bytes, &state.revision.runtime) {
            Ok(warning) => result.warnings.extend(warning),
            Err(_) if result.outgoing_saved => {
                return Err(error(
                    "Outgoing account was saved; switching did not complete or its outcome is unknown. Refresh",
                ));
            }
            Err(e) => return Err(e),
        }
        Ok(result)
    }
    pub fn off_checked(&self, revision: &GrokAuthRevision) -> Result<AuthOffResult> {
        self.off_inner(Some(revision))
    }
    pub(super) fn off_inner(&self, revision: Option<&GrokAuthRevision>) -> Result<AuthOffResult> {
        let p = paths()?;
        let _operation = FileLock::new(&p.operation, Duration::from_secs(10))?;
        let _official = FileLock::new(
            p.runtime.with_extension("json.lock"),
            Duration::from_secs(10),
        )?;
        let mut state = load(&p)?;
        if let Some(revision) = revision {
            check(&state, revision)?;
        }
        let mut changed = false;
        if let Ok(doc) = document(state.runtime.as_deref()) {
            for (scope, value) in doc.iter().filter(|(s, v)| credential(s, v)) {
                if let Some(name) = matching(&state.store, scope, value)? {
                    state.store.accounts.insert(name, captured(scope, value));
                    changed = true;
                }
            }
        }
        if changed && save_store(&p, &state)?.is_some() {
            return Err(error(
                "Outgoing accounts were written but durability is unconfirmed; runtime was not deleted. Refresh",
            ));
        }
        if token(read(&p.runtime)?.as_deref()) != state.revision.runtime {
            return Err(error("Grok session changed before logout; refresh"));
        }
        crate::application::auth_off::grok_auth_off_locked(&p.runtime)
    }
}
