use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelRateOverride {
    pub model: String,
    pub input_price: f64,
    pub output_price: f64,
    pub cache_read_price: Option<f64>,
    pub cache_write_price: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModelRate {
    pub input_per_million: f64,
    pub cache_read_per_million: f64,
    pub output_per_million: f64,
    pub cache_creation_per_million: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PricingComputation {
    pub normalized_model: String,
    pub pricing_status: String,
    pub pricing_source: String,
    pub rate_label: Option<String>,
    pub input_rate_per_million: Option<f64>,
    pub cache_read_rate_per_million: Option<f64>,
    pub output_rate_per_million: Option<f64>,
    pub cost_with_cache_usd: f64,
    pub cost_without_cache_usd: f64,
}

#[derive(Debug, Clone, Default)]
pub struct ModelRateCatalog {
    overrides: Vec<ModelRateOverride>,
}

impl ModelRateCatalog {
    pub fn official() -> Self {
        Self {
            overrides: Vec::new(),
        }
    }

    pub fn with_overrides(overrides: Vec<ModelRateOverride>) -> Self {
        Self { overrides }
    }

    pub fn calculate(
        &self,
        model: &str,
        input_tokens: i64,
        output_tokens: i64,
        cache_read_tokens: i64,
        cache_creation_tokens: i64,
    ) -> PricingComputation {
        let normalized_model = normalize_model_id(model);
        let prompt_tokens = input_tokens
            .saturating_add(cache_read_tokens)
            .saturating_add(cache_creation_tokens);

        let Some((rate, source, status)) = self
            .override_rate(&normalized_model)
            .or_else(|| official_rate(&normalized_model, prompt_tokens))
        else {
            return PricingComputation {
                normalized_model,
                pricing_status: "unpriced".to_string(),
                pricing_source: "unpriced".to_string(),
                rate_label: None,
                input_rate_per_million: None,
                cache_read_rate_per_million: None,
                output_rate_per_million: None,
                cost_with_cache_usd: 0.0,
                cost_without_cache_usd: 0.0,
            };
        };

        let cache_creation_rate = rate
            .cache_creation_per_million
            .unwrap_or(rate.input_per_million);
        let cost_with_cache_usd = (input_tokens as f64 * rate.input_per_million
            + output_tokens as f64 * rate.output_per_million
            + cache_read_tokens as f64 * rate.cache_read_per_million
            + cache_creation_tokens as f64 * cache_creation_rate)
            / 1_000_000.0;
        let cost_without_cache_usd = (input_tokens
            .saturating_add(cache_read_tokens)
            .saturating_add(cache_creation_tokens) as f64
            * rate.input_per_million
            + output_tokens as f64 * rate.output_per_million)
            / 1_000_000.0;

        PricingComputation {
            normalized_model,
            pricing_status: status.to_string(),
            pricing_source: source.to_string(),
            rate_label: Some(rate_label(rate)),
            input_rate_per_million: Some(rate.input_per_million),
            cache_read_rate_per_million: Some(rate.cache_read_per_million),
            output_rate_per_million: Some(rate.output_per_million),
            cost_with_cache_usd,
            cost_without_cache_usd,
        }
    }

    pub fn rate_summary(&self, model: &str) -> Option<String> {
        let normalized_model = normalize_model_id(model);
        self.override_rate(&normalized_model)
            .or_else(|| official_rate(&normalized_model, 0))
            .map(|(rate, _, _)| rate_label(rate))
    }

    fn override_rate(
        &self,
        normalized_model: &str,
    ) -> Option<(ModelRate, &'static str, &'static str)> {
        self.overrides.iter().find_map(|item| {
            let item_model = normalize_model_id(&item.model);
            (item_model == normalized_model).then_some((
                ModelRate {
                    input_per_million: item.input_price,
                    cache_read_per_million: item.cache_read_price.unwrap_or(item.input_price),
                    output_per_million: item.output_price,
                    cache_creation_per_million: item.cache_write_price,
                },
                "override",
                "priced",
            ))
        })
    }
}

pub fn official_model_rate_overrides() -> Vec<ModelRateOverride> {
    [
        ("claude-opus-4-7", 5.0, 25.0, 0.5, 6.25),
        ("claude-opus-4-6", 5.0, 25.0, 0.5, 6.25),
        ("claude-opus-4-5-20251101", 5.0, 25.0, 0.5, 6.25),
        ("claude-sonnet-4-6", 3.0, 15.0, 0.3, 3.75),
        ("claude-sonnet-4-5-20250929", 3.0, 15.0, 0.3, 3.75),
        ("claude-fable-5", 10.0, 50.0, 1.0, 12.5),
        ("claude-mythos-5", 10.0, 50.0, 1.0, 12.5),
        ("claude-3-5-sonnet-20241022", 3.0, 15.0, 0.3, 3.75),
        ("claude-haiku-4-5-20251001", 1.0, 5.0, 0.1, 1.25),
        ("claude-haiku-4-5", 1.0, 5.0, 0.1, 1.25),
        ("claude-3-5-haiku-20241022", 0.8, 4.0, 0.08, 1.0),
        ("claude-3-opus-20240229", 15.0, 75.0, 1.5, 18.75),
        ("gpt-5.5", 5.0, 30.0, 0.5, 5.0),
        ("gpt-5.4", 2.5, 15.0, 0.25, 2.5),
        ("gpt-5.4-mini", 0.75, 4.5, 0.075, 0.75),
        ("gpt-5.3-codex", 1.75, 14.0, 0.175, 1.75),
        ("gemini-3.1-pro-preview", 2.0, 12.0, 0.2, 2.0),
        ("gemini-3-pro-preview", 2.0, 12.0, 0.2, 2.0),
        ("gemini-3-flash-preview", 0.5, 3.0, 0.05, 0.5),
        ("gemini-2.5-pro", 1.25, 10.0, 0.125, 1.25),
        ("gemini-2.5-flash", 0.3, 2.5, 0.03, 0.3),
    ]
    .into_iter()
    .map(
        |(model, input, output, cache_read, cache_write)| ModelRateOverride {
            model: model.to_string(),
            input_price: input,
            output_price: output,
            cache_read_price: Some(cache_read),
            cache_write_price: Some(cache_write),
        },
    )
    .collect()
}

pub fn official_model_rate_override_for(model: &str) -> Option<ModelRateOverride> {
    let normalized = normalize_model_id(model);
    official_rate(&normalized, 0).map(|(rate, _, _)| ModelRateOverride {
        model: canonical_official_model_id(&normalized).to_string(),
        input_price: rate.input_per_million,
        output_price: rate.output_per_million,
        cache_read_price: Some(rate.cache_read_per_million),
        cache_write_price: rate.cache_creation_per_million,
    })
}

pub fn normalize_model_id(model: &str) -> String {
    let mut normalized = model
        .trim()
        .to_ascii_lowercase()
        .replace('_', "-")
        .trim_start_matches("anthropic/")
        .trim_start_matches("anthropic.")
        .trim_start_matches("anthropic-")
        .trim_start_matches("openai/")
        .trim_start_matches("openai.")
        .trim_start_matches("openai-")
        .trim_start_matches("google/")
        .trim_start_matches("google.")
        .trim_start_matches("google-")
        .to_string();

    if normalized.starts_with("claude-") {
        for family in ["opus", "sonnet", "haiku"] {
            normalized = normalized.replace(
                &format!("claude-{family}-4."),
                &format!("claude-{family}-4-"),
            );
        }
    }

    normalized
}

fn canonical_official_model_id(model: &str) -> &str {
    match model {
        "fable-5" => "claude-fable-5",
        "mythos-5" => "claude-mythos-5",
        _ => model,
    }
}

fn official_rate(
    model: &str,
    prompt_tokens: i64,
) -> Option<(ModelRate, &'static str, &'static str)> {
    if model.starts_with("claude-opus-4-5")
        || model.starts_with("claude-opus-4-6")
        || model.starts_with("claude-opus-4-7")
    {
        return Some((
            anthropic_rate(5.0, 25.0, 0.5),
            "official:anthropic",
            "priced",
        ));
    }

    if model.starts_with("claude-opus-4") || model.starts_with("claude-3-opus") {
        return Some((
            anthropic_rate(15.0, 75.0, 1.5),
            "official:anthropic",
            "priced",
        ));
    }

    if model.starts_with("claude-sonnet-4-5")
        || model.starts_with("claude-sonnet-4-6")
        || model.starts_with("claude-3-5-sonnet")
    {
        return Some((
            anthropic_rate(3.0, 15.0, 0.3),
            "official:anthropic",
            "priced",
        ));
    }

    if matches!(
        model,
        "claude-fable-5" | "fable-5" | "claude-mythos-5" | "mythos-5"
    ) {
        return Some((
            anthropic_rate(10.0, 50.0, 1.0),
            "official:anthropic",
            "priced",
        ));
    }

    if model.starts_with("claude-haiku-4-5") {
        return Some((
            anthropic_rate(1.0, 5.0, 0.1),
            "official:anthropic",
            "priced",
        ));
    }

    if model.starts_with("claude-3-5-haiku") {
        return Some((
            anthropic_rate(0.8, 4.0, 0.08),
            "official:anthropic",
            "priced",
        ));
    }

    if model.starts_with("gpt-5.4-mini") {
        return Some((
            openai_rate(0.75, 4.5, 0.075, prompt_tokens),
            "official:openai",
            "priced",
        ));
    }

    if model.starts_with("gpt-5.5") {
        return Some((
            openai_rate(5.0, 30.0, 0.5, prompt_tokens),
            "official:openai",
            "priced",
        ));
    }

    if model.starts_with("gpt-5.4") {
        return Some((
            openai_rate(2.5, 15.0, 0.25, prompt_tokens),
            "official:openai",
            "priced",
        ));
    }

    if model.starts_with("gpt-5.3-codex") {
        return Some((
            openai_rate(1.75, 14.0, 0.175, prompt_tokens),
            "official:openai",
            "priced",
        ));
    }

    if model.contains("codex-mini") {
        return Some((basic_rate(0.15, 0.60, 0.0375), "legacy:openai", "priced"));
    }

    if model.contains("o4-mini") {
        return Some((basic_rate(0.55, 2.20, 0.1375), "legacy:openai", "priced"));
    }

    if model == "o3" || model.starts_with("o3-") || model.starts_with("gpt-4") {
        return Some((basic_rate(2.0, 8.0, 0.5), "legacy:openai", "priced"));
    }

    if model.starts_with("gemini-3.1-pro-preview") || model.starts_with("gemini-3-pro-preview") {
        return Some((
            gemini_pro_rate(2.0, 12.0, 0.2, prompt_tokens),
            if model.starts_with("gemini-3-pro-preview") {
                "official:google:legacy_alias"
            } else {
                "official:google"
            },
            if model.starts_with("gemini-3-pro-preview") {
                "legacy_alias"
            } else {
                "priced"
            },
        ));
    }

    if model.starts_with("gemini-3-flash-preview") {
        return Some((basic_rate(0.5, 3.0, 0.05), "official:google", "priced"));
    }

    if model.starts_with("gemini-2.5-pro") {
        return Some((
            gemini_pro_rate(1.25, 10.0, 0.125, prompt_tokens),
            "official:google",
            "priced",
        ));
    }

    if model.starts_with("gemini-2.5-flash") {
        return Some((basic_rate(0.3, 2.5, 0.03), "official:google", "priced"));
    }

    None
}

fn basic_rate(input: f64, output: f64, cache_read: f64) -> ModelRate {
    ModelRate {
        input_per_million: input,
        cache_read_per_million: cache_read,
        output_per_million: output,
        cache_creation_per_million: Some(input),
    }
}

fn anthropic_rate(input: f64, output: f64, cache_read: f64) -> ModelRate {
    ModelRate {
        input_per_million: input,
        cache_read_per_million: cache_read,
        output_per_million: output,
        cache_creation_per_million: Some(input * 1.25),
    }
}

fn openai_rate(input: f64, output: f64, cache_read: f64, prompt_tokens: i64) -> ModelRate {
    if prompt_tokens > 272_000 {
        ModelRate {
            input_per_million: input * 2.0,
            cache_read_per_million: cache_read * 2.0,
            output_per_million: output * 1.5,
            cache_creation_per_million: Some(input * 2.0),
        }
    } else {
        basic_rate(input, output, cache_read)
    }
}

fn gemini_pro_rate(input: f64, output: f64, cache_read: f64, prompt_tokens: i64) -> ModelRate {
    if prompt_tokens > 200_000 {
        ModelRate {
            input_per_million: input * 2.0,
            cache_read_per_million: cache_read * 2.0,
            output_per_million: output * 1.5,
            cache_creation_per_million: Some(input * 2.0),
        }
    } else {
        basic_rate(input, output, cache_read)
    }
}

fn rate_label(rate: ModelRate) -> String {
    format!(
        "{}/{}/{}",
        trim_price(rate.input_per_million),
        trim_price(rate.cache_read_per_million),
        trim_price(rate.output_per_million)
    )
}

fn trim_price(value: f64) -> String {
    let text = format!("{value:.6}");
    text.trim_end_matches('0').trim_end_matches('.').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_cost(model: &str, input: i64, output: i64, cache: i64, expected: f64) {
        let catalog = ModelRateCatalog::official();
        let actual = catalog
            .calculate(model, input, output, cache, 0)
            .cost_with_cache_usd;
        assert!((actual - expected).abs() < 0.000_001, "{model}: {actual}");
    }

    #[test]
    fn calculates_current_official_prices() {
        assert_cost("claude-opus-4-6", 1_000_000, 1_000_000, 1_000_000, 30.5);
        assert_cost("claude-opus-4.7", 1_000_000, 1_000_000, 1_000_000, 30.5);
        assert_cost("claude-haiku-4-5", 1_000_000, 1_000_000, 1_000_000, 6.1);
        assert_cost("claude-fable-5", 1_000_000, 400_000, 200_000, 30.2);
        assert_cost("claude-mythos-5", 1_000_000, 400_000, 200_000, 30.2);
        assert_cost("gpt-5.4", 100_000, 100_000, 100_000, 1.775);
        assert_cost("gpt-5.5", 100_000, 100_000, 100_000, 3.55);
        assert_cost("gpt-5.4-mini", 100_000, 100_000, 100_000, 0.5325);
        assert_cost("gpt-5.3-codex", 100_000, 100_000, 100_000, 1.5925);
        assert_cost(
            "gemini-3-flash-preview",
            1_000_000,
            1_000_000,
            1_000_000,
            3.55,
        );
    }

    #[test]
    fn applies_long_context_tiers_per_record() {
        assert_cost("gpt-5.4", 273_000, 1_000_000, 0, 23.865);
        assert_cost("gemini-3.1-pro-preview", 201_000, 1_000_000, 0, 18.804);
    }

    #[test]
    fn marks_unknown_models_as_unpriced() {
        let actual = ModelRateCatalog::official().calculate("unknown-model", 1, 1, 1, 1);
        assert_eq!(actual.pricing_status, "unpriced");
        assert_eq!(actual.cost_with_cache_usd, 0.0);
    }

    #[test]
    fn prices_claude_fable_and_mythos_aliases_without_substring_matches() {
        let catalog = ModelRateCatalog::official();
        for model in [
            "claude-fable-5",
            "fable-5",
            "anthropic/claude-fable-5",
            "anthropic.claude-fable-5",
            "anthropic-claude-fable-5",
            "claude-mythos-5",
            "mythos-5",
            "anthropic/claude-mythos-5",
            "anthropic.claude-mythos-5",
            "anthropic-claude-mythos-5",
        ] {
            let actual = catalog.calculate(model, 1_000_000, 400_000, 200_000, 300_000);
            assert_eq!(actual.pricing_status, "priced", "{model}");
            assert_eq!(actual.pricing_source, "official:anthropic", "{model}");
            assert_eq!(actual.rate_label.as_deref(), Some("10/1/50"), "{model}");
            assert_eq!(actual.input_rate_per_million, Some(10.0), "{model}");
            assert_eq!(actual.cache_read_rate_per_million, Some(1.0), "{model}");
            assert_eq!(actual.output_rate_per_million, Some(50.0), "{model}");
            assert!(
                (actual.cost_with_cache_usd - 33.95).abs() < 0.000_001,
                "{model}"
            );
            assert!(
                (actual.cost_without_cache_usd - 35.0).abs() < 0.000_001,
                "{model}"
            );
        }

        for model in [
            "not-fable-5",
            "not-mythos-5",
            "claude-mythos-preview",
            "not-anthropic/claude-fable-5",
        ] {
            let actual = catalog.calculate(model, 1, 1, 1, 1);
            assert_eq!(actual.pricing_status, "unpriced", "{model}");
        }
    }

    #[test]
    fn returns_normalized_official_override_for_aliases() {
        let Some(actual) = official_model_rate_override_for("anthropic/claude-opus-4.6") else {
            panic!("official alias should resolve to a model rate");
        };
        assert_eq!(actual.model, "claude-opus-4-6");
        assert_eq!(actual.input_price, 5.0);
        assert_eq!(actual.output_price, 25.0);
        assert_eq!(actual.cache_read_price, Some(0.5));

        let Some(fable) = official_model_rate_override_for("fable-5") else {
            panic!("short fable alias should resolve to a model rate");
        };
        assert_eq!(fable.model, "claude-fable-5");
        assert_eq!(fable.input_price, 10.0);
        assert_eq!(fable.output_price, 50.0);
        assert_eq!(fable.cache_read_price, Some(1.0));
        assert_eq!(fable.cache_write_price, Some(12.5));
    }
}
