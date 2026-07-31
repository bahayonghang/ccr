#![allow(clippy::unwrap_used)]

use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use tempfile::TempDir;

struct ProfileOpenFixture {
    _temp_dir: TempDir,
    home: PathBuf,
    root: PathBuf,
}

impl ProfileOpenFixture {
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
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_ccr"));
        cmd.env("CCR_ROOT", &self.root);
        cmd.env("CCR_LOCK_DIR", self.home.join(".locks"));
        cmd.env("HOME", &self.home);
        cmd.env("USERPROFILE", &self.home);
        cmd.env("NO_COLOR", "1");
        cmd.env("CLICOLOR", "0");
        cmd.env("COLUMNS", "120");
        cmd.env("CCR_LOG_LEVEL", "off");
        // 清除终端编辑器变量，确保走 SystemAssociation 分支，避免 CI 环境阻塞
        cmd.env_remove("VISUAL");
        cmd.env_remove("EDITOR");
        cmd
    }

    fn run_output(&self, args: &[&str]) -> Output {
        self.command().args(args).output().unwrap()
    }

    fn run_json_opt(&self, args: &[&str]) -> (Output, Option<Value>) {
        let output = self.run_output(args);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let json = serde_json::from_str::<Value>(&stdout).ok();
        (output, json)
    }

    fn profiles_file(&self, platform: &str) -> PathBuf {
        self.root
            .join("platforms")
            .join(platform)
            .join("profiles.toml")
    }
}

fn assert_success(output: &Output) {
    // `open::that` may fail in headless CI; we treat that as an allowed external failure.
    // The key invariant is that ensure-exists ran (file created) before the open call.
    // If the command failed, it must be due to the open call, not ensure-exists.
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        assert!(
            stderr.contains("系统关联程序启动失败") || stderr.contains("ExternalCommandError"),
            "unexpected failure: status={:?}\nstderr={stderr}",
            output.status,
        );
    }
}

#[test]
fn profile_open_creates_claude_file_when_missing() {
    let fixture = ProfileOpenFixture::new();
    assert!(!fixture.profiles_file("claude").exists());

    let (output, json_opt) = fixture.run_json_opt(&["claude", "profile", "open", "--json"]);

    // 文件必须在 ensure-exists 阶段创建（open 调用之前），与 open 是否成功无关
    assert!(
        fixture.profiles_file("claude").exists(),
        "profiles.toml must be created before open is called"
    );

    // 若 open 成功，验证 JSON 字段
    if output.status.success() {
        let json = json_opt.expect("expected valid JSON on success");
        assert_eq!(json["ok"], true);
        assert_eq!(json["platform"], "claude");
        assert_eq!(json["created"], true);
        assert_eq!(json["registered"], true);
        assert!(
            json["profiles_file"]
                .as_str()
                .unwrap()
                .ends_with("profiles.toml")
        );
        assert_eq!(json["editor"], "system");
    } else {
        assert_success(&output);
    }
}

#[test]
fn profile_open_creates_codex_file_when_missing() {
    let fixture = ProfileOpenFixture::new();
    assert!(!fixture.profiles_file("codex").exists());

    let (output, _) = fixture.run_json_opt(&["codex", "profile", "open", "--json"]);

    assert!(
        fixture.profiles_file("codex").exists(),
        "codex profiles.toml must be created before open is called"
    );
    assert_success(&output);
}

#[test]
fn profile_open_creates_grok_file_when_missing() {
    let fixture = ProfileOpenFixture::new();
    assert!(!fixture.profiles_file("grok").exists());

    let (output, _) = fixture.run_json_opt(&["grok", "profile", "open", "--json"]);

    assert!(
        fixture.profiles_file("grok").exists(),
        "grok profiles.toml must be created before open is called"
    );
    assert_success(&output);
}

#[test]
fn profile_open_idempotent_does_not_overwrite_existing_file() {
    let fixture = ProfileOpenFixture::new();
    let profiles_path = fixture.profiles_file("grok");

    // 预先写入任意内容
    fs::create_dir_all(profiles_path.parent().unwrap()).unwrap();
    let sentinel = b"# SENTINEL_CONTENT_MUST_SURVIVE\n";
    fs::write(&profiles_path, sentinel).unwrap();

    let (output, json_opt) = fixture.run_json_opt(&["grok", "profile", "open", "--json"]);

    // 文件内容不得被覆盖
    assert_eq!(
        fs::read(&profiles_path).unwrap(),
        sentinel,
        "existing file must not be overwritten"
    );

    if output.status.success() {
        let json = json_opt.expect("expected valid JSON");
        assert_eq!(json["created"], false);
    } else {
        assert_success(&output);
    }
}

#[test]
fn profile_open_json_has_all_required_fields() {
    let fixture = ProfileOpenFixture::new();

    let (output, json_opt) = fixture.run_json_opt(&["claude", "profile", "open", "--json"]);

    if output.status.success() {
        let json = json_opt.expect("expected valid JSON");
        // 六个必需字段齐全
        assert!(json.get("ok").is_some(), "missing ok");
        assert!(json.get("platform").is_some(), "missing platform");
        assert!(json.get("profiles_file").is_some(), "missing profiles_file");
        assert!(json.get("created").is_some(), "missing created");
        assert!(json.get("registered").is_some(), "missing registered");
        assert!(json.get("editor").is_some(), "missing editor");
    } else {
        assert_success(&output);
    }
}
