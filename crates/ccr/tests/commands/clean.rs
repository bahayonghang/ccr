#![allow(clippy::unwrap_used)]

use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Output, Stdio};

fn run_clean(args: &[&str], current_dir: &Path, home: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ccr"))
        .args(args)
        .current_dir(current_dir)
        .env("HOME", home)
        .env("USERPROFILE", home)
        .env("CCR_ROOT", home.join(".ccr"))
        .env("CCR_BACKUP_DIR", home.join(".claude").join("backups"))
        .env("CCR_LOCK_DIR", home.join(".claude").join(".locks"))
        .env("NO_COLOR", "1")
        .env("CLICOLOR", "0")
        .env("COLUMNS", "120")
        .output()
        .unwrap()
}

fn run_clean_with_input(args: &[&str], current_dir: &Path, home: &Path, input: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_ccr"))
        .args(args)
        .current_dir(current_dir)
        .env("HOME", home)
        .env("USERPROFILE", home)
        .env("CCR_ROOT", home.join(".ccr"))
        .env("CCR_BACKUP_DIR", home.join(".claude").join("backups"))
        .env("CCR_LOCK_DIR", home.join(".claude").join(".locks"))
        .env("NO_COLOR", "1")
        .env("CLICOLOR", "0")
        .env("COLUMNS", "120")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(input.as_bytes()).unwrap();
    }

    child.wait_with_output().unwrap()
}

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

fn write_old_backup(home: &Path, name: &str) -> std::path::PathBuf {
    let backup_path = home.join(".claude").join("backups").join(name);
    write_file(&backup_path, "old backup");
    let old_time = std::time::SystemTime::now() - std::time::Duration::from_secs(10 * 24 * 60 * 60);
    filetime::set_file_mtime(&backup_path, filetime::FileTime::from_system_time(old_time)).unwrap();
    backup_path
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

#[test]
fn clean_planfiles_dry_run_includes_hidden_and_ignored_dirs() {
    let temp_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let hidden = temp_dir.path().join(".hidden");
    let ignored = temp_dir.path().join("ignored");

    write_file(&temp_dir.path().join(".gitignore"), "ignored/\n");
    write_file(&hidden.join("task_plan.md"), "hidden task");
    write_file(&ignored.join("findings.md"), "ignored findings");

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
    assert!(stdout.contains("命中数量: 2 个"));
    assert!(hidden.join("task_plan.md").exists());
    assert!(ignored.join("findings.md").exists());
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

#[test]
fn clean_menu_default_number_runs_planfiles_target() {
    let temp_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let nested = temp_dir.path().join("nested");
    let old_backup = write_old_backup(home_dir.path(), "old.bak");

    write_file(&temp_dir.path().join("task_plan.md"), "root task");
    write_file(&nested.join("findings.md"), "nested findings");
    write_file(&nested.join("notes.md"), "keep");

    let output = run_clean_with_input(&["clean"], temp_dir.path(), home_dir.path(), "\n\n");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("1.planfiles -"));
    assert!(stdout.contains("2.backups -"));
    assert!(stdout.contains("确认执行规划文件清理操作? (Y/n):"));
    assert!(!temp_dir.path().join("task_plan.md").exists());
    assert!(!nested.join("findings.md").exists());
    assert!(nested.join("notes.md").exists());
    assert!(old_backup.exists());
}

#[test]
fn clean_menu_number_can_run_backups_target() {
    let temp_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let old_backup = write_old_backup(home_dir.path(), "old.bak");

    write_file(&temp_dir.path().join("task_plan.md"), "root task");

    let output = run_clean_with_input(&["clean"], temp_dir.path(), home_dir.path(), "2\ny\n");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(temp_dir.path().join("task_plan.md").exists());
    assert!(!old_backup.exists());
}

#[test]
fn clean_menu_can_cancel_without_running_target() {
    let temp_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let old_backup = write_old_backup(home_dir.path(), "old.bak");

    write_file(&temp_dir.path().join("task_plan.md"), "root task");

    let output = run_clean_with_input(&["clean"], temp_dir.path(), home_dir.path(), "q\n");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(temp_dir.path().join("task_plan.md").exists());
    assert!(old_backup.exists());
}

#[test]
fn clean_menu_auto_yes_runs_default_planfiles_target() {
    let temp_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let old_backup = write_old_backup(home_dir.path(), "old.bak");

    write_file(&temp_dir.path().join("task_plan.md"), "root task");

    let output = run_clean(&["-y", "clean"], temp_dir.path(), home_dir.path());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!temp_dir.path().join("task_plan.md").exists());
    assert!(old_backup.exists());
}

#[test]
fn clean_backups_subcommand_dry_run_keeps_old_backup() {
    let temp_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let old_backup = write_old_backup(home_dir.path(), "old.bak");

    let output = run_clean(
        &["clean", "backups", "--days", "7", "--dry-run"],
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
    assert!(stdout.contains("将删除文件: 1 个"));
    assert!(old_backup.exists());
}
