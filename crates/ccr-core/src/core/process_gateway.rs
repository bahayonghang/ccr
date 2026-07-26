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
}

impl ManagedProcess {
    pub fn spawn(mut command: Command) -> io::Result<Self> {
        configure_process_tree(&mut command);
        command.kill_on_drop(true);
        let child = command.spawn()?;
        let pid = child
            .id()
            .ok_or_else(|| io::Error::other("child PID unavailable"))?;
        let tree = PlatformProcessTree::attach(&child, pid)?;
        Ok(Self {
            child,
            pid,
            tree,
            reaped: false,
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
        let status = self.child.wait().await;
        self.reaped = true;
        status
    }

    pub async fn terminate_tree(&mut self, grace: Duration) -> io::Result<ExitStatus> {
        self.tree.terminate_graceful(self.pid)?;
        match tokio::time::timeout(grace, self.child.wait()).await {
            Ok(status) => {
                self.reaped = true;
                status
            }
            Err(_) => {
                self.tree.terminate_forceful(self.pid)?;
                let status = self.child.wait().await;
                self.reaped = true;
                status
            }
        }
    }
}

impl Drop for ManagedProcess {
    fn drop(&mut self) {
        if !self.reaped {
            let _ = self.tree.terminate_forceful(self.pid);
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
    fn attach(_child: &Child, _pid: u32) -> io::Result<Self> {
        Ok(Self)
    }

    fn terminate_graceful(&self, pid: u32) -> io::Result<()> {
        signal_process_group(pid, 15)
    }

    fn terminate_forceful(&self, pid: u32) -> io::Result<()> {
        signal_process_group(pid, 9)
    }
}

#[cfg(unix)]
fn signal_process_group(pid: u32, signal: i32) -> io::Result<()> {
    unsafe extern "C" {
        fn kill(pid: i32, signal: i32) -> i32;
    }

    // SAFETY: a negative PID targets the process group created for this child.
    let result = unsafe { kill(-(pid as i32), signal) };
    if result == 0 {
        Ok(())
    } else {
        let error = io::Error::last_os_error();
        if error.raw_os_error() == Some(3) {
            Ok(())
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
    fn attach(child: &Child, _pid: u32) -> io::Result<Self> {
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
            information.basic_limit_information.limit_flags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
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
    use tokio::io::AsyncWriteExt;

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
