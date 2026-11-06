//! Explicit captured-root budget/pricing commands. No upstream usage writes.
use super::*;
use crate::runtime::ConfigSource;
use ccr_app::usage::policies;
use ccr_core::core::lock::LockManager;
use ccr_store::{BudgetManager, CostTracker, PricingManager};
use serde_json::Value;

struct PolicyPaths {
    budget: PathBuf,
    pricing: PathBuf,
    stats: PathBuf,
}

fn policy_paths(
    paths: Option<&InsightsPaths>,
    fixture: bool,
    local_defaults: impl FnOnce() -> Result<PolicyPaths, InsightsError>,
) -> Result<PolicyPaths, InsightsError> {
    if let Some(paths) = paths {
        let claude = paths.provider_home.join(".claude");
        return Ok(PolicyPaths {
            budget: claude.join("budget.toml"),
            pricing: claude.join("pricing.toml"),
            stats: claude.join("stats"),
        });
    }
    if fixture {
        return Err(InsightsError::Unconfigured);
    }
    // CCR policies exist independently of llmusage or an indexed archive.
    // Resolve the same defaults as Tauri, only on the bounded worker.
    local_defaults()
}

fn execute(paths: PolicyPaths, command: PolicyCommand) -> Result<Value, String> {
    match command {
        PolicyCommand::Budgets => policies::read_budget(paths.budget, paths.stats),
        PolicyCommand::UpdateBudget(patch) => policies::update_budget(paths.budget, patch),
        PolicyCommand::Pricing => policies::get_pricing_list(paths.pricing),
        PolicyCommand::SetPricing { model, pricing } => {
            policies::set_pricing(paths.pricing, model, pricing)
        }
        PolicyCommand::RemovePricing(model) => policies::remove_pricing(paths.pricing, model),
        PolicyCommand::ResetPricing => policies::reset_pricing(paths.pricing),
    }
}

pub enum PolicyCommand {
    Budgets,
    UpdateBudget(Value),
    Pricing,
    SetPricing { model: String, pricing: Value },
    RemovePricing(String),
    ResetPricing,
}
impl InsightsRequests {
    pub fn policy(
        &self,
        key: RequestKey,
        command: PolicyCommand,
    ) -> Result<TaskSubscription<Value, InsightsError>, AdmissionError> {
        let paths = self.paths.clone();
        let fixture = matches!(self.services.source, ConfigSource::Fixture { .. });
        let local = matches!(key.environment, EnvironmentId::Local(_));
        // Budget status constructs its existing cost tracker and may create the
        // explicit stats directory. Keep every command app-owned through completion.
        self.services.executor.job(key, move |_, _| {
            let result = (|| {
                if !local {
                    return Err(InsightsError::LocalOnly);
                }
                let policy_paths = policy_paths(paths.as_ref(), fixture, || {
                    Ok(PolicyPaths {
                        budget: BudgetManager::default_config_path()
                            .map_err(|_| InsightsError::Query)?,
                        pricing: PricingManager::default_config_path()
                            .map_err(|_| InsightsError::Query)?,
                        stats: CostTracker::default_storage_dir()
                            .map_err(|_| InsightsError::Query)?,
                    })
                })?;
                let locks = if fixture {
                    paths
                        .as_ref()
                        .ok_or(InsightsError::Unconfigured)?
                        .provider_home
                        .join(".ccr/locks")
                } else {
                    LockManager::with_default_path()
                        .map_err(|_| InsightsError::Archive)?
                        .lock_dir()
                        .to_path_buf()
                };
                LockManager::with_scoped_directory(&locks, || Ok(execute(policy_paths, command)))
                    .map_err(|_| InsightsError::Archive)?
                    .map_err(|_| InsightsError::Query)
            })();
            match result {
                Ok(value) => Completion::Success(value),
                Err(error) => Completion::Failed(error),
            }
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn policy_defaults_work_without_a_usage_database_but_fixture_never_falls_back() {
        let root = tempfile::tempdir().unwrap();
        let resolved = || {
            Ok(PolicyPaths {
                budget: root.path().join("budget.toml"),
                pricing: root.path().join("pricing.toml"),
                stats: root.path().join("stats"),
            })
        };
        assert!(matches!(
            policy_paths(None, true, || panic!(
                "fixture must not resolve local defaults"
            )),
            Err(InsightsError::Unconfigured)
        ));
        LockManager::with_scoped_directory(&root.path().join("locks"), || {
            execute(
                policy_paths(None, false, resolved).unwrap(),
                PolicyCommand::UpdateBudget(serde_json::json!({"dailyLimit":12.5})),
            )
            .unwrap();
            execute(
                policy_paths(None, false, resolved).unwrap(),
                PolicyCommand::SetPricing {
                    model: "fixture-model".into(),
                    pricing: serde_json::json!({
                        "model":"fixture-model", "input_price":1.25, "output_price":2.5,
                    }),
                },
            )
            .unwrap();
            Ok(())
        })
        .unwrap();
        assert_eq!(
            BudgetManager::new(root.path().join("budget.toml"))
                .unwrap()
                .get_config()
                .daily_limit,
            Some(12.5)
        );
        assert_eq!(
            PricingManager::new(root.path().join("pricing.toml"))
                .unwrap()
                .get_pricing("fixture-model")
                .unwrap()
                .input_price,
            1.25
        );
        let explicit = InsightsPaths {
            usage_root: root.path().join("usage"),
            archive_db: root.path().join("archive.db"),
            provider_home: root.path().join("explicit"),
        };
        assert_eq!(
            policy_paths(Some(&explicit), true, || panic!(
                "explicit fixture paths must win"
            ))
            .unwrap()
            .budget,
            root.path().join("explicit/.claude/budget.toml")
        );
        assert!(!explicit.usage_root.exists());
        assert!(!explicit.archive_db.exists());
    }
}
