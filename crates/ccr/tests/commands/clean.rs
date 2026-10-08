#![allow(clippy::unwrap_used)]

use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Output, Stdio};

fn clean_command(args: &[&str], current_dir: &Path, home: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_ccr"));
    command
        .args(args)
        .current_dir(current_dir)
        .env("HOME", home)
        .env("USERPROFILE", home)
        .env("CCR_ROOT", home.join(".ccr"))
        .env("CCR_BACKUP_DIR", home.join(".claude").join("backups"))
        .env("CCR_LOCK_DIR", home.join(".claude").join(".locks"))
        .env("NO_COLOR", "1")
        .env("CLICOLOR", "0")
        .env("COLUMNS", "120");
    command
}

fn run_clean(args: &[&str], current_dir: &Path, home: &Path) -> Output {
    clean_command(args, current_dir, home).output().unwrap()
}

fn run_clean_with_input(args: &[&str], current_dir: &Path, home: &Path, input: &str) -> Output {
    let mut child = clean_command(args, current_dir, home)
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

fn storage_command(
    args: &[&str],
    current_dir: &Path,
    home: &Path,
    extra_env: &[(&str, &Path)],
) -> Command {
    let mut command = clean_command(args, current_dir, home);
    command.env_remove("CCR_DATA_DIR");
    command.env_remove("LLMUSAGE_HOME");
    for (key, value) in extra_env {
        command.env(key, value);
    }
    command
}

fn run_storage(
    args: &[&str],
    current_dir: &Path,
    home: &Path,
    extra_env: &[(&str, &Path)],
) -> Output {
    storage_command(args, current_dir, home, extra_env)
        .output()
        .unwrap()
}

fn run_storage_with_input(
    args: &[&str],
    current_dir: &Path,
    home: &Path,
    extra_env: &[(&str, &Path)],
    input: &str,
) -> Output {
    let mut child = storage_command(args, current_dir, home, extra_env)
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

struct SeededStorage {
    backend_target: std::path::PathBuf,
    frontend_modules: std::path::PathBuf,
    migration_bak: std::path::PathBuf,
    legacy_llmusage: std::path::PathBuf,
    keepers: Vec<std::path::PathBuf>,
}

fn seed_storage_tree(root: &Path) -> SeededStorage {
    let backend_target = root.join("ccr-ui").join("backend").join("target");
    let frontend_modules = root.join("ccr-ui").join("frontend").join("node_modules");
    let migration_bak = root
        .join("analytics")
        .join("usage.db.pre-migration-v16.20260727.bak");
    let legacy_llmusage = root.join("llmusage");
    let candidate_bytes = vec![b'x'; 512 * 1024];

    write_file(&backend_target.join("cache.bin"), &candidate_bytes);
    write_file(
        &frontend_modules.join("pkg").join("index.js"),
        &candidate_bytes,
    );
    write_file(&migration_bak, &candidate_bytes);
    write_file(&legacy_llmusage.join("llmusage.db"), &candidate_bytes);
    write_file(
        &root
            .join("ccr-ui")
            .join("backend")
            .join("src")
            .join("main.rs"),
        "keeper-ui-backend",
    );
    write_file(
        &root
            .join("ccr-ui")
            .join("frontend")
            .join("src")
            .join("App.tsx"),
        "keeper-ui-frontend",
    );

    let keeper_paths = [
        root.join("config.toml"),
        root.join("sync.toml"),
        root.join("sync_folders.toml"),
        root.join("desktop-shell.json"),
        root.join("ui_state.json"),
        root.join("platforms/claude/token.txt"),
        root.join("checkin/state.json"),
        root.join("skills/skill.md"),
        root.join("locks/lockfile"),
        root.join("history/claude.json"),
        root.join("backups/keep.bak"),
        root.join("logs/app.log"),
        root.join("data.db"),
        root.join("analytics/usage.db"),
        root.join("analytics/usage.db-wal"),
        root.join("analytics/usage.db-shm"),
        root.join("analytics/nested/usage.db.pre-migration-hidden.bak"),
        root.join("analytics/other.bak"),
    ];
    let mut keepers = Vec::new();
    for path in keeper_paths {
        write_file(&path, "keeper-marker");
        keepers.push(path);
    }

    SeededStorage {
        backend_target,
        frontend_modules,
        migration_bak,
        legacy_llmusage,
        keepers,
    }
}

fn write_file(path: &Path, content: &(impl AsRef<[u8]> + ?Sized)) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

fn assert_storage_candidates(seed: &SeededStorage, present: bool) {
    assert_eq!(seed.backend_target.exists(), present);
    assert_eq!(seed.frontend_modules.exists(), present);
    assert_eq!(seed.migration_bak.exists(), present);
    assert_eq!(seed.legacy_llmusage.exists(), present);
}

fn assert_storage_keepers(seed: &SeededStorage) {
    for keeper in &seed.keepers {
        assert!(keeper.exists(), "keeper removed: {}", keeper.display());
        assert_eq!(fs::read(keeper).unwrap(), b"keeper-marker");
    }
}

fn write_old_backup(home: &Path, name: &str) -> std::path::PathBuf {
    let backup_path = home.join(".claude").join("backups").join(name);
    write_file(&backup_path, "old backup");
    let old_time = std::time::SystemTime::now() - std::time::Duration::from_secs(10 * 24 * 60 * 60);
    filetime::set_file_mtime(&backup_path, filetime::FileTime::from_system_time(old_time)).unwrap();
    backup_path
}

#[test]
fn clean_planfiles_dry_run_defaults_to_root_files_only() {
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
    assert!(stdout.contains("命中数量: 1 个"));
    assert!(stdout.contains("命中: task_plan.md"));
    assert!(!stdout.contains("nested"));
    assert!(temp_dir.path().join("task_plan.md").exists());
    assert!(nested.join("findings.md").exists());
    assert!(nested.join("progress.md").exists());
    assert!(temp_dir.path().join("README.md").exists());
}

#[test]
fn clean_planfiles_all_dry_run_finds_nested_files() {
    let temp_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let nested = temp_dir.path().join("nested").join("child");

    write_file(&temp_dir.path().join("task_plan.md"), "root task");
    write_file(&nested.join("findings.md"), "nested findings");
    write_file(&nested.join("progress.md"), "nested progress");

    let output = run_clean(
        &["clean", "planfiles", "--all", "--dry-run"],
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
    assert!(stdout.contains("nested"));
    assert!(temp_dir.path().join("task_plan.md").exists());
    assert!(nested.join("findings.md").exists());
    assert!(nested.join("progress.md").exists());
}

#[test]
fn clean_planfiles_yes_removes_only_root_target_files() {
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
    assert!(stdout.contains("已删除文件: 1 个"));
    assert!(!temp_dir.path().join("task_plan.md").exists());
    assert!(nested.join("findings.md").exists());
    assert!(nested.join("progress.md").exists());
    assert!(nested.join("notes.md").exists());
}

#[test]
fn clean_planfiles_all_yes_removes_nested_target_files() {
    let temp_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let nested = temp_dir.path().join("nested");

    write_file(&temp_dir.path().join("task_plan.md"), "root task");
    write_file(&nested.join("findings.md"), "nested findings");
    write_file(&nested.join("progress.md"), "nested progress");
    write_file(&nested.join("notes.md"), "keep");

    let output = run_clean(
        &["-y", "clean", "planfiles", "--all"],
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
fn clean_planfiles_dry_run_defaults_to_root_only_for_hidden_and_ignored_dirs() {
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
    assert!(stdout.contains("没有找到需要清理的规划文件"));
    assert!(hidden.join("task_plan.md").exists());
    assert!(ignored.join("findings.md").exists());
}

#[test]
fn clean_planfiles_all_dry_run_includes_hidden_and_ignored_dirs() {
    let temp_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let hidden = temp_dir.path().join(".hidden");
    let ignored = temp_dir.path().join("ignored");

    write_file(&temp_dir.path().join(".gitignore"), "ignored/\n");
    write_file(&hidden.join("task_plan.md"), "hidden task");
    write_file(&ignored.join("findings.md"), "ignored findings");

    let output = run_clean(
        &["clean", "planfiles", "--all", "--dry-run"],
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
    assert!(nested.join("findings.md").exists());
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
    assert!(stdout.contains("确认执行清理操作?"));
    assert!(
        !home_dir
            .path()
            .join(".ccr/platforms/claude/profiles.toml")
            .exists()
    );
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
    let nested = temp_dir.path().join("nested");
    let old_backup = write_old_backup(home_dir.path(), "old.bak");

    write_file(&temp_dir.path().join("task_plan.md"), "root task");
    write_file(&nested.join("findings.md"), "nested findings");

    let output = run_clean(&["-y", "clean"], temp_dir.path(), home_dir.path());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!temp_dir.path().join("task_plan.md").exists());
    assert!(nested.join("findings.md").exists());
    assert!(old_backup.exists());
}

#[test]
fn clean_all_runs_recursive_planfiles_without_menu() {
    let temp_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let nested = temp_dir.path().join("nested");
    let old_backup = write_old_backup(home_dir.path(), "old.bak");

    write_file(&temp_dir.path().join("task_plan.md"), "root task");
    write_file(&nested.join("findings.md"), "nested findings");

    let output = run_clean(&["-y", "clean", "--all"], temp_dir.path(), home_dir.path());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("扫描范围"));
    assert!(stdout.contains("--all"));
    assert!(!stdout.contains("清理内容（输入编号执行"));
    assert!(!temp_dir.path().join("task_plan.md").exists());
    assert!(!nested.join("findings.md").exists());
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
    assert!(
        !home_dir
            .path()
            .join(".ccr/platforms/claude/profiles.toml")
            .exists()
    );
}

#[test]
fn clean_backups_corrupt_config_fails_without_deleting_backup() {
    let temp_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let old_backup = write_old_backup(home_dir.path(), "old.bak");
    let config_path = home_dir.path().join(".ccr/platforms/claude/profiles.toml");
    write_file(&config_path, "invalid = [");

    let output = run_clean(
        &["clean", "backups", "--days", "7", "--force"],
        temp_dir.path(),
        home_dir.path(),
    );

    assert_eq!(
        output.status.code(),
        Some(ccr_core::core::error::exit_codes::CONFIG_FORMAT_INVALID)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("清理备份文件"));
    assert!(String::from_utf8_lossy(&output.stderr).contains("配置格式无效"));
    assert!(old_backup.exists());
    assert_eq!(fs::read_to_string(config_path).unwrap(), "invalid = [");
}

#[test]
fn clean_storage_dry_run_lists_redundant_paths_without_deleting() {
    let temp_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let root = home_dir.path().join(".ccr");
    let seed = seed_storage_tree(&root);

    let output = run_storage(
        &["clean", "storage", "--dry-run"],
        temp_dir.path(),
        home_dir.path(),
        &[],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "stdout:\n{stdout}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        stdout.contains(&seed.backend_target.display().to_string()),
        "missing {}\nstdout:\n{stdout}",
        seed.backend_target.display()
    );
    assert!(stdout.contains(&seed.frontend_modules.display().to_string()));
    assert!(stdout.contains(&seed.migration_bak.display().to_string()));
    assert!(stdout.contains(&seed.legacy_llmusage.display().to_string()));
    assert!(stdout.contains("将释放空间: 0.50 MB"));
    assert!(stdout.contains("可清理项: 4 个"));
    assert!(stdout.contains("ccr clean storage"));
    assert!(!stdout.contains("keeper-marker"));
    assert_storage_candidates(&seed, true);
    assert_storage_keepers(&seed);
}

#[test]
fn clean_storage_force_deletes_only_redundant_copies() {
    let temp_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let root = home_dir.path().join(".ccr");
    let seed = seed_storage_tree(&root);

    let output = run_storage(
        &["clean", "storage", "--force"],
        temp_dir.path(),
        home_dir.path(),
        &[],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "stdout:\n{stdout}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("已删除: 4 个"));
    assert!(stdout.contains("释放空间: 2.00 MB"));
    assert!(!stdout.contains("keeper-marker"));
    assert_storage_candidates(&seed, false);
    assert_storage_keepers(&seed);
    assert!(
        root.join("ccr-ui")
            .join("backend")
            .join("src")
            .join("main.rs")
            .exists()
    );
    assert!(
        root.join("ccr-ui")
            .join("frontend")
            .join("src")
            .join("App.tsx")
            .exists()
    );
}

#[test]
fn clean_storage_global_yes_skips_confirmation() {
    let temp_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let root = home_dir.path().join(".ccr");
    let seed = seed_storage_tree(&root);

    let output = run_storage(
        &["-y", "clean", "storage"],
        temp_dir.path(),
        home_dir.path(),
        &[],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "stdout:\n{stdout}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!stdout.contains("确认执行存储清理操作?"));
    assert!(stdout.contains("已删除: 4 个"));
    assert_storage_candidates(&seed, false);
    assert_storage_keepers(&seed);
}

#[test]
fn clean_storage_confirmation_deletes_only_redundant_copies() {
    let temp_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let root = home_dir.path().join(".ccr");
    let seed = seed_storage_tree(&root);

    let output = run_storage_with_input(
        &["clean", "storage"],
        temp_dir.path(),
        home_dir.path(),
        &[],
        "y\n",
    );
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "stdout:\n{stdout}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("确认执行存储清理操作?"));
    assert!(stdout.contains("已删除: 4 个"));
    assert_storage_candidates(&seed, false);
    assert_storage_keepers(&seed);
}

#[test]
fn clean_storage_cancel_deletes_nothing() {
    let temp_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let root = home_dir.path().join(".ccr");
    let seed = seed_storage_tree(&root);

    let output = run_storage_with_input(
        &["clean", "storage"],
        temp_dir.path(),
        home_dir.path(),
        &[],
        "n\n",
    );
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "stdout:\n{stdout}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("确认执行存储清理操作?"));
    assert!(stdout.contains("已取消清理操作"));
    assert!(!stdout.contains("已删除:"));
    assert_storage_candidates(&seed, true);
    assert_storage_keepers(&seed);
}

#[test]
fn clean_storage_empty_root_exits_successfully() {
    let temp_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();

    let output = run_storage(
        &["clean", "storage", "--force"],
        temp_dir.path(),
        home_dir.path(),
        &[],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "stdout:\n{stdout}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("没有可清理的存储项"));
}

#[test]
fn clean_storage_keeps_legacy_directory_when_llmusage_home_matches() {
    let temp_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let root = home_dir.path().join(".ccr");
    let seed = seed_storage_tree(&root);

    let output = run_storage(
        &["clean", "storage", "--force"],
        temp_dir.path(),
        home_dir.path(),
        &[("LLMUSAGE_HOME", seed.legacy_llmusage.as_path())],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "stdout:\n{stdout}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("LLMUSAGE_HOME 指向该目录，已保留"));
    assert!(stdout.contains("已删除: 3 个"));
    assert!(seed.legacy_llmusage.exists());
    assert!(!seed.backend_target.exists());
    assert!(!seed.frontend_modules.exists());
    assert!(!seed.migration_bak.exists());
    assert_storage_keepers(&seed);
}

#[test]
fn clean_storage_uses_ccr_data_dir_before_ccr_root() {
    let temp_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let data_dir = tempfile::tempdir().unwrap();
    let ignored = seed_storage_tree(&home_dir.path().join(".ccr"));
    let selected = seed_storage_tree(data_dir.path());

    let output = run_storage(
        &["clean", "storage", "--force"],
        temp_dir.path(),
        home_dir.path(),
        &[("CCR_DATA_DIR", data_dir.path())],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "stdout:\n{stdout}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains(&data_dir.path().display().to_string()));
    assert_storage_candidates(&selected, false);
    assert_storage_keepers(&selected);
    assert_storage_candidates(&ignored, true);
    assert_storage_keepers(&ignored);
}

#[test]
fn clean_storage_corrupt_config_returns_before_delete() {
    let temp_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let root = home_dir.path().join(".ccr");
    let seed = seed_storage_tree(&root);
    let config_path = root.join("platforms").join("claude").join("profiles.toml");
    write_file(&config_path, "invalid = [");

    let output = run_storage(
        &["clean", "storage", "--force"],
        temp_dir.path(),
        home_dir.path(),
        &[],
    );

    assert_eq!(
        output.status.code(),
        Some(ccr_core::core::error::exit_codes::CONFIG_FORMAT_INVALID)
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("配置格式无效"));
    assert_storage_candidates(&seed, true);
    assert_eq!(fs::read_to_string(&config_path).unwrap(), "invalid = [");
}

#[test]
fn clean_storage_skip_confirmation_setting_deletes_without_prompt() {
    let temp_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let root = home_dir.path().join(".ccr");
    let seed = seed_storage_tree(&root);
    write_file(
        &root.join("platforms").join("claude").join("profiles.toml"),
        "default_config = \"default\"\ncurrent_config = \"\"\n\n[settings]\nskip_confirmation = true\n",
    );

    let output = run_storage(&["clean", "storage"], temp_dir.path(), home_dir.path(), &[]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "stdout:\n{stdout}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("自动确认模式已启用，将跳过确认"));
    assert!(!stdout.contains("确认执行存储清理操作?"));
    assert!(stdout.contains("已删除: 4 个"));
    assert_storage_candidates(&seed, false);
    assert!(
        root.join("platforms")
            .join("claude")
            .join("profiles.toml")
            .exists()
    );
}

#[test]
fn clean_menu_does_not_list_storage() {
    let temp_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();

    let output = run_clean_with_input(&["clean"], temp_dir.path(), home_dir.path(), "q\n");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "stdout:\n{stdout}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("1.planfiles -"));
    assert!(stdout.contains("2.backups -"));
    assert!(!stdout.contains("storage"));
}

#[cfg(unix)]
#[test]
fn clean_storage_does_not_follow_or_delete_symlink_candidates() {
    use std::os::unix::fs::symlink;

    let temp_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let root = home_dir.path().join(".ccr");
    let seed = seed_storage_tree(&root);
    let outside_target = outside.path().join("backend-target");
    let outside_modules = outside.path().join("frontend-modules");
    write_file(&outside_target.join("secret.bin"), "outside-target");
    write_file(&outside_modules.join("secret.js"), "outside-modules");
    fs::remove_dir_all(&seed.backend_target).unwrap();
    fs::remove_dir_all(&seed.frontend_modules).unwrap();
    fs::create_dir_all(seed.backend_target.parent().unwrap()).unwrap();
    fs::create_dir_all(seed.frontend_modules.parent().unwrap()).unwrap();
    symlink(&outside_target, &seed.backend_target).unwrap();
    symlink(&outside_modules, &seed.frontend_modules).unwrap();

    let output = run_storage(
        &["clean", "storage", "--force"],
        temp_dir.path(),
        home_dir.path(),
        &[],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "stdout:\n{stdout}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("符号链接，不跟随也不删除"));
    assert!(seed.backend_target.exists());
    assert!(seed.frontend_modules.exists());
    assert!(!seed.migration_bak.exists());
    assert!(!seed.legacy_llmusage.exists());
    assert_eq!(
        fs::read_to_string(outside_target.join("secret.bin")).unwrap(),
        "outside-target"
    );
    assert_eq!(
        fs::read_to_string(outside_modules.join("secret.js")).unwrap(),
        "outside-modules"
    );
    assert_storage_keepers(&seed);
}
