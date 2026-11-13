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

/// Seeds a captured Claude root with same-name entries and legacy arrays holding credentials.
fn extension_fixture(root: &std::path::Path) -> (std::path::PathBuf, Vec<u8>) {
    let roots = ccr_cli::application::platform_roots::PlatformRoots::fixture(
        &root.join("desktop-resource-fixture"),
    )
    .unwrap();
    let config = roots.claude.config_dir.clone();
    for (relative, content) in [
        ("agents/reviewer.md", "---\nname: reviewer\n---\nagent-body"),
        (
            "agents/team/copy.md",
            "---\nname: reviewer\n---\nagent-body",
        ),
        (
            "skills/deploy/SKILL.md",
            "---\nname: deploy\n---\nskill-body",
        ),
        ("commands/deploy.md", "command-body"),
        (
            "plugins/installed_plugins.json",
            r#"{"version":2,"plugins":{"demo@alpha":[],"demo@beta":[]}}"#,
        ),
    ] {
        let path = config.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }
    let settings = roots.claude.settings_file.clone();
    let original = br#"{"agents":[{"name":"old","model":"m","token":"agentAAAASENTINELBBBB"}],"plugins":[{"id":"p","name":"p","version":"1","config":{"apiKey":"keyAAAASENTINELBBBB"}}]}"#.to_vec();
    std::fs::create_dir_all(settings.parent().unwrap()).unwrap();
    std::fs::write(&settings, &original).unwrap();
    (settings, original)
}

#[gpui_kit::test]
async fn claude_extensions_show_same_name_sources_and_gate_full_legacy_export(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    let (settings, original) = extension_fixture(root.path());
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
                    gpui_kit::Bounds::centered(None, gpui_kit::size(px(1360.), px(1400.)), cx),
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
    cx.update_window(handle, |_, window, cx| {
        view.update(cx, |this, cx| {
            this.open_extensions(Platform::Claude, window, cx)
        })
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(5), |_, cx| {
        !view.read(cx).busy && !view.read(cx).extension_sources.is_empty()
    })
    .await;
    let plaintext = |cx: &gpui_kit::App| {
        view.read(cx)
            .legacy_output
            .read(cx)
            .value()
            .contains("SENTINEL")
    };
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        // 同名条目仍各自列出，并标出另一来源。
        for id in [
            "claude-extension-same-name-0-0",
            "claude-extension-same-name-0-1",
            "claude-extension-same-name-1-0",
            "claude-extension-same-name-2-0",
            "claude-extension-same-name-3-0",
            "claude-extension-same-name-3-1",
        ] {
            assert!(window.try_find(id).is_some(), "{id}");
        }
        let label = |window: &mut Window, id: &str| {
            window
                .find(id.to_owned())
                .label()
                .unwrap_or_default()
                .to_owned()
        };
        let skill = label(window, "claude-extension-same-name-1-0");
        assert!(skill.contains("Same name “deploy”"), "{skill}");
        assert!(skill.contains("Legacy slash commands · user · deploy.md"));
        assert!(skill.contains("client loading unverified"));
        let plugin = label(window, "claude-extension-same-name-3-0");
        assert!(plugin.contains("demo@beta"), "{plugin}");
        assert!(plugin.contains("No documented precedence; client effect unverified"));
        assert!(
            label(window, "claude-extension-same-name-0-0")
                .contains("only one same-name subagent is loaded; which one is unverified")
        );
        assert!(
            label(window, "claude-extension-source-0")
                .ends_with("Discovered; client loading unverified")
        );
        assert!(label(window, "claude-extension-source-6").ends_with("Not connected"));
        assert!(
            label(window, "claude-extension-source-4")
                .starts_with("Legacy CCR entries — Claude does not read these arrays")
        );
        let sources = view.read(cx).extension_sources.clone();
        let skill = sources[1].entries[0].same_name.clone().unwrap();
        assert_eq!(skill.peers[0].location, "deploy.md");
        assert_eq!(
            skill.rule,
            ccr_app::resources::claude_extensions::SameNameRule::SkillOverCommand
        );
        assert_eq!(
            sources[3].entries[1].same_name.clone().unwrap().rule,
            ccr_app::resources::claude_extensions::SameNameRule::Unverified
        );
        assert!(window.try_find("claude-extension-legacy-output").is_none());
        // 脱敏导出是默认动作，不经确认。
        window.click("claude-extension-legacy-export", cx);
        assert!(view.read(cx).decision.is_none());
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(5), |_, cx| {
        !view.read(cx).busy && view.read(cx).legacy_full == Some(false)
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        let value = view.read(cx).legacy_output.read(cx).value().to_string();
        assert!(value.contains("\"plugins\""));
        assert!(!plaintext(cx));
        assert!(
            view.read(cx)
                .message
                .is_some_and(|message| message.contains("not saved"))
        );
        // 完整导出必须确认：从外部焦点打开，初始焦点在取消，Tab 留在决定子树，Esc 返回触发者。
        let outside = host.read(cx).outside.clone();
        window.focus(&outside, cx);
        window.click("claude-extension-legacy-export-full", cx);
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        assert_eq!(
            window.find("claude-extension-legacy-cancel").focused(),
            Some(true)
        );
        // 确认期间底层动作不可操作：禁用按钮没有焦点绑定，点击也不发起读取。
        for id in [
            "claude-extension-legacy-export",
            "claude-extension-legacy-save-file",
            "claude-extension-legacy-close",
        ] {
            assert_eq!(window.find(id).focused(), None, "{id} disabled");
        }
        window.click("claude-extension-legacy-export", cx);
        assert!(!view.read(cx).busy);
        window.press("tab", cx);
        assert_eq!(
            window.find("claude-extension-legacy-confirm").focused(),
            Some(true)
        );
        window.press("tab", cx);
        assert_eq!(
            window.find("claude-extension-legacy-cancel").focused(),
            Some(true)
        );
        window.press("shift-tab", cx);
        assert_eq!(
            window.find("claude-extension-legacy-confirm").focused(),
            Some(true)
        );
        window.press("escape", cx);
        window.render_frame(cx);
        assert!(view.read(cx).decision.is_none());
        assert!(
            window
                .try_find("claude-extension-legacy-confirmation")
                .is_none()
        );
        assert_eq!(window.find("resource-outside").focused(), Some(true));
        // 取消后没有发起读取，界面里也没有明文。
        assert!(!view.read(cx).busy);
        assert_eq!(view.read(cx).legacy_full, Some(false));
        assert!(!plaintext(cx));
        window.click("claude-extension-legacy-export-full", cx);
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        window.click("claude-extension-legacy-cancel", cx);
        window.render_frame(cx);
        assert!(view.read(cx).decision.is_none());
        assert!(!view.read(cx).busy);
        assert!(!plaintext(cx));
        window.click("claude-extension-legacy-export-full", cx);
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        window.click("claude-extension-legacy-confirm", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(5), |_, cx| {
        !view.read(cx).busy && view.read(cx).legacy_full == Some(true)
    })
    .await;
    let destination = root.path().join("legacy.json");
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(plaintext(cx));
        assert!(!destination.exists(), "showing a copy never writes a file");
        window.click("claude-extension-legacy-save-file", cx);
    })
    .unwrap();
    assert!(cx.did_prompt_for_new_path());
    cx.simulate_new_path_selection(|_| None);
    cx.run_until_parked();
    assert!(!destination.exists());
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("claude-extension-legacy-save-file", cx);
    })
    .unwrap();
    let chosen = destination.clone();
    cx.simulate_new_path_selection(move |_| Some(chosen));
    cx.wait_for(handle, Duration::from_secs(5), |_, cx| {
        !view.read(cx).busy
            && view
                .read(cx)
                .message
                .is_some_and(|message| message.contains("Export file saved"))
    })
    .await;
    assert!(
        std::fs::read_to_string(&destination)
            .unwrap()
            .contains("keyAAAASENTINELBBBB")
    );
    // 已有文件不覆盖。
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("claude-extension-legacy-save-file", cx);
    })
    .unwrap();
    let chosen = destination.clone();
    cx.simulate_new_path_selection(move |_| Some(chosen));
    cx.wait_for(handle, Duration::from_secs(5), |_, cx| {
        !view.read(cx).busy
            && view
                .read(cx)
                .message
                .is_some_and(|message| message.contains("already exists"))
    })
    .await;
    // 离页清除明文；离页前发出的读取回包不再生效。
    cx.update_window(handle, |_, window, cx| {
        view.update(cx, |this, cx| {
            this.leave(window, cx);
            this.open_extensions(Platform::Claude, window, cx);
        });
        assert!(view.read(cx).legacy_full.is_none());
        assert!(view.read(cx).legacy_pending.is_none());
        assert!(!plaintext(cx));
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(5), |_, cx| !view.read(cx).busy)
        .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("claude-extension-legacy-export-full", cx);
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        window.click("claude-extension-legacy-confirm", cx);
        assert!(view.read(cx).busy);
        view.update(cx, |this, cx| this.leave(window, cx));
    })
    .unwrap();
    for _ in 0..30 {
        cx.run_until_parked();
        std::thread::sleep(Duration::from_millis(10));
    }
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(view.read(cx).legacy_full.is_none());
        assert!(view.read(cx).legacy_pending.is_none());
        assert!(!plaintext(cx));
        assert!(window.try_find("claude-extension-legacy-output").is_none());
        window.remove_window();
    })
    .unwrap();
    assert_eq!(std::fs::read(&settings).unwrap(), original);
    owner.shutdown();
}
