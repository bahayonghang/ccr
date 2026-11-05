//! 定价管理命令

use ccr_store::PricingManager;
use serde_json::Value;

#[ccr_tauri_command_macros::command]
pub async fn set_pricing(model: String, pricing: Value) -> Result<Value, String> {
    let result = tokio::task::spawn_blocking(move || {
        let config_path = PricingManager::default_config_path()
            .map_err(|e| format!("Failed to open pricing manager: {e}"))?;
        ccr_app::usage::policies::set_pricing(config_path, model, pricing)
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    Ok(result)
}

#[ccr_tauri_command_macros::command]
pub async fn get_pricing_list() -> Result<Value, String> {
    let result = tokio::task::spawn_blocking(|| {
        let config_path = PricingManager::default_config_path()
            .map_err(|e| format!("Failed to open pricing manager: {e}"))?;
        ccr_app::usage::policies::get_pricing_list(config_path)
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    Ok(result)
}

#[ccr_tauri_command_macros::command]
pub async fn remove_pricing(model: String) -> Result<Value, String> {
    let result = tokio::task::spawn_blocking(move || {
        let config_path = PricingManager::default_config_path()
            .map_err(|e| format!("Failed to open pricing manager: {e}"))?;
        ccr_app::usage::policies::remove_pricing(config_path, model)
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    Ok(result)
}

#[ccr_tauri_command_macros::command]
pub async fn reset_pricing() -> Result<Value, String> {
    let result = tokio::task::spawn_blocking(|| {
        let config_path = PricingManager::default_config_path()
            .map_err(|e| format!("Failed to open pricing manager: {e}"))?;
        ccr_app::usage::policies::reset_pricing(config_path)
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    Ok(result)
}
