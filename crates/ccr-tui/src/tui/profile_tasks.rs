//! Bounded background Profile reads and one complete synchronous mutation.
//! Only read generations can be discarded; mutation outcomes are always delivered.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, mpsc};

use ccr_cli::application::profile_activity::{
    ProfileActivityError, ProfileActivitySnapshot, load_profile_activity,
};
use ccr_cli::application::profile_edit::{EditProfileOutcome, edit_profile};
use ccr_cli::application::profile_lifecycle::{ApplyProfileRequest, ProfileOutcome};
use ccr_cli::application::{ProfileOffResult, profile_off_for_platform};
use ccr_cli::managers::ConfigManager;
use ccr_cli::models::{
    ClaudeRuntimeSummary, CodexRuntimeSummary, Platform, PlatformConfig, PlatformPaths,
    ProfileConfig,
};
use ccr_cli::platforms::{GrokActivationState, GrokPlatform, create_platform};
use ccr_core::core::error::CcrError;
use indexmap::IndexMap;

use super::profile_edit::EditRequest;
use super::runtime::AsyncTaskExecutor;

pub(super) struct ReadIssue {
    pub locations: Vec<String>,
    pub reason: String,
}

pub(super) struct ProfileSnapshot {
    pub instance: Option<Arc<dyn PlatformConfig>>,
    pub configs: std::result::Result<IndexMap<String, ProfileConfig>, ReadIssue>,
    pub current: std::result::Result<Option<String>, ReadIssue>,
    pub claude_runtime: Option<ClaudeRuntimeSummary>,
    pub codex_runtime: Option<CodexRuntimeSummary>,
    pub grok_activation: Option<GrokActivationState>,
    pub runtime_error: Option<String>,
    pub activity: Option<std::result::Result<ProfileActivitySnapshot, ProfileActivityError>>,
    /// `profiles.toml` path resolved while reading the snapshot. Renderers must not resolve it again.
    pub profiles_path: String,
    /// Catalog revision captured with the snapshot. `None` means the editor must ask for a reload.
    pub revision: Option<String>,
}

pub(super) fn read_profiles(platform: Platform) -> std::result::Result<ProfileSnapshot, String> {
    let instance = create_platform(platform).map_err(|error| error.to_string())?;
    let mut snapshot = read_instance_profiles(instance.as_ref());
    snapshot.instance = Some(instance);
    match platform {
        Platform::Claude => match ccr_cli::services::ClaudeAuthService::new()
            .and_then(|service| service.get_runtime_summary())
        {
            Ok(summary) => snapshot.claude_runtime = Some(summary),
            Err(error) => snapshot.runtime_error = Some(error.to_string()),
        },
        Platform::Codex => match ccr_cli::services::CodexAuthService::new()
            .and_then(|service| service.get_runtime_summary())
        {
            Ok(summary) => snapshot.codex_runtime = Some(summary),
            Err(error) => snapshot.runtime_error = Some(error.to_string()),
        },
        Platform::Grok => {
            match GrokPlatform::new().and_then(|grok| grok.inspect_activation_state()) {
                Ok(state) => {
                    // A recorded pointer with drift is not an effective current profile.
                    snapshot.current = Ok(match &state {
                        GrokActivationState::Active { name } => Some(name.clone()),
                        _ => None,
                    });
                    snapshot.grok_activation = Some(state);
                }
                Err(error) => snapshot.runtime_error = Some(error.to_string()),
            }
        }
        _ => {}
    }
    if let Ok(configs) = &snapshot.configs {
        let names = configs.keys().cloned().collect::<Vec<_>>();
        snapshot.activity = Some(load_profile_activity(platform, &names));
    }
    Ok(snapshot)
}

// Separately injectable repository seam: no auth service or history discovery.
pub(super) fn read_instance_profiles(instance: &dyn PlatformConfig) -> ProfileSnapshot {
    let platform = instance.platform_type();
    let paths = PlatformPaths::new(platform).ok();
    let profiles_path = paths
        .as_ref()
        .map(|paths| paths.profiles_file.display().to_string())
        .unwrap_or_else(|| format!("~/.ccr/platforms/{}/profiles.toml", platform.short_name()));
    let registry_path = paths
        .as_ref()
        .map(|paths| paths.registry_file.display().to_string())
        .unwrap_or_else(|| "~/.ccr/config.toml".to_string());
    let mut current_paths = vec![registry_path];
    if platform == Platform::Codex {
        current_paths.push(profiles_path.clone());
    }
    let current = if platform == Platform::Grok {
        // Production resolves this once using the richer typed activation state.
        Ok(None)
    } else {
        instance.get_current_profile().map_err(|error| ReadIssue {
            locations: current_paths,
            reason: error.to_string(),
        })
    };
    let configs = instance.load_profiles().map_err(|error| ReadIssue {
        locations: vec![profiles_path.clone()],
        reason: error.to_string(),
    });
    let revision = paths
        .as_ref()
        .and_then(|paths| catalog_revision(&paths.profiles_file));
    ProfileSnapshot {
        instance: None,
        configs,
        current,
        claude_runtime: None,
        codex_runtime: None,
        grok_activation: None,
        runtime_error: None,
        activity: None,
        profiles_path,
        revision,
    }
}

fn catalog_revision(path: &std::path::Path) -> Option<String> {
    match ConfigManager::new(path).snapshot() {
        Ok(snapshot) => Some(snapshot.version),
        Err(error) => {
            if !matches!(error, CcrError::ConfigMissing(_)) {
                tracing::warn!("Profile catalog revision unavailable: {error}");
            }
            None
        }
    }
}

#[derive(Clone)]
pub(super) enum MutationRequest {
    Apply(ApplyProfileRequest),
    Off(Platform),
    Edit(Box<EditRequest>),
}

impl MutationRequest {
    pub fn platform(&self) -> Platform {
        match self {
            Self::Apply(request) => request.platform,
            Self::Off(platform) => *platform,
            Self::Edit(request) => request.platform(),
        }
    }
    pub fn profile(&self) -> Option<&str> {
        match self {
            Self::Apply(request) => Some(&request.name),
            Self::Off(_) => None,
            Self::Edit(request) => Some(request.name()),
        }
    }
}

pub(super) enum MutationOutcome {
    Applied(ProfileOutcome),
    Off(ProfileOffResult),
    Edited(Box<EditProfileOutcome>),
}

pub(super) type ProfileLoader =
    Arc<dyn Fn(Platform) -> std::result::Result<ProfileSnapshot, String> + Send + Sync>;
pub(super) type ProfileWriter =
    Arc<dyn Fn(MutationRequest) -> std::result::Result<MutationOutcome, String> + Send + Sync>;

fn mutate(request: MutationRequest) -> std::result::Result<MutationOutcome, String> {
    // The entire lifecycle, including its thread-local journal, stays in this worker.
    match request {
        MutationRequest::Apply(request) => {
            super::profile_backend::apply(request).map(MutationOutcome::Applied)
        }
        MutationRequest::Off(platform) => {
            profile_off_for_platform(platform).map(MutationOutcome::Off)
        }
        MutationRequest::Edit(request) => edit_profile(request.into_inner())
            .map(|outcome| MutationOutcome::Edited(Box::new(outcome))),
    }
    .map_err(|error| error.to_string())
}

struct ReadJob {
    platform: Platform,
    generation: u64,
    receiver: mpsc::Receiver<std::result::Result<ProfileSnapshot, String>>,
}
struct MutationJob {
    request: MutationRequest,
    receiver: mpsc::Receiver<std::result::Result<MutationOutcome, String>>,
}

pub(super) enum ProfileCompletion {
    Read(Platform, std::result::Result<Box<ProfileSnapshot>, String>),
    Mutation(
        MutationRequest,
        std::result::Result<MutationOutcome, String>,
    ),
    ReadInterrupted(Platform),
    MutationInterrupted(MutationRequest),
}

pub(crate) struct ProfileTasks {
    executor: AsyncTaskExecutor,
    loader: ProfileLoader,
    writer: ProfileWriter,
    reads: Vec<ReadJob>,
    generations: HashMap<Platform, u64>,
    queued: HashSet<Platform>,
    mutation: Option<MutationJob>,
    pub(super) exit_requested: bool,
    pub(super) time_refresh: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for ProfileTasks {
    fn default() -> Self {
        Self::new(AsyncTaskExecutor::Disabled)
    }
}

impl ProfileTasks {
    pub(super) fn new(executor: AsyncTaskExecutor) -> Self {
        Self {
            executor,
            loader: Arc::new(read_profiles),
            writer: Arc::new(mutate),
            reads: Vec::new(),
            generations: HashMap::new(),
            queued: HashSet::new(),
            mutation: None,
            exit_requested: false,
            time_refresh: None,
        }
    }

    #[cfg(test)]
    pub(super) fn with_backends(
        executor: AsyncTaskExecutor,
        loader: ProfileLoader,
        writer: ProfileWriter,
    ) -> Self {
        Self {
            loader,
            writer,
            ..Self::new(executor)
        }
    }

    pub(super) fn available(&self) -> bool {
        !matches!(self.executor, AsyncTaskExecutor::Disabled)
    }

    pub(super) fn mutation_pending(&self, platform: Platform, profile: Option<&str>) -> bool {
        self.mutation.as_ref().is_some_and(|job| {
            job.request.platform() == platform
                && profile
                    .is_none_or(|name| job.request.profile().is_none_or(|target| name == target))
        })
    }

    pub(super) fn mutation_active(&self) -> bool {
        self.mutation.is_some()
    }
    pub(super) fn pending(&self) -> bool {
        !self.reads.is_empty() || self.mutation_active()
    }
    pub(super) fn reading(&self, platform: Platform) -> bool {
        self.reads.iter().any(|job| job.platform == platform) || self.queued.contains(&platform)
    }
    pub(super) fn ready_to_exit(&self) -> bool {
        self.exit_requested && !self.mutation_active()
    }

    pub(super) fn request_read(&mut self, platform: Platform, refresh: bool) -> bool {
        if !self.available() || self.exit_requested {
            return false;
        }
        if refresh {
            *self.generations.entry(platform).or_default() += 1;
        }
        if self.reads.iter().any(|job| job.platform == platform)
            || self.mutation_pending(platform, None)
        {
            if refresh {
                self.queued.insert(platform);
            }
            return false;
        }
        self.queued.remove(&platform);
        let generation = *self.generations.entry(platform).or_default();
        let (sender, receiver) = mpsc::channel();
        let loader = Arc::clone(&self.loader);
        self.executor.spawn_blocking(move || {
            let _ = sender.send(loader(platform));
        });
        self.reads.push(ReadJob {
            platform,
            generation,
            receiver,
        });
        true
    }

    pub(super) fn start_mutation(&mut self, request: MutationRequest) -> bool {
        if !self.available() || self.mutation_active() || self.exit_requested {
            return false;
        }
        let platform = request.platform();
        *self.generations.entry(platform).or_default() += 1;
        // Any concurrent pre-write read is obsolete. One fresh read follows the outcome.
        self.queued.insert(platform);
        let writer = Arc::clone(&self.writer);
        let worker_request = request.clone();
        let (sender, receiver) = mpsc::channel();
        self.executor.spawn_blocking(move || {
            let _ = sender.send(writer(worker_request));
        });
        self.mutation = Some(MutationJob { request, receiver });
        true
    }

    pub(super) fn drain(&mut self) -> Vec<ProfileCompletion> {
        let mut completed = Vec::new();
        if let Some(job) = self.mutation.as_ref() {
            match job.receiver.try_recv() {
                Ok(result) => {
                    let request = job.request.clone();
                    self.mutation = None;
                    completed.push(ProfileCompletion::Mutation(request, result));
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    let request = job.request.clone();
                    self.mutation = None;
                    completed.push(ProfileCompletion::MutationInterrupted(request));
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }
        let mut index = 0;
        while index < self.reads.len() {
            let job = &self.reads[index];
            let result = match job.receiver.try_recv() {
                Err(mpsc::TryRecvError::Empty) => {
                    index += 1;
                    continue;
                }
                result => result,
            };
            let job = self.reads.swap_remove(index);
            if self.generations.get(&job.platform) == Some(&job.generation) {
                completed.push(match result {
                    Ok(snapshot) => ProfileCompletion::Read(job.platform, snapshot.map(Box::new)),
                    Err(_) => ProfileCompletion::ReadInterrupted(job.platform),
                });
            }
        }
        for platform in self.queued.clone() {
            self.request_read(platform, false);
        }
        completed
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use ccr_cli::application::profile_lifecycle::ProfileStatus;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{Duration, Instant};

    pub(super) fn snapshot(name: &str) -> ProfileSnapshot {
        ProfileSnapshot {
            instance: None,
            configs: Ok(IndexMap::from([(
                name.to_owned(),
                ProfileConfig::default(),
            )])),
            current: Ok(Some(name.to_owned())),
            claude_runtime: None,
            codex_runtime: None,
            grok_activation: None,
            runtime_error: None,
            activity: Some(Ok(ProfileActivitySnapshot::default())),
            profiles_path: "fixture/profiles.toml".to_string(),
            revision: Some("rev-fixture".to_string()),
        }
    }

    fn applied(request: MutationRequest) -> std::result::Result<MutationOutcome, String> {
        let MutationRequest::Apply(request) = request else {
            return Err("fixture only applies".into());
        };
        Ok(MutationOutcome::Applied(ProfileOutcome {
            operation_id: request.operation_id,
            platform: request.platform.short_name().into(),
            profile: request.name,
            previous_profile: None,
            status: ProfileStatus::Applied,
            activation_committed: true,
            warnings: vec![],
            recovery_paths: vec![],
        }))
    }

    fn drain_until(
        tasks: &mut ProfileTasks,
        predicate: impl Fn(&[ProfileCompletion]) -> bool,
    ) -> Vec<ProfileCompletion> {
        let mut completions = Vec::new();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            completions.extend(tasks.drain());
            if predicate(&completions) {
                return completions;
            }
            assert!(Instant::now() < deadline, "fixture worker did not complete");
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    #[test]
    fn missing_catalog_has_no_revision_and_creates_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("profiles.toml");
        assert!(catalog_revision(&path).is_none());
        assert!(!path.exists());
    }

    #[test]
    fn edit_outcome_is_consumed_once_and_a_second_writer_is_rejected() {
        use ccr_cli::application::profile_edit::{EditProfileRequest, FieldEdit, SecretEdit};
        use ccr_cli::models::ApplyPolicy;
        use serde_json::json;
        let request = MutationRequest::Edit(Box::new(EditRequest(EditProfileRequest {
            platform: Platform::Codex,
            name: "work".into(),
            rename: None,
            expected_revision: Some("rev-1".into()),
            fields: vec![FieldEdit::Set {
                field: "model".into(),
                value: json!("gpt"),
            }],
            secret: SecretEdit::Keep,
            apply: ApplyPolicy::IfCurrent,
        })));
        let mut tasks = ProfileTasks::with_backends(
            AsyncTaskExecutor::from_current_or_test(),
            Arc::new(|_| Ok(snapshot("work"))),
            Arc::new(|request| {
                let MutationRequest::Edit(edit) = request else {
                    return Err("expected edit".into());
                };
                let inner = edit.into_inner();
                Ok(MutationOutcome::Edited(Box::new(
                    ccr_cli::application::profile_edit::EditProfileOutcome {
                        platform: inner.platform,
                        old_name: inner.name.clone(),
                        new_name: inner.name,
                        changed_fields: vec!["model".into()],
                        reapplied: false,
                        revision: Some("rev-2".into()),
                        lifecycle: ccr_cli::application::profile_edit::EditLifecycle::Profile(
                            ProfileOutcome {
                                operation_id: "op".into(),
                                platform: "codex".into(),
                                profile: "work".into(),
                                previous_profile: None,
                                status: ProfileStatus::Applied,
                                activation_committed: false,
                                warnings: vec![],
                                recovery_paths: vec![],
                            },
                        ),
                        warnings: vec![],
                    },
                )))
            }),
        );
        assert!(tasks.start_mutation(request.clone()));
        assert!(!tasks.start_mutation(request));
        let outcomes = drain_until(&mut tasks, |events| {
            events
                .iter()
                .any(|event| matches!(event, ProfileCompletion::Mutation(_, Ok(_))))
        });
        assert!(matches!(
            &outcomes[0],
            ProfileCompletion::Mutation(
                MutationRequest::Edit(_),
                Ok(MutationOutcome::Edited(outcome))
            ) if outcome.new_name == "work"
        ));
    }

    #[test]
    fn disabled_executor_never_claims_work_started() {
        let mut tasks = ProfileTasks::default();
        assert!(!tasks.request_read(Platform::Codex, false));
        assert!(!tasks.start_mutation(MutationRequest::Off(Platform::Codex)));
        assert!(!tasks.pending());
        assert!(!tasks.reading(Platform::Codex));
    }

    #[test]
    fn late_pre_mutation_read_cannot_replace_committed_snapshot() {
        let (read_started_tx, read_started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let release_rx = std::sync::Mutex::new(release_rx);
        let reads = Arc::new(AtomicUsize::new(0));
        let counted_reads = Arc::clone(&reads);
        let writes = Arc::new(AtomicUsize::new(0));
        let counted_writes = Arc::clone(&writes);
        let mut tasks = ProfileTasks::with_backends(
            AsyncTaskExecutor::from_current_or_test(),
            Arc::new(move |_| {
                if counted_reads.fetch_add(1, Ordering::SeqCst) == 0 {
                    read_started_tx.send(()).unwrap();
                    let _ = release_rx
                        .lock()
                        .unwrap()
                        .recv_timeout(Duration::from_secs(5));
                    Ok(snapshot("old"))
                } else {
                    Ok(snapshot("new"))
                }
            }),
            Arc::new(move |request| {
                counted_writes.fetch_add(1, Ordering::SeqCst);
                applied(request)
            }),
        );
        assert!(tasks.request_read(Platform::Codex, false));
        read_started_rx
            .recv_timeout(Duration::from_secs(5))
            .unwrap();
        let request = MutationRequest::Apply(ApplyProfileRequest::new(Platform::Codex, "new"));
        assert!(tasks.start_mutation(request.clone()));
        assert!(!tasks.start_mutation(request));
        let outcomes = drain_until(&mut tasks, |events| {
            events
                .iter()
                .any(|event| matches!(event, ProfileCompletion::Mutation(_, _)))
        });
        assert!(
            outcomes
                .iter()
                .all(|event| !matches!(event, ProfileCompletion::Read(_, _)))
        );
        release_tx.send(()).unwrap();
        let outcomes = drain_until(&mut tasks, |events| {
            events
                .iter()
                .any(|event| matches!(event, ProfileCompletion::Read(_, _)))
        });
        assert_eq!(reads.load(Ordering::SeqCst), 2);
        assert_eq!(writes.load(Ordering::SeqCst), 1);
        assert_eq!(outcomes.len(), 1);
        let ProfileCompletion::Read(Platform::Codex, Ok(snapshot)) = &outcomes[0] else {
            panic!("expected one fresh snapshot");
        };
        assert_eq!(
            snapshot
                .current
                .as_ref()
                .ok()
                .and_then(|name| name.as_deref()),
            Some("new")
        );
    }

    #[test]
    fn repeated_refreshes_coalesce_and_old_results_are_discarded() {
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let release_rx = std::sync::Mutex::new(release_rx);
        let reads = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&reads);
        let mut tasks = ProfileTasks::with_backends(
            AsyncTaskExecutor::from_current_or_test(),
            Arc::new(move |_| {
                if counter.fetch_add(1, Ordering::SeqCst) == 0 {
                    started_tx.send(()).unwrap();
                    let _ = release_rx
                        .lock()
                        .unwrap()
                        .recv_timeout(Duration::from_secs(5));
                }
                Ok(snapshot("work"))
            }),
            Arc::new(applied),
        );
        tasks.request_read(Platform::Grok, false);
        started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        for _ in 0..100 {
            assert!(!tasks.request_read(Platform::Grok, true));
        }
        release_tx.send(()).unwrap();
        let outcomes = drain_until(&mut tasks, |events| {
            events
                .iter()
                .any(|event| matches!(event, ProfileCompletion::Read(_, _)))
        });
        assert_eq!(reads.load(Ordering::SeqCst), 2);
        assert_eq!(outcomes.len(), 1);
        assert!(!tasks.pending());
    }

    #[test]
    fn lost_mutation_channel_delivers_unknown_outcome_without_replay() {
        let mut tasks = ProfileTasks::default();
        let (sender, receiver) = mpsc::channel();
        drop(sender);
        tasks.mutation = Some(MutationJob {
            request: MutationRequest::Off(Platform::Grok),
            receiver,
        });
        tasks.exit_requested = true;
        assert!(!tasks.ready_to_exit());
        assert!(matches!(
            tasks.drain().as_slice(),
            [ProfileCompletion::MutationInterrupted(
                MutationRequest::Off(Platform::Grok)
            )]
        ));
        assert!(tasks.ready_to_exit());
        assert!(!tasks.pending());
        assert!(tasks.drain().is_empty());
    }

    #[test]
    #[ignore = "controlled fixture timing; report, never a wall-clock CI threshold"]
    fn profile_async_responsiveness_benchmark() {
        let reads = Arc::new(AtomicUsize::new(0));
        let counted = Arc::clone(&reads);
        let mut tasks = ProfileTasks::with_backends(
            AsyncTaskExecutor::from_current_or_test(),
            Arc::new(move |_| {
                counted.fetch_add(1, Ordering::SeqCst);
                std::thread::sleep(Duration::from_millis(10));
                Ok(snapshot("work"))
            }),
            Arc::new(applied),
        );
        let mut ack = Vec::new();
        let mut complete = Vec::new();
        for _ in 0..30 {
            let start = Instant::now();
            assert!(tasks.request_read(Platform::Codex, true));
            ack.push(start.elapsed().as_micros());
            drain_until(&mut tasks, |events| !events.is_empty());
            complete.push(start.elapsed().as_micros());
        }
        ack.sort_unstable();
        complete.sort_unstable();
        println!(
            "Async current-platform reload: samples=30 reads={} delay/read=10ms; ack p50={}us p95={}us; data-ready p50={}us p95={}us",
            reads.load(Ordering::SeqCst),
            ack[15],
            ack[28],
            complete[15],
            complete[28]
        );
    }
}
