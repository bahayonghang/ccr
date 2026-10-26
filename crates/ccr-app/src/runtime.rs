//! Bounded, application-owned execution. UI frameworks never enter Tokio or do I/O.
use std::{
    io,
    sync::{Arc, Mutex},
    time::Duration,
};

use async_channel::{Receiver, Sender};
pub use ccr_config::Platform;
use tokio::{
    runtime::{Builder, Runtime},
    sync::{Notify, Semaphore},
    time::Instant,
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};

pub const NOTIFICATION_CAPACITY: usize = 128;
pub const MAX_OPERATIONS: usize = 128;
pub const BLOCKING_CONCURRENCY: usize = 4;
pub const PROGRESS_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EnvironmentId {
    Local(String),
    Wsl(String),
    Ssh(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequestKey {
    pub environment: EnvironmentId,
    pub platform: Platform,
    pub generation: u64,
    pub request_id: u64,
}

/// Equality includes request identity as well as environment and filter generation.
#[derive(Default)]
pub struct ResponseGate(Option<RequestKey>);
impl ResponseGate {
    pub fn replace(&mut self, key: RequestKey) {
        self.0 = Some(key);
    }
    pub fn clear(&mut self) {
        self.0 = None;
    }
    pub fn accepts(&self, key: &RequestKey) -> bool {
        self.0.as_ref() == Some(key)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Progress {
    pub completed: u64,
    pub total: Option<u64>,
}

/// Only the runner returns a terminal outcome, after its cleanup has finished.
#[derive(Debug, PartialEq, Eq)]
pub enum Completion<T, E> {
    Success(T),
    Failed(E),
    Cancelled,
    CleanupFailed(E),
    WorkerFailed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdmissionError {
    Full,
    Stopped,
}

#[derive(Debug)]
pub enum TaskEvent<T, E> {
    Progress(Progress),
    Terminal(Arc<Completion<T, E>>),
}

struct PendingProgress {
    value: Mutex<Option<Progress>>,
    changed: Notify,
}

/// Latest-value progress is separate from authoritative terminal state and logs.
#[derive(Clone)]
pub struct ProgressReporter(Arc<PendingProgress>);
impl ProgressReporter {
    pub fn report(&self, progress: Progress) {
        *lock(&self.0.value) = Some(progress);
        self.0.changed.notify_one();
    }
}

struct State<T, E> {
    terminal: Mutex<Option<Arc<Completion<T, E>>>>,
    cancel: CancellationToken,
}

enum Wake {
    Progress(Progress),
    Terminal,
}

/// Dropping a read detaches its subscription and requests cancellation. A job
/// subscription can be dropped without cancelling the application-owned runner.
pub struct TaskSubscription<T, E> {
    key: RequestKey,
    state: Arc<State<T, E>>,
    notifications: Receiver<Wake>,
    cancel_on_drop: bool,
    terminal_delivered: bool,
}
impl<T, E> TaskSubscription<T, E> {
    pub fn key(&self) -> &RequestKey {
        &self.key
    }
    pub fn request_cancel(&self) {
        self.state.cancel.cancel();
    }
    pub fn is_cancel_requested(&self) -> bool {
        self.state.cancel.is_cancelled()
    }
    pub fn terminal(&self) -> Option<Arc<Completion<T, E>>> {
        lock(&self.state.terminal).clone()
    }
    pub async fn next(&mut self) -> Option<TaskEvent<T, E>> {
        if self.terminal_delivered {
            return None;
        }
        loop {
            // Completion is never enqueued behind lossy progress. Checking before
            // recv plus a wake (or an already full queue) closes the notification race.
            if let Some(terminal) = self.terminal() {
                self.terminal_delivered = true;
                return Some(TaskEvent::Terminal(terminal));
            }
            match self.notifications.recv().await.ok()? {
                Wake::Progress(value) => return Some(TaskEvent::Progress(value)),
                Wake::Terminal => continue,
            }
        }
    }
}
impl<T, E> Drop for TaskSubscription<T, E> {
    fn drop(&mut self) {
        self.notifications.close();
        if self.cancel_on_drop {
            self.state.cancel.cancel();
        }
    }
}

/// One long-lived runtime, owned outside the native event loop. shutdown joins
/// started blocking work; it must be called after the event loop returns.
pub struct RuntimeOwner {
    runtime: Runtime,
    handle: RuntimeHandle,
}

#[derive(Clone)]
pub struct RuntimeHandle {
    executor: tokio::runtime::Handle,
    tasks: TaskTracker,
    shutdown: CancellationToken,
    admission: Arc<Semaphore>,
    blocking: Arc<Semaphore>,
    stopped: Arc<Mutex<bool>>,
}

impl RuntimeOwner {
    pub fn new() -> io::Result<Self> {
        let runtime = Builder::new_multi_thread()
            .worker_threads(2)
            .max_blocking_threads(BLOCKING_CONCURRENCY)
            .enable_all()
            .thread_name("ccr-app")
            .build()?;
        let handle = RuntimeHandle {
            executor: runtime.handle().clone(),
            tasks: TaskTracker::new(),
            shutdown: CancellationToken::new(),
            admission: Arc::new(Semaphore::new(MAX_OPERATIONS)),
            blocking: Arc::new(Semaphore::new(BLOCKING_CONCURRENCY)),
            stopped: Arc::new(Mutex::new(false)),
        };
        Ok(Self { runtime, handle })
    }
    pub fn handle(&self) -> RuntimeHandle {
        self.handle.clone()
    }
    pub fn shutdown(self) {
        self.runtime.block_on(self.handle.shutdown());
    }
}

impl RuntimeHandle {
    pub fn active_tasks(&self) -> usize {
        self.tasks.len()
    }
    /// Requests cancellation and waits for runners. Calling this future never
    /// writes a terminal state on their behalf.
    pub async fn shutdown(&self) {
        {
            let mut stopped = lock(&self.stopped);
            *stopped = true;
            self.shutdown.cancel();
            self.admission.close();
            self.tasks.close();
        }
        self.tasks.wait().await;
    }

    pub fn read<T, E, F>(
        &self,
        key: RequestKey,
        work: F,
    ) -> Result<TaskSubscription<T, E>, AdmissionError>
    where
        T: Send + Sync + 'static,
        E: Send + Sync + 'static,
        F: FnOnce(CancellationToken, ProgressReporter) -> Result<T, E> + Send + 'static,
    {
        self.start(key, true, move |cancel, progress| {
            let result = work(cancel.clone(), progress);
            match result {
                _ if cancel.is_cancelled() => Completion::Cancelled,
                Ok(value) => Completion::Success(value),
                Err(error) => Completion::Failed(error),
            }
        })
    }

    /// A side-effect runner must retain its own process/cleanup semantics and
    /// choose its terminal only after cleanup. Page drop is merely unsubscription.
    pub fn job<T, E, F>(
        &self,
        key: RequestKey,
        work: F,
    ) -> Result<TaskSubscription<T, E>, AdmissionError>
    where
        T: Send + Sync + 'static,
        E: Send + Sync + 'static,
        F: FnOnce(CancellationToken, ProgressReporter) -> Completion<T, E> + Send + 'static,
    {
        self.start(key, false, work)
    }

    fn start<T, E, F>(
        &self,
        key: RequestKey,
        cancel_on_drop: bool,
        work: F,
    ) -> Result<TaskSubscription<T, E>, AdmissionError>
    where
        T: Send + Sync + 'static,
        E: Send + Sync + 'static,
        F: FnOnce(CancellationToken, ProgressReporter) -> Completion<T, E> + Send + 'static,
    {
        let stopped = lock(&self.stopped);
        if *stopped {
            return Err(AdmissionError::Stopped);
        }
        let admission = self.admission.clone().try_acquire_owned().map_err(|_| {
            if self.shutdown.is_cancelled() {
                AdmissionError::Stopped
            } else {
                AdmissionError::Full
            }
        })?;
        let state = Arc::new(State {
            terminal: Mutex::new(None),
            cancel: self.shutdown.child_token(),
        });
        let reporter = ProgressReporter(Arc::new(PendingProgress {
            value: Mutex::new(None),
            changed: Notify::new(),
        }));
        let (tx, notifications) = async_channel::bounded(NOTIFICATION_CAPACITY);
        let runner_state = state.clone();
        let blocking = self.blocking.clone();
        let executor = self.executor.clone();
        self.tasks.spawn_on(
            async move {
                let _admission = admission;
                let permit = tokio::select! {
                    biased;
                    _ = runner_state.cancel.cancelled() => None,
                    permit = blocking.acquire_owned() => permit.ok(),
                };
                let completion = if let Some(permit) = permit {
                    let cancel = runner_state.cancel.clone();
                    let worker_reporter = reporter.clone();
                    let mut worker = executor.spawn_blocking(move || {
                        let _permit = permit;
                        if cancel.is_cancelled() {
                            Completion::Cancelled
                        } else {
                            work(cancel, worker_reporter)
                        }
                    });
                    let mut progress = Box::pin(forward_progress(reporter, tx.clone()));
                    tokio::select! {
                        result = &mut worker => result.unwrap_or(Completion::WorkerFailed),
                        () = &mut progress => worker.await.unwrap_or(Completion::WorkerFailed),
                    }
                } else {
                    Completion::Cancelled
                };
                publish_terminal(&runner_state, &tx, completion);
            },
            &self.executor,
        );
        Ok(TaskSubscription {
            key,
            state,
            notifications,
            cancel_on_drop,
            terminal_delivered: false,
        })
    }
}

fn publish_terminal<T, E>(state: &State<T, E>, tx: &Sender<Wake>, completion: Completion<T, E>) {
    let mut terminal = lock(&state.terminal);
    if terminal.is_none() {
        *terminal = Some(Arc::new(completion));
    }
    // Full means the consumer already has a wake. Closed means it detached.
    let _ = tx.try_send(Wake::Terminal);
}

async fn forward_progress(reporter: ProgressReporter, tx: Sender<Wake>) {
    let mut next = Instant::now();
    loop {
        reporter.0.changed.notified().await;
        tokio::time::sleep_until(next).await;
        let value = lock(&reporter.0.value).take();
        if let Some(value) = value {
            if tx.try_send(Wake::Progress(value)).is_err() && tx.is_closed() {
                return;
            }
            next = Instant::now() + PROGRESS_INTERVAL;
        }
    }
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    // Poisoning cannot authorize another terminal transition; recover the retained value.
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[cfg(test)]
mod tests;
