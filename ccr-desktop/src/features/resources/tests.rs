//! Rendered fixtures only. Actual native OS and IME evidence belong to main.
#![allow(clippy::unwrap_used)]
use super::*;
use crate::runtime::ConfigSource;
use ccr_app::runtime::RuntimeOwner;
use gpui_kit::test::{TestAppContextExt, TestWindowExt};
use gpui_kit::{
    InputEvent, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, TestAppContext,
    WindowOptions,
};
use std::time::Duration;

struct Host {
    view: Entity<ResourcesView>,
    outside: FocusHandle,
    _subscription: gpui_kit::Subscription,
}
impl Render for Host {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .child(
                div()
                    .id("resource-outside")
                    .test_support()
                    .track_focus(&self.outside)
                    .tab_stop(true)
                    .child("Outside"),
            )
            .child(self.view.clone())
    }
}

#[gpui_kit::test]
async fn scoped_editor_uses_real_workers_original_revision_and_cancel_teardown(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    let target = root
        .path()
        .join("desktop-resource-fixture/home/.claude/CLAUDE.md");
    std::fs::create_dir_all(target.parent().unwrap()).unwrap();
    std::fs::write(&target, "fixture-source-private-sentinel").unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let services = Services::new(
        owner.handle(),
        ConfigSource::Fixture {
            profiles: root.path().join("profiles.toml"),
            registry: root.path().join("registry.toml"),
            delay: Duration::ZERO,
        },
    );
    let (handle, host) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| {
                let view = cx.new(|cx| ResourcesView::new(services, window, cx));
                let subscription = cx.observe(&view, |_, _, cx| cx.notify());
                Host {
                    view,
                    outside: cx.focus_handle(),
                    _subscription: subscription,
                }
            })
        })
        .unwrap()
    });
    let view = cx.update(|cx| host.read(cx).view.clone());
    cx.update_window(handle, |_, _, cx| view.update(cx, |this, cx| this.load(cx)))
        .unwrap();
    cx.wait_for(handle, Duration::from_secs(5), |_, cx| {
        !view.read(cx).busy && view.read(cx).rows.len() == 1
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        let outside = host.read(cx).outside.clone();
        window.focus(&outside, cx);
        window.render_frame(cx);
        let position = window
            .find("resource-open-claude-user-memory")
            .bounds()
            .center();
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
        assert!(view.read(cx).decision.is_some());
        assert!(view.read(cx).session.is_none());
        assert!(view.read(cx).editor.read(cx).value().is_empty());
        // Match the native callback-before-draw boundary, without seeding focus inside.
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        window.focus(&outside, cx);
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        assert_eq!(
            window.find("resource-decision-cancel").focused(),
            Some(true)
        );
        window.press("tab", cx);
        assert_eq!(
            window.find("resource-decision-confirm").focused(),
            Some(true)
        );
        window.press("shift-tab", cx);
        assert_eq!(
            window.find("resource-decision-cancel").focused(),
            Some(true)
        );
        window.press("escape", cx);
        assert!(view.read(cx).decision.is_none());
        assert_eq!(window.find("resource-outside").focused(), Some(true));
        assert!(view.read(cx).session.is_none());
        window.click("resource-open-claude-user-memory", cx);
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        window.click("resource-decision-confirm", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(5), |_, cx| {
        !view.read(cx).busy && view.read(cx).session.is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(
            view.read(cx).editor.read(cx).value().as_ref(),
            "fixture-source-private-sentinel"
        );
        let session = view.read(cx).session.clone().unwrap();
        let result = ccr_app::resources::system_prompts::save_prompt_file_with_locks(
            &session.spec,
            &target,
            &root.path().join("external-backups"),
            "external winner",
            &session.snapshot.token,
            &root.path().join("desktop-resource-fixture/locks"),
        )
        .unwrap();
        assert_eq!(result["status"], "saved");
        view.read(cx).editor.clone().update(cx, |state, cx| {
            state.set_value("retained-native-draft", window, cx)
        });
        window.render_frame(cx);
        window.click("resource-save", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(5), |_, cx| !view.read(cx).busy)
        .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(
            view.read(cx).editor.read(cx).value().as_ref(),
            "retained-native-draft"
        );
        assert!(
            view.read(cx)
                .message
                .is_some_and(|message| message.contains("original revision retained"))
        );
        assert!(view.read(cx).session.is_some());
        assert_eq!(std::fs::read_to_string(&target).unwrap(), "external winner");
        window.click("resource-reload", cx);
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        assert_eq!(
            window.find("resource-decision-cancel").focused(),
            Some(true)
        );
        window.press("escape", cx);
        assert_eq!(
            view.read(cx).editor.read(cx).value().as_ref(),
            "retained-native-draft"
        );
        window.click("resource-editor-close", cx);
        window.render_frame(cx);
        assert!(view.read(cx).session.is_none());
        assert!(view.read(cx).editor.read(cx).value().is_empty());
        assert!(view.read(cx).pending_content.is_none());
        assert!(view.read(cx).decision.is_none());
        view.update(cx, |this, cx| this.leave(window, cx));
        window.remove_window();
    })
    .unwrap();
    owner.shutdown();
}

#[gpui_kit::test]
fn leave_discards_scoped_pending_text_and_old_response_identity(cx: &mut TestAppContext) {
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
    let (handle, view) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| ResourcesView::new(services, window, cx))
        })
        .unwrap()
    });
    cx.update_window(handle, |_, window, cx| {
        let key = RequestKey {
            environment: EnvironmentId::Local("fixture".into()),
            platform: Platform::Claude,
            generation: 8,
            request_id: 11,
        };
        view.update(cx, |this, cx| {
            this.gate.replace(key.clone());
            this.pending_content = Some("pending-private-sentinel".into());
            this.decision = Some(Decision::Open("claude-user-memory"));
            this.decision_entry_pending = true;
            this.leave(window, cx);
        });
        window.render_frame(cx);
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        assert!(view.read(cx).pending_content.is_none());
        assert!(view.read(cx).editor.read(cx).value().is_empty());
        assert!(view.read(cx).decision.is_none());
        assert!(!view.read(cx).gate.accepts(&key));
        view.update(cx, |this, cx| this.focus_main(window, cx));
        window.render_frame(cx);
        assert_eq!(
            window.find("resource-platform-claude").focused(),
            Some(true)
        );
        window.remove_window();
    })
    .unwrap();
    owner.shutdown();
}

fn template_click(window: &mut Window, id: &'static str, cx: &mut gpui_kit::App) {
    let position = window.find(id).bounds().center();
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
    window.click(id, cx);
}
#[track_caller]
fn template_wait(
    cx: &mut TestAppContext,
    handle: gpui_kit::AnyWindowHandle,
    mut ready: impl FnMut(&mut Window, &mut gpui_kit::App) -> bool,
) {
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        cx.run_until_parked();
        if cx
            .update_window(handle, |_, window, cx| {
                window.render_frame(cx);
                ready(window, cx)
            })
            .unwrap()
        {
            cx.run_until_parked();
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "template UI did not reach expected state"
        );
        cx.executor().advance_clock(Duration::from_millis(10));
        std::thread::yield_now();
    }
}

#[gpui_kit::test]
fn builtin_template_presets_feed_the_original_instruction_session(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("desktop-resource-fixture/home");
    let target = home.join(".claude/CLAUDE.md");
    std::fs::create_dir_all(target.parent().unwrap()).unwrap();
    std::fs::write(&target, "original instruction").unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let services = Services::new(
        owner.handle(),
        ConfigSource::Fixture {
            profiles: root.path().join("profiles.toml"),
            registry: root.path().join("registry.toml"),
            delay: Duration::ZERO,
        },
    );
    let (handle, host) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(gpui_kit::WindowBounds::Windowed(
                    gpui_kit::Bounds::centered(None, gpui_kit::size(px(1360.), px(900.)), cx),
                )),
                ..Default::default()
            },
            cx,
            |window, cx| {
                cx.new(|cx| {
                    let view = cx.new(|cx| ResourcesView::new(services, window, cx));
                    let subscription = cx.observe(&view, |_, _, cx| cx.notify());
                    Host {
                        view,
                        outside: cx.focus_handle(),
                        _subscription: subscription,
                    }
                })
            },
        )
        .unwrap()
    });
    let view = cx.update(|cx| host.read(cx).view.clone());
    cx.update_window(handle, |_, _, cx| view.update(cx, |this, cx| this.load(cx)))
        .unwrap();
    template_wait(cx, handle, |_, cx| {
        !view.read(cx).busy && !view.read(cx).rows.is_empty()
    });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        template_click(window, "resource-open-claude-user-memory", cx);
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        template_click(window, "resource-decision-confirm", cx);
    })
    .unwrap();
    template_wait(cx, handle, |_, cx| {
        !view.read(cx).busy && view.read(cx).session.is_some()
    });
    cx.run_until_parked();
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        template_click(window, "resource-templates", cx);
    })
    .unwrap();
    template_wait(cx, handle, |window, cx| {
        window.render_frame(cx);
        window.try_find("template-row-0").is_some()
    });
    cx.run_until_parked();
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        template_click(window, "template-row-0", cx);
        window.render_frame(cx);
        template_click(window, "template-save", cx);
        window.render_frame(cx);
        template_click(window, "template-confirm", cx);
    })
    .unwrap();
    let preset_path = ccr_skills::PromptsManager::for_home(ccr_config::Platform::Claude, &home)
        .config_path()
        .to_owned();
    template_wait(cx, handle, |window, cx| {
        window.render_frame(cx);
        preset_path.exists() && window.try_find("template-row-0").is_some()
    });
    assert_eq!(
        std::fs::read_to_string(&target).unwrap(),
        "original instruction"
    );
    let builtin = ccr_skills::get_builtin_prompts().remove(0);
    let saved = ccr_skills::PromptsManager::for_home(ccr_config::Platform::Claude, &home)
        .list_presets()
        .unwrap();
    assert_eq!(saved.len(), 1);
    assert_eq!(saved[0].content, builtin.content);
    cx.run_until_parked();
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        template_click(window, "template-row-0", cx);
        window.render_frame(cx);
        template_click(window, "template-use", cx);
        window.render_frame(cx);
        window.press("escape", cx);
        window.render_frame(cx);
        assert!(window.try_find("template-confirm").is_none());
        assert_eq!(
            std::fs::read_to_string(&target).unwrap(),
            "original instruction"
        );
        template_click(window, "template-use", cx);
        window.render_frame(cx);
        template_click(window, "template-confirm", cx);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(
            view.read(cx).editor.read(cx).value().as_ref(),
            builtin.content
        );
        assert_eq!(
            std::fs::read_to_string(&target).unwrap(),
            "original instruction"
        );
        template_click(window, "resource-save", cx);
    })
    .unwrap();
    template_wait(cx, handle, |_, cx| !view.read(cx).busy);
    assert_eq!(std::fs::read_to_string(&target).unwrap(), builtin.content);
    cx.update_window(handle, |_, window, cx| {
        view.update(cx, |this, cx| this.leave(window, cx));
        window.remove_window();
    })
    .unwrap();
    owner.shutdown();
}
