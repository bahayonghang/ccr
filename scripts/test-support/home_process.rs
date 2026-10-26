//! Re-execute a home-resolution test with a child-only environment.
//! Include only from test or explicitly enabled test-support modules.

use std::path::{Path, PathBuf};
use std::process::Command;

const CASE: &str = "CCR_TEST_HOME_PROCESS_CASE";
const ROOT: &str = "CCR_TEST_HOME_PROCESS_ROOT";
const MARKER: &str = ".ccr-home-process-fixture";

pub fn is_child() -> bool {
    std::env::var_os(CASE).is_some()
}

pub fn child_root() -> PathBuf {
    assert!(is_child(), "home fixture requires an isolated test child");
    let root = PathBuf::from(std::env::var_os(ROOT).expect("child fixture root"));
    assert!(root.is_absolute(), "child fixture root must be absolute");
    let root = root.canonicalize().expect("canonical child fixture root");
    assert!(
        root.join(MARKER).is_file(),
        "child fixture ownership marker"
    );
    root
}

/// True in the parent after its exact test passed in a fresh child.
pub fn run_current_test() -> bool {
    if is_child() {
        return false;
    }
    run_case("default", |_, _| {});
    true
}

/// Used when the existing contract explicitly tests different HOME precedence.
pub fn run_case(case: &str, configure: impl FnOnce(&mut Command, &Path)) {
    assert!(
        !is_child(),
        "home process cases must be dispatched by the parent"
    );
    let name = std::thread::current()
        .name()
        .expect("Rust test thread name")
        .to_owned();
    let fixture = tempfile::tempdir().expect("home process fixture");
    let root = fixture.path().canonicalize().expect("canonical fixture");
    std::fs::write(root.join(MARKER), name.as_bytes()).expect("fixture marker");
    let before = (std::env::var_os("HOME"), std::env::var_os("USERPROFILE"));
    let mut command = Command::new(std::env::current_exe().expect("current test executable"));
    command
        .args(["--exact", &name, "--nocapture"])
        .env(CASE, case)
        .env(ROOT, &root)
        .env("HOME", &root)
        .env("USERPROFILE", &root);
    configure(&mut command, &root);
    let output = command.output().expect("isolated test child");
    assert_eq!(
        before,
        (std::env::var_os("HOME"), std::env::var_os("USERPROFILE")),
        "child test changed parent home environment"
    );
    assert!(
        output.status.success(),
        "{name} ({case}) failed: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("1 passed;"),
        "{name} ({case}) did not execute exactly one passing test: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

pub fn case() -> String {
    std::env::var(CASE).expect("isolated home case")
}

/// A repeated lifecycle fixture may reset only its verified, parent-owned home.
pub fn reset_child_home() {
    let root = child_root();
    for entry in std::fs::read_dir(&root).expect("child fixture contents") {
        let entry = entry.expect("child fixture entry");
        if entry.file_name() == MARKER {
            continue;
        }
        let path = entry.path();
        let kind = entry.file_type().expect("fixture entry type");
        if kind.is_dir() && !kind.is_symlink() {
            let resolved = path.canonicalize().expect("canonical fixture child");
            assert!(resolved.starts_with(&root) && resolved != root);
            std::fs::remove_dir_all(&path).expect("remove owned fixture child");
        } else {
            std::fs::remove_file(&path).expect("remove owned fixture file");
        }
    }
}
