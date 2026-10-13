use ccr_config::models::platform::Platform;
use ccr_store::{ProfileActivityAvailability, ProfileActivityError, read_profile_activity};
use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use std::{collections::BTreeMap, fs, path::Path, time::SystemTime};
use tempfile::TempDir;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const SCHEMA: &str = "CREATE TABLE history (
    id INTEGER PRIMARY KEY, timestamp, operation TEXT, result TEXT,
    to_config TEXT, extra TEXT, env_changes TEXT, notes TEXT);
    CREATE INDEX idx_history_operation ON history(operation);
    CREATE INDEX idx_history_timestamp ON history(timestamp);";

fn fixture() -> TestResult<(TempDir, Connection)> {
    let dir = tempfile::tempdir()?;
    let conn = Connection::open(dir.path().join("data.db"))?;
    conn.execute_batch(SCHEMA)?;
    Ok((dir, conn))
}

fn insert(
    conn: &Connection,
    timestamp: &str,
    operation: &str,
    result: &str,
    platform: &str,
    name: &str,
) -> TestResult {
    conn.execute(
        "INSERT INTO history(timestamp, operation, result, extra, to_config, env_changes, notes)
         VALUES (?1, ?2, ?3, ?4, ?5, 'SECRET_SENTINEL', 'SECRET_SENTINEL')",
        params![timestamp, operation, result, platform, name],
    )?;
    Ok(())
}

fn utc(text: &str) -> TestResult<DateTime<Utc>> {
    Ok(DateTime::parse_from_rfc3339(text)?.with_timezone(&Utc))
}

fn directory_snapshot(dir: &Path) -> TestResult<BTreeMap<String, (Vec<u8>, SystemTime)>> {
    fs::read_dir(dir)?
        .map(|entry| {
            let entry = entry?;
            Ok((
                entry.file_name().to_string_lossy().into_owned(),
                (fs::read(entry.path())?, entry.metadata()?.modified()?),
            ))
        })
        .collect()
}

#[test]
fn absent_database_and_schema_stay_absent_without_any_filesystem_changes() -> TestResult {
    let dir = tempfile::tempdir()?;
    let missing = dir.path().join("missing").join("data.db");
    let result = read_profile_activity(&missing, Platform::Codex, &["work".into()])?;
    assert_eq!(
        result.availability,
        ProfileActivityAvailability::MissingDatabase
    );
    assert_eq!(fs::read_dir(dir.path())?.count(), 0);

    let path = dir.path().join("data.db");
    let conn = Connection::open(&path)?;
    conn.execute_batch("CREATE TABLE unrelated (id INTEGER); PRAGMA user_version = 7;")?;
    drop(conn);
    let before = directory_snapshot(dir.path())?;
    let result = read_profile_activity(&path, Platform::Codex, &["work".into()])?;
    assert_eq!(
        result.availability,
        ProfileActivityAvailability::MissingSchema
    );
    assert_eq!(directory_snapshot(dir.path())?, before);

    let conn = Connection::open(&path)?;
    conn.execute_batch("CREATE TABLE history(timestamp TEXT, to_config TEXT);")?;
    drop(conn);
    let before = directory_snapshot(dir.path())?;
    assert_eq!(
        read_profile_activity(&path, Platform::Codex, &["work".into()])?.availability,
        ProfileActivityAvailability::MissingSchema
    );
    assert_eq!(directory_snapshot(dir.path())?, before);
    Ok(())
}

#[test]
fn exact_successful_platform_and_target_are_required_and_credentials_are_not_decoded() -> TestResult
{
    let (dir, conn) = fixture()?;
    for (platform, hour) in [("codex", "10"), ("claude", "11"), ("grok", "12")] {
        insert(
            &conn,
            &format!("2026-10-12T{hour}:00:00Z"),
            "Switch",
            "Success",
            platform,
            "work",
        )?;
    }
    for (operation, result, platform, name) in [
        ("Switch", "Failure", "codex", "work"),
        ("Switch", "Warning", "codex", "work"),
        ("Switch", "unknown", "codex", "work"),
        ("Update", "Success", "codex", "work"),
        ("unknown", "Success", "codex", "work"),
        ("Switch", "Success", "", "work"),
        ("Switch", "Success", "Codex", "work"),
        ("Switch", "Success", " codex ", "work"),
        ("Switch", "Success", "{\"platform\":\"codex\"}", "work"),
        ("Switch", "Success", "codex", "Work"),
        ("Switch", "Success", "codex", " work "),
        ("Switch", "Success", "codex", ""),
    ] {
        insert(
            &conn,
            "2099-01-01T00:00:00Z",
            operation,
            result,
            platform,
            name,
        )?;
    }
    conn.execute(
        "INSERT INTO history(timestamp, operation, result, to_config)
        VALUES ('2099-01-01T00:00:00Z', 'Switch', 'Success', 'work')",
        [],
    )?;
    drop(conn);
    let before = directory_snapshot(dir.path())?;
    for (platform, hour) in [
        (Platform::Codex, "10"),
        (Platform::Claude, "11"),
        (Platform::Grok, "12"),
    ] {
        let snapshot = read_profile_activity(
            &dir.path().join("data.db"),
            platform,
            &["work".into(), "missing".into(), " ".into()],
        )?;
        assert_eq!(snapshot.availability, ProfileActivityAvailability::Ready);
        assert_eq!(snapshot.profiles.len(), 1);
        assert_eq!(
            snapshot.profiles["work"].recent_activations,
            [utc(&format!("2026-10-12T{hour}:00:00Z"))?]
        );
        assert_eq!(snapshot.profiles["work"].invalid_timestamps, 0);
        assert!(!format!("{snapshot:?}").contains("SECRET_SENTINEL"));
    }
    assert_eq!(directory_snapshot(dir.path())?, before);
    Ok(())
}

#[test]
fn utc_order_top_three_and_repeated_activations_survive_lexically_misleading_offsets() -> TestResult
{
    let (dir, conn) = fixture()?;
    for timestamp in [
        "2026-10-12T12:00:00+10:00", // 02:00Z; lexically largest hour, oldest instant
        "2026-10-12T01:00:00-07:00", // 08:00Z; actual latest instant
        "2026-10-12T07:00:00Z",
        "2026-10-12T07:00:00Z", // distinct successful records may share a timestamp
        "2026-10-12T06:00:00Z",
    ] {
        insert(&conn, timestamp, "Switch", "Success", "codex", "work")?;
    }
    let result = read_profile_activity(
        &dir.path().join("data.db"),
        Platform::Codex,
        &["work".into(), "work".into()],
    )?;
    assert_eq!(
        result.profiles["work"].recent_activations,
        [
            utc("2026-10-12T08:00:00Z")?,
            utc("2026-10-12T07:00:00Z")?,
            utc("2026-10-12T07:00:00Z")?
        ]
    );
    Ok(())
}

#[test]
fn invalid_dates_remain_partial_and_future_dates_remain_real_instants() -> TestResult {
    let (dir, conn) = fixture()?;
    for timestamp in [
        "bad",
        "2026-10-12",
        "2026-10-12T12:00:00",
        "2026-02-30T00:00:00Z",
        "",
    ] {
        insert(&conn, timestamp, "Switch", "Success", "codex", "work")?;
    }
    conn.execute(
        "INSERT INTO history(timestamp, operation, result, extra, to_config)
        VALUES (NULL, 'Switch', 'Success', 'codex', 'work'),
               (17777777, 'Switch', 'Success', 'codex', 'work'),
               (x'FFFF', 'Switch', 'Success', 'codex', 'work')",
        [],
    )?;
    insert(
        &conn,
        "2100-01-01T00:00:00+03:00",
        "Switch",
        "Success",
        "codex",
        "work",
    )?;
    insert(
        &conn,
        "2026-10-12T00:00:00Z",
        "Switch",
        "Success",
        "codex",
        "work",
    )?;
    insert(&conn, "bad", "Switch", "Success", "codex", "other")?;
    let result = read_profile_activity(
        &dir.path().join("data.db"),
        Platform::Codex,
        &["work".into()],
    )?;
    assert_eq!(result.profiles["work"].invalid_timestamps, 8);
    assert_eq!(
        result.profiles["work"].recent_activations,
        [utc("2099-12-31T21:00:00Z")?, utc("2026-10-12T00:00:00Z")?]
    );
    assert!(!result.profiles.contains_key("other"));
    Ok(())
}

#[test]
fn cleared_history_and_empty_requests_are_ready_without_invented_timestamps() -> TestResult {
    let (dir, conn) = fixture()?;
    insert(
        &conn,
        "2026-10-12T00:00:00Z",
        "Switch",
        "Success",
        "grok",
        "work",
    )?;
    let path = dir.path().join("data.db");
    assert!(
        read_profile_activity(&path, Platform::Grok, &[])?
            .profiles
            .is_empty()
    );
    conn.execute("DELETE FROM history", [])?;
    let snapshot = read_profile_activity(&path, Platform::Grok, &["work".into()])?;
    assert_eq!(snapshot.availability, ProfileActivityAvailability::Ready);
    assert!(snapshot.profiles.is_empty());
    Ok(())
}

#[test]
fn corrupt_or_locked_storage_reports_error_without_repair_or_sensitive_output() -> TestResult {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("data.db");
    fs::write(&path, "SECRET_SENTINEL invalid database")?;
    let before = directory_snapshot(dir.path())?;
    let err = read_profile_activity(&path, Platform::Codex, &["work".into()])
        .expect_err("corrupt fixture must fail without repairing the database");
    assert_eq!(err, ProfileActivityError::Query);
    assert!(!format!("{err} {err:?}").contains("SECRET_SENTINEL"));
    assert_eq!(directory_snapshot(dir.path())?, before);

    let (dir, conn) = fixture()?;
    conn.execute_batch("BEGIN EXCLUSIVE;")?;
    assert_eq!(
        read_profile_activity(
            &dir.path().join("data.db"),
            Platform::Codex,
            &["work".into()]
        ),
        Err(ProfileActivityError::Query)
    );
    conn.execute_batch("ROLLBACK;")?;
    Ok(())
}

#[test]
fn active_wal_records_are_visible_without_changing_database_or_wal() -> TestResult {
    let (dir, conn) = fixture()?;
    conn.execute_batch("PRAGMA journal_mode = WAL;")?;
    insert(
        &conn,
        "2026-10-12T00:00:00Z",
        "Switch",
        "Success",
        "codex",
        "work",
    )?;
    let path = dir.path().join("data.db");
    let database = fs::read(&path)?;
    let wal = fs::read(dir.path().join("data.db-wal"))?;
    let files: Vec<_> = fs::read_dir(dir.path())?
        .map(|entry| entry.map(|entry| entry.file_name()))
        .collect::<std::io::Result<_>>()?;
    let snapshot = read_profile_activity(&path, Platform::Codex, &["work".into()])?;
    assert_eq!(
        snapshot.profiles["work"].recent_activations,
        [utc("2026-10-12T00:00:00Z")?]
    );
    assert_eq!(fs::read(&path)?, database);
    assert_eq!(fs::read(dir.path().join("data.db-wal"))?, wal);
    let after: Vec<_> = fs::read_dir(dir.path())?
        .map(|entry| entry.map(|entry| entry.file_name()))
        .collect::<std::io::Result<_>>()?;
    assert_eq!(after, files);
    Ok(())
}

/// Run explicitly in release mode; timing is evidence, never a parallel CI gate.
#[test]
#[ignore]
fn benchmark_profile_activity_owned_history() -> TestResult {
    use std::time::Instant;
    for count in [0, 10_000, 100_000] {
        let (dir, mut conn) = fixture()?;
        let transaction = conn.transaction()?;
        {
            let mut insert = transaction.prepare(
                "INSERT INTO history(timestamp, operation, result, extra, to_config)
                VALUES (?1, 'Switch', ?2, ?3, ?4)",
            )?;
            for row in 0..count {
                insert.execute(params![
                    if row % 97 == 0 {
                        "invalid"
                    } else if row % 2 == 0 {
                        "2026-10-12T12:00:00+08:00"
                    } else {
                        "2026-10-12T00:00:00-07:00"
                    },
                    if row % 11 == 0 { "Failure" } else { "Success" },
                    ["codex", "claude", "grok", "legacy"][row % 4],
                    format!("profile-{}", row % 1_000),
                ])?;
            }
        }
        transaction.commit()?;
        let plan: Vec<String> = conn
            .prepare(
                "EXPLAIN QUERY PLAN SELECT to_config, timestamp FROM history
            WHERE operation = 'Switch' AND result = 'Success' AND extra = ?1",
            )?
            .query_map(["codex"], |row| row.get(3))?
            .collect::<rusqlite::Result<_>>()?;
        drop(conn);
        let names: Vec<_> = (0..1_000).map(|row| format!("profile-{row}")).collect();
        let path = dir.path().join("data.db");
        let first = Instant::now();
        read_profile_activity(&path, Platform::Codex, &names)?;
        let first_ms = first.elapsed().as_secs_f64() * 1_000.0;
        let mut samples = Vec::new();
        for _ in 0..30 {
            let start = Instant::now();
            let snapshot = read_profile_activity(&path, Platform::Codex, &names)?;
            assert!(
                snapshot
                    .profiles
                    .values()
                    .all(|value| value.recent_activations.len() <= 3)
            );
            samples.push(start.elapsed().as_secs_f64() * 1_000.0);
        }
        samples.sort_by(f64::total_cmp);
        println!(
            "history rows={count} names=1000 bytes={} samples=30 first_ms={first_ms:.3} median_ms={:.3} p95_ms={:.3} max_ms={:.3} plan={plan:?}",
            fs::metadata(&path)?.len(),
            samples[14],
            samples[28],
            samples[29]
        );
    }
    Ok(())
}
