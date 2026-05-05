// 🧹 clean 命令实现
// 📅 支持清理旧备份文件，也支持递归清理规划文件

#![allow(clippy::unused_async)]

use crate::cli::definitions::DEFAULT_CLEAN_BACKUP_DAYS;
use crate::services::{BackupService, ConfigService};
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::logging::ColorOutput;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const PLANFILES_TARGETS: [&str; 3] = ["task_plan.md", "findings.md", "progress.md"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CleanTarget {
    Planfiles,
    Backups,
}

#[derive(Debug, Clone, Copy)]
struct CleanTargetSpec {
    target: CleanTarget,
    name: &'static str,
    description: &'static str,
    default_selected: bool,
}

const CLEAN_TARGETS: [CleanTargetSpec; 2] = [
    CleanTargetSpec {
        target: CleanTarget::Planfiles,
        name: "planfiles",
        description: "清理 task_plan.md / findings.md / progress.md",
        default_selected: true,
    },
    CleanTargetSpec {
        target: CleanTarget::Backups,
        name: "backups",
        description: "清理 7 天前旧备份",
        default_selected: false,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CleanMenuSelection {
    Target(CleanTarget),
    Cancel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PlanfileMatch {
    path: PathBuf,
    size: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct PlanfilesCleanResult {
    matches: Vec<PlanfileMatch>,
    total_size: u64,
}

impl PlanfilesCleanResult {
    fn matched_count(&self) -> usize {
        self.matches.len()
    }
}

/// 🧹 交互式清理入口
///
/// 裸 `ccr clean` 进入编号菜单。回车选择默认编号，`-y/--yes`
/// 会执行默认编号并跳过目标命令的确认。
pub async fn clean_menu_command(auto_yes: bool) -> Result<()> {
    ColorOutput::title("交互式清理");
    println!();

    let default_index = default_clean_menu_index(&CLEAN_TARGETS);
    render_clean_target_menu(default_index);

    let (target, force_target_confirmation) = if auto_yes {
        let spec = &CLEAN_TARGETS[default_index];
        ColorOutput::info(&format!(
            "⚡ 自动确认模式已启用，将执行默认清理项: {}",
            spec.name
        ));
        (spec.target, true)
    } else {
        match prompt_clean_target_selection(default_index).await? {
            Some(target) => (target, false),
            None => {
                println!();
                ColorOutput::info("未选择任何清理项");
                return Ok(());
            }
        }
    };

    println!();
    run_clean_target(target, force_target_confirmation).await?;

    Ok(())
}

/// 🧹 清理旧备份文件
///
/// 执行流程:
/// 1. 📁 扫描备份目录 (~/.claude/backups/)
/// 2. 🔍 识别 .bak 文件
/// 3. 📅 检查文件修改时间
/// 4. 🗑️ 删除超过指定天数的文件
/// 5. 📊 统计清理结果(文件数、释放空间)
///
/// 参数:
/// - days: 保留天数(删除 N 天前的文件)
/// - dry_run: 模拟运行(不实际删除)
/// - force: 跳过确认提示（危险操作）
pub async fn clean_backups_command(days: u64, dry_run: bool, force: bool) -> Result<()> {
    ColorOutput::title("清理备份文件");
    println!();

    // ⚡ 检查自动确认模式：--force 参数 OR 配置文件中的 skip_confirmation
    let config_service = ConfigService::with_default()?;
    let config = config_service.load_config()?;
    let skip_confirmation = force || config.settings.skip_confirmation;

    if config.settings.skip_confirmation && !force {
        ColorOutput::info("⚡ 自动确认模式已启用，将跳过确认");
    }

    // 使用 BackupService
    let service = BackupService::with_default()?;
    let backup_dir = service.backup_dir();

    if !backup_dir.exists() {
        ColorOutput::info("备份目录不存在,无需清理");
        return Ok(());
    }

    ColorOutput::info(&format!("备份目录: {}", backup_dir.display()));
    ColorOutput::info(&format!("清理策略: 删除 {} 天前的备份", days));

    if dry_run {
        ColorOutput::warning("⚠ 模拟运行模式(不会实际删除文件)");
    }

    // 🚨 非 dry-run 模式需要确认（除非 YOLO 模式）
    if !dry_run && !skip_confirmation {
        println!();
        ColorOutput::warning("⚠️  警告: 即将删除旧备份文件！");
        ColorOutput::info("提示: 使用 --dry-run 参数可以先预览将要删除的文件");
        println!();

        if !confirm_cleanup("确认执行清理操作?").await? {
            ColorOutput::info("已取消清理操作");
            return Ok(());
        }
    }

    println!();
    ColorOutput::separator();
    println!();

    // 使用 BackupService 清理
    let status_msg = if skip_confirmation && !dry_run {
        "⚡ 执行清理 (自动确认模式)"
    } else {
        "执行清理"
    };
    if !dry_run {
        ColorOutput::step(status_msg);
    }
    let result = service.clean_old_backups(days, dry_run)?;

    println!();
    ColorOutput::separator();
    println!();

    // 显示结果
    if result.deleted_count > 0 || result.skipped_count > 0 {
        ColorOutput::title("清理摘要");
        println!();

        if result.deleted_count > 0 {
            if dry_run {
                ColorOutput::info(&format!("将删除文件: {} 个", result.deleted_count));
            } else {
                ColorOutput::success(&format!("✓ 已删除文件: {} 个", result.deleted_count));
            }
        }

        if result.skipped_count > 0 {
            ColorOutput::info(&format!("保留文件: {} 个", result.skipped_count));
        }

        if result.total_size > 0 {
            let size_mb = result.total_size as f64 / 1024.0 / 1024.0;
            if dry_run {
                ColorOutput::info(&format!("将释放空间: {:.2} MB", size_mb));
            } else {
                ColorOutput::success(&format!("✓ 释放空间: {:.2} MB", size_mb));
            }
        }
    } else {
        ColorOutput::success("✓ 没有需要清理的文件");
    }

    if dry_run {
        println!();
        ColorOutput::info("提示: 运行 'ccr clean backups' (不带 --dry-run) 执行实际清理");
    }

    Ok(())
}

/// 🧹 递归清理当前目录下的规划文件
pub async fn clean_planfiles_command(dry_run: bool, force: bool) -> Result<()> {
    /*
     * ========================================================================
     * 步骤1：扫描规划文件
     * ========================================================================
     * 目标目录：当前工作目录
     * 操作：
     * 1) 递归扫描 task_plan.md、findings.md、progress.md
     * 2) 汇总命中路径和空间占用
     */
    tracing::info!(dry_run, force, "开始扫描规划文件");

    let current_dir = std::env::current_dir()
        .map_err(|e| CcrError::FileIoError(format!("获取当前目录失败: {}", e)))?;

    ColorOutput::title("清理规划文件");
    println!();
    ColorOutput::info(&format!("扫描目录: {}", current_dir.display()));
    ColorOutput::info("目标文件: task_plan.md, findings.md, progress.md");

    if dry_run {
        ColorOutput::warning("⚠ 模拟运行模式(不会实际删除文件)");
    }

    let result = scan_planfiles(&current_dir)?;
    if result.matched_count() == 0 {
        ColorOutput::success("✓ 没有找到需要清理的规划文件");
        tracing::info!("当前目录下没有规划文件");
        return Ok(());
    }

    println!();
    ColorOutput::separator();
    println!();

    ColorOutput::step("命中文件");
    for entry in &result.matches {
        // 1.1 输出命中路径，便于 dry-run 和确认前核对
        ColorOutput::info(&format!(
            "命中: {}",
            display_match_path(&current_dir, &entry.path)
        ));
    }

    println!();
    ColorOutput::info(&format!("命中数量: {} 个", result.matched_count()));
    if result.total_size > 0 {
        ColorOutput::info(&format!(
            "{}: {:.2} MB",
            if dry_run {
                "预计释放空间"
            } else {
                "待释放空间"
            },
            result.total_size as f64 / 1024.0 / 1024.0
        ));
    }

    // 1.2 非 dry-run 模式下确认删除
    if !dry_run && !force {
        println!();
        ColorOutput::warning("⚠️  警告: 即将删除当前目录下的规划文件！");
        ColorOutput::info("提示: 使用 --dry-run 参数可以先预览将要删除的文件");
        println!();

        if !confirm_cleanup_default_yes("确认执行规划文件清理操作?").await? {
            ColorOutput::info("已取消清理操作");
            return Ok(());
        }
    }

    if dry_run {
        println!();
        ColorOutput::info("提示: 运行 'ccr clean planfiles' 执行实际清理");
        tracing::info!(matched = result.matched_count(), "规划文件预览完成");
        return Ok(());
    }

    println!();
    ColorOutput::separator();
    println!();

    /*
     * ========================================================================
     * 步骤2：删除命中的规划文件
     * ========================================================================
     * 目标文件：步骤1命中的规划文件
     * 操作：
     * 1) 逐个删除命中的规划文件
     * 2) 输出最终统计结果
     */
    tracing::info!(matched = result.matched_count(), "开始删除规划文件");

    let status_msg = if force {
        "⚡ 执行清理 (自动确认模式)"
    } else {
        "执行清理"
    };
    ColorOutput::step(status_msg);

    delete_planfiles(&result)?;

    println!();
    ColorOutput::separator();
    println!();

    ColorOutput::title("清理摘要");
    println!();
    ColorOutput::success(&format!("✓ 已删除文件: {} 个", result.matched_count()));
    if result.total_size > 0 {
        ColorOutput::success(&format!(
            "✓ 释放空间: {:.2} MB",
            result.total_size as f64 / 1024.0 / 1024.0
        ));
    }

    tracing::info!(
        deleted_count = result.matched_count(),
        total_size = result.total_size,
        "规划文件清理完成"
    );
    Ok(())
}

async fn run_clean_target(target: CleanTarget, force: bool) -> Result<()> {
    match target {
        CleanTarget::Planfiles => clean_planfiles_command(false, force).await,
        CleanTarget::Backups => {
            clean_backups_command(DEFAULT_CLEAN_BACKUP_DAYS, false, force).await
        }
    }
}

fn default_clean_menu_index(targets: &[CleanTargetSpec]) -> usize {
    targets
        .iter()
        .position(|spec| spec.default_selected)
        .unwrap_or(0)
}

fn render_clean_target_menu(default_index: usize) {
    println!(
        "清理内容（输入编号执行，回车 = {}，输入 q 取消）",
        default_index + 1
    );

    for (index, spec) in CLEAN_TARGETS.iter().enumerate() {
        println!("{}.{} - {}", index + 1, spec.name, spec.description);
    }
}

async fn prompt_clean_target_selection(default_index: usize) -> Result<Option<CleanTarget>> {
    loop {
        let input =
            read_prompt_line(format!("请选择清理内容 [默认 {}]: ", default_index + 1)).await?;

        match parse_clean_menu_selection(&input, &CLEAN_TARGETS, default_index) {
            Some(CleanMenuSelection::Target(target)) => return Ok(Some(target)),
            Some(CleanMenuSelection::Cancel) => return Ok(None),
            None => {
                ColorOutput::warning(&format!(
                    "无效编号，请输入 1-{}，或输入 q 取消",
                    CLEAN_TARGETS.len()
                ));
            }
        }
    }
}

async fn confirm_cleanup(question: &str) -> Result<bool> {
    confirm_cleanup_with_default(question, false).await
}

async fn confirm_cleanup_default_yes(question: &str) -> Result<bool> {
    confirm_cleanup_with_default(question, true).await
}

async fn confirm_cleanup_with_default(question: &str, default_yes: bool) -> Result<bool> {
    let default_hint = if default_yes { "Y/n" } else { "y/N" };
    confirm_yes_no(format!("{question} ({default_hint}): "), default_yes).await
}

async fn confirm_yes_no(prompt: String, default_yes: bool) -> Result<bool> {
    let input = read_prompt_line(prompt).await?;
    Ok(parse_yes_no_answer(input.trim(), default_yes))
}

async fn read_prompt_line(prompt: String) -> Result<String> {
    tokio::task::spawn_blocking({
        move || -> std::io::Result<String> {
            use std::io::{self, Write};

            print!("{prompt}");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            Ok(input)
        }
    })
    .await
    .map_err(|e| CcrError::FileIoError(format!("读取确认输入失败: {}", e)))?
    .map_err(|e| CcrError::FileIoError(format!("读取确认输入失败: {}", e)))
}

fn parse_clean_menu_selection(
    input: &str,
    targets: &[CleanTargetSpec],
    default_index: usize,
) -> Option<CleanMenuSelection> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return targets
            .get(default_index)
            .map(|spec| CleanMenuSelection::Target(spec.target));
    }

    if trimmed.eq_ignore_ascii_case("q")
        || trimmed.eq_ignore_ascii_case("quit")
        || trimmed.eq_ignore_ascii_case("cancel")
        || trimmed == "0"
    {
        return Some(CleanMenuSelection::Cancel);
    }

    let selected_number = trimmed.parse::<usize>().ok()?;
    let selected_index = selected_number.checked_sub(1)?;
    targets
        .get(selected_index)
        .map(|spec| CleanMenuSelection::Target(spec.target))
}

fn parse_yes_no_answer(input: &str, default_yes: bool) -> bool {
    if input.is_empty() {
        return default_yes;
    }

    if input.eq_ignore_ascii_case("y") || input.eq_ignore_ascii_case("yes") {
        return true;
    }

    if input.eq_ignore_ascii_case("n") || input.eq_ignore_ascii_case("no") {
        return false;
    }

    false
}

fn scan_planfiles(root: &Path) -> Result<PlanfilesCleanResult> {
    let mut matches = Vec::new();
    let mut total_size = 0_u64;

    // 1.1 默认不跟随符号链接，避免跨目录误删
    for entry in WalkDir::new(root).follow_links(false) {
        let entry = entry.map_err(|e| CcrError::FileIoError(format!("递归扫描目录失败: {}", e)))?;
        if !entry.file_type().is_file() || !is_planfile_target(entry.path()) {
            continue;
        }

        // 1.2 读取文件大小，后续直接复用统计结果
        let metadata = entry
            .metadata()
            .map_err(|e| CcrError::FileIoError(format!("读取文件元数据失败: {}", e)))?;
        let size = metadata.len();

        total_size += size;
        matches.push(PlanfileMatch {
            path: entry.path().to_path_buf(),
            size,
        });
    }

    matches.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(PlanfilesCleanResult {
        matches,
        total_size,
    })
}

fn delete_planfiles(result: &PlanfilesCleanResult) -> Result<()> {
    for entry in &result.matches {
        // 2.1 逐个删除，任何失败都保留明确错误
        fs::remove_file(&entry.path)
            .map_err(|e| CcrError::FileIoError(format!("删除文件失败: {}", e)))?;
    }

    Ok(())
}

fn is_planfile_target(path: &Path) -> bool {
    path.file_name()
        .and_then(|value| value.to_str())
        .map(|name| PLANFILES_TARGETS.contains(&name))
        .unwrap_or(false)
}

fn display_match_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn scan_planfiles_matches_nested_targets() {
        let temp_dir = tempdir().unwrap();
        let nested = temp_dir.path().join("nested").join("child");
        fs::create_dir_all(&nested).unwrap();

        fs::write(temp_dir.path().join("task_plan.md"), "root task").unwrap();
        fs::write(nested.join("findings.md"), "nested findings").unwrap();
        fs::write(nested.join("progress.md"), "nested progress").unwrap();
        fs::write(temp_dir.path().join("README.md"), "keep").unwrap();

        let result = scan_planfiles(temp_dir.path()).unwrap();

        assert_eq!(result.matched_count(), 3);
        assert_eq!(
            result
                .matches
                .iter()
                .map(|entry| entry.path.file_name().unwrap().to_str().unwrap())
                .collect::<Vec<_>>(),
            vec!["findings.md", "progress.md", "task_plan.md"]
        );
    }

    #[test]
    fn delete_planfiles_removes_only_target_files() {
        let temp_dir = tempdir().unwrap();
        let nested = temp_dir.path().join("nested");
        fs::create_dir_all(&nested).unwrap();

        fs::write(temp_dir.path().join("task_plan.md"), "root task").unwrap();
        fs::write(nested.join("findings.md"), "nested findings").unwrap();
        fs::write(nested.join("notes.md"), "keep").unwrap();

        let result = scan_planfiles(temp_dir.path()).unwrap();
        delete_planfiles(&result).unwrap();

        assert!(!temp_dir.path().join("task_plan.md").exists());
        assert!(!nested.join("findings.md").exists());
        assert!(nested.join("notes.md").exists());
    }

    #[test]
    fn display_match_path_prefers_relative_output() {
        let root = PathBuf::from("D:/workspace/project");
        let path = root.join("nested").join("task_plan.md");

        assert_eq!(
            display_match_path(&root, &path),
            PathBuf::from("nested")
                .join("task_plan.md")
                .display()
                .to_string()
        );
    }

    #[test]
    fn yes_no_prompt_defaults_to_yes_for_empty_input() {
        assert!(parse_yes_no_answer("", true));
        assert!(!parse_yes_no_answer("", false));
        assert!(parse_yes_no_answer("yes", false));
        assert!(!parse_yes_no_answer("n", true));
    }

    #[test]
    fn clean_menu_selection_defaults_to_planfiles() {
        let default_index = default_clean_menu_index(&CLEAN_TARGETS);

        assert_eq!(
            parse_clean_menu_selection("", &CLEAN_TARGETS, default_index),
            Some(CleanMenuSelection::Target(CleanTarget::Planfiles))
        );
    }

    #[test]
    fn clean_menu_selection_parses_registered_target_numbers() {
        let default_index = default_clean_menu_index(&CLEAN_TARGETS);

        assert_eq!(
            parse_clean_menu_selection("2", &CLEAN_TARGETS, default_index),
            Some(CleanMenuSelection::Target(CleanTarget::Backups))
        );
    }

    #[test]
    fn clean_menu_selection_accepts_cancel_inputs() {
        let default_index = default_clean_menu_index(&CLEAN_TARGETS);

        assert_eq!(
            parse_clean_menu_selection("q", &CLEAN_TARGETS, default_index),
            Some(CleanMenuSelection::Cancel)
        );
        assert_eq!(
            parse_clean_menu_selection("0", &CLEAN_TARGETS, default_index),
            Some(CleanMenuSelection::Cancel)
        );
    }

    #[test]
    fn clean_menu_selection_rejects_unknown_numbers() {
        let default_index = default_clean_menu_index(&CLEAN_TARGETS);

        assert_eq!(
            parse_clean_menu_selection("3", &CLEAN_TARGETS, default_index),
            None
        );
    }

    #[test]
    fn clean_targets_register_planfiles_and_backups_with_planfiles_default() {
        assert_eq!(
            CLEAN_TARGETS
                .iter()
                .map(|spec| spec.name)
                .collect::<Vec<_>>(),
            vec!["planfiles", "backups"]
        );
        assert_eq!(default_clean_menu_index(&CLEAN_TARGETS), 0);
    }

    #[test]
    fn backup_service_clean() {
        let temp_dir = tempdir().unwrap();
        let backup_dir = temp_dir.path().to_path_buf();

        // 创建测试备份文件
        let old_file = backup_dir.join("old.bak");
        let new_file = backup_dir.join("new.bak");
        let other_file = backup_dir.join("other.txt");

        File::create(&old_file).unwrap().write_all(b"old").unwrap();
        File::create(&new_file).unwrap().write_all(b"new").unwrap();
        File::create(&other_file)
            .unwrap()
            .write_all(b"other")
            .unwrap();

        // 设置旧文件的修改时间为 10 天前
        let old_time =
            std::time::SystemTime::now() - std::time::Duration::from_secs(10 * 24 * 60 * 60);
        filetime::set_file_mtime(&old_file, filetime::FileTime::from_system_time(old_time))
            .unwrap();

        let service = BackupService::new(backup_dir);

        // 清理 7 天前的文件(dry run)
        let result = service.clean_old_backups(7, true).unwrap();
        assert_eq!(result.deleted_count, 1); // old.bak 应该被标记删除
        assert_eq!(result.skipped_count, 1); // new.bak 应该被保留
        assert!(old_file.exists()); // dry run 不应实际删除

        // 实际清理
        let result = service.clean_old_backups(7, false).unwrap();
        assert_eq!(result.deleted_count, 1);
        assert!(!old_file.exists()); // 应该被删除
        assert!(new_file.exists()); // 应该保留
        assert!(other_file.exists()); // 非 .bak 文件应该保留
    }

    #[test]
    fn backup_service_scan() {
        let temp_dir = tempdir().unwrap();
        let backup_dir = temp_dir.path().to_path_buf();

        // 创建多个备份文件
        for i in 0..5 {
            let filename = format!("backup{}.bak", i);
            File::create(backup_dir.join(&filename))
                .unwrap()
                .write_all(format!("content{}", i).as_bytes())
                .unwrap();
        }

        let service = BackupService::new(backup_dir);
        let backups = service.scan_backup_directory().unwrap();

        assert_eq!(backups.len(), 5);
        // 验证按修改时间排序
        for i in 0..backups.len() - 1 {
            assert!(backups[i].modified >= backups[i + 1].modified);
        }
    }

    #[cfg(unix)]
    #[test]
    fn scan_planfiles_does_not_follow_symlink_directories() {
        use std::os::unix::fs::symlink;

        let temp_dir = tempdir().unwrap();
        let external_dir = tempdir().unwrap();
        let linked = temp_dir.path().join("linked");

        fs::write(external_dir.path().join("task_plan.md"), "outside task").unwrap();
        symlink(external_dir.path(), &linked).unwrap();

        let result = scan_planfiles(temp_dir.path()).unwrap();

        assert_eq!(result.matched_count(), 0);
        assert!(external_dir.path().join("task_plan.md").exists());
    }
}
