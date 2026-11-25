use super::*;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, Ordering};

fn data(services: &Services) -> CheckinData {
    let CheckinSnapshot::Accounts(data) = read_source(&services.source).unwrap() else {
        panic!("expected accounts");
    };
    data
}

fn plan(services: &Services, action: JobAction, scope: JobScope) -> AccountJob {
    AccountJob::prepare(&data(services), action, scope).unwrap()
}

#[test]
fn mixed_jobs_confirm_identity_retain_results_throttle_and_preserve_failed_samples() {
    let root = tempfile::tempdir().unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let services =
        Services::new(owner.handle(), source(root.path())).with_checkin_loopback_fixture();
    let network = tokio::runtime::Runtime::new().unwrap();
    let listener = network
        .block_on(tokio::net::TcpListener::bind("127.0.0.1:0"))
        .unwrap();
    let addr = listener.local_addr().unwrap();
    let requests = Arc::new(Mutex::new(Vec::new()));
    let calls = requests.clone();
    let force_failure = Arc::new(AtomicBool::new(false));
    let fail_good = force_failure.clone();
    let server = network.spawn(async move {
        loop {
            let (mut stream,_) = listener.accept().await.unwrap();
            let request = read_headers(&mut stream).await.to_ascii_lowercase();
            let user = request.lines().find_map(|line|line.strip_prefix("new-api-user: ")).unwrap();
            let method = request.split_whitespace().next().unwrap();
            calls.lock().unwrap().push((method.to_string(),user.to_string()));
            tokio::time::sleep(Duration::from_millis(50)).await;
            let (status, body) = if user == "bad" || (user == "good" && fail_good.load(Ordering::SeqCst)) {
                ("401 Unauthorized", r#"{"success":false,"message":"Unauthorized"}"#)
            } else if method == "post" {
                ("200 OK",r#"{"success":true,"message":"Fixture saved session=private-echo and cookie fixture-secret","data":{"reward":"1 USD"}}"#)
            } else if user == "today" {
                ("200 OK",r#"{"success":true,"data":{"check_in_today":true,"quota":500000,"used_quota":100000}}"#)
            } else {
                ("200 OK",r#"{"success":true,"data":{"quota":500000,"used_quota":100000}}"#)
            };
            let header = format!("HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len());
            stream.write_all(header.as_bytes()).await.unwrap();
        }
    });
    let created = run(
        &services,
        Operation::AddProvider(provider(&format!("http://{addr}"))),
        1,
    );
    let provider_id = created.snapshot.as_ref().unwrap().providers[0].id.clone();
    let mut ids = BTreeMap::new();
    for (index, user) in ["good", "today", "bad", "off"].into_iter().enumerate() {
        let mut a = account(provider_id.clone());
        a.name = user.into();
        a.api_user = user.into();
        let added = run(&services, Operation::AddAccount(a), index as u64 + 2);
        ids.insert(user, added.created_account.clone().unwrap());
    }
    let stale = plan(&services, JobAction::Checkin, JobScope::AllEnabled);
    let toggle = UpdateAccountRequest {
        name: None,
        cookies_json: None,
        api_user: None,
        enabled: Some(false),
        extra_config: None,
    };
    run(
        &services,
        Operation::UpdateAccount(ids["off"].clone(), toggle),
        7,
    );
    assert_eq!(
        run(&services, Operation::Job(stale), 8).result,
        Some(Err(OperationError::TargetsChanged))
    );
    assert!(requests.lock().unwrap().is_empty());
    let selected = JobScope::Selected(ids.values().cloned().collect());
    let job = plan(&services, JobAction::Checkin, selected);
    services
        .start_checkin(key(9), Operation::Job(job.clone()))
        .unwrap();
    assert_eq!(
        services.start_checkin(key(10), Operation::Job(job)),
        Err(OperationError::Busy)
    );
    let mut watch = services.observe_checkin();
    network.block_on(async {
        while watch.borrow_and_update().busy {
            tokio::time::timeout(Duration::from_secs(10), watch.changed())
                .await
                .unwrap()
                .unwrap();
        }
    });
    drop(watch);
    let late = services.observe_checkin();
    let retained = late.borrow().last_job.clone().unwrap();
    let items: BTreeMap<_, _> = retained
        .items
        .iter()
        .map(|i| (i.target.name.as_str(), i))
        .collect();
    assert_eq!(
        items["good"].status,
        JobStatus::Checked(CheckinStatus::Success)
    );
    let message = items["good"].message.as_deref().unwrap();
    assert!(message.contains("Fixture saved"));
    assert!(!message.contains("private-echo") && !message.contains("fixture-secret"));
    let stored = services
        .checkin_records(&RecordQuery {
            account: Some(ids["good"].clone()),
            ..Default::default()
        })
        .unwrap()
        .unwrap();
    let record = &stored.rows[0];
    assert_eq!(record.reward.as_deref(), Some("1 USD"));
    assert_eq!(record.reason.as_deref(), Some(message));
    assert_eq!(
        items["today"].status,
        JobStatus::Checked(CheckinStatus::AlreadyCheckedIn)
    );
    assert_eq!(
        items["off"].skip_reason.as_deref(),
        Some("account_disabled")
    );
    assert_eq!(
        items["bad"].status,
        JobStatus::Checked(CheckinStatus::Failed)
    );
    assert_eq!(items["bad"].error_code.as_deref(), Some("cookie_expired"));
    assert_eq!(items["bad"].target.id, ids["bad"]);
    assert_eq!(requests.lock().unwrap().len(), 5);
    let all = run(
        &services,
        Operation::Job(plan(&services, JobAction::Balance, JobScope::AllEnabled)),
        11,
    );
    let all = all.last_job.as_ref().unwrap();
    assert_eq!(all.items.len(), 3);
    assert_eq!(
        all.items
            .iter()
            .filter(|i| i.status == JobStatus::Updated)
            .count(),
        2
    );
    assert_eq!(requests.lock().unwrap().len(), 8);
    let before = data(&services);
    assert_eq!(before.balances.groups["USD"].accounts, 2);
    let prior = before
        .accounts
        .iter()
        .find(|r| r.id == ids["good"])
        .unwrap();
    assert!(prior.last_balance_check_at.is_some());
    let selected = BTreeSet::from([ids["good"].clone(), ids["off"].clone()]);
    let fresh = run(
        &services,
        Operation::Job(plan(
            &services,
            JobAction::Balance,
            JobScope::Selected(selected),
        )),
        12,
    );
    let fresh = fresh.last_job.as_ref().unwrap();
    assert_eq!(fresh.items.len(), 1);
    assert_eq!(fresh.items[0].status, JobStatus::Recent);
    assert_eq!(requests.lock().unwrap().len(), 8);
    force_failure.store(true, Ordering::SeqCst);
    let single = || {
        Operation::Job(plan(
            &services,
            JobAction::Balance,
            JobScope::Single(ids["good"].clone()),
        ))
    };
    let failed = run(&services, single(), 13);
    let feedback = failed.last_job.as_ref().unwrap().items[0]
        .message
        .as_deref()
        .unwrap();
    assert!(feedback.contains("HTTP 401"), "{feedback}");
    assert!(!feedback.contains("非 JSON"), "{feedback}");
    assert_eq!(
        failed.last_job.as_ref().unwrap().items[0]
            .error_code
            .as_deref(),
        Some("cookie_expired")
    );
    let after = data(&services);
    assert_eq!(
        after.accounts.iter().find(|r| r.id == ids["good"]).unwrap(),
        prior
    );
    assert_eq!(requests.lock().unwrap().len(), 9);
    force_failure.store(false, Ordering::SeqCst);
    let conn = rusqlite::Connection::open(root.path().join("checkin.db")).unwrap();
    conn.execute_batch("CREATE TRIGGER reject_refresh BEFORE UPDATE OF last_balance_check_at ON checkin_accounts BEGIN SELECT RAISE(ABORT, 'fixture persistence failure'); END;").unwrap();
    let failed = run(&services, single(), 14);
    assert_eq!(
        failed.last_job.as_ref().unwrap().items[0].status,
        JobStatus::Failed
    );
    let after = data(&services);
    assert_eq!(
        after.accounts.iter().find(|r| r.id == ids["good"]).unwrap(),
        prior
    );
    assert_eq!(requests.lock().unwrap().len(), 10);
    server.abort();
    owner.shutdown();
}

// Validate the same synthetic import that is handed to native acceptance.
#[cfg(feature = "native-checkin-fixture")]
#[test]
fn documented_fixture_imports_via_preview_and_runs_the_marked_custom_provider() {
    use crate::runtime::LaunchMode;
    use std::io::{BufRead, BufReader};
    use std::process::{Child, Command, Stdio};
    struct Server(Child);
    impl Drop for Server {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts/checkin_fixture.py");
    let python = if cfg!(windows) { "python" } else { "python3" };
    let mut server = Server(
        Command::new(python)
            .arg("-B")
            .arg(script)
            .args(["--delay-ms", "0"])
            .stdout(Stdio::piped())
            .spawn()
            .unwrap(),
    );
    let mut output = BufReader::new(server.0.stdout.take().unwrap());
    let mut line = String::new();
    output.read_line(&mut line).unwrap();
    assert!(line.contains("--fixture-checkin-loopback"));
    line.clear();
    output.read_line(&mut line).unwrap();
    let file = PathBuf::from(line.trim().strip_prefix("Import: ").unwrap());
    let root = file.parent().unwrap();
    let outputs = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../outputs/desktop")
        .canonicalize()
        .unwrap();
    assert_eq!(
        root.canonicalize().unwrap().parent(),
        Some(outputs.as_path())
    );
    assert!(
        root.file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("checkin-")
    );
    let args = vec![
        "--fixture-checkin-loopback".into(),
        "--fixture-profiles".into(),
        root.join("profiles.toml").into_os_string(),
        "--fixture-registry".into(),
        root.join("config.toml").into_os_string(),
    ];
    let LaunchMode::CheckinFixture(source) = LaunchMode::parse(args).unwrap() else {
        panic!("expected check-in fixture");
    };
    let runtime = RuntimeOwner::new().unwrap();
    let services = Services::new(runtime.handle(), source).with_checkin_loopback_fixture();
    let mut preview = services
        .checkin_import_preview(key(1), file.clone())
        .unwrap();
    let session = tokio::runtime::Runtime::new().unwrap().block_on(async {
        loop {
            if let Some(ccr_app::runtime::TaskEvent::Terminal(result)) = preview.next().await {
                if let Completion::Success(session) = result.as_ref() {
                    break session.clone();
                }
                panic!("fixture import preview failed");
            }
        }
    });
    assert!(!root.join("checkin.db").exists());
    let imported = run(
        &services,
        Operation::Extended(ExtendedCommand::Import {
            session: session.clone(),
            overwrite: false,
        }),
        2,
    );
    assert_eq!(imported.result, Some(Ok(Outcome::Extended)));
    assert_eq!(data(&services).accounts.len(), 5);
    let job = plan(&services, JobAction::Checkin, JobScope::AllEnabled);
    let state = run(&services, Operation::Job(job), 3);
    let items = &state.last_job.as_ref().unwrap().items;
    assert_eq!(items.len(), 4);
    for status in [
        CheckinStatus::Success,
        CheckinStatus::AlreadyCheckedIn,
        CheckinStatus::Skipped,
        CheckinStatus::Failed,
    ] {
        assert_eq!(
            items
                .iter()
                .filter(|item| item.status == JobStatus::Checked(status))
                .count(),
            1
        );
    }
    drop(server);
    let journal: Vec<_> = output.lines().collect::<Result<_, _>>().unwrap();
    assert_eq!(journal.len(), 5);
    assert_eq!(
        journal.iter().filter(|line| line.contains("Ready")).count(),
        2
    );
    assert_eq!(
        journal.iter().filter(|line| line.contains("Today")).count(),
        1
    );
    assert_eq!(
        journal
            .iter()
            .filter(|line| line.contains("Expired"))
            .count(),
        2
    );
    drop(services);
    runtime.shutdown();
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn queued_balance_items_recheck_changes_from_another_client_before_sending() {
    let root = tempfile::tempdir().unwrap();
    let runtime = RuntimeOwner::new().unwrap();
    let services =
        Services::new(runtime.handle(), source(root.path())).with_checkin_loopback_fixture();
    let network = tokio::runtime::Runtime::new().unwrap();
    let listener = network
        .block_on(tokio::net::TcpListener::bind("127.0.0.1:0"))
        .unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let (first, reached) = std::sync::mpsc::channel();
    let (release, wait) = tokio::sync::oneshot::channel();
    let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let count = calls.clone();
    let server = network.spawn(async move {
        let mut first_response = Some(wait);
        loop {
            let (mut socket,_) = listener.accept().await.unwrap();
            assert!(read_headers(&mut socket).await.starts_with("GET /api/user/self"));
            count.fetch_add(1, Ordering::SeqCst);
            if let Some(wait) = first_response.take() { first.send(()).unwrap(); wait.await.unwrap(); }
            let body = r#"{"success":true,"data":{"quota":500000,"used_quota":100000}}"#;
            let reply = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len());
            socket.write_all(reply.as_bytes()).await.unwrap();
        }
    });
    let added = run(&services, Operation::AddProvider(provider(&base)), 1);
    let provider_id = &added.snapshot.as_ref().unwrap().providers[0].id;
    for name in ["one", "two", "three"] {
        let mut request = account(provider_id.clone());
        request.name = name.into();
        assert_eq!(
            run(&services, Operation::AddAccount(request), 2).result,
            Some(Ok(Outcome::Saved))
        );
    }
    let job = plan(&services, JobAction::Balance, JobScope::AllEnabled);
    let disabled = job.targets[1].id.clone();
    let refreshed = job.targets[2].id.clone();
    services.start_checkin(key(3), Operation::Job(job)).unwrap();
    reached.recv_timeout(Duration::from_secs(5)).unwrap();
    let connection = rusqlite::Connection::open(root.path().join("checkin.db")).unwrap();
    connection
        .execute(
            "UPDATE checkin_accounts SET enabled=0 WHERE id=?1",
            [&disabled],
        )
        .unwrap();
    let sample = BalanceSnapshot::new(refreshed.clone(), 10., 2., 8., "USD".into());
    let tx = connection.unchecked_transaction().unwrap();
    checkin_repo::insert_balance(&tx, &sample).unwrap();
    tx.execute(
        "UPDATE checkin_accounts SET last_balance_check_at=?1 WHERE id=?2",
        [sample.recorded_at.to_rfc3339(), refreshed.clone()],
    )
    .unwrap();
    tx.commit().unwrap();
    release.send(()).unwrap();
    let mut observer = services.observe_checkin();
    network.block_on(async {
        while observer.borrow_and_update().busy {
            tokio::time::timeout(Duration::from_secs(5), observer.changed())
                .await
                .unwrap()
                .unwrap();
        }
    });
    let results = observer.borrow().last_job.clone().unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        results.items[1].admission_error,
        Some(OperationError::TargetsChanged)
    );
    assert_eq!(results.items[2].status, JobStatus::Recent);
    server.abort();
    runtime.shutdown();
}

#[test]
fn selected_balance_limit_counts_only_eligible_existing_accounts() {
    let mut data = CheckinData::default();
    for i in 0..1001 {
        let id = i.to_string();
        data.accounts.push(CheckinAccountRow {
            id: id.clone(),
            name: id,
            provider_id: "p".into(),
            provider_name: None,
            enabled: i < 1000,
            last_checkin_at: None,
            last_balance_check_at: None,
            balance: StoredBalance::Missing,
        });
    }
    let scope = JobScope::Selected(data.accounts.iter().map(|r| r.id.clone()).collect());
    assert_eq!(
        AccountJob::prepare(&data, JobAction::Balance, scope.clone())
            .unwrap()
            .targets
            .len(),
        1000
    );
    assert!(matches!(
        AccountJob::prepare(&data, JobAction::Checkin, scope.clone()),
        Err(OperationError::Invalid)
    ));
    data.accounts[1000].enabled = true;
    assert!(matches!(
        AccountJob::prepare(&data, JobAction::Balance, scope.clone()),
        Err(OperationError::Invalid)
    ));
    data.accounts.pop();
    assert!(matches!(
        AccountJob::prepare(&data, JobAction::Balance, scope),
        Err(OperationError::TargetsChanged)
    ));
}
