use super::*;
use crate::runtime::checkin::history::RecordQuery;

#[test]
fn read_history_without_storage_never_creates_it() {
    let root = tempfile::tempdir().unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let service = services(&owner, root.path());
    assert!(
        service
            .checkin_records(&RecordQuery::default())
            .unwrap()
            .is_none()
    );
    assert!(std::fs::read_dir(root.path()).unwrap().next().is_none());
    std::fs::write(root.path().join("checkin.db"), b"unreadable fixture").unwrap();
    assert!(service.checkin_records(&RecordQuery::default()).is_err());
    owner.shutdown();
}

#[test]
fn account_history_preserves_evidence_across_units_and_gaps() {
    use crate::runtime::checkin::history::DayBalance;
    use ccr_db::models::checkin::CheckinRecord;
    use chrono::{Datelike, NaiveDate};
    let root = tempfile::tempdir().unwrap();
    seed(root.path());
    let conn = rusqlite::Connection::open(root.path().join("checkin.db")).unwrap();
    for (day, unit, value) in [
        (2, " USD ", 0.),
        (5, "EUR", 3.),
        (7, "EUR", f64::INFINITY),
        (9, "", 20.),
        (11, "USD", 4.),
    ] {
        let mut row = BalanceSnapshot::new("unmeasured".into(), value, 0., value, unit.into());
        row.recorded_at = at(&format!("2024-04-{day:02}T12:00:00Z"));
        checkin_repo::insert_balance(&conn, &row).unwrap();
    }
    let mut r = CheckinRecord::success("unmeasured".into(), None, None);
    r.checked_in_at = at("2024-04-01T18:00:00Z");
    checkin_repo::insert_record(&conn, &r).unwrap();
    // These newer records and snapshots used to hide history beyond manager caps.
    for index in 0..1100 {
        let mut later = BalanceSnapshot::new("unmeasured".into(), 99., 0., 99., "CNY".into());
        later.recorded_at = at("2024-06-01T12:00:00Z") + chrono::Duration::seconds(index);
        checkin_repo::insert_balance(&conn, &later).unwrap();
        let mut rec = CheckinRecord::success("unmeasured".into(), None, None);
        rec.checked_in_at = later.recorded_at;
        checkin_repo::insert_record(&conn, &rec).unwrap();
    }
    let owner = RuntimeOwner::new().unwrap();
    let s = services(&owner, root.path());
    let april = NaiveDate::from_ymd_opt(2024, 4, 1).unwrap();
    let history = s
        .checkin_account_history("unmeasured", april, 30)
        .unwrap()
        .unwrap();
    assert_eq!(history.days.len(), 30);
    assert!(history.days[0].checked);
    assert!(matches!(history.days[0].balance, DayBalance::Missing));
    assert!(
        matches!(&history.days[1].balance, DayBalance::Measured(n) if n.unit=="USD" && n.remaining==0.)
    );
    assert!(
        matches!(&history.days[3].balance, DayBalance::CarriedForward(n) if n.unit=="USD" && n.source.day()==2)
    );
    assert!(
        matches!(&history.days[4].balance, DayBalance::Measured(n) if n.unit=="EUR" && n.remaining==3.)
    );
    assert!(matches!(
        history.days[6].balance,
        DayBalance::Unusable { .. }
    ));
    assert!(matches!(history.days[7].balance, DayBalance::Missing));
    assert!(matches!(
        history.days[8].balance,
        DayBalance::Unusable { .. }
    ));
    assert!(matches!(history.days[9].balance, DayBalance::Missing));
    let may = NaiveDate::from_ymd_opt(2024, 5, 1).unwrap();
    let next = s
        .checkin_account_history("unmeasured", may, 7)
        .unwrap()
        .unwrap();
    assert!(
        matches!(&next.days[0].balance, DayBalance::CarriedForward(n) if n.unit=="USD" && n.source==april+chrono::Duration::days(10))
    );
    assert!(!root.path().join("checkin-crypto").exists());
    owner.shutdown();
}

#[test]
fn filtered_records_retain_saved_fields_and_enabled_today_counts() {
    use ccr_db::models::checkin::CheckinRecord;
    let root = tempfile::tempdir().unwrap();
    seed(root.path());
    let db = rusqlite::Connection::open(root.path().join("checkin.db")).unwrap();
    db.execute(
        "UPDATE checkin_accounts SET enabled=0 WHERE id='unmeasured'",
        [],
    )
    .unwrap();
    let failure = CheckinRecord::failed(
        "measured".into(),
        "access refused session=legacy-session-value token=controlled-private-token-value".into(),
        Some("cookie_expired".into()),
    );
    checkin_repo::insert_record(&db, &failure).unwrap();
    let success = CheckinRecord::success(
        "measured".into(),
        Some("credited".into()),
        Some("bonus-voucher".into()),
    )
    .with_balance(None, Some(0.));
    checkin_repo::insert_record(&db, &success).unwrap();
    checkin_repo::insert_record(
        &db,
        &CheckinRecord::failed("unmeasured".into(), "refused".into(), None),
    )
    .unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let s = services(&owner, root.path());
    let mut q = RecordQuery {
        account: Some("measured".into()),
        provider: Some("provider-1".into()),
        status: Some("success".into()),
        keyword: "fixture provider".into(),
        page: 7,
    };
    let page = s.checkin_records(&q).unwrap().unwrap();
    assert_eq!(
        (
            page.total,
            page.page,
            page.enabled,
            page.checked,
            page.failed
        ),
        (1, 1, 1, 1, 0)
    );
    let row = &page.rows[0];
    assert_eq!(row.account.as_deref(), Some("Desk account"));
    assert_eq!(row.provider.as_deref(), Some("Fixture provider"));
    assert_eq!(row.reward.as_deref(), Some("bonus-voucher"));
    assert_eq!(row.before, None);
    assert_eq!(row.after.as_deref(), Some("0"));
    for keyword in ["bonus-voucher", "cookie_expired"] {
        q.keyword = keyword.into();
        q.status = None;
        assert_eq!(s.checkin_records(&q).unwrap().unwrap().total, 0);
    }
    q.keyword = "refused".into();
    q.status = Some("failed".into());
    let failed = s.checkin_records(&q).unwrap().unwrap();
    assert!(failed.rows[0].needs_credentials());
    let message = failed.rows[0].reason.as_deref().unwrap();
    assert!(message.contains("access refused"));
    assert!(!message.contains("controlled-private-token-value"));
    assert!(!message.contains("legacy-session-value"));
    assert!(!root.path().join("checkin-crypto").exists());
    owner.shutdown();
}
