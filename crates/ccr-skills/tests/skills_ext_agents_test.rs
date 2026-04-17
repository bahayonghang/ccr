//! Phase 1 — skills_ext Agent 抽象集成测试。
//!
//! 验证：
//! - 注册表包含 5 个 agent 且 id 唯一
//! - 路径约定与 skill-hub agents.ts 注册表一致
//! - `AgentId::from_str` 接受常见别名、未知输入归为 Unknown
//! - `paths::*` 辅助函数返回以 `.ccr/skills` 结尾的路径

use std::path::{Path, PathBuf};

use ccr_skills::skills_ext::paths::{skills_root, trash_root, versions_root};
use ccr_skills::skills_ext::{AgentId, all_agents, locator_for};

#[test]
fn registry_contains_five_agents() {
    let agents = all_agents();
    assert_eq!(agents.len(), 5, "预期 5 个 agent，当前 {}", agents.len());
}

#[test]
fn agent_ids_are_unique_and_cover_all_five() {
    let ids: Vec<AgentId> = all_agents().iter().map(|a| a.id()).collect();
    let mut dedup = ids.clone();
    dedup.sort_by_key(|id| id.as_kebab());
    dedup.dedup();
    assert_eq!(dedup.len(), ids.len(), "AgentId 有重复");

    assert!(ids.contains(&AgentId::ClaudeCode));
    assert!(ids.contains(&AgentId::Codex));
    assert!(ids.contains(&AgentId::Gemini));
    assert!(ids.contains(&AgentId::Droid));
    assert!(ids.contains(&AgentId::OpenCode));
}

#[test]
fn claude_code_paths_follow_convention() {
    let home = PathBuf::from("/home/test");
    let project = PathBuf::from("/tmp/my-project");
    let locator = locator_for(AgentId::ClaudeCode).expect("claude-code 必须在注册表中");
    assert_eq!(
        locator.global_paths(&home),
        vec![PathBuf::from("/home/test").join(".claude").join("skills")]
    );
    assert_eq!(
        locator.project_paths(&project),
        vec![
            PathBuf::from("/tmp/my-project")
                .join(".claude")
                .join("skills")
        ]
    );
}

#[test]
fn codex_and_gemini_have_only_global_paths() {
    let home = PathBuf::from("/h");
    let project = PathBuf::from("/p");

    for id in [AgentId::Codex, AgentId::Gemini] {
        let locator = locator_for(id).expect("Codex/Gemini 必须在注册表中");
        assert_eq!(
            locator.global_paths(&home).len(),
            1,
            "{:?} 全局路径应为 1 条",
            id
        );
        assert!(
            locator.project_paths(&project).is_empty(),
            "{:?} 不应有项目级路径",
            id
        );
    }
}

#[test]
fn opencode_uses_xdg_config_layout() {
    let home = PathBuf::from("/u");
    let opencode = locator_for(AgentId::OpenCode).expect("opencode locator 必须存在");
    assert_eq!(
        opencode.global_paths(&home),
        vec![
            PathBuf::from("/u")
                .join(".config")
                .join("opencode")
                .join("skills")
        ]
    );
}

#[test]
fn droid_uses_factory_dir_in_global_and_project() {
    let home = PathBuf::from("/u");
    let project = PathBuf::from("/proj");
    let droid = locator_for(AgentId::Droid).expect("droid locator 必须存在");
    assert_eq!(
        droid.global_paths(&home),
        vec![PathBuf::from("/u").join(".factory").join("skills")]
    );
    assert_eq!(
        droid.project_paths(&project),
        vec![PathBuf::from("/proj").join(".factory").join("skills")]
    );
}

#[test]
fn agent_id_parse_handles_aliases() {
    assert_eq!(AgentId::parse("claude-code"), AgentId::ClaudeCode);
    assert_eq!(AgentId::parse("Claude"), AgentId::ClaudeCode);
    assert_eq!(AgentId::parse("CLAUDECODE"), AgentId::ClaudeCode);
    assert_eq!(AgentId::parse("gemini-cli"), AgentId::Gemini);
    assert_eq!(AgentId::parse("OPENCODE"), AgentId::OpenCode);
    assert_eq!(AgentId::parse("  codex  "), AgentId::Codex);
    assert_eq!(AgentId::parse("nonesuch"), AgentId::Unknown);
    assert_eq!(AgentId::parse(""), AgentId::Unknown);
}

#[test]
fn agent_id_kebab_case_is_stable() {
    assert_eq!(AgentId::ClaudeCode.as_kebab(), "claude-code");
    assert_eq!(AgentId::Codex.as_kebab(), "codex");
    assert_eq!(AgentId::Gemini.as_kebab(), "gemini");
    assert_eq!(AgentId::Droid.as_kebab(), "droid");
    assert_eq!(AgentId::OpenCode.as_kebab(), "opencode");
    assert_eq!(AgentId::Unknown.as_kebab(), "unknown");
}

#[test]
fn locator_for_unknown_returns_none() {
    assert!(locator_for(AgentId::Unknown).is_none());
}

#[test]
fn registry_display_names_and_icons_are_non_empty() {
    for agent in all_agents() {
        assert!(
            !agent.display_name().is_empty(),
            "{:?} 缺 display_name",
            agent.id()
        );
        assert!(!agent.icon().is_empty(), "{:?} 缺 icon", agent.id());
    }
}

#[test]
fn skills_root_uses_dot_ccr_hierarchy() {
    if let Some(root) = skills_root() {
        let tail = Path::new(".ccr").join("skills");
        assert!(
            root.ends_with(&tail),
            "skills_root 路径异常: {root:?}（期望以 .ccr/skills 结尾）"
        );
    }
}

#[test]
fn versions_and_trash_are_under_skills_root() {
    if let (Some(root), Some(versions), Some(trash)) =
        (skills_root(), versions_root(), trash_root())
    {
        assert!(versions.starts_with(&root), "versions 不在 skills_root 下");
        assert!(trash.starts_with(&root), "trash 不在 skills_root 下");
        assert!(versions.ends_with("versions"));
        assert!(trash.ends_with("trash"));
    }
}
