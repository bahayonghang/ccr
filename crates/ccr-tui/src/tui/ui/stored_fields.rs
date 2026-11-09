//! Read-only projection of one cached profile. No file or catalog access.
use ratatui::{
    style::Style,
    text::{Line, Span},
};
use unicode_width::UnicodeWidthStr;

use super::detail_layout::wrap_lines;
use super::{pad_text, theme, truncate_text};
use ccr_cli::models::{Platform, ProfileConfig};
use ccr_cli::platforms::GrokPlatform;

const SECRET_MARKER: &str = "‹已设置›";

pub(super) fn stored_fields_heading(path: Option<&str>) -> String {
    match path.map(str::trim).filter(|path| !path.is_empty()) {
        Some(path) => crate::tui_format!("Stored fields · {path}", "存储字段 · {path}"),
        None => crate::tui_text!("Stored fields", "存储字段").to_string(),
    }
}

pub(super) fn stored_field_lines(
    platform: Platform,
    config: &ProfileConfig,
    width: usize,
) -> Vec<Line<'static>> {
    let rows = stored_rows(platform, config);
    if rows.is_empty() {
        return vec![Line::styled(
            crate::tui_text!("Nothing stored", "没有已存储字段"),
            theme::muted_style(),
        )];
    }
    if width == 0 {
        return Vec::new();
    }
    let key_width = rows
        .iter()
        .map(|(key, _, _)| key.width())
        .max()
        .unwrap_or(1)
        .min(width.saturating_sub(4).max(1));
    let mut lines = Vec::new();
    for (key, value, style) in rows {
        let key_text = pad_text(&truncate_text(&key, key_width), key_width);
        let prefix = format!("{key_text} = ");
        let prefix_width = prefix.width();
        if prefix_width >= width {
            lines.push(Line::from(Span::styled(
                truncate_text(&prefix, width),
                theme::secondary_text_style(),
            )));
            lines.extend(wrap_lines(
                vec![Line::from(Span::styled(value, style))],
                width,
            ));
            continue;
        }
        let wrapped = wrap_lines(
            vec![Line::from(Span::styled(value, style))],
            width - prefix_width,
        );
        if wrapped.is_empty() {
            lines.push(Line::from(Span::styled(
                prefix,
                theme::secondary_text_style(),
            )));
            continue;
        }
        for (index, value_line) in wrapped.into_iter().enumerate() {
            let mut spans = vec![Span::styled(
                if index == 0 {
                    prefix.clone()
                } else {
                    " ".repeat(prefix_width)
                },
                theme::secondary_text_style(),
            )];
            spans.extend(value_line.spans);
            lines.push(Line::from(spans));
        }
    }
    lines
}

fn stored_rows(platform: Platform, config: &ProfileConfig) -> Vec<(String, String, Style)> {
    let mut rows = Vec::new();
    push_text(&mut rows, "description", config.description.as_deref());
    push_text(
        &mut rows,
        "base_url",
        config
            .base_url
            .as_deref()
            .map(GrokPlatform::safe_base_url_for_display)
            .as_deref(),
    );
    push_text(&mut rows, "model", config.model.as_deref());
    push_text(
        &mut rows,
        "small_fast_model",
        config.small_fast_model.as_deref(),
    );
    push_text(
        &mut rows,
        "default_opus_model",
        config.default_opus_model.as_deref(),
    );
    push_text(
        &mut rows,
        "default_sonnet_model",
        config.default_sonnet_model.as_deref(),
    );
    push_text(
        &mut rows,
        "default_haiku_model",
        config.default_haiku_model.as_deref(),
    );
    push_text(
        &mut rows,
        "default_fable_model",
        config.default_fable_model.as_deref(),
    );
    push_text(
        &mut rows,
        "default_opus_model_name",
        config.default_opus_model_name.as_deref(),
    );
    push_text(
        &mut rows,
        "default_sonnet_model_name",
        config.default_sonnet_model_name.as_deref(),
    );
    push_text(
        &mut rows,
        "default_haiku_model_name",
        config.default_haiku_model_name.as_deref(),
    );
    push_text(
        &mut rows,
        "default_fable_model_name",
        config.default_fable_model_name.as_deref(),
    );
    push_text(
        &mut rows,
        "subagent_model",
        config.subagent_model.as_deref(),
    );
    push_text(
        &mut rows,
        "custom_model_option",
        config.custom_model_option.as_deref(),
    );
    push_text(
        &mut rows,
        "custom_model_option_name",
        config.custom_model_option_name.as_deref(),
    );
    push_text(&mut rows, "effort_level", config.effort_level.as_deref());
    push_text(
        &mut rows,
        "claude_code_auto_compact_window",
        config.claude_code_auto_compact_window.as_deref(),
    );
    push_text(
        &mut rows,
        "api_timeout_ms",
        config.api_timeout_ms.as_deref(),
    );
    push_text(
        &mut rows,
        "claude_code_disable_nonessential_traffic",
        config.claude_code_disable_nonessential_traffic.as_deref(),
    );
    push_text(&mut rows, "provider", config.provider.as_deref());
    push_text(&mut rows, "provider_type", config.provider_type.as_deref());
    push_text(&mut rows, "account", config.account.as_deref());
    if let Some(tags) = config
        .tags
        .as_ref()
        .filter(|tags| tags.iter().any(|tag| !tag.trim().is_empty()))
        && let Ok(rendered) = serde_json::to_string(tags)
    {
        rows.push(("tags".to_string(), rendered, theme::primary_text_style()));
    }
    if let Some(count) = config.usage_count {
        rows.push((
            "usage_count".to_string(),
            count.to_string(),
            theme::primary_text_style(),
        ));
    }
    if let Some(enabled) = config.enabled {
        rows.push((
            "enabled".to_string(),
            enabled.to_string(),
            theme::primary_text_style(),
        ));
    }
    if platform != Platform::Grok && auth_token_is_set(config) {
        rows.push((
            "auth_token".to_string(),
            SECRET_MARKER.to_string(),
            theme::muted_style(),
        ));
    }
    for (key, value) in &config.platform_data {
        push_platform_value(&mut rows, platform, key, value);
    }
    rows
}

fn push_text(rows: &mut Vec<(String, String, Style)>, key: &str, value: Option<&str>) {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return;
    };
    rows.push((
        key.to_string(),
        serde_json::Value::String(value.to_string()).to_string(),
        theme::primary_text_style(),
    ));
}

fn push_platform_value(
    rows: &mut Vec<(String, String, Style)>,
    platform: Platform,
    key: &str,
    value: &serde_json::Value,
) {
    if is_secret_key(key) {
        if platform == Platform::Grok || !json_present(value) {
            return;
        }
        rows.push((
            key.to_string(),
            SECRET_MARKER.to_string(),
            theme::muted_style(),
        ));
        return;
    }
    if key == "env_key" {
        match value {
            serde_json::Value::String(name) if !name.trim().is_empty() => {
                push_text(rows, key, Some(name));
            }
            serde_json::Value::Null => {}
            _ => rows.push((
                key.to_string(),
                crate::tui_text!("invalid", "无效").to_string(),
                theme::warning_style(),
            )),
        }
        return;
    }
    let Some(rendered) = format_json(value) else {
        return;
    };
    rows.push((key.to_string(), rendered, theme::primary_text_style()));
}

fn format_json(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::Null => None,
        serde_json::Value::String(text) if text.trim().is_empty() => None,
        other => Some(other.to_string()),
    }
}

fn json_present(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Null => false,
        serde_json::Value::String(text) => !text.trim().is_empty(),
        _ => true,
    }
}

fn auth_token_is_set(config: &ProfileConfig) -> bool {
    config
        .auth_token
        .as_ref()
        .is_some_and(|secret| !secret.expose().trim().is_empty())
}

fn is_secret_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    key == "api_key"
        || key.ends_with("_api_key")
        || key == "auth_token"
        || key == "token"
        || key.ends_with("_token")
        || key.ends_with("-token")
}

#[cfg(test)]
mod tests {
    use super::*;
    use ccr_cli::models::Platform;
    use serde_json::json;

    fn line_text(lines: &[Line<'_>]) -> String {
        lines
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|span| span.content.as_ref())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn config_with_sentinel() -> ProfileConfig {
        let mut config = ProfileConfig::new();
        config.model = Some("grok-4.6".into());
        config.base_url = Some(
            "https://user:secret-password@example.invalid/v1?token=sentinel-query#frag".into(),
        );
        config.auth_token = Some(ccr_core::Secret::new("SENTINEL-TOKEN-do-not-render"));
        config
            .platform_data
            .insert("zeta".into(), json!("last-by-insertion"));
        config
            .platform_data
            .insert("api_key".into(), json!("SENTINEL-TOKEN-do-not-render"));
        config
            .platform_data
            .insert("env_key".into(), json!("EXAMPLE_API_KEY"));
        config.platform_data.insert("alpha".into(), json!(true));
        config
    }

    #[test]
    fn secrets_are_marked_or_omitted_and_platform_data_keeps_insertion_order() {
        let config = config_with_sentinel();
        let claude = line_text(&stored_field_lines(Platform::Claude, &config, 80));
        assert!(claude.contains("auth_token"));
        assert!(claude.contains(SECRET_MARKER));
        assert!(claude.contains("api_key"));
        assert!(!claude.contains("SENTINEL-TOKEN-do-not-render"));
        assert!(!claude.contains("secret-password"));
        assert!(!claude.contains("sentinel-query"));
        assert!(claude.contains("https://example.invalid/v1"));
        assert!(claude.contains("EXAMPLE_API_KEY"));
        let zeta = claude.find("zeta").expect("zeta field");
        let alpha = claude.find("alpha").expect("alpha field");
        assert!(zeta < alpha, "{claude}");

        let grok = line_text(&stored_field_lines(Platform::Grok, &config, 80));
        assert!(!grok.contains("SENTINEL-TOKEN-do-not-render"));
        assert!(!grok.contains("api_key"), "{grok}");
        assert!(!grok.contains("auth_token"), "{grok}");
        assert!(!grok.contains(SECRET_MARKER), "{grok}");
        assert!(grok.contains("EXAMPLE_API_KEY"));
        assert!(grok.lines().all(|line| line.width() <= 80));
    }

    #[test]
    fn heading_uses_the_captured_path_without_resolving_one() {
        assert_eq!(
            stored_fields_heading(Some(" D:/fixture/grok/profiles.toml ")),
            "Stored fields · D:/fixture/grok/profiles.toml"
        );
        assert_eq!(stored_fields_heading(None), "Stored fields");
    }
}
