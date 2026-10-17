//! profile open 子命令实现
//!
//! 用 `$VISUAL` 或 `$EDITOR` 打开目录的规范 TOML。编辑器退出后把有变化的内容写回
//! SQLite，并删除临时文件。未设置编辑器时命令失败。

use crate::models::{Platform, PlatformPaths};
use ccr_config::{ConfigManager, register_platform_if_missing};
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::guarded_write::restrict_owner_only;
use ccr_core::core::logging::ColorOutput;
use serde::Serialize;
use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

// --- 模板集中 ---

/// 返回指定平台的内嵌 profiles.toml 模板（编译期注入）
pub fn template_for(platform: Platform) -> &'static str {
    match platform {
        Platform::Claude => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/claude/profiles.example.toml"
        )),
        Platform::Codex => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/codex/profiles.toml"
        )),
        Platform::Grok => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/grok/profiles.toml"
        )),
        _ => unreachable!("template_for 只支持三个 auth-profile 平台"),
    }
}

// --- 编辑器解析 ---

struct EditorTarget {
    program: String,
    args: Vec<String>,
    /// "$VISUAL" 或 "$EDITOR"，用于输出提示
    source: &'static str,
}

/// 读取环境变量决定编辑器。两者都未设置时返回 `None`。
fn resolve_editor() -> Option<EditorTarget> {
    resolve_editor_from(
        std::env::var("VISUAL").ok().as_deref(),
        std::env::var("EDITOR").ok().as_deref(),
    )
}

/// 纯函数版本，便于单元测试（不直接读进程环境变量）
fn resolve_editor_from(visual: Option<&str>, editor: Option<&str>) -> Option<EditorTarget> {
    for (value, source) in [(visual, "$VISUAL"), (editor, "$EDITOR")] {
        if let Some(s) = value {
            let trimmed = s.trim();
            if !trimmed.is_empty() {
                let mut parts = trimmed.split_whitespace();
                let program = parts.next().unwrap_or_default().to_string();
                if program.is_empty() {
                    continue;
                }
                let args: Vec<String> = parts.map(str::to_string).collect();
                return Some(EditorTarget {
                    program,
                    args,
                    source,
                });
            }
        }
    }
    None
}

fn editor_required() -> CcrError {
    CcrError::ConfigError("未设置 $VISUAL 或 $EDITOR。请设置其中一个后重试".into())
}

/// 阻塞等待编辑器退出。非零退出码返回 ExternalCommandError。
fn spawn_editor(target: &EditorTarget, path: &Path) -> Result<()> {
    let status = std::process::Command::new(&target.program)
        .args(&target.args)
        .arg(path)
        .status()
        .map_err(|e| CcrError::ExternalCommandError(format!("{} 无法启动: {e}", target.source)))?;
    if !status.success() {
        return Err(CcrError::ExternalCommandError(format!(
            "{} 退出码: {}",
            target.source,
            status.code().unwrap_or(-1)
        )));
    }
    Ok(())
}

struct OwnedTemp {
    path: PathBuf,
}

impl OwnedTemp {
    /// 在系统临时目录创建编辑文件。先收紧权限，再写入可能含密钥的文本。
    fn create(content: &str) -> Result<Self> {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let path = std::env::temp_dir().join(format!(
            "ccr-profile-edit-{}-{nanos}.toml",
            std::process::id()
        ));
        if path.file_name().and_then(|name| name.to_str()) == Some("profiles.toml") {
            return Err(CcrError::ConfigError("临时文件名无效".into()));
        }
        let mut options = OpenOptions::new();
        options.read(true).write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let file = options.open(&path).map_err(|error| {
            CcrError::FileIoError(format!("创建临时文件失败 {}: {error}", path.display()))
        })?;
        drop(file);
        if let Err(error) = restrict_owner_only(&path) {
            let _ = fs::remove_file(&path);
            return Err(error);
        }
        fs::write(&path, content).map_err(|error| {
            let _ = fs::remove_file(&path);
            CcrError::FileIoError(format!("写入临时文件失败 {}: {error}", path.display()))
        })?;
        Ok(Self { path })
    }
}

impl Drop for OwnedTemp {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

// --- 输出结构 ---

#[derive(Serialize)]
struct ProfileOpenOutput<'a> {
    ok: bool,
    platform: &'a str,
    profiles_file: String,
    created: bool,
    registered: bool,
    /// "$VISUAL" 或 "$EDITOR"
    editor: &'a str,
}

#[derive(Debug)]
struct OpenReport {
    profiles_db: PathBuf,
    created: bool,
    registered: bool,
    editor: &'static str,
}

// --- 命令入口 ---

/// 用平台对应的 `parse_platform` 解析名称并检查支持性
fn parse_platform_for_open(platform_name: &str) -> Result<Platform> {
    use std::str::FromStr;
    let platform = Platform::from_str(platform_name)
        .map_err(|_| CcrError::PlatformNotFound(platform_name.to_string()))?;
    if !Platform::auth_profile_supported().contains(&platform) {
        return Err(CcrError::PlatformNotSupported(platform_name.to_string()));
    }
    Ok(platform)
}

fn prepare_catalog(paths: &PlatformPaths, template: &str) -> Result<(String, String, bool)> {
    let manager = ConfigManager::new(&paths.profiles_file);
    match manager.canonical_toml()? {
        Some((content, revision)) => Ok((content, revision, false)),
        None => {
            manager.replace_toml(template, "")?;
            let (content, revision) = manager
                .canonical_toml()?
                .ok_or_else(|| CcrError::ConfigError("配置目录创建后无法读取".into()))?;
            Ok((content, revision, true))
        }
    }
}

fn open_profile(platform_name: &str, json: bool) -> Result<OpenReport> {
    let editor = resolve_editor().ok_or_else(editor_required)?;
    let platform = parse_platform_for_open(platform_name)?;
    let template = template_for(platform);
    let paths = PlatformPaths::new(platform)?;
    paths.ensure_directories()?;
    let (content, revision, created) = prepare_catalog(&paths, template)?;
    let registered = register_platform_if_missing(platform_name, platform.display_name())?;
    let temp = OwnedTemp::create(&content)?;
    if temp.path.starts_with(&paths.platform_dir) {
        return Err(CcrError::ConfigError("临时文件不能放在平台目录".into()));
    }

    if !json {
        if created {
            ColorOutput::success(&format!(
                "已创建 profiles 目录: {}",
                paths.profiles_db().display()
            ));
        }
        if registered {
            ColorOutput::success(&format!("已注册平台: {platform_name}"));
        }
        ColorOutput::step(&format!(
            "正在用 {} 打开: {}",
            editor.source,
            temp.path.display()
        ));
    }

    spawn_editor(&editor, &temp.path)?;
    let edited = fs::read_to_string(&temp.path).map_err(|error| {
        CcrError::FileIoError(format!("读取临时文件失败 {}: {error}", temp.path.display()))
    })?;
    if edited != content {
        ConfigManager::new(&paths.profiles_file).replace_toml(&edited, &revision)?;
    }

    Ok(OpenReport {
        profiles_db: paths.profiles_db(),
        created,
        registered,
        editor: editor.source,
    })
}

/// 打开指定平台的 profile 目录。
///
/// 未设置 `$VISUAL` 或 `$EDITOR` 时直接失败。临时文件位于系统临时目录，
/// 编辑器退出后导入有变化的 TOML，并删除临时文件。
pub async fn platform_profile_open_command(platform_name: &str, json: bool) -> Result<()> {
    let report = open_profile(platform_name, json)?;
    if json {
        let output = ProfileOpenOutput {
            ok: true,
            platform: platform_name,
            profiles_file: report.profiles_db.display().to_string(),
            created: report.created,
            registered: report.registered,
            editor: report.editor,
        };
        println!(
            "{}",
            serde_json::to_string(&output).map_err(CcrError::JsonError)?
        );
    }
    Ok(())
}

// --- 单元测试 ---

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::{OwnedTemp, open_profile, resolve_editor_from};
    use crate::models::{Platform, PlatformPaths};
    use ccr_config::ConfigManager;
    use std::ffi::OsString;
    use std::fs;
    use std::path::{Path, PathBuf};

    struct EnvVar {
        key: &'static str,
        previous: Option<OsString>,
    }

    impl EnvVar {
        fn set(key: &'static str, value: Option<&str>) -> Self {
            let previous = std::env::var_os(key);
            // SAFETY: profile open tests hold TestHome's process-wide env lock.
            unsafe {
                match value {
                    Some(value) => std::env::set_var(key, value),
                    None => std::env::remove_var(key),
                }
            }
            Self { key, previous }
        }
    }

    impl Drop for EnvVar {
        fn drop(&mut self) {
            // SAFETY: restoration runs before TestHome releases the env lock.
            unsafe {
                match self.previous.take() {
                    Some(value) => std::env::set_var(self.key, value),
                    None => std::env::remove_var(self.key),
                }
            }
        }
    }

    fn no_editor() -> (EnvVar, EnvVar) {
        (EnvVar::set("VISUAL", None), EnvVar::set("EDITOR", None))
    }

    fn set_editor(command: &str) -> (EnvVar, EnvVar) {
        (
            EnvVar::set("VISUAL", None),
            EnvVar::set("EDITOR", Some(command)),
        )
    }

    fn quiet_editor() -> String {
        if cfg!(windows) {
            "cmd.exe /C exit /B 0".to_string()
        } else {
            "true".to_string()
        }
    }

    fn copy_editor(source: &Path) -> String {
        if cfg!(windows) {
            format!("cmd.exe /C copy /Y {}", source.display())
        } else {
            format!("cp {}", source.display())
        }
    }

    fn failing_editor() -> String {
        if cfg!(windows) {
            "cmd.exe /C exit /B 1".to_string()
        } else {
            "false".to_string()
        }
    }

    fn our_temps() -> Vec<PathBuf> {
        let prefix = format!("ccr-profile-edit-{}-", std::process::id());
        let Ok(entries) = fs::read_dir(std::env::temp_dir()) else {
            return Vec::new();
        };
        entries
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with(&prefix))
            })
            .collect()
    }

    #[test]
    fn resolve_prefers_visual_over_editor() {
        let target = resolve_editor_from(Some("nano"), Some("vim")).unwrap();
        assert_eq!(target.source, "$VISUAL");
        assert_eq!(target.program, "nano");
    }

    #[test]
    fn resolve_falls_back_to_editor_when_visual_empty() {
        let target = resolve_editor_from(Some("  "), Some("vim")).unwrap();
        assert_eq!(target.source, "$EDITOR");
        assert_eq!(target.program, "vim");
    }

    #[test]
    fn resolve_splits_editor_args() {
        let target = resolve_editor_from(None, Some("code --wait")).unwrap();
        assert_eq!(target.source, "$EDITOR");
        assert_eq!(target.program, "code");
        assert_eq!(target.args, vec!["--wait"]);
    }

    #[test]
    fn resolve_none_when_both_unset_or_blank() {
        assert!(resolve_editor_from(None, None).is_none());
        assert!(resolve_editor_from(Some(""), Some("")).is_none());
        assert!(resolve_editor_from(Some("   "), Some("   ")).is_none());
    }

    #[test]
    fn temp_file_is_outside_the_platform_dir_and_owner_only() {
        let _home = crate::test_support::TestHome::new();
        let paths = PlatformPaths::new(Platform::Claude).unwrap();
        let temp = OwnedTemp::create("auth_token = \"secret-sentinel\"\n").unwrap();
        assert_ne!(temp.path.file_name().unwrap(), "profiles.toml");
        assert!(!temp.path.starts_with(&paths.platform_dir));
        assert_owner_only(&temp.path);
        let path = temp.path.clone();
        drop(temp);
        assert!(!path.exists());
    }

    #[test]
    fn open_without_editor_creates_nothing() {
        let _home = crate::test_support::TestHome::new();
        let _editor = no_editor();
        let error = open_profile("claude", true).unwrap_err();
        assert!(error.to_string().contains("$EDITOR"), "{error}");
        let paths = PlatformPaths::new(Platform::Claude).unwrap();
        assert!(!paths.profiles_file.exists());
        assert!(!paths.profiles_db().exists());
        assert!(our_temps().is_empty());
    }

    #[test]
    fn open_imports_changed_toml_and_deletes_temp() {
        let _home = crate::test_support::TestHome::new();
        let root = PathBuf::from(std::env::var("CCR_ROOT").unwrap());
        let replacement = root.join("replacement.toml");
        fs::write(
            &replacement,
            "[edited]\nmodel = \"changed\"\nauth_token = \"secret-sentinel\"\n",
        )
        .unwrap();
        let _editor = set_editor(&copy_editor(&replacement));

        let report = open_profile("claude", true).unwrap();
        assert!(report.created);
        assert_eq!(report.editor, "$EDITOR");
        assert!(!report.profiles_db.with_file_name("profiles.toml").exists());
        let loaded = ConfigManager::new(&report.profiles_db).load().unwrap();
        assert!(loaded.sections.contains_key("edited"));
        assert!(our_temps().is_empty());
    }

    #[test]
    fn invalid_editor_result_keeps_the_catalog_and_deletes_temp() {
        let _home = crate::test_support::TestHome::new();
        let _quiet = set_editor(&quiet_editor());
        let created = open_profile("grok", true).unwrap();
        let before = ConfigManager::new(&created.profiles_db)
            .canonical_toml()
            .unwrap()
            .unwrap();
        drop(_quiet);

        let root = PathBuf::from(std::env::var("CCR_ROOT").unwrap());
        let replacement = root.join("invalid.toml");
        fs::write(
            &replacement,
            "not [ valid\nauth_token = \"secret-sentinel\"\n",
        )
        .unwrap();
        let _editor = set_editor(&copy_editor(&replacement));
        let error = open_profile("grok", true).unwrap_err();
        assert!(error.to_string().contains("无效"), "{error}");
        assert!(!error.to_string().contains("secret-sentinel"), "{error}");
        let after = ConfigManager::new(&created.profiles_db)
            .canonical_toml()
            .unwrap()
            .unwrap();
        assert_eq!(after, before);
        assert!(our_temps().is_empty());
        assert!(!created.profiles_db.with_file_name("profiles.toml").exists());
    }

    #[test]
    fn editor_failure_does_not_import_and_deletes_temp() {
        let _home = crate::test_support::TestHome::new();
        let _quiet = set_editor(&quiet_editor());
        let created = open_profile("codex", true).unwrap();
        let before = ConfigManager::new(&created.profiles_db)
            .canonical_toml()
            .unwrap()
            .unwrap();
        drop(_quiet);
        let _editor = set_editor(&failing_editor());
        let error = open_profile("codex", true).unwrap_err();
        assert!(error.to_string().contains("退出码"), "{error}");
        let after = ConfigManager::new(&created.profiles_db)
            .canonical_toml()
            .unwrap()
            .unwrap();
        assert_eq!(after.0, before.0);
        assert_eq!(after.1, before.1);
        assert!(our_temps().is_empty());
    }

    fn assert_owner_only(path: &Path) {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(path).unwrap().permissions().mode() & 0o777,
                0o600,
                "{}",
                path.display()
            );
        }
        #[cfg(windows)]
        assert_windows_owner_only_dacl(path);
        #[cfg(not(any(unix, windows)))]
        {
            let _ = path;
            eprintln!("NOT_RUN owner-only permission assertion");
        }
    }

    #[cfg(windows)]
    fn assert_windows_owner_only_dacl(path: &Path) {
        use std::os::windows::ffi::OsStrExt;
        let wide = path
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect::<Vec<_>>();
        #[link(name = "advapi32")]
        unsafe extern "system" {
            fn GetFileSecurityW(
                file_name: *const u16,
                requested_information: u32,
                security_descriptor: *mut std::ffi::c_void,
                length: u32,
                length_needed: *mut u32,
            ) -> i32;
        }
        let mut needed = 0_u32;
        unsafe {
            GetFileSecurityW(
                wide.as_ptr(),
                0x0000_0004,
                std::ptr::null_mut(),
                0,
                &mut needed,
            );
        }
        assert_ne!(needed, 0);
        let mut descriptor = vec![0_u8; needed as usize];
        let ok = unsafe {
            GetFileSecurityW(
                wide.as_ptr(),
                0x0000_0004,
                descriptor.as_mut_ptr().cast(),
                needed,
                &mut needed,
            )
        };
        assert_ne!(ok, 0);
        let control = u16::from_le_bytes(descriptor[2..4].try_into().unwrap());
        assert_ne!(control & 0x1000, 0, "the DACL must be protected");
        let offset = u32::from_le_bytes(descriptor[16..20].try_into().unwrap()) as usize;
        let ace_count = u16::from_le_bytes(descriptor[offset + 4..offset + 6].try_into().unwrap());
        assert_eq!(ace_count, 1, "only the process user receives access");
    }
}
