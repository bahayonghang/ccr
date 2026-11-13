//! Fixed resource targets on captured roots, using the shared guarded writer.
pub mod claude_files;
pub mod codex_agents;
pub mod platform_files;
pub mod templates;
use super::{ConfigSource, Services};
use ccr_app::{
    platform_settings::{self, Snapshot},
    resources::system_prompts::{self, PromptFileSpec},
    runtime::{AdmissionError, Completion, EnvironmentId, Platform, RequestKey, TaskSubscription},
};
use ccr_cli::application::platform_roots::PlatformRoots;
use std::{path::PathBuf, sync::Arc};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceError {
    Unsupported,
    Invalid,
    Storage,
}
impl ResourceError {
    pub fn message(self) -> &'static str {
        match self {
            Self::Unsupported => {
                "This resource action is not supported in this environment or platform. / 此环境或平台不支持该资源操作"
            }
            Self::Invalid => {
                "The resource target or edit session is invalid. Reload before retrying. / 资源目标或编辑会话无效，请重读后重试"
            }
            Self::Storage => {
                "Cannot read or save this resource. Draft retained; check access and backups. / 无法读取或保存资源，保留草稿，请检查权限及备份"
            }
        }
    }
}

#[derive(Clone)]
pub struct PromptRow {
    pub id: &'static str,
    pub filename: &'static str,
    pub exists: bool,
    pub size: Option<u64>,
    pub path: PathBuf,
}

/// Scoped instruction text can contain private data and has no Debug/Serialize.
pub struct PromptSession {
    roots: PlatformRoots,
    pub spec: PromptFileSpec,
    pub snapshot: Snapshot,
    environment: EnvironmentId,
    platform: Platform,
}
pub enum ResourceCommand {
    ListClaudeExtensions,
    /// Legacy Claude settings arrays as a temporary copy; `full` includes credentials.
    ExportClaudeLegacy {
        full: bool,
    },
    /// Writes an already reviewed export to a new file chosen by the user.
    SaveClaudeLegacyExport {
        content: String,
        destination: PathBuf,
    },
    ListPrompts,
    OpenPrompt(&'static str),
    SavePrompt {
        session: Arc<PromptSession>,
        content: String,
    },
    CreatePrompt {
        session: Arc<PromptSession>,
    },
}
pub enum ResourceResult {
    Extensions(Vec<ccr_app::resources::claude_extensions::Source>),
    /// Temporary export text; it can hold credentials and is never logged or persisted here.
    LegacyExport {
        content: String,
        full: bool,
    },
    ExportSaved {
        conflict: bool,
    },
    Prompts(Vec<PromptRow>),
    Opened(Arc<PromptSession>),
    Saved {
        conflict: bool,
        size_warning: bool,
    },
}

pub(super) fn roots(source: &ConfigSource) -> Result<PlatformRoots, ResourceError> {
    match source {
        ConfigSource::Benchmark(layout) => Ok(layout.roots.clone()),
        ConfigSource::Local => PlatformRoots::capture().map_err(|_| ResourceError::Storage),
        ConfigSource::Fixture {
            profiles, registry, ..
        } => {
            if !profiles.is_absolute() || !registry.is_absolute() {
                return Err(ResourceError::Invalid);
            }
            let root = profiles
                .parent()
                .ok_or(ResourceError::Invalid)?
                .join("desktop-resource-fixture");
            PlatformRoots::fixture(&root).map_err(|_| ResourceError::Invalid)
        }
    }
}

fn execute(
    source: &ConfigSource,
    key: &RequestKey,
    command: ResourceCommand,
) -> Result<ResourceResult, ResourceError> {
    if !matches!(key.environment, EnvironmentId::Local(_)) {
        return Err(ResourceError::Unsupported);
    }
    // Resolve capability before default roots, so unsupported targets cannot fall back.
    let specs = system_prompts::platform_specs(key.platform.short_name())
        .map_err(|_| ResourceError::Unsupported)?;
    match command {
        ResourceCommand::ListClaudeExtensions => {
            if key.platform != Platform::Claude {
                return Err(ResourceError::Unsupported);
            }
            let roots = roots(source)?;
            Ok(ResourceResult::Extensions(
                ccr_app::resources::claude_extensions::discover(
                    &roots.claude.config_dir,
                    &roots.claude.settings_file,
                    &roots.ccr.join("backups/claude-resources"),
                    &roots.locks,
                ),
            ))
        }
        ResourceCommand::ExportClaudeLegacy { full } => {
            if key.platform != Platform::Claude {
                return Err(ResourceError::Unsupported);
            }
            let roots = roots(source)?;
            ccr_app::resources::claude_extensions::legacy_export(&roots.claude.settings_file, full)
                .map(|content| ResourceResult::LegacyExport { content, full })
                .map_err(|_| ResourceError::Storage)
        }
        ResourceCommand::SaveClaudeLegacyExport {
            content,
            destination,
        } => {
            if key.platform != Platform::Claude {
                return Err(ResourceError::Unsupported);
            }
            check_export_destination(source, &destination)?;
            // 沿用配置方案导出的受保护写入：拒绝链接路径、只写新文件、锁与原子写入。
            ccr_app::config::source::save_export(&destination, &content)
                .map(|written| ResourceResult::ExportSaved { conflict: !written })
                .map_err(|_| ResourceError::Storage)
        }
        ResourceCommand::ListPrompts => {
            let roots = roots(source)?;
            let rows = specs
                .into_iter()
                .map(|spec| {
                    let path = system_prompts::resolve_with_claude_root(
                        spec,
                        &roots.home,
                        &roots.claude.config_dir,
                    );
                    let description = system_prompts::file_description(spec, &path);
                    PromptRow {
                        id: spec.id,
                        filename: spec.filename,
                        exists: description["exists"].as_bool().unwrap_or(false),
                        size: description["size"].as_u64(),
                        path,
                    }
                })
                .collect();
            Ok(ResourceResult::Prompts(rows))
        }
        ResourceCommand::OpenPrompt(id) => {
            let spec = *specs
                .into_iter()
                .find(|spec| spec.id == id)
                .ok_or(ResourceError::Invalid)?;
            let roots = roots(source)?;
            let path = system_prompts::resolve_with_claude_root(
                &spec,
                &roots.home,
                &roots.claude.config_dir,
            );
            let snapshot =
                platform_settings::read_text(&path).map_err(|_| ResourceError::Storage)?;
            Ok(ResourceResult::Opened(Arc::new(PromptSession {
                roots,
                spec,
                snapshot,
                environment: key.environment.clone(),
                platform: key.platform,
            })))
        }
        ResourceCommand::SavePrompt { session, content } => {
            check_session(source, &session)?;
            save(key, &session, &content, &session.snapshot.token)
        }
        ResourceCommand::CreatePrompt { session } => {
            check_session(source, &session)?;
            save(key, &session, "", "")
        }
    }
}

/// 夹具与基准环境只允许写回各自的临时根，Local 由用户在保存对话框中选择位置。
fn check_export_destination(
    source: &ConfigSource,
    destination: &std::path::Path,
) -> Result<(), ResourceError> {
    let root = match source {
        ConfigSource::Local => return Ok(()),
        ConfigSource::Benchmark(layout) => layout.root.clone(),
        ConfigSource::Fixture { profiles, .. } => profiles
            .parent()
            .ok_or(ResourceError::Invalid)?
            .to_path_buf(),
    };
    let root = root.canonicalize().map_err(|_| ResourceError::Invalid)?;
    destination
        .parent()
        .and_then(|parent| parent.canonicalize().ok())
        .filter(|parent| parent.starts_with(&root))
        .map(|_| ())
        .ok_or(ResourceError::Invalid)
}

fn check_session(source: &ConfigSource, session: &PromptSession) -> Result<(), ResourceError> {
    if let Some(layout) = source.benchmark() {
        if session.roots.home != layout.roots.home
            || session.roots.ccr != layout.roots.ccr
            || session.roots.locks != layout.roots.locks
        {
            return Err(ResourceError::Invalid);
        }
        layout
            .check_path(&session.snapshot.source)
            .map_err(|_| ResourceError::Invalid)?;
    }
    Ok(())
}

fn save(
    key: &RequestKey,
    session: &PromptSession,
    content: &str,
    token: &str,
) -> Result<ResourceResult, ResourceError> {
    if key.environment != session.environment || key.platform != session.platform {
        return Err(ResourceError::Invalid);
    }
    let fixed_path = system_prompts::resolve_with_claude_root(
        &session.spec,
        &session.roots.home,
        &session.roots.claude.config_dir,
    );
    if fixed_path != session.snapshot.source {
        return Err(ResourceError::Invalid);
    }
    let result = system_prompts::save_prompt_file_with_locks(
        &session.spec,
        &fixed_path,
        &session
            .roots
            .ccr
            .join("backups")
            .join(session.platform.short_name()),
        content,
        token,
        &session.roots.locks,
    )
    .map_err(|_| ResourceError::Storage)?;
    Ok(ResourceResult::Saved {
        conflict: result["status"] == "conflict",
        size_warning: result["warning"] == "size",
    })
}

impl Services {
    pub fn resource_request(
        &self,
        key: RequestKey,
        command: ResourceCommand,
    ) -> Result<TaskSubscription<ResourceResult, ResourceError>, AdmissionError> {
        let source = self.source.clone();
        let run_key = key.clone();
        if matches!(
            command,
            ResourceCommand::ListPrompts
                | ResourceCommand::OpenPrompt(_)
                | ResourceCommand::ListClaudeExtensions
                | ResourceCommand::ExportClaudeLegacy { .. }
        ) {
            self.executor
                .read(key, move |_, _| execute(&source, &run_key, command))
        } else {
            self.executor.job(key, move |cancel, _| {
                if cancel.is_cancelled() {
                    return Completion::Cancelled;
                }
                match execute(&source, &run_key, command) {
                    Ok(value) => Completion::Success(value),
                    Err(error) => Completion::Failed(error),
                }
            })
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn key(platform: Platform) -> RequestKey {
        RequestKey {
            environment: EnvironmentId::Local("fixture".into()),
            platform,
            generation: 1,
            request_id: 1,
        }
    }
    fn fixture(root: &std::path::Path) -> ConfigSource {
        ConfigSource::Fixture {
            profiles: root.join("profiles.toml"),
            registry: root.join("config.toml"),
            delay: std::time::Duration::ZERO,
        }
    }
    #[test]
    fn fixed_resource_missing_read_does_not_create_and_unsupported_never_falls_back() {
        let root = tempfile::tempdir().unwrap();
        let source = fixture(root.path());
        let ResourceResult::Prompts(rows) = execute(
            &source,
            &key(Platform::Claude),
            ResourceCommand::ListPrompts,
        )
        .unwrap() else {
            panic!("rows")
        };
        assert_eq!(rows.len(), 1);
        assert!(!rows[0].exists);
        assert!(!root.path().join("desktop-resource-fixture").exists());
        assert!(matches!(
            execute(&source, &key(Platform::Grok), ResourceCommand::ListPrompts),
            Err(ResourceError::Unsupported)
        ));
        let mut remote = key(Platform::Claude);
        remote.environment = EnvironmentId::Ssh("fixture".into());
        assert!(matches!(
            execute(&ConfigSource::Local, &remote, ResourceCommand::ListPrompts),
            Err(ResourceError::Unsupported)
        ));
    }
    #[test]
    fn two_resource_clients_keep_original_token_and_reject_identity_change() {
        let root = tempfile::tempdir().unwrap();
        let source = fixture(root.path());
        let key = key(Platform::Codex);
        let ResourceResult::Opened(a) =
            execute(&source, &key, ResourceCommand::OpenPrompt("codex-agents")).unwrap()
        else {
            panic!("a")
        };
        let ResourceResult::Opened(b) =
            execute(&source, &key, ResourceCommand::OpenPrompt("codex-agents")).unwrap()
        else {
            panic!("b")
        };
        assert!(matches!(
            execute(
                &source,
                &key,
                ResourceCommand::SavePrompt {
                    session: a.clone(),
                    content: "winner".into()
                }
            )
            .unwrap(),
            ResourceResult::Saved {
                conflict: false,
                ..
            }
        ));
        assert!(matches!(
            execute(
                &source,
                &key,
                ResourceCommand::SavePrompt {
                    session: b.clone(),
                    content: "stale".into()
                }
            )
            .unwrap(),
            ResourceResult::Saved { conflict: true, .. }
        ));
        assert!(matches!(
            execute(
                &source,
                &key,
                ResourceCommand::CreatePrompt { session: b.clone() }
            )
            .unwrap(),
            ResourceResult::Saved { conflict: true, .. }
        ));
        let mut other = key.clone();
        other.platform = Platform::Claude;
        assert!(matches!(
            execute(
                &source,
                &other,
                ResourceCommand::SavePrompt {
                    session: b,
                    content: "wrong platform".into()
                }
            ),
            Err(ResourceError::Invalid)
        ));
        assert_eq!(
            std::fs::read_to_string(&a.snapshot.source).unwrap(),
            "winner"
        );
    }
    #[test]
    fn claude_extension_reads_reject_other_platforms_and_remote_without_fallback() {
        let root = tempfile::tempdir().unwrap();
        let source = fixture(root.path());
        for command in [
            ResourceCommand::ListClaudeExtensions,
            ResourceCommand::ExportClaudeLegacy { full: false },
        ] {
            assert!(matches!(
                execute(&source, &key(Platform::Codex), command),
                Err(ResourceError::Unsupported)
            ));
        }
        let mut remote = key(Platform::Claude);
        remote.environment = EnvironmentId::Ssh("fixture".into());
        for command in [
            ResourceCommand::ListClaudeExtensions,
            ResourceCommand::ExportClaudeLegacy { full: true },
            ResourceCommand::SaveClaudeLegacyExport {
                content: "x".into(),
                destination: root.path().join("remote.json"),
            },
        ] {
            assert!(matches!(
                execute(&ConfigSource::Local, &remote, command),
                Err(ResourceError::Unsupported)
            ));
        }
        assert!(!root.path().join("remote.json").exists());
        assert!(!root.path().join("desktop-resource-fixture").exists());
    }
    #[test]
    fn legacy_export_is_read_only_and_saves_only_new_files_inside_the_fixture() {
        let root = tempfile::tempdir().unwrap();
        let source = fixture(root.path());
        let settings = roots(&source).unwrap().claude.settings_file;
        std::fs::create_dir_all(settings.parent().unwrap()).unwrap();
        let original = br#"{"plugins":[{"id":"p","config":{"apiKey":"keyAAAASENTINELBBBB"}}]}"#;
        std::fs::write(&settings, original).unwrap();
        let export = |full| match execute(
            &source,
            &key(Platform::Claude),
            ResourceCommand::ExportClaudeLegacy { full },
        ) {
            Ok(ResourceResult::LegacyExport { content, full: got }) if got == full => content,
            _ => panic!("export"),
        };
        assert!(!export(false).contains("SENTINEL"));
        let full = export(true);
        assert!(full.contains("keyAAAASENTINELBBBB"));
        assert!(matches!(
            execute(
                &source,
                &key(Platform::Codex),
                ResourceCommand::ExportClaudeLegacy { full: true }
            ),
            Err(ResourceError::Unsupported)
        ));
        let save = |destination: std::path::PathBuf| {
            execute(
                &source,
                &key(Platform::Claude),
                ResourceCommand::SaveClaudeLegacyExport {
                    content: full.clone(),
                    destination,
                },
            )
        };
        let destination = root.path().join("legacy.json");
        assert!(matches!(
            save(destination.clone()),
            Ok(ResourceResult::ExportSaved { conflict: false })
        ));
        assert_eq!(std::fs::read_to_string(&destination).unwrap(), full);
        std::fs::write(&destination, "existing").unwrap();
        assert!(matches!(
            save(destination.clone()),
            Ok(ResourceResult::ExportSaved { conflict: true })
        ));
        assert_eq!(std::fs::read_to_string(&destination).unwrap(), "existing");
        let outside = tempfile::tempdir().unwrap();
        assert!(matches!(
            save(outside.path().join("legacy.json")),
            Err(ResourceError::Invalid)
        ));
        assert!(!outside.path().join("legacy.json").exists());
        assert_eq!(std::fs::read(&settings).unwrap(), original);
    }
}
