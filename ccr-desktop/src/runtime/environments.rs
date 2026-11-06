//! Environment reads stay on the blocking worker. Render does not spawn processes.
use super::Services;
use ccr_app::environments::{EnvironmentReport, InspectOptions, admit_local_only};
use ccr_app::runtime::{AdmissionError, RequestKey, TaskSubscription};

pub fn load(
    services: &Services,
    key: RequestKey,
) -> Result<TaskSubscription<EnvironmentReport, &'static str>, AdmissionError> {
    let environment = key.environment.clone();
    let live = !services.is_fixture();
    let delay = services.fixture_delay();
    services.executor.read(key, move |cancel, _| {
        admit_local_only(&environment)?;
        // Fixture delay is cancellable QA only. A live read does not sleep here.
        if !delay.is_zero() {
            let deadline = std::time::Instant::now() + delay;
            while !cancel.is_cancelled() && std::time::Instant::now() < deadline {
                std::thread::sleep(
                    deadline
                        .saturating_duration_since(std::time::Instant::now())
                        .min(std::time::Duration::from_millis(10)),
                );
            }
        }
        let mut options = if live {
            InspectOptions::live()
        } else {
            InspectOptions::fixture()
        };
        // WSL discovery is explicitly deferred; refreshing settings never probes it.
        options.query_wsl = false;
        Ok(ccr_app::environments::inspect(options, &|| {
            cancel.is_cancelled()
        }))
    })
}

use ccr_app::environments::ssh::{ConfigFile, Document, Host, HostInput, Probe, Workspace};
use ccr_app::runtime::Completion;
use std::sync::Arc;

pub enum SshAction {
    Hosts,
    HostDetails(String),
    SaveHost(HostInput),
    DeleteHost(String),
    Probe(String),
    Confirm(String),
    Connect(String),
    Disconnect,
    Read(String, ConfigFile),
    Save(Document, String),
}
pub enum SshResult {
    Hosts(Vec<Host>),
    HostDetails(HostInput),
    HostSaved(Host),
    HostDeleted,
    Probe(Probe),
    Connected,
    Disconnected,
    Confirmed,
    Document(Document),
}
impl Services {
    fn ssh_workspace(&self) -> Result<Arc<Workspace>, &'static str> {
        if let Some(owner) = self.ssh.get() {
            return Ok(owner.clone());
        }
        let (database, known_hosts) = match &self.source {
            super::ConfigSource::Local => (
                Some(ccr_db::database::ui_database_path().map_err(|_| "ssh_database_unavailable")?),
                ccr_app::environments::ssh_security::app_known_hosts_path()
                    .map_err(|_| "ssh_database_unavailable")?,
            ),
            super::ConfigSource::Fixture { registry, .. } => {
                if !registry.is_absolute() {
                    return Err("ssh_database_unavailable");
                }
                let root = registry.parent().ok_or("ssh_database_unavailable")?;
                let database = root.join("ssh.db");
                (Some(database), root.join("ssh/known_hosts"))
            }
        };
        let owner = Arc::new(Workspace::new(database, known_hosts));
        let _ = self.ssh.set(owner);
        self.ssh.get().cloned().ok_or("ssh_database_unavailable")
    }
    pub fn ssh_action(
        &self,
        key: RequestKey,
        action: SshAction,
    ) -> Result<TaskSubscription<SshResult, &'static str>, AdmissionError> {
        let services = self.clone();
        let environment = key.environment.clone();
        self.executor.job(key, move |cancel, _| {
            let work = async {
                let target = match &action {
                    SshAction::Probe(id) | SshAction::Connect(id) | SshAction::Read(id, _) => {
                        Some(id)
                    }
                    SshAction::Save(document, _) => Some(&document.environment),
                    _ => None,
                };
                if let Some(target) = target
                    && environment != ccr_app::runtime::EnvironmentId::Ssh(target.clone())
                {
                    return Err("ssh_environment_mismatch");
                }
                // Fixture launches never reach a live transport. Tests explicitly inject
                // an in-memory owner before invoking the same UI/runtime actions.
                if services.is_fixture()
                    && !owner_is_fixture_transport(&services)
                    && matches!(
                        &action,
                        SshAction::Probe(_)
                            | SshAction::Connect(_)
                            | SshAction::Confirm(_)
                            | SshAction::Read(_, _)
                            | SshAction::Save(_, _)
                    )
                {
                    return Err("ssh_fixture_offline");
                }
                let owner = services.ssh_workspace()?;
                match action {
                    SshAction::Hosts => owner.hosts().map(SshResult::Hosts),
                    SshAction::HostDetails(id) => {
                        owner.host_details(&id).map(SshResult::HostDetails)
                    }
                    SshAction::SaveHost(input) => {
                        owner.save_host(input).await.map(SshResult::HostSaved)
                    }
                    SshAction::DeleteHost(id) => owner
                        .delete_host(&id)
                        .await
                        .map(|()| SshResult::HostDeleted),
                    SshAction::Probe(id) => {
                        owner.probe(&id, cancel.clone()).await.map(SshResult::Probe)
                    }
                    SshAction::Confirm(id) => owner
                        .confirm(&id, cancel.clone())
                        .await
                        .map(|()| SshResult::Confirmed),
                    SshAction::Connect(id) => owner
                        .connect(&id, cancel.clone())
                        .await
                        .map(|()| SshResult::Connected),
                    SshAction::Disconnect => {
                        owner.disconnect().await;
                        Ok(SshResult::Disconnected)
                    }
                    SshAction::Read(id, file) => owner
                        .read(&id, file, cancel.clone())
                        .await
                        .map(SshResult::Document),
                    SshAction::Save(document, content) => owner
                        .save(&document, &content, cancel.clone())
                        .await
                        .map(SshResult::Document),
                }
            };
            let result = tokio::runtime::Handle::current().block_on(work);
            services.monitoring.terminal(
                &services.source,
                "environment",
                if result.is_ok() {
                    "environment.operation.completed"
                } else {
                    "environment.operation.failed"
                },
                result.is_err(),
            );
            match result {
                Ok(value) => Completion::Success(value),
                Err("ssh_write_outcome_unknown") => Completion::Failed("ssh_write_outcome_unknown"),
                Err("ssh_cleanup_failed") => Completion::CleanupFailed("ssh_cleanup_failed"),
                Err("ssh_cancelled") => Completion::Cancelled,
                Err(error) => Completion::Failed(error),
            }
        })
    }
}

fn owner_is_fixture_transport(services: &Services) -> bool {
    #[cfg(test)]
    {
        services.ssh.get().is_some_and(|owner| owner.is_fixture())
    }
    #[cfg(not(test))]
    {
        let _ = services;
        false
    }
}

#[cfg(test)]
mod ssh_tests {
    use super::*;
    use ccr_app::runtime::{EnvironmentId, Platform, RuntimeOwner, TaskEvent};
    #[gpui_kit::test]
    async fn fixture_host_store_never_enables_live_ssh(cx: &mut gpui_kit::TestAppContext) {
        cx.executor().allow_parking();
        let root = tempfile::tempdir().unwrap_or_else(|_| panic!("root"));
        let owner = RuntimeOwner::new().unwrap_or_else(|_| panic!("runtime"));
        let services = Services::new(
            owner.handle(),
            super::super::ConfigSource::Fixture {
                profiles: root.path().join("profiles.toml"),
                registry: root.path().join("registry.toml"),
                delay: std::time::Duration::ZERO,
            },
        );
        let key = RequestKey {
            environment: EnvironmentId::Local("local".into()),
            platform: Platform::Claude,
            generation: 1,
            request_id: 1,
        };
        let mut save = services
            .ssh_action(
                key.clone(),
                SshAction::SaveHost(HostInput {
                    id: None,
                    name: "Offline fixture".into(),
                    host: "must-not-connect.example".into(),
                    port: 22,
                    user: "fixture".into(),
                    remote_home: "~".into(),
                }),
            )
            .unwrap_or_else(|_| panic!("save admission"));
        let Some(TaskEvent::Terminal(result)) = save.next().await else {
            panic!("save result")
        };
        let Completion::Success(SshResult::HostSaved(host)) = result.as_ref() else {
            panic!("host save")
        };
        assert!(root.path().join("ssh.db").is_file());
        let mut connect = services
            .ssh_action(
                RequestKey {
                    environment: EnvironmentId::Ssh(host.id.clone()),
                    ..key
                },
                SshAction::Connect(host.id.clone()),
            )
            .unwrap_or_else(|_| panic!("connect admission"));
        assert!(
            matches!(connect.next().await,Some(TaskEvent::Terminal(result)) if matches!(result.as_ref(),Completion::Failed("ssh_fixture_offline")))
        );
        assert!(!root.path().join("ssh/known_hosts").exists());
        owner.shutdown();
    }
}
