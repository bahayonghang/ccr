// 清理 CCR 根目录里的构建缓存和停用数据库副本。
// 只处理设计里点名的四类路径，不扫描任意 target 或 node_modules。

use crate::services::ConfigService;
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::logging::ColorOutput;
use std::ffi::OsStr;
use std::fs;
use std::path::{Component, Path, PathBuf};

const USAGE_MIGRATION_PREFIX: &str = "usage.db.pre-migration-";
const USAGE_MIGRATION_SUFFIX: &str = ".bak";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StorageCandidateKind {
    UiBackendTarget,
    UiFrontendModules,
    UsageMigrationBak,
    LegacyLlmUsage,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StorageCandidate {
    kind: StorageCandidateKind,
    path: PathBuf,
    bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StorageProtectionReason {
    Symlink,
    ActiveLlmUsageHome,
}

impl StorageProtectionReason {
    fn message(self) -> &'static str {
        match self {
            Self::Symlink => "符号链接，不跟随也不删除",
            Self::ActiveLlmUsageHome => "LLMUSAGE_HOME 指向该目录，已保留",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StorageProtection {
    path: PathBuf,
    reason: StorageProtectionReason,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct StoragePlan {
    candidates: Vec<StorageCandidate>,
    protections: Vec<StorageProtection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StorageDeleteFailure {
    path: PathBuf,
    message: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct StorageCleanResult {
    deleted_count: usize,
    released_bytes: u64,
    failures: Vec<StorageDeleteFailure>,
}

/// Remove rebuildable caches and retired database copies under the CCR root.
///
/// `force` skips the confirmation prompt. Callers pass global `--yes` through
/// the same flag. `--dry-run` lists candidates and does not delete them.
pub async fn clean_storage_command(dry_run: bool, force: bool) -> Result<()> {
    ColorOutput::title("清理存储");
    println!();

    // 与 clean backups 相同：缺失配置仍询问，其他加载错误在删除前返回。
    let skip_from_config = load_skip_confirmation()?;
    let skip_confirmation = force || skip_from_config;
    if skip_from_config && !force {
        ColorOutput::info("自动确认模式已启用，将跳过确认");
    }

    let root = resolve_storage_root()?;
    ColorOutput::key_value("根目录", &root.display().to_string(), 2);
    if dry_run {
        ColorOutput::info("模拟运行模式(不会实际删除文件)");
    }

    let llmusage_home = env_path("LLMUSAGE_HOME");
    let plan = plan_storage_cleanup(&root, llmusage_home.as_deref())?;
    tracing::debug!(
        candidates = plan.candidates.len(),
        protections = plan.protections.len(),
        dry_run,
        "存储清理候选已生成"
    );
    print_protections(&plan.protections);

    if plan.candidates.is_empty() {
        ColorOutput::info("没有可清理的存储项");
        return Ok(());
    }

    print_candidates(&plan.candidates);
    if dry_run {
        crate::commands::common::print_next_steps(&[("执行实际清理", "ccr clean storage")]);
        return Ok(());
    }

    if !skip_confirmation {
        println!();
        if !super::clean::confirm_cleanup("确认执行存储清理操作?").await? {
            ColorOutput::info("已取消清理操作");
            return Ok(());
        }
    }

    let result = delete_storage_candidates(&plan.candidates);
    println!();
    if result.failures.is_empty() {
        ColorOutput::success(&format!("已删除: {} 个", result.deleted_count));
    } else {
        ColorOutput::info(&format!("已删除: {} 个", result.deleted_count));
    }
    ColorOutput::key_value("释放空间", &format_megabytes(result.released_bytes), 2);
    for failure in &result.failures {
        ColorOutput::key_value("失败", &failure.path.display().to_string(), 2);
    }
    tracing::debug!(
        deleted = result.deleted_count,
        failed = result.failures.len(),
        "存储清理结束"
    );
    storage_result_status(&result)
}

fn load_skip_confirmation() -> Result<bool> {
    let config_service = ConfigService::with_default()?;
    match config_service.load_config() {
        Ok(config) => Ok(config.settings.skip_confirmation),
        Err(CcrError::ConfigMissing(_)) => Ok(false),
        Err(error) => Err(error),
    }
}

fn resolve_storage_root() -> Result<PathBuf> {
    resolve_storage_root_from(
        env_path("CCR_DATA_DIR").as_deref(),
        env_path("CCR_ROOT").as_deref(),
        dirs::home_dir().as_deref(),
    )
}

fn resolve_storage_root_from(
    data_dir: Option<&Path>,
    ccr_root: Option<&Path>,
    home: Option<&Path>,
) -> Result<PathBuf> {
    if let Some(data_dir) = data_dir {
        return Ok(data_dir.to_path_buf());
    }
    if let Some(ccr_root) = ccr_root {
        return Ok(ccr_root.to_path_buf());
    }
    let home = home.ok_or_else(|| CcrError::ConfigError("无法获取用户主目录".into()))?;
    Ok(home.join(".ccr"))
}

fn env_path(key: &str) -> Option<PathBuf> {
    std::env::var_os(key).and_then(|value| {
        if value.is_empty() {
            None
        } else {
            Some(PathBuf::from(value))
        }
    })
}

fn plan_storage_cleanup(root: &Path, llmusage_home: Option<&Path>) -> Result<StoragePlan> {
    let mut plan = StoragePlan::default();
    push_directory_candidate(
        &mut plan,
        root,
        StorageCandidateKind::UiBackendTarget,
        root.join("ccr-ui").join("backend").join("target"),
        false,
    )?;
    push_directory_candidate(
        &mut plan,
        root,
        StorageCandidateKind::UiFrontendModules,
        root.join("ccr-ui").join("frontend").join("node_modules"),
        false,
    )?;
    push_migration_backups(&mut plan, root)?;

    let legacy = root.join("llmusage");
    // 空的 LLMUSAGE_HOME 不保护旧目录。只有规范化后与 <root>/llmusage 相同才保留。
    let protect_home = llmusage_home.is_some_and(|home| same_storage_path(home, &legacy));
    push_directory_candidate(
        &mut plan,
        root,
        StorageCandidateKind::LegacyLlmUsage,
        legacy,
        protect_home,
    )?;

    plan.candidates
        .sort_by(|left, right| left.path.cmp(&right.path));
    plan.protections
        .sort_by(|left, right| left.path.cmp(&right.path));
    Ok(plan)
}

fn push_directory_candidate(
    plan: &mut StoragePlan,
    root: &Path,
    kind: StorageCandidateKind,
    path: PathBuf,
    protect_as_active_home: bool,
) -> Result<()> {
    let Some(metadata) = symlink_metadata_optional(&path)? else {
        return Ok(());
    };
    let file_type = metadata.file_type();
    if file_type.is_symlink() {
        return push_protection(plan, root, path, StorageProtectionReason::Symlink);
    }
    if !file_type.is_dir() {
        return Ok(());
    }
    if protect_as_active_home {
        return push_protection(
            plan,
            root,
            path,
            StorageProtectionReason::ActiveLlmUsageHome,
        );
    }
    let bytes = directory_regular_file_bytes(&path)?;
    push_candidate(plan, root, kind, path, bytes)
}

fn push_migration_backups(plan: &mut StoragePlan, root: &Path) -> Result<()> {
    let analytics = root.join("analytics");
    let Some(metadata) = symlink_metadata_optional(&analytics)? else {
        return Ok(());
    };
    let file_type = metadata.file_type();
    // 不进入 analytics 符号链接，避免把根目录外的文件当成迁移快照。
    if file_type.is_symlink() || !file_type.is_dir() {
        return Ok(());
    }

    let entries = fs::read_dir(&analytics)
        .map_err(|error| storage_io_error("读取存储目录失败", &analytics, &error))?;
    for entry in entries {
        let entry =
            entry.map_err(|error| storage_io_error("读取存储目录失败", &analytics, &error))?;
        if !is_usage_migration_bak(&entry.file_name()) {
            continue;
        }
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| storage_io_error("读取存储项元数据失败", &path, &error))?;
        let file_type = metadata.file_type();
        if file_type.is_symlink() {
            push_protection(plan, root, path, StorageProtectionReason::Symlink)?;
            continue;
        }
        if file_type.is_file() {
            push_candidate(
                plan,
                root,
                StorageCandidateKind::UsageMigrationBak,
                path,
                metadata.len(),
            )?;
        }
    }
    Ok(())
}

fn push_candidate(
    plan: &mut StoragePlan,
    root: &Path,
    kind: StorageCandidateKind,
    path: PathBuf,
    bytes: u64,
) -> Result<()> {
    ensure_inside_root(root, &path)?;
    plan.candidates.push(StorageCandidate { kind, path, bytes });
    Ok(())
}

fn push_protection(
    plan: &mut StoragePlan,
    root: &Path,
    path: PathBuf,
    reason: StorageProtectionReason,
) -> Result<()> {
    ensure_inside_root(root, &path)?;
    plan.protections.push(StorageProtection { path, reason });
    Ok(())
}

fn is_usage_migration_bak(name: &OsStr) -> bool {
    let Some(name) = name.to_str() else {
        return false;
    };
    name.starts_with(USAGE_MIGRATION_PREFIX) && name.ends_with(USAGE_MIGRATION_SUFFIX)
}

fn directory_regular_file_bytes(root: &Path) -> Result<u64> {
    let mut total = 0_u64;
    let mut pending = vec![root.to_path_buf()];
    while let Some(dir) = pending.pop() {
        let entries = fs::read_dir(&dir)
            .map_err(|error| storage_io_error("读取存储目录失败", &dir, &error))?;
        for entry in entries {
            let entry =
                entry.map_err(|error| storage_io_error("读取存储目录失败", &dir, &error))?;
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path)
                .map_err(|error| storage_io_error("读取存储项元数据失败", &path, &error))?;
            let file_type = metadata.file_type();
            // 遍历时不进入符号链接目录，也不把链接本身算进字节数。
            if file_type.is_symlink() {
                continue;
            }
            if file_type.is_dir() {
                pending.push(path);
                continue;
            }
            if file_type.is_file() {
                total = total.saturating_add(metadata.len());
            }
        }
    }
    Ok(total)
}

fn delete_storage_candidates(candidates: &[StorageCandidate]) -> StorageCleanResult {
    let mut result = StorageCleanResult::default();
    for candidate in candidates {
        let deleted = match candidate.kind {
            StorageCandidateKind::UsageMigrationBak => fs::remove_file(&candidate.path),
            StorageCandidateKind::UiBackendTarget
            | StorageCandidateKind::UiFrontendModules
            | StorageCandidateKind::LegacyLlmUsage => fs::remove_dir_all(&candidate.path),
        };
        match deleted {
            Ok(()) => {
                result.deleted_count += 1;
                result.released_bytes = result.released_bytes.saturating_add(candidate.bytes);
            }
            Err(error) => result.failures.push(StorageDeleteFailure {
                path: candidate.path.clone(),
                message: error.to_string(),
            }),
        }
    }
    result
}

fn storage_result_status(result: &StorageCleanResult) -> Result<()> {
    if result.failures.is_empty() {
        return Ok(());
    }
    Err(CcrError::FileIoError(storage_failure_summary(result)))
}

fn storage_failure_summary(result: &StorageCleanResult) -> String {
    let details = result
        .failures
        .iter()
        .map(|failure| format!("{} ({})", failure.path.display(), failure.message))
        .collect::<Vec<_>>()
        .join("; ");
    format!("部分存储项删除失败: {details}")
}

fn ensure_inside_root(root: &Path, path: &Path) -> Result<()> {
    if is_inside_root(root, path) {
        return Ok(());
    }
    Err(CcrError::ValidationError(format!(
        "存储清理路径超出根目录: {}",
        path.display()
    )))
}

fn is_inside_root(root: &Path, path: &Path) -> bool {
    let root = normalized_absolute(root);
    let path = normalized_absolute(path);
    path.starts_with(root)
}

fn same_storage_path(left: &Path, right: &Path) -> bool {
    // 两边都存在时用 canonicalize。有一边不存在时比较去掉 . 和 .. 的绝对路径。
    if let Some((left_canonical, right_canonical)) = canonical_existing_paths(left, right) {
        return left_canonical == right_canonical;
    }
    normalized_absolute(left) == normalized_absolute(right)
}

fn canonical_existing_paths(left: &Path, right: &Path) -> Option<(PathBuf, PathBuf)> {
    if !path_exists(left) || !path_exists(right) {
        return None;
    }
    let left_canonical = fs::canonicalize(left).ok()?;
    let right_canonical = fs::canonicalize(right).ok()?;
    Some((left_canonical, right_canonical))
}

fn path_exists(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok()
}

fn normalized_absolute(path: &Path) -> PathBuf {
    let absolute = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());
    normalize_components(&absolute)
}

fn normalize_components(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => match normalized.components().next_back() {
                Some(Component::Normal(_)) => {
                    normalized.pop();
                }
                Some(Component::ParentDir) | None => normalized.push(Component::ParentDir),
                Some(Component::RootDir | Component::Prefix(_) | Component::CurDir) => {}
            },
            other => normalized.push(other),
        }
    }
    normalized
}

fn symlink_metadata_optional(path: &Path) -> Result<Option<fs::Metadata>> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => Ok(Some(metadata)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(storage_io_error("读取存储项元数据失败", path, &error)),
    }
}

fn storage_io_error(action: &str, path: &Path, error: &std::io::Error) -> CcrError {
    CcrError::FileIoError(format!("{action} {}: {error}", path.display()))
}

fn format_megabytes(bytes: u64) -> String {
    format!("{:.2} MB", bytes as f64 / 1024.0 / 1024.0)
}

fn print_protections(protections: &[StorageProtection]) {
    for protection in protections {
        ColorOutput::warning(protection.reason.message());
        ColorOutput::key_value("路径", &protection.path.display().to_string(), 2);
    }
}

fn print_candidates(candidates: &[StorageCandidate]) {
    for candidate in candidates {
        ColorOutput::key_value("路径", &candidate.path.display().to_string(), 2);
        ColorOutput::key_value("将释放空间", &format_megabytes(candidate.bytes), 2);
    }
    ColorOutput::key_value("可清理项", &format!("{} 个", candidates.len()), 2);
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn write_bytes(path: &Path, contents: &[u8]) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, contents).unwrap();
    }

    #[test]
    fn storage_root_prefers_data_dir_then_ccr_root_then_home() {
        let data_dir = Path::new("/data");
        let ccr_root = Path::new("/ccr");
        let home = Path::new("/home/user");

        assert_eq!(
            resolve_storage_root_from(Some(data_dir), Some(ccr_root), Some(home)).unwrap(),
            data_dir
        );
        assert_eq!(
            resolve_storage_root_from(None, Some(ccr_root), Some(home)).unwrap(),
            ccr_root
        );
        assert_eq!(
            resolve_storage_root_from(None, None, Some(home)).unwrap(),
            home.join(".ccr")
        );
        assert!(resolve_storage_root_from(None, None, None).is_err());
    }

    #[test]
    fn migration_names_match_only_direct_backup_files() {
        assert!(is_usage_migration_bak(OsStr::new(
            "usage.db.pre-migration-v16.20260727.bak"
        )));
        assert!(is_usage_migration_bak(OsStr::new(
            "usage.db.pre-migration-.bak"
        )));
        assert!(!is_usage_migration_bak(OsStr::new("usage.db")));
        assert!(!is_usage_migration_bak(OsStr::new("usage.db-wal")));
        assert!(!is_usage_migration_bak(OsStr::new("usage.db-shm")));
        assert!(!is_usage_migration_bak(OsStr::new("other.bak")));
        assert!(!is_usage_migration_bak(OsStr::new(
            "usage.db.pre-migration-v16.bak.txt"
        )));
    }

    #[test]
    fn plan_selects_only_the_four_redundant_kinds() {
        let temp = tempdir().unwrap();
        let root = temp.path();
        write_bytes(&root.join("ccr-ui/backend/target/cache.bin"), &[b'a'; 10]);
        write_bytes(&root.join("ccr-ui/backend/src/main.rs"), b"source");
        write_bytes(
            &root.join("ccr-ui/frontend/node_modules/pkg/index.js"),
            &[b'b'; 4],
        );
        write_bytes(&root.join("ccr-ui/frontend/src/App.tsx"), b"ui");
        write_bytes(&root.join("analytics/usage.db"), b"live");
        write_bytes(&root.join("analytics/usage.db-wal"), b"wal");
        write_bytes(&root.join("analytics/usage.db-shm"), b"shm");
        write_bytes(
            &root.join("analytics/usage.db.pre-migration-v16.one.bak"),
            &[b'c'; 7],
        );
        write_bytes(
            &root.join("analytics/usage.db.pre-migration-v16.two.bak"),
            &[b'd'; 3],
        );
        write_bytes(
            &root.join("analytics/nested/usage.db.pre-migration-hidden.bak"),
            b"nested",
        );
        write_bytes(&root.join("analytics/other.bak"), b"other");
        fs::create_dir_all(root.join("analytics/usage.db.pre-migration-dir.bak")).unwrap();
        write_bytes(&root.join("llmusage/llmusage.db"), &[b'e'; 6]);
        write_bytes(&root.join("data.db"), b"data");
        write_bytes(&root.join("config.toml"), b"config");
        write_bytes(&root.join("platforms/claude/token.txt"), b"secret");
        write_bytes(&root.join("checkin/state.json"), b"checkin");

        let plan = plan_storage_cleanup(root, None).unwrap();

        assert!(plan.protections.is_empty());
        assert_eq!(plan.candidates.len(), 5);
        let by_kind = |kind: StorageCandidateKind| {
            plan.candidates
                .iter()
                .filter(|candidate| candidate.kind == kind)
                .count()
        };
        assert_eq!(by_kind(StorageCandidateKind::UiBackendTarget), 1);
        assert_eq!(by_kind(StorageCandidateKind::UiFrontendModules), 1);
        assert_eq!(by_kind(StorageCandidateKind::UsageMigrationBak), 2);
        assert_eq!(by_kind(StorageCandidateKind::LegacyLlmUsage), 1);
        assert_eq!(
            plan.candidates
                .iter()
                .find(|candidate| candidate.kind == StorageCandidateKind::UiBackendTarget)
                .unwrap()
                .bytes,
            10
        );
        assert_eq!(
            plan.candidates
                .iter()
                .find(|candidate| candidate.kind == StorageCandidateKind::LegacyLlmUsage)
                .unwrap()
                .bytes,
            6
        );
        assert!(
            plan.candidates
                .iter()
                .all(|candidate| is_inside_root(root, &candidate.path))
        );
        assert!(
            !plan
                .candidates
                .iter()
                .any(|candidate| candidate.path.ends_with("usage.db"))
        );
    }

    #[test]
    fn plan_ignores_missing_paths() {
        let temp = tempdir().unwrap();
        let plan = plan_storage_cleanup(temp.path(), None).unwrap();
        assert!(plan.candidates.is_empty());
        assert!(plan.protections.is_empty());
    }

    #[test]
    fn active_llmusage_home_protects_only_that_directory() {
        let temp = tempdir().unwrap();
        let root = temp.path();
        write_bytes(&root.join("ccr-ui/backend/target/cache.bin"), b"cache");
        write_bytes(&root.join("llmusage/llmusage.db"), b"old");
        let legacy = root.join("llmusage");

        let plan = plan_storage_cleanup(root, Some(&legacy)).unwrap();

        assert_eq!(plan.protections.len(), 1);
        assert_eq!(
            plan.protections[0].reason,
            StorageProtectionReason::ActiveLlmUsageHome
        );
        assert!(
            !plan
                .candidates
                .iter()
                .any(|candidate| candidate.kind == StorageCandidateKind::LegacyLlmUsage)
        );
        assert!(
            plan.candidates
                .iter()
                .any(|candidate| candidate.kind == StorageCandidateKind::UiBackendTarget)
        );
    }

    #[test]
    fn different_llmusage_home_does_not_protect_legacy_directory() {
        let temp = tempdir().unwrap();
        let root = temp.path();
        write_bytes(&root.join("llmusage/llmusage.db"), b"old");
        let other = temp.path().join("active-home");
        fs::create_dir_all(&other).unwrap();

        let plan = plan_storage_cleanup(root, Some(&other)).unwrap();

        assert!(plan.protections.is_empty());
        assert!(
            plan.candidates
                .iter()
                .any(|candidate| candidate.kind == StorageCandidateKind::LegacyLlmUsage)
        );
    }

    #[test]
    fn same_storage_path_uses_canonicalize_or_normalized_absolute_path() {
        let temp = tempdir().unwrap();
        let dir = temp.path().join("llmusage");
        fs::create_dir_all(&dir).unwrap();
        let via_parent = temp.path().join("other").join("..").join("llmusage");
        fs::create_dir_all(temp.path().join("other")).unwrap();
        assert!(same_storage_path(&dir, &via_parent));

        let missing = temp.path().join("missing");
        let missing_via_parent = temp.path().join("missing").join("..").join("missing");
        assert!(!path_exists(&missing));
        assert!(same_storage_path(&missing, &missing_via_parent));

        let elsewhere = temp.path().join("elsewhere");
        fs::create_dir_all(&elsewhere).unwrap();
        assert!(!same_storage_path(&dir, &elsewhere));
    }

    #[test]
    fn paths_outside_the_root_are_rejected() {
        let temp = tempdir().unwrap();
        let root = temp.path().join("root");
        fs::create_dir_all(&root).unwrap();
        assert!(is_inside_root(&root, &root.join("ccr-ui/backend/target")));
        assert!(!is_inside_root(&root, &temp.path().join("outside")));
    }

    #[test]
    fn storage_result_status_returns_error_and_keeps_success_counts() {
        let result = StorageCleanResult {
            deleted_count: 2,
            released_bytes: 30,
            failures: vec![StorageDeleteFailure {
                path: PathBuf::from("ccr-ui/backend/target"),
                message: "access denied".to_string(),
            }],
        };

        match storage_result_status(&result) {
            Err(CcrError::FileIoError(message)) => {
                assert!(message.contains("access denied"), "{message}");
                assert!(message.contains("ccr-ui/backend/target"), "{message}");
            }
            other => panic!("unexpected status: {other:?}"),
        }
        assert_eq!(result.deleted_count, 2);
        assert_eq!(result.released_bytes, 30);
    }

    #[test]
    fn storage_result_status_is_ok_when_no_candidate_fails() {
        let result = StorageCleanResult {
            deleted_count: 1,
            released_bytes: 4,
            failures: Vec::new(),
        };
        assert!(storage_result_status(&result).is_ok());
    }

    #[test]
    fn delete_continues_after_one_candidate_fails() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("ccr-ui/backend/target");
        write_bytes(&target.join("cache.bin"), b"abcd");
        let missing = temp
            .path()
            .join("analytics/usage.db.pre-migration-missing.bak");
        let candidates = vec![
            StorageCandidate {
                kind: StorageCandidateKind::UsageMigrationBak,
                path: missing,
                bytes: 1,
            },
            StorageCandidate {
                kind: StorageCandidateKind::UiBackendTarget,
                path: target.clone(),
                bytes: 4,
            },
        ];

        let result = delete_storage_candidates(&candidates);

        assert_eq!(result.deleted_count, 1);
        assert_eq!(result.released_bytes, 4);
        assert_eq!(result.failures.len(), 1);
        assert!(!target.exists());
        assert!(storage_result_status(&result).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn plan_records_symlink_candidates_without_following_them() {
        use std::os::unix::fs::symlink;

        let temp = tempdir().unwrap();
        let root = temp.path();
        let outside = temp.path().join("outside");
        write_bytes(&outside.join("secret.bin"), &[b'z'; 100]);
        let target = root.join("ccr-ui/backend/target");
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        symlink(&outside, &target).unwrap();

        let linked_file = root.join("ccr-ui/frontend/node_modules");
        fs::create_dir_all(&linked_file).unwrap();
        write_bytes(&linked_file.join("kept.js"), b"mod");
        let outside_file = temp.path().join("outside-file.bin");
        write_bytes(&outside_file, &[b'q'; 40]);
        symlink(&outside_file, &linked_file.join("linked.bin")).unwrap();
        let outside_dir = temp.path().join("outside-dir");
        write_bytes(&outside_dir.join("nested.bin"), &[b'n'; 20]);
        symlink(&outside_dir, &linked_file.join("linked-dir")).unwrap();

        let bak_link = root.join("analytics/usage.db.pre-migration-link.bak");
        fs::create_dir_all(bak_link.parent().unwrap()).unwrap();
        symlink(&outside_file, &bak_link).unwrap();

        let plan = plan_storage_cleanup(root, None).unwrap();

        assert!(
            !plan
                .candidates
                .iter()
                .any(|candidate| candidate.kind == StorageCandidateKind::UiBackendTarget)
        );
        let modules = plan
            .candidates
            .iter()
            .find(|candidate| candidate.kind == StorageCandidateKind::UiFrontendModules)
            .unwrap();
        assert_eq!(modules.bytes, 3);
        assert!(plan.protections.iter().any(|protection| {
            protection.reason == StorageProtectionReason::Symlink && protection.path == target
        }));
        assert!(plan.protections.iter().any(|protection| {
            protection.reason == StorageProtectionReason::Symlink && protection.path == bak_link
        }));
        assert!(outside.join("secret.bin").exists());
    }
}
