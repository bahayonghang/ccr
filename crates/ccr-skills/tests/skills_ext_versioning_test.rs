//! Phase 2 — skills_ext Version Store 集成测试。
//!
//! 覆盖：
//! - 快照往返（写 → 读 → bytewise equal）
//! - 内容去重（相同 hash 复用旧版本）
//! - FIFO 50 版本上限
//! - LCS diff 基础场景 + 2000 行截断保护
//! - 回滚含子目录文件
//! - 回滚前自动安全快照

use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use tempfile::TempDir;

use ccr_skills::skills_ext::lcs::{self, DiffLineKind};
use ccr_skills::skills_ext::versioning::{FsVersionStore, MAX_VERSIONS_PER_SKILL, SnapshotSource};

fn make_skill(tmp: &TempDir, name: &str, content: &str) -> PathBuf {
    let dir = tmp.path().join(name);
    fs::create_dir_all(&dir).expect("创建 skill 目录");
    fs::write(dir.join("SKILL.md"), content).expect("写 SKILL.md");
    dir
}

fn open_store(tmp: &TempDir) -> FsVersionStore {
    FsVersionStore::with_root(tmp.path().join("store")).expect("打开 store")
}

#[test]
fn test_snapshot_roundtrip() {
    let tmp = TempDir::new().expect("tempdir");
    let store = open_store(&tmp);
    let skill = make_skill(&tmp, "my-skill", "hello world\n");

    let meta = store
        .snapshot(&skill, "my-skill", "initial", SnapshotSource::Manual)
        .expect("snapshot");
    let v = store
        .get(&skill, &meta.id)
        .expect("get")
        .expect("version exists");

    assert_eq!(v.content, "hello world\n");
    assert_eq!(v.skill_name, "my-skill");
    assert_eq!(v.source, SnapshotSource::Manual);
    assert_eq!(v.id, meta.id);
    assert_eq!(v.content_hash, meta.content_hash);
}

#[test]
fn test_dedup_same_content() {
    let tmp = TempDir::new().expect("tempdir");
    let store = open_store(&tmp);
    let skill = make_skill(&tmp, "dup", "same\n");

    let m1 = store
        .snapshot(&skill, "dup", "first", SnapshotSource::Manual)
        .expect("snapshot 1");
    let m2 = store
        .snapshot(&skill, "dup", "second", SnapshotSource::Manual)
        .expect("snapshot 2");

    assert_eq!(m1.id, m2.id, "相同内容必须复用最新版本 id");
    assert_eq!(
        store.history(&skill).expect("history").len(),
        1,
        "历史里只应有 1 个版本"
    );
}

#[test]
fn test_fifo_cap_50() {
    let tmp = TempDir::new().expect("tempdir");
    let store = open_store(&tmp);
    let skill = make_skill(&tmp, "cap", "0\n");

    // 写 MAX + 5 个**内容各异**的版本，触发 FIFO 淘汰
    for i in 0..(MAX_VERSIONS_PER_SKILL + 5) {
        fs::write(skill.join("SKILL.md"), format!("{i}\n")).expect("改 SKILL.md");
        store
            .snapshot(&skill, "cap", &format!("v{i}"), SnapshotSource::Manual)
            .expect("snapshot");
    }

    let history = store.history(&skill).expect("history");
    assert_eq!(
        history.len(),
        MAX_VERSIONS_PER_SKILL,
        "版本数应精确等于 {}",
        MAX_VERSIONS_PER_SKILL
    );

    let messages: Vec<String> = history.iter().map(|m| m.message.clone()).collect();
    assert!(!messages.iter().any(|m| m == "v0"), "v0 应已 FIFO 淘汰");
    assert!(!messages.iter().any(|m| m == "v4"), "v4 应已 FIFO 淘汰");
    assert!(messages.iter().any(|m| m == "v5"), "v5 应保留");
    assert!(
        messages
            .iter()
            .any(|m| m == &format!("v{}", MAX_VERSIONS_PER_SKILL + 4)),
        "最新版本应保留"
    );
}

#[test]
fn test_diff_lcs_basic() {
    let old_text = "a\nb\nc";
    let new_text = "a\nX\nc";
    let lines = lcs::diff(old_text, new_text);

    let adds: Vec<&str> = lines
        .iter()
        .filter(|l| l.kind == DiffLineKind::Add)
        .map(|l| l.content.as_str())
        .collect();
    let removes: Vec<&str> = lines
        .iter()
        .filter(|l| l.kind == DiffLineKind::Remove)
        .map(|l| l.content.as_str())
        .collect();
    let sames: Vec<&str> = lines
        .iter()
        .filter(|l| l.kind == DiffLineKind::Same)
        .map(|l| l.content.as_str())
        .collect();

    assert_eq!(adds, vec!["X"], "仅新增一行 X");
    assert_eq!(removes, vec!["b"], "仅删除一行 b");
    assert_eq!(sames, vec!["a", "c"], "a 与 c 保持不变");
}

#[test]
fn test_diff_lcs_truncates_large_input() {
    // 生成 3000 行完全不同的内容；LCS 被截断到 2000 行，应在秒级完成不 panic。
    let old_text: String = (0..3000)
        .map(|i| format!("old-{i}"))
        .collect::<Vec<_>>()
        .join("\n");
    let new_text: String = (0..3000)
        .map(|i| format!("new-{i}"))
        .collect::<Vec<_>>()
        .join("\n");

    let start = Instant::now();
    let lines = lcs::diff(&old_text, &new_text);
    let elapsed = start.elapsed();

    assert!(!lines.is_empty());
    assert!(
        elapsed.as_secs() < 10,
        "LCS 在 2000 行截断下应 <10s，实际 {:?}",
        elapsed
    );
}

#[test]
fn test_rollback_restores_skill_md_and_subdir_files() {
    let tmp = TempDir::new().expect("tempdir");
    let store = open_store(&tmp);
    let skill = make_skill(&tmp, "rb", "version-1\n");

    // 加子目录附属文件
    fs::create_dir_all(skill.join("assets")).expect("mkdir assets");
    fs::write(skill.join("assets").join("note.txt"), "asset-1").expect("写 asset");
    fs::write(skill.join("README.txt"), "readme-1").expect("写 readme");

    let v1 = store
        .snapshot(&skill, "rb", "v1", SnapshotSource::Manual)
        .expect("snapshot v1");

    // 修改 SKILL.md 与附属
    fs::write(skill.join("SKILL.md"), "version-2\n").expect("改 SKILL.md");
    fs::write(skill.join("assets").join("note.txt"), "asset-2").expect("改 asset");
    fs::write(skill.join("README.txt"), "readme-2").expect("改 readme");

    store.rollback(&skill, &v1.id).expect("rollback");

    assert_eq!(
        fs::read_to_string(skill.join("SKILL.md")).expect("读 SKILL.md"),
        "version-1\n",
        "SKILL.md 应被回滚"
    );
    assert_eq!(
        fs::read_to_string(skill.join("assets").join("note.txt")).expect("读 asset"),
        "asset-1",
        "子目录文件应被回滚"
    );
    assert_eq!(
        fs::read_to_string(skill.join("README.txt")).expect("读 readme"),
        "readme-1",
        "根下其他文本文件应被回滚"
    );
}

#[test]
fn test_rollback_deletes_orphan_files_not_in_snapshot() {
    // P1-5 回归：snapshot={SKILL.md, a.txt}，当前={SKILL.md, a.txt, b.txt}
    // 回滚后 b.txt 必须被删除，而不是保留形成混合状态。
    let tmp = TempDir::new().expect("tempdir");
    let store = open_store(&tmp);
    let skill = make_skill(&tmp, "orphan", "v1\n");
    fs::write(skill.join("a.txt"), "keep-me").expect("写 a");

    let v1 = store
        .snapshot(&skill, "orphan", "v1", SnapshotSource::Manual)
        .expect("snapshot v1");

    // 模拟用户后来新增 b.txt + 修改 SKILL.md
    fs::write(skill.join("b.txt"), "orphan-to-delete").expect("写 b");
    fs::write(skill.join("SKILL.md"), "v2\n").expect("改 SKILL.md");
    assert!(skill.join("b.txt").exists(), "前置：b.txt 必须存在");

    store.rollback(&skill, &v1.id).expect("rollback");

    // 核心断言：b.txt 必须被清除
    assert!(
        !skill.join("b.txt").exists(),
        "P1-5: orphan 文件 b.txt 必须在 rollback 时被删除"
    );
    // a.txt 与 SKILL.md 应该是 v1 状态
    assert_eq!(
        fs::read_to_string(skill.join("a.txt")).expect("读 a"),
        "keep-me"
    );
    assert_eq!(
        fs::read_to_string(skill.join("SKILL.md")).expect("读 SKILL.md"),
        "v1\n"
    );
}

#[test]
fn test_rollback_creates_safety_snapshot_for_unsaved_state() {
    let tmp = TempDir::new().expect("tempdir");
    let store = open_store(&tmp);
    let skill = make_skill(&tmp, "safety", "a\n");

    let v1 = store
        .snapshot(&skill, "safety", "v1", SnapshotSource::Manual)
        .expect("snapshot v1");

    // 模拟用户未保存的修改
    fs::write(skill.join("SKILL.md"), "b-precious-unsaved\n").expect("未保存状态");

    store.rollback(&skill, &v1.id).expect("rollback");

    // 回滚后历史里必须能找回一个 content = "b-precious-unsaved\n" 的自动备份
    let history = store.history(&skill).expect("history");
    let has_safety_snapshot = history.iter().any(|meta| {
        let v = store
            .get(&skill, &meta.id)
            .expect("get")
            .expect("version exists");
        v.content == "b-precious-unsaved\n" && v.source == SnapshotSource::Auto
    });

    assert!(
        has_safety_snapshot,
        "回滚前必须对未保存状态自动快照，否则会丢数据"
    );
}
