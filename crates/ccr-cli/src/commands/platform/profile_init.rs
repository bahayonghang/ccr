use crate::models::{Platform, PlatformPaths};
use ccr_config::{
    ConfigManager, migrate_catalogs, parse_profiles_from_str, register_platform_if_missing,
};
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::logging::ColorOutput;
use serde::Serialize;
use std::path::PathBuf;
use std::str::FromStr;
use std::thread;
use std::time::Duration;

#[derive(Serialize)]
struct ProfileInitOutput<'a> {
    ok: bool,
    platform: &'a str,
    profiles_file: String,
    created: bool,
    registered: bool,
}

/// ensure_profiles_file 的结果
pub(super) struct ProfileFileEnsured {
    pub path: PathBuf,
    pub created: bool,
    pub registered: bool,
}

/// 确保平台目录库存在（不存在时从模板写入），并注册平台。
///
/// 不创建活的 `profiles.toml`。模板仍是交换用 TOML。
pub(super) fn ensure_profiles_file(
    platform: Platform,
    platform_name: &str,
    template: &str,
) -> Result<ProfileFileEnsured> {
    let paths = PlatformPaths::new(platform)?;
    paths.ensure_directories()?;

    parse_profiles_from_str(template).map_err(|error| {
        CcrError::ConfigError(format!("内置 {platform_name} profiles 模板无效: {error}"))
    })?;

    let manager = ConfigManager::new(&paths.profiles_file);
    // 并发初始化时，先创建的一方会留下尚未写入 document 的库文件。
    // 空 token 不能覆盖已有 revision，等到目录可读后再读一次。
    let created = ensure_catalog(&paths, &manager, template)?;
    let registered = register_platform_if_missing(platform_name, platform.display_name())?;

    Ok(ProfileFileEnsured {
        path: paths.profiles_db(),
        created,
        registered,
    })
}

const ENSURE_ATTEMPTS: u32 = 40;
const ENSURE_PAUSE: Duration = Duration::from_millis(25);

fn ensure_catalog(paths: &PlatformPaths, manager: &ConfigManager, template: &str) -> Result<bool> {
    let mut last_error = None;
    for attempt in 0..ENSURE_ATTEMPTS {
        match ensure_catalog_once(paths, manager, template) {
            Ok(created) => return Ok(created),
            Err(error) if catalog_init_pending(&error) && attempt + 1 < ENSURE_ATTEMPTS => {
                last_error = Some(error);
                thread::sleep(ENSURE_PAUSE);
            }
            Err(error) => return Err(error),
        }
    }
    Err(last_error.unwrap_or_else(|| CcrError::ConfigError("配置目录尚未完成迁移".into())))
}

fn ensure_catalog_once(
    paths: &PlatformPaths,
    manager: &ConfigManager,
    template: &str,
) -> Result<bool> {
    if catalog_exists(paths)? {
        return Ok(false);
    }
    match manager.replace_toml(template, "") {
        Ok(_) => Ok(true),
        Err(error) => {
            let conflict = matches!(
                &error,
                CcrError::ValidationError(message) if message.contains("配置版本冲突")
            );
            if (conflict || catalog_init_pending(&error)) && catalog_exists_now(paths)? {
                Ok(false)
            } else {
                Err(error)
            }
        }
    }
}

fn catalog_init_pending(error: &CcrError) -> bool {
    let message = error.to_string();
    message.contains("配置目录尚未完成迁移")
        || message.contains("读取配置目录失败")
        || message.contains("创建文件失败")
}

fn catalog_exists_now(paths: &PlatformPaths) -> Result<bool> {
    match catalog_exists(paths) {
        Ok(exists) => Ok(exists),
        Err(error) if catalog_init_pending(&error) => Ok(false),
        Err(error) => Err(error),
    }
}

fn catalog_exists(paths: &PlatformPaths) -> Result<bool> {
    let manager = ConfigManager::new(&paths.profiles_file);
    match manager.canonical_toml() {
        Ok(catalog) => Ok(catalog.is_some()),
        Err(error) if error.to_string().contains("配置尚未迁移") => {
            migrate_catalogs()?;
            Ok(manager.canonical_toml()?.is_some())
        }
        Err(error) => Err(error),
    }
}

/// Initializes one platform's profile directory and copy-ready template.
pub async fn platform_profile_init_command(
    platform_name: &str,
    template: &str,
    json: bool,
) -> Result<()> {
    let platform = Platform::from_str(platform_name)
        .map_err(|_| CcrError::PlatformNotFound(platform_name.to_string()))?;
    if !Platform::auth_profile_supported().contains(&platform) {
        return Err(CcrError::PlatformNotSupported(platform_name.to_string()));
    }

    let ensured = ensure_profiles_file(platform, platform_name, template)?;
    let profiles_file = ensured.path.display().to_string();

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&ProfileInitOutput {
                ok: true,
                platform: platform_name,
                profiles_file,
                created: ensured.created,
                registered: ensured.registered,
            })?
        );
        return Ok(());
    }

    ColorOutput::title(&format!("初始化 {} Profiles", platform.display_name()));
    println!();
    if ensured.created {
        ColorOutput::success(&format!("已创建 profiles 目录: {profiles_file}"));
    } else {
        ColorOutput::info(&format!("profiles 目录已存在，保持不变: {profiles_file}"));
    }
    if ensured.registered {
        ColorOutput::success(&format!("已注册平台: {platform_name}"));
    } else {
        ColorOutput::info(&format!("平台已注册: {platform_name}"));
    }

    crate::commands::common::print_next_steps(&[(
        "查看 profiles",
        &format!("ccr {platform_name} profile list"),
    )]);
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::platforms::{ClaudePlatform, CodexPlatform, GrokPlatform};
    use ccr_config::{CcsConfig, PlatformConfig};

    const CLAUDE_TEMPLATE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/claude/profiles.example.toml"
    ));
    const CODEX_TEMPLATE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/codex/profiles.toml"
    ));
    const GROK_TEMPLATE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/grok/profiles.toml"
    ));
    const GROK_DOCS_TEMPLATE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/examples/grok-profiles.toml"
    ));

    fn assert_template_valid(template: &str, platform: &dyn PlatformConfig) {
        let config: CcsConfig = toml::from_str(template).unwrap();
        assert!(config.current_config.is_empty());

        let profiles = parse_profiles_from_str(template).unwrap();
        assert!(!profiles.is_empty());
        for profile in profiles.values() {
            platform.validate_profile(profile).unwrap();
        }
    }

    #[test]
    fn embedded_profile_templates_are_inactive_and_valid() {
        assert_template_valid(CLAUDE_TEMPLATE, &ClaudePlatform::new().unwrap());
        assert_template_valid(CODEX_TEMPLATE, &CodexPlatform::new().unwrap());
        assert_template_valid(GROK_TEMPLATE, &GrokPlatform::new().unwrap());
    }

    #[test]
    fn grok_example_and_docs_mirror_are_identical() {
        assert_eq!(GROK_TEMPLATE.as_bytes(), GROK_DOCS_TEMPLATE.as_bytes());
    }

    #[test]
    fn ensure_profiles_file_writes_the_catalog_and_not_live_toml() {
        let _home = crate::test_support::TestHome::new();
        let paths = PlatformPaths::new(Platform::Claude).unwrap();
        let first = ensure_profiles_file(Platform::Claude, "claude", CLAUDE_TEMPLATE).unwrap();
        assert!(first.created);
        assert!(first.path.ends_with("profiles.db"));
        assert!(paths.profiles_db().is_file());
        assert!(!paths.profiles_file.exists());
        let loaded = ConfigManager::new(&paths.profiles_file).load().unwrap();
        assert!(!loaded.sections.is_empty());

        let second = ensure_profiles_file(Platform::Claude, "claude", CLAUDE_TEMPLATE).unwrap();
        assert!(!second.created);
        assert!(!paths.profiles_file.exists());
    }

    #[test]
    fn concurrent_ensure_profiles_file_keeps_one_catalog() {
        let _home = crate::test_support::TestHome::new();
        let threads: Vec<_> = (0..8)
            .map(|_| {
                std::thread::spawn(|| {
                    ensure_profiles_file(Platform::Grok, "grok", GROK_TEMPLATE).unwrap()
                })
            })
            .collect();
        let results: Vec<_> = threads
            .into_iter()
            .map(|thread| thread.join().unwrap())
            .collect();
        assert_eq!(results.iter().filter(|item| item.created).count(), 1);
        let paths = PlatformPaths::new(Platform::Grok).unwrap();
        assert!(paths.profiles_db().is_file());
        assert!(!paths.profiles_file.exists());
    }
}
