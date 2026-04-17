//! Phase 7 — skills_ext 扫描增强：插件感知 + 冲突检测 + 健康诊断 集成测试。

use std::fs;

use tempfile::TempDir;

use ccr_skills::skills_ext::conflicts::detect_conflicts;
use ccr_skills::skills_ext::health::compute as compute_health;
use ccr_skills::skills_ext::plugins::{enabled_plugin_install_locations, find_plugin_skills_dirs};

// ============= plugins.rs =============

#[test]
fn test_plugin_locations_missing_config_returns_empty() {
    let tmp = TempDir::new().expect("tempdir");
    let locs = enabled_plugin_install_locations(tmp.path());
    assert!(locs.is_empty());
}

#[test]
fn test_plugin_locations_reads_enabled_repositories_only() {
    let tmp = TempDir::new().expect("tempdir");
    let plugins_dir = tmp.path().join(".claude").join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir");
    let config = serde_json::json!({
        "repositories": {
            "foo/bar": { "installLocation": "/fake/foo" },
            "baz/qux": { "installLocation": "/fake/baz" },
            "empty-entry": {}
        }
    });
    fs::write(
        plugins_dir.join("config.json"),
        serde_json::to_string_pretty(&config).expect("ser"),
    )
    .expect("write config");

    let mut locs = enabled_plugin_install_locations(tmp.path());
    locs.sort();
    assert_eq!(locs.len(), 2);
    assert!(locs.iter().any(|p| p.ends_with("foo")));
    assert!(locs.iter().any(|p| p.ends_with("baz")));
}

#[test]
fn test_plugin_locations_ignores_invalid_json() {
    let tmp = TempDir::new().expect("tempdir");
    let plugins_dir = tmp.path().join(".claude").join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir");
    fs::write(plugins_dir.join("config.json"), "{not valid json").expect("write");
    let locs = enabled_plugin_install_locations(tmp.path());
    assert!(locs.is_empty());
}

#[test]
fn test_find_plugin_skills_dirs_discovers_nested_skills() {
    let tmp = TempDir::new().expect("tempdir");
    let plugin_root = tmp.path().join("my-plugin");
    fs::create_dir_all(plugin_root.join("sub").join("skills")).expect("mkdir nested");
    fs::create_dir_all(plugin_root.join("skills")).expect("mkdir root");

    let mut found = find_plugin_skills_dirs(&plugin_root).expect("walk");
    found.sort();
    assert_eq!(found.len(), 2);
    assert!(found.iter().any(|p| p == &plugin_root.join("skills")));
    assert!(
        found
            .iter()
            .any(|p| p == &plugin_root.join("sub").join("skills"))
    );
}

#[test]
fn test_find_plugin_skills_dirs_skips_node_modules() {
    let tmp = TempDir::new().expect("tempdir");
    let plugin_root = tmp.path().join("p");
    fs::create_dir_all(plugin_root.join("node_modules").join("skills")).expect("mkdir nm");
    fs::create_dir_all(plugin_root.join(".git").join("skills")).expect("mkdir git");
    let found = find_plugin_skills_dirs(&plugin_root).expect("walk");
    assert!(
        found.is_empty(),
        "node_modules + .git 目录下的 skills/ 必须被跳过，实际: {found:?}"
    );
}

// ============= conflicts.rs =============

#[test]
fn test_detect_conflicts_flags_same_name_different_paths() {
    let entries = [
        ("a", "my-skill", "/home/alice/.claude/skills/my-skill"),
        ("b", "my-skill", "/tmp/project/.claude/skills/my-skill"),
        ("c", "other-skill", "/home/alice/.claude/skills/other-skill"),
    ];
    let groups = detect_conflicts(&entries);
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].name, "my-skill");
    assert_eq!(groups[0].skill_ids.len(), 2);
    assert_eq!(groups[0].real_paths.len(), 2);
}

#[test]
fn test_detect_conflicts_dedups_same_real_path() {
    // Symlink 情况：两条 skill 指向同一 realPath，不算冲突
    let entries = [("a", "s", "/shared/real"), ("b", "s", "/shared/real")];
    let groups = detect_conflicts(&entries);
    assert!(
        groups.is_empty(),
        "同 realPath（如符号链接指向同一真实目录）不应视为冲突"
    );
}

#[test]
fn test_detect_conflicts_returns_empty_for_unique_names() {
    let entries = [
        ("a", "one", "/p1"),
        ("b", "two", "/p2"),
        ("c", "three", "/p3"),
    ];
    assert!(detect_conflicts(&entries).is_empty());
}

// ============= health.rs =============

#[test]
fn test_health_report_aggregates_counts() {
    let conflicts = vec![];
    let merge = vec![];
    let disabled = vec!["x".to_string()];
    let report = compute_health(42, &conflicts, &merge, &disabled, 3);
    assert_eq!(report.total, 42);
    assert_eq!(report.conflicts, 0);
    assert_eq!(report.merge_suggestions, 0);
    assert_eq!(report.disabled, 1);
    assert_eq!(report.plugin_locations, 3);
}
