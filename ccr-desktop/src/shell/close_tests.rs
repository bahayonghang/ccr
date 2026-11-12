#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::Shell;
use crate::runtime::{Services, profiles::tests::fixture};
use ccr_app::runtime::{Completion, EnvironmentId, Platform, RequestKey, RuntimeOwner};
use gpui_kit::test::TestWindowExt;
use gpui_kit::{
    AppContext, Bounds, TestAppContext, VisualTestContext, WindowBounds, WindowOptions, px, size,
};
use std::{
    sync::mpsc,
    time::{Duration, Instant},
};

fn wait_for(
    cx: &mut TestAppContext,
    handle: gpui_kit::AnyWindowHandle,
    stage: &str,
    mut predicate: impl FnMut(&mut gpui_kit::Window) -> bool,
) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        cx.run_until_parked();
        if cx
            .update_window(handle, |_, window, cx| {
                window.render_frame(cx);
                predicate(window)
            })
            .unwrap()
        {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "rendered fixture setup timed out: {stage}"
        );
        cx.executor().advance_clock(Duration::from_millis(10));
        std::thread::yield_now();
    }
}

fn wait_closed(cx: &mut TestAppContext, handle: gpui_kit::AnyWindowHandle) {
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut last_tick = Instant::now();
    loop {
        cx.run_until_parked();
        if cx.update_window(handle, |_, _, _| ()).is_err() {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "final preference write did not close the window"
        );
        // 实际文件写入由 Tokio worker 执行；不能让忙等次数提前耗尽两秒排队预算。
        let now = Instant::now();
        cx.executor().advance_clock(now - last_tick);
        last_tick = now;
        std::thread::yield_now();
    }
}

#[gpui_kit::test]
fn quit_action_and_saved_preferences_share_the_window_close_policy(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let (root, source) = fixture();
    let owner = RuntimeOwner::new().unwrap();
    cx.update(gpui_kit::init);
    for reopened in [false, true] {
        let (handle, shell) = cx.update(|cx| {
            gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
                cx.new(|cx| Shell::new(Services::new(owner.handle(), source.clone()), window, cx))
            })
            .unwrap()
        });
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.click("nav-settings", cx);
        })
        .unwrap();
        wait_for(
            cx,
            handle,
            if reopened {
                "reopened preferences load"
            } else {
                "initial preferences load"
            },
            |window| window.find("theme-dark").disabled() != Some(true),
        );
        if !reopened {
            cx.update_window(handle, |_, window, cx| {
                window.dispatch_action(Box::new(super::close::Quit), cx);
            })
            .unwrap();
            cx.run_until_parked();
            cx.update_window(handle, |_, window, cx| {
                window.render_frame(cx);
                assert!(window.try_find("dialog").is_some());
            })
            .unwrap();
            // click() samples a position once, then draws between pointer
            // events. The sliding button must settle first: the kit's one-shot
            // animation uses wall time, not the test executor's virtual clock.
            let mounted_at = Instant::now();
            wait_for(cx, handle, "quit dialog entrance settled", |window| {
                mounted_at.elapsed() >= *gpui_kit::component::dialog::ANIMATION_DURATION
                    && window.find("cancel").visible()
            });
            cx.update_window(handle, |_, window, cx| {
                window.simulate_next_frame(cx);
                window.render_frame(cx);
                window.click("cancel", cx);
            })
            .unwrap();
            // Drain the deferred close before clicking the underlying settings.
            wait_for(cx, handle, "cancel dialog dismissal", |window| {
                window.try_find("dialog").is_none()
            });
            cx.update_window(handle, |_, window, cx| {
                // Rapid preference changes are immediate and serialize on disk.
                window.click("theme-dark", cx);
                assert_eq!(
                    gpui_kit::component::Theme::global(cx).mode,
                    gpui_kit::component::ThemeMode::Dark,
                    "the theme click must reach the preference control"
                );
                window.click("language-zh", cx);
                assert!(matches!(
                    cx.global::<crate::locales::Language>(),
                    crate::locales::Language::Chinese
                ));
                window.click("close-direct", cx);
                window.render_frame(cx);
                assert_eq!(
                    window.find("close-direct").checked(),
                    Some(true),
                    "the direct-close click must change the rendered policy"
                );
                assert_eq!(
                    shell.read(cx).preferences_generation,
                    3,
                    "all three clicks must submit their preference snapshots"
                );
            })
            .unwrap();
            wait_for(cx, handle, "three preference writes complete", |window| {
                assert!(
                    window.try_find("preferences-retry").is_none(),
                    "preference persistence failed"
                );
                window.find("preferences-reveal").disabled() != Some(true)
            });
            let value: serde_json::Value = serde_json::from_slice(
                &std::fs::read(root.path().join("desktop-gpui/preferences.json")).unwrap(),
            )
            .unwrap();
            assert_eq!(value["theme"], "dark");
            assert_eq!(value["language"], "zh");
            assert_eq!(value["confirm_quit"], false);
        } else {
            cx.update_window(handle, |_, window, cx| {
                assert_eq!(
                    gpui_kit::component::Theme::global(cx).mode,
                    gpui_kit::component::ThemeMode::Dark
                );
                assert_eq!(window.find("nav-settings").label(), Some("设置"));
                assert_eq!(window.find("close-direct").label(), Some("直接退出"));
            })
            .unwrap();
            let mut visual = VisualTestContext::from_window(handle, cx);
            assert!(
                !visual.simulate_close(),
                "direct OS close must wait for the final preference write"
            );
        }
        cx.update_window(handle, |_, window, cx| {
            window.press(
                if cfg!(target_os = "macos") {
                    "cmd-q"
                } else {
                    "ctrl-q"
                },
                cx,
            );
        })
        .unwrap();
        cx.run_until_parked();
        wait_closed(cx, handle);
    }
    owner.shutdown();
}

#[gpui_kit::test]
async fn window_bounds_are_saved_before_close_and_restored_by_the_launch_path(
    cx: &mut TestAppContext,
) {
    use ccr_app::runtime::{BLOCKING_CONCURRENCY, TaskEvent};
    cx.executor().allow_parking();
    let (root, source) = fixture();
    let mut owner = ShutdownOnDrop(Some(RuntimeOwner::new().unwrap()));
    let executor = owner.0.as_ref().unwrap().handle();
    let services = Services::new(executor.clone(), source);
    let requests = crate::runtime::preferences::PreferencesRequests::new(&services);
    let path = root.path().join("desktop-gpui/preferences.json");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(
        &path,
        serde_json::json!({
            "version":1, "theme":"light", "language":"en", "confirm_quit":false,
            "window":{"x":50,"y":75,"width":1280,"height":800,"maximized":false},
        })
        .to_string(),
    )
    .unwrap();
    cx.update(gpui_kit::init);
    let (handle, shell) = crate::open_app_window(services.clone(), &mut cx.to_async())
        .await
        .unwrap();
    cx.update_window(handle, |_, window, cx| {
        assert_eq!(
            window.window_bounds().get_bounds().origin,
            gpui_kit::point(px(50.), px(75.))
        );
        assert_eq!(
            window.window_bounds().get_bounds().size,
            size(px(1280.), px(800.))
        );
        window.render_frame(cx);
        window.click("nav-settings", cx);
        window.click("theme-dark", cx);
        window.click("language-zh", cx);
    })
    .unwrap();
    wait_for(cx, handle, "theme and language saved", |window| {
        window.find("preferences-reveal").disabled() != Some(true)
    });
    let before = std::fs::read(&path).unwrap();
    let mut blockers = Vec::new();
    for index in 0..BLOCKING_CONCURRENCY {
        let (entered, receive) = mpsc::channel();
        blockers.push(
            executor
                .job(
                    RequestKey {
                        environment: EnvironmentId::Local("window-close-fixture".into()),
                        platform: Platform::Claude,
                        generation: 0,
                        request_id: index as u64,
                    },
                    move |cancel, _| {
                        entered.send(()).unwrap();
                        tokio::runtime::Handle::current().block_on(cancel.cancelled());
                        Completion::<(), ()>::Cancelled
                    },
                )
                .unwrap(),
        );
        receive.recv_timeout(Duration::from_secs(5)).unwrap();
    }
    // The platform resize callback, not direct model assignment, updates the
    // saved geometry. Close flushes the pending debounce before it can expire.
    VisualTestContext::from_window(handle, cx).simulate_resize(size(px(1320.), px(820.)));
    cx.update_window(handle, |_, window, cx| {
        assert_eq!(shell.read(cx).window_placement.unwrap().width, 1320.);
        window.dispatch_action(Box::new(super::close::Quit), cx);
    })
    .unwrap();
    cx.run_until_parked();
    let generation = cx
        .update_window(handle, |_, window, cx| {
            assert!(shell.read(cx).preferences_closing);
            assert!(window.try_find("dialog").is_none());
            window.dispatch_action(Box::new(super::close::Quit), cx);
            shell.read(cx).preferences_generation
        })
        .unwrap();
    cx.run_until_parked();
    cx.update_window(handle, |_, _, cx| {
        assert_eq!(shell.read(cx).preferences_generation, generation)
    })
    .unwrap();
    assert_eq!(
        std::fs::read(&path).unwrap(),
        before,
        "queued final save has not completed"
    );
    for blocker in blockers {
        blocker.request_cancel();
    }
    wait_closed(cx, handle);
    let mut read = requests.load().unwrap();
    let Some(TaskEvent::Terminal(result)) = read.next().await else {
        panic!("preference read terminal missing");
    };
    let Completion::Success(saved) = result.as_ref() else {
        panic!("preference read failed");
    };
    assert!(saved.theme == crate::theme::Preference::Dark);
    assert!(saved.language == crate::locales::Language::Chinese);
    assert!(!saved.confirm_quit);
    assert_eq!(saved.window.unwrap().width, 1320.);
    assert_eq!(saved.window.unwrap().height, 820.);
    let (reopened, _) = crate::open_app_window(services, &mut cx.to_async())
        .await
        .unwrap();
    cx.update_window(reopened, |_, window, cx| {
        assert_eq!(
            window.window_bounds().get_bounds().origin,
            gpui_kit::point(px(50.), px(75.))
        );
        assert_eq!(
            window.window_bounds().get_bounds().size,
            size(px(1320.), px(820.))
        );
        assert!(cx.global::<crate::locales::Language>() == &crate::locales::Language::Chinese);
        window.dispatch_action(Box::new(super::close::Quit), cx);
    })
    .unwrap();
    wait_closed(cx, reopened);
    owner.0.take().unwrap().shutdown();
    assert_eq!(executor.active_tasks(), 0);
}

#[gpui_kit::test]
fn preference_retry_recovers_the_failed_read_or_write(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let (root, source) = fixture();
    let path = root.path().join("desktop-gpui/preferences.json");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, b"{broken").unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let (handle, _) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| Shell::new(Services::new(owner.handle(), source), window, cx))
        })
        .unwrap()
    });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("nav-settings", cx);
    })
    .unwrap();
    wait_for(cx, handle, "failed load", |w| {
        w.try_find("preferences-retry").is_some()
    });
    let saved_light = br#"{"version":1,"theme":"light","language":"en","confirm_quit":true}"#;
    std::fs::write(&path, saved_light).unwrap();
    cx.update_window(handle, |_, window, cx| {
        assert_eq!(
            window.find("preferences-retry").label(),
            Some("Reload preferences")
        );
        window.click("preferences-retry", cx);
    })
    .unwrap();
    wait_for(cx, handle, "reload", |w| {
        w.find("theme-dark").disabled() != Some(true)
    });
    std::fs::write(&path, b"{broken").unwrap();
    cx.update_window(handle, |_, window, cx| window.click("theme-dark", cx))
        .unwrap();
    wait_for(cx, handle, "failed save", |w| {
        w.try_find("preferences-retry").is_some()
    });
    assert_eq!(std::fs::read(&path).unwrap(), b"{broken");
    std::fs::write(&path, saved_light).unwrap();
    cx.update_window(handle, |_, window, cx| {
        assert_eq!(window.find("preferences-retry").label(), Some("Retry save"));
        assert_eq!(window.find("theme-dark").checked(), Some(true));
        window.click("preferences-retry", cx);
    })
    .unwrap();
    wait_for(cx, handle, "retry write", |w| {
        w.find("preferences-reveal").disabled() != Some(true)
    });
    let persisted: serde_json::Value =
        serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(persisted["theme"], "dark");
    cx.update_window(handle, |_, window, _| window.remove_window())
        .unwrap();
    owner.shutdown();
}

#[gpui_kit::test]
async fn corrupt_launch_preferences_remain_visible_and_are_not_replaced_on_quit(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let (root, source) = fixture();
    let owner = RuntimeOwner::new().unwrap();
    let path = root.path().join("desktop-gpui/preferences.json");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, b"{broken").unwrap();
    cx.update(|cx| {
        gpui_kit::init(cx);
        cx.set_reduce_motion(true);
    });
    let (handle, shell) =
        crate::open_app_window(Services::new(owner.handle(), source), &mut cx.to_async())
            .await
            .unwrap();
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("nav-settings", cx);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(window.try_find("preferences-retry").is_some());
        assert!(!shell.read(cx).preferences_loaded);
        // The kit does not expose a disabled flag for every button snapshot.
        // Exercise the actual control instead: unreadable prefs cannot be
        // replaced by selecting a theme over default values.
        window.click("theme-dark", cx);
        assert!(shell.read(cx).preference == crate::theme::Preference::Light);
        assert_eq!(shell.read(cx).preferences_generation, 0);
        window.dispatch_action(Box::new(super::close::Quit), cx);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.simulate_next_frame(cx);
        window.press("tab", cx);
        assert_eq!(window.find("ok").focused(), Some(true));
        window.press("enter", cx);
    })
    .unwrap();
    wait_closed(cx, handle);
    assert_eq!(std::fs::read(&path).unwrap(), b"{broken");
    owner.shutdown();
}

#[gpui_kit::test]
fn close_request_requires_confirmation_and_cancel_preserves_window(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let (root, source) = fixture();
    let owner = RuntimeOwner::new().unwrap();
    for language in [
        crate::locales::Language::English,
        crate::locales::Language::Chinese,
    ] {
        let (cancel_label, quit_label) = match language {
            crate::locales::Language::English => ("Keep working", "Quit"),
            crate::locales::Language::Chinese => ("继续使用", "退出"),
        };
        let (handle, _) = cx.update(|cx| {
            gpui_kit::init(cx);
            cx.set_reduce_motion(true);
            gpui_kit::open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                        None,
                        size(px(1280.), px(800.)),
                        cx,
                    ))),
                    ..Default::default()
                },
                cx,
                |window, cx| {
                    cx.new(|cx| {
                        Shell::new(Services::new(owner.handle(), source.clone()), window, cx)
                    })
                },
            )
            .unwrap()
        });
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.click("nav-settings", cx);
        })
        .unwrap();
        wait_for(cx, handle, "close policy preferences load", |window| {
            window.find("theme-dark").disabled() != Some(true)
        });
        let mut visual = VisualTestContext::from_window(handle, cx);
        visual.update(|_, cx| cx.set_global(language));
        assert!(!visual.simulate_close());
        visual.update(|window, cx| {
            window.press("enter", cx);
            assert!(
                window.try_find("dialog").is_none(),
                "root Enter must safely cancel"
            );
        });
        assert!(!visual.simulate_close());
        visual.update(|window, cx| {
            window.render_frame(cx);
            window.simulate_next_frame(cx);
            window.render_frame(cx);
            assert!(window.try_find("dialog").is_some(), "close must ask first");
            assert_eq!(window.find("cancel").label(), Some(cancel_label));
            assert_eq!(window.find("cancel").focused(), Some(true));
            window.press("tab", cx);
            assert_eq!(window.find("ok").focused(), Some(true));
            window.simulate_next_frame(cx);
            window.render_frame(cx);
            assert_eq!(
                window.find("ok").focused(),
                Some(true),
                "no late focus theft"
            );
            window.press("tab", cx);
            assert_eq!(window.find("cancel").focused(), Some(true));
            window.press("shift-tab", cx);
            assert_eq!(window.find("ok").focused(), Some(true));
            window.press("escape", cx);
            assert!(window.try_find("dialog").is_none());
        });
        // Dismiss and reopen before the old opening's next-frame callback runs.
        assert!(!visual.simulate_close());
        visual.update(|window, cx| {
            window.render_frame(cx);
            window.press("escape", cx);
        });
        assert!(!visual.simulate_close());
        visual.update(|window, cx| {
            window.render_frame(cx);
            window.press("tab", cx);
            window.press("tab", cx);
            assert_eq!(window.find("ok").focused(), Some(true));
            window.simulate_next_frame(cx);
            window.render_frame(cx);
            assert_eq!(
                window.find("ok").focused(),
                Some(true),
                "dismissed opening must not steal focus after reopen"
            );
            window.press("shift-tab", cx);
            window.press("enter", cx);
            assert!(window.try_find("dialog").is_none());
        });
        assert!(!visual.simulate_close());
        assert!(!visual.simulate_close());
        visual.update(|window, cx| {
            window.render_frame(cx);
            window.click("cancel", cx);
            assert!(window.try_find("dialog").is_none(), "close must not stack");
        });
        assert!(!visual.simulate_close());
        visual.update(|window, cx| {
            window.render_frame(cx);
            window.simulate_next_frame(cx);
            window.render_frame(cx);
            assert_eq!(window.find("cancel").label(), Some(cancel_label));
            assert_eq!(window.find("ok").label(), Some(quit_label));
            assert_eq!(window.find("cancel").focused(), Some(true));
            window.press("enter", cx);
            assert!(
                window.try_find("dialog").is_none(),
                "Enter on Cancel stays open"
            );
        });
        assert!(!visual.simulate_close());
        visual.update(|window, cx| {
            window.render_frame(cx);
            window.simulate_next_frame(cx);
            window.press("tab", cx);
            assert_eq!(window.find("ok").focused(), Some(true));
            window.press("enter", cx);
        });
        wait_closed(cx, handle);
    }
    owner.shutdown();
    drop(root);
}

struct ShutdownOnDrop(Option<RuntimeOwner>);
impl Drop for ShutdownOnDrop {
    fn drop(&mut self) {
        if let Some(owner) = self.0.take() {
            owner.shutdown();
        }
    }
}

#[gpui_kit::test]
fn cancelling_close_preserves_nested_profile_confirmation_draft_and_running_job(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let (_root, source) = fixture();
    let context = crate::runtime::profiles::context(&source, ccr_config::Platform::Claude).unwrap();
    let revision = context.manager().snapshot().unwrap().version;
    let mut owner = ShutdownOnDrop(Some(RuntimeOwner::new().unwrap()));
    let executor = owner.0.as_ref().unwrap().handle();
    let (handle, _) = cx.update(|cx| {
        gpui_kit::init(cx);
        cx.set_reduce_motion(true);
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(1280.), px(800.)),
                    cx,
                ))),
                ..Default::default()
            },
            cx,
            |window, cx| {
                cx.new(|cx| Shell::new(Services::new(executor.clone(), source), window, cx))
            },
        )
        .unwrap()
    });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("nav-claude-code", cx);
        window.click("tool-profiles", cx);
    })
    .unwrap();
    wait_for(
        cx,
        handle,
        "profile rows loaded and worker idle",
        |window| window.try_find("configuration:alpha").is_some() && executor.active_tasks() == 0,
    );
    cx.update_window(handle, |_, window, cx| {
        window.click("configuration:alpha", cx)
    })
    .unwrap();
    wait_for(cx, handle, "profile edit enabled", |window| {
        // Kit snapshots expose Some(true) for disabled and None when cleared.
        // Successful editor input below additionally proves input acceptance.
        window.find("profile-edit").disabled() != Some(true)
    });
    cx.update_window(handle, |_, window, cx| {
        window.click("profile-edit", cx);
        window.click("profile-description", cx);
        window.press("ctrl-a", cx);
        window.input("Unconfirmed draft", cx);
        for _ in 0..12 {
            window.press("tab", cx);
            if window.find("profile-advanced").focused() == Some(true) {
                break;
            }
        }
        assert_eq!(window.find("profile-advanced").focused(), Some(true));
        window.press("enter", cx);
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        assert_eq!(window.find("profile-confirm-cancel").focused(), Some(true));
    })
    .unwrap();
    let (entered_tx, entered_rx) = mpsc::channel();
    let job = executor
        .job(
            RequestKey {
                environment: EnvironmentId::Local("close-fixture".into()),
                platform: Platform::Claude,
                generation: 0,
                request_id: 42,
            },
            move |cancel, _| {
                entered_tx.send(()).unwrap();
                tokio::runtime::Handle::current().block_on(cancel.cancelled());
                Completion::<(), ()>::Cancelled
            },
        )
        .unwrap();
    entered_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    let mut visual = VisualTestContext::from_window(handle, cx);
    for cancel in ["escape", "click"] {
        assert!(!visual.simulate_close());
        visual.update(|window, cx| {
            window.render_frame(cx);
            window.simulate_next_frame(cx);
            window.render_frame(cx);
            assert_eq!(window.find("cancel").focused(), Some(true));
            if cancel == "escape" {
                window.press("escape", cx);
            } else {
                window.click("cancel", cx);
            }
            assert!(window.try_find("dialog").is_none());
            assert!(window.try_find("profile-confirmation").is_some());
            assert_eq!(window.find("profile-confirm-cancel").focused(), Some(true));
            assert_eq!(
                window.find("profile-description").value(),
                Some("Unconfirmed draft")
            );
        });
        assert!(!job.is_cancel_requested());
        assert!(job.terminal().is_none());
        assert_eq!(context.manager().snapshot().unwrap().version, revision);
    }
    assert!(!visual.simulate_close());
    visual.update(|window, cx| {
        window.render_frame(cx);
        window.click("ok", cx);
    });
    wait_closed(cx, handle);
    // TestAppContext has no real OS event loop; final join is exercised explicitly.
    owner.0.take().unwrap().shutdown();
    assert!(matches!(
        job.terminal().as_deref(),
        Some(Completion::Cancelled)
    ));
    assert_eq!(executor.active_tasks(), 0);
    assert_eq!(context.manager().snapshot().unwrap().version, revision);
}
