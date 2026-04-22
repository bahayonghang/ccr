#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn run_clean(args: &[&str], current_dir: &Path, home: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ccr"))
        .args(args)
        .current_dir(current_dir)
        .env("HOME", home)
        .env("USERPROFILE", home)
        .env("NO_COLOR", "1")
        .env("CLICOLOR", "0")
        .env("COLUMNS", "120")
        .output()
        .unwrap()
}

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

#[test]
fn clean_planfiles_dry_run_keeps_nested_files() {
    let temp_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let nested = temp_dir.path().join("nested").join("child");

    write_file(&temp_dir.path().join("task_plan.md"), "root task");
    write_file(&nested.join("findings.md"), "nested findings");
    write_file(&nested.join("progress.md"), "nested progress");
    write_file(&temp_dir.path().join("README.md"), "keep");

    let output = run_clean(
        &["clean", "planfiles", "--dry-run"],
        temp_dir.path(),
        home_dir.path(),
    );
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("命中数量: 3 个"));
    assert!(temp_dir.path().join("task_plan.md").exists());
    assert!(nested.join("findings.md").exists());
    assert!(nested.join("progress.md").exists());
    assert!(temp_dir.path().join("README.md").exists());
}

#[test]
fn clean_planfiles_yes_removes_only_target_files() {
    let temp_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let nested = temp_dir.path().join("nested");

    write_file(&temp_dir.path().join("task_plan.md"), "root task");
    write_file(&nested.join("findings.md"), "nested findings");
    write_file(&nested.join("progress.md"), "nested progress");
    write_file(&nested.join("notes.md"), "keep");

    let output = run_clean(
        &["-y", "clean", "planfiles"],
        temp_dir.path(),
        home_dir.path(),
    );
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("已删除文件: 3 个"));
    assert!(!temp_dir.path().join("task_plan.md").exists());
    assert!(!nested.join("findings.md").exists());
    assert!(!nested.join("progress.md").exists());
    assert!(nested.join("notes.md").exists());
}

#[cfg(unix)]
#[test]
fn clean_planfiles_dry_run_skips_symlink_directories() {
    use std::os::unix::fs::symlink;

    let temp_dir = tempfile::tempdir().unwrap();
    let external_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let link_dir = temp_dir.path().join("linked");

    write_file(&external_dir.path().join("task_plan.md"), "outside task");
    symlink(external_dir.path(), &link_dir).unwrap();

    let output = run_clean(
        &["clean", "planfiles", "--dry-run"],
        temp_dir.path(),
        home_dir.path(),
    );
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("没有找到需要清理的规划文件"));
    assert!(external_dir.path().join("task_plan.md").exists());
}
