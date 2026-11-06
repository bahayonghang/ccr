#![allow(clippy::unwrap_used)]
use super::*;
use crate::runtime::resources::codex_agents::Outcome;
use ccr_app::runtime::RuntimeOwner;
use gpui_kit::test::{TestAppContextExt, TestWindowExt};
use gpui_kit::{Bounds, TestAppContext, WindowBounds, WindowOptions, size};
use std::time::Duration;

#[gpui_kit::test]
async fn native_create_conflict_and_detached_result_preserve_agent_draft(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    let runtime = RuntimeOwner::new().unwrap();
    let services = Services::new(
        runtime.handle(),
        crate::runtime::ConfigSource::Fixture {
            profiles: root.path().join("profiles.toml"),
            registry: root.path().join("registry.toml"),
            delay: Duration::ZERO,
        },
    );
    let retained = services.clone();
    let (handle, view) = cx.update(|cx| {
        gpui_kit::init(cx);
        cx.set_global(crate::locales::Language::English);
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(1280.), px(1000.)),
                    cx,
                ))),
                ..Default::default()
            },
            cx,
            |window, cx| cx.new(|cx| CodexAgentsView::new(services, window, cx)),
        )
        .unwrap()
    });
    view.update(cx, |v, cx| v.activate(cx));
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| !view.read(cx).busy)
        .await;
    cx.update_window(handle,|_,window,cx|{window.render_frame(cx);window.click("codex-agent-new",cx);window.render_frame(cx);
        view.read(cx).name.clone().update(cx,|state,cx|state.set_value("reviewer",window,cx));
        view.read(cx).editor.clone().update(cx,|state,cx|state.set_value("name='template'\ndescription='Review'\ndeveloper_instructions='Inspect'\ncustom_flag=true\n",window,cx));
        window.click("codex-agent-save",cx);view.update(cx,|v,cx|v.suspend(window,cx));
    }).unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, _| {
        !retained.codex_agents_state().busy
    })
    .await;
    let path = root
        .path()
        .join("desktop-resource-fixture/home/.codex/agents/reviewer.toml");
    assert!(path.exists());
    view.update(cx, |v, cx| {
        assert!(v.session.is_none());
        assert!(v.preview.is_none());
        assert!(v.editor.read(cx).value().is_empty());
    });
    assert_eq!(
        retained.codex_agents_state().result,
        Some(Ok(Outcome::Saved))
    );
    view.update(cx, |v, cx| {
        v.activate(cx);
    });
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| !view.read(cx).busy)
        .await;
    view.update(cx, |v, cx| {
        v.request(Read::Open(Scope::Global, "reviewer".into()), cx)
    });
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        view.read(cx).session.is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        view.read(cx).editor.clone().update(cx, |state, cx| {
            state.set_value(
                "name='reviewer'\ndescription='Draft'\ndeveloper_instructions='Inspect'\n",
                window,
                cx,
            )
        });
        std::fs::write(
            &path,
            "name='reviewer'\ndescription='External'\ndeveloper_instructions='Inspect'\n",
        )
        .unwrap();
        window.click("codex-agent-save", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, _| {
        !retained.codex_agents_state().busy
    })
    .await;
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        view.read(cx).operation.result == Some(Err(runtime::Error::Conflict))
    })
    .await;
    view.update(cx, |v, cx| {
        assert!(v.editing);
        assert!(v.editor.read(cx).value().contains("Draft"));
        assert!(v.blocks_switch());
    });
    cx.run_until_parked();
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        for id in [
            "codex-agent-project",
            "codex-agent-global",
            "codex-agents-sources",
        ] {
            window.click(id, cx);
            let state = view.read(cx);
            assert_eq!(
                state.scope,
                Scope::Global,
                "{id} changed an active editor's scope"
            );
            assert!(
                !state.sources_mode,
                "{id} changed an active editor's source"
            );
            assert!(state.editing);
            assert_eq!(state.session.as_ref().unwrap().scope, Scope::Global);
            assert!(state.editor.read(cx).value().contains("Draft"));
        }
        window.click("codex-agent-discard", cx);
        view.read(cx).project.clone().update(cx, |state, cx| {
            state.set_value(
                root.path()
                    .join("desktop-resource-fixture/home/missing-project")
                    .to_string_lossy()
                    .into_owned(),
                window,
                cx,
            )
        });
        window.click("codex-agent-project", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| !view.read(cx).busy)
        .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(view.read(cx).message.is_some());
        assert!(view.read(cx).session.is_none());
        assert!(
            window.try_find(("codex-agent-open", 0usize)).is_none(),
            "failed project load must not leave Global rows actionable"
        );
    })
    .unwrap();
    assert!(std::fs::read_to_string(path).unwrap().contains("External"));
    cx.update_window(handle, |_, window, _| window.remove_window())
        .unwrap();
    runtime.shutdown();
}
