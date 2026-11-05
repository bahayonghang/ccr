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
        surface: Surface::Settings,
        platform: Platform::Claude,
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
        surface: Surface::Auth,
        platform: Platform::Codex,
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
        surface: Surface::Settings,
        platform: Platform::Claude,
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
