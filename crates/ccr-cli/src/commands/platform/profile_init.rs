use crate::models::{Platform, PlatformPaths};
use ccr_config::{parse_profiles_from_str, register_platform_if_missing};
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::logging::ColorOutput;
use ccr_core::core::{BackupPolicy, VersionedWriteOutcome, WriteOptions, write_guarded_versioned};
use serde::Serialize;
use std::path::PathBuf;
use std::str::FromStr;

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

/// 确保 profiles.toml 存在（不存在时创建），并注册平台
///
/// 纯逻辑函数，不输出到终端。调用方负责根据返回值决定输出内容。
pub(super) fn ensure_profiles_file(
    platform: Platform,
    platform_name: &str,
    template: &str,
) -> Result<ProfileFileEnsured> {
    let paths = PlatformPaths::new(platform)?;
    paths.ensure_directories()?;

    // 模板来自仓库 examples；运行时仍先解析，避免写出损坏的内嵌配置。
    parse_profiles_from_str(template).map_err(|error| {
        CcrError::ConfigError(format!("内置 {platform_name} profiles 模板无效: {error}"))
    })?;

    let created = if paths.profiles_file.exists() {
        false
    } else {
        matches!(
            write_guarded_versioned(
                &paths.profiles_file,
                template.as_bytes(),
                "",
                &WriteOptions {
                    secret: true,
                    backup: BackupPolicy::None,
                    ..Default::default()
                },
            )?,
            VersionedWriteOutcome::Written
        )
    };
    let registered = register_platform_if_missing(platform_name, platform.display_name())?;

    Ok(ProfileFileEnsured {
        path: paths.profiles_file,
        created,
        registered,
    })
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
        ColorOutput::success(&format!("已创建 profiles 模板: {profiles_file}"));
    } else {
        ColorOutput::info(&format!("profiles 文件已存在，保持不变: {profiles_file}"));
    }
    if ensured.registered {
        ColorOutput::success(&format!("已注册平台: {platform_name}"));
    } else {
        ColorOutput::info(&format!("平台已注册: {platform_name}"));
    }

    println!();
    ColorOutput::info("下一步:");
    println!("  1. 编辑模板: {profiles_file}");
    println!("  2. 或创建 profile: ccr {platform_name} profile create --help");
    println!("  3. 查看 profiles: ccr {platform_name} profile list");
    println!("  4. 激活 profile: ccr {platform_name} profile switch <name>");
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
}
