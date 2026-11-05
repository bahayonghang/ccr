use super::*;
use ccr_app::runtime::RuntimeOwner;
use gpui_kit::test::TestWindowExt;
use gpui_kit::{TestAppContext, WindowOptions};

#[gpui_kit::test]
fn rendered_catalog_blocks_typed_only_commands_and_invalid_draft_never_starts(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let fixture = tempfile::tempdir().unwrap_or_else(|_| panic!("fixture"));
    let owner = RuntimeOwner::new().unwrap_or_else(|_| panic!("runtime"));
    let services = Services::new(
        owner.handle(),
        crate::runtime::ConfigSource::Fixture {
            profiles: fixture.path().join("profiles.toml"),
            registry: fixture.path().join("config.toml"),
            delay: std::time::Duration::ZERO,
        },
    );
    let (handle, view) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| JobsView::new(services, window, cx))
        })
        .unwrap_or_else(|_| panic!("window"))
    });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("job-command-delete", cx);
        assert_eq!(
            view.read(cx).selected,
            "status",
            "typed destructive paths remain disabled in generic passthrough"
        );
        view.read(cx).args.clone().update(cx, |state, cx| {
            state.set_value("not-json-secret-sentinel", window, cx)
        });
        window.render_frame(cx);
        window.click("command-start", cx);
        assert!(view.read(cx).job_id.is_none());
        assert!(view.read(cx).snapshot.is_none());
        assert!(!view.read(cx).busy);
        assert!(
            !view
                .read(cx)
                .message
                .unwrap_or_default()
                .contains("secret-sentinel")
        );
        assert_eq!(
            view.read(cx).args.read(cx).value().as_str(),
            "not-json-secret-sentinel"
        );
    })
    .unwrap_or_else(|_| panic!("controls"));
    assert!(
        fixture
            .path()
            .read_dir()
            .unwrap_or_else(|_| panic!("read fixture"))
            .next()
            .is_none()
    );
    owner.shutdown();
}
