//! Fixed instruction-file policy shared by Tauri and GPUI adapters.
//! Explicit paths are supplied by captured adapter roots; no ambient HOME discovery.
use crate::platform_settings::{self, SaveOutcome};
use ccr_core::core::BackupPolicy;
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

const SIZE_WARNING_BYTES: usize = 64 * 1024;
const CODEX_LIMIT_HINT: usize = 32 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptLocation {
    Claude,
    Codex,
    Gemini,
    OpenCode,
}

#[derive(Debug, Clone, Copy)]
pub struct PromptFileSpec {
    pub platform: &'static str,
    pub id: &'static str,
    pub label_key: &'static str,
    pub filename: &'static str,
    pub location: PromptLocation,
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

pub fn normalize_platform(platform: &str) -> Option<&'static str> {
    match platform.trim().to_ascii_lowercase().as_str() {
        "claude" | "claude-code" => Some("claude"),
        "codex" => Some("codex"),
        "gemini" | "gemini-cli" | "antigravity" => Some("gemini"),
        "opencode" => Some("opencode"),
        _ => None,
    }
}

pub fn platform_specs(platform: &str) -> Result<Vec<&'static PromptFileSpec>, String> {
    let normalized = normalize_platform(platform)
        .ok_or_else(|| format!("不支持的系统提示词平台: {platform}"))?;
    Ok(PROMPT_FILES
        .iter()
        .filter(|spec| spec.platform == normalized)
        .collect())
}

pub fn prompt_spec(platform: &str, id: &str) -> Result<&'static PromptFileSpec, String> {
    platform_specs(platform)?
        .into_iter()
        .find(|spec| spec.id == id)
        .ok_or_else(|| format!("平台 {platform} 不支持系统提示词文件: {id}"))
}

/// Default home layout only. Claude callers use [`resolve_with_claude_root`] so
/// `CLAUDE_CONFIG_DIR` is honoured.
pub fn resolve_from_home(spec: &PromptFileSpec, home: &Path) -> PathBuf {
    match spec.location {
        PromptLocation::Claude => home.join(".claude").join(spec.filename),
        PromptLocation::Codex => home.join(".codex").join(spec.filename),
        PromptLocation::Gemini => home.join(".gemini").join(spec.filename),
        PromptLocation::OpenCode => home.join(".config").join("opencode").join(spec.filename),
    }
}

/// Resolve with the adapter's captured Claude configuration root.
pub fn resolve_with_claude_root(spec: &PromptFileSpec, home: &Path, claude: &Path) -> PathBuf {
    if spec.location == PromptLocation::Claude {
        claude.join(spec.filename)
    } else {
        resolve_from_home(spec, home)
    }
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

pub fn file_description(spec: &PromptFileSpec, path: &Path) -> Value {
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

/// Default home layout only; adapters list the captured root with [`list_claude_rules_in`].
pub fn list_claude_rules(home: &Path) -> Vec<Value> {
    list_claude_rules_in(&home.join(".claude"))
}

/// List rule files under the adapter's captured Claude configuration root.
pub fn list_claude_rules_in(claude: &Path) -> Vec<Value> {
    let Ok(entries) = fs::read_dir(claude.join("rules")) else {
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

pub fn augment_file_result(mut result: Value, spec: &PromptFileSpec) -> Value {
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

/// Keep legacy wire feedback identical for adapters with their own error mapping.
pub fn augment_save_result(mut result: Value, spec: &PromptFileSpec, content: &str) -> Value {
    if result["status"] == "saved" {
        if content.len() > SIZE_WARNING_BYTES {
            result["warning"] = json!("size");
        }
        if spec.location == PromptLocation::Codex {
            result["limitHint"] = json!(CODEX_LIMIT_HINT);
        }
    }
    result
}

pub fn save_prompt_file_with_locks(
    spec: &PromptFileSpec,
    path: &Path,
    backups: &Path,
    content: &str,
    token: &str,
    locks: &Path,
) -> Result<Value, String> {
    let outcome = platform_settings::write_versioned(
        path,
        content,
        token,
        BackupPolicy::Dir {
            dir: backups.to_path_buf(),
            prefix: spec.filename.to_owned(),
        },
        false,
        Some(locks),
    )
    .map_err(|_| "prompt_write_failed".to_string())?;
    let result = match outcome {
        SaveOutcome::Saved { token } => json!({"status":"saved", "token":token}),
        SaveOutcome::Conflict => json!({"status":"conflict"}),
    };
    Ok(augment_save_result(result, spec, content))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn captured_claude_root_overrides_only_claude_memory() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let claude = root.path().join("custom-claude");
        assert_eq!(
            resolve_with_claude_root(
                prompt_spec("claude", "claude-user-memory").unwrap(),
                &home,
                &claude
            ),
            claude.join("CLAUDE.md")
        );
        assert_eq!(
            resolve_with_claude_root(
                prompt_spec("codex", "codex-agents").unwrap(),
                &home,
                &claude
            ),
            home.join(".codex/AGENTS.md")
        );
        assert!(!home.exists());
        assert!(!claude.exists());
    }

    #[test]
    fn fixed_registry_missing_reads_and_sorted_rules_do_not_create_targets() {
        let root = tempfile::tempdir().unwrap();
        for (platform, id, relative) in [
            ("claude", "claude-user-memory", ".claude/CLAUDE.md"),
            ("codex", "codex-agents", ".codex/AGENTS.md"),
            ("antigravity", "gemini-md", ".gemini/GEMINI.md"),
            ("opencode", "opencode-agents", ".config/opencode/AGENTS.md"),
        ] {
            let spec = prompt_spec(platform, id).unwrap();
            let path = resolve_from_home(spec, root.path());
            assert!(path.ends_with(relative));
            assert!(!platform_settings::read_text(&path).unwrap().exists);
            assert!(!path.parent().unwrap().exists());
        }
        assert!(prompt_spec("codex", "../outside").is_err());
        assert!(platform_specs("grok").is_err());
        let rules = root.path().join(".claude/rules");
        fs::create_dir_all(&rules).unwrap();
        fs::write(rules.join("z.md"), "z").unwrap();
        fs::write(rules.join("a.md"), "a").unwrap();
        fs::write(rules.join("ignored.txt"), "ignored").unwrap();
        let listed = list_claude_rules(root.path());
        assert_eq!(listed.len(), 2);
        assert_eq!(listed[0]["name"], "a.md");
        assert_eq!(listed[1]["name"], "z.md");
    }

    #[test]
    fn captured_claude_root_rules_ignore_default_home_rules() {
        let root = tempfile::tempdir().unwrap();
        let claude = root.path().join("custom-claude");
        fs::create_dir_all(claude.join("rules")).unwrap();
        fs::create_dir_all(root.path().join(".claude/rules")).unwrap();
        fs::write(claude.join("rules/custom.md"), "custom").unwrap();
        fs::write(root.path().join(".claude/rules/ambient.md"), "ambient").unwrap();
        let listed = list_claude_rules_in(&claude);
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0]["name"], "custom.md");
        assert!(list_claude_rules_in(&root.path().join("missing")).is_empty());
    }

    #[test]
    fn create_save_reread_and_two_original_clients_preserve_winner_and_backup() {
        let root = tempfile::tempdir().unwrap();
        let spec = prompt_spec("codex", "codex-agents").unwrap();
        let path = resolve_from_home(spec, root.path());
        let backup = root.path().join("backups");
        let locks = root.path().join("locks");
        let create = save_prompt_file_with_locks(spec, &path, &backup, "", "", &locks).unwrap();
        assert_eq!(create["status"], "saved");
        assert_eq!(create["limitHint"], 32 * 1024);
        assert!(!backup.exists());
        let original = platform_settings::read_text(&path).unwrap();
        let token = original.token.as_str();
        assert_eq!(
            save_prompt_file_with_locks(spec, &path, &backup, "first", token, &locks).unwrap()["status"],
            "saved"
        );
        assert_eq!(
            save_prompt_file_with_locks(spec, &path, &backup, "second", token, &locks).unwrap()["status"],
            "conflict"
        );
        assert_eq!(
            save_prompt_file_with_locks(spec, &path, &backup, "", "", &locks).unwrap()["status"],
            "conflict"
        );
        assert_eq!(
            platform_settings::read_text(&path).unwrap().content,
            "first"
        );
        assert_eq!(fs::read_dir(&backup).unwrap().count(), 1);
    }

    #[test]
    fn failed_backup_and_invalid_utf8_preserve_data_and_hide_source_payload() {
        let root = tempfile::tempdir().unwrap();
        let spec = prompt_spec("claude", "claude-user-memory").unwrap();
        let path = resolve_from_home(spec, root.path());
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "private-source-sentinel").unwrap();
        let before = platform_settings::read_text(&path).unwrap();
        let blocked = root.path().join("blocked-backup");
        fs::write(&blocked, "not a directory").unwrap();
        let error = save_prompt_file_with_locks(
            spec,
            &path,
            &blocked,
            "replace",
            &before.token,
            &root.path().join("locks"),
        )
        .unwrap_err();
        assert!(!error.contains("sentinel"));
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "private-source-sentinel"
        );
        fs::write(&path, [0xff, 0xfe]).unwrap();
        assert!(platform_settings::read_text(&path).is_err());
    }

    #[test]
    fn size_warning_has_no_plaintext_and_matches_original_wire_keys() {
        let root = tempfile::tempdir().unwrap();
        let spec = prompt_spec("opencode", "opencode-agents").unwrap();
        let path = resolve_from_home(spec, root.path());
        let content = format!(
            "sensitive-prompt-sentinel{}",
            "x".repeat(SIZE_WARNING_BYTES)
        );
        let result = save_prompt_file_with_locks(
            spec,
            &path,
            &root.path().join("backups"),
            &content,
            "",
            &root.path().join("locks"),
        )
        .unwrap();
        assert_eq!(result["status"], "saved");
        assert_eq!(result["warning"], "size");
        assert!(!result.to_string().contains("sentinel"));
        assert_eq!(result.as_object().unwrap().len(), 3);
    }
}
