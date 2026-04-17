//! Phase 4 — skills_ext Enable/Disable Toggle 集成测试。

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

use serde_json::{Value, json};
use tempfile::TempDir;

use ccr_skills::skills_ext::toggle::{ToggleError, ToggleStore};

fn settings_path(tmp: &TempDir) -> PathBuf {
    tmp.path().join("settings.json")
}

fn open_store(tmp: &TempDir) -> ToggleStore {
    ToggleStore::with_path(settings_path(tmp))
}

fn read_settings(tmp: &TempDir) -> Value {
    let text = fs::read_to_string(settings_path(tmp)).expect("读 settings");
    serde_json::from_str(&text).expect("settings 必须是合法 JSON")
}

fn deny_list(settings: &Value) -> Vec<String> {
    settings
        .get("permissions")
        .and_then(|p| p.get("deny"))
        .and_then(|d| d.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn test_toggle_disable_adds_skill_rule_preserves_others() {
    let tmp = TempDir::new().expect("tempdir");
    // 预置：已有 Bash / MCP / 其他 Skill 规则
    let initial = json!({
        "permissions": {
            "deny": [
                "Bash(rm -rf *)",
                "MCP(filesystem:write)",
                "Skill(existing-skill)"
            ]
        },
        "theme": "dark"
    });
    fs::write(
        settings_path(&tmp),
        serde_json::to_string_pretty(&initial).expect("ser"),
    )
    .expect("write initial");

    let store = open_store(&tmp);
    store
        .set_enabled("new-skill", false)
        .expect("disable new-skill");

    let after = read_settings(&tmp);
    let deny = deny_list(&after);

    assert!(
        deny.contains(&"Bash(rm -rf *)".to_string()),
        "Bash 规则必须保留"
    );
    assert!(
        deny.contains(&"MCP(filesystem:write)".to_string()),
        "MCP 规则必须保留"
    );
    assert!(
        deny.contains(&"Skill(existing-skill)".to_string()),
        "原 Skill 规则必须保留"
    );
    assert!(
        deny.contains(&"Skill(new-skill)".to_string()),
        "新禁用必须追加 Skill(new-skill)"
    );
    assert_eq!(deny.len(), 4);

    // theme 等其他顶层 key 必须保留
    assert_eq!(after.get("theme"), Some(&json!("dark")));
}

#[test]
fn test_toggle_enable_removes_target_rule_only() {
    let tmp = TempDir::new().expect("tempdir");
    let initial = json!({
        "permissions": {
            "deny": [
                "Bash(curl)",
                "Skill(remove-me)",
                "Skill(keep-me)"
            ]
        }
    });
    fs::write(
        settings_path(&tmp),
        serde_json::to_string_pretty(&initial).expect("ser"),
    )
    .expect("write");

    let store = open_store(&tmp);
    store.set_enabled("remove-me", true).expect("enable");

    let after = read_settings(&tmp);
    let deny = deny_list(&after);

    assert!(!deny.contains(&"Skill(remove-me)".to_string()));
    assert!(deny.contains(&"Skill(keep-me)".to_string()));
    assert!(deny.contains(&"Bash(curl)".to_string()));
    assert_eq!(deny.len(), 2);
}

#[test]
fn test_idempotent_disable_does_not_duplicate() {
    let tmp = TempDir::new().expect("tempdir");
    let store = open_store(&tmp);

    store.set_enabled("x", false).expect("disable 1");
    store.set_enabled("x", false).expect("disable 2");
    store.set_enabled("x", false).expect("disable 3");

    let deny = deny_list(&read_settings(&tmp));
    let count = deny.iter().filter(|s| s == &"Skill(x)").count();
    assert_eq!(count, 1, "禁用幂等：Skill(x) 只出现一次");
}

#[test]
fn test_idempotent_enable_when_not_disabled() {
    let tmp = TempDir::new().expect("tempdir");
    let store = open_store(&tmp);

    // 从未禁用过就 enable 不应报错，也不应写出奇怪文件
    store.set_enabled("y", true).expect("enable fresh");

    assert!(
        store.is_enabled("y").expect("is_enabled"),
        "未禁用即视为启用"
    );
}

#[test]
fn test_missing_settings_file_is_created_fresh() {
    let tmp = TempDir::new().expect("tempdir");
    assert!(!settings_path(&tmp).exists());

    let store = open_store(&tmp);
    store.set_enabled("fresh", false).expect("disable");

    assert!(settings_path(&tmp).exists(), "settings.json 应被创建");
    let deny = deny_list(&read_settings(&tmp));
    assert_eq!(deny, vec!["Skill(fresh)"]);
}

#[test]
fn test_missing_permissions_object_is_created() {
    let tmp = TempDir::new().expect("tempdir");
    // 有 settings.json 但无 permissions 字段
    fs::write(
        settings_path(&tmp),
        serde_json::to_string_pretty(&json!({"theme":"light"})).expect("ser"),
    )
    .expect("write");

    let store = open_store(&tmp);
    store.set_enabled("late", false).expect("disable");

    let after = read_settings(&tmp);
    assert_eq!(after.get("theme"), Some(&json!("light")));
    assert_eq!(deny_list(&after), vec!["Skill(late)"]);
}

#[test]
fn test_list_disabled_returns_only_skill_rules() {
    let tmp = TempDir::new().expect("tempdir");
    let initial = json!({
        "permissions": {
            "deny": [
                "Bash(rm)",
                "Skill(a)",
                "MCP(foo)",
                "Skill(b-c_1)",
                "Skill(bad rule)",
                "NotSkill(x)"
            ]
        }
    });
    fs::write(
        settings_path(&tmp),
        serde_json::to_string_pretty(&initial).expect("ser"),
    )
    .expect("write");

    let store = open_store(&tmp);
    let disabled = store.list_disabled().expect("list");

    assert_eq!(disabled, vec!["a".to_string(), "b-c_1".to_string()]);
}

#[test]
fn test_invalid_skill_name_rejected() {
    let tmp = TempDir::new().expect("tempdir");
    let store = open_store(&tmp);

    for bad in &["", "has space", "paren(x)", "中文", "a/b"] {
        match store.set_enabled(bad, false) {
            Err(ToggleError::InvalidSkillName(n)) => assert_eq!(n, *bad),
            other => panic!("期望 InvalidSkillName('{bad}')，实际 {other:?}"),
        }
    }
}

#[test]
fn test_concurrent_toggles_keep_json_valid() {
    let tmp = TempDir::new().expect("tempdir");
    let store = Arc::new(open_store(&tmp));

    let mut handles = Vec::new();
    for i in 0..8 {
        let s = Arc::clone(&store);
        handles.push(thread::spawn(move || {
            let name = format!("p{i}");
            for _ in 0..5 {
                s.set_enabled(&name, false).expect("disable");
                s.set_enabled(&name, true).expect("enable");
            }
        }));
    }
    for h in handles {
        h.join().expect("thread panic");
    }

    // 无论并发如何，最终文件必须是合法 JSON；且因为每个线程最后一步是 enable，
    // 所以所有 p0..p7 的 deny 条目都应被清理。
    let final_settings = read_settings(&tmp);
    let deny = deny_list(&final_settings);
    for i in 0..8 {
        let rule = format!("Skill(p{i})");
        assert!(!deny.contains(&rule), "p{i} 应已启用，当前 deny: {deny:?}");
    }
}

#[test]
fn test_empty_settings_file_handled() {
    let tmp = TempDir::new().expect("tempdir");
    fs::write(settings_path(&tmp), "").expect("write empty");

    let store = open_store(&tmp);
    store.set_enabled("e", false).expect("disable after empty");
    assert_eq!(deny_list(&read_settings(&tmp)), vec!["Skill(e)"]);
}
