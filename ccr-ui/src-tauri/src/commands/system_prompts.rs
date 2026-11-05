//! Local user-level instruction file management for supported AI CLIs.

use std::path::{Path, PathBuf};

use ccr_app::resources::system_prompts::{
    PromptFileSpec, PromptLocation, augment_file_result, augment_save_result, file_description,
    list_claude_rules, normalize_platform, platform_specs, prompt_spec, resolve_from_home,
};
use ccr_config::{Platform, PlatformPaths};
use serde_json::{Value, json};
use tauri::State;

use crate::state::AppState;

use super::opencode::opencode_config_dir;
use super::settings_raw::{ensure_local_env, read_raw_file, write_raw_file_versioned};
use super::wire::OpenJsonValueDto;

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

fn save_prompt_file(
    spec: &PromptFileSpec,
    path: &Path,
    backup_dir: &Path,
    content: &str,
    token: &str,
) -> Result<Value, String> {
    let result = write_raw_file_versioned(path, backup_dir, spec.filename, content, token, false)?;
    Ok(augment_save_result(result, spec, content))
}

#[ccr_tauri_command_macros::command]
pub async fn system_prompts_list(
    state: State<'_, AppState>,
    platform: String,
) -> Result<OpenJsonValueDto, String> {
    if let Some(response) = ensure_local_env(state.inner()).await {
        return response.try_into();
    }
    let normalized = normalize_platform(&platform)
        .ok_or_else(|| format!("不支持的系统提示词平台: {platform}"))?;
    let specs = platform_specs(normalized)?;
    let resolved: Vec<_> = specs
        .iter()
        .map(|spec| resolve_prompt_path(spec).map(|path| (*spec, path)))
        .collect::<Result<_, _>>()?;
    let home = dirs::home_dir().ok_or_else(|| "无法获取用户主目录".to_string())?;
    tokio::task::spawn_blocking(move || -> Result<Value, String> {
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
    .map_err(|error| format!("列出系统提示词后台任务失败: {error}"))??
    .try_into()
}

#[ccr_tauri_command_macros::command]
pub async fn system_prompts_get(
    state: State<'_, AppState>,
    platform: String,
    id: String,
) -> Result<OpenJsonValueDto, String> {
    if let Some(response) = ensure_local_env(state.inner()).await {
        return response.try_into();
    }
    let spec = prompt_spec(&platform, &id)?;
    let path = resolve_prompt_path(spec)?;
    tokio::task::spawn_blocking(move || {
        read_raw_file(&path).map(|value| augment_file_result(value, spec))
    })
    .await
    .map_err(|error| format!("读取系统提示词后台任务失败: {error}"))??
    .try_into()
}

#[ccr_tauri_command_macros::command]
pub async fn system_prompts_save(
    state: State<'_, AppState>,
    platform: String,
    id: String,
    content: String,
    token: String,
) -> Result<OpenJsonValueDto, String> {
    if let Some(response) = ensure_local_env(state.inner()).await {
        return response.try_into();
    }
    let spec = prompt_spec(&platform, &id)?;
    let path = resolve_prompt_path(spec)?;
    let backups = backup_dir(spec)?;
    tokio::task::spawn_blocking(move || save_prompt_file(spec, &path, &backups, &content, &token))
        .await
        .map_err(|error| format!("保存系统提示词后台任务失败: {error}"))??
        .try_into()
}

#[ccr_tauri_command_macros::command]
pub async fn system_prompts_create(
    state: State<'_, AppState>,
    platform: String,
    id: String,
) -> Result<OpenJsonValueDto, String> {
    if let Some(response) = ensure_local_env(state.inner()).await {
        return response.try_into();
    }
    let spec = prompt_spec(&platform, &id)?;
    let path = resolve_prompt_path(spec)?;
    let backups = backup_dir(spec)?;
    tokio::task::spawn_blocking(move || save_prompt_file(spec, &path, &backups, "", ""))
        .await
        .map_err(|error| format!("创建系统提示词后台任务失败: {error}"))??
        .try_into()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use ccr_core::core::content_version_token;
    use tempfile::tempdir;

    use super::*;

    const SIZE_WARNING_BYTES: usize = 64 * 1024;
    const CODEX_LIMIT_HINT: usize = 32 * 1024;

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
