use std::future::Future;

use ccr_store::CostTracker;

use crate::state::{AppState, CacheFillRegistration};

const STATS_CACHE_TTL_SECS: u64 = 30;

pub(crate) fn create_cost_tracker() -> Result<CostTracker, String> {
    let storage_dir =
        CostTracker::default_storage_dir().map_err(|e| format!("Failed to get stats dir: {e}"))?;
    CostTracker::new(storage_dir).map_err(|e| format!("Failed to create cost tracker: {e}"))
}

pub(crate) fn normalize_usage_platform(raw: &str) -> &'static str {
    match raw.to_lowercase().as_str() {
        "claude" | "claude-code" | "claude code" => "claude",
        "codex" | "openai-codex" | "openai codex" => "codex",
        "gemini" | "google-gemini" | "google gemini" => "gemini",
        _ => "claude",
    }
}

pub(crate) async fn run_cached_stats_command<F, Fut>(
    state: &AppState,
    cache_key: String,
    compute: F,
) -> Result<serde_json::Value, String>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<serde_json::Value, String>>,
{
    if let Some(cached) = state.cache_get(&cache_key).await {
        tracing::debug!(cache_key = %cache_key, "stats cache hit");
        return Ok(cached);
    }

    match state.begin_cache_fill(&cache_key).await {
        CacheFillRegistration::Wait(notify) => {
            tracing::debug!(cache_key = %cache_key, "stats cache wait");
            notify.notified().await;
            if let Some(cached) = state.cache_get(&cache_key).await {
                tracing::debug!(cache_key = %cache_key, "stats cache hit after wait");
                return Ok(cached);
            }
            tracing::debug!(cache_key = %cache_key, "stats cache fallback compute");
            compute().await
        }
        CacheFillRegistration::Leader => match compute().await {
            Ok(payload) => {
                state
                    .cache_set(cache_key.clone(), payload.clone(), STATS_CACHE_TTL_SECS)
                    .await;
                state.finish_cache_fill(&cache_key).await;
                tracing::debug!(cache_key = %cache_key, "stats cache fill complete");
                Ok(payload)
            }
            Err(err) => {
                state.finish_cache_fill(&cache_key).await;
                Err(err)
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_usage_platform_accepts_known_aliases() {
        assert_eq!(normalize_usage_platform("claude-code"), "claude");
        assert_eq!(normalize_usage_platform("OpenAI Codex"), "codex");
        assert_eq!(normalize_usage_platform("google-gemini"), "gemini");
    }

    #[test]
    fn normalize_usage_platform_keeps_unknown_data_in_legacy_bucket() {
        assert_eq!(normalize_usage_platform("unknown"), "claude");
    }
}
