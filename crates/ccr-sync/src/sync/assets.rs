//! Fixed sync assets and transfer orchestration shared by desktop clients.
//! Sensitive content always uses the existing v2 envelope and pull transaction.
use super::{SyncConfig, SyncService, WebDavConfig};
use ccr_core::Secret;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncAssetKind {
    Directory,
    File,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncAssetGroup {
    Ccr,
    Claude,
    Codex,
}

impl SyncAssetGroup {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ccr => "ccr",
            Self::Claude => "claude",
            Self::Codex => "codex",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SyncAssetDefinition {
    pub id: &'static str,
    pub group: SyncAssetGroup,
    pub name: &'static str,
    pub description: &'static str,
    pub kind: SyncAssetKind,
    pub sensitive: bool,
    pub local_path: &'static str,
    pub remote_segment: &'static str,
    pub canonical_name: Option<&'static str>,
}

pub const SYNC_ASSETS: &[SyncAssetDefinition] = &[
    SyncAssetDefinition {
        id: "ccr-platforms",
        group: SyncAssetGroup::Ccr,
        name: "CCR Platforms",
        description: "All CCR platform configuration, including auth, URL, key, and provider routing fields.",
        kind: SyncAssetKind::Directory,
        sensitive: true,
        local_path: "~/.ccr/platforms/",
        remote_segment: "platforms/",
        canonical_name: None,
    },
    SyncAssetDefinition {
        id: "claude-settings",
        group: SyncAssetGroup::Claude,
        name: "settings.json",
        description: "Claude Code user settings only; the rest of ~/.claude is intentionally excluded.",
        kind: SyncAssetKind::File,
        sensitive: true,
        local_path: "~/.claude/settings.json",
        remote_segment: "claude/settings.json",
        canonical_name: Some("settings.json"),
    },
    SyncAssetDefinition {
        id: "claude-memory",
        group: SyncAssetGroup::Claude,
        name: "CLAUDE.md",
        description: "Claude Code global memory and instructions file.",
        kind: SyncAssetKind::File,
        sensitive: false,
        local_path: "~/.claude/CLAUDE.md",
        remote_segment: "claude/CLAUDE.md",
        canonical_name: Some("CLAUDE.md"),
    },
    SyncAssetDefinition {
        id: "codex-config",
        group: SyncAssetGroup::Codex,
        name: "config.toml",
        description: "Codex user configuration only; the rest of ~/.codex is intentionally excluded.",
        kind: SyncAssetKind::File,
        sensitive: true,
        local_path: "~/.codex/config.toml",
        remote_segment: "codex/config.toml",
        canonical_name: Some("config.toml"),
    },
    SyncAssetDefinition {
        id: "codex-agents",
        group: SyncAssetGroup::Codex,
        name: "AGENTS.md",
        description: "Codex global instructions file.",
        kind: SyncAssetKind::File,
        sensitive: false,
        local_path: "~/.codex/AGENTS.md",
        remote_segment: "codex/AGENTS.md",
        canonical_name: Some("AGENTS.md"),
    },
];

#[derive(Clone)]
pub struct AssetRoots {
    pub ccr: PathBuf,
    pub claude: PathBuf,
    pub codex: PathBuf,
}
impl AssetRoots {
    pub fn from_home(home: &Path, ccr: PathBuf) -> Self {
        Self {
            ccr,
            claude: home.join(".claude"),
            codex: home.join(".codex"),
        }
    }
    pub fn resolve(
        &self,
        asset: &SyncAssetDefinition,
    ) -> Result<(PathBuf, Option<String>), String> {
        let path = match asset.id {
            "ccr-platforms" => self.ccr.join("platforms"),
            "claude-settings" => self.claude.join("settings.json"),
            "claude-memory" => self.claude.join("CLAUDE.md"),
            "codex-config" => self.codex.join("config.toml"),
            "codex-agents" => self.codex.join("AGENTS.md"),
            _ => return Err("Unknown sync asset".into()),
        };
        reject_links(&path)?;
        if path.exists() || asset.kind == SyncAssetKind::Directory {
            return Ok((path, asset.canonical_name.map(str::to_string)));
        }
        if let (Some(parent), Some(name)) = (path.parent(), asset.canonical_name)
            && let Some(found) = find_case_insensitive_child(parent, name)
        {
            reject_links(&found)?;
            return Ok((found, Some(name.into())));
        }
        Ok((path, asset.canonical_name.map(str::to_string)))
    }
}
pub fn find_case_insensitive_child(parent: &Path, name: &str) -> Option<PathBuf> {
    std::fs::read_dir(parent)
        .ok()?
        .flatten()
        .find(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .eq_ignore_ascii_case(name)
        })
        .map(|entry| entry.path())
}
fn reject_links(path: &Path) -> Result<(), String> {
    if !path.is_absolute() {
        return Err("Asset root must be absolute".into());
    }
    for ancestor in path.ancestors() {
        match std::fs::symlink_metadata(ancestor) {
            Ok(meta) => {
                if meta.file_type().is_symlink() {
                    return Err("Linked asset paths are not supported".into());
                }
                #[cfg(windows)]
                {
                    use std::os::windows::fs::MetadataExt;
                    if meta.file_attributes() & 0x400 != 0 {
                        return Err("Linked asset paths are not supported".into());
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err("Cannot inspect asset path".into()),
        }
    }
    Ok(())
}
pub fn asset_by_id(id: &str) -> Result<&'static SyncAssetDefinition, String> {
    SYNC_ASSETS
        .iter()
        .find(|a| a.id == id)
        .ok_or_else(|| format!("Unknown sync asset: {id}"))
}
pub fn normalize_remote_base(base: &str) -> String {
    let trimmed = base.trim();
    if trimmed.is_empty() {
        return "/ccr".into();
    }
    let value = if trimmed.starts_with('/') {
        trimmed.to_string()
    } else {
        format!("/{trimmed}")
    };
    value.trim_end_matches('/').into()
}
pub fn asset_remote_path(asset: &SyncAssetDefinition, webdav: &WebDavConfig) -> String {
    let base = normalize_remote_base(&webdav.base_remote_path);
    let segment = asset.remote_segment.trim_start_matches('/');
    let joined = format!("{}/{}", base, segment).replace('\\', "/");
    if asset.kind == SyncAssetKind::Directory && !joined.ends_with('/') {
        format!("{joined}/")
    } else {
        joined
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncAssetAction {
    PushCreate,
    PushOverwrite,
    Pull,
    Conflict,
    Missing,
}

pub fn decide_sync_asset_action(
    local_exists: bool,
    remote_exists: bool,
    force: bool,
) -> SyncAssetAction {
    match (local_exists, remote_exists, force) {
        (true, false, _) => SyncAssetAction::PushCreate,
        (false, true, _) => SyncAssetAction::Pull,
        (true, true, true) => SyncAssetAction::PushOverwrite,
        (true, true, false) => SyncAssetAction::Conflict,
        (false, false, _) => SyncAssetAction::Missing,
    }
}

pub fn require_asset_passphrase<'a>(
    asset: &SyncAssetDefinition,
    passphrase: Option<&'a ccr_core::Secret>,
) -> Result<&'a str, String> {
    let passphrase = passphrase
        .map(ccr_core::Secret::expose)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            format!(
                "sync_envelope_passphrase_required: Sensitive asset {} requires an operation passphrase.",
                asset.id
            )
        })?;
    Ok(passphrase)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetDirection {
    Push,
    Pull,
    Sync,
}
#[derive(Clone)]
pub struct AssetInfo {
    pub id: &'static str,
    pub local_path: PathBuf,
    pub remote_path: String,
    pub local_exists: bool,
    pub remote_exists: Option<bool>,
    pub canonical_name: Option<String>,
}
pub fn asset_remote_exists_state<E>(result: Result<bool, E>) -> Option<bool> {
    result.ok()
}

pub struct AssetService {
    webdav: WebDavConfig,
    roots: AssetRoots,
    #[cfg(feature = "test-support")]
    loopback: bool,
}
impl AssetService {
    pub fn new(webdav: WebDavConfig, roots: AssetRoots) -> Self {
        Self {
            webdav,
            roots,
            #[cfg(feature = "test-support")]
            loopback: false,
        }
    }
    #[cfg(feature = "test-support")]
    pub fn for_test_loopback(webdav: WebDavConfig, roots: AssetRoots) -> Self {
        Self {
            webdav,
            roots,
            loopback: true,
        }
    }
    async fn service(&self, asset: &SyncAssetDefinition) -> Result<SyncService, String> {
        if !self.webdav.enabled
            || self.webdav.url.is_empty()
            || self.webdav.username.is_empty()
            || self.webdav.password.is_empty()
        {
            return Err("WebDAV account is incomplete. Please configure WebDAV first.".into());
        }
        let config = SyncConfig {
            enabled: self.webdav.enabled,
            webdav_url: self.webdav.url.clone(),
            username: self.webdav.username.clone(),
            password: self.webdav.password.clone(),
            remote_path: asset_remote_path(asset, &self.webdav),
            auto_sync: false,
        };
        #[cfg(feature = "test-support")]
        if self.loopback {
            return SyncService::new_for_test_loopback(&config)
                .await
                .map_err(|e| e.to_string());
        }
        SyncService::new(&config).await.map_err(|e| e.to_string())
    }
    /// Local inventory does not probe the network. Callers explicitly request probing.
    pub async fn info(&self, id: &str, probe: bool) -> Result<AssetInfo, String> {
        let asset = asset_by_id(id)?;
        let (path, canonical_name) = self.roots.resolve(asset)?;
        let local_exists = tokio::fs::try_exists(&path)
            .await
            .map_err(|_| "Cannot inspect local asset")?;
        let remote_exists = if probe {
            match self.service(asset).await {
                Ok(service) => asset_remote_exists_state(service.remote_exists().await),
                Err(_) => None,
            }
        } else {
            None
        };
        Ok(AssetInfo {
            id: asset.id,
            local_path: path,
            remote_path: asset_remote_path(asset, &self.webdav),
            local_exists,
            remote_exists,
            canonical_name,
        })
    }
    pub async fn execute(
        &self,
        id: &str,
        direction: AssetDirection,
        force: bool,
        passphrase: Option<&Secret>,
        migrate_plaintext_v1: bool,
    ) -> Result<Option<PathBuf>, String> {
        let asset = asset_by_id(id)?;
        let (path, _) = self.roots.resolve(asset)?;
        // Validate the secret before starting a remote operation. It is never persisted.
        let password = if asset.sensitive {
            Some(require_asset_passphrase(asset, passphrase)?)
        } else {
            None
        };
        let service = self.service(asset).await?;
        let local_exists = tokio::fs::try_exists(&path)
            .await
            .map_err(|_| "Cannot inspect local asset")?;
        if direction == AssetDirection::Push && !local_exists {
            return Err("Local asset does not exist".into());
        }
        // Explicit force upload retains the established write-only server support.
        let remote_exists = if direction == AssetDirection::Push && force {
            false
        } else {
            service
                .remote_exists()
                .await
                .map_err(|e| format!("Failed to check remote path: {e}"))?
        };
        let action = match direction {
            AssetDirection::Push => {
                if remote_exists && !force {
                    return Err("sync_conflict: Remote asset already exists; rerun with force to overwrite.".into());
                }
                SyncAssetAction::PushOverwrite
            }
            AssetDirection::Pull => {
                if !remote_exists {
                    return Err("Remote asset does not exist. Upload this asset first.".into());
                }
                if local_exists && !force {
                    return Err("sync_conflict: Local asset already exists; rerun with force to overwrite and create a backup.".into());
                }
                SyncAssetAction::Pull
            }
            AssetDirection::Sync => decide_sync_asset_action(local_exists, remote_exists, force),
        };
        match action {
            SyncAssetAction::PushCreate | SyncAssetAction::PushOverwrite => {
                if let Some(password) = password {
                    service.push_sensitive(&path, asset.id, password).await
                } else {
                    service.push(&path, None).await
                }.map_err(|e| format!("Push failed: {e}"))?;
                Ok(None)
            }
            SyncAssetAction::Pull => {
                if let Some(password) = password {
                    service.pull_sensitive_with_backup(&path, asset.id, password, migrate_plaintext_v1).await
                } else {
                    service.pull_with_backup(&path).await
                }.map_err(|e| format!("Pull failed: {e}"))
            }
            SyncAssetAction::Conflict => Err("sync_conflict: Local and remote assets both exist; choose force/prefer-local to overwrite remote.".into()),
            SyncAssetAction::Missing => Err("Neither local nor remote asset exists".into()),
        }
    }
}
