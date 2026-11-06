#![allow(clippy::unwrap_used)]
use super::*;
use ccr_app::runtime::{Platform, RuntimeOwner};
use ccr_checkin::BalanceManager;
use ccr_core::Secret;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
fn source(root: &Path) -> ConfigSource {
    ConfigSource::Fixture {
        profiles: root.join("profiles.toml"),
        registry: root.join("registry.toml"),
        delay: Duration::ZERO,
    }
}
fn provider(url: &str) -> CreateProviderRequest {
    CreateProviderRequest {
        name: "Fixture".into(),
        base_url: url.into(),
        checkin_path: None,
        balance_path: None,
        user_info_path: None,
        auth_header: None,
        auth_prefix: None,
        builtin_id: None,
    }
}
fn account(provider_id: String) -> CreateAccountRequest {
    CreateAccountRequest {
        provider_id,
        name: "Fixture account".into(),
        cookies_json: Secret::new(r#"{"session":"fixture-secret"}"#),
        api_user: "7".into(),
        extra_config: "{}".into(),
    }
}
fn key(n: u64) -> RequestKey {
    RequestKey {
        environment: EnvironmentId::Local("fixture".into()),
        platform: Platform::Claude,
        generation: n,
        request_id: n,
    }
}
fn run(services: &Services, operation: Operation, n: u64) -> Arc<OperationState> {
    services.start_checkin(key(n), operation).unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(15);
    loop {
        let state = services.checkin_operation_state();
        if !state.busy {
            return state;
        }
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(10));
    }
}
#[test]
fn encrypted_crud_isolated_and_detached_results_survive() {
    let root = tempfile::tempdir().unwrap();
    let other = tempfile::tempdir().unwrap();
    let runtime = RuntimeOwner::new().unwrap();
    let services = Services::new(runtime.handle(), source(root.path()));
    let observer = services.observe_checkin();
    drop(observer);
    let result = run(
        &services,
        Operation::AddProvider(provider("https://fixture.invalid")),
        1,
    );
    assert_eq!(result.result, Some(Ok(Outcome::Saved)));
    let id = result.snapshot.as_ref().unwrap().providers[0].id.clone();
    let result = run(&services, Operation::AddAccount(account(id.clone())), 2);
    let account_id = result.snapshot.as_ref().unwrap().accounts[0].id.clone();
    let storage = services
        .checkin_operations
        .storage(&services.source)
        .unwrap();
    let encrypted = storage
        .accounts()
        .get(&account_id)
        .unwrap()
        .cookies_json_encrypted;
    assert!(!encrypted.contains("fixture-secret"));
    assert_eq!(
        storage
            .accounts()
            .get_cookies_json(&account_id)
            .unwrap()
            .0
            .expose(),
        r#"{"session":"fixture-secret"}"#
    );
    let request = UpdateAccountRequest {
        name: Some("Renamed".into()),
        cookies_json: None,
        api_user: None,
        enabled: Some(false),
        extra_config: None,
    };
    assert_eq!(
        run(
            &services,
            Operation::UpdateAccount(account_id.clone(), request),
            3
        )
        .result,
        Some(Ok(Outcome::Saved))
    );
    assert_eq!(
        storage
            .accounts()
            .get(&account_id)
            .unwrap()
            .cookies_json_encrypted,
        encrypted
    );
    assert_eq!(
        run(&services, Operation::DeleteProvider(id.clone()), 4).result,
        Some(Err(OperationError::ProviderInUse))
    );
    assert_eq!(
        run(&services, Operation::Balance(account_id.clone()), 5).result,
        Some(Err(OperationError::FixtureNetwork))
    );
    let service = CheckinService::with_db_and_client(
        storage.crypto.clone(),
        storage.db.clone(),
        reqwest::Client::new(),
    );
    let result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(service.checkin(&account_id))
        .unwrap();
    assert_eq!(result.status, CheckinStatus::Skipped);
    assert_eq!(
        run(&services, Operation::Records(account_id.clone()), 6)
            .snapshot
            .as_ref()
            .unwrap()
            .records
            .len(),
        1
    );
    let second = Services::new(runtime.handle(), source(other.path()));
    assert!(
        run(&second, Operation::Manage, 7)
            .snapshot
            .as_ref()
            .unwrap()
            .accounts
            .is_empty()
    );
    assert_eq!(
        run(&services, Operation::DeleteAccount(account_id), 8).result,
        Some(Ok(Outcome::Deleted))
    );
    assert_eq!(
        run(&services, Operation::DeleteProvider(id), 9).result,
        Some(Ok(Outcome::Deleted))
    );
    let late = services.observe_checkin();
    assert_eq!(late.borrow().result, Some(Ok(Outcome::Deleted)));
    runtime.shutdown();
    assert_eq!(
        services.start_checkin(key(10), Operation::Manage),
        Err(OperationError::Interrupted)
    );
}
#[test]
fn real_loopback_checkin_and_balance_use_captured_database_and_crypto() {
    let root = tempfile::tempdir().unwrap();
    let runtime = RuntimeOwner::new().unwrap();
    let http = tokio::runtime::Runtime::new().unwrap();
    let listener = http
        .block_on(tokio::net::TcpListener::bind("127.0.0.1:0"))
        .unwrap();
    let addr = listener.local_addr().unwrap();
    let server=http.spawn(async move {
        for expected in ["GET /api/user/self", "POST /api/user/checkin", "GET /api/user/self"] {
            let (mut stream,_)=listener.accept().await.unwrap();let mut data=Vec::new();let mut buf=[0u8;4096];
            while !data.windows(4).any(|s|s==b"\r\n\r\n") {let n=stream.read(&mut buf).await.unwrap();assert!(n>0);data.extend_from_slice(&buf[..n]);}
            let request=String::from_utf8(data).unwrap();assert!(request.starts_with(expected));assert!(request.to_ascii_lowercase().contains("cookie: session=fixture-secret"));
            let body=if expected.starts_with("POST") {r#"{"success":true,"message":"ok","data":{"reward":"1 USD"}}"#}else{r#"{"success":true,"data":{"quota":500000,"used_quota":100000}}"#};
            let response=format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",body.len(),body);
            stream.write_all(response.as_bytes()).await.unwrap();
        }
    });
    let mut services = Services::new(runtime.handle(), source(root.path()));
    services.checkin_operations = Arc::new(CheckinOperations {
        loopback_fixture: true,
        ..Default::default()
    });
    let id = run(
        &services,
        Operation::AddProvider(provider(&format!("http://{addr}"))),
        1,
    )
    .snapshot
    .as_ref()
    .unwrap()
    .providers[0]
        .id
        .clone();
    let id = run(&services, Operation::AddAccount(account(id)), 2)
        .snapshot
        .as_ref()
        .unwrap()
        .accounts[0]
        .id
        .clone();
    assert_eq!(
        run(&services, Operation::Checkin(id.clone()), 3).result,
        Some(Ok(Outcome::Checked(CheckinStatus::Success)))
    );
    assert_eq!(
        run(&services, Operation::Balance(id.clone()), 4).result,
        Some(Ok(Outcome::BalanceUpdated))
    );
    http.block_on(async {
        tokio::time::timeout(Duration::from_secs(5), server)
            .await
            .unwrap()
            .unwrap()
    });
    let storage = services
        .checkin_operations
        .storage(&services.source)
        .unwrap();
    assert!(
        BalanceManager::with_db(storage.db.clone())
            .get_latest(&id)
            .unwrap()
            .is_some()
    );
    assert_eq!(
        run(&services, Operation::Records(id), 5)
            .snapshot
            .as_ref()
            .unwrap()
            .records[0]
            .status,
        CheckinStatus::Success
    );
    runtime.shutdown();
}
