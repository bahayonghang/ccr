//! Phase 3 — skills_ext Trash Store 集成测试。
//!
//! 覆盖 6 个核心场景：
//! - 软删除 → list
//! - 恢复到原路径
//! - 恢复时名冲突走 `-restored-<ts>` 后缀
//! - 永久删除
//! - 7 天 TTL + 注入时钟
//! - 并发（同 store 不同 skill）不 panic

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;

use chrono::{Duration, Utc};
use tempfile::TempDir;

use ccr_skills::skills_ext::trash::{
    Clock, FsTrashStore, MutableClock, TRASH_TTL_DAYS, TrashError,
};

fn make_skill(tmp: &TempDir, name: &str, content: &str) -> PathBuf {
    let dir = tmp.path().join(name);
    fs::create_dir_all(&dir).expect("mkdir skill");
    fs::write(dir.join("SKILL.md"), content).expect("write SKILL.md");
    dir
}

fn open_store(tmp: &TempDir) -> FsTrashStore {
    FsTrashStore::with_root(tmp.path().join("trash")).expect("open store")
}

#[test]
fn test_move_to_trash_and_list() {
    let tmp = TempDir::new().expect("tempdir");
    let store = open_store(&tmp);
    let skill = make_skill(&tmp, "a", "hello\n");

    let entry = store
        .move_to_trash(&skill, "a")
        .expect("move_to_trash 成功");

    // 原路径已消失
    assert!(!skill.exists(), "原 skill 目录必须已移走");

    // list 能查到
    let list = store.list().expect("list");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, entry.id);
    assert_eq!(list[0].skill_name, "a");
    assert_eq!(list[0].original_path, skill.to_string_lossy());
}

#[test]
fn test_restore_to_original_path() {
    let tmp = TempDir::new().expect("tempdir");
    let store = open_store(&tmp);
    let skill = make_skill(&tmp, "r", "body\n");
    let original = skill.clone();

    let entry = store.move_to_trash(&skill, "r").expect("move");
    assert!(!original.exists());

    let restored = store.restore(&entry.id).expect("restore");
    assert_eq!(restored, original);
    assert!(original.exists());
    assert_eq!(
        fs::read_to_string(original.join("SKILL.md")).expect("read"),
        "body\n"
    );

    // 条目已从 list 清掉
    assert!(store.list().expect("list").is_empty());
}

#[test]
fn test_restore_name_collision_adds_suffix() {
    let tmp = TempDir::new().expect("tempdir");
    let store = open_store(&tmp);
    let skill = make_skill(&tmp, "c", "old\n");
    let original = skill.clone();

    let entry = store.move_to_trash(&skill, "c").expect("move");

    // 模拟原位置被重建了同名 skill
    fs::create_dir_all(&original).expect("mkdir new");
    fs::write(original.join("SKILL.md"), "new-recreated").expect("写新内容");

    let restored = store.restore(&entry.id).expect("restore");
    assert_ne!(restored, original, "必须落到带后缀的新路径");
    let name = restored
        .file_name()
        .expect("name")
        .to_string_lossy()
        .into_owned();
    assert!(
        name.starts_with("c-restored-"),
        "恢复后文件名必须形如 c-restored-<ts>，实际 {name}"
    );

    // 新旧都在磁盘上
    assert!(original.exists());
    assert!(restored.exists());
    assert_eq!(
        fs::read_to_string(original.join("SKILL.md")).expect("读原位"),
        "new-recreated"
    );
    assert_eq!(
        fs::read_to_string(restored.join("SKILL.md")).expect("读恢复"),
        "old\n"
    );
}

#[test]
fn test_permanent_delete() {
    let tmp = TempDir::new().expect("tempdir");
    let store = open_store(&tmp);
    let skill = make_skill(&tmp, "p", "x");
    let entry = store.move_to_trash(&skill, "p").expect("move");

    assert!(
        store.permanent_delete(&entry.id).expect("perm delete"),
        "首次删除返回 true"
    );
    assert!(store.list().expect("list").is_empty());

    assert!(
        !store
            .permanent_delete(&entry.id)
            .expect("perm delete again"),
        "再删返回 false（不存在）"
    );
}

#[test]
fn test_purge_expired_after_7_days() {
    let tmp = TempDir::new().expect("tempdir");

    // 模拟 10 天前软删除
    let past = Utc::now() - Duration::days(10);
    let clock = Arc::new(MutableClock::new(past));
    let clock_trait: Arc<dyn Clock> = clock.clone();
    let store = FsTrashStore::with_root_and_clock(tmp.path().join("t"), clock_trait)
        .expect("open with clock");

    let skill_a = make_skill(&tmp, "aa", "a");
    let skill_b = make_skill(&tmp, "bb", "b");
    store.move_to_trash(&skill_a, "aa").expect("move a");
    store.move_to_trash(&skill_b, "bb").expect("move b");

    assert_eq!(store.list().expect("list").len(), 2);

    // 快进时钟到"现在" — 两条 entry 均已超过 TTL
    clock.advance_to(Utc::now());

    let purged = store.purge_expired().expect("purge");
    assert_eq!(purged, 2, "两条都应清理");
    assert!(store.list().expect("list").is_empty());

    // 清理常量自检
    assert_eq!(TRASH_TTL_DAYS, 7);
}

#[test]
fn test_purge_keeps_fresh_entries() {
    let tmp = TempDir::new().expect("tempdir");
    let clock = Arc::new(MutableClock::new(Utc::now()));
    let clock_trait: Arc<dyn Clock> = clock.clone();
    let store = FsTrashStore::with_root_and_clock(tmp.path().join("t"), clock_trait).expect("open");

    let skill = make_skill(&tmp, "fresh", "x");
    store.move_to_trash(&skill, "fresh").expect("move");

    // 时钟只前进 1 天 — 未过期
    clock.advance_to(Utc::now() + Duration::days(1));

    let purged = store.purge_expired().expect("purge");
    assert_eq!(purged, 0);
    assert_eq!(store.list().expect("list").len(), 1);
}

#[test]
fn test_concurrent_move_no_panic() {
    let tmp = TempDir::new().expect("tempdir");
    let store = Arc::new(open_store(&tmp));
    let skill_a = make_skill(&tmp, "ca", "a");
    let skill_b = make_skill(&tmp, "cb", "b");

    let sa = Arc::clone(&store);
    let sb = Arc::clone(&store);

    let h1 = thread::spawn(move || sa.move_to_trash(&skill_a, "ca"));
    let h2 = thread::spawn(move || sb.move_to_trash(&skill_b, "cb"));

    let r1 = h1.join().expect("t1 未 panic").expect("move a 成功");
    let r2 = h2.join().expect("t2 未 panic").expect("move b 成功");

    assert_ne!(r1.id, r2.id, "并发场景必须生成不同的 trash_id");
    assert_eq!(store.list().expect("list").len(), 2);
}

#[test]
fn test_move_nonexistent_skill_returns_source_not_found() {
    let tmp = TempDir::new().expect("tempdir");
    let store = open_store(&tmp);
    let ghost = tmp.path().join("does-not-exist");

    match store.move_to_trash(&ghost, "ghost") {
        Err(TrashError::SourceNotFound(p)) => assert_eq!(p, ghost),
        other => panic!("期望 SourceNotFound，实际 {other:?}"),
    }
}

#[test]
fn test_restore_unknown_trash_id_returns_entry_not_found() {
    let tmp = TempDir::new().expect("tempdir");
    let store = open_store(&tmp);
    match store.restore("nonesuch") {
        Err(TrashError::EntryNotFound(id)) => assert_eq!(id, "nonesuch"),
        other => panic!("期望 EntryNotFound，实际 {other:?}"),
    }
}

#[test]
fn test_list_handles_missing_root() {
    let tmp = TempDir::new().expect("tempdir");
    // 打开后删掉根目录，list 应返回空 vec 而非 error
    let store = open_store(&tmp);
    let root = tmp.path().join("trash");
    fs::remove_dir_all(&root).expect("rm root");
    assert!(!Path::new(&root).exists());
    let list = store.list().expect("list should gracefully return empty");
    assert!(list.is_empty());
}
