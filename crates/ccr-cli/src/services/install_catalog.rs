//! Manual install catalog: copy-able commands and documentation links.
//!
//! Provides platform-specific install instructions for users who prefer
//! manual control over the installation process.

use std::collections::BTreeSet;

use crate::services::install_types::{
    InstallFlowError, ManualCatalog, ManualCommand, PackageManager, Platform,
};

/// Official documentation URL for llmusage.
const DOCS_URL: &str = "https://github.com/bahayonghang/llmuasage";

/// Build the manual install catalog covering all supported platforms.
///
/// Returns an error if any platform cannot be covered (should not happen
/// with the current static catalog).
pub fn build_catalog() -> Result<ManualCatalog, InstallFlowError> {
    let entries = vec![
        // ── macOS ────────────────────────────────────────────────────────
        ManualCommand {
            platform: Platform::Macos,
            package_manager: Some(PackageManager::Homebrew),
            title: "使用 Homebrew 安装 (推荐)".to_string(),
            command_line: "brew install llmusage".to_string(),
            notes: Some("需要先安装 Homebrew: https://brew.sh".to_string()),
        },
        ManualCommand {
            platform: Platform::Macos,
            package_manager: Some(PackageManager::Cargo),
            title: "使用 Cargo 从源码编译".to_string(),
            command_line: "cargo install --locked llmusage".to_string(),
            notes: Some("需要 Rust 工具链: https://rustup.rs".to_string()),
        },
        // ── Linux ────────────────────────────────────────────────────────
        ManualCommand {
            platform: Platform::Linux,
            package_manager: Some(PackageManager::Cargo),
            title: "使用 Cargo 从源码编译".to_string(),
            command_line: "cargo install --locked llmusage".to_string(),
            notes: Some("需要 Rust 工具链: https://rustup.rs".to_string()),
        },
        // ── Windows ──────────────────────────────────────────────────────
        ManualCommand {
            platform: Platform::Windows,
            package_manager: Some(PackageManager::Winget),
            title: "使用 winget 安装 (推荐)".to_string(),
            command_line: "winget install --id llmusage --source winget".to_string(),
            notes: Some("Windows 10 1709+ 自带 winget".to_string()),
        },
        ManualCommand {
            platform: Platform::Windows,
            package_manager: Some(PackageManager::Scoop),
            title: "使用 Scoop 安装".to_string(),
            command_line: "scoop install llmusage".to_string(),
            notes: Some("需要先安装 Scoop: https://scoop.sh".to_string()),
        },
        ManualCommand {
            platform: Platform::Windows,
            package_manager: Some(PackageManager::Cargo),
            title: "使用 Cargo 从源码编译".to_string(),
            command_line: "cargo install --locked llmusage".to_string(),
            notes: Some("需要 Rust 工具链: https://rustup.rs".to_string()),
        },
    ];

    let catalog = ManualCatalog {
        entries,
        docs_url: DOCS_URL.to_string(),
    };

    // Verify totality: all platforms must be covered.
    let covered = covered_platforms(&catalog);
    let required: BTreeSet<Platform> = [Platform::Macos, Platform::Linux, Platform::Windows].into();

    for platform in &required {
        if !covered.contains(platform) {
            return Err(InstallFlowError::ManualCatalogUnavailable {
                missing_platform: *platform,
            });
        }
    }

    Ok(catalog)
}

/// Returns the set of platforms covered by the catalog entries.
fn covered_platforms(catalog: &ManualCatalog) -> BTreeSet<Platform> {
    catalog.entries.iter().map(|e| e.platform).collect()
}

/// Get manual commands filtered by a specific platform.
pub fn commands_for_platform(catalog: &ManualCatalog, platform: Platform) -> Vec<&ManualCommand> {
    catalog
        .entries
        .iter()
        .filter(|e| e.platform == platform)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_covers_all_platforms() {
        let catalog = build_catalog().expect("catalog should build successfully");
        let covered = covered_platforms(&catalog);
        assert!(covered.contains(&Platform::Macos));
        assert!(covered.contains(&Platform::Linux));
        assert!(covered.contains(&Platform::Windows));
    }

    #[test]
    fn catalog_has_docs_url() {
        let catalog = build_catalog().expect("catalog should build");
        assert!(!catalog.docs_url.is_empty());
        assert!(catalog.docs_url.starts_with("https://"));
    }

    #[test]
    fn macos_has_at_least_two_options() {
        let catalog = build_catalog().expect("catalog should build");
        let macos_cmds = commands_for_platform(&catalog, Platform::Macos);
        assert!(macos_cmds.len() >= 2);
    }

    #[test]
    fn windows_has_at_least_three_options() {
        let catalog = build_catalog().expect("catalog should build");
        let win_cmds = commands_for_platform(&catalog, Platform::Windows);
        assert!(win_cmds.len() >= 3);
    }

    #[test]
    fn all_commands_are_non_empty() {
        let catalog = build_catalog().expect("catalog should build");
        for entry in &catalog.entries {
            assert!(!entry.command_line.is_empty());
            assert!(!entry.title.is_empty());
        }
    }
}
