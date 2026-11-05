//! Existing budget/pricing orchestration with explicit paths. These are CCR legacy cost policies,
//! not repricing commands for the read-only upstream llmusage database.
use ccr_store::{BudgetManager, CostTracker, ModelPricing, PricingManager};
use serde_json::Value;
use std::path::PathBuf;

pub fn read_budget(config_path: PathBuf, storage_dir: PathBuf) -> Result<Value, String> {
    let budget_manager =
        BudgetManager::new(config_path).map_err(|e| format!("BudgetManager init error: {}", e))?;

    let tracker =
        CostTracker::new(storage_dir).map_err(|e| format!("CostTracker init error: {}", e))?;

    let status = budget_manager
        .check_status(&tracker)
        .map_err(|e| format!("Budget status error: {}", e))?;
    let config = budget_manager.get_config();

    Ok(serde_json::json!({
        "enabled": status.enabled,
        "dailyLimit": config.daily_limit,
        "weeklyLimit": config.weekly_limit,
        "monthlyLimit": config.monthly_limit,
        "warnAtPercent": config.warn_at_percent,
        "currentCosts": {
            "today": status.current_costs.today,
            "thisWeek": status.current_costs.this_week,
            "thisMonth": status.current_costs.this_month,
        },
        "warnings": status.warnings.iter().map(|w| serde_json::json!({
            "period": w.period.to_string(),
            "currentCost": w.current_cost,
            "limit": w.limit,
            "usagePercent": w.usage_percent,
        })).collect::<Vec<_>>(),
    }))
}

pub fn update_budget(config_path: PathBuf, budgets: Value) -> Result<Value, String> {
    let fields = budgets
        .as_object()
        .ok_or_else(|| "budgets must be an object".to_string())?;
    if fields
        .get("enabled")
        .is_some_and(|value| !value.is_boolean())
    {
        return Err("enabled must be a boolean".to_string());
    }
    for key in ["dailyLimit", "weeklyLimit", "monthlyLimit"] {
        if let Some(value) = fields.get(key)
            && !value.is_null()
            && !value
                .as_f64()
                .is_some_and(|value| value.is_finite() && value >= 0.0)
        {
            return Err(format!("{key} must be a finite nonnegative number or null"));
        }
    }
    let warn_at_percent = fields
        .get("warnAtPercent")
        .map(|value| {
            value
                .as_u64()
                .filter(|value| *value <= 100)
                .and_then(|value| u8::try_from(value).ok())
                .ok_or_else(|| "warnAtPercent must be an integer from 0 to 100".to_string())
        })
        .transpose()?;
    // Validate the entire patch before the first setter. Valid setters retain
    // their existing sequential disk writes and I/O failure semantics.
    let mut budget_manager =
        BudgetManager::new(config_path).map_err(|e| format!("BudgetManager init error: {}", e))?;

    if let Some(enabled) = budgets.get("enabled").and_then(|v| v.as_bool()) {
        if enabled {
            budget_manager
                .enable()
                .map_err(|e| format!("Failed to enable budget: {}", e))?;
        } else {
            budget_manager
                .disable()
                .map_err(|e| format!("Failed to disable budget: {}", e))?;
        }
    }

    if let Some(daily_val) = budgets.get("dailyLimit") {
        let limit = if daily_val.is_null() {
            None
        } else {
            daily_val
                .as_f64()
                .map(Some)
                .ok_or_else(|| "dailyLimit must be a number or null".to_string())?
        };
        budget_manager
            .set_daily_limit(limit)
            .map_err(|e| format!("Failed to set daily limit: {}", e))?;
    }

    if let Some(weekly_val) = budgets.get("weeklyLimit") {
        let limit = if weekly_val.is_null() {
            None
        } else {
            weekly_val
                .as_f64()
                .map(Some)
                .ok_or_else(|| "weeklyLimit must be a number or null".to_string())?
        };
        budget_manager
            .set_weekly_limit(limit)
            .map_err(|e| format!("Failed to set weekly limit: {}", e))?;
    }

    if let Some(monthly_val) = budgets.get("monthlyLimit") {
        let limit = if monthly_val.is_null() {
            None
        } else {
            monthly_val
                .as_f64()
                .map(Some)
                .ok_or_else(|| "monthlyLimit must be a number or null".to_string())?
        };
        budget_manager
            .set_monthly_limit(limit)
            .map_err(|e| format!("Failed to set monthly limit: {}", e))?;
    }

    if let Some(pct) = warn_at_percent {
        budget_manager
            .set_warn_threshold(pct)
            .map_err(|e| format!("Failed to set warn threshold: {}", e))?;
    }

    let config = budget_manager.get_config();
    Ok(serde_json::json!({
        "enabled": config.enabled,
        "dailyLimit": config.daily_limit,
        "weeklyLimit": config.weekly_limit,
        "monthlyLimit": config.monthly_limit,
        "warnAtPercent": config.warn_at_percent,
    }))
}

pub fn set_pricing(config_path: PathBuf, model: String, pricing: Value) -> Result<Value, String> {
    let model_pricing: ModelPricing =
        serde_json::from_value(pricing).map_err(|e| format!("Invalid pricing data: {e}"))?;

    let mut manager = PricingManager::new(config_path)
        .map_err(|e| format!("Failed to open pricing manager: {e}"))?;

    manager
        .set_pricing(model, model_pricing)
        .map_err(|e| format!("Failed to set pricing: {e}"))?;

    let config = manager.get_config();
    serde_json::to_value(config).map_err(|e| format!("Serialization error: {e}"))
}

pub fn get_pricing_list(config_path: PathBuf) -> Result<Value, String> {
    let manager = PricingManager::new(config_path)
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
}

pub fn remove_pricing(config_path: PathBuf, model: String) -> Result<Value, String> {
    let mut manager = PricingManager::new(config_path)
        .map_err(|e| format!("Failed to open pricing manager: {e}"))?;

    let removed = manager
        .remove_pricing(&model)
        .map_err(|e| format!("Failed to remove pricing: {e}"))?;

    Ok::<_, String>(serde_json::json!({
        "removed": removed.is_some(),
        "model": model,
    }))
}

pub fn reset_pricing(config_path: PathBuf) -> Result<Value, String> {
    let mut manager = PricingManager::new(config_path)
        .map_err(|e| format!("Failed to open pricing manager: {e}"))?;

    manager
        .reset_to_defaults()
        .map_err(|e| format!("Failed to reset pricing: {e}"))?;

    let config = manager.get_config();
    serde_json::to_value(config).map_err(|e| format!("Serialization error: {e}"))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use ccr_core::core::lock::LockManager;
    use serde_json::json;

    #[test]
    fn invalid_budget_patch_never_truncates_percent_or_writes_earlier_fields() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("budget.toml");
        LockManager::with_scoped_directory(&root.path().join("locks"), || {
            update_budget(path.clone(), json!({"warnAtPercent":80,"dailyLimit":12})).unwrap();
            let before = std::fs::read(&path).unwrap();
            for patch in [
                json!({"warnAtPercent":300}),
                json!({"dailyLimit":1,"warnAtPercent":300}),
                json!({"enabled":true,"weeklyLimit":"invalid"}),
                json!({"enabled":1}),
                json!({"warnAtPercent":null}),
                json!({"warnAtPercent":1.5}),
                json!({"dailyLimit":1,"monthlyLimit":-2}),
                json!([]),
            ] {
                assert!(update_budget(path.clone(), patch).is_err());
                assert_eq!(std::fs::read(&path).unwrap(), before);
            }
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn budget_patch_preserves_omitted_fields_and_null_clears_through_existing_owner() {
        let root = tempfile::tempdir().unwrap();
        let budget = root.path().join("budget.toml");
        LockManager::with_scoped_directory(&root.path().join("locks"), || {
            update_budget(
                budget.clone(),
                json!({"weeklyLimit": 40, "monthlyLimit": 100}),
            )
            .unwrap();
            let changed = update_budget(
                budget.clone(),
                json!({"dailyLimit": 5, "monthlyLimit": null}),
            )
            .unwrap();
            assert_eq!(changed["dailyLimit"], 5.0);
            assert_eq!(changed["weeklyLimit"], 40.0);
            assert!(changed["monthlyLimit"].is_null());
            let before = std::fs::read(&budget).unwrap();
            assert!(update_budget(budget.clone(), json!({"dailyLimit": -1})).is_err());
            assert_eq!(std::fs::read(&budget).unwrap(), before);
            let result = read_budget(budget, root.path().join("stats")).unwrap();
            assert_eq!(result["weeklyLimit"], 40.0);
            assert!(!result["enabled"].as_bool().unwrap());
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn pricing_list_set_remove_reset_and_invalid_no_write_use_same_persistence() {
        let root = tempfile::tempdir().unwrap();
        let pricing = root.path().join("pricing.toml");
        let defaults = get_pricing_list(pricing.clone()).unwrap();
        assert!(!pricing.exists());
        LockManager::with_scoped_directory(&root.path().join("locks"), || {
            let row = json!({"model":"native-fixture", "input_price":1.23456789, "output_price":2.5,
                "cache_read_price":0.1, "cache_write_price":null});
            set_pricing(pricing.clone(), "native-fixture".into(), row).unwrap();
            let list = get_pricing_list(pricing.clone()).unwrap();
            assert!(
                list["items"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|row| row["model"] == "native-fixture"
                        && row["pricing"]["input_price"] == 1.23456789)
            );
            let before = std::fs::read(&pricing).unwrap();
            assert!(
                set_pricing(
                    pricing.clone(),
                    "invalid".into(),
                    json!({"model":"invalid","input_price":-1,"output_price":2})
                )
                .is_err()
            );
            assert_eq!(std::fs::read(&pricing).unwrap(), before);
            assert_eq!(
                remove_pricing(pricing.clone(), "native-fixture".into()).unwrap()["removed"],
                true
            );
            reset_pricing(pricing.clone()).unwrap();
            assert_eq!(
                get_pricing_list(pricing).unwrap()["total"],
                defaults["total"]
            );
            Ok(())
        })
        .unwrap();
    }
}
