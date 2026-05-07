#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use tempfile::TempDir;

struct LegacyRoutingFixture {
    _temp_dir: TempDir,
    home: PathBuf,
    root: PathBuf,
}

impl LegacyRoutingFixture {
    fn new() -> Self {
        let temp_dir = tempfile::tempdir().unwrap();
        let home = temp_dir.path().join("home");
        let root = home.join(".ccr");
        fs::create_dir_all(&root).unwrap();
        Self {
            _temp_dir: temp_dir,
            home,
            root,
        }
    }

    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_ccr"));
        command.env("CCR_ROOT", &self.root);
        command.env("CCR_LOCK_DIR", self.home.join(".locks"));
        command.env("HOME", &self.home);
        command.env("USERPROFILE", &self.home);
        command.env("NO_COLOR", "1");
        command.env("CLICOLOR", "0");
        command.env("COLUMNS", "120");
        command
    }

    fn run_output(&self, args: &[&str]) -> Output {
        self.command().args(args).output().unwrap()
    }
}

fn assert_migration_error(output: Output, expected: &[&str]) {
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "expected failure\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        stderr
    );
    assert!(stderr.contains("legacy"), "{stderr}");
    assert!(stderr.contains("current_platform"), "{stderr}");
    for needle in expected {
        assert!(stderr.contains(needle), "missing {needle:?}\n{stderr}");
    }
}

#[test]
fn root_switch_reports_platform_scoped_profile_migration() {
    let fixture = LegacyRoutingFixture::new();

    assert_migration_error(
        fixture.run_output(&["switch", "team"]),
        &[
            "ccr claude profile switch team",
            "ccr codex profile switch team",
        ],
    );
}

#[test]
fn shortcut_switch_reports_platform_scoped_profile_migration() {
    let fixture = LegacyRoutingFixture::new();

    assert_migration_error(
        fixture.run_output(&["team"]),
        &[
            "ccr claude profile switch team",
            "ccr codex profile switch team",
        ],
    );
}

#[test]
fn platform_state_commands_report_runtime_migration() {
    let fixture = LegacyRoutingFixture::new();

    assert_migration_error(
        fixture.run_output(&["platform", "switch", "codex"]),
        &["ccr current", "ccr claude profile", "ccr codex profile"],
    );
    assert_migration_error(
        fixture.run_output(&["platform", "current"]),
        &["ccr current", "ccr claude profile", "ccr codex profile"],
    );
}

#[test]
fn platform_profile_commands_report_platform_scoped_profile_migration() {
    let fixture = LegacyRoutingFixture::new();

    assert_migration_error(
        fixture.run_output(&["platform", "profile", "create", "gemini", "team"]),
        &["ccr current", "ccr claude profile", "ccr codex profile"],
    );
}
