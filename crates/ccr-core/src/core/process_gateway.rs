//! Cross-platform child-process tree ownership and cleanup.

use std::io;
use std::process::ExitStatus;
use std::time::Duration;

use tokio::io::{AsyncBufRead, AsyncBufReadExt};
use tokio::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command};

#[derive(Debug, PartialEq, Eq)]
pub struct BoundedLine {
    pub text: String,
    pub truncated: bool,
}

pub async fn read_bounded_line<R>(
    reader: &mut R,
    max_bytes: usize,
) -> io::Result<Option<BoundedLine>>
where
    R: AsyncBufRead + Unpin,
{
    let mut bytes = Vec::with_capacity(max_bytes.min(8 * 1024));
    let mut saw_record = false;
    let mut truncated = false;

    loop {
        let available = reader.fill_buf().await?;
        if available.is_empty() {
            if !saw_record {
                return Ok(None);
            }
            break;
        }

        let newline = available.iter().position(|byte| *byte == b'\n');
        let content_bytes = newline.unwrap_or(available.len());
        saw_record |= content_bytes > 0 || newline.is_some();

        let remaining = max_bytes.saturating_sub(bytes.len());
        let retained = content_bytes.min(remaining);
        bytes.extend_from_slice(&available[..retained]);
        truncated |= content_bytes > remaining;

        let consumed = content_bytes + usize::from(newline.is_some());
        reader.consume(consumed);
        if newline.is_some() {
            break;
        }
    }

    if bytes.last() == Some(&b'\r') {
        bytes.pop();
    }
    let mut text = String::from_utf8_lossy(&bytes).into_owned();
    if text.len() > max_bytes {
        let mut end = max_bytes;
        while end > 0 && !text.is_char_boundary(end) {
            end -= 1;
        }
        text.truncate(end);
        truncated = true;
    }

    Ok(Some(BoundedLine { text, truncated }))
}

pub struct ManagedProcess {
    child: Child,
    pid: u32,
    tree: PlatformProcessTree,
    reaped: bool,
    tree_cleaned: bool,
    detached: bool,
}

impl ManagedProcess {
    pub fn spawn(command: Command) -> io::Result<Self> {
        Self::spawn_with_mode(command, false)
    }

    /// Spawn a child whose descendants are expected to outlive it.
    ///
    /// A successful `wait` does not terminate descendants and Drop does not
    /// terminate the tree; an explicit `terminate_tree` still reclaims the
    /// whole tree. The child still gets its own process group / Job Object so
    /// that explicit reclamation stays possible.
    pub fn spawn_detached(command: Command) -> io::Result<Self> {
        Self::spawn_with_mode(command, true)
    }

    /// 共享 spawn 实现：detached 决定作业对象限制与 wait/Drop 的后代清理行为。
    fn spawn_with_mode(mut command: Command, detached: bool) -> io::Result<Self> {
        configure_process_tree(&mut command);
        command.kill_on_drop(true);
        let child = command.spawn()?;
        let pid = child
            .id()
            .ok_or_else(|| io::Error::other("child PID unavailable"))?;
        let tree = PlatformProcessTree::attach(&child, pid, detached)?;
        Ok(Self {
            child,
            pid,
            tree,
            reaped: false,
            tree_cleaned: false,
            detached,
        })
    }

    pub const fn pid(&self) -> u32 {
        self.pid
    }

    pub fn take_stdin(&mut self) -> Option<ChildStdin> {
        self.child.stdin.take()
    }

    pub fn take_stdout(&mut self) -> Option<ChildStdout> {
        self.child.stdout.take()
    }

    pub fn take_stderr(&mut self) -> Option<ChildStderr> {
        self.child.stderr.take()
    }

    pub async fn wait(&mut self) -> io::Result<ExitStatus> {
        let status = self.reap().await?;
        if self.detached {
            // 分离模式：后代是本命令的产物，成功等待后不清理。
            return Ok(status);
        }
        if !self.tree_cleaned {
            self.tree.terminate_forceful(self.pid)?;
            self.confirm_tree_exit(tokio::time::Instant::now() + Duration::from_secs(5))
                .await?;
        }
        Ok(status)
    }

    pub async fn terminate_tree(&mut self, grace: Duration) -> io::Result<ExitStatus> {
        if self.tree_cleaned {
            return self.reap().await;
        }
        let deadline = tokio::time::Instant::now() + grace;
        self.tree.terminate_graceful(self.pid)?;
        let graceful_status = tokio::time::timeout(grace / 2, self.reap()).await;
        // The direct child can exit while a descendant ignores graceful termination.
        self.tree.terminate_forceful(self.pid)?;
        let status = match graceful_status {
            Ok(status) => status?,
            Err(_) => tokio::time::timeout_at(deadline, self.reap())
                .await
                .map_err(|_| cleanup_timeout())??,
        };
        self.confirm_tree_exit(deadline).await?;
        Ok(status)
    }

    async fn reap(&mut self) -> io::Result<ExitStatus> {
        let status = self.child.wait().await?;
        self.reaped = true;
        Ok(status)
    }

    async fn confirm_tree_exit(&mut self, deadline: tokio::time::Instant) -> io::Result<()> {
        loop {
            if !self.tree.is_running(self.pid)? {
                self.tree_cleaned = true;
                return Ok(());
            }
            let now = tokio::time::Instant::now();
            if now >= deadline {
                return Err(cleanup_timeout());
            }
            tokio::time::sleep_until((now + Duration::from_millis(10)).min(deadline)).await;
        }
    }
}

fn cleanup_timeout() -> io::Error {
    io::Error::new(io::ErrorKind::TimedOut, "process_tree_cleanup_timeout")
}

impl Drop for ManagedProcess {
    fn drop(&mut self) {
        if !self.detached && !self.tree_cleaned {
            let _ = self.tree.terminate_forceful(self.pid);
        }
        if !self.reaped {
            let _ = self.child.start_kill();
        }
    }
}

#[cfg(unix)]
fn configure_process_tree(command: &mut Command) {
    use std::os::unix::process::CommandExt;

    command.as_std_mut().process_group(0);
}

#[cfg(windows)]
fn configure_process_tree(_command: &mut Command) {}

#[cfg(unix)]
struct PlatformProcessTree;

#[cfg(unix)]
impl PlatformProcessTree {
    fn attach(_child: &Child, _pid: u32, _detached: bool) -> io::Result<Self> {
        Ok(Self)
    }

    fn terminate_graceful(&self, pid: u32) -> io::Result<()> {
        signal_process_group(pid, 15).map(|_| ())
    }

    fn terminate_forceful(&self, pid: u32) -> io::Result<()> {
        signal_process_group(pid, 9).map(|_| ())
    }

    fn is_running(&self, pid: u32) -> io::Result<bool> {
        signal_process_group(pid, 0)
    }
}

#[cfg(unix)]
fn signal_process_group(pid: u32, signal: i32) -> io::Result<bool> {
    unsafe extern "C" {
        fn kill(pid: i32, signal: i32) -> i32;
    }

    // SAFETY: a negative PID targets the process group created for this child.
    let result = unsafe { kill(-(pid as i32), signal) };
    if result == 0 {
        Ok(true)
    } else {
        let error = io::Error::last_os_error();
        if error.raw_os_error() == Some(3) {
            Ok(false)
        } else {
            Err(error)
        }
    }
}

#[cfg(windows)]
struct PlatformProcessTree {
    job: *mut std::ffi::c_void,
}

#[cfg(windows)]
unsafe impl Send for PlatformProcessTree {}

#[cfg(windows)]
impl PlatformProcessTree {
    fn attach(child: &Child, _pid: u32, detached: bool) -> io::Result<Self> {
        use std::ptr;

        const JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE: u32 = 0x0000_2000;
        const JOB_OBJECT_EXTENDED_LIMIT_INFORMATION_CLASS: i32 = 9;

        #[repr(C)]
        #[derive(Default)]
        struct BasicLimitInformation {
            per_process_user_time_limit: i64,
            per_job_user_time_limit: i64,
            limit_flags: u32,
            minimum_working_set_size: usize,
            maximum_working_set_size: usize,
            active_process_limit: u32,
            affinity: usize,
            priority_class: u32,
            scheduling_class: u32,
        }

        #[repr(C)]
        #[derive(Default)]
        struct IoCounters {
            read_operation_count: u64,
            write_operation_count: u64,
            other_operation_count: u64,
            read_transfer_count: u64,
            write_transfer_count: u64,
            other_transfer_count: u64,
        }

        #[repr(C)]
        #[derive(Default)]
        struct ExtendedLimitInformation {
            basic_limit_information: BasicLimitInformation,
            io_info: IoCounters,
            process_memory_limit: usize,
            job_memory_limit: usize,
            peak_process_memory_used: usize,
            peak_job_memory_used: usize,
        }

        unsafe extern "system" {
            fn CreateJobObjectW(
                job_attributes: *const std::ffi::c_void,
                name: *const u16,
            ) -> *mut std::ffi::c_void;
            fn SetInformationJobObject(
                job: *mut std::ffi::c_void,
                information_class: i32,
                information: *const std::ffi::c_void,
                information_length: u32,
            ) -> i32;
            fn AssignProcessToJobObject(
                job: *mut std::ffi::c_void,
                process: *mut std::ffi::c_void,
            ) -> i32;
            fn CloseHandle(handle: *mut std::ffi::c_void) -> i32;
        }

        let process_handle = child
            .raw_handle()
            .ok_or_else(|| io::Error::other("child handle unavailable"))?;

        // SAFETY: kernel32 job APIs receive initialized structures and a live child handle.
        unsafe {
            let job = CreateJobObjectW(ptr::null(), ptr::null());
            if job.is_null() {
                return Err(io::Error::last_os_error());
            }
            let mut information = ExtendedLimitInformation::default();
            if !detached {
                // 默认模式：作业句柄关闭即终止整棵树。分离模式不设该限制，
                // 后代可随句柄关闭继续存活；显式 terminate_tree 仍回收整棵树。
                information.basic_limit_information.limit_flags =
                    JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            }
            if SetInformationJobObject(
                job,
                JOB_OBJECT_EXTENDED_LIMIT_INFORMATION_CLASS,
                (&raw const information).cast(),
                std::mem::size_of::<ExtendedLimitInformation>() as u32,
            ) == 0
            {
                let error = io::Error::last_os_error();
                CloseHandle(job);
                return Err(error);
            }
            if AssignProcessToJobObject(job, process_handle.cast()) == 0 {
                let error = io::Error::last_os_error();
                CloseHandle(job);
                return Err(error);
            }
            Ok(Self { job })
        }
    }

    fn terminate_graceful(&self, _pid: u32) -> io::Result<()> {
        self.terminate(1)
    }

    fn terminate_forceful(&self, _pid: u32) -> io::Result<()> {
        self.terminate(1)
    }

    fn is_running(&self, _pid: u32) -> io::Result<bool> {
        #[repr(C)]
        #[derive(Default)]
        struct BasicAccountingInformation {
            total_user_time: i64,
            total_kernel_time: i64,
            this_period_total_user_time: i64,
            this_period_total_kernel_time: i64,
            total_page_fault_count: u32,
            total_processes: u32,
            active_processes: u32,
            total_terminated_processes: u32,
        }

        unsafe extern "system" {
            fn QueryInformationJobObject(
                job: *mut std::ffi::c_void,
                information_class: i32,
                information: *mut std::ffi::c_void,
                information_length: u32,
                return_length: *mut u32,
            ) -> i32;
        }

        let mut information = BasicAccountingInformation::default();
        // SAFETY: the owned job handle and writable accounting structure are valid.
        let result = unsafe {
            QueryInformationJobObject(
                self.job,
                1, // JobObjectBasicAccountingInformation
                (&raw mut information).cast(),
                std::mem::size_of::<BasicAccountingInformation>() as u32,
                std::ptr::null_mut(),
            )
        };
        if result == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(information.active_processes > 0)
        }
    }

    fn terminate(&self, exit_code: u32) -> io::Result<()> {
        unsafe extern "system" {
            fn TerminateJobObject(job: *mut std::ffi::c_void, exit_code: u32) -> i32;
        }
        // SAFETY: this guard owns the live job handle.
        if unsafe { TerminateJobObject(self.job, exit_code) } == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}

#[cfg(windows)]
impl Drop for PlatformProcessTree {
    fn drop(&mut self) {
        unsafe extern "system" {
            fn CloseHandle(handle: *mut std::ffi::c_void) -> i32;
        }
        // SAFETY: this guard owns the handle exactly once.
        unsafe {
            CloseHandle(self.job);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Stdio;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    async fn bounded_line_reader_handles_newline_crlf_and_utf8_boundary() {
        let mut reader = tokio::io::BufReader::new("first\r\nsecond\n".as_bytes());

        assert_eq!(
            read_bounded_line(&mut reader, 32).await.expect("read CRLF"),
            Some(BoundedLine {
                text: "first".to_owned(),
                truncated: false,
            })
        );
        assert_eq!(
            read_bounded_line(&mut reader, 32).await.expect("read LF"),
            Some(BoundedLine {
                text: "second".to_owned(),
                truncated: false,
            })
        );

        let mut reader = tokio::io::BufReader::new("é\n".as_bytes());
        assert_eq!(
            read_bounded_line(&mut reader, 2)
                .await
                .expect("read truncated UTF-8"),
            Some(BoundedLine {
                text: "e".to_owned(),
                truncated: true,
            })
        );
    }

    #[tokio::test]
    async fn bounded_line_reader_caps_unterminated_input() {
        let (mut writer, reader) = tokio::io::duplex(8 * 1024);
        let writer_task = tokio::spawn(async move {
            writer
                .write_all(&vec![b'x'; 128 * 1024])
                .await
                .expect("write oversized test line");
            writer.shutdown().await.expect("close test writer");
        });
        let mut reader = tokio::io::BufReader::new(reader);

        let line = read_bounded_line(&mut reader, 64 * 1024)
            .await
            .expect("read bounded line")
            .expect("oversized input should produce one line");
        writer_task.await.expect("writer task should finish");

        assert!(line.truncated);
        assert_eq!(line.text.len(), 64 * 1024);
        assert!(
            read_bounded_line(&mut reader, 64 * 1024)
                .await
                .expect("read end of stream")
                .is_none()
        );
    }

    #[tokio::test]
    async fn managed_process_exposes_pipes_and_waits() {
        let mut command = shell_command("read a; printf '%s' \"$a\"; printf 'err' >&2");
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut process = ManagedProcess::spawn(command).expect("spawn managed process");

        assert!(process.pid() > 0);
        let mut stdin = process.take_stdin().expect("piped stdin");
        let mut stdout = process.take_stdout().expect("piped stdout");
        let mut stderr = process.take_stderr().expect("piped stderr");
        assert!(process.take_stdin().is_none());
        assert!(process.take_stdout().is_none());
        assert!(process.take_stderr().is_none());

        stdin.write_all(b"hello\n").await.expect("write stdin");
        drop(stdin);
        let mut stdout_text = String::new();
        let mut stderr_text = String::new();
        stdout
            .read_to_string(&mut stdout_text)
            .await
            .expect("read stdout");
        stderr
            .read_to_string(&mut stderr_text)
            .await
            .expect("read stderr");

        let status = process.wait().await.expect("wait for managed process");
        assert!(status.success());
        assert_eq!(stdout_text, "hello");
        assert_eq!(stderr_text, "err");
    }

    #[tokio::test]
    async fn dropping_unreaped_process_terminates_it() {
        let process = ManagedProcess::spawn(shell_command("sleep 30"))
            .expect("spawn long-running managed process");
        let pid = process.pid();
        assert!(test_process_is_running(pid));

        drop(process);
        for _ in 0..40 {
            if !test_process_is_running(pid) {
                return;
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
        panic!("dropped managed process {pid} is still running");
    }

    #[cfg(windows)]
    fn shell_command(script: &str) -> Command {
        let mut command = Command::new("powershell.exe");
        let script = if script.starts_with("sleep") {
            "Start-Sleep -Seconds 30"
        } else {
            "$line=[Console]::In.ReadLine(); [Console]::Out.Write($line); [Console]::Error.Write('err')"
        };
        command.args(["-NoProfile", "-NonInteractive", "-Command", script]);
        command
    }

    #[tokio::test]
    async fn live_tree_confirmation_times_out_without_claiming_cleanup() {
        let mut process =
            ManagedProcess::spawn(shell_command("sleep 30")).expect("spawn live process");
        let started = tokio::time::Instant::now();
        let error = process
            .confirm_tree_exit(started + Duration::from_millis(30))
            .await
            .expect_err("a live tree cannot confirm successful cleanup");
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
        assert!(!process.tree_cleaned && !process.reaped);
        assert!(started.elapsed() < Duration::from_millis(500));
        process
            .terminate_tree(Duration::from_secs(2))
            .await
            .expect("clean up the fixture after confirmation timeout");
    }

    #[cfg(unix)]
    fn shell_command(script: &str) -> Command {
        let mut command = Command::new("sh");
        command.args(["-c", script]);
        command
    }

    #[cfg(windows)]
    fn test_process_is_running(pid: u32) -> bool {
        process_is_running(pid)
    }

    #[cfg(unix)]
    fn test_process_is_running(pid: u32) -> bool {
        unix_process_is_running(pid)
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn managed_process_terminates_windows_descendant_tree() {
        let temp = tempfile::tempdir().expect("tempdir");
        let pid_file = temp.path().join("grandchild.pid");
        let script = format!(
            "$ErrorActionPreference='Stop'; Start-Sleep -Milliseconds 300; \
             $child=Start-Process -FilePath 'cmd.exe' -ArgumentList '/C','ping -n 30 127.0.0.1 >NUL' -PassThru; \
             [IO.File]::WriteAllText('{}', [string]$child.Id); Start-Sleep -Seconds 30",
            pid_file.to_string_lossy().replace('\'', "''")
        );
        let mut command = Command::new("powershell.exe");
        command.args(["-NoProfile", "-NonInteractive", "-Command", &script]);
        let mut process = ManagedProcess::spawn(command).expect("managed parent");

        let grandchild_pid = wait_for_pid_file(&pid_file).await;
        assert!(process_is_running(grandchild_pid));

        process
            .terminate_tree(Duration::from_secs(1))
            .await
            .expect("terminate managed tree");
        tokio::time::sleep(Duration::from_millis(100)).await;

        assert!(!process_is_running(grandchild_pid));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn managed_process_terminates_unix_descendant_group() {
        let temp = tempfile::tempdir().expect("tempdir");
        let pid_file = temp.path().join("grandchild.pid");
        let script = format!(
            "sleep 30 & echo $! > '{}' ; wait",
            pid_file.to_string_lossy().replace('\'', "'\\''")
        );
        let mut command = Command::new("sh");
        command.args(["-c", &script]);
        let mut process = ManagedProcess::spawn(command).expect("managed parent");

        let grandchild_pid = wait_for_unix_pid_file(&pid_file).await;
        assert!(unix_process_is_running(grandchild_pid));

        process
            .terminate_tree(Duration::from_secs(1))
            .await
            .expect("terminate managed group");
        tokio::time::sleep(Duration::from_millis(100)).await;

        assert!(!unix_process_is_running(grandchild_pid));
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn reaped_windows_parent_still_cleans_its_descendant_tree() {
        for drop_after_reap in [false, true] {
            let temp = tempfile::tempdir().expect("tempdir");
            let pid_file = temp.path().join("grandchild.pid");
            let script = format!(
                "$ErrorActionPreference='Stop'; Start-Sleep -Milliseconds 300; \
                 $child=Start-Process -FilePath 'cmd.exe' -ArgumentList '/C','ping -n 30 127.0.0.1 >NUL' -PassThru; \
                 [IO.File]::WriteAllText('{}', [string]$child.Id)",
                pid_file.to_string_lossy().replace('\'', "''")
            );
            let mut command = Command::new("powershell.exe");
            command.args(["-NoProfile", "-NonInteractive", "-Command", &script]);
            let mut process = ManagedProcess::spawn(command).expect("managed parent");
            let descendant = wait_for_pid_file(&pid_file).await;
            process.reap().await.expect("reap direct child");
            assert!(process_is_running(descendant));
            assert!(process.reaped && !process.tree_cleaned);
            if drop_after_reap {
                drop(process);
            } else {
                assert!(
                    process
                        .wait()
                        .await
                        .expect("wait and clean owned tree")
                        .success()
                );
                assert!(process.tree_cleaned);
            }
            for _ in 0..100 {
                if !process_is_running(descendant) {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            assert!(!process_is_running(descendant));
        }
    }

    #[cfg(unix)]
    async fn unix_ignoring_descendant(
        directory: &std::path::Path,
        parent_exits: bool,
    ) -> (ManagedProcess, u32) {
        let pid_file = directory.join("grandchild.pid");
        let ready_file = directory.join("grandchild.ready");
        let quote = |path: &std::path::Path| path.to_string_lossy().replace('\'', "'\\''");
        let child_script = format!(
            "trap '' TERM; printf ready > '{}'; exec sleep 30",
            quote(&ready_file)
        );
        let after_start = if parent_exits {
            "exit 0"
        } else {
            "wait \"$descendant\""
        };
        let script = format!(
            "trap 'exit 0' TERM; sh -c '{}' & descendant=$!; \
             while [ ! -f '{}' ]; do sleep 0.01; done; \
             printf '%s' \"$descendant\" > '{}'; {after_start}",
            child_script.replace('\'', "'\\''"),
            quote(&ready_file),
            quote(&pid_file),
        );
        let mut command = Command::new("sh");
        command.args(["-c", &script]);
        let process = ManagedProcess::spawn(command).expect("managed parent");
        let descendant = wait_for_unix_pid_file(&pid_file).await;
        assert!(unix_process_is_running(descendant));
        (process, descendant)
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn termination_escalates_after_unix_parent_exits_on_term() {
        let temp = tempfile::tempdir().expect("tempdir");
        let (mut process, descendant) = unix_ignoring_descendant(temp.path(), false).await;
        let started = tokio::time::Instant::now();
        let status = process
            .terminate_tree(Duration::from_secs(2))
            .await
            .expect("terminate descendants after graceful parent exit");
        assert!(
            status.success(),
            "the parent must exit through its TERM trap"
        );
        assert!(started.elapsed() < Duration::from_millis(2500));
        assert!(!unix_process_is_running(descendant));
        assert!(process.reaped && process.tree_cleaned);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn wait_terminates_unix_descendants_after_parent_exit() {
        let temp = tempfile::tempdir().expect("tempdir");
        let (mut process, descendant) = unix_ignoring_descendant(temp.path(), true).await;
        let status = tokio::time::timeout(Duration::from_secs(6), process.wait())
            .await
            .expect("wait must bound descendant cleanup")
            .expect("clean descendants after normal parent exit");
        assert!(status.success());
        assert!(!unix_process_is_running(descendant));
        assert!(process.reaped && process.tree_cleaned);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn dropping_reaped_unix_parent_still_terminates_descendants() {
        let temp = tempfile::tempdir().expect("tempdir");
        let (mut process, descendant) = unix_ignoring_descendant(temp.path(), true).await;
        process.reap().await.expect("reap direct child");
        assert!(process.reaped && !process.tree_cleaned);
        drop(process);
        for _ in 0..100 {
            if !unix_process_is_running(descendant) {
                return;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        panic!("descendant {descendant} survived the reaped-parent Drop fallback");
    }

    #[cfg(unix)]
    async fn wait_for_unix_pid_file(path: &std::path::Path) -> u32 {
        for _ in 0..100 {
            if let Ok(raw) = std::fs::read_to_string(path)
                && let Ok(pid) = raw.trim().parse()
            {
                return pid;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        panic!("grandchild PID file was not written");
    }

    #[cfg(unix)]
    fn unix_process_is_running(pid: u32) -> bool {
        unsafe extern "C" {
            fn kill(pid: i32, signal: i32) -> i32;
        }

        // SAFETY: signal 0 checks existence/permission without sending a signal.
        unsafe { kill(pid as i32, 0) == 0 }
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn detached_wait_leaves_windows_descendants_running() {
        let temp = tempfile::tempdir().expect("tempdir");
        let pid_file = temp.path().join("grandchild.pid");
        let script = format!(
            "$ErrorActionPreference='Stop'; Start-Sleep -Milliseconds 300; \
             $child=Start-Process -FilePath 'cmd.exe' -ArgumentList '/C','ping -n 30 127.0.0.1 >NUL' -PassThru; \
             [IO.File]::WriteAllText('{}', [string]$child.Id)",
            pid_file.to_string_lossy().replace('\'', "''")
        );
        let mut command = Command::new("powershell.exe");
        command.args(["-NoProfile", "-NonInteractive", "-Command", &script]);
        let mut process = ManagedProcess::spawn_detached(command).expect("detached parent");

        let grandchild_pid = wait_for_pid_file(&pid_file).await;
        assert!(process_is_running(grandchild_pid));
        assert!(
            process
                .wait()
                .await
                .expect("wait for detached parent")
                .success()
        );
        drop(process);
        tokio::time::sleep(Duration::from_millis(300)).await;

        // 分离模式合同：成功等待并 Drop 后，孙进程仍须存活（它是命令的产物）。
        assert!(
            process_is_running(grandchild_pid),
            "detached grandchild must survive wait and Drop"
        );

        terminate_test_process(grandchild_pid);
        assert_eventually_gone(grandchild_pid).await;
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn detached_terminate_tree_reclaims_windows_descendants() {
        let temp = tempfile::tempdir().expect("tempdir");
        let pid_file = temp.path().join("grandchild.pid");
        let script = format!(
            "$ErrorActionPreference='Stop'; Start-Sleep -Milliseconds 300; \
             $child=Start-Process -FilePath 'cmd.exe' -ArgumentList '/C','ping -n 30 127.0.0.1 >NUL' -PassThru; \
             [IO.File]::WriteAllText('{}', [string]$child.Id); Start-Sleep -Seconds 30",
            pid_file.to_string_lossy().replace('\'', "''")
        );
        let mut command = Command::new("powershell.exe");
        command.args(["-NoProfile", "-NonInteractive", "-Command", &script]);
        let mut process = ManagedProcess::spawn_detached(command).expect("detached parent");

        let grandchild_pid = wait_for_pid_file(&pid_file).await;
        assert!(process_is_running(grandchild_pid));

        process
            .terminate_tree(Duration::from_secs(1))
            .await
            .expect("reclaim detached tree");
        tokio::time::sleep(Duration::from_millis(100)).await;

        // 分离模式仍支持显式回收：作业对象未设 kill-on-close，但 terminate_tree 有效。
        assert!(!process_is_running(grandchild_pid));
        assert!(process.reaped && process.tree_cleaned);
    }

    #[cfg(unix)]
    async fn unix_detached_parent(directory: &std::path::Path) -> (ManagedProcess, u32) {
        let pid_file = directory.join("grandchild.pid");
        let ready_file = directory.join("grandchild.ready");
        let quote = |path: &std::path::Path| path.to_string_lossy().replace('\'', "'\\''");
        let child_script = format!(
            "trap '' TERM; printf ready > '{}'; exec sleep 30",
            quote(&ready_file)
        );
        let script = format!(
            "sh -c '{}' & descendant=$!; \
             while [ ! -f '{}' ]; do sleep 0.01; done; \
             printf '%s' \"$descendant\" > '{}'; exit 0",
            child_script.replace('\'', "'\\''"),
            quote(&ready_file),
            quote(&pid_file),
        );
        let mut command = Command::new("sh");
        command.args(["-c", &script]);
        let process = ManagedProcess::spawn_detached(command).expect("detached parent");
        let descendant = wait_for_unix_pid_file(&pid_file).await;
        assert!(unix_process_is_running(descendant));
        (process, descendant)
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn detached_wait_leaves_unix_descendants_running() {
        let temp = tempfile::tempdir().expect("tempdir");
        let (mut process, descendant) = unix_detached_parent(temp.path()).await;

        assert!(
            process
                .wait()
                .await
                .expect("wait for detached parent")
                .success()
        );
        drop(process);
        tokio::time::sleep(Duration::from_millis(200)).await;

        // 分离模式合同：成功等待并 Drop 后，后代仍须存活（它是命令的产物）。
        assert!(
            unix_process_is_running(descendant),
            "detached descendant must survive wait and Drop"
        );

        terminate_test_process(descendant);
        assert_eventually_gone(descendant).await;
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn detached_terminate_tree_reclaims_unix_descendants() {
        let temp = tempfile::tempdir().expect("tempdir");
        let (mut process, descendant) = unix_detached_parent(temp.path()).await;

        process
            .terminate_tree(Duration::from_secs(1))
            .await
            .expect("reclaim detached group");
        tokio::time::sleep(Duration::from_millis(100)).await;

        // 分离模式仍支持显式回收：终止整棵进程组（含后代）。
        assert!(!unix_process_is_running(descendant));
        assert!(process.reaped && process.tree_cleaned);
    }

    async fn assert_eventually_gone(pid: u32) {
        for _ in 0..100 {
            if !test_process_is_running(pid) {
                return;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        panic!("fixture process {pid} survived test cleanup");
    }

    #[cfg(windows)]
    fn terminate_test_process(pid: u32) {
        // 测试清理：连同后代一起结束 fixture 进程树。
        let pid_arg = pid.to_string();
        let _ = std::process::Command::new("taskkill")
            .args(["/PID", pid_arg.as_str(), "/T", "/F"])
            .output();
    }

    #[cfg(unix)]
    fn terminate_test_process(pid: u32) {
        unsafe extern "C" {
            fn kill(pid: i32, signal: i32) -> i32;
        }

        // SAFETY: SIGKILL targets only the fixture PID recorded in the pid file.
        unsafe {
            kill(pid as i32, 9);
        }
    }

    #[cfg(windows)]
    async fn wait_for_pid_file(path: &std::path::Path) -> u32 {
        for _ in 0..100 {
            if let Ok(raw) = std::fs::read_to_string(path)
                && let Ok(pid) = raw.trim().parse()
            {
                return pid;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        panic!("grandchild PID file was not written");
    }

    #[cfg(windows)]
    fn process_is_running(pid: u32) -> bool {
        const SYNCHRONIZE: u32 = 0x0010_0000;
        const WAIT_TIMEOUT: u32 = 258;
        unsafe extern "system" {
            fn OpenProcess(
                desired_access: u32,
                inherit_handle: i32,
                process_id: u32,
            ) -> *mut std::ffi::c_void;
            fn WaitForSingleObject(handle: *mut std::ffi::c_void, milliseconds: u32) -> u32;
            fn CloseHandle(handle: *mut std::ffi::c_void) -> i32;
        }

        // SAFETY: the opened synchronization handle is closed before returning.
        unsafe {
            let handle = OpenProcess(SYNCHRONIZE, 0, pid);
            if handle.is_null() {
                return false;
            }
            let running = WaitForSingleObject(handle, 0) == WAIT_TIMEOUT;
            CloseHandle(handle);
            running
        }
    }
}
