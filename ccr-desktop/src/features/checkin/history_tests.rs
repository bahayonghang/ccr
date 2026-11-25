use super::*;

#[gpui_kit::test]
fn sidebar_records_filter_and_return_to_the_failed_account(cx: &mut TestAppContext) {
    use ccr_db::models::checkin::CheckinRecord;
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    seed(root.path());
    let conn = rusqlite::Connection::open(root.path().join("checkin.db")).unwrap();
    let mut failed = CheckinRecord::failed(
        "measured".into(),
        "session refused".into(),
        Some("cookie_expired".into()),
    );
    failed.id = "repair".into();
    failed.checked_in_at = at("2024-04-04T12:00:00Z");
    checkin_repo::insert_record(&conn, &failed).unwrap();
    for n in 0..20 {
        let mut record = CheckinRecord::failed(
            "measured".into(),
            "cookie was marked expired in message only".into(),
            Some("network_error".into()),
        );
        record.checked_in_at = failed.checked_in_at + chrono::Duration::seconds(n + 1);
        checkin_repo::insert_record(&conn, &record).unwrap();
    }
    let owner = RuntimeOwner::new().unwrap();
    let handle = open_size(cx, &owner, root.path(), 960., 640.);
    cx.update_window(handle, |_, w, cx| {
        w.render_frame(cx);
        w.click("nav-checkin", cx);
    })
    .unwrap();
    wait_ui(cx, handle, |w| w.try_find("checkin-account-1").is_some());
    cx.update_window(handle, |_, w, cx| {
        w.click("checkin-tab-records", cx);
        w.render_frame(cx);
        assert!(
            w.try_find("checkin-filter-failed").is_some(),
            "records tab exposes status filters"
        );
    })
    .unwrap();
    click_control(cx, handle, "checkin-filter-failed");
    wait_ui(cx, handle, |w| {
        labels(w).contains("Matching records: 21 · Page 1/2")
    });
    click_control(cx, handle, "checkin-records-next");
    wait_ui(cx, handle, |w| {
        w.try_find("checkin-record-repair-repair").is_some()
    });
    click_control(cx, handle, "checkin-record-repair-repair");
    wait_ui(cx, handle, |w| {
        w.try_find("checkin-credential-read-error").is_some()
    });
    click_control(cx, handle, "checkin-cancel-edit");
    cx.update_window(handle, |_, w, cx| {
        w.simulate_next_frame(cx);
        w.render_frame(cx);
        w.simulate_next_frame(cx);
        w.render_frame(cx);
        assert!(labels(w).contains("Matching records: 21 · Page 2/2"));
        assert_eq!(w.find("checkin-record-repair-repair").focused(), Some(true));
    })
    .unwrap();
    click_control(cx, handle, "checkin-record-choose-provider");
    click_control(cx, handle, "checkin-record-option-1");
    wait_ui(cx, handle, |w| {
        labels(w).contains("Matching records: 21 · Page 1/2")
    });
    click_control(cx, handle, "checkin-record-choose-account");
    click_control(cx, handle, "checkin-record-option-2");
    click_control(cx, handle, "checkin-record-keyword");
    cx.update_window(handle, |_, w, cx| {
        w.input("session refused", cx);
        w.render_frame(cx);
    })
    .unwrap();
    wait_ui(cx, handle, |w| {
        labels(w).contains("Matching records: 1 · Page 1/1")
    });
    click_control(cx, handle, "checkin-record-detail-repair");
    wait_ui(cx, handle, |w| {
        w.try_find("checkin-calendar-summary").is_some()
    });
    cx.update_window(handle, |_, w, _| {
        assert!(w.try_find("checkin-record-count").is_none())
    })
    .unwrap();
    click_control(cx, handle, "checkin-back");
    cx.update_window(handle, |_, w, cx| {
        w.simulate_next_frame(cx);
        w.render_frame(cx);
        w.simulate_next_frame(cx);
        w.render_frame(cx);
        assert_eq!(w.find("checkin-record-detail-repair").focused(), Some(true));
    })
    .unwrap();
    click_control(cx, handle, "checkin-record-clear");
    click_control(cx, handle, "checkin-record-keyword");
    cx.update_window(handle, |_, w, cx| {
        w.input("cookie_expired", cx);
        w.render_frame(cx);
    })
    .unwrap();
    wait_ui(cx, handle, |w| {
        w.try_find("checkin-record-no-match").is_some()
    });
    click_control(cx, handle, "checkin-record-clear");
    wait_ui(cx, handle, |w| {
        labels(w).contains("Matching records: 21 · Page 1/2")
    });
    cx.update_window(handle, |_, w, _| w.remove_window())
        .unwrap();
    owner.shutdown();
}

fn click_control(cx: &mut TestAppContext, handle: gpui_kit::AnyWindowHandle, id: &str) {
    for _ in 0..80 {
        let clicked = cx
            .update_window(handle, |_, w, cx| {
                w.render_frame(cx);
                let target = w.find(id.to_owned()).bounds().center().y;
                let page = w.find("checkin-page").bounds();
                let center = page.origin.y + page.size.height * 0.5;
                if target > page.origin.y + px(10.)
                    && target < page.origin.y + page.size.height - px(10.)
                {
                    w.click(id.to_owned(), cx);
                    w.render_frame(cx);
                    true
                } else {
                    let delta = (center - target).clamp(px(-350.), px(350.));
                    w.scroll(
                        "checkin-page",
                        gpui_kit::ScrollDelta::Pixels(gpui_kit::point(px(0.), delta)),
                        cx,
                    );
                    false
                }
            })
            .unwrap();
        if clicked {
            return;
        }
        cx.run_until_parked();
    }
    panic!("could not reveal {id}");
}

#[track_caller]
fn wait_ui(
    cx: &mut TestAppContext,
    handle: gpui_kit::AnyWindowHandle,
    mut ready: impl FnMut(&mut Window) -> bool,
) {
    let caller = std::panic::Location::caller();
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    loop {
        cx.run_until_parked();
        let done = cx
            .update_window(handle, |_, w, cx| {
                w.render_frame(cx);
                let done = ready(w);
                if !done && std::time::Instant::now() >= deadline {
                    panic!(
                        "unexpected page at {caller}: {:?}",
                        w.try_find("checkin-record-count")
                            .and_then(|n| n.label().map(str::to_owned))
                    );
                }
                done
            })
            .unwrap();
        if done {
            return;
        }
        cx.executor().advance_clock(Duration::from_millis(5));
        std::thread::yield_now();
    }
}

#[gpui_kit::test]
fn wide_detail_shares_calendar_observations_with_the_trend_and_rejects_replaced_windows(
    cx: &mut TestAppContext,
) {
    use chrono::{Datelike, Local, Months};
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    seed(root.path());
    let previous = Local::now()
        .date_naive()
        .with_day(1)
        .unwrap()
        .checked_sub_months(Months::new(1))
        .unwrap();
    let day2 = previous.with_day(2).unwrap();
    let day3 = previous.with_day(3).unwrap();
    let db = rusqlite::Connection::open(root.path().join("checkin.db")).unwrap();
    let mut zero = BalanceSnapshot::new("unmeasured".into(), 10., 10., 0., " USD ".into());
    zero.recorded_at = at(&format!("{day2}T12:00:00Z"));
    checkin_repo::insert_balance(&db, &zero).unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let h = open_size(cx, &owner, root.path(), 1280., 800.);
    cx.update_window(h, |_, w, cx| {
        w.render_frame(cx);
        w.click("nav-checkin", cx);
    })
    .unwrap();
    wait_ui(cx, h, |w| w.try_find("checkin-account-1").is_some());
    click_control(cx, h, "checkin-account-0");
    click_control(cx, h, "checkin-month-prev");
    wait_ui(cx, h, |w| {
        w.try_find(format!("checkin-calendar-{day2}")).is_some()
    });
    click_control(cx, h, &format!("checkin-calendar-{day3}"));
    click_control(cx, h, "checkin-history-values");
    cx.update_window(h, |_, w, _| {
        assert!(w.try_find("checkin-account-1").is_some());
        let exact = w.find("checkin-selected-day").label().unwrap().to_string();
        assert!(exact.contains(&format!(
            "Carried forward · 0 USD · Total 10 · Used 10 · Source {day2}"
        )));
        assert_eq!(
            w.find(format!("checkin-trend-day-{day3}")).label(),
            Some(exact.as_str())
        );
        assert_eq!(
            w.find(format!("checkin-trend-point-USD-{day3}")).label(),
            Some(exact.as_str())
        );
        assert!(
            w.find(format!("checkin-trend-day-{day2}"))
                .label()
                .unwrap()
                .contains("Measured · 0 USD")
        );
    })
    .unwrap();
    // Open a different account and month before waiting for either read.
    click_control(cx, h, "checkin-account-1");
    click_control(cx, h, "checkin-month-prev");
    let older = previous.checked_sub_months(Months::new(1)).unwrap();
    wait_ui(cx, h, |w| {
        w.try_find(format!("checkin-trend-day-{older}")).is_some()
    });
    cx.update_window(h, |_, w, _| {
        assert!(w.try_find(format!("checkin-trend-day-{day2}")).is_none());
        assert!(
            w.find(format!("checkin-trend-day-{older}"))
                .label()
                .unwrap()
                .contains("Carried forward · 7.5 USD")
        );
    })
    .unwrap();
    click_control(cx, h, "checkin-account-0");
    wait_ui(cx, h, |w| w.try_find("checkin-trend-empty").is_some());
    cx.update_window(h, |_, w, _| {
        assert!(
            w.find(format!("checkin-trend-day-{older}"))
                .label()
                .unwrap()
                .contains("No balance sample")
        );
        w.remove_window();
    })
    .unwrap();
    owner.shutdown();
}
