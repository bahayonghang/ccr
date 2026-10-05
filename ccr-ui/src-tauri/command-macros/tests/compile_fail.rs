use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

struct FixtureDirectory(PathBuf);

impl FixtureDirectory {
    fn create() -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock must follow the Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "ccr-command-macro-compile-fail-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(path.join("src")).expect("fixture directory must be created");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for FixtureDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn assert_diagnostic_at(stderr: &str, message: &str, source_line: &str) {
    assert_eq!(
        stderr.matches(message).count(),
        1,
        "diagnostic must occur exactly once:\n{stderr}"
    );

    let diagnostic = stderr
        .split_once(message)
        .expect("expected diagnostic must be present")
        .1;
    let source_offset = diagnostic
        .find(source_line)
        .expect("diagnostic must quote the rejected source line");
    let marker_line = diagnostic[source_offset + source_line.len()..]
        .lines()
        .find(|line| !line.trim().is_empty())
        .expect("diagnostic must include a span marker");

    assert!(
        marker_line.contains('^'),
        "diagnostic must point at the rejected syntax:\n{stderr}"
    );
}

#[test]
fn compiler_diagnostics_keep_messages_and_source_spans() {
    let fixture = FixtureDirectory::create();
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let dependency_path = manifest_dir.to_string_lossy().replace('\\', "/");

    fs::write(
        fixture.path().join("Cargo.toml"),
        format!(
            r#"[package]
name = "ccr-command-macro-compile-fail"
version = "0.0.0"
edition = "2024"
publish = false

[workspace]

[dependencies]
ccr-tauri-command-macros = {{ path = "{dependency_path}" }}
"#,
        ),
    )
    .expect("fixture manifest must be written");
    fs::write(
        fixture.path().join("src/main.rs"),
        r#"use ccr_tauri_command_macros::command;

#[command]
fn sync_command() -> Result<(), String> {
    Ok(())
}

#[command]
async fn wrong_return() -> String {
    String::new()
}

fn main() {}
"#,
    )
    .expect("fixture source must be written");

    let output = Command::new(env!("CARGO"))
        .args(["check", "--offline", "--quiet"])
        .current_dir(fixture.path())
        .env("CARGO_TARGET_DIR", fixture.path().join("target"))
        .env("CARGO_TERM_COLOR", "never")
        .output()
        .expect("nested cargo check must run");

    assert!(!output.status.success(), "invalid fixture must not compile");
    let stderr = String::from_utf8(output.stderr).expect("cargo diagnostics must be UTF-8");
    let normalized = stderr.replace('\\', "/");

    assert_diagnostic_at(
        &normalized,
        "CCR runtime-managed Tauri commands must be async",
        "4 | fn sync_command() -> Result<(), String> {",
    );
    assert_diagnostic_at(
        &normalized,
        "CCR runtime-managed Tauri commands must return Result<T, String>",
        "9 | async fn wrong_return() -> String {",
    );
}
