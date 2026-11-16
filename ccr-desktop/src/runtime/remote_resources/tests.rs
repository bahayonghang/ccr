#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;
use crate::runtime::{
    mcp::{BatchInput, BatchOutcome, McpTarget},
    resources::{
        self, ResourceCommand as RC, ResourceResult as RR, claude_files as cf, codex_agents as ca,
        platform_files as pf, templates as tp,
    },
};
use ccr_app::{
    environments::ssh::{HostInput, Workspace, fixture::FixtureTransport},
    mcp_document::EntryMutation,
    runtime::{Completion, Platform, RequestKey, RuntimeOwner, TaskEvent, TaskSubscription},
};
use serde_json::json;
use std::time::Duration;
struct Fixture {
    root: tempfile::TempDir,
    owner: RuntimeOwner,
    services: Services,
    workspace: Arc<Workspace>,
    transport: Arc<FixtureTransport>,
    id: String,
}
async fn terminal<T: Send + Sync + 'static, E: Send + Sync + 'static>(
    mut task: TaskSubscription<T, E>,
) -> Arc<Completion<T, E>> {
    tokio::time::timeout(Duration::from_secs(10), async {
        while let Some(event) = task.next().await {
            if let TaskEvent::Terminal(value) = event {
                return value;
            }
        }
        panic!("missing terminal")
    })
    .await
    .unwrap()
}
impl Fixture {
    async fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let owner = RuntimeOwner::new().unwrap();
        let services = Services::new(
            owner.handle(),
            ConfigSource::Fixture {
                profiles: root.path().join("profiles.toml"),
                registry: root.path().join("registry.toml"),
                delay: Duration::ZERO,
            },
        );
        let (workspace, transport) = Workspace::fixture(
            root.path().join("hosts.db"),
            root.path().join("known_hosts"),
        );
        let workspace = Arc::new(workspace);
        services
            .ssh
            .set(workspace.clone())
            .unwrap_or_else(|_| panic!("fixture owner"));
        let host = workspace
            .save_host(HostInput {
                name: "Resources".into(),
                host: "resources.fixture.invalid".into(),
                user: "fixture".into(),
                port: 22,
                remote_home: "/srv/resources".into(),
                ..Default::default()
            })
            .await
            .unwrap();
        let probe = workspace
            .probe(&host.id, CancellationToken::new())
            .await
            .unwrap();
        workspace
            .confirm(&probe.challenge_id, CancellationToken::new())
            .await
            .unwrap();
        workspace
            .connect(&host.id, CancellationToken::new())
            .await
            .unwrap();
        services.select_environment(EnvironmentId::Ssh(host.id.clone()));
        for path in ["/work/project", "/work/codex-project", "/work/mcp-project"] {
            transport.set_directory(path);
        }
        for path in ["desktop-resource-fixture", "desktop-mcp-fixture"] {
            std::fs::write(root.path().join(path), "local-poison").unwrap();
        }
        Self {
            root,
            owner,
            services,
            workspace,
            transport,
            id: host.id,
        }
    }
    fn key(&self, platform: Platform) -> RequestKey {
        RequestKey {
            environment: EnvironmentId::Ssh(self.id.clone()),
            platform,
            generation: 1,
            request_id: 1,
        }
    }
    fn set(&self, relative: &str, content: &str) {
        self.transport.set_file(
            "resources.fixture.invalid",
            &format!("/srv/resources/{relative}"),
            content,
        );
    }
    fn file(&self, relative: &str) -> Option<String> {
        self.transport.file(
            "resources.fixture.invalid",
            &format!("/srv/resources/{relative}"),
        )
    }
    async fn prompt(
        &self,
        platform: Platform,
        command: RC,
    ) -> Arc<Completion<RR, resources::ResourceError>> {
        terminal(
            self.services
                .resource_request(self.key(platform), command)
                .unwrap(),
        )
        .await
    }
    async fn claude(&self, command: cf::Command) -> Arc<Completion<cf::Outcome, cf::Error>> {
        terminal(
            self.services
                .claude_resource_request(self.key(Platform::Claude), command)
                .unwrap(),
        )
        .await
    }
    async fn agent_done(&self) -> Result<ca::Outcome, ca::Error> {
        tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                let state = self.services.codex_agents_state();
                if !state.busy {
                    return state.result.unwrap();
                }
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .unwrap()
    }
    async fn file_done(&self) -> Result<pf::Outcome, pf::Error> {
        tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                let state = self.services.platform_files_state();
                if !state.busy {
                    return state.result.unwrap();
                }
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .unwrap()
    }
    async fn finish(self) {
        tokio::task::spawn_blocking(move || self.owner.shutdown())
            .await
            .unwrap();
        assert_eq!(
            std::fs::read_to_string(self.root.path().join("desktop-resource-fixture")).unwrap(),
            "local-poison"
        );
    }
}
#[tokio::test]
async fn ssh_instruction_metadata_reads_no_bodies_and_four_editors_keep_cas_backups_and_session() {
    let f = Fixture::new().await;
    for (platform, id, path) in [
        (Platform::Claude, "claude-user-memory", ".claude/CLAUDE.md"),
        (Platform::Codex, "codex-agents", ".codex/AGENTS.md"),
        (Platform::Gemini, "gemini-md", ".gemini/GEMINI.md"),
        (
            Platform::OpenCode,
            "opencode-agents",
            ".config/opencode/AGENTS.md",
        ),
    ] {
        f.set(path, "original");
        let before = f.transport.batches().len();
        let listed = f.prompt(platform, RC::ListPrompts).await;
        let Completion::Success(RR::Prompts(rows)) = listed.as_ref() else {
            panic!("metadata list");
        };
        assert_eq!(
            rows[0].path.to_string_lossy(),
            format!("/srv/resources/{path}")
        );
        assert_eq!(rows[0].size, Some(8));
        assert!(
            !f.transport.batches()[before..]
                .iter()
                .any(|line| line.starts_with("get "))
        );
        let opened = f.prompt(platform, RC::OpenPrompt(id)).await;
        let Completion::Success(RR::Opened(a)) = opened.as_ref() else {
            panic!("open");
        };
        let second = f.prompt(platform, RC::OpenPrompt(id)).await;
        let Completion::Success(RR::Opened(b)) = second.as_ref() else {
            panic!("open2");
        };
        assert_eq!(a.source_label(), format!("/srv/resources/{path}"));
        assert!(matches!(
            f.prompt(
                platform,
                RC::SavePrompt {
                    session: a.clone(),
                    content: "changed".into()
                }
            )
            .await
            .as_ref(),
            Completion::Success(RR::Saved {
                conflict: false,
                ..
            })
        ));
        assert_eq!(f.file(path).as_deref(), Some("changed"));
        assert!(matches!(
            f.prompt(
                platform,
                RC::SavePrompt {
                    session: b.clone(),
                    content: "stale".into()
                }
            )
            .await
            .as_ref(),
            Completion::Failed(resources::ResourceError::Remote(Error::Conflict))
        ));
        assert!(
            f.transport
                .batches()
                .iter()
                .any(|line| line.starts_with(&format!("rename /srv/resources/{path}.bak.")))
        );
    }
    let result = f
        .prompt(Platform::Codex, RC::OpenPrompt("codex-agents"))
        .await;
    let Completion::Success(RR::Opened(old)) = result.as_ref() else {
        panic!("old");
    };
    f.workspace.disconnect().await;
    f.workspace
        .connect(&f.id, CancellationToken::new())
        .await
        .unwrap();
    assert!(matches!(
        f.prompt(
            Platform::Codex,
            RC::SavePrompt {
                session: old.clone(),
                content: "after-reconnect".into()
            }
        )
        .await
        .as_ref(),
        Completion::Failed(resources::ResourceError::Remote(_))
    ));
    assert_eq!(f.file(".codex/AGENTS.md").as_deref(), Some("changed"));
    f.finish().await;
}
#[tokio::test]
async fn ssh_claude_rules_styles_collections_discovery_and_presets_reuse_owners() {
    let f = Fixture::new().await;
    for kind in [
        cf::Kind::Rules,
        cf::Kind::OutputStyles,
        cf::Kind::Agents,
        cf::Kind::SlashCommands,
        cf::Kind::Plugins,
    ] {
        let name = if kind.is_file() {
            "sample.md"
        } else {
            "sample"
        };
        let result = f
            .claude(cf::Command::Open {
                kind,
                name: name.into(),
                creating: true,
            })
            .await;
        let Completion::Success(cf::Outcome::Opened(session)) = result.as_ref() else {
            panic!("open claude");
        };
        let content = if kind.is_file() {
            "# retained markdown".into()
        } else {
            session.initial_text()
        };
        assert!(matches!(
            f.claude(cf::Command::Commit {
                session: session.clone(),
                content: Some(content)
            })
            .await
            .as_ref(),
            Completion::Success(cf::Outcome::Committed)
        ));
        let result = f.claude(cf::Command::List(kind)).await;
        let Completion::Success(cf::Outcome::Listed(rows)) = result.as_ref() else {
            panic!("list");
        };
        assert!(rows.iter().any(|row| row.name == name));
    }
    f.set(
        ".claude/agents/reviewer.md",
        "---\nname: reviewer\n---\nprivate instructions",
    );
    f.set(
        ".claude/skills/review/SKILL.md",
        "---\nname: review\n---\nprivate skill",
    );
    f.set(".claude/commands/review.md", "private slash");
    f.set(
        ".claude/plugins/installed_plugins.json",
        r#"{"plugins":{"sample@fixture":[{}]}}"#,
    );
    let discovery = f.prompt(Platform::Claude, RC::ListClaudeExtensions).await;
    let Completion::Success(RR::Extensions(sources)) = discovery.as_ref() else {
        panic!("discovery");
    };
    assert!(
        sources
            .iter()
            .flat_map(|s| &s.entries)
            .any(|row| row.text.contains("reviewer.md"))
    );
    assert!(
        sources
            .iter()
            .flat_map(|s| &s.entries)
            .any(|row| row.text.contains("review/SKILL.md"))
    );
    let exported = f
        .prompt(Platform::Claude, RC::ExportClaudeLegacy { full: false })
        .await;
    assert!(matches!(
        exported.as_ref(),
        Completion::Success(RR::LegacyExport { full: false, .. })
    ));
    let destination = f.root.path().join("reviewed-legacy-export.json");
    let content = match exported.as_ref() {
        Completion::Success(RR::LegacyExport { content, .. }) => content.clone(),
        _ => panic!("export"),
    };
    let task = f
        .services
        .resource_request(
            f.key(Platform::Claude),
            RC::SaveClaudeLegacyExport {
                content: content.clone(),
                destination: destination.clone(),
            },
        )
        .unwrap();
    assert_eq!(
        task.key().environment,
        EnvironmentId::Local("application".into())
    );
    assert!(matches!(
        terminal(task).await.as_ref(),
        Completion::Success(RR::ExportSaved { conflict: false })
    ));
    assert_eq!(std::fs::read_to_string(destination).unwrap(), content);
    for platform in [Platform::Claude, Platform::Codex, Platform::Gemini] {
        let result = terminal(
            f.services
                .template_request(f.key(platform), tp::Command::List)
                .unwrap(),
        )
        .await;
        let Completion::Success(tp::Outcome::Listed(catalog)) = result.as_ref() else {
            panic!("presets");
        };
        let mutation =
            ccr_app::resources::prompt_presets::Mutation::Upsert(ccr_skills::PromptPreset {
                name: "fixture".into(),
                description: None,
                target_file: ccr_skills::PromptTarget::Agents,
                content: "fixture prompt".into(),
                tags: vec![],
            });
        assert!(matches!(
            terminal(
                f.services
                    .template_request(
                        f.key(platform),
                        tp::Command::Commit {
                            catalog: catalog.clone(),
                            mutation
                        }
                    )
                    .unwrap()
            )
            .await
            .as_ref(),
            Completion::Success(tp::Outcome::Committed)
        ));
        assert!(
            f.file(&format!(
                ".ccr/platforms/{}/prompts.toml",
                platform.short_name()
            ))
            .unwrap()
            .contains("fixture prompt")
        );
    }
    f.finish().await;
}
#[tokio::test]
async fn ssh_platform_user_project_resources_edit_delete_and_plugin_metadata_without_code() {
    let f = Fixture::new().await;
    for project in [false, true] {
        for kind in [
            pf::Kind::OpenCodeAgent,
            pf::Kind::OpenCodeCommand,
            pf::Kind::AntigravityCommand,
        ] {
            let scope = if project {
                pf::Scope::Project("/work/project".into())
            } else {
                pf::Scope::User
            };
            let name = format!("sample.{}", kind.extension());
            let result = terminal(
                f.services
                    .platform_files_read(
                        f.key(pf::platform(kind)),
                        pf::Read::Open {
                            kind,
                            scope: scope.clone(),
                            name: name.clone(),
                            creating: true,
                        },
                    )
                    .unwrap(),
            )
            .await;
            let Completion::Success(pf::ReadResult::Opened(session)) = result.as_ref() else {
                panic!("open platform");
            };
            assert!(session.source_label().starts_with(if project {
                "/work/project/"
            } else {
                "/srv/resources/"
            }));
            f.services
                .commit_platform_file(
                    f.key(pf::platform(kind)),
                    session.clone(),
                    Some(kind.initial_text().into()),
                )
                .unwrap();
            assert_eq!(f.file_done().await, Ok(pf::Outcome::Saved));
            let result = terminal(
                f.services
                    .platform_files_read(
                        f.key(pf::platform(kind)),
                        pf::Read::Open {
                            kind,
                            scope,
                            name,
                            creating: false,
                        },
                    )
                    .unwrap(),
            )
            .await;
            let Completion::Success(pf::ReadResult::Opened(session)) = result.as_ref() else {
                panic!("reopen platform");
            };
            f.services
                .commit_platform_file(f.key(pf::platform(kind)), session.clone(), None)
                .unwrap();
            assert_eq!(f.file_done().await, Ok(pf::Outcome::Deleted));
        }
    }
    f.set(".config/opencode/plugins/tool.js", "secret plugin code");
    f.set(".config/opencode/plugins/module.CJS", "private CJS");
    f.set(".config/opencode/plugins/module.mjs", "private MJS");
    f.set(
        ".config/opencode/plugins/空 格.js",
        "private unsupported code",
    );
    f.transport
        .set_unsupported_path("/srv/resources/.config/opencode/plugins/shared.ts");
    let before = f.transport.batches().len();
    let listed = terminal(
        f.services
            .platform_files_read(
                f.key(Platform::OpenCode),
                pf::Read::List {
                    kind: pf::Kind::OpenCodePlugin,
                    scope: pf::Scope::User,
                },
            )
            .unwrap(),
    )
    .await;
    let Completion::Success(pf::ReadResult::Listed(rows)) = listed.as_ref() else {
        panic!("plugins");
    };
    assert_eq!(
        rows.iter().find(|row| row.key == "tool.js").unwrap().bytes,
        18
    );
    assert!(
        rows.iter()
            .any(|row| row.key == "module.CJS" && !row.unsupported)
    );
    assert!(
        rows.iter()
            .any(|row| row.key == "module.mjs" && !row.unsupported)
    );
    assert_eq!(rows.iter().filter(|row| row.unsupported).count(), 2);
    assert!(
        !f.transport.batches()[before..]
            .iter()
            .any(|line| line.starts_with("get "))
    );
    f.set(
        ".gemini/antigravity-cli/extensions/example/gemini-extension.json",
        r#"{"name":"example","version":"1.0"}"#,
    );
    let result = terminal(
        f.services
            .platform_files_read(
                f.key(Platform::Gemini),
                pf::Read::Open {
                    kind: pf::Kind::Extension,
                    scope: pf::Scope::User,
                    name: "example/gemini-extension.json".into(),
                    creating: false,
                },
            )
            .unwrap(),
    )
    .await;
    let Completion::Success(pf::ReadResult::Opened(session)) = result.as_ref() else {
        panic!("manifest");
    };
    assert!(session.initial_text().contains("example"));
    assert!(matches!(
        f.services
            .commit_platform_file(f.key(Platform::Gemini), session.clone(), None),
        Err(pf::Error::Unsupported)
    ));
    f.finish().await;
}
#[tokio::test]
async fn ssh_codex_agents_crud_and_cross_scope_copy_use_distinct_remote_roots() {
    let f = Fixture::new().await;
    let content = "name = 'reviewer'\ndescription = 'Fixture review'\ndeveloper_instructions = 'Review carefully'\n";
    f.services
        .start_codex_agents(
            f.key(Platform::Codex),
            ca::Mutation::Add {
                scope: ca::Scope::Global,
                name: "reviewer".into(),
                content: content.into(),
            },
        )
        .unwrap();
    assert_eq!(f.agent_done().await, Ok(ca::Outcome::Saved));
    let opened = terminal(
        f.services
            .codex_agents_read(
                f.key(Platform::Codex),
                ca::Read::Open(ca::Scope::Global, "reviewer".into()),
            )
            .unwrap(),
    )
    .await;
    let Completion::Success(ca::ReadResult::Opened(session)) = opened.as_ref() else {
        panic!("codex");
    };
    f.services
        .start_codex_agents(
            f.key(Platform::Codex),
            ca::Mutation::Copy {
                session: session.clone(),
                target: ca::Scope::Project("/work/codex-project".into()),
                name: "copy".into(),
            },
        )
        .unwrap();
    assert_eq!(f.agent_done().await, Ok(ca::Outcome::Copied));
    assert!(
        f.transport
            .file(
                "resources.fixture.invalid",
                "/work/codex-project/.codex/agents/copy.toml"
            )
            .is_some()
    );
    assert!(f.file(".codex/agents/copy.toml").is_none());
    let opened = terminal(
        f.services
            .codex_agents_read(
                f.key(Platform::Codex),
                ca::Read::Open(
                    ca::Scope::Project("/work/codex-project".into()),
                    "copy".into(),
                ),
            )
            .unwrap(),
    )
    .await;
    let Completion::Success(ca::ReadResult::Opened(session)) = opened.as_ref() else {
        panic!("project");
    };
    f.services
        .start_codex_agents(
            f.key(Platform::Codex),
            ca::Mutation::Rename {
                session: session.clone(),
                name: "renamed".into(),
            },
        )
        .unwrap();
    assert_eq!(f.agent_done().await, Ok(ca::Outcome::Renamed));
    assert!(
        f.transport
            .file(
                "resources.fixture.invalid",
                "/work/codex-project/.codex/agents/copy.toml"
            )
            .is_none()
    );
    assert!(
        f.transport
            .file(
                "resources.fixture.invalid",
                "/work/codex-project/.codex/agents/renamed.toml"
            )
            .is_some()
    );
    f.finish().await;
}
#[tokio::test]
async fn ssh_mcp_edit_copy_project_keys_and_batch_have_per_source_cas_and_partial_results() {
    let f = Fixture::new().await;
    let targets = vec![
        McpTarget::ClaudeUser,
        McpTarget::Codex,
        McpTarget::Antigravity,
    ];
    let preset = ccr_skills::get_builtin_presets()
        .into_iter()
        .find(|p| p.api_key_env.is_none())
        .unwrap();
    let prepared = terminal(
        f.services
            .mcp_batch_prepare_request(
                f.key(Platform::Claude),
                BatchInput::Preset {
                    id: preset.id.clone(),
                    api_key: None,
                },
                targets.clone(),
                false,
            )
            .unwrap(),
    )
    .await;
    let Completion::Success(prepared) = prepared.as_ref() else {
        panic!("prepare");
    };
    let operation = f
        .services
        .mcp_batch_save_request(f.key(Platform::Claude), prepared.clone())
        .unwrap();
    let wait = |operation: Arc<crate::runtime::mcp::BatchOperation>| async move {
        tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                let state = operation.snapshot();
                if !state.running {
                    return state;
                }
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .unwrap()
    };
    let result = wait(operation).await;
    assert_eq!(result.rows.len(), 3);
    assert!(
        result.rows.iter().all(|r| r.outcome == BatchOutcome::Saved),
        "{result:?}"
    );
    let prepared = terminal(
        f.services
            .mcp_batch_prepare_request(
                f.key(Platform::Claude),
                BatchInput::Sync {
                    source: McpTarget::ClaudeUser,
                },
                vec![McpTarget::Codex, McpTarget::Antigravity],
                true,
            )
            .unwrap(),
    )
    .await;
    let Completion::Success(prepared) = prepared.as_ref() else {
        panic!("sync");
    };
    f.set(".codex/config.toml", "model = 'external'\n");
    let result = wait(
        f.services
            .mcp_batch_save_request(f.key(Platform::Claude), prepared.clone())
            .unwrap(),
    )
    .await;
    assert!(
        result
            .rows
            .iter()
            .filter(|r| r.target == McpTarget::Codex)
            .all(|r| matches!(
                r.outcome,
                BatchOutcome::Failed(crate::runtime::mcp::McpError::Remote(Error::Conflict))
            ))
    );
    assert!(
        result
            .rows
            .iter()
            .filter(|r| r.target == McpTarget::Antigravity)
            .all(|r| r.outcome == BatchOutcome::Saved)
    );
    for target in [
        McpTarget::ClaudeLocal("/work/mcp-project".into()),
        McpTarget::ClaudeProject("/work/mcp-project".into()),
    ] {
        let opened = terminal(
            f.services
                .mcp_edit_request(f.key(Platform::Claude), target)
                .unwrap(),
        )
        .await;
        let Completion::Success(session) = opened.as_ref() else {
            panic!("project editor");
        };
        assert!(matches!(
            terminal(
                f.services
                    .mcp_mutate_request(
                        f.key(Platform::Claude),
                        session.clone(),
                        "fixture".into(),
                        EntryMutation::Add(
                            json!({"command":"fixture-mcp","env":{"TOKEN":"mcp-secret-sentinel"}})
                        )
                    )
                    .unwrap()
            )
            .await
            .as_ref(),
            Completion::Success(())
        ));
    }
    assert!(
        f.file(".claude.json")
            .unwrap()
            .contains("/work/mcp-project")
    );
    assert!(
        !f.file(".claude.json")
            .unwrap()
            .contains("ccr-configuration-")
    );
    assert!(
        f.transport
            .file("resources.fixture.invalid", "/work/mcp-project/.mcp.json")
            .unwrap()
            .contains("fixture-mcp")
    );
    let catalog = terminal(
        f.services
            .mcp_project_request(f.key(Platform::Claude), Some("/work/mcp-project".into()))
            .unwrap(),
    )
    .await;
    let Completion::Success(catalog) = catalog.as_ref() else {
        panic!("catalog");
    };
    assert!(!format!("{catalog:?}").contains("mcp-secret-sentinel"));
    let source = catalog
        .entries
        .iter()
        .find(|entry| {
            entry.source == crate::runtime::mcp::McpSource::ProjectFile && entry.name == "fixture"
        })
        .unwrap()
        .clone();
    let prepared = terminal(
        f.services
            .mcp_copy_prepare_request(f.key(Platform::Claude), source, McpTarget::Codex)
            .unwrap(),
    )
    .await;
    let Completion::Success(copy) = prepared.as_ref() else {
        panic!("single copy");
    };
    assert!(matches!(
        terminal(
            f.services
                .mcp_copy_save_request(
                    f.key(Platform::Claude),
                    copy.clone(),
                    "copied-fixture".into()
                )
                .unwrap()
        )
        .await
        .as_ref(),
        Completion::Success(())
    ));
    let written = f.file(".codex/config.toml").unwrap();
    assert!(written.contains("copied-fixture") && written.contains("mcp-secret-sentinel"));
    f.finish().await;
}
#[tokio::test]
async fn unsupported_remote_names_and_links_remain_visible_without_blocking_healthy_rules() {
    let f = Fixture::new().await;
    f.set(".claude/rules/healthy.md", "healthy");
    f.set(".claude/rules/空 格.md", "private unrepresentable body");
    f.transport
        .set_unsupported_path("/srv/resources/.claude/rules/shared.md");
    let before = f.transport.batches().len();
    let result = f.claude(cf::Command::List(cf::Kind::Rules)).await;
    assert!(
        !f.transport.batches()[before..]
            .iter()
            .any(|line| line.starts_with("get "))
    );
    let Completion::Success(cf::Outcome::Listed(rows)) = result.as_ref() else {
        panic!("health");
    };
    assert!(
        rows.iter()
            .any(|r| r.name == "healthy.md" && !r.unsupported)
    );
    assert_eq!(rows.iter().filter(|r| r.unsupported).count(), 2);
    let opened = f
        .claude(cf::Command::Open {
            kind: cf::Kind::Rules,
            name: "healthy.md".into(),
            creating: false,
        })
        .await;
    let Completion::Success(cf::Outcome::Opened(session)) = opened.as_ref() else {
        panic!("healthy");
    };
    assert!(matches!(
        f.claude(cf::Command::Commit {
            session: session.clone(),
            content: Some("updated".into())
        })
        .await
        .as_ref(),
        Completion::Success(cf::Outcome::Committed)
    ));
    assert!(
        !f.transport
            .batches()
            .iter()
            .any(|line| line.starts_with("get ")
                && (line.contains("空 格") || line.contains("shared.md")))
    );
    assert_eq!(
        f.file(".claude/rules/healthy.md").as_deref(),
        Some("updated")
    );
    f.finish().await;
}
#[tokio::test]
async fn ssh_codex_source_installs_preserve_posix_tracking_and_cached_owner_actions() {
    let f = Fixture::new().await;
    f.set(".ccr/platforms/codex/agent-sources/sources.json",&json!({"sources":[{"id":"src_fixture","repoUrl":"https://github.com/example/repo","owner":"example","repo":"repo","defaultBranch":"main","status":"ok","agentCount":1,"diagnosticsCount":0,"scanComplete":true}]}).to_string());
    f.set(".ccr/platforms/codex/agent-sources/catalogs/src_fixture.json",&json!({"sourceId":"src_fixture","repoUrl":"https://github.com/example/repo","defaultBranch":"main","status":"ok","scanComplete":true,"truncated":false,"scannedAt":"2026-01-01T00:00:00Z","agents":[{"id":"agent_fixture","sourcePath":"agents/reviewer.toml","fileName":"reviewer.toml","blobSha":"blob-fixture","contentHash":"hash-fixture","category":"agents","categoryLabel":"Agents","name":"reviewer","rawToml":"name='reviewer'\ndescription='Fixture review'\ndeveloper_instructions='review carefully'\n"}]}).to_string());
    let sources = terminal(
        f.services
            .codex_agents_read(f.key(Platform::Codex), ca::Read::Sources)
            .unwrap(),
    )
    .await;
    assert!(
        matches!(sources.as_ref(),Completion::Success(ca::ReadResult::Sources(rows)) if rows.len()==1)
    );
    f.services
        .start_codex_agents(
            f.key(Platform::Codex),
            ca::Mutation::Source(ca::SourceAction::Install(
                ccr_app::codex_agents::sources::CodexAgentSourceInstallRequest {
                    source_id: "src_fixture".into(),
                    agent_id: "agent_fixture".into(),
                    target_name: Some("reviewer".into()),
                    conflict_mode: None,
                },
            )),
        )
        .unwrap();
    assert_eq!(f.agent_done().await, Ok(ca::Outcome::SourceUpdated));
    let stored = f
        .file(".ccr/platforms/codex/agent-sources/installs.json")
        .unwrap();
    assert!(stored.contains("/srv/resources/.codex/agents/reviewer.toml"));
    assert!(!stored.contains("ccr-configuration-"));
    let catalog = terminal(
        f.services
            .codex_agents_read(
                f.key(Platform::Codex),
                ca::Read::Catalog("src_fixture".into()),
            )
            .unwrap(),
    )
    .await;
    let Completion::Success(ca::ReadResult::Catalog(catalog)) = catalog.as_ref() else {
        panic!("source catalog");
    };
    let value = serde_json::to_value(catalog).unwrap();
    assert_eq!(
        value["installs"][0]["targetPath"],
        "/srv/resources/.codex/agents/reviewer.toml"
    );
    let install = value["installs"][0]["id"].as_str().unwrap().to_owned();
    for action in [
        ca::SourceAction::AcceptLocal(install.clone()),
        ca::SourceAction::Untrack(install),
        ca::SourceAction::Remove("src_fixture".into()),
    ] {
        f.services
            .start_codex_agents(f.key(Platform::Codex), ca::Mutation::Source(action))
            .unwrap();
        assert_eq!(f.agent_done().await, Ok(ca::Outcome::SourceUpdated));
    }
    assert!(f.file(".codex/agents/reviewer.toml").is_some());
    assert!(
        f.file(".ccr/platforms/codex/agent-sources/catalogs/src_fixture.json")
            .is_none()
    );
    f.finish().await;
}
#[tokio::test]
async fn ssh_queued_resource_read_cannot_gain_reconnected_authority() {
    let f = Fixture::new().await;
    let gate = Arc::new(std::sync::Barrier::new(
        ccr_app::runtime::BLOCKING_CONCURRENCY + 1,
    ));
    let release = Arc::new((std::sync::Mutex::new(false), std::sync::Condvar::new()));
    let mut jobs = Vec::new();
    for _ in 0..ccr_app::runtime::BLOCKING_CONCURRENCY {
        let gate = gate.clone();
        let release = release.clone();
        jobs.push(
            f.services
                .executor
                .job(f.key(Platform::Claude), move |_, _| {
                    gate.wait();
                    let mut open = release.0.lock().unwrap();
                    while !*open {
                        open = release.1.wait(open).unwrap();
                    }
                    Completion::<(), ()>::Success(())
                })
                .unwrap(),
        );
    }
    tokio::task::spawn_blocking(move || gate.wait())
        .await
        .unwrap();
    let queued = f
        .services
        .resource_request(f.key(Platform::Claude), RC::ListPrompts)
        .unwrap();
    f.workspace.disconnect().await;
    f.workspace
        .connect(&f.id, CancellationToken::new())
        .await
        .unwrap();
    *release.0.lock().unwrap() = true;
    release.1.notify_all();
    assert!(matches!(
        terminal(queued).await.as_ref(),
        Completion::Failed(resources::ResourceError::Remote(_))
    ));
    for job in jobs {
        assert!(matches!(
            terminal(job).await.as_ref(),
            Completion::Success(())
        ));
    }
    f.finish().await;
}
#[tokio::test]
async fn ssh_mcp_cancellation_during_remote_verification_keeps_preimage_and_reports_terminal() {
    let f = Fixture::new().await;
    f.set(".codex/config.toml", "model='kept'\n");
    let opened = terminal(
        f.services
            .mcp_edit_request(f.key(Platform::Codex), McpTarget::Codex)
            .unwrap(),
    )
    .await;
    let Completion::Success(session) = opened.as_ref() else {
        panic!("editor");
    };
    let before = f.transport.batches().len();
    let started = f.transport.io_started();
    f.transport.delay_io(Duration::from_secs(5));
    let task = f
        .services
        .mcp_mutate_request(
            f.key(Platform::Codex),
            session.clone(),
            "fixture".into(),
            EntryMutation::Add(json!({"command":"fixture-mcp"})),
        )
        .unwrap();
    tokio::time::timeout(Duration::from_secs(10), async {
        while f.transport.io_started() == started {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap();
    task.request_cancel();
    let result = terminal(task).await;
    assert!(matches!(
        result.as_ref(),
        Completion::Cancelled
            | Completion::Failed(crate::runtime::mcp::McpError::Remote(
                Error::Cancelled | Error::Ssh("ssh_cancelled")
            ))
    ));
    assert_eq!(
        f.file(".codex/config.toml").as_deref(),
        Some("model='kept'\n")
    );
    assert!(
        !f.transport.batches()[before..]
            .iter()
            .any(|line| line.starts_with("put "))
    );
    f.finish().await;
}

#[tokio::test]
async fn missing_remote_project_is_refused_without_creating_or_listing_the_root() {
    let f = Fixture::new().await;
    let before = f.transport.batches().len();
    let result = terminal(
        f.services
            .platform_files_read(
                f.key(Platform::OpenCode),
                pf::Read::Open {
                    kind: pf::Kind::OpenCodeAgent,
                    scope: pf::Scope::Project("/missing/project".into()),
                    name: "fixture.md".into(),
                    creating: true,
                },
            )
            .unwrap(),
    )
    .await;
    assert!(matches!(
        result.as_ref(),
        Completion::Failed(pf::Error::Remote(Error::InvalidPath))
    ));
    assert!(
        !f.transport.batches()[before..]
            .iter()
            .any(|line| line.starts_with("list ")
                || line.starts_with("get ")
                || line.starts_with("put "))
    );
    f.finish().await;
}
