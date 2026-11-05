//! Explicit captured-root budget/pricing commands. No upstream usage writes.
use super::*;
use crate::runtime::ConfigSource;
use ccr_app::usage::policies;
use ccr_core::core::lock::LockManager;
use serde_json::Value;

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
                let paths = paths.ok_or(InsightsError::Unconfigured)?;
                let locks = if fixture {
                    paths.provider_home.join(".ccr/locks")
                } else {
                    LockManager::with_default_path()
                        .map_err(|_| InsightsError::Archive)?
                        .lock_dir()
                        .to_path_buf()
                };
                let claude = paths.provider_home.join(".claude");
                LockManager::with_scoped_directory(&locks, || {
                    Ok(match command {
                        PolicyCommand::Budgets => {
                            policies::read_budget(claude.join("budget.toml"), claude.join("stats"))
                        }
                        PolicyCommand::UpdateBudget(patch) => {
                            policies::update_budget(claude.join("budget.toml"), patch)
                        }
                        PolicyCommand::Pricing => {
                            policies::get_pricing_list(claude.join("pricing.toml"))
                        }
                        PolicyCommand::SetPricing { model, pricing } => {
                            policies::set_pricing(claude.join("pricing.toml"), model, pricing)
                        }
                        PolicyCommand::RemovePricing(model) => {
                            policies::remove_pricing(claude.join("pricing.toml"), model)
                        }
                        PolicyCommand::ResetPricing => {
                            policies::reset_pricing(claude.join("pricing.toml"))
                        }
                    })
                })
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
