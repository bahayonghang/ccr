//! State-free owner for the existing explicit CDK recharge command.
use crate::managers::checkin::builtin_providers::resolve_builtin_for_provider;
use crate::services::cdk_service::{CdkExtraConfig, CdkService};
use crate::{AccountManager, ProviderManager};
use ccr_core::Secret;
use ccr_db::database::DbAccess;
use serde_json::Value;
use std::path::Path;

pub async fn execute(
    checkin_dir: &Path,
    db: DbAccess,
    client: reqwest::Client,
    account_id: &str,
    code: &Secret,
) -> Result<Value, String> {
    let directory = checkin_dir.to_path_buf();
    let account_id = account_id.to_owned();
    let (provider, config, extra, cookies, api_user, topup) =
        tokio::task::spawn_blocking(move || {
            let account_manager = AccountManager::with_db(&directory, db.clone());
            let account = account_manager
                .get(&account_id)
                .map_err(|e| format!("Account not found: {e}"))?;
            let provider = ProviderManager::with_db(db)
                .get(&account.provider_id)
                .map_err(|e| format!("Provider not found: {e}"))?;
            let config = resolve_builtin_for_provider(&provider)
                .and_then(|p| p.cdk_config)
                .ok_or_else(|| {
                    format!("Provider {} does not support CDK recharge", provider.name)
                })?;
            let extra = CdkExtraConfig::from_json(&account.extra_config);
            let (cookies, api_user) = account_manager
                .get_cookies_json(&account_id)
                .map_err(|e| format!("Failed to decrypt cookies: {e}"))?;
            let cookies: std::collections::HashMap<String, String> =
                serde_json::from_str(cookies.expose()).unwrap_or_default();
            let topup = config
                .topup_path
                .as_ref()
                .map(|path| format!("{}{}", provider.base_url.trim_end_matches('/'), path));
            Ok::<_, String>((provider, config, extra, cookies, api_user, topup))
        })
        .await
        .map_err(|error| format!("Task join error: {error}"))??;
    if !code.expose().is_empty() {
        let url = topup
            .as_deref()
            .ok_or_else(|| format!("No topup URL configured for provider {}", provider.name))?;
        let cookie = cookies
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join("; ");
        let mut request = client
            .post(url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json");
        if !cookie.is_empty() {
            request = request.header("Cookie", cookie);
        }
        if !api_user.is_empty() {
            request = request.header("new-api-user", api_user);
        }
        let response = request
            .json(&serde_json::json!({"key":code.expose()}))
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {e}"))?;
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Ok(
            serde_json::json!({"success":status.is_success(),"cdk_code":code.expose(),"status_code":status.as_u16(),"response":body}),
        );
    }
    let result = CdkService::new(None)
        .fetch_and_topup(
            &config.cdk_type,
            &extra,
            topup.as_deref(),
            &cookies,
            &api_user,
        )
        .await;
    serde_json::to_value(result).map_err(|e| format!("Serialization error: {e}"))
}

/// Update only supplied CDK credential leaves; omitted settings and extensions survive.
pub fn update_credentials(
    checkin_dir: &Path,
    db: DbAccess,
    account_id: &str,
    patch: &Secret,
) -> Result<(), String> {
    let patch: Value = serde_json::from_str(patch.expose())
        .map_err(|_| "Invalid CDK credential object".to_string())?;
    let fields = patch
        .as_object()
        .ok_or_else(|| "Expected a CDK credential object".to_string())?;
    if fields.keys().any(|key| {
        !matches!(
            key.as_str(),
            "fuli_cookies" | "b4u_cdk_cookies" | "x666_access_token"
        )
    }) {
        return Err("Unknown CDK credential field".into());
    }
    serde_json::from_value::<CdkExtraConfig>(patch.clone())
        .map_err(|_| "Invalid CDK credential value".to_string())?;
    let manager = AccountManager::with_db(checkin_dir, db);
    let account = manager.get(account_id).map_err(|e| e.to_string())?;
    let mut current: Value = serde_json::from_str(&account.extra_config)
        .map_err(|_| "Invalid stored account configuration".to_string())?;
    let current = current
        .as_object_mut()
        .ok_or_else(|| "Expected stored account configuration object".to_string())?;
    for (key, value) in fields {
        if value.is_null() {
            current.remove(key);
        } else if let (Some(Value::Object(existing)), Value::Object(incoming)) =
            (current.get_mut(key), value)
        {
            for (name, value) in incoming {
                existing.insert(name.clone(), value.clone());
            }
        } else {
            current.insert(key.clone(), value.clone());
        }
    }
    manager
        .update(
            account_id,
            crate::models::checkin::UpdateAccountRequest {
                name: None,
                cookies_json: None,
                api_user: None,
                enabled: None,
                extra_config: Some(serde_json::to_string(current).map_err(|e| e.to_string())?),
            },
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}
