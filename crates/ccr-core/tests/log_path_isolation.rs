use std::fs::{self, File, FileTimes};
use std::path::Path;
use std::process::Command;
use std::time::{Duration, SystemTime};

#[test]
#[ignore = "invoked by isolated child-process log path tests"]
fn log_path_probe() {
    assert_eq!(std::env::var("CCR_LOG_PATH_PROBE").as_deref(), Ok("1"));
    assert!(std::env::var_os("CCR_ROOT").is_some_and(|root| !root.is_empty()));
    match std::env::var("CCR_LOG_PATH_MODE").as_deref() {
        Ok("console") => ccr_core::init_logger(),
        Ok("file-only") => ccr_core::init_file_only_logger(),
        mode => panic!("unknown log path probe mode: {mode:?}"),
    }
}

fn write_log(path: &Path, content: &[u8], age_days: u64) {
    fs::write(path, content).expect("synthetic log is writable");
    let file = File::options()
        .write(true)
        .open(path)
        .expect("synthetic log opens");
    let modified = SystemTime::now() - Duration::from_secs(age_days * 24 * 60 * 60);
    file.set_times(FileTimes::new().set_modified(modified))
        .expect("synthetic timestamp is writable");
}

fn assert_log_path_isolation(mode: &str) {
    let temp = tempfile::tempdir().expect("synthetic log home");
    let home = temp.path().join("home");
    let root = temp.path().join("selected-ccr");
    let selected_logs = root.join("logs");
    let other_logs = temp.path().join("other-ccr/logs");
    let home_logs = home.join(".ccr/logs");
    for dir in [&selected_logs, &other_logs, &home_logs] {
        fs::create_dir_all(dir).expect("synthetic log directory");
    }

    let expired = selected_logs.join("ccr.log.2000-01-01");
    let retained = selected_logs.join("ccr.log.2000-01-02");
    let unmanaged = selected_logs.join("other.log.2000-01-01");
    write_log(&expired, b"expired synthetic log", 15);
    write_log(&retained, b"retained synthetic log", 13);
    write_log(&unmanaged, b"unmanaged synthetic log", 15);
    let sentinel_paths = [
        other_logs.join("ccr.log.2000-01-01"),
        home_logs.join("ccr.log.2000-01-01"),
    ];
    for path in &sentinel_paths {
        write_log(path, b"cross-root synthetic sentinel", 15);
    }
    let sentinels: Vec<_> = sentinel_paths
        .iter()
        .map(|path| {
            (
                fs::read(path).expect("sentinel bytes"),
                fs::metadata(path)
                    .expect("sentinel metadata")
                    .modified()
                    .expect("sentinel timestamp"),
            )
        })
        .collect();

    let output = Command::new(std::env::current_exe().expect("test executable"))
        .args(["--exact", "log_path_probe", "--ignored", "--nocapture"])
        .env("CCR_LOG_PATH_PROBE", "1")
        .env("CCR_LOG_PATH_MODE", mode)
        .env("CCR_ROOT", &root)
        .env("HOME", &home)
        .env("USERPROFILE", &home)
        .env("CCR_LOG_LEVEL", "off")
        .env_remove("RUST_LOG")
        .output()
        .expect("isolated log probe starts");
    assert!(
        output.status.success(),
        "status={} stdout={} stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    assert!(!expired.exists());
    assert_eq!(
        fs::read(&retained).expect("recent managed log remains"),
        b"retained synthetic log"
    );
    assert_eq!(
        fs::read(&unmanaged).expect("unmanaged log remains"),
        b"unmanaged synthetic log"
    );
    let today = selected_logs.join(format!("ccr.log.{}", chrono::Utc::now().format("%Y-%m-%d")));
    assert!(
        today.is_file(),
        "logger initializes the selected root even with off filter"
    );
    for (path, (bytes, modified)) in sentinel_paths.iter().zip(sentinels) {
        assert_eq!(fs::read(path).expect("cross-root sentinel remains"), bytes);
        assert_eq!(
            fs::metadata(path)
                .expect("cross-root metadata")
                .modified()
                .expect("cross-root timestamp"),
            modified
        );
    }
    assert_eq!(
        fs::read_dir(&other_logs)
            .expect("other root inventory")
            .count(),
        1
    );
    assert_eq!(
        fs::read_dir(&home_logs)
            .expect("synthetic home inventory")
            .count(),
        1
    );
}

#[test]
fn console_logger_uses_selected_root_and_preserves_other_roots() {
    assert_log_path_isolation("console");
}

#[test]
fn file_only_logger_uses_selected_root_and_preserves_other_roots() {
    assert_log_path_isolation("file-only");
}
