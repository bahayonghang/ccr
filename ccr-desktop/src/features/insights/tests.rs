#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use crate::runtime::{ConfigSource, insights::InsightsPaths};
use ccr_app::runtime::RuntimeOwner;
use ccr_db::database::{
    create_pool,
    repositories::usage_repo::{AgentSessionArchiveRow, UsageSourceState},
};
use ccr_usage::fixtures::{SeedBucket, create_projection_db, seed_bucket};
use gpui_kit::{
    TestAppContext, WindowOptions,
    test::{TestAppContextExt, TestWindowExt},
};
use std::time::Duration;

fn services(owner: &RuntimeOwner, root: &std::path::Path) -> Services {
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
#[gpui_kit::test]
async fn rendered_usage_controls_read_fixture_filters_and_keep_exact_costs(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let owner = RuntimeOwner::new().unwrap();
    let root = tempfile::tempdir().unwrap();
    let paths = create_projection_db(&root.path().join("llmusage"));
    let pool = create_pool(&paths.db_path, None).unwrap();
    seed_bucket(
        &pool.get().unwrap(),
        &SeedBucket {
            cost_with_cache_usd: 0.123456789,
            pricing_status: "static".into(),
            pricing_source: Some("static-v1".into()),
            ..Default::default()
        },
    );
    drop(pool);
    let before = std::fs::read(&paths.db_path).unwrap();
    let (handle, view) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| InsightsView::new(services(&owner, root.path()), window, cx))
        })
        .unwrap()
    });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("insights-usage", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        !view.read(cx).loading
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        assert!(view.read(cx).error.is_none());
        assert!(view.read(cx).summary().contains("100"));
        window.render_frame(cx);
        window.click("insights-section-Models", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        !view.read(cx).loading
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("insights-row:gpt-5", cx);
        window.render_frame(cx);
        assert!(window.find("insights-exact-detail").visible());
        assert!(view.read(cx).details(0).join(" ").contains("0.123456789"));
        assert!(view.read(cx).details(0).join(" ").contains("static-v1"));
        view.update(cx, |this, cx| {
            this.fields[2].update(cx, |state, cx| {
                state.set_value("not-this-provider", window, cx)
            });
            this.fields[4].update(cx, |state, cx| state.set_value("2026-07-01", window, cx));
            this.fields[5].update(cx, |state, cx| state.set_value("2026-07-01", window, cx));
        });
        window.click("insights-refresh", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        !view.read(cx).loading
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        assert!(view.read(cx).error.is_none());
        assert_eq!(view.read(cx).row_count(), 0);
        view.update(cx, |this, cx| {
            this.fields[2].update(cx, |state, cx| state.set_value("openai", window, cx));
        });
        window.render_frame(cx);
        window.click("insights-refresh", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        !view.read(cx).loading
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        assert_eq!(view.read(cx).row_count(), 1);
        assert_eq!(view.read(cx).row(0).unwrap().left, "100");
        view.update(cx, |this, cx| {
            this.suspend(cx);
            assert!(this.snapshot.is_none());
            assert!(this.selected.is_none());
            assert!(!this.detail_open);
        });
        window.remove_window();
    })
    .unwrap();
    assert_eq!(before, std::fs::read(paths.db_path).unwrap());
    owner.shutdown();
}
#[gpui_kit::test]
fn archive_candidate_rows_are_virtualized_and_keyboard_selection_uses_opaque_ids(
    cx: &mut TestAppContext,
) {
    cx.executor().allow_parking();
    let owner = RuntimeOwner::new().unwrap();
    let root = tempfile::tempdir().unwrap();
    let now = chrono::Utc::now();
    let rows = (0..10_000)
        .map(|i| AgentSessionArchiveRow {
            archive_id: format!("as-fixture-{i:05}"),
            session_id: format!("session-{i}"),
            platform: if i % 2 == 0 { "pi" } else { "omp" }.into(),
            source_variant: "fixture".into(),
            title: Some(format!("Archive fixture {i}")),
            cwd: String::new(),
            message_count: i,
            user_message_count: i,
            assistant_message_count: 0,
            tool_use_count: 0,
            created_at: now,
            updated_at: now,
            source_state: UsageSourceState::Live,
            source_fidelity: "partial".into(),
        })
        .collect();
    let (handle, view) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| {
                let mut view = InsightsView::new(services(&owner, root.path()), window, cx);
                view.page = InsightsPage::Sessions;
                view.snapshot = Some(Arc::new(InsightsSnapshot::Sessions {
                    page: ccr_app::sessions::SessionPage {
                        items: rows,
                        next_cursor: None,
                    },
                    statuses: Vec::new(),
                }));
                view
            })
        })
        .unwrap()
    });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        assert!(view.read(cx).rendered_rows > 0 && view.read(cx).rendered_rows < 100);
        view.update(cx, |this, cx| this.focus_main(window, cx));
        window.press("down", cx);
        window.press("down", cx);
        assert_eq!(view.read(cx).selected.as_deref(), Some("as-fixture-00001"));
        assert!(view.read(cx).row(1).unwrap().label.starts_with("omp"));
        window.press("escape", cx);
        assert!(view.read(cx).selected.is_none());
        view.update(cx, |this, cx| this.suspend(cx));
        assert!(view.read(cx).snapshot.is_none());
        window.remove_window();
    })
    .unwrap();
    owner.shutdown();
}

#[gpui_kit::test]
async fn records_last_page_renders_its_row_after_a_scrolled_full_page(cx: &mut TestAppContext) {
    use ccr_usage::fixtures::{SeedEvent, seed_event};
    cx.executor().allow_parking();
    let owner = RuntimeOwner::new().unwrap();
    let root = tempfile::tempdir().unwrap();
    let paths = create_projection_db(&root.path().join("llmusage"));
    let pool = create_pool(&paths.db_path, None).unwrap();
    for i in 0..101 {
        seed_event(
            &pool.get().unwrap(),
            &SeedEvent {
                event_key: format!("page-event-{i:03}"),
                ..Default::default()
            },
        );
    }
    drop(pool);
    let (handle, view) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| InsightsView::new(services(&owner, root.path()), window, cx))
        })
        .unwrap()
    });
    cx.update_window(handle, |_, window, cx| {
        view.update(cx, |this, cx| {
            this.section = Section::Logs;
            this.set_page(InsightsPage::Usage, cx);
        });
        window.render_frame(cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        !view.read(cx).loading
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        assert_eq!(view.read(cx).row_count(), 100);
        view.read(cx)
            .scroll
            .scroll_to_item_strict(99, gpui_kit::ScrollStrategy::Bottom);
        window.render_frame(cx);
        let last = view.read(cx).row_key(99).unwrap().to_string();
        assert!(
            window
                .find(SharedString::from(format!("insights-row:{last}")))
                .visible()
        );
        window.click("insights-next", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        !view.read(cx).loading
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        assert_eq!(view.read(cx).row_count(), 1);
        window.render_frame(cx);
        let final_key = view.read(cx).row_key(0).unwrap().to_string();
        assert!(
            window
                .find(SharedString::from(format!("insights-row:{final_key}")))
                .visible()
        );
        assert!(view.read(cx).rendered_rows > 0);
        assert!(!view.read(cx).has_next());
        window.remove_window();
    })
    .unwrap();
    owner.shutdown();
}

#[gpui_kit::test]
async fn rendered_archive_drill_in_pages_real_provider_messages_and_clears_plaintext(
    cx: &mut TestAppContext,
) {
    use ccr_db::database::{
        migrations::run_all_migrations,
        repositories::usage_repo::{self, UsageSessionArchiveEntry},
    };
    cx.executor().allow_parking();
    let owner = RuntimeOwner::new().unwrap();
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let source = home.join(".codex/sessions/2026/09/24/rollout-native.jsonl");
    std::fs::create_dir_all(source.parent().unwrap()).unwrap();
    let line = "{\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"user\",\"content\":[{\"type\":\"input_text\",\"text\":\"scoped native transcript\"}]}}\n";
    std::fs::write(&source, line.repeat(250)).unwrap();
    let original = std::fs::read(&source).unwrap();
    let pool = create_pool(&root.path().join("archive.db"), None).unwrap();
    run_all_migrations(&pool.get().unwrap(), root.path()).unwrap();
    let now = chrono::Utc::now();
    let registry = ccr_store::sessions::AgentSessionProviderRegistry::new(home.clone())
        .without_environment_overrides();
    let discovered = registry
        .discover(AgentSessionAgentId::Codex)
        .unwrap()
        .remove(0);
    let fingerprint = registry.quick_fingerprint(&discovered).unwrap();
    let path = discovered.physical_path.to_string_lossy().to_string();
    // Archive identity is the provider's exact path, not a manually mixed Windows string.
    let id = usage_repo::agent_session_archive_id("codex", &path, "");
    usage_repo::upsert_session_archive_entry(
        &pool.get().unwrap(),
        &UsageSessionArchiveEntry {
            archive_id: id.clone(),
            session_id: "rollout-native".into(),
            platform: "codex".into(),
            title: Some("Native archive fixture".into()),
            cwd: String::new(),
            file_path: path,
            file_hash: Some(fingerprint.clone()),
            source_variant: "codex-live".into(),
            source_kind: "file".into(),
            source_member_id: String::new(),
            source_size: None,
            source_mtime_ns: None,
            source_stat_hash: Some(fingerprint),
            message_count: 250,
            user_message_count: 250,
            assistant_message_count: 0,
            tool_use_count: 0,
            source_fidelity: "partial".into(),
            created_at: now,
            updated_at: now,
            source_state: UsageSourceState::Live,
            last_seen_at: Some(now),
            raw_deleted_at: None,
            archived_at: now,
        },
    )
    .unwrap();
    drop(pool);
    let (handle, view) = cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| InsightsView::new(services(&owner, root.path()), window, cx))
        })
        .unwrap()
    });
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        window.click("insights-sessions", cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        !view.read(cx).loading
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        assert!(view.read(cx).error.is_none());
        assert_eq!(view.read(cx).row_count(), 1);
        window.render_frame(cx);
        window.click(SharedString::from(format!("insights-row:{id}")), cx);
    })
    .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        !view.read(cx).loading
    })
    .await;
    let first_keys = cx
        .update_window(handle, |_, window, cx| {
            assert!(view.read(cx).error.is_none());
            assert_eq!(view.read(cx).row_count(), 100);
            assert!(view.read(cx).summary().contains("partial"));
            let keys = (0..100)
                .map(|i| view.read(cx).row_key(i).unwrap().to_string())
                .collect::<Vec<_>>();
            window.render_frame(cx);
            window.click(SharedString::from(format!("insights-row:{}", keys[0])), cx);
            window.render_frame(cx);
            assert!(window.find("insights-exact-detail").visible());
            assert!(
                view.read(cx)
                    .details(0)
                    .join(" ")
                    .contains("scoped native transcript")
            );
            window.click("insights-next", cx);
            keys
        })
        .unwrap();
    cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
        !view.read(cx).loading
    })
    .await;
    cx.update_window(handle, |_, window, cx| {
        assert_eq!(view.read(cx).row_count(), 100);
        assert!((0..100).all(|i| {
            !first_keys
                .iter()
                .any(|key| Some(key.as_str()) == view.read(cx).row_key(i))
        }));
        window.render_frame(cx);
        window.click("insights-usage", cx);
        assert!(view.read(cx).snapshot.is_none());
        assert!(view.read(cx).selected.is_none());
        assert!(!view.read(cx).detail_open);
        view.update(cx, |this, cx| this.suspend(cx));
        window.remove_window();
    })
    .unwrap();
    assert_eq!(original, std::fs::read(source).unwrap());
    owner.shutdown();
}
