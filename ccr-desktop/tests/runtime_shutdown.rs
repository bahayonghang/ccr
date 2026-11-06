//! Windows OS-process and runtime evidence, not a native-window close receipt.
#![cfg(windows)]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use ccr_app::{
    jobs::{
        commands::{CommandJobs, Status},
        policy::CommandExecutionRequest,
        process::tokio_command,
    },
    runtime::{AdmissionError, Completion, EnvironmentId, Platform, RequestKey, RuntimeOwner},
};
use std::{
    ffi::c_void,
    os::windows::process::CommandExt,
    path::{Path, PathBuf},
    sync::{Arc, mpsc},
    time::{Duration, Instant},
};

const FIXTURE_ROOT: &str = "CCR_GPUI_SHUTDOWN_FIXTURE_ROOT";

fn key(request_id: u64) -> RequestKey {
    RequestKey {
        environment: EnvironmentId::Local("shutdown-fixture".into()),
        platform: Platform::Claude,
        generation: 0,
        request_id,
    }
}

// Always request cancellation and join, including when a fixture assertion fails.
struct ShutdownOnDrop(Option<RuntimeOwner>);
impl Drop for ShutdownOnDrop {
    fn drop(&mut self) {
        if let Some(owner) = self.0.take() {
            owner.shutdown();
        }
    }
}

struct ProcessHandle(*mut c_void);
impl ProcessHandle {
    fn open(pid: u32) -> Self {
        // SAFETY: synchronization and failure-cleanup access to the fixture PID.
        let handle = unsafe { OpenProcess(0x0010_0001, 0, pid) };
        assert!(!handle.is_null(), "open live fixture process {pid}");
        Self(handle)
    }
    fn assert_running(&self) {
        // SAFETY: this object retains a valid synchronization handle until Drop.
        assert_eq!(unsafe { WaitForSingleObject(self.0, 0) }, 258);
    }
    fn assert_exited(&self, name: &str) {
        // A signaled retained handle proves exit; a failed PID lookup does not.
        // Windows may finish signaling after job accounting reaches zero. This
        // bound is shorter than the synthetic child's natural ten-second run.
        let started = Instant::now();
        // SAFETY: this object retains a valid synchronization handle until Drop.
        let initial = unsafe { WaitForSingleObject(self.0, 0) };
        // SAFETY: same retained handle; no PID reuse can change the target.
        let final_wait = unsafe { WaitForSingleObject(self.0, 1_000) };
        eprintln!(
            "{name}: initial_wait={initial}, final_wait={final_wait}, wait_us={}",
            started.elapsed().as_micros()
        );
        assert_eq!(final_wait, 0, "{name}");
    }
}
impl Drop for ProcessHandle {
    fn drop(&mut self) {
        // SAFETY: only this fixture's retained process handle can be targeted.
        // A failed assertion must not leave its synthetic process running.
        unsafe {
            if WaitForSingleObject(self.0, 0) == 258 {
                TerminateProcess(self.0, 1);
                WaitForSingleObject(self.0, 5_000);
            }
            CloseHandle(self.0);
        }
    }
}
#[link(name = "kernel32")]
unsafe extern "system" {
    fn OpenProcess(access: u32, inherit: i32, pid: u32) -> *mut c_void;
    fn WaitForSingleObject(handle: *mut c_void, milliseconds: u32) -> u32;
    fn TerminateProcess(handle: *mut c_void, exit_code: u32) -> i32;
    fn CloseHandle(handle: *mut c_void) -> i32;
}

#[test]
fn shutdown_reclaims_a_view_detached_command_and_its_descendant() {
    let output_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../outputs/desktop");
    std::fs::create_dir_all(&output_root).unwrap();
    let fixture = tempfile::tempdir_in(output_root).unwrap();
    std::fs::write(
        fixture.path().join(".ccr-gpui-profile-fixture"),
        "ccr-gpui-profiles-fixture-v1",
    )
    .unwrap();
    let mut owner = ShutdownOnDrop(Some(RuntimeOwner::new().unwrap()));
    let executor = owner.0.as_ref().unwrap().handle();
    let jobs = Arc::new(CommandJobs::default());
    let admitted = jobs
        .admit(
            key(1),
            CommandExecutionRequest::background("version".into(), None, None),
        )
        .unwrap();
    let id = admitted.snapshot.job_id;
    let ticket = admitted.ticket.unwrap();
    let worker_jobs = jobs.clone();
    let mut command = tokio_command(std::env::current_exe().unwrap());
    command
        .args(["--ignored", "--exact", "owned_tree_fixture", "--nocapture"])
        .env(FIXTURE_ROOT, fixture.path());
    // Start before either process exists: natural completion takes ten seconds.
    // Bound the whole causal chain, not only the final handle wait.
    let started = Instant::now();
    let subscription = executor
        .job(key(1), move |cancel, _| {
            match worker_jobs.run_fixture_blocking(ticket, cancel, command) {
                Ok(snapshot) => Completion::Success(snapshot),
                Err(error) => Completion::Failed(error),
            }
        })
        .unwrap();

    // The descendant announces readiness from inside its existing closed mode.
    let deadline = Instant::now() + Duration::from_secs(10);
    let (parent, descendant) = loop {
        let snapshot = jobs.status(&id).unwrap();
        if snapshot
            .stdout_lines
            .iter()
            .any(|line| line == "Synthetic process progress 0/20")
            && let Ok(raw) = std::fs::read_to_string(fixture.path().join("processes.json"))
            && let Ok([parent, descendant]) = serde_json::from_str::<[u32; 2]>(&raw)
        {
            break (ProcessHandle::open(parent), ProcessHandle::open(descendant));
        }
        assert!(
            !snapshot.status.is_terminal(),
            "fixture failed: {snapshot:?}"
        );
        assert!(Instant::now() < deadline, "fixture did not become ready");
        std::thread::sleep(Duration::from_millis(10));
    };
    parent.assert_running();
    descendant.assert_running();
    assert!(subscription.terminal().is_none());
    drop(subscription); // Leaving the view must not cancel the app-owned job.
    assert_eq!(jobs.status(&id).unwrap().status, Status::Running);
    assert!(executor.active_tasks() > 0);

    owner.0.take().unwrap().shutdown();

    assert_eq!(jobs.status(&id).unwrap().status, Status::Cancelled);
    parent.assert_exited("parent");
    descendant.assert_exited("descendant");
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "shutdown must reclaim the fixture before its natural ten-second exit"
    );
    assert_eq!(executor.active_tasks(), 0);
    assert!(matches!(
        executor.read(key(2), |_, _| Ok::<_, ()>(())),
        Err(AdmissionError::Stopped)
    ));
}

#[test]
fn final_owner_join_survives_an_expired_quit_observer() {
    let mut owner = ShutdownOnDrop(Some(RuntimeOwner::new().unwrap()));
    let executor = owner.0.as_ref().unwrap().handle();
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let read = executor
        .read(key(3), move |_, _| {
            entered_tx.send(()).unwrap();
            release_rx.recv().unwrap();
            Ok::<_, ()>(())
        })
        .unwrap();
    entered_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    let observer_runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .unwrap();
    // This models GPUI's bounded observer; it does not call an OS window close.
    let observer = observer_runtime.block_on(async {
        tokio::time::timeout(Duration::from_millis(200), executor.shutdown()).await
    });
    assert!(observer.is_err());
    assert!(read.is_cancel_requested());
    assert!(read.terminal().is_none());
    assert!(executor.active_tasks() > 0);

    let final_owner = owner.0.take().unwrap();
    let (joining_tx, joining_rx) = mpsc::channel();
    let (joined_tx, joined_rx) = mpsc::channel();
    let final_join = std::thread::spawn(move || {
        joining_tx.send(()).unwrap();
        final_owner.shutdown();
        joined_tx.send(()).unwrap();
    });
    joining_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    assert!(matches!(
        joined_rx.recv_timeout(Duration::from_millis(50)),
        Err(mpsc::RecvTimeoutError::Timeout)
    ));
    release_tx.send(()).unwrap();
    joined_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    final_join.join().unwrap();
    assert_eq!(executor.active_tasks(), 0);
    assert!(matches!(
        read.terminal().as_deref(),
        Some(Completion::Cancelled)
    ));
}

// Only the managed test child runs this helper. It invokes the existing native
// synthetic child, never a shell, real CCR command, network, or user data.
#[test]
#[ignore = "closed helper process for shutdown_reclaims_a_view_detached_command_and_its_descendant"]
fn owned_tree_fixture() {
    let root = PathBuf::from(std::env::var_os(FIXTURE_ROOT).expect("fixture root"))
        .canonicalize()
        .unwrap();
    let allowed = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../outputs/desktop")
        .canonicalize()
        .unwrap();
    assert!(root.starts_with(allowed));
    assert_eq!(
        std::fs::read_to_string(root.join(".ccr-gpui-profile-fixture")).unwrap(),
        "ccr-gpui-profiles-fixture-v1"
    );
    let mut descendant = std::process::Command::new(env!("CARGO_BIN_EXE_ccr-gpui"))
        .arg("--native-command-fixture")
        .arg(&root)
        .creation_flags(0x0800_0000) // CREATE_NO_WINDOW
        .spawn()
        .unwrap();
    std::fs::write(
        root.join("processes.json"),
        serde_json::to_vec(&[std::process::id(), descendant.id()]).unwrap(),
    )
    .unwrap();
    assert!(descendant.wait().unwrap().success());
}
