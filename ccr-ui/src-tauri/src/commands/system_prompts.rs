//! Local user-level instruction file management for supported AI CLIs.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use ccr_config::{Platform, PlatformPaths};
use serde_json::{Value, json};
use tauri::State;

use crate::state::AppState;

use super::opencode::opencode_config_dir;
use super::settings_raw::{ensure_local_env, read_raw_file, write_raw_file_versioned};

const SIZE_WARNING_BYTES: usize = 64 * 1024;
const CODEX_LIMIT_HINT: usize = 32 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PromptLocation {
    Claude,
    Codex,
    Gemini,
    OpenCode,
}

#[derive(Debug, Clone, Copy)]
struct PromptFileSpec {
    platform: &'static str,
    id: &'static str,
    label_key: &'static str,
    filename: &'static str,
    location: PromptLocation,
}

const PROMPT_FILES: &[PromptFileSpec] = &[
    PromptFileSpec {
        platform: "claude",
        id: "claude-user-memory",
        label_key: "systemPrompts.files.claudeUserMemory",
        filename: "CLAUDE.md",
        location: PromptLocation::Claude,
    },
    PromptFileSpec {
        platform: "codex",
        id: "codex-agents",
        label_key: "systemPrompts.files.codexAgents",
        filename: "AGENTS.md",
        location: PromptLocation::Codex,
    },
    PromptFileSpec {
        platform: "gemini",
        id: "gemini-md",
        label_key: "systemPrompts.files.geminiMd",
        filename: "GEMINI.md",
        location: PromptLocation::Gemini,
    },
    PromptFileSpec {
        platform: "opencode",
        id: "opencode-agents",
        label_key: "systemPrompts.files.opencodeAgents",
        filename: "AGENTS.md",
        location: PromptLocation::OpenCode,
    },
];

fn normalize_platform(platform: &str) -> Option<&'static str> {
    match platform.trim().to_ascii_lowercase().as_str() {
        "claude" | "claude-code" => Some("claude"),
        "codex" => Some("codex"),
        "gemini" | "gemini-cli" | "antigravity" => Some("gemini"),
        "opencode" => Some("opencode"),
        _ => None,
    }
}

fn platform_specs(platform: &str) -> Result<Vec<&'static PromptFileSpec>, String> {
    let normalized = normalize_platform(platform)
        .ok_or_else(|| format!("不支持的系统提示词平台: {platform}"))?;
    Ok(PROMPT_FILES
        .iter()
        .filter(|spec| spec.platform == normalized)
        .collect())
}

fn prompt_spec(platform: &str, id: &str) -> Result<&'static PromptFileSpec, String> {
    platform_specs(platform)?
        .into_iter()
        .find(|spec| spec.id == id)
        .ok_or_else(|| format!("平台 {platform} 不支持系统提示词文件: {id}"))
}

fn resolve_from_home(spec: &PromptFileSpec, home: &Path) -> PathBuf {
    match spec.location {
        PromptLocation::Claude => home.join(".claude").join(spec.filename),
        PromptLocation::Codex => home.join(".codex").join(spec.filename),
        PromptLocation::Gemini => home.join(".gemini").join(spec.filename),
        PromptLocation::OpenCode => home.join(".config").join("opencode").join(spec.filename),
    }
}

fn resolve_prompt_path(spec: &PromptFileSpec) -> Result<PathBuf, String> {
    if spec.location == PromptLocation::OpenCode {
        return Ok(opencode_config_dir()?.join(spec.filename));
    }
    let home = dirs::home_dir().ok_or_else(|| "无法获取用户主目录".to_string())?;
    Ok(resolve_from_home(spec, &home))
}

fn backup_dir(spec: &PromptFileSpec) -> Result<PathBuf, String> {
    let platform = match spec.location {
        PromptLocation::Claude => Platform::Claude,
        PromptLocation::Codex => Platform::Codex,
        PromptLocation::Gemini => Platform::Gemini,
        PromptLocation::OpenCode => {
            return PlatformPaths::new(Platform::Claude)
                .map(|paths| paths.root.join("backups").join("opencode"))
                .map_err(|error| format!("解析 OpenCode 备份目录失败: {error}"));
        }
    };
    PlatformPaths::new(platform)
        .map(|paths| paths.backups_dir)
        .map_err(|error| format!("解析 {} 备份目录失败: {error}", platform.display_name()))
}

fn timestamp_millis(path: &Path) -> Option<u64> {
    fs::metadata(path)
        .ok()?
        .modified()
        .ok()?
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|value| value.as_millis() as u64)
}

fn file_description(spec: &PromptFileSpec, path: &Path) -> Value {
    let metadata = fs::metadata(path).ok();
    json!({
        "id": spec.id,
        "labelKey": spec.label_key,
        "path": path,
        "exists": metadata.is_some(),
        "size": metadata.as_ref().map(std::fs::Metadata::len),
        "mtime": timestamp_millis(path),
        "editable": true,
        "limitHint": (spec.location == PromptLocation::Codex).then_some(CODEX_LIMIT_HINT),
    })
}

fn list_claude_rules(home: &Path) -> Vec<Value> {
    let rules_dir = home.join(".claude").join("rules");
    let Ok(entries) = fs::read_dir(rules_dir) else {
        return Vec::new();
    };
    let mut rules: Vec<_> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
        })
        .map(|path| {
            let metadata = fs::metadata(&path).ok();
            json!({
                "name": path.file_name().and_then(|name| name.to_str()).unwrap_or_default(),
                "path": path,
                "size": metadata.as_ref().map(std::fs::Metadata::len),
            })
        })
        .collect();
    rules.sort_by(|left, right| left["name"].as_str().cmp(&right["name"].as_str()));
    rules
}

#[cfg(test)]
fn list_for_home(platform: &str, home: &Path) -> Result<Value, String> {
    let specs = platform_specs(platform)?;
    let files: Vec<_> = specs
        .iter()
        .map(|spec| file_description(spec, &resolve_from_home(spec, home)))
        .collect();
    let rules = if normalize_platform(platform) == Some("claude") {
        list_claude_rules(home)
    } else {
        Vec::new()
    };
    Ok(json!({ "status": "ok", "files": files, "rules": rules }))
}

fn augment_file_result(mut result: Value, spec: &PromptFileSpec) -> Value {
    if let Some(object) = result.as_object_mut() {
        object.insert(
            "limitHint".to_string(),
            (spec.location == PromptLocation::Codex)
                .then_some(json!(CODEX_LIMIT_HINT))
                .unwrap_or(Value::Null),
        );
    }
    result
}

fn save_prompt_file(
    spec: &PromptFileSpec,
    path: &Path,
    backup_dir: &Path,
    content: &str,
    token: &str,
) -> Result<Value, String> {
    let mut result =
        write_raw_file_versioned(path, backup_dir, spec.filename, content, token, false)?;
    if result["status"] == "saved" {
        if content.len() > SIZE_WARNING_BYTES {
            result["warning"] = json!("size");
        }
        if spec.location == PromptLocation::Codex {
            result["limitHint"] = json!(CODEX_LIMIT_HINT);
        }
    }
    Ok(result)
}

#[tauri::command]
pub async fn system_prompts_list(
    state: State<'_, AppState>,
    platform: String,
) -> Result<Value, String> {
    if let Some(response) = ensure_local_env(state.inner()).await {
        return Ok(response);
    }
    let normalized = normalize_platform(&platform)
        .ok_or_else(|| format!("不支持的系统提示词平台: {platform}"))?;
    let specs = platform_specs(normalized)?;
    let resolved: Vec<_> = specs
        .iter()
        .map(|spec| resolve_prompt_path(spec).map(|path| (*spec, path)))
        .collect::<Result<_, _>>()?;
    let home = dirs::home_dir().ok_or_else(|| "无法获取用户主目录".to_string())?;
    tokio::task::spawn_blocking(move || {
        let files: Vec<_> = resolved
            .iter()
            .map(|(spec, path)| file_description(spec, path))
            .collect();
        let rules = if normalized == "claude" {
            list_claude_rules(&home)
        } else {
            Vec::new()
        };
        Ok(json!({ "status": "ok", "files": files, "rules": rules }))
    })
    .await
    .map_err(|error| format!("列出系统提示词后台任务失败: {error}"))?
}

#[tauri::command]
pub async fn system_prompts_get(
    state: State<'_, AppState>,
    platform: String,
    id: String,
) -> Result<Value, String> {
    if let Some(response) = ensure_local_env(state.inner()).await {
        return Ok(response);
    }
    let spec = prompt_spec(&platform, &id)?;
    let path = resolve_prompt_path(spec)?;
    tokio::task::spawn_blocking(move || {
        read_raw_file(&path).map(|value| augment_file_result(value, spec))
    })
    .await
    .map_err(|error| format!("读取系统提示词后台任务失败: {error}"))?
}

#[tauri::command]
pub async fn system_prompts_save(
    state: State<'_, AppState>,
    platform: String,
    id: String,
    content: String,
    token: String,
) -> Result<Value, String> {
    if let Some(response) = ensure_local_env(state.inner()).await {
        return Ok(response);
    }
    let spec = prompt_spec(&platform, &id)?;
    let path = resolve_prompt_path(spec)?;
    let backups = backup_dir(spec)?;
    tokio::task::spawn_blocking(move || save_prompt_file(spec, &path, &backups, &content, &token))
        .await
        .map_err(|error| format!("保存系统提示词后台任务失败: {error}"))?
}

#[tauri::command]
pub async fn system_prompts_create(
    state: State<'_, AppState>,
    platform: String,
    id: String,
) -> Result<Value, String> {
    if let Some(response) = ensure_local_env(state.inner()).await {
        return Ok(response);
    }
    let spec = prompt_spec(&platform, &id)?;
    let path = resolve_prompt_path(spec)?;
    let backups = backup_dir(spec)?;
    tokio::task::spawn_blocking(move || save_prompt_file(spec, &path, &backups, "", ""))
        .await
        .map_err(|error| format!("创建系统提示词后台任务失败: {error}"))?
}

#[cfg(test)]
mod tests {
    use std::fs;

    use ccr_core::core::content_version_token;
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn registry_resolves_all_supported_home_paths() {
        let home = Path::new("/home/tester");
        let cases = [
            ("claude", "claude-user-memory", ".claude/CLAUDE.md"),
            ("codex", "codex-agents", ".codex/AGENTS.md"),
            ("antigravity", "gemini-md", ".gemini/GEMINI.md"),
            ("opencode", "opencode-agents", ".config/opencode/AGENTS.md"),
        ];
        for (platform, id, suffix) in cases {
            let spec = prompt_spec(platform, id).unwrap();
            assert!(resolve_from_home(spec, home).ends_with(suffix));
        }
    }

    #[test]
    fn list_reports_missing_file_and_sorted_claude_rules() {
        let temp_dir = tempdir().unwrap();
        let rules = temp_dir.path().join(".claude/rules");
        fs::create_dir_all(&rules).unwrap();
        fs::write(rules.join("z.md"), "z").unwrap();
        fs::write(rules.join("a.md"), "a").unwrap();

        let result = list_for_home("claude", temp_dir.path()).unwrap();

        assert_eq!(result["files"][0]["exists"], false);
        assert_eq!(result["rules"][0]["name"], "a.md");
        assert_eq!(result["rules"][1]["name"], "z.md");
    }

    #[test]
    fn create_and_save_use_version_tokens_and_backup() {
        let temp_dir = tempdir().unwrap();
        let target = temp_dir.path().join(".codex/AGENTS.md");
        let backups = temp_dir.path().join("backups/codex");
        let spec = prompt_spec("codex", "codex-agents").unwrap();

        let created = save_prompt_file(spec, &target, &backups, "", "").unwrap();
        assert_eq!(created["status"], "saved");
        assert_eq!(created["limitHint"], CODEX_LIMIT_HINT);

        let current_token = content_version_token(b"");
        let saved = save_prompt_file(spec, &target, &backups, "# Rules\n", &current_token).unwrap();
        assert_eq!(saved["status"], "saved");
        assert_eq!(fs::read_to_string(&target).unwrap(), "# Rules\n");
        assert_eq!(fs::read_dir(&backups).unwrap().count(), 1);
    }

    #[test]
    fn stale_save_and_duplicate_create_preserve_existing_content() {
        let temp_dir = tempdir().unwrap();
        let target = temp_dir.path().join(".claude/CLAUDE.md");
        let backups = temp_dir.path().join("backups/claude");
        let spec = prompt_spec("claude", "claude-user-memory").unwrap();
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(&target, "external change").unwrap();

        let stale = save_prompt_file(spec, &target, &backups, "probe", "stale").unwrap();
        let duplicate = save_prompt_file(spec, &target, &backups, "", "").unwrap();

        assert_eq!(stale["status"], "conflict");
        assert_eq!(duplicate["status"], "conflict");
        assert_eq!(fs::read_to_string(&target).unwrap(), "external change");
        assert!(!backups.exists());
    }

    #[test]
    fn oversized_content_warns_without_leaking_probe() {
        let temp_dir = tempdir().unwrap();
        let target = temp_dir.path().join(".config/opencode/AGENTS.md");
        let backups = temp_dir.path().join(".ccr/backups/opencode");
        let spec = prompt_spec("opencode", "opencode-agents").unwrap();
        let probe = "DO_NOT_LEAK_PROBE";
        let content = format!("{probe}{}", "x".repeat(SIZE_WARNING_BYTES));

        let result = save_prompt_file(spec, &target, &backups, &content, "").unwrap();

        assert_eq!(result["status"], "saved");
        assert_eq!(result["warning"], "size");
        assert!(!result.to_string().contains(probe));
        assert!(target.exists());
        assert!(!backups.exists(), "first creation must not create a backup");
    }
}
