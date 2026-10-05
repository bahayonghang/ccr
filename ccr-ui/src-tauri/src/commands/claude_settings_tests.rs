use super::*;
use crate::platform::{CliStatus, PlatformInfo};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use tokio::sync::{Barrier, RwLock};

struct ReadBarrier {
    started: Barrier,
    resume: Barrier,
}

impl ReadBarrier {
    fn new() -> Self {
        Self {
            started: Barrier::new(2),
            resume: Barrier::new(2),
        }
    }
}

struct RemoteSettingsEnvironment {
    id: &'static str,
    content: String,
    reads: AtomicUsize,
    writes: Mutex<Vec<Value>>,
    barrier: Option<Arc<ReadBarrier>>,
}

impl RemoteSettingsEnvironment {
    fn new(id: &'static str, barrier: Option<Arc<ReadBarrier>>) -> Self {
        Self {
            id,
            content: json!({ "env": { "OWNER": id }, "future_option": [id] }).to_string(),
            reads: AtomicUsize::new(0),
            writes: Mutex::new(Vec::new()),
            barrier,
        }
    }
}

#[async_trait::async_trait]
impl ExecutionEnvironment for RemoteSettingsEnvironment {
    fn env_type(&self) -> EnvironmentType {
        EnvironmentType::Ssh
    }
    fn display_name(&self) -> String {
        self.id.to_string()
    }
    fn env_id(&self) -> String {
        self.id.to_string()
    }
    async fn list_platforms(&self) -> Result<Vec<PlatformInfo>, EnvError> {
        Ok(Vec::new())
    }
    async fn detect_cli_status(&self) -> Result<Vec<CliStatus>, EnvError> {
        Ok(Vec::new())
    }
    async fn read_config(&self, platform: &str, path: &str) -> Result<String, EnvError> {
        assert_eq!((platform, path), ("claude", "settings.json"));
        self.reads.fetch_add(1, Ordering::SeqCst);
        if let Some(barrier) = &self.barrier {
            barrier.started.wait().await;
            barrier.resume.wait().await;
        }
        Ok(self.content.clone())
    }
    async fn write_config(
        &self,
        platform: &str,
        path: &str,
        content: &str,
    ) -> Result<(), EnvError> {
        assert_eq!((platform, path), ("claude", "settings.json"));
        self.writes
            .lock()
            .unwrap()
            .push(serde_json::from_str(content).unwrap());
        Ok(())
    }
}

#[tokio::test]
async fn settings_update_remains_on_captured_environment_after_switch() {
    let barrier = Arc::new(ReadBarrier::new());
    let first = Arc::new(RemoteSettingsEnvironment::new(
        "ssh:a",
        Some(barrier.clone()),
    ));
    let second = Arc::new(RemoteSettingsEnvironment::new("ssh:b", None));
    let mut registry = EnvironmentRegistry::new();
    registry.register(first.clone());
    registry.register(second.clone());
    let registry = RwLock::new(registry);
    let update = update_settings_from_registry(&registry, None, |settings| {
        settings.env.insert("EDIT".into(), "saved".into());
        Ok(())
    });
    let switch = async {
        barrier.started.wait().await;
        registry.write().await.switch_by_id("ssh:b").unwrap();
        barrier.resume.wait().await;
    };
    let (result, ()) = tokio::time::timeout(Duration::from_secs(5), async {
        tokio::join!(update, switch)
    })
    .await
    .expect("environment-switch test completed");
    result.unwrap();
    let writes = first.writes.lock().unwrap();
    assert_eq!(
        writes.len(),
        1,
        "the captured environment must own the write"
    );
    assert_eq!(writes[0]["env"]["EDIT"], "saved");
    assert_eq!(writes[0]["future_option"], json!(["ssh:a"]));
    assert!(
        second.writes.lock().unwrap().is_empty(),
        "new active environment must not receive A's settings"
    );
    assert_eq!(second.reads.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn settings_expected_environment_mismatch_rejects_read_and_update_before_io() {
    let first = Arc::new(RemoteSettingsEnvironment::new("ssh:a", None));
    let second = Arc::new(RemoteSettingsEnvironment::new("ssh:b", None));
    let mut registry = EnvironmentRegistry::new();
    registry.register(first.clone());
    registry.register(second.clone());
    registry.switch_by_id("ssh:b").unwrap();
    let registry = RwLock::new(registry);

    // The frontend observed A, but B became active before command admission.
    let read_error = read_settings_from_registry(&registry, Some("ssh:a"))
        .await
        .unwrap_err();
    let update_error =
        update_settings_from_registry(&registry, Some("ssh:a"), |_| -> Result<(), String> {
            panic!("mismatch must reject before mutation")
        })
        .await
        .unwrap_err();
    assert_eq!(read_error, settings_environment_changed());
    assert_eq!(update_error, settings_environment_changed());
    for environment in [first, second] {
        assert_eq!(environment.reads.load(Ordering::SeqCst), 0);
        assert!(environment.writes.lock().unwrap().is_empty());
    }
}

#[tokio::test]
async fn settings_expected_environment_read_keeps_its_admitted_target() {
    let barrier = Arc::new(ReadBarrier::new());
    let first = Arc::new(RemoteSettingsEnvironment::new(
        "ssh:a",
        Some(barrier.clone()),
    ));
    let second = Arc::new(RemoteSettingsEnvironment::new("ssh:b", None));
    let mut registry = EnvironmentRegistry::new();
    registry.register(first.clone());
    registry.register(second.clone());
    let registry = RwLock::new(registry);
    let read = read_settings_from_registry(&registry, Some("ssh:a"));
    let switch = async {
        barrier.started.wait().await;
        registry.write().await.switch_by_id("ssh:b").unwrap();
        barrier.resume.wait().await;
    };
    let (result, ()) =
        tokio::time::timeout(Duration::from_secs(5), async { tokio::join!(read, switch) })
            .await
            .expect("admitted read completed");
    assert_eq!(result.unwrap()["env"]["OWNER"], "ssh:a");
    assert_eq!(registry.read().await.active().unwrap().env_id(), "ssh:b");
    assert_eq!(first.reads.load(Ordering::SeqCst), 1);
    assert_eq!(second.reads.load(Ordering::SeqCst), 0);
    assert!(first.writes.lock().unwrap().is_empty());
    assert!(second.writes.lock().unwrap().is_empty());
}

#[tokio::test]
async fn settings_expected_environment_update_keeps_original_arc_after_a_b_a_replacement() {
    let barrier = Arc::new(ReadBarrier::new());
    let first = Arc::new(RemoteSettingsEnvironment::new(
        "ssh:a",
        Some(barrier.clone()),
    ));
    let second = Arc::new(RemoteSettingsEnvironment::new("ssh:b", None));
    let replacement = Arc::new(RemoteSettingsEnvironment::new("ssh:a", None));
    let mut registry = EnvironmentRegistry::new();
    registry.register(first.clone());
    registry.register(second.clone());
    let registry = RwLock::new(registry);
    let update = update_settings_from_registry(&registry, Some("ssh:a"), |settings| {
        settings.env.insert("EDIT".into(), "saved".into());
        Ok(())
    });
    let switch = async {
        barrier.started.wait().await;
        {
            let mut registry = registry.write().await;
            registry.switch_by_id("ssh:b").unwrap();
            // refresh_environments may rebuild an environment with the same ID.
            registry.clear();
            registry.register(replacement.clone());
            registry.register(second.clone());
            registry.switch_by_id("ssh:a").unwrap();
        }
        barrier.resume.wait().await;
    };
    let (result, ()) = tokio::time::timeout(Duration::from_secs(5), async {
        tokio::join!(update, switch)
    })
    .await
    .expect("A-B-A update completed");
    result.unwrap();
    let writes = first.writes.lock().unwrap();
    assert_eq!(writes.len(), 1);
    assert_eq!(writes[0]["env"]["EDIT"], "saved");
    assert_eq!(first.reads.load(Ordering::SeqCst), 1);
    for environment in [second, replacement] {
        assert_eq!(environment.reads.load(Ordering::SeqCst), 0);
        assert!(environment.writes.lock().unwrap().is_empty());
    }
}

#[tokio::test]
async fn settings_empty_registry_only_preserves_unbound_local_fallback() {
    let registry = RwLock::new(EnvironmentRegistry::new());
    let legacy = capture_settings_environment(&registry, None).await.unwrap();
    assert_eq!(legacy.env_type(), EnvironmentType::Local);
    assert_eq!(legacy.env_id(), "local");
    for expected in ["local", "", "ssh:a"] {
        let result = capture_settings_environment(&registry, Some(expected)).await;
        assert!(matches!(result, Err(error) if error == settings_environment_changed()));
    }
}

#[tokio::test]
async fn settings_matching_local_environment_keeps_atomic_settings_manager() {
    let temp = tempfile::tempdir().unwrap();
    let settings_path = temp.path().join("settings.json");
    let backups = temp.path().join("backups");
    let locks = temp.path().join("locks");
    let mut process_env = crate::test_support::TestProcessEnv::new();
    process_env.set("CCR_SETTINGS_PATH", settings_path.as_os_str());
    process_env.set("CCR_BACKUP_DIR", backups.as_os_str());
    process_env.set("CCR_LOCK_DIR", locks.as_os_str());
    std::fs::write(
        &settings_path,
        r#"{"env":{"KEEP":"local"},"extension":{"active":true}}"#,
    )
    .unwrap();
    let mut registry = EnvironmentRegistry::new();
    registry.register(Arc::new(LocalEnvironment::new()));
    let registry = RwLock::new(registry);
    update_settings_from_registry(&registry, Some("local"), |settings| {
        settings.env.insert("EDIT".into(), "saved".into());
        Ok(())
    })
    .await
    .unwrap();
    let saved: Value = serde_json::from_slice(&std::fs::read(&settings_path).unwrap()).unwrap();
    assert_eq!(saved["env"], json!({ "KEEP": "local", "EDIT": "saved" }));
    assert_eq!(saved["extension"], json!({ "active": true }));
    assert_eq!(std::fs::read_dir(&backups).unwrap().count(), 1);
}
