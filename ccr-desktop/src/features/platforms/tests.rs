//! Headless consumer coverage, not native OS focus/IME or OAuth evidence.
use super::*;
use ccr_app::{
    platform_settings::{Document, Format},
    runtime::RuntimeOwner,
};
use ccr_cli::application::platform_roots::PlatformRoots;
use ccr_core::core::BackupPolicy;
use gpui_kit::test::{TestAppContextExt, TestWindowExt};
use gpui_kit::{
    InputEvent, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, TestAppContext,
    WindowOptions,
};

#[gpui_kit::test]
async fn configuration_sources_are_discoverable_and_managed_layers_stay_read_only(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap_or_else(|_| panic!("fixture"));
    let source = crate::runtime::ConfigSource::Fixture {
        profiles: root.path().join("profiles.toml"),
        registry: root.path().join("registry.toml"),
        delay: std::time::Duration::ZERO,
    };
    let owner = RuntimeOwner::new().unwrap_or_else(|_| panic!("runtime"));
    let services = Services::new(owner.handle(), source);
    let (handle, view) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| {
                let mut view = PlatformsView::new(services, window, cx);
                view.open(Platform::Claude, Surface::Settings, window, cx);
                view
            })
        })
        .unwrap_or_else(|_| panic!("window"))
    });
    cx.wait_for(handle, std::time::Duration::from_secs(10), |_, cx| {
        !view.read(cx).busy
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("platform-show-layers", cx);
    })
    .unwrap_or_else(|_| panic!("click"));
    cx.wait_for(handle, std::time::Duration::from_secs(10), |_, cx| {
        let view = view.read(cx);
        view.layers_open && !view.layers.read(cx).busy
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(window.try_find("platform-layers").is_some());
        let page = view.read(cx);
        let layers = &page.layers.read(cx).rows;
        assert_eq!(layers.len(), 4);
        assert!(!layers[0].editable);
        assert_eq!(layers[1].exists, None);
        assert!(
            layers[0]
                .path
                .as_ref()
                .is_some_and(|path| path.starts_with(root.path()))
        );
        assert!(window.try_find("platform-save").is_none());
    })
    .unwrap_or_else(|_| panic!("inspect"));
    owner.shutdown();
}

#[gpui_kit::test]
async fn codex_api_account_requires_confirmation_and_clears_plaintext_after_commit(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap_or_else(|_| panic!("fixture"));
    let source = crate::runtime::ConfigSource::Fixture {
        profiles: root.path().join("profiles.toml"),
        registry: root.path().join("registry.toml"),
        delay: std::time::Duration::ZERO,
    };
    let owner = RuntimeOwner::new().unwrap_or_else(|_| panic!("runtime"));
    let services = Services::new(owner.handle(), source);
    let (handle, view) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| {
                let mut view = PlatformsView::new(services, window, cx);
                view.platform = Platform::Codex;
                view.surface = Surface::Auth;
                view.load(cx);
                view
            })
        })
        .unwrap_or_else(|_| panic!("window"))
    });
    cx.wait_for(handle, std::time::Duration::from_secs(10), |_, cx| {
        view.read(cx).session.is_some()
    })
    .await;
    let path = cx.update(|cx| {
        view.read(cx)
            .session
            .as_ref()
            .unwrap_or_else(|| panic!("session"))
            .roots
            .codex_store
            .join("auth/ui-account.json")
    });
    cx.update_window(handle, |_, window, cx| {
        view.read(cx)
            .name
            .clone()
            .update(cx, |input, cx| input.set_value("ui-account", window, cx));
        view.read(cx).content.clone().update(cx, |input, cx| {
            input.set_value(r#"{"api_key":"ui-secret-sentinel"}"#, window, cx)
        });
        window.render_frame(cx);
        window.click("platform-account-api-key", cx);
        window.render_frame(cx);
        assert!(!path.exists());
        window.click("platform-decision-cancel", cx);
        assert!(!path.exists());
        assert!(
            view.read(cx)
                .content
                .read(cx)
                .value()
                .contains("ui-secret-sentinel")
        );
        window.render_frame(cx);
        window.click("platform-account-api-key", cx);
        window.render_frame(cx);
        window.click("platform-decision-confirm", cx);
    })
    .unwrap_or_else(|_| panic!("confirm"));
    cx.wait_for(handle, std::time::Duration::from_secs(10), |_, cx| {
        !view.read(cx).busy
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(path.exists());
        assert!(view.read(cx).content.read(cx).value().is_empty());
        assert!(view.read(cx).decision_content.is_empty());
        assert!(view.read(cx).session.is_none());
        window.remove_window();
    })
    .unwrap_or_else(|_| panic!("saved"));
    owner.shutdown();
}

#[gpui_kit::test]
fn oauth_browser_uses_the_shared_allowlist_without_opening_callbacks_or_unsafe_urls(
    cx: &mut TestAppContext,
) {
    for rejected in [
        "file:///tmp/fixture",
        "javascript:alert(1)",
        "https://auth.openai.com.invalid/oauth/authorize",
        "https://user@auth.openai.com/oauth/authorize",
        "http://localhost:1455/auth/callback",
    ] {
        cx.update(|cx| assert!(!open_authorization_url(rejected, cx)));
        assert!(cx.opened_url().is_none());
    }
    let authorize = "https://auth.openai.com/oauth/authorize?state=isolated-fixture";
    cx.update(|cx| assert!(open_authorization_url(authorize, cx)));
    assert_eq!(cx.opened_url().as_deref(), Some(authorize));
}

struct DecisionHost {
    view: Entity<PlatformsView>,
    outside: FocusHandle,
    _subscription: gpui_kit::Subscription,
}
impl Render for DecisionHost {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .child(
                div()
                    .id("platform-outside-focus")
                    .test_support()
                    .track_focus(&self.outside)
                    .tab_stop(true)
                    .child("Outside platform decision"),
            )
            .child(self.view.clone())
    }
}

#[gpui_kit::test]
fn confirmation_enters_after_callback_before_draw(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap_or_else(|_| panic!("fixture"));
    let roots = PlatformRoots::fixture(root.path()).unwrap_or_else(|_| panic!("roots"));
    let document = Document::new(
        root.path().join("settings.json"),
        None,
        BackupPolicy::None,
        Some(root.path().join("locks")),
        Format::Json,
    );
    let snapshot = document.read().unwrap_or_else(|_| panic!("snapshot"));
    let session = Arc::new(PlatformSession {
        roots,
        document,
        snapshot,
        accounts: Vec::new(),
        signed_in: false,
        provider_ids: Vec::new(),
        provider_report: crate::runtime::platforms::ProviderReport::Unsupported,
        surface: Surface::Settings,
        platform: Platform::Claude,
        file_state: crate::runtime::platforms::FileState::Present,
        file_label: "settings.json",
        fields: Vec::new(),
        credential: crate::runtime::platforms::CredentialState::NotDetected,
        runtime_auth_present: None,
        auth_revision: Default::default(),
    });
    let owner = RuntimeOwner::new().unwrap_or_else(|_| panic!("runtime"));
    let services = Services::new(
        owner.handle(),
        crate::runtime::ConfigSource::Fixture {
            profiles: root.path().join("profiles.toml"),
            registry: root.path().join("registry.toml"),
            delay: std::time::Duration::ZERO,
        },
    );
    let (handle, host) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| {
                let view = cx.new(|cx| {
                    let mut view = PlatformsView::new(services, window, cx);
                    view.session = Some(session);
                    view
                });
                let subscription = cx.observe(&view, |_, _, cx| cx.notify());
                DecisionHost {
                    view,
                    outside: cx.focus_handle(),
                    _subscription: subscription,
                }
            })
        })
        .unwrap_or_else(|_| panic!("window"))
    });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        let outside = host.read(cx).outside.clone();
        window.focus(&outside, cx);
        window.render_frame(cx);
        let position = window.find("platform-source-open").bounds().center();
        window.dispatch_event(
            MouseMoveEvent {
                position,
                pressed_button: None,
                modifiers: Default::default(),
            }
            .to_platform_input(),
            cx,
        );
        window.render_frame(cx);
        window.dispatch_event(
            MouseDownEvent {
                button: MouseButton::Left,
                position,
                modifiers: Default::default(),
                click_count: 1,
                first_mouse: false,
            }
            .to_platform_input(),
            cx,
        );
        window.render_frame(cx);
        window.dispatch_event(
            MouseUpEvent {
                button: MouseButton::Left,
                position,
                modifiers: Default::default(),
                click_count: 1,
            }
            .to_platform_input(),
            cx,
        );
        assert!(host.read(cx).view.read(cx).decision.is_some());
        // Locked GPUI drains callbacks before drawing the new decision subtree.
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        window.focus(&outside, cx);
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        assert_eq!(
            window.find("platform-decision-cancel").focused(),
            Some(true),
            "initial decision entry must survive callback-before-draw and outside focus"
        );
        window.press("tab", cx);
        assert_eq!(
            window.find("platform-decision-confirm").focused(),
            Some(true)
        );
        window.press("shift-tab", cx);
        assert_eq!(
            window.find("platform-decision-cancel").focused(),
            Some(true)
        );
        window.press("escape", cx);
        assert!(host.read(cx).view.read(cx).decision.is_none());
        assert_eq!(window.find("platform-outside-focus").focused(), Some(true));
        window.remove_window();
    })
    .unwrap_or_else(|_| panic!("decision boundary"));
    owner.shutdown();
}

#[gpui_kit::test]
async fn oauth_cancel_joins_fixture_cleanup_and_clears_callback_draft(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap_or_else(|_| panic!("fixture"));
    let roots = PlatformRoots::fixture(root.path()).unwrap_or_else(|_| panic!("roots"));
    let mut pending = ccr_cli::application::codex_oauth::new_oauth_pending()
        .unwrap_or_else(|_| panic!("pending"));
    let port = std::net::TcpListener::bind(("127.0.0.1", 0))
        .unwrap_or_else(|_| panic!("fixture port"))
        .local_addr()
        .unwrap_or_else(|_| panic!("fixture address"))
        .port();
    pending.port = port;
    pending.redirect_uri = format!("http://localhost:{port}/auth/callback");
    pending.auth_url = "Synthetic fixture authorization is disabled".into();
    let pending_path = roots.ccr.join("platforms/codex/oauth_pending.json");
    let store = ccr_codex::services::CodexOAuthPendingStore::with_path(pending_path.clone());
    ccr_core::core::LockManager::with_scoped_directory(&roots.locks, || store.save(&pending))
        .unwrap_or_else(|_| panic!("fixture pending store"));
    let document = Document::new(
        roots.codex_runtime.join("auth.json"),
        None,
        BackupPolicy::None,
        Some(roots.locks.clone()),
        Format::Json,
    );
    let snapshot = document.read().unwrap_or_else(|_| panic!("snapshot"));
    let session = Arc::new(PlatformSession {
        roots,
        document,
        snapshot,
        accounts: Vec::new(),
        signed_in: false,
        provider_ids: Vec::new(),
        provider_report: crate::runtime::platforms::ProviderReport::Unsupported,
        surface: Surface::Auth,
        platform: Platform::Codex,
        file_state: crate::runtime::platforms::FileState::Present,
        file_label: "config.toml",
        fields: Vec::new(),
        credential: crate::runtime::platforms::CredentialState::NotDetected,
        runtime_auth_present: None,
        auth_revision: Default::default(),
    });
    let owner = RuntimeOwner::new().unwrap_or_else(|_| panic!("runtime"));
    let services = Services::new(
        owner.handle(),
        crate::runtime::ConfigSource::Fixture {
            profiles: root.path().join("profiles.toml"),
            registry: root.path().join("registry.toml"),
            delay: std::time::Duration::ZERO,
        },
    );
    let (handle, view) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| {
                let mut view = PlatformsView::new(services, window, cx);
                view.platform = Platform::Codex;
                view.surface = Surface::Auth;
                view.session = Some(session);
                view
            })
        })
        .unwrap_or_else(|_| panic!("window"))
    });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("platform-oauth-start", cx);
    })
    .unwrap_or_else(|_| panic!("start OAuth"));
    cx.wait_for(handle, std::time::Duration::from_secs(5), |_, cx| {
        !view.read(cx).diagnostic_blocked()
    })
    .await;
    cx.update_window(handle, |_, _, cx| {
        assert!(view.read(cx).oauth_status.is_none());
        assert!(!view.read(cx).services.platform_oauth_active());
        // Host preflight refuses isolated sources. Exercise the retained owner's
        // cancellation separately using the captured, preseeded ephemeral port.
        view.update(cx, |this, cx| this.start_oauth(cx));
    })
    .unwrap_or_else(|_| panic!("start captured fixture owner"));
    cx.wait_for(handle, std::time::Duration::from_secs(5), |_, cx| {
        view.read(cx)
            .oauth_status
            .as_ref()
            .is_some_and(|status| status.phase == NativePhase::Waiting)
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        view.read(cx).content.clone().update(cx, |state, cx| {
            state.set_value("invalid-callback-secret-sentinel", window, cx);
        });
        window.click("platform-oauth-cancel", cx);
    })
    .unwrap_or_else(|_| panic!("cancel OAuth"));
    cx.wait_for(handle, std::time::Duration::from_secs(5), |_, cx| {
        view.read(cx)
            .oauth_status
            .as_ref()
            .is_some_and(|status| status.phase == NativePhase::Cancelled)
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(view.read(cx).content.read(cx).value().is_empty());
        assert!(view.read(cx).oauth_url.read(cx).value().is_empty());
        assert!(!view.read(cx).services.platform_oauth_active());
        assert!(!pending_path.exists());
        assert!(std::net::TcpListener::bind(("127.0.0.1", port)).is_ok());
        assert!(
            view.read(cx)
                .message
                .as_deref()
                .is_some_and(|message| message.contains("cancelled after"))
        );
        window.remove_window();
    })
    .unwrap_or_else(|_| panic!("terminal cleanup"));
    owner.shutdown();
}

#[gpui_kit::test]
fn source_requires_acknowledgement_and_close_clears_secret_and_response_identity(
    cx: &mut TestAppContext,
) {
    let root = tempfile::tempdir().unwrap_or_else(|_| panic!("fixture"));
    let roots = PlatformRoots::fixture(root.path()).unwrap_or_else(|_| panic!("roots"));
    let document = Document::new(
        root.path().join("settings.json"),
        None,
        BackupPolicy::None,
        Some(root.path().join("locks")),
        Format::Json,
    );
    let secret = "native-platform-secret-sentinel";
    std::fs::write(
        document.path(),
        format!(r#"{{"env":{{"TOKEN":"{secret}"}}}}"#),
    )
    .unwrap_or_else(|_| panic!("fixture document"));
    let snapshot = document.read().unwrap_or_else(|_| panic!("snapshot"));
    let session = Arc::new(PlatformSession {
        roots,
        document,
        snapshot,
        accounts: Vec::new(),
        signed_in: false,
        provider_ids: Vec::new(),
        provider_report: crate::runtime::platforms::ProviderReport::Unsupported,
        surface: Surface::Settings,
        platform: Platform::Claude,
        file_state: crate::runtime::platforms::FileState::Present,
        file_label: "settings.json",
        fields: Vec::new(),
        credential: crate::runtime::platforms::CredentialState::NotDetected,
        runtime_auth_present: None,
        auth_revision: Default::default(),
    });
    let owner = RuntimeOwner::new().unwrap_or_else(|_| panic!("runtime"));
    let services = Services::new(
        owner.handle(),
        crate::runtime::ConfigSource::Fixture {
            profiles: root.path().join("profiles.toml"),
            registry: root.path().join("registry.toml"),
            delay: std::time::Duration::ZERO,
        },
    );
    let (handle, view) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| {
                let mut view = PlatformsView::new(services, window, cx);
                view.session = Some(session);
                view
            })
        })
        .unwrap_or_else(|_| panic!("window"))
    });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(!view.read(cx).content.read(cx).value().contains(secret));
        window.click("platform-source-open", cx);
        assert!(!view.read(cx).source_mode);
        window.render_frame(cx);
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        assert_eq!(
            window.find("platform-decision-cancel").focused(),
            Some(true)
        );
        window.click("platform-decision-confirm", cx);
        window.render_frame(cx);
        assert!(view.read(cx).content.read(cx).value().contains(secret));
        let key = RequestKey {
            environment: EnvironmentId::Local("fixture".into()),
            platform: Platform::Claude,
            generation: 99,
            request_id: 99,
        };
        view.update(cx, |view, _| view.gate.replace(key.clone()));
        window.click("platform-clear", cx);
        window.render_frame(cx);
        let view = view.read(cx);
        assert!(view.session.is_none());
        assert!(view.content.read(cx).value().is_empty());
        assert!(!view.gate.accepts(&key));
        assert!(!view.source_mode);
    })
    .unwrap_or_else(|_| panic!("interaction"));
    owner.shutdown();
}

#[gpui_kit::test]
async fn opencode_theme_catalog_updates_real_draft_and_preserves_other_fields(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap_or_else(|_| panic!("fixture"));
    let roots = PlatformRoots::fixture(root.path()).unwrap_or_else(|_| panic!("roots"));
    let path = roots.home.join(".config/opencode/tui.json");
    std::fs::create_dir_all(path.parent().unwrap_or_else(|| panic!("parent")))
        .unwrap_or_else(|_| panic!("mkdir"));
    std::fs::write(&path, r#"{"theme":"dark","unknown":true}"#).unwrap_or_else(|_| panic!("write"));
    let document = Document::new(
        path.clone(),
        None,
        BackupPolicy::None,
        Some(root.path().join("locks")),
        Format::Json,
    );
    let snapshot = document.read().unwrap_or_else(|_| panic!("read"));
    let session = Arc::new(PlatformSession {
        roots,
        document,
        snapshot,
        accounts: Vec::new(),
        signed_in: false,
        provider_ids: Vec::new(),
        provider_report: ProviderReport::Unsupported,
        surface: Surface::Tui,
        platform: Platform::OpenCode,
        file_state: FileState::Present,
        file_label: "tui.json",
        fields: Vec::new(),
        credential: CredentialState::NotDetected,
        runtime_auth_present: None,
        auth_revision: Default::default(),
    });
    let owner = RuntimeOwner::new().unwrap_or_else(|_| panic!("runtime"));
    let services = Services::new(
        owner.handle(),
        crate::runtime::ConfigSource::Fixture {
            profiles: root.path().join("profiles.toml"),
            registry: root.path().join("registry.toml"),
            delay: std::time::Duration::ZERO,
        },
    );
    let (handle, view) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| {
                let mut view = PlatformsView::new(services, window, cx);
                view.platform = Platform::OpenCode;
                view.surface = Surface::Tui;
                view.session = Some(session);
                view.content.update(cx, |input, cx| {
                    input.set_value(r#"{"mouse":false}"#, window, cx)
                });
                view
            })
        })
        .unwrap_or_else(|_| panic!("window"))
    });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click(("opencode-theme", 5usize), cx);
        let patch: serde_json::Value =
            serde_json::from_str(&view.read(cx).content.read(cx).value())
                .unwrap_or_else(|_| panic!("patch"));
        assert_eq!(patch["theme"], "dracula");
        assert_eq!(patch["mouse"], false);
        view.update(cx, |v, cx| v.save(cx));
    })
    .unwrap_or_else(|_| panic!("select"));
    cx.wait_for(handle, std::time::Duration::from_secs(10), |_, cx| {
        !view.read(cx).busy
    })
    .await;
    let saved: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("saved")))
            .unwrap_or_else(|_| panic!("json"));
    assert_eq!(saved["theme"], "dracula");
    assert_eq!(saved["mouse"], false);
    assert_eq!(saved["unknown"], true);
    cx.update_window(handle, |_, window, _| window.remove_window())
        .unwrap_or_else(|_| panic!("close"));
    owner.shutdown();
}

#[gpui_kit::test]
fn embedded_opencode_keeps_a_reachable_tui_document_entry(cx: &mut TestAppContext) {
    let root = tempfile::tempdir().unwrap_or_else(|_| panic!("fixture"));
    let roots = PlatformRoots::fixture(root.path()).unwrap_or_else(|_| panic!("roots"));
    let path = roots.home.join(".config/opencode/tui.json");
    std::fs::create_dir_all(path.parent().unwrap_or_else(|| panic!("parent")))
        .unwrap_or_else(|_| panic!("mkdir"));
    std::fs::write(&path, r#"{"theme":"dark"}"#).unwrap_or_else(|_| panic!("write"));
    let document = Document::new(
        path.clone(),
        None,
        BackupPolicy::None,
        Some(root.path().join("locks")),
        Format::Json,
    );
    let snapshot = document.read().unwrap_or_else(|_| panic!("read"));
    let session = Arc::new(PlatformSession {
        roots,
        document,
        snapshot,
        accounts: Vec::new(),
        signed_in: false,
        provider_ids: Vec::new(),
        provider_report: ProviderReport::Unsupported,
        surface: Surface::Settings,
        platform: Platform::OpenCode,
        file_state: FileState::Present,
        file_label: "settings.json",
        fields: Vec::new(),
        credential: CredentialState::NotDetected,
        runtime_auth_present: None,
        auth_revision: Default::default(),
    });
    let owner = RuntimeOwner::new().unwrap_or_else(|_| panic!("runtime"));
    let services = Services::new(
        owner.handle(),
        crate::runtime::ConfigSource::Fixture {
            profiles: root.path().join("profiles.toml"),
            registry: root.path().join("registry.toml"),
            delay: std::time::Duration::ZERO,
        },
    );
    let (handle, view) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| {
                let mut view = PlatformsView::new(services, window, cx);
                // 外壳总是以 pinned 打开平台目的地，这正是隐藏 surfaces 行的条件。
                view.open(Platform::OpenCode, Surface::Settings, window, cx);
                view.session = Some(session);
                // 这里只验证布局与入口可达；正在进行的读取请求会让按钮保持禁用。
                view.busy = false;
                view
            })
        })
        .unwrap_or_else(|_| panic!("window"))
    });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        // tui.json 没有对应的平台 tab，嵌入后必须仍然可达。
        assert!(
            window.try_find("platform-tui").is_some(),
            "embedded OpenCode settings must keep the TUI document entry"
        );
        window.click("platform-tui", cx);
        assert!(matches!(view.read(cx).surface, Surface::Tui));
    })
    .unwrap_or_else(|_| panic!("embedded tui entry"));
    cx.update_window(handle, |_, window, _| window.remove_window())
        .unwrap_or_else(|_| panic!("close"));
    owner.shutdown();
}
