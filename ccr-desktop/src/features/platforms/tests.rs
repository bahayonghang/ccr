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
                .is_some_and(|path| matches!(path, ccr_app::environments::io::EnvironmentPath::Local(path) if path.starts_with(root.path())))
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
        window.render_frame(cx);
        for id in [
            "platform-tools-login",
            "platform-tools-add",
            "platform-tools-quota",
            "platform-tools-diagnostics",
        ] {
            assert!(
                window.try_find(id).is_some(),
                "missing account workspace {id}"
            );
        }
        assert!(window.try_find("platform-account-content").is_none());
        window.click("platform-tools-add", cx);
        window.render_frame(cx);
        assert!(window.try_find("platform-oauth-start").is_none());
        assert!(window.try_find("platform-quota-current").is_none());
        window.render_frame(cx);
        view.read(cx)
            .tools
            .name
            .clone()
            .update(cx, |input, cx| input.set_value("ui-account", window, cx));
        view.read(cx).tools.api.clone().update(cx, |input, cx| {
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
                .tools
                .api
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
        assert!(view.read(cx).tools.api.read(cx).value().is_empty());
        assert!(view.read(cx).decision_content.is_empty());
        assert!(
            view.read(cx)
                .session
                .as_ref()
                .is_some_and(|s| s.accounts.iter().any(|a| a.name == "ui-account"))
        );
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
        remote: None,
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
    let roots = PlatformRoots::fixture(&root.path().join("desktop-platform-fixture"))
        .unwrap_or_else(|_| panic!("roots"));
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
    let owner = RuntimeOwner::new().unwrap_or_else(|_| panic!("runtime"));
    let mut services = Services::new(
        owner.handle(),
        crate::runtime::ConfigSource::Fixture {
            profiles: root.path().join("profiles.toml"),
            registry: root.path().join("registry.toml"),
            delay: std::time::Duration::ZERO,
        },
    );
    let fixture = Arc::new(crate::runtime::platforms::AccountFixture::default());
    services.account_fixture = Some(fixture.clone());
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
    use std::sync::atomic::Ordering::Relaxed;
    cx.wait_for(handle, std::time::Duration::from_secs(5), |_, cx| {
        !view.read(cx).busy
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("platform-tools-login", cx);
        window.render_frame(cx);
        assert_eq!(fixture.preflights.load(Relaxed), 0);
        assert_eq!(fixture.starts.load(Relaxed), 0);
        window.click("platform-oauth-start", cx);
        window.click("platform-oauth-start", cx);
    })
    .unwrap_or_else(|_| panic!("start from accounts"));
    cx.wait_for(handle, std::time::Duration::from_secs(5), |_, cx| {
        view.read(cx)
            .oauth_status
            .as_ref()
            .is_some_and(|status| status.phase == NativePhase::Waiting)
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        view.read(cx)
            .tools
            .callback
            .clone()
            .update(cx, |state, cx| {
                state.set_value("invalid-callback-secret-sentinel", window, cx);
            });
        window.click("platform-oauth-cancel", cx);
        assert!(view.read(cx).tools.cancelling);
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
        assert!(view.read(cx).tools.callback.read(cx).value().is_empty());
        assert!(
            view.read(cx)
                .oauth_status
                .as_ref()
                .is_none_or(|s| s.authorization_url.expose().is_empty())
        );
        assert!(!view.read(cx).services.platform_oauth_active());
        assert!(!pending_path.exists());
        assert!(std::net::TcpListener::bind(("127.0.0.1", port)).is_ok());
        assert!(
            view.read(cx)
                .tools
                .login_feedback
                .as_deref()
                .is_some_and(|message| message.contains("cancelled after"))
        );
        assert_eq!(fixture.preflights.load(Relaxed), 1);
        assert_eq!(fixture.starts.load(Relaxed), 1);
        assert_eq!(fixture.callbacks.load(Relaxed), 0);
        assert_eq!(fixture.commits.load(Relaxed), 0);
        ccr_core::core::LockManager::with_scoped_directory(&roots.locks, || store.save(&pending))
            .unwrap_or_else(|_| panic!("fixture pending"));
        window.click("platform-oauth-start", cx);
    })
    .unwrap_or_else(|_| panic!("second explicit login"));
    cx.wait_for(handle, std::time::Duration::from_secs(5), |_, cx| {
        view.read(cx)
            .oauth_status
            .as_ref()
            .is_some_and(|s| s.phase == NativePhase::Waiting)
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("platform-tools-back", cx);
        window.render_frame(cx);
        assert!(
            window
                .find("platform-tools-login")
                .label()
                .is_some_and(|s| s.contains("progress") || s.contains("进行中"))
        );
        window.click("platform-tools-login", cx);
        window.render_frame(cx);
        assert_eq!(fixture.starts.load(Relaxed), 2);
        let url = format!(
            "{}?code=synthetic-callback-sentinel&state={}",
            pending.redirect_uri,
            pending.state.expose()
        );
        view.read(cx)
            .tools
            .callback
            .clone()
            .update(cx, |i, cx| i.set_value(url, window, cx));
        window.render_frame(cx);
        assert_secret_hidden(window, "synthetic-callback-sentinel");
        window.click("platform-oauth-callback", cx);
    })
    .unwrap_or_else(|_| panic!("manual callback"));
    cx.wait_for(handle, std::time::Duration::from_secs(5), |_, cx| {
        view.read(cx).oauth_control_phase.is_none()
            && view
                .read(cx)
                .oauth_status
                .as_ref()
                .is_some_and(|s| s.phase == NativePhase::Callback)
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(view.read(cx).tools.callback.read(cx).value().is_empty());
        assert!(window.try_find("platform-oauth-callback").is_none());
        assert!(window.try_find("platform-oauth-complete").is_some());
        assert_eq!(fixture.callbacks.load(Relaxed), 1);
        assert_eq!(fixture.commits.load(Relaxed), 0);
        window.click("platform-oauth-complete", cx);
    })
    .unwrap_or_else(|_| panic!("explicit commit"));
    cx.wait_for(handle, std::time::Duration::from_secs(5), |_, cx| {
        !view.read(cx).services.platform_oauth_active()
            && view.read(cx).oauth_control_phase.is_none()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(fixture.commits.load(Relaxed), 1);
        assert!(
            view.read(cx)
                .oauth_status
                .as_ref()
                .is_some_and(|s| s.phase == NativePhase::Failed)
        );
        assert!(!roots.codex_runtime.join("auth.json").exists());
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
        remote: None,
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
        remote: None,
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
    std::fs::create_dir_all(&roots.codex_runtime)
        .unwrap_or_else(|_| panic!("create runtime fixture directory"));
    let running = roots.codex_runtime.join("auth.json");
    let first_runtime = br#"{"OPENAI_API_KEY":"synthetic-first-account-key"}"#;
    std::fs::write(&running, first_runtime)
        .unwrap_or_else(|_| panic!("write initial runtime credential"));
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
        for id in [
            "platform-account-name",
            "platform-account-content",
            "platform-account-switch",
            "platform-account-rename",
            "platform-account-delete",
        ] {
            assert!(
                window.try_find(id).is_none(),
                "{id} should open only when needed"
            );
        }
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
        assert_eq!(
            std::fs::read(&running).unwrap_or_else(|_| panic!("read unchanged runtime credential")),
            first_runtime
        );
        window.click("platform-auth-off", cx);
        window.render_frame(cx);
        assert!(
            window
                .find("platform-confirmation")
                .label()
                .is_some_and(|label| label.contains("first") && label.contains("Saved accounts"))
        );
        window.press("escape", cx);
        window.render_frame(cx);
        window.click("platform-account-rename", cx);
        window.render_frame(cx);
        assert!(window.try_find("platform-account-name").is_some());
        window.click("platform-name-cancel", cx);
        window.render_frame(cx);
        window.click("platform-account-delete", cx);
        assert_eq!(view.read(cx).decision_name, "second");
        window.render_frame(cx);
        let alert = window.find("platform-confirmation");
        assert!(
            alert
                .label()
                .is_some_and(|t| t.contains("second") && t.contains("does not log out"))
        );
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
        window.click("platform-account-save", cx);
        window.render_frame(cx);
        view.read(cx)
            .account_input
            .clone()
            .update(cx, |input, cx| input.set_value("third", window, cx));
        window.render_frame(cx);
        window.click("platform-name-submit", cx);
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
    std::fs::write(
        roots.codex_runtime.join("config.toml"),
        r#"cli_auth_credentials_store = "keyring""#,
    )
    .unwrap_or_else(|_| panic!("select unsupported credential store"));
    cx.update(|cx| view.update(cx, |v, cx| v.load(cx)));
    cx.wait_for(handle, std::time::Duration::from_secs(10), |_, cx| {
        !view.read(cx).busy
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("platform-account-first", cx);
        window.render_frame(cx);
        assert_eq!(window.find("platform-account-first").selected(), Some(true));
        for action in [
            "platform-account-switch",
            "platform-account-rename",
            "platform-account-delete",
            "platform-auth-off",
            "platform-account-save",
        ] {
            assert_eq!(
                window.find(action).focused(),
                None,
                "{action} cannot write this store"
            );
        }
        assert!(window.find("platform-load").focused().is_some());
    })
    .unwrap_or_else(|_| panic!("inspect read-only account controls"));
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
        remote: None,
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
        remote: None,
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

fn assert_secret_hidden(window: &gpui_kit::Window, sentinel: &str) {
    assert!(
        gpui_kit::base::test_support::snapshots(window)
            .iter()
            .all(|node| [node.label(), node.value()]
                .into_iter()
                .flatten()
                .all(|s| !s.contains(sentinel))),
        "sensitive value reached an accessibility node"
    );
}

#[gpui_kit::test]
#[allow(clippy::unwrap_used)]
async fn codex_manual_quota_targets_history_and_late_results_from_accounts(
    cx: &mut TestAppContext,
) {
    use crate::runtime::platforms::{AccountFixture, QuotaRow, QuotaTarget};
    use std::time::Duration;
    let timeout = Duration::from_secs(10);
    cx.executor().allow_parking();
    let dir = tempfile::tempdir().unwrap();
    let roots = PlatformRoots::fixture(&dir.path().join("desktop-platform-fixture")).unwrap();
    ccr_core::core::LockManager::with_scoped_directory(&roots.locks, || {
        let owner = roots.codex_auth();
        let export = owner.build_export_account_from_auth_json(
            ccr_codex::CodexAuthJson {
                openai_api_key: Some("fixture-key".into()),
                tokens: None,
                last_refresh: None,
            },
            None,
            None,
            None,
            None,
        )?;
        ccr_cli::application::codex_oauth::commit_account(
            &owner,
            export,
            Some("first".into()),
            false,
        )
        .unwrap();
        Ok(())
    })
    .unwrap();
    let row = |name: &str, failed: bool| QuotaRow {
        account: name.into(),
        hourly: (!failed).then_some(0),
        weekly: None,
        hourly_reset: Some(1_800_000_000),
        weekly_reset: None,
        fetched_at: "2026-11-19T12:30:00Z".into(),
        failed,
        cache_hit: Some(false),
        history_warning: false,
    };
    let fixture = Arc::new(AccountFixture {
        quota_delay: Duration::from_millis(150),
        ..Default::default()
    });
    fixture.replies.lock().unwrap().extend([
        Ok(vec![row("runtime", false)]),
        Ok(vec![row("first", false)]),
        Ok(vec![row("first", true), row("second", false)]),
        Err(crate::runtime::platforms::PlatformError::Storage),
        Ok(vec![row("stale", false)]),
    ]);
    let owner = RuntimeOwner::new().unwrap();
    let mut services = Services::new(
        owner.handle(),
        crate::runtime::ConfigSource::Fixture {
            registry: dir.path().join("registry.toml"),
            profiles: dir.path().join("profiles.toml"),
            delay: Duration::ZERO,
        },
    );
    services.account_fixture = Some(fixture.clone());
    let (handle, view) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| {
                let mut v = PlatformsView::new(services, window, cx);
                v.open(Platform::Codex, Surface::Auth, window, cx);
                v
            })
        })
        .unwrap()
    });
    cx.wait_for(handle, timeout, |_, cx| !view.read(cx).busy)
        .await;
    cx.update_window(handle, |_, w, cx| {
        w.render_frame(cx);
        w.click("platform-tools-quota", cx);
        w.render_frame(cx);
        assert!(
            w.find("platform-quota-state")
                .label()
                .unwrap()
                .contains("No quota")
        );
        w.click("platform-quota-selected", cx);
        assert!(fixture.calls.lock().unwrap().is_empty());
        w.click("platform-tools-back", cx);
        w.render_frame(cx);
        assert_eq!(w.find("platform-tools-quota").focused(), Some(true));
        w.click("platform-account-first", cx);
        w.click("platform-tools-quota", cx);
        w.render_frame(cx);
        assert!(fixture.calls.lock().unwrap().is_empty());
        w.click("platform-quota-current", cx);
        w.click("platform-quota-current", cx);
    })
    .unwrap();
    cx.wait_for(handle, timeout, |_, cx| !view.read(cx).busy)
        .await;
    cx.update_window(handle, |_, w, cx| {
        w.render_frame(cx);
        let facts = w.find("platform-quota-result-0");
        let facts = facts.label().unwrap();
        assert!(
            facts.contains("0%")
                && facts.contains("unavailable")
                && facts.contains("2026-11-19")
                && facts.contains("2027-01-15")
        );
        assert_eq!(*fixture.calls.lock().unwrap(), [QuotaTarget::Runtime]);
        w.click("platform-quota-selected", cx);
    })
    .unwrap();
    cx.wait_for(handle, timeout, |_, cx| !view.read(cx).busy)
        .await;
    cx.update_window(handle, |_, w, cx| {
        w.render_frame(cx);
        assert_eq!(
            *fixture.calls.lock().unwrap(),
            [QuotaTarget::Runtime, QuotaTarget::Saved("first".into())]
        );
        w.click("platform-quota-all", cx);
    })
    .unwrap();
    cx.wait_for(handle, timeout, |_, cx| !view.read(cx).busy)
        .await;
    cx.update_window(handle, |_, w, cx| {
        w.render_frame(cx);
        assert_eq!(
            fixture.calls.lock().unwrap().last(),
            Some(&QuotaTarget::AllSaved)
        );
        assert!(
            w.find("platform-quota-state")
                .label()
                .unwrap()
                .contains("Partial")
        );
        assert!(
            w.find("platform-quota-history-0")
                .label()
                .unwrap()
                .contains("Last successful")
        );
        w.click("platform-quota-selected", cx);
    })
    .unwrap();
    cx.wait_for(handle, timeout, |_, cx| !view.read(cx).busy)
        .await;
    cx.update_window(handle, |_, w, cx| {
        w.render_frame(cx);
        assert!(
            w.find("platform-quota-state")
                .label()
                .unwrap()
                .contains("failed")
        );
        assert!(
            w.find("platform-quota-history-0")
                .label()
                .unwrap()
                .contains("first")
        );
        assert!(w.try_find("platform-quota-result-0").is_none());
        w.click("platform-quota-current", cx);
        w.render_frame(cx);
        w.click("platform-tools-add", cx);
        w.render_frame(cx);
        assert!(w.try_find("platform-quota-state").is_none());
        w.click("platform-tools-import", cx);
    })
    .unwrap();
    cx.wait_for(handle, timeout, |_, cx| !view.read(cx).busy)
        .await;
    cx.update_window(handle, |_, w, cx| {
        w.render_frame(cx);
        assert_eq!(w.find("platform-tools-import").focused(), Some(true));
        assert!(view.read(cx).quotas.is_empty());
        view.read(cx)
            .tools
            .import
            .clone()
            .update(cx, |i, cx| i.set_value("masked-import-sentinel", w, cx));
        w.render_frame(cx);
        assert_secret_hidden(w, "masked-import-sentinel");
        w.click("platform-account-import", cx);
        w.render_frame(cx);
        assert_secret_hidden(w, "masked-import-sentinel");
        w.click("platform-decision-cancel", cx);
        w.render_frame(cx);
        w.click("platform-tools-diagnostics", cx);
        w.render_frame(cx);
        w.click("platform-decision-cancel", cx);
        assert!(!view.read(cx).tools.import.read(cx).value().is_empty());
        w.render_frame(cx);
        w.click("platform-tools-diagnostics", cx);
        w.render_frame(cx);
        w.click("platform-decision-confirm", cx);
        w.render_frame(cx);
        assert!(w.try_find("codex-port-check").is_some());
        assert!(view.read(cx).tools.import.read(cx).value().is_empty());
        assert_eq!(
            fixture
                .preflights
                .load(std::sync::atomic::Ordering::Relaxed),
            0
        );
        w.press("escape", cx);
        w.render_frame(cx);
        assert_eq!(w.find("platform-tools-diagnostics").focused(), Some(true));
        w.remove_window();
    })
    .unwrap();
    owner.shutdown();
}
