//! Explicit, opt-in benchmark storage. Unreviewed capabilities fail closed.
//! This is not a performance result or a substitute for native storage checks.
use ccr_cli::application::{
    platform_roots::PlatformRoots, profile_lifecycle::ProfileLifecycleContext,
};
use ccr_config::{Platform, PlatformPaths};
use serde::Deserialize;
use std::io::Read;
use std::{
    ffi::OsString,
    path::{Component, Path, PathBuf},
    sync::{Arc, OnceLock},
};

const MARKER: &str = "ccr-tauri-fixture.json";
static CURRENT: OnceLock<Arc<FixtureLayout>> = OnceLock::new();

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Marker {
    version: u32,
    fixture_id: String,
}

#[derive(Clone)]
pub struct FixtureLayout {
    pub root: PathBuf,
    pub id: String,
    pub roots: PlatformRoots,
}

pub fn current() -> Option<&'static Arc<FixtureLayout>> {
    CURRENT.get()
}

/// Called before logger, migrations, Tauri context and WebView construction.
pub fn parse(
    args: impl IntoIterator<Item = OsString>,
) -> Result<Option<Arc<FixtureLayout>>, String> {
    let mut args = args.into_iter();
    let mut root = None;
    while let Some(arg) = args.next() {
        if arg == "--fixture-root" {
            if root.is_some() {
                return Err("fixture_duplicate_root".into());
            }
            root = Some(PathBuf::from(args.next().ok_or("fixture_root_required")?));
        } else if arg.to_string_lossy().starts_with("--fixture-root=") {
            return Err("fixture_use_separate_root_argument".into());
        }
    }
    root.map(|root| FixtureLayout::load(&root).map(Arc::new))
        .transpose()
}

pub fn install(layout: Arc<FixtureLayout>) -> Result<(), String> {
    CURRENT
        .set(layout)
        .map_err(|_| "fixture_identity_already_installed".into())
}

fn unlinked(path: &Path) -> Result<(), String> {
    for ancestor in path.ancestors() {
        match std::fs::symlink_metadata(ancestor) {
            Ok(meta) => {
                if meta.file_type().is_symlink() {
                    return Err("fixture_linked_path".into());
                }
                #[cfg(windows)]
                {
                    use std::os::windows::fs::MetadataExt;
                    if meta.file_attributes() & 0x400 != 0 {
                        return Err("fixture_linked_path".into());
                    }
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err("fixture_path_unavailable".into()),
        }
    }
    Ok(())
}

impl FixtureLayout {
    pub fn load(root: &Path) -> Result<Self, String> {
        if cfg!(target_os = "macos") {
            return Err("fixture_webview_storage_unsupported_on_macos".into());
        }
        if !root.is_absolute() || root.components().any(|c| matches!(c, Component::ParentDir)) {
            return Err("fixture_absolute_root_required".into());
        }
        unlinked(root)?;
        let root = root
            .canonicalize()
            .map_err(|_| "fixture_root_unavailable")?;
        if !root.is_dir() {
            return Err("fixture_root_not_directory".into());
        }
        let marker = root.join(MARKER);
        unlinked(&marker)?;
        let meta = std::fs::metadata(&marker).map_err(|_| "fixture_marker_required")?;
        if !meta.is_file() || meta.len() > 4096 {
            return Err("fixture_marker_invalid".into());
        }
        let mut marker_bytes = Vec::new();
        std::fs::File::open(marker)
            .map_err(|_| "fixture_marker_unreadable")?
            .take(4097)
            .read_to_end(&mut marker_bytes)
            .map_err(|_| "fixture_marker_unreadable")?;
        if marker_bytes.len() > 4096 {
            return Err("fixture_marker_invalid".into());
        }
        let marker: Marker =
            serde_json::from_slice(&marker_bytes).map_err(|_| "fixture_marker_invalid")?;
        if marker.version != 1
            || marker.fixture_id.is_empty()
            || marker.fixture_id.len() > 64
            || !marker
                .fixture_id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
        {
            return Err("fixture_marker_invalid".into());
        }
        // Every existing descendant may be consumed by a domain scanner or
        // migration. Reject links before any of those owners are constructed.
        let mut pending = vec![root.clone()];
        while let Some(directory) = pending.pop() {
            for entry in std::fs::read_dir(directory).map_err(|_| "fixture_tree_unreadable")? {
                let entry = entry.map_err(|_| "fixture_tree_unreadable")?;
                unlinked(&entry.path())?;
                if entry
                    .file_type()
                    .map_err(|_| "fixture_tree_unreadable")?
                    .is_dir()
                {
                    pending.push(entry.path());
                }
            }
        }
        let roots = PlatformRoots::fixture(&root).map_err(|_| "fixture_roots_invalid")?;
        Ok(Self {
            root,
            id: marker.fixture_id,
            roots,
        })
    }

    pub fn path(&self, relative: &str) -> Result<PathBuf, String> {
        let relative = Path::new(relative);
        if relative
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
        {
            return Err("fixture_relative_path_required".into());
        }
        let path = self.root.join(relative);
        unlinked(&path)?;
        let existing = path
            .ancestors()
            .find(|p| p.exists())
            .ok_or("fixture_parent_missing")?;
        if !existing
            .canonicalize()
            .map_err(|_| "fixture_parent_unavailable")?
            .starts_with(&self.root)
        {
            return Err("fixture_path_escape".into());
        }
        Ok(path)
    }

    pub fn prepare(&self) -> Result<(), String> {
        for role in [
            "home",
            "ccr",
            "ui",
            "ccr/analytics",
            "llmusage",
            "logs",
            "preferences",
            "locks",
            "backups",
            "crypto",
            "project",
            "webview",
        ] {
            std::fs::create_dir_all(self.path(role)?)
                .map_err(|_| "fixture_directory_unavailable")?;
        }
        let paths = Platform::implemented()
            .into_iter()
            .map(|platform| self.platform_paths(platform))
            .collect::<Result<Vec<_>, _>>()?;
        ccr_core::core::LockManager::with_scoped_directory(&self.roots.locks, || {
            ccr_config::migrate_catalogs_at(&paths)
        })
        .map_err(|_| "fixture_catalog_migration_failed".to_string())
    }

    pub fn platform_paths(&self, platform: Platform) -> Result<PlatformPaths, String> {
        let root = self.path("ccr")?;
        let platform_dir = root.join("platforms").join(platform.short_name());
        unlinked(&platform_dir)?;
        let paths = PlatformPaths {
            registry_file: root.join("config.toml"),
            profiles_file: platform_dir.join("profiles.toml"),
            settings_file: platform_dir.join("settings.json"),
            history_file: root
                .join("history")
                .join(format!("{}.json", platform.short_name())),
            backups_dir: root.join("backups").join(platform.short_name()),
            platform_dir,
            root,
        };
        for path in [
            &paths.registry_file,
            &paths.profiles_file,
            &paths.profiles_db(),
            &paths.backups_dir,
        ] {
            unlinked(path)?;
        }
        Ok(paths)
    }

    pub fn profile_context(&self, platform: Platform) -> ccr_core::Result<ProfileLifecycleContext> {
        let paths = self
            .platform_paths(platform)
            .map_err(ccr_core::CcrError::ConfigError)?;
        let history = self.roots.ccr.join("history.db");
        let context = match platform {
            Platform::Claude => ProfileLifecycleContext::for_claude(
                paths,
                self.roots.claude.clone(),
                self.roots.locks.clone(),
                history,
            ),
            Platform::Codex => ProfileLifecycleContext::for_codex(
                paths,
                self.roots.codex_runtime.clone(),
                self.roots.locks.clone(),
                history,
            ),
            Platform::Grok => ProfileLifecycleContext::for_grok(
                paths,
                self.roots.grok_runtime.join("config.toml"),
                self.roots.locks.clone(),
                history,
            ),
            _ => Err(ccr_core::CcrError::PlatformNotSupported(
                "fixture_profile_platform".into(),
            )),
        }?;
        Ok(context.with_process_environment(false))
    }
}

pub fn profile_context(platform: Platform) -> ccr_core::Result<Option<ProfileLifecycleContext>> {
    current()
        .map(|layout| layout.profile_context(platform))
        .transpose()
}

pub fn with_locks<T>(work: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    match current() {
        Some(layout) => {
            ccr_core::core::LockManager::with_scoped_directory(&layout.roots.locks, || Ok(work()))
                .map_err(|_| "fixture_lock_scope_unavailable".to_string())?
        }
        None => work(),
    }
}

pub fn claude_platform() -> ccr_core::Result<ccr_cli::platforms::ClaudePlatform> {
    match current() {
        Some(layout) => Ok(ccr_cli::platforms::ClaudePlatform::from_parts(
            layout
                .platform_paths(Platform::Claude)
                .map_err(ccr_core::CcrError::ConfigError)?,
            ccr::SettingsManager::new(
                &layout.roots.claude.settings_file,
                &layout.roots.claude.backups_dir,
                ccr_core::core::LockManager::new(&layout.roots.locks),
            ),
            layout.roots.locks.clone(),
        )),
        None => ccr_cli::platforms::ClaudePlatform::new(),
    }
}

pub fn platform_paths(platform: Platform) -> ccr_core::Result<PlatformPaths> {
    match current() {
        Some(layout) => layout
            .platform_paths(platform)
            .map_err(ccr_core::CcrError::ConfigError),
        None => PlatformPaths::new(platform),
    }
}

pub fn admit(command: &str) -> Result<(), String> {
    if let Some(layout) = current() {
        if !captured_command(command) {
            return Err(format!("fixture_capability_unavailable:{command}"));
        }
        // Recheck fixed owners at admission; a replaced fixture leaf must not
        // redirect a later pooled connection or a captured file read.
        for path in [
            "ui/ccr-ui.db",
            "ccr/analytics/usage.db",
            "llmusage/llmusage.db",
            "preferences/desktop-shell.json",
            "locks",
            "webview",
            "home/.claude",
            "home/.claude/settings.json",
            "home/.claude/.credentials.json",
            "home/.claude.json",
            "home/.claude/backups",
            "home/.codex",
            "home/.codex/config.toml",
            "home/.codex/auth.json",
            "project",
        ] {
            let _ = layout.path(path)?;
        }
    }
    Ok(())
}

// Explicitly reviewed call graphs only. No risk/verb-based automatic permission.
const CAPTURED_COMMANDS: &[&str] = &[
    "claude_list_auth_accounts",
    "claude_get_auth_current",
    "get_home_insights",
    "claude_list_mcp_servers",
    "claude_add_mcp_server",
    "claude_update_mcp_server",
    "claude_delete_mcp_server",
    "claude_list_agents",
    "claude_add_agent",
    "claude_update_agent",
    "claude_delete_agent",
    "claude_list_plugins",
    "claude_add_plugin",
    "claude_update_plugin",
    "claude_delete_plugin",
    "claude_list_slash_commands",
    "claude_add_slash_command",
    "claude_update_slash_command",
    "claude_delete_slash_command",
    "list_configs",
    "add_config",
    "update_config",
    "delete_config",
    "rename_config",
    "duplicate_config",
    "switch_config",
    "claude_list_profiles",
    "claude_get_profile",
    "claude_add_profile",
    "claude_update_profile",
    "claude_delete_profile",
    "claude_apply_profile",
    "claude_profile_off",
    "claude_get_profiles_raw",
    "claude_save_profiles_raw",
    "claude_export_profiles",
    "list_environments",
    "get_current_environment",
    "switch_environment",
    "refresh_environments",
    "agent_sessions_list",
    "agent_sessions_get_detail",
    "agent_sessions_get_provider_status",
    "agent_sessions_start_refresh",
    "agent_sessions_get_refresh_status",
    "get_usage_summary_v2",
    "get_usage_trends_v2",
    "get_usage_by_model_v2",
    "get_usage_by_provider_v2",
    "get_usage_by_project_v2",
    "get_usage_heatmap_v2",
    "get_usage_logs_v2",
    "get_usage_dashboard_v2",
    "get_home_usage_overview_v2",
    "get_recent_events",
    "get_monitoring_feed",
    "append_frontend_logs",
    "get_runtime_metrics",
    "check_version",
    "health_check",
    "get_favorites",
    "add_favorite",
    "remove_favorite",
    "get_recent_items",
    "clear_recent_items",
    "add_recent_item",
    "shell_get_preferences",
    "shell_set_preferences",
    "shell_show_main_window",
    "shell_request_quit",
    "codex_get_tray_snapshot",
    "get_usage_capabilities_v2",
    "claude_get_settings",
    "claude_update_settings",
];

fn captured_command(command: &str) -> bool {
    CAPTURED_COMMANDS.contains(&command)
}

pub fn webview_builder<'a, R: tauri::Runtime, M: tauri::Manager<R>>(
    builder: tauri::WebviewWindowBuilder<'a, R, M>,
) -> Result<tauri::WebviewWindowBuilder<'a, R, M>, String> {
    match current() {
        Some(layout) => Ok(builder.data_directory(layout.path("webview")?).title(format!("CCR · Fixture {}",layout.id))),
        None => Ok(builder),
    }
}

#[cfg(all(test, not(target_os = "macos")))]
mod tests;
