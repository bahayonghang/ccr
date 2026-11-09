//! Agent sessions page: filters, split pane, drill-in and list semantics.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use crate::features::insights::{InsightsPage, InsightsView, Surface};
use crate::runtime::insights::{InsightsPaths, InsightsSnapshot};
use crate::runtime::{ConfigSource, Services};
use crate::shell::Shell;
use crate::theme;
use ccr_app::runtime::{Platform, RuntimeOwner};
use ccr_app::sessions::SessionPage;
use ccr_db::database::repositories::usage_repo::{AgentSessionArchiveRow, UsageSourceState};
use ccr_store::sessions::{AgentSessionAgentId, AgentSessionProviderStatus, ProviderAvailability};
use gpui_kit::test::{TestAppContextExt, TestWindowExt};
use gpui_kit::{
    AnyWindowHandle, AppContext, Entity, SharedString, TestAppContext, WindowBounds, WindowOptions,
    px, size,
};
use std::sync::Arc;
use std::time::Duration;

/// Services whose archive does not exist: list reads fail, so tests inject snapshots.
fn empty_services(owner: &RuntimeOwner, root: &std::path::Path) -> Services {
    Services::new(
        owner.handle(),
        ConfigSource::Fixture {
            profiles: root.join("profiles.toml"),
            registry: root.join("config.toml"),
            delay: Duration::ZERO,
        },
    )
    .with_insights_paths(InsightsPaths {
        usage_root: root.join("llmusage"),
        archive_db: root.join("archive.db"),
        provider_home: root.join("home"),
    })
}

fn sized(width: f32, height: f32) -> WindowOptions {
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(gpui_kit::Bounds::new(
            gpui_kit::point(px(0.), px(0.)),
            size(px(width), px(height)),
        ))),
        ..Default::default()
    }
}

fn open_view(
    cx: &mut TestAppContext,
    services: Services,
    width: f32,
    height: f32,
) -> (AnyWindowHandle, Entity<InsightsView>) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        let (handle, view) = gpui_kit::open_window(sized(width, height), cx, |window, cx| {
            cx.new(|cx| InsightsView::new(services, window, cx))
        })
        .unwrap();
        (handle, view)
    })
}

/// The full shell at a window size; returns its Insights entity for state checks.
fn open_shell(
    cx: &mut TestAppContext,
    services: Services,
    width: f32,
    height: f32,
) -> (AnyWindowHandle, Entity<InsightsView>) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        let (handle, shell) = gpui_kit::open_window(sized(width, height), cx, |window, cx| {
            cx.new(|cx| Shell::new(services, window, cx))
        })
        .unwrap();
        let insights = shell.read(cx).insights_view();
        (handle, insights)
    })
}

fn row(index: usize, title: &str) -> AgentSessionArchiveRow {
    let now = chrono::Utc::now();
    AgentSessionArchiveRow {
        archive_id: format!("as-test-{index:03}"),
        session_id: format!("session-{index}"),
        platform: "claude".into(),
        source_variant: "claude-code".into(),
        title: Some(title.into()),
        cwd: "D:\\Documents\\research\\thesis".into(),
        message_count: 272,
        user_message_count: 100,
        assistant_message_count: 172,
        tool_use_count: 18,
        created_at: now,
        updated_at: now,
        source_state: UsageSourceState::Live,
        source_fidelity: "partial".into(),
    }
}

fn status(
    agent: AgentSessionAgentId,
    availability: ProviderAvailability,
    sources: usize,
    error: Option<&str>,
) -> AgentSessionProviderStatus {
    AgentSessionProviderStatus {
        agent,
        availability,
        variants: Vec::new(),
        source_count: sources,
        fidelity: None,
        error_category: error.map(str::to_string),
    }
}

fn statuses() -> Vec<AgentSessionProviderStatus> {
    vec![
        status(
            AgentSessionAgentId::Grok,
            ProviderAvailability::NotInstalled,
            0,
            None,
        ),
        status(
            AgentSessionAgentId::Claude,
            ProviderAvailability::NoData,
            0,
            None,
        ),
        status(
            AgentSessionAgentId::Codex,
            ProviderAvailability::Available,
            3,
            None,
        ),
        status(
            AgentSessionAgentId::Kimi,
            ProviderAvailability::Error,
            0,
            Some("permission_denied"),
        ),
    ]
}

/// 直接放入一页列表快照；不发起读取。
fn inject(
    view: &mut InsightsView,
    items: Vec<AgentSessionArchiveRow>,
    statuses: Vec<AgentSessionProviderStatus>,
    next_cursor: Option<String>,
) {
    view.surface = Surface::Sessions;
    view.page = InsightsPage::Sessions;
    view.snapshot = Some(Arc::new(InsightsSnapshot::Sessions {
        page: SessionPage { items, next_cursor },
        statuses,
        indexed: true,
    }));
}

fn requested_agents(
    view: &Entity<InsightsView>,
    cx: &mut gpui_kit::App,
) -> Option<Vec<AgentSessionAgentId>> {
    view.update(cx, |this, cx| this.session_list_request(cx).unwrap().agents)
}

#[gpui_kit::test]
fn provider_chips_toggle_several_agents_into_the_request(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let owner = RuntimeOwner::new().unwrap();
    let root = tempfile::tempdir().unwrap();
    let (handle, view) = open_view(cx, empty_services(&owner, root.path()), 1280., 800.);
    view.update(cx, |this, cx| {
        inject(this, vec![row(0, "Fixture")], statuses(), None);
        cx.notify();
    });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(requested_agents(&view, cx), None);
        assert_eq!(window.find("session-provider-codex").checked(), Some(false));
        assert_eq!(
            window.find("session-provider-codex").label(),
            Some("Codex · 3 sources")
        );
        window.click("session-provider-codex", cx);
        assert_eq!(
            requested_agents(&view, cx),
            Some(vec![AgentSessionAgentId::Codex])
        );
        window.render_frame(cx);
        assert_eq!(window.find("session-provider-codex").checked(), Some(true));
        // 无文件的提供方仍可选，且与已选的并存：多选。
        window.click("session-provider-claude", cx);
        assert_eq!(
            requested_agents(&view, cx),
            Some(vec![
                AgentSessionAgentId::Claude,
                AgentSessionAgentId::Codex
            ])
        );
        window.render_frame(cx);
        window.click("session-provider-codex", cx);
        assert_eq!(
            requested_agents(&view, cx),
            Some(vec![AgentSessionAgentId::Claude])
        );
        window.remove_window();
    })
    .unwrap();
    owner.shutdown();
}

#[gpui_kit::test]
fn not_installed_and_unreadable_provider_chips_cannot_be_selected(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let owner = RuntimeOwner::new().unwrap();
    let root = tempfile::tempdir().unwrap();
    let (handle, view) = open_view(cx, empty_services(&owner, root.path()), 1280., 800.);
    view.update(cx, |this, cx| {
        inject(this, vec![row(0, "Fixture")], statuses(), None);
        cx.notify();
    });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        let generation = view.read(cx).generation;
        assert_eq!(
            window.find("session-provider-grok").label(),
            Some("Grok · not installed")
        );
        assert_eq!(
            window.find("session-provider-kimi").label(),
            Some("Kimi · unreadable")
        );
        // 禁用的 chip 没有焦点绑定，激活也不会改筛选或发起读取。
        for id in ["session-provider-grok", "session-provider-kimi"] {
            assert_eq!(window.find(id).focused(), None);
            assert_eq!(window.find(id).checked(), Some(false));
            window.click(id, cx);
        }
        assert_eq!(view.read(cx).generation, generation);
        assert_eq!(requested_agents(&view, cx), None);
        assert_eq!(window.find("session-provider-codex").focused(), Some(false));
        window.remove_window();
    })
    .unwrap();
    owner.shutdown();
}

#[gpui_kit::test]
fn platform_scope_locks_the_provider_and_states_it_in_one_line(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let owner = RuntimeOwner::new().unwrap();
    let root = tempfile::tempdir().unwrap();
    let (handle, view) = open_view(cx, empty_services(&owner, root.path()), 1280., 800.);
    cx.update_window(handle, |_, window, cx| {
        view.update(cx, |this, cx| {
            this.open_platform(InsightsPage::Sessions, Platform::Claude, window, cx);
            inject(this, vec![row(0, "Fixture")], statuses(), None);
            cx.notify();
        });
        window.render_frame(cx);
        assert!(window.try_find("session-provider-codex").is_none());
        assert!(window.try_find("session-providers").is_none());
        assert_eq!(
            window.find("session-providers-locked").label(),
            Some("Showing Claude sessions")
        );
        assert_eq!(
            requested_agents(&view, cx),
            Some(vec![AgentSessionAgentId::Claude])
        );
        view.update(cx, |this, cx| {
            this.toggle_session_agent(AgentSessionAgentId::Codex, cx)
        });
        assert_eq!(
            requested_agents(&view, cx),
            Some(vec![AgentSessionAgentId::Claude])
        );
        // 回到全局页：锁定解除，chip 重新出现。
        view.update(cx, |this, cx| {
            this.open_global(InsightsPage::Sessions, window, cx);
            inject(this, vec![row(0, "Fixture")], statuses(), None);
            cx.notify();
        });
        window.render_frame(cx);
        assert!(window.try_find("session-providers-locked").is_none());
        assert!(window.try_find("session-provider-codex").is_some());
        assert_eq!(requested_agents(&view, cx), None);
        window.remove_window();
    })
    .unwrap();
    owner.shutdown();
}

#[gpui_kit::test]
fn long_titles_stay_on_one_line_inside_the_row(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let owner = RuntimeOwner::new().unwrap();
    let root = tempfile::tempdir().unwrap();
    let (handle, view) = open_view(cx, empty_services(&owner, root.path()), 1280., 800.);
    let long = "Investigate why the incremental archive refresh skips restored sessions ".repeat(6);
    // 第 0 行带很长的项目路径，第 1 行没有路径；两行的摘要文字相同。
    let mut deep = row(0, &long);
    deep.cwd = r"D:\Documents\LYH\research\thesis\chapters\".repeat(8);
    let mut bare = row(1, &long);
    bare.cwd = String::new();
    view.update(cx, |this, cx| {
        inject(this, vec![deep, bare], statuses(), None);
        cx.notify();
    });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        let row = window.find("insights-row:as-test-000").bounds();
        let title = window.find("session-row-title:as-test-000").bounds();
        assert_eq!(row.size.height, px(theme::TWO_LINE_ROW_HEIGHT));
        assert!(
            title.size.width <= row.size.width,
            "title {title:?} wider than row {row:?}"
        );
        assert!(title.right() <= row.right());
        // 单行：标题高度就是一行 20px，不会折成两行把第二行挤出 60px 的行。
        assert!(title.size.height <= px(20.), "title wraps: {title:?}");
        // 第二行：摘要保持自然宽度（与无路径的行一样宽），只有路径被省略。
        let meta = window.find("session-row-meta:as-test-000").bounds();
        let natural = window.find("session-row-meta:as-test-001").bounds();
        assert_eq!(meta.size.width, natural.size.width, "metadata compressed");
        let cwd = window.find("session-row-cwd:as-test-000").bounds();
        assert!(cwd.left() >= meta.right(), "path overlaps metadata");
        assert!(cwd.right() <= row.right(), "path overflows the row");
        assert!(window.try_find("session-row-cwd:as-test-001").is_none());
        window.remove_window();
    })
    .unwrap();
    owner.shutdown();
}

#[gpui_kit::test]
fn paging_bar_follows_the_next_cursor_and_the_first_page(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let owner = RuntimeOwner::new().unwrap();
    let root = tempfile::tempdir().unwrap();
    let (handle, view) = open_view(cx, empty_services(&owner, root.path()), 1280., 800.);
    view.update(cx, |this, cx| {
        inject(
            this,
            vec![row(0, "First")],
            statuses(),
            Some("cursor".into()),
        );
        cx.notify();
    });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(window.find("insights-next").visible());
        assert!(window.try_find("session-first-page").is_none());
        assert!(window.find("session-page-count").bounds().size.width > px(0.));
        view.update(cx, |this, cx| {
            inject(this, vec![row(0, "Last")], statuses(), None);
            cx.notify();
        });
        window.render_frame(cx);
        assert!(window.try_find("insights-next").is_none());
        assert!(window.try_find("session-paging").is_none());
        // 不在第一页时可以回到最新一页。
        view.update(cx, |this, cx| {
            this.cursor = Some("cursor".into());
            cx.notify();
        });
        window.render_frame(cx);
        assert!(window.find("session-first-page").visible());
        assert!(window.try_find("insights-next").is_none());
        window.remove_window();
    })
    .unwrap();
    owner.shutdown();
}

#[gpui_kit::test]
fn search_typing_waits_for_the_debounce_and_enter_reads_at_once(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let owner = RuntimeOwner::new().unwrap();
    let root = tempfile::tempdir().unwrap();
    let (handle, view) = open_view(cx, empty_services(&owner, root.path()), 1280., 800.);
    view.update(cx, |this, cx| {
        inject(this, vec![row(0, "Fixture")], statuses(), None);
        cx.notify();
    });
    let generation = cx
        .update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.click("insights-filter-6", cx);
            window.input("arc", cx);
            view.read(cx).generation
        })
        .unwrap();
    cx.executor()
        .advance_clock(super::state::SEARCH_DEBOUNCE - Duration::from_millis(1));
    cx.run_until_parked();
    assert_eq!(view.read_with(cx, |this, _| this.generation), generation);
    cx.executor().advance_clock(Duration::from_millis(1));
    cx.run_until_parked();
    // 连续三个字符只在最后一次输入后查询一次。
    assert_eq!(
        view.read_with(cx, |this, _| this.generation),
        generation + 1
    );
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("insights-filter-6", cx);
        window.input("h", cx);
    })
    .unwrap();
    cx.run_until_parked();
    // 输入已经排上防抖，但时钟没走：此刻还没有查询。
    let before = view.read_with(cx, |this, _| this.generation);
    cx.update_window(handle, |_, window, cx| window.press("enter", cx))
        .unwrap();
    cx.run_until_parked();
    // Enter 不等防抖，立即查询一次。
    let generation = before + 1;
    assert_eq!(view.read_with(cx, |this, _| this.generation), generation);
    // Enter 让未到期的防抖 token 过期，到期后不再查询。
    cx.executor()
        .advance_clock(super::state::SEARCH_DEBOUNCE * 2);
    cx.run_until_parked();
    assert_eq!(view.read_with(cx, |this, _| this.generation), generation);
    let query =
        cx.update(|cx| view.update(cx, |this, cx| this.session_list_request(cx).unwrap().query));
    assert_eq!(query.as_deref(), Some("arch"));
    cx.update_window(handle, |_, window, _| window.remove_window())
        .unwrap();
    owner.shutdown();
}

/// 真实归档 + 一个不存在的行：快速连续选择只显示后一个；列表刷新与筛选丢弃在途详情。
#[gpui_kit::test]
async fn detail_slot_keeps_only_the_latest_selection_and_list_changes_drop_it(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let (_root, owner, services, _, id) =
        crate::runtime::insights::session_operations::tests::fixture();
    let (handle, view) = open_view(cx, services, 1280., 800.);
    view.update(cx, |this, cx| this.open_product(InsightsPage::Sessions, cx));
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        !view.read(cx).loading
    })
    .await;
    // 在真实行之后追加一个归档里不存在的行。
    view.update(cx, |this, cx| {
        let (items, statuses) = match this.snapshot.as_deref() {
            Some(InsightsSnapshot::Sessions { page, statuses, .. }) => {
                (page.items.clone(), statuses.clone())
            }
            _ => panic!("sessions snapshot"),
        };
        let mut items = items;
        let mut missing = items[0].clone();
        missing.archive_id = "as-missing-fixture".into();
        items.push(missing);
        inject(this, items, statuses, None);
        cx.notify();
    });
    // 先选不存在的行，再选真实行：只显示真实行的详情。
    view.update(cx, |this, cx| {
        this.select(1, cx);
        this.select(0, cx);
    });
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        !view.read(cx).detail.loading
    })
    .await;
    std::thread::sleep(Duration::from_millis(100));
    cx.run_until_parked();
    view.read_with(cx, |this, _| {
        assert_eq!(this.detail.archive_id.as_deref(), Some(id.as_str()));
        assert_eq!(
            this.session_detail()
                .map(|detail| detail.archive_id.as_str()),
            Some(id.as_str())
        );
        assert!(this.detail.error.is_none());
        // 打开详情不清列表快照。
        assert!(matches!(
            this.snapshot.as_deref(),
            Some(InsightsSnapshot::Sessions { .. })
        ));
    });
    // 反过来：先真实行、再不存在的行，晚到的真实详情不得出现。
    view.update(cx, |this, cx| {
        this.select(0, cx);
        this.select(1, cx);
    });
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        !view.read(cx).detail.loading
    })
    .await;
    std::thread::sleep(Duration::from_millis(100));
    cx.run_until_parked();
    view.read_with(cx, |this, _| {
        assert_eq!(
            this.detail.archive_id.as_deref(),
            Some("as-missing-fixture")
        );
        assert!(this.session_detail().is_none());
        assert!(this.detail.error.is_some());
    });
    // 列表翻页保留详情槽。
    view.update(cx, |this, cx| {
        this.select(0, cx);
    });
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        view.read(cx).session_detail().is_some()
    })
    .await;
    view.update(cx, |this, cx| {
        let items = match this.snapshot.as_deref() {
            Some(InsightsSnapshot::Sessions { page, .. }) => page.items.clone(),
            _ => panic!("sessions snapshot"),
        };
        inject(this, items, statuses(), Some("next".into()));
        this.next(cx);
        assert_eq!(this.cursor.as_deref(), Some("next"));
        assert!(this.session_detail().is_some());
    });
    // 在途详情遇到列表刷新：详情槽清空，晚到的响应被丢弃。
    view.update(cx, |this, cx| {
        this.select(0, cx);
        assert!(this.detail.loading);
        this.refresh(cx);
        assert!(this.detail.archive_id.is_none());
        assert!(this.detail.task.is_none());
        assert!(!this.detail.loading);
    });
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        !view.read(cx).loading
    })
    .await;
    std::thread::sleep(Duration::from_millis(100));
    cx.run_until_parked();
    view.read_with(cx, |this, _| {
        assert!(this.session_detail().is_none());
        assert!(this.detail.archive_id.is_none());
    });
    // 筛选变化同样关闭详情。
    view.update(cx, |this, cx| {
        this.select(0, cx);
        this.set_session_fidelity(Some("partial"), cx);
        assert!(this.detail.archive_id.is_none());
        assert_eq!(
            this.session_list_request(cx).unwrap().fidelity.as_deref(),
            Some("partial")
        );
    });
    cx.update_window(handle, |_, window, _| window.remove_window())
        .unwrap();
    owner.shutdown();
}

/// 宽窗打开会话后列表仍在；列表与 pane 按 3:2 并排。
#[gpui_kit::test]
async fn wide_windows_keep_the_list_beside_the_opened_session(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let (_root, owner, services, _, id) =
        crate::runtime::insights::session_operations::tests::fixture();
    let row = SharedString::from(format!("insights-row:{id}"));
    for (width, height) in [(1280., 800.), (2000., 1080.)] {
        let (handle, view) = open_shell(cx, services.clone(), width, height);
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.click("nav-agent-sessions", cx);
        })
        .unwrap();
        let wanted = row.clone();
        cx.wait_for(handle, Duration::from_secs(10), move |window, _| {
            window.try_find(wanted.clone()).is_some()
        })
        .await;
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            // 未打开会话：空态 pane 与列表同一列宽规则。
            let list = window.find("session-list").bounds();
            let empty = window.find("session-pane-empty").bounds();
            let body = window.find("session-body").bounds();
            assert!(list.bottom() < body.bottom(), "few rows must hug content");
            assert!(
                empty.bottom() < body.bottom(),
                "empty pane must hug content"
            );
            let ratio = f32::from(list.size.width) / f32::from(empty.size.width);
            assert!((ratio - 1.5).abs() < 0.01, "list/empty ratio {ratio}");
            window.click(row.clone(), cx);
        })
        .unwrap();
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            view.read(cx).session_detail().is_some()
        })
        .await;
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            let list = window.find("session-list").bounds();
            let pane = window.find("session-pane").bounds();
            println!(
                "{width}x{height}: session-list {} wide, session-pane {} wide",
                f32::from(list.size.width),
                f32::from(pane.size.width)
            );
            assert_eq!(list.top(), pane.top());
            assert_eq!(pane.left() - list.right(), px(theme::SECTION_GAP));
            assert!(list.size.width >= px(theme::LIST_MIN_WIDTH));
            assert!(pane.size.width >= px(theme::DETAIL_MIN_WIDTH));
            let ratio = f32::from(list.size.width) / f32::from(pane.size.width);
            assert!((ratio - 1.5).abs() < 0.01, "list/pane ratio {ratio}");
            assert!(window.find(row.clone()).visible());
            assert!(window.try_find("insights-back").is_none());
            assert!(window.find("session-transcript").visible());
            let variant = window
                .find("session-detail-variant")
                .label()
                .unwrap()
                .to_string();
            assert_eq!(variant, "Codex · codex-live");
            let counts = window
                .find("session-detail-counts")
                .label()
                .unwrap()
                .to_string();
            assert!(counts.contains("messages"), "{counts}");
            assert!(
                !counts.contains("Codex") && !counts.contains("codex-live"),
                "detail counts repeat the tag: {counts}"
            );
            // Codex 操作在 pane 头部，决策控件不落在折叠线以下。
            for id in ["session-export", "session-detail-title"] {
                let node = window.find(id).bounds();
                assert!(node.left() >= pane.left() && node.right() <= pane.right());
                assert!(node.bottom() <= px(height), "{id} below the fold");
            }
            let page = window.find("session-page").bounds();
            assert!(list.left() >= page.left() && pane.right() <= page.right());
            window.remove_window();
        })
        .unwrap();
    }
    owner.shutdown();
}

/// 窄窗：打开会话即 drill-in，焦点进入 pane；返回后焦点回到列表；离页后延迟回调不抢焦点。
#[gpui_kit::test]
async fn narrow_windows_drill_in_and_return_to_the_list(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let (_root, owner, services, _, id) =
        crate::runtime::insights::session_operations::tests::fixture();
    let row = SharedString::from(format!("insights-row:{id}"));
    let (handle, view) = open_shell(cx, services, 960., 640.);
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("nav-agent-sessions", cx);
    })
    .unwrap();
    let wanted = row.clone();
    cx.wait_for(handle, Duration::from_secs(10), move |window, _| {
        window.try_find(wanted.clone()).is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(
            window.find("session-cards").bounds().top() < px(640. * 0.45),
            "first row must be above 45% height"
        );
        assert!(window.try_find("session-pane").is_none());
        assert!(window.try_find("session-pane-empty").is_none());
        window.click(row.clone(), cx);
        window.render_frame(cx);
        assert!(window.try_find("session-list").is_none());
        assert!(window.find("session-pane").visible());
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        assert_eq!(window.find("insights-back").focused(), Some(true));
        // 只筛选隐藏列表的控件不占详情的高度。
        assert!(window.try_find("session-providers").is_none());
        assert!(window.try_find("insights-filter-6").is_none());
        assert!(window.try_find("session-toolbar").is_none());
        assert!(window.find("session-title").visible());
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        view.read(cx).session_detail().is_some()
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        let pane = window.find("session-pane").bounds();
        let page = window.find("session-page").bounds();
        assert!(pane.right() <= page.right(), "no horizontal overflow");
        window.click("insights-back", cx);
        window.render_frame(cx);
        assert!(window.try_find("session-pane").is_none());
        assert!(window.find(row.clone()).visible());
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        assert_eq!(window.find("session-listbox").focused(), Some(true));
        assert_eq!(view.read(cx).selected.as_deref(), Some(id.as_str()));
        assert!(window.find("session-providers").visible());
        assert!(window.find("insights-filter-6").visible());
        // 键盘：Enter 打开当前选中行。
        window.press("enter", cx);
        window.render_frame(cx);
        assert!(window.find("session-pane").visible());
        window.click("insights-back", cx);
        window.render_frame(cx);
        // 离页后，延迟的 drill-in 焦点回调不抢走侧栏焦点。
        window.click(row.clone(), cx);
        window.click("nav-checkin", cx);
        assert_eq!(window.find("nav-checkin").focused(), Some(true));
        window.simulate_next_frame(cx);
        window.render_frame(cx);
        assert_eq!(window.find("nav-checkin").focused(), Some(true));
        window.remove_window();
    })
    .unwrap();
    owner.shutdown();
}

/// 废纸篓：宽窗占 pane、窄窗在列表上方；恢复确认在长列表之前，焦点与主按钮都在视口内。
#[gpui_kit::test]
async fn trash_decisions_stay_above_the_long_list_in_the_split_pane(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let (root, owner, services, _, _) =
        crate::runtime::insights::session_operations::tests::fixture();
    let trash =
        ccr_codex::CodexSessionTrashService::with_codex_home(root.path().join("home/.codex"));
    ccr_core::core::LockManager::with_scoped_directory(&root.path().join("locks"), || {
        trash.trash_sessions(["synthetic-session"])
    })
    .unwrap();
    for (width, height) in [(1280., 800.), (960., 640.)] {
        let (handle, view) = open_shell(cx, services.clone(), width, height);
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.click("nav-agent-sessions", cx);
        })
        .unwrap();
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            !view.read(cx).loading
        })
        .await;
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.click("session-open-trash", cx);
        })
        .unwrap();
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            view.read(cx).session_trash_loaded()
        })
        .await;
        // 长废纸篓：一页 6 行都渲染出来，确认区仍须在它们之前。
        view.update(cx, |this, cx| {
            this.repeat_session_trash(10_000);
            cx.notify();
        });
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            assert!(window.try_find("session-restore:5").is_some());
            assert!(window.find("session-trash-panel").visible());
            assert!(window.find("insights-filter-6").visible());
            assert!(window.find("session-trash-close").visible());
            if width >= theme::SPLIT_WIDTH {
                // 宽窗：废纸篓占 pane，列表在左。
                let list = window.find("session-list").bounds();
                let pane = window.find("session-pane").bounds();
                assert!(pane.left() > list.right());
            } else {
                // 窄窗：废纸篓在列表上方。
                let pane = window.find("session-pane").bounds();
                let list = window.find("session-list").bounds();
                assert!(pane.bottom() <= list.top());
            }
            window.click("session-restore:0", cx);
        })
        .unwrap();
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            view.read(cx).session_restore_pending()
        })
        .await;
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.simulate_next_frame(cx);
            window.render_frame(cx);
            let confirm = window.find("session-restore-confirm").bounds();
            let first = window.find("session-restore:0").bounds();
            assert!(confirm.bottom() <= first.top(), "decision below the list");
            if width >= theme::SPLIT_WIDTH {
                assert!(confirm.bottom() <= px(height), "confirm below the fold");
                assert_eq!(window.find("session-restore-cancel").focused(), Some(true));
            }
            window.press("escape", cx);
            window.render_frame(cx);
            assert!(window.try_find("session-restore-confirm").is_none());
            window.click("session-trash-close", cx);
            window.render_frame(cx);
            assert!(window.try_find("session-trash-panel").is_none());
            assert!(!view.read(cx).session_ops_trash_open());
            window.remove_window();
        })
        .unwrap();
    }
    owner.shutdown();
}

/// 读取失败后保留最近提供方状态，筛选仍可操作，但不冒充已刷新列表。
#[gpui_kit::test]
async fn failed_list_read_keeps_provider_filters_and_codex_trash_visibility(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let owner = RuntimeOwner::new().unwrap();
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("archive.db"), b"invalid sqlite fixture").unwrap();
    let (handle, view) = open_view(cx, empty_services(&owner, root.path()), 1280., 800.);
    view.update(cx, |this, cx| {
        inject(this, vec![row(0, "Fixture")], statuses(), None);
        this.agents.insert(AgentSessionAgentId::Codex);
        this.submit(cx);
    });
    cx.wait_for(handle, Duration::from_secs(5), |_, cx| {
        !view.read(cx).loading
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(view.read(cx).error.is_some());
        assert_eq!(window.find("session-page-count").label(), Some(""));
        assert_eq!(window.find("session-provider-codex").checked(), Some(true));
        assert!(window.find("session-list-empty").visible());
        assert!(window.try_find("session-cards").is_none());
        assert!(window.try_find("session-pane-empty").is_none());
        assert!(window.find("session-open-trash").visible());
        view.update(cx, |this, cx| {
            inject(
                this,
                vec![row(0, "Fixture")],
                vec![status(
                    AgentSessionAgentId::Codex,
                    ProviderAvailability::NotInstalled,
                    0,
                    None,
                )],
                None,
            );
            this.error = None;
            cx.notify();
        });
        window.render_frame(cx);
        assert!(window.try_find("session-open-trash").is_none());
        window.remove_window();
    })
    .unwrap();
    owner.shutdown();
}

/// 多行时列表被可用高度约束，分页栏仍位于面板内。
#[gpui_kit::test]
fn many_session_rows_scroll_inside_the_content_sized_panel(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let owner = RuntimeOwner::new().unwrap();
    let root = tempfile::tempdir().unwrap();
    let (handle, view) = open_view(cx, empty_services(&owner, root.path()), 2000., 1080.);
    view.update(cx, |this, cx| {
        inject(
            this,
            (0..100).map(|i| row(i, "Fixture")).collect(),
            statuses(),
            Some("cursor".into()),
        );
        cx.notify();
    });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        let list = window.find("session-list").bounds();
        let body = window.find("session-body").bounds();
        assert!(list.bottom() <= body.bottom());
        assert!(list.size.height < px(100. * theme::TWO_LINE_ROW_HEIGHT));
        assert!(window.find("session-paging").bounds().bottom() <= list.bottom());
        window.remove_window();
    })
    .unwrap();
    owner.shutdown();
}
