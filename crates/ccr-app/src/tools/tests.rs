#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;
#[test]
fn probes_keep_the_closed_tool_catalog() {
    for (tool, program) in VERSION_TOOLS {
        assert!(ProcessDescriptor::cli_probe(tool, program, Duration::from_secs(1)).is_ok());
    }
    assert!(ProcessDescriptor::cli_probe("arbitrary", "tool", Duration::from_secs(1)).is_err());
}

#[tokio::test]
async fn version_probe_uses_only_the_controlled_executable_and_fixed_version_argument() {
    let root = tempfile::tempdir().unwrap();
    let path = root
        .path()
        .join(if cfg!(windows) { "codex.cmd" } else { "codex" });
    let script = if cfg!(windows) {
        "@echo off\r\nif not \"%~1\"==\"--version\" exit /b 2\r\necho codex fixture-1.2.3\r\n"
    } else {
        "#!/bin/sh\n[ \"$1\" = \"--version\" ] || exit 2\nprintf 'codex fixture-1.2.3\\n'\n"
    };
    std::fs::write(&path, script).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
    }
    let entry = probe_version("codex", &path, 3500).await;
    assert!(entry.installed, "{entry:?}");
    assert_eq!(entry.status, "ok");
    assert_eq!(entry.version.as_deref(), Some("codex fixture-1.2.3"));
}

#[test]
fn update_result_requires_clean_success_without_timeout_or_truncation() {
    #[cfg(windows)]
    fn exit(code: u32) -> std::process::ExitStatus {
        use std::os::windows::process::ExitStatusExt;
        std::process::ExitStatus::from_raw(code)
    }
    #[cfg(unix)]
    fn exit(code: i32) -> std::process::ExitStatus {
        use std::os::unix::process::ExitStatusExt;
        std::process::ExitStatus::from_raw(code << 8)
    }
    for (code, timed_out, truncated, success) in [
        (0, false, false, true),
        (1, false, false, false),
        (0, true, false, false),
        (0, false, true, false),
    ] {
        let report = UpdateReport::from(CappedProcessOutput {
            status: exit(code),
            stdout: b"result".to_vec(),
            stderr: Vec::new(),
            stdout_bytes: 6,
            stderr_bytes: 0,
            stdout_truncated: truncated,
            stderr_truncated: false,
            timed_out,
            duration: Duration::ZERO,
        });
        assert_eq!(report.success, success);
        assert_eq!(report.stdout, "result");
    }
}
