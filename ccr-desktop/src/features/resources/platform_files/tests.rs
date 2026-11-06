#![allow(clippy::unwrap_used)]
use super::*;
use crate::features::resources::ResourcesView;
use ccr_app::runtime::RuntimeOwner;
use gpui_kit::test::{TestAppContextExt, TestWindowExt};
use gpui_kit::{Bounds, TestAppContext, WindowBounds, WindowOptions, size};
use std::time::Duration;
#[gpui_kit::test]
async fn native_file_create_confirm_detach_and_conflict_preserve_original_source(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let services = Services::new(
        owner.handle(),
        crate::runtime::ConfigSource::Fixture {
            profiles: root.path().join("profiles.toml"),
            registry: root.path().join("registry.toml"),
            delay: Duration::ZERO,
        },
    );
    let retained = services.clone();
    let (handle, parent) = cx.update(|cx| {
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
            |window, cx| {
                cx.new(|cx| {
                    let mut view = ResourcesView::new(services, window, cx);
                    view.platform = Platform::OpenCode;
                    view
                })
            },
        )
        .unwrap()
    });
    let view = cx.update(|cx| parent.read(cx).opencode.clone());
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("resource-opencode", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| !view.read(cx).busy)
        .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        view.read(cx)
            .name
            .clone()
            .update(cx, |state, cx| state.set_value("review.md", window, cx));
        window.click("platform-file-new", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        view.read(cx).session.is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        view.read(cx).editor.clone().update(cx, |s, cx| {
            s.set_value(
                "---\ndescription: Review\ncustom: true\n---\n\nNew instructions\n",
                window,
                cx,
            )
        });
        window.click("platform-file-save", cx);
        window.render_frame(cx);
        assert!(window.find("platform-file-confirmation").visible());
        assert!(!retained.platform_files_state().busy);
        window.click("platform-file-confirm", cx);
        parent.update(cx, |v, cx| v.leave(window, cx));
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, _| {
        !retained.platform_files_state().busy
    })
    .await;
    assert_eq!(
        retained.platform_files_state().result,
        Some(Ok(runtime::Outcome::Saved))
    );
    let path = root
        .path()
        .join("desktop-resource-fixture/home/.config/opencode/agents/review.md");
    assert!(
        std::fs::read_to_string(&path)
            .unwrap()
            .contains("custom: true")
    );
    view.update(cx, |v, cx| {
        assert!(v.session.is_none());
        assert!(v.editor.read(cx).value().is_empty());
    });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("resource-opencode", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| !view.read(cx).busy)
        .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click(("platform-file-open", 0usize), cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        view.read(cx).session.is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        view.read(cx).editor.clone().update(cx, |s, cx| {
            s.set_value("---\ndescription: Draft\n---\nDraft content", window, cx)
        });
        std::fs::write(&path, "---\ndescription: External\n---\nExternal content").unwrap();
        window.click("platform-file-save", cx);
        window.render_frame(cx);
        window.click("platform-file-confirm", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        view.read(cx).operation.result == Some(Err(Error::Conflict))
    })
    .await;
    view.update(cx, |v, cx| {
        assert!(v.editor.read(cx).value().contains("Draft content"))
    });
    assert!(
        std::fs::read_to_string(path)
            .unwrap()
            .contains("External content")
    );
    let manifest = root.path().join("desktop-resource-fixture/home/.gemini/antigravity-cli/extensions/fixture/gemini-extension.json");
    std::fs::create_dir_all(manifest.parent().unwrap()).unwrap();
    std::fs::write(
        &manifest,
        r#"{"description":"Fixture extension","version":"1"}"#,
    )
    .unwrap();
    cx.update_window(handle, |_, window, cx| {
        parent.update(cx, |v, cx| {
            v.leave(window, cx);
            v.platform = Platform::Gemini;
        });
        window.render_frame(cx);
        window.click("resource-antigravity", cx);
    })
    .unwrap();
    let antigravity = cx.update(|cx| parent.read(cx).antigravity.clone());
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        !antigravity.read(cx).busy
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click(("platform-file-kind", 1usize), cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        !antigravity.read(cx).busy
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(window.try_find("platform-file-new").is_none());
        window.click(("platform-file-open", 0usize), cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        antigravity.read(cx).session.is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(window.try_find("platform-file-save").is_none());
        assert!(window.try_find("platform-file-delete").is_none());
        window.remove_window();
    })
    .unwrap();
    owner.shutdown();
}
