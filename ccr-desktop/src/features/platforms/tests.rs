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
        // 配置来源是页内展开区：设置表单与保存仍在原位可用，展开区排在表单之后。
        assert!(window.try_find("platform-save").is_some());
        let form = window.find("platform-common-form").bounds();
        let sources = window.find("platform-layers").bounds();
        assert!(
            sources.top() >= form.bottom(),
            "configuration sources expand after the settings form"
        );
    })
    .unwrap_or_else(|_| panic!("inspect"));
    let sources_entity = cx.update(|cx| view.read(cx).layers.entity_id());
    cx.update_window(handle, |_, window, cx| {
        // 按焦点顺序走到开关（裸视图没有外壳的 Tab 绑定）：Enter 收起后焦点仍在开关，展开区移除。
        for _ in 0..80 {
            if window.find("platform-show-layers").focused() == Some(true) {
                break;
            }
            window.focus_next(cx);
            window.render_frame(cx);
        }
        assert_eq!(window.find("platform-show-layers").focused(), Some(true));
        window.press("enter", cx);
        window.render_frame(cx);
        assert!(!view.read(cx).layers_open);
        assert!(window.try_find("platform-layers").is_none());
        assert!(window.try_find("platform-save").is_some());
        assert_eq!(window.find("platform-show-layers").focused(), Some(true));
        // 再次展开：焦点留在开关，表单仍在。
        window.press("enter", cx);
        window.render_frame(cx);
        assert!(view.read(cx).layers_open);
        assert!(window.try_find("platform-layers").is_some());
        assert!(window.try_find("platform-common-form").is_some());
        assert_eq!(window.find("platform-show-layers").focused(), Some(true));
        // 焦点进入展开区后用鼠标收起：焦点回到开关，不落在已移除的输入框上。
        window.click("platform-layers-project", cx);
        window.render_frame(cx);
        assert_eq!(window.find("platform-layers-project").focused(), Some(true));
        window.click("platform-show-layers", cx);
        window.render_frame(cx);
        assert!(!view.read(cx).layers_open);
        assert!(window.try_find("platform-layers").is_none());
        assert_eq!(window.find("platform-show-layers").focused(), Some(true));
    })
    .unwrap_or_else(|_| panic!("toggle focus"));
    // 展开与收起复用同一个来源视图，不重建焦点所有者。
    assert_eq!(
        cx.update(|cx| view.read(cx).layers.entity_id()),
        sources_entity
    );
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
async fn account_actions_follow_the_selected_row_and_saving_ignores_selection(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap_or_else(|_| panic!("fixture"));
    let roots = PlatformRoots::fixture(&root.path().join("desktop-platform-fixture"))
        .unwrap_or_else(|_| panic!("roots"));
    // 夹具账户只写入隔离根下的 Codex 账户库，锁也限定在夹具目录。
    let service = roots.codex_auth();
    ccr_core::core::LockManager::with_scoped_directory(&roots.locks, || {
        for (name, key) in [
            ("first", "synthetic-first-account-key"),
            ("second", "synthetic-second-account-key"),
        ] {
            let auth = ccr_codex::CodexAuthJson {
                openai_api_key: Some(key.into()),
                tokens: None,
                last_refresh: None,
            };
            let export = service
                .build_export_account_from_auth_json(auth, None, None, None, None)
                .unwrap_or_else(|_| panic!("fixture export"));
            ccr_cli::application::codex_oauth::commit_account(
                &service,
                export,
                Some(name.into()),
                false,
            )
            .unwrap_or_else(|_| panic!("fixture account"));
        }
        Ok::<(), ccr_core::CcrError>(())
    })
    .unwrap_or_else(|_| panic!("fixture accounts"));
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
                view.open(Platform::Codex, Surface::Auth, window, cx);
                view
            })
        })
        .unwrap_or_else(|_| panic!("window"))
    });
    cx.wait_for(handle, std::time::Duration::from_secs(10), |_, cx| {
        let view = view.read(cx);
        !view.busy && view.session.is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(window.try_find("platform-accounts-current").is_some());
        assert!(window.try_find("platform-accounts-saved").is_some());
        // 账户 tab 不显示设置 tab 的配置来源开关。
        assert!(window.try_find("platform-show-layers").is_none());
        // 没有选中项：针对所选账户的动作禁用（无焦点绑定、点击无效），保存当前登录仍可用。
        for id in [
            "platform-account-switch",
            "platform-account-rename",
            "platform-account-delete",
        ] {
            assert_eq!(window.find(id).focused(), None, "{id} must be disabled");
        }
        assert_eq!(window.find("platform-account-save").focused(), Some(false));
        window.click("platform-account-delete", cx);
        assert!(view.read(cx).decision.is_none());
        assert_eq!(
            window.find("platform-account-first").selected(),
            Some(false)
        );
        assert!(
            window
                .find("platform-accounts-selection")
                .label()
                .is_some_and(|label| label.contains("No saved account selected"))
        );
        // 新账户名输入写成另一个已保存账户的名字，也不能决定操作对象。
        view.read(cx)
            .name
            .clone()
            .update(cx, |input, cx| input.set_value("first", window, cx));
        window.click("platform-account-first", cx);
        window.render_frame(cx);
        assert_eq!(window.find("platform-account-first").selected(), Some(true));
        assert_eq!(
            window.find("platform-account-delete").focused(),
            Some(false)
        );
        window.click("platform-account-delete", cx);
        assert_eq!(view.read(cx).decision_name, "first");
        window.render_frame(cx);
        window.click("platform-decision-cancel", cx);
        window.render_frame(cx);
        // 改选 second：同一按钮的目标随选中项改变。
        window.click("platform-account-second", cx);
        window.render_frame(cx);
        assert_eq!(
            window.find("platform-account-first").selected(),
            Some(false)
        );
        assert_eq!(
            window.find("platform-account-second").selected(),
            Some(true)
        );
        window.click("platform-account-rename", cx);
        assert_eq!(view.read(cx).decision_name, "second");
        window.render_frame(cx);
        window.click("platform-decision-cancel", cx);
        window.render_frame(cx);
        window.click("platform-account-delete", cx);
        assert_eq!(view.read(cx).decision_name, "second");
        window.render_frame(cx);
        window.click("platform-decision-cancel", cx);
        window.render_frame(cx);
        window.click("platform-account-switch", cx);
    })
    .unwrap_or_else(|_| panic!("select"));
    cx.wait_for(handle, std::time::Duration::from_secs(10), |_, cx| {
        !view.read(cx).busy
    })
    .await;
    let runtime = std::fs::read_to_string(roots.codex_runtime.join("auth.json"))
        .unwrap_or_else(|_| panic!("switched runtime"));
    assert!(runtime.contains("synthetic-second-account-key"));
    assert!(!runtime.contains("synthetic-first-account-key"));
    // 提交后账户集合可能变化，旧选中项被清除，重读后不自动选第一行。
    assert!(cx.update(|cx| view.read(cx).selected_account.is_none()));
    cx.update(|cx| view.update(cx, |view, cx| view.load(cx)));
    cx.wait_for(handle, std::time::Duration::from_secs(10), |_, cx| {
        let view = view.read(cx);
        !view.busy && view.session.is_some()
    })
    .await;
    let first = roots.codex_store.join("auth/first.json");
    let first_before = std::fs::read(&first).unwrap_or_else(|_| panic!("first account"));
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(
            window.find("platform-account-second").selected(),
            Some(false)
        );
        // 选中 first 后保存当前登录：目标是新账户名，而不是选中项。
        window.click("platform-account-first", cx);
        view.read(cx)
            .name
            .clone()
            .update(cx, |input, cx| input.set_value("third", window, cx));
        window.render_frame(cx);
        window.click("platform-account-save", cx);
        // 进程警告只是提示，不必等待隔离诊断结束即可确认保存。
        window.render_frame(cx);
        window.click("codex-diagnostic-confirm", cx);
    })
    .unwrap_or_else(|_| panic!("save current"));
    cx.wait_for(handle, std::time::Duration::from_secs(10), |_, cx| {
        !view.read(cx).busy
    })
    .await;
    let third = std::fs::read_to_string(roots.codex_store.join("auth/third.json"))
        .unwrap_or_else(|_| panic!("saved current login"));
    assert!(third.contains("synthetic-second-account-key"));
    assert_eq!(
        std::fs::read(&first).unwrap_or_else(|_| panic!("first account")),
        first_before
    );
    cx.update_window(handle, |_, window, _| window.remove_window())
        .unwrap_or_else(|_| panic!("close"));
    owner.shutdown();
}

fn scope_session(
    root: &std::path::Path,
    platform: Platform,
    surface: Surface,
    file_label: &'static str,
    provider_report: ProviderReport,
) -> Arc<PlatformSession> {
    let roots = PlatformRoots::fixture(root).unwrap_or_else(|_| panic!("roots"));
    let document = Document::new(
        root.join("settings.json"),
        None,
        BackupPolicy::None,
        Some(root.join("locks")),
        Format::Json,
    );
    let snapshot = document.read().unwrap_or_else(|_| panic!("snapshot"));
    Arc::new(PlatformSession {
        roots,
        document,
        snapshot,
        accounts: Vec::new(),
        signed_in: false,
        provider_ids: vec!["fixture-provider".into()],
        provider_report,
        surface,
        platform,
        file_state: FileState::Present,
        file_label,
        // 一个运行设置字段和一个账户来源字段：用来确认每个 tab 只显示自己那部分。
        fields: vec![
            crate::runtime::platforms::FieldFact {
                key: "model",
                value: "scope-fixture-model".into(),
            },
            crate::runtime::platforms::FieldFact {
                key: "oauth_source",
                value: "scope-fixture-source".into(),
            },
        ],
        credential: CredentialState::PlatformUnsupported,
        runtime_auth_present: None,
        auth_revision: Default::default(),
    })
}

#[gpui_kit::test]
fn settings_scope_groups_follow_each_platform_surface(cx: &mut TestAppContext) {
    let root = tempfile::tempdir().unwrap_or_else(|_| panic!("fixture"));
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
            cx.new(|cx| PlatformsView::new(services, window, cx))
        })
        .unwrap_or_else(|_| panic!("window"))
    });
    let ready = || ProviderReport::Ready { hidden: 0 };
    // (平台, 页面, 文件, 供应商状态, [运行设置, 供应商目录, CCR 元数据], 配置来源开关)
    let cases = [
        (
            Platform::Claude,
            Surface::Settings,
            "settings.json",
            ProviderReport::Unsupported,
            [true, false, false],
            true,
        ),
        (
            Platform::Codex,
            Surface::Settings,
            "config.toml",
            ProviderReport::Unsupported,
            [true, false, false],
            true,
        ),
        (
            Platform::Grok,
            Surface::Settings,
            "config.toml",
            ProviderReport::Unsupported,
            [true, false, false],
            true,
        ),
        (
            Platform::OpenCode,
            Surface::Settings,
            "opencode.json",
            ready(),
            [true, true, false],
            false,
        ),
        (
            Platform::OpenCode,
            Surface::Tui,
            "tui.json",
            ProviderReport::Unsupported,
            [true, false, false],
            false,
        ),
        (
            Platform::Gemini,
            Surface::LegacySettings,
            "legacy settings.json",
            ProviderReport::Unsupported,
            [true, false, false],
            false,
        ),
        (
            Platform::Codex,
            Surface::Providers,
            "model_providers.json",
            ready(),
            [false, true, false],
            false,
        ),
        (
            Platform::OpenCode,
            Surface::Providers,
            "opencode.json",
            ready(),
            [false, true, false],
            false,
        ),
        (
            Platform::Codex,
            Surface::Models,
            "custom-models.toml",
            ProviderReport::Unsupported,
            [false, false, true],
            false,
        ),
        (
            Platform::Claude,
            Surface::Auth,
            "settings.json",
            ProviderReport::Unsupported,
            [false, false, false],
            false,
        ),
        (
            Platform::Grok,
            Surface::Auth,
            "config.toml",
            ProviderReport::Unsupported,
            [false, false, false],
            false,
        ),
    ];
    for (platform, surface, file, report, groups, layers) in cases {
        let session = scope_session(root.path(), platform, surface, file, report);
        cx.update_window(handle, |_, window, cx| {
            view.update(cx, |view, cx| {
                view.pinned = true;
                view.platform = platform;
                view.surface = surface;
                view.session = Some(session);
                cx.notify();
            });
            window.render_frame(cx);
            let case = format!("{}:{file}", platform.short_name());
            for (id, expected) in [
                "platform-group-runtime",
                "platform-group-provider-catalog",
                "platform-group-ccr-metadata",
            ]
            .into_iter()
            .zip(groups)
            {
                assert_eq!(window.try_find(id).is_some(), expected, "{case} {id}");
            }
            assert_eq!(
                window.try_find("platform-show-layers").is_some(),
                layers,
                "{case} configuration sources"
            );
            let auth = surface == Surface::Auth;
            assert_eq!(window.try_find("platform-accounts-current").is_some(), auth);
            assert_eq!(window.try_find("platform-accounts-saved").is_some(), auth);
            // 选中哪个 tab 只显示那一项：账户 tab 只有凭据观测与账户来源，设置类 tab 只有设置文件。
            assert_eq!(
                window.try_find("platform-auth-status").is_some(),
                auth,
                "{case} credential observation"
            );
            assert_eq!(
                window.try_find("platform-file-status").is_some(),
                !auth,
                "{case} settings file status"
            );
            assert!(
                window.try_find("platform-field-model").is_none(),
                "{case} runtime settings fields stay off this projection"
            );
            assert_eq!(
                window.try_find("platform-oauth-source-1").is_some(),
                auth,
                "{case} account source"
            );
            let labels: Vec<String> = gpui_kit::base::test_support::snapshots(window)
                .iter()
                .filter_map(|node| node.label().map(str::to_owned))
                .collect();
            assert_eq!(
                labels
                    .iter()
                    .any(|label| label.starts_with("Accounts are not supported")),
                auth,
                "{case} no placeholder for unsupported accounts outside the accounts tab"
            );
            // 模型 tab 的字段是 CCR 模型清单本身，不属于这条约束。
            assert!(
                surface == Surface::Models
                    || !labels
                        .iter()
                        .any(|label| label.contains("scope-fixture-model")),
                "{case} runtime field value must not reach this projection"
            );
            if groups[1] {
                // 供应商条目归入供应商目录分组，不并入运行设置。
                assert!(
                    window.try_find("platform-provider-status").is_some(),
                    "{case}"
                );
            }
            if platform == Platform::Gemini {
                assert!(
                    window
                        .find("platform-group-runtime-scope")
                        .label()
                        .is_some_and(|label| label.contains("unverified")),
                    "Gemini-compatible files must not claim client recognition"
                );
            }
        })
        .unwrap_or_else(|_| panic!("scope case"));
    }
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
