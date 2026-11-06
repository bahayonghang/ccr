//! Codex model-provider library; persistence and private backups stay domain-owned.
use super::*;
use ccr_codex::{
    CodexModelProviderApiKey, CodexModelProviderRecord, services::CodexModelProviderStoreService,
};

fn service(roots: &PlatformRoots) -> CodexModelProviderStoreService {
    CodexModelProviderStoreService::from_parts(&roots.codex_store, &roots.locks)
}

pub fn project(roots: &PlatformRoots) -> Result<(ProviderReport, Vec<String>), PlatformError> {
    let store = service(roots).load().map_err(domain_error)?;
    let mut ids = Vec::new();
    let mut hidden = 0;
    for provider in store.providers {
        if let Some(id) = safe_name(&provider.id) {
            ids.push(id);
        } else {
            hidden += 1;
        }
    }
    Ok((ProviderReport::Ready { hidden }, ids))
}

fn required(value: &Value, key: &str) -> Result<String, PlatformError> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
        .ok_or(PlatformError::Invalid)
}
fn optional(value: &Value, key: &str) -> Result<Option<String>, PlatformError> {
    match value.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => {
            Ok((!value.trim().is_empty()).then(|| value.trim().to_string()))
        }
        _ => Err(PlatformError::Invalid),
    }
}

impl PlatformSession {
    /// Safe provider metadata; never return the backend record or API keys.
    pub fn provider_fields(&self, id: &str) -> Vec<FieldFact> {
        let Ok(root) = serde_json::from_str::<Value>(&self.snapshot.content) else {
            return Vec::new();
        };
        let entry = if self.platform == Platform::Codex {
            root.get("providers")
                .and_then(Value::as_array)
                .and_then(|rows| {
                    rows.iter()
                        .find(|row| row.get("id").and_then(Value::as_str) == Some(id))
                })
        } else {
            root.get("provider")
                .or_else(|| root.get("providers"))
                .and_then(|rows| rows.get(id))
        };
        let Some(entry) = entry else {
            return Vec::new();
        };
        let mut fields = Vec::new();
        for key in ["name", "base_url", "website_url", "api_key_url", "npm"] {
            if let Some(value) = entry.get(key).and_then(Value::as_str)
                && (!key.ends_with("url") || safe_url(value))
                && let Some(value) = safe_scalar(value)
            {
                fields.push(FieldFact { key, value });
            }
        }
        if let Some(value) = entry
            .get("options")
            .and_then(|v| v.get("baseURL"))
            .and_then(Value::as_str)
            && safe_url(value)
            && let Some(value) = safe_scalar(value)
        {
            fields.push(FieldFact {
                key: "baseURL",
                value,
            });
        }
        fields
    }
}

fn safe_url(value: &str) -> bool {
    reqwest::Url::parse(value).is_ok_and(|url| {
        matches!(url.scheme(), "http" | "https")
            && url.username().is_empty()
            && url.password().is_none()
            && url.query().is_none()
            && url.fragment().is_none()
    })
}

pub fn apply(
    session: &PlatformSession,
    id: &str,
    patch: Option<&str>,
) -> Result<ResultValue, PlatformError> {
    if session.surface != Surface::Providers || session.platform != Platform::Codex {
        return Err(PlatformError::Unsupported);
    }
    let id = safe_name(id).ok_or(PlatformError::Invalid)?;
    let provider = patch
        .map(|patch| {
            let mut value: Value =
                serde_json::from_str(patch).map_err(|_| PlatformError::Invalid)?;
            let object = value.as_object().ok_or(PlatformError::Invalid)?;
            if object.keys().any(|key| {
                ![
                    "name",
                    "base_url",
                    "website_url",
                    "api_key_url",
                    "api_key",
                    "api_key_name",
                ]
                .contains(&key.as_str())
            }) {
                return Err(PlatformError::Invalid);
            }
            // Omitted metadata remains in the captured original provider, just
            // like omitted keys. The domain's versioned mutation still owns CAS.
            let original: Value = if session.snapshot.exists {
                serde_json::from_str(&session.snapshot.content)
                    .map_err(|_| PlatformError::Invalid)?
            } else {
                serde_json::json!({})
            };
            if let Some(existing) = original
                .get("providers")
                .and_then(Value::as_array)
                .and_then(|rows| {
                    rows.iter()
                        .find(|row| row.get("id").and_then(Value::as_str) == Some(id.as_str()))
                })
            {
                for key in ["name", "base_url", "website_url", "api_key_url"] {
                    if value.get(key).is_none()
                        && let Some(original) = existing.get(key)
                    {
                        value
                            .as_object_mut()
                            .ok_or(PlatformError::Invalid)?
                            .insert(key.into(), original.clone());
                    }
                }
            }
            let now = chrono::Utc::now();
            let api_key_name = optional(&value, "api_key_name")?;
            let api_keys = optional(&value, "api_key")?
                .map(|api_key| CodexModelProviderApiKey {
                    id: format!("key-{}", now.timestamp_micros()),
                    name: api_key_name.unwrap_or_else(|| "Primary".into()),
                    api_key,
                    created_at: now,
                    updated_at: now,
                })
                .into_iter()
                .collect();
            Ok(CodexModelProviderRecord {
                id: id.clone(),
                name: required(&value, "name")?,
                base_url: required(&value, "base_url")?.trim_end_matches('/').into(),
                website_url: optional(&value, "website_url")?,
                api_key_url: optional(&value, "api_key_url")?,
                api_keys,
                created_at: now,
                updated_at: now,
            })
        })
        .transpose()?;
    let saved = service(&session.roots)
        .mutate_versioned(&session.snapshot.token, &id, provider)
        .map_err(domain_error)?;
    Ok(if saved {
        ResultValue::AccountChanged { warning: false }
    } else {
        ResultValue::Saved(SaveOutcome::Conflict)
    })
}
