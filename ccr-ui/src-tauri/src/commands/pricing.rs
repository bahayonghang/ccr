//! 定价管理命令

use ccr_store::{ModelPricing, PricingManager};
use serde_json::Value;

#[tauri::command]
pub async fn set_pricing(model: String, pricing: Value) -> Result<Value, String> {
    let result = tokio::task::spawn_blocking(move || {
        let model_pricing: ModelPricing =
            serde_json::from_value(pricing).map_err(|e| format!("Invalid pricing data: {e}"))?;

        let mut manager = PricingManager::with_default()
            .map_err(|e| format!("Failed to open pricing manager: {e}"))?;

        manager
            .set_pricing(model, model_pricing)
            .map_err(|e| format!("Failed to set pricing: {e}"))?;

        let config = manager.get_config();
        serde_json::to_value(config).map_err(|e| format!("Serialization error: {e}"))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    Ok(result)
}

#[tauri::command]
pub async fn get_pricing_list() -> Result<Value, String> {
    let result = tokio::task::spawn_blocking(|| {
        let manager = PricingManager::with_default()
            .map_err(|e| format!("Failed to open pricing manager: {e}"))?;

        let list = manager.export_pricing();
        let items: Vec<Value> = list
            .into_iter()
            .map(|(model, pricing)| {
                serde_json::json!({
                    "model": model,
                    "pricing": pricing,
                })
            })
            .collect();

        Ok::<_, String>(serde_json::json!({ "items": items, "total": items.len() }))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    Ok(result)
}

#[tauri::command]
pub async fn remove_pricing(model: String) -> Result<Value, String> {
    let result = tokio::task::spawn_blocking(move || {
        let mut manager = PricingManager::with_default()
            .map_err(|e| format!("Failed to open pricing manager: {e}"))?;

        let removed = manager
            .remove_pricing(&model)
            .map_err(|e| format!("Failed to remove pricing: {e}"))?;

        Ok::<_, String>(serde_json::json!({
            "removed": removed.is_some(),
            "model": model,
        }))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    Ok(result)
}

#[tauri::command]
pub async fn reset_pricing() -> Result<Value, String> {
    let result = tokio::task::spawn_blocking(|| {
        let mut manager = PricingManager::with_default()
            .map_err(|e| format!("Failed to open pricing manager: {e}"))?;

        manager
            .reset_to_defaults()
            .map_err(|e| format!("Failed to reset pricing: {e}"))?;

        let config = manager.get_config();
        serde_json::to_value(config).map_err(|e| format!("Serialization error: {e}"))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    Ok(result)
}
