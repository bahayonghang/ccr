#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;
use crate::runtime::ConfigSource;
use ccr_app::runtime::RuntimeOwner;
use gpui_kit::{TestAppContext, WindowOptions, test::TestWindowExt};
#[gpui_kit::test]
async fn tool_update_review_can_be_cancelled_and_fixture_controls_do_not_launch(
    cx: &mut TestAppContext,
) {
    use gpui_kit::AppContext;
    cx.executor().allow_parking();
    let owner = RuntimeOwner::new().unwrap();
    let root = tempfile::tempdir().unwrap();
    let services = Services::new(
        owner.handle(),
        ConfigSource::Fixture {
            profiles: root.path().join("profiles.toml"),
            registry: root.path().join("config.toml"),
            delay: std::time::Duration::ZERO,
        },
    );
    let (handle, view) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions::default(), cx, |_, cx| {
            cx.new(|cx| ToolsView::new(&services, cx))
        })
        .unwrap()
    });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("tools-versions", cx);
        window.render_frame(cx);
        assert_eq!(view.read(cx).snapshot.status, Status::Idle);
        window.click("tools-review-update", cx);
        window.render_frame(cx);
        window.render_frame(cx);
        assert!(window.try_find("tools-update-confirmation").is_some());
        window.click("tools-update-cancel", cx);
        window.render_frame(cx);
        assert!(window.try_find("tools-update-confirmation").is_none());
        assert_eq!(owner.handle().active_tasks(), 0);
        window.remove_window();
    })
    .unwrap();
    owner.shutdown();
}
