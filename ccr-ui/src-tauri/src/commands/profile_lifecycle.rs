use serde_json::{Value, json};
use std::fs;
use std::path::Path;

pub(crate) fn resolve_profile_target_name(
    platform_label: &str,
    current_name: &str,
    request: &Value,
) -> Result<String, String> {
    let Some(raw) = request.get("name") else {
        return Ok(current_name.to_string());
    };

    let target = raw
        .as_str()
        .ok_or_else(|| format!("{platform_label} Profile 名称必须是字符串"))?
        .trim();

    if target.is_empty() {
        return Err(format!("{platform_label} Profile 名称不能为空"));
    }

    Ok(target.to_string())
}

pub(crate) fn profiles_export_payload_from_path(
    profiles_file: &Path,
    filename_prefix: &str,
    include_secrets: bool,
) -> Result<Value, String> {
    if !include_secrets {
        return Err("Redacted profiles export is not supported".to_string());
    }

    let content = fs::read_to_string(profiles_file)
        .map_err(|e| format!("Failed to read profiles.toml: {e}"))?;
    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let filename = format!("{filename_prefix}-{timestamp}.toml");

    Ok(json!({
        "content": content,
        "filename": filename,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_name_defaults_to_current_name() {
        let target = resolve_profile_target_name("Codex", "current", &json!({})).unwrap();
        assert_eq!(target, "current");
    }

    #[test]
    fn target_name_rejects_blank_values() {
        let error = resolve_profile_target_name("Claude", "current", &json!({ "name": " " }))
            .unwrap_err();
        assert!(error.contains("不能为空"));
    }
}
