#![allow(clippy::unwrap_used)]

use serde_json::{Value, json};
use std::fs;
use std::process::{Command, Output};
use tempfile::TempDir;

const SCOPE: &str = "https://auth.x.ai::client";
const SECRET: &str = "GROK_AUTH_SECRET_SENTINEL";

struct Fixture(TempDir);

impl Fixture {
    fn new() -> Self {
        let fixture = Self(tempfile::tempdir().unwrap());
        for dir in ["grok", "ccr/platforms/grok"] {
            fs::create_dir_all(fixture.0.path().join(dir)).unwrap();
        }
        for file in [
            "grok/config.toml",
            "grok/mcp_credentials.json",
            "ccr/platforms/grok/profiles.toml",
        ] {
            fs::write(fixture.0.path().join(file), "preserve").unwrap();
        }
        fixture
    }

    fn command(&self, args: &[&str]) -> Command {
        let home = self.0.path();
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_ccr"));
        cmd.args(["grok", "auth"])
            .args(args)
            .env("HOME", home)
            .env("USERPROFILE", home)
            .env("CCR_ROOT", home.join("ccr"))
            .env("CCR_LOCK_DIR", home.join("locks"))
            .env("GROK_HOME", home.join("grok"))
            .env_remove("CCR_CONFIG_PATH")
            .env("NO_COLOR", "1")
            .env("CCR_LOG_LEVEL", "off");
        cmd
    }

    fn run(&self, args: &[&str], success: bool) -> Output {
        let output = self.command(args).output().unwrap();
        for bytes in [&output.stdout, &output.stderr] {
            assert!(!String::from_utf8_lossy(bytes).contains(SECRET));
        }
        assert_eq!(
            output.status.success(),
            success,
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        output
    }

    fn json(&self, args: &[&str]) -> Value {
        serde_json::from_slice(&self.run(args, true).stdout).unwrap()
    }

    fn runtime(&self, user: &str, token: &str) {
        self.write_runtime(
            json!({(SCOPE): credential(user, token), "xai::api_key": {"key": "preserve"}}),
        );
    }

    fn write_runtime(&self, value: Value) {
        fs::write(self.0.path().join("grok/auth.json"), value.to_string()).unwrap();
    }

    fn read(&self, file: &str) -> Vec<u8> {
        fs::read(self.0.path().join(file)).unwrap()
    }
}

fn credential(user: &str, token: &str) -> Value {
    json!({"auth_mode":"oidc", "oidc_issuer":"https://auth.x.ai", "oidc_client_id":"client", "key":token, "user_id":user, "create_time":"2026-01-01T00:00:00Z"})
}

#[test]
fn grok_auth_save_original_command() {
    let f = Fixture::new();
    f.runtime("a", SECRET);
    let before = f.read("grok/auth.json");
    f.run(&["save", "gmail"], true);
    assert_eq!(f.read("grok/auth.json"), before);
    assert_eq!(f.json(&["list", "--json"])["accounts"][0]["name"], "gmail");
    f.run(&["list"], true);
}

#[test]
fn grok_auth_round_trip_delete_and_off_preserve_boundaries() {
    let f = Fixture::new();
    f.runtime("a", SECRET);
    f.json(&["save", "a", "--json"]);
    f.runtime("b", "b-token");
    f.json(&["save", "b", "--json"]);
    f.runtime("a", "a-refreshed");
    assert_eq!(f.json(&["switch", "b", "--json"])["outgoing_saved"], true);
    f.json(&["switch", "a", "--json"]);
    let runtime: Value = serde_json::from_slice(&f.read("grok/auth.json")).unwrap();
    assert_eq!(runtime[SCOPE]["key"], "a-refreshed");
    assert_eq!(runtime["xai::api_key"]["key"], "preserve");
    let before = f.read("grok/auth.json");
    f.json(&["delete", "b", "-f", "--json"]);
    assert_eq!(f.read("grok/auth.json"), before);
    f.json(&["off", "--json"]);
    assert_eq!(f.json(&["current", "--json"])["logged_in"], false);
    assert_eq!(
        f.json(&["list", "--json"])["accounts"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    for file in [
        "grok/config.toml",
        "grok/mcp_credentials.json",
        "ccr/platforms/grok/profiles.toml",
    ] {
        assert_eq!(f.read(file), b"preserve");
    }
}

#[test]
fn grok_auth_scope_selection_and_overwrite_fail_without_writes() {
    let f = Fixture::new();
    assert_eq!(f.json(&["list", "--json"])["accounts"], json!([]));
    f.run(&["save", "none"], false);
    assert!(!f.0.path().join("ccr/platforms/grok/auth").exists());
    fs::write(f.0.path().join("grok/auth.json"), "damaged").unwrap();
    assert!(f.json(&["list", "--json"])["runtime_error"].is_string());
    assert_eq!(f.json(&["current", "--json"])["logged_in"], true);
    f.run(&["save", "none"], false);
    assert!(!f.0.path().join("ccr/platforms/grok/auth").exists());
    let mut second = credential("b", SECRET);
    second["oidc_client_id"] = json!("other");
    f.write_runtime(json!({(SCOPE): credential("a", SECRET), "https://auth.x.ai::other": second}));
    let before = f.read("grok/auth.json");
    f.run(&["save", "a"], false);
    f.run(&["save", "a", "--scope", "invalid"], false);
    assert!(!f.0.path().join("ccr/platforms/grok/auth").exists());
    f.json(&["save", "a", "--scope", SCOPE, "--json"]);
    let store = f.read("ccr/platforms/grok/auth/accounts.json");
    f.run(&["save", "a", "--scope", SCOPE, "-y"], false);
    assert_eq!(f.read("ccr/platforms/grok/auth/accounts.json"), store);
    f.json(&["save", "a", "--scope", SCOPE, "--force", "--json"]);
    assert_eq!(f.read("grok/auth.json"), before);
}

#[test]
fn grok_auth_delete_cancel_global_yes_and_error_exit() {
    use std::io::Write;
    use std::process::Stdio;
    let f = Fixture::new();
    f.runtime("a", SECRET);
    f.json(&["save", "a", "--json"]);
    let before = f.read("ccr/platforms/grok/auth/accounts.json");
    let mut child = f
        .command(&["delete", "a", "--json"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"n\n").unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap()["cancelled"],
        true
    );
    assert_eq!(f.read("ccr/platforms/grok/auth/accounts.json"), before);
    f.run(&["switch", "missing"], false);
    f.run(&["delete", "missing", "--force"], false);
    f.json(&["delete", "a", "-y", "--json"]);
    assert_eq!(f.json(&["list", "--json"])["accounts"], json!([]));
    assert!(f.0.path().join("grok/auth.json").exists());
}

#[test]
fn grok_auth_corruption_and_unknown_outgoing_are_safe() {
    let f = Fixture::new();
    f.runtime("a", SECRET);
    f.json(&["save", "a", "--json"]);
    f.runtime("unknown", SECRET);
    let before = f.read("grok/auth.json");
    f.run(&["switch", "a"], false);
    assert_eq!(f.read("grok/auth.json"), before);
    fs::write(
        f.0.path().join("ccr/platforms/grok/auth/accounts.json"),
        "broken",
    )
    .unwrap();
    for args in [
        vec!["list", "--json"],
        vec!["save", "b"],
        vec!["switch", "a"],
        vec!["delete", "a", "-f"],
    ] {
        f.run(&args, false);
    }
    assert_eq!(f.json(&["current", "--json"])["logged_in"], true);
    assert_eq!(f.read("grok/auth.json"), before);
    assert_eq!(f.read("ccr/platforms/grok/auth/accounts.json"), b"broken");
}

#[test]
fn grok_auth_help_describes_saved_accounts() {
    let f = Fixture::new();
    let output = f.run(&["--help"], true);
    let help = String::from_utf8(output.stdout).unwrap();
    for command in ["save", "list", "switch", "delete", "current", "off"] {
        assert!(help.contains(command));
    }
    assert!(help.contains("--scope"));
    assert!(!help.contains("不保存 Grok 账号快照"));
    for command in ["save", "list", "switch", "delete"] {
        f.run(&[command, "--help"], true);
    }
}
