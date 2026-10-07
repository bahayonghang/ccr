#![allow(clippy::unwrap_used)]

use serde_json::{Value, json};
use std::fs;
use std::io::Write;
use std::process::{Command, Output, Stdio};
use tempfile::TempDir;

const SECRET: &str = "CLI_OUTPUT_SECRET_SENTINEL";
const SCOPE: &str = "https://auth.x.ai::client";

struct Fixture(TempDir);

impl Fixture {
    fn new() -> Self {
        let fixture = Self(tempfile::tempdir().unwrap());
        for dir in [
            "ccr/platforms/codex",
            "ccr/platforms/claude",
            "ccr/platforms/grok",
            "codex",
            "claude",
            "grok",
        ] {
            fs::create_dir_all(fixture.0.path().join(dir)).unwrap();
        }
        fixture.write(
            "codex/config.toml",
            b"cli_auth_credentials_store = \"file\"\n",
        );
        fixture
    }

    fn write(&self, path: &str, bytes: &[u8]) {
        fs::write(self.0.path().join(path), bytes).unwrap();
    }

    fn read(&self, path: &str) -> Vec<u8> {
        fs::read(self.0.path().join(path)).unwrap()
    }

    fn command(&self, args: &[&str]) -> Command {
        let home = self.0.path();
        let mut command = Command::new(env!("CARGO_BIN_EXE_ccr"));
        for key in [
            "CCR_CONFIG_PATH",
            "CLICOLOR_FORCE",
            "FORCE_COLOR",
            "OPENAI_API_KEY",
            "ANTHROPIC_BASE_URL",
            "ANTHROPIC_API_KEY",
            "ANTHROPIC_AUTH_TOKEN",
            "CLAUDE_CODE_OAUTH_TOKEN",
            "CLAUDE_CODE_USE_BEDROCK",
            "CLAUDE_CODE_USE_VERTEX",
            "CLAUDE_CODE_USE_FOUNDRY",
            "XAI_API_KEY",
        ] {
            command.env_remove(key);
        }
        command
            .args(args)
            .current_dir(home)
            .env("HOME", home)
            .env("USERPROFILE", home)
            .env("CCR_ROOT", home.join("ccr"))
            .env("CCR_DATA_DIR", home.join("ccr"))
            .env("CCR_LOCK_DIR", home.join("locks"))
            .env("CCR_CODEX_DIR", home.join("codex"))
            .env("CODEX_HOME", home.join("codex"))
            .env("CLAUDE_CONFIG_DIR", home.join("claude"))
            .env("CLAUDE_JSON_PATH", home.join("claude-state.json"))
            .env("CCR_SETTINGS_PATH", home.join("claude/settings.json"))
            .env("CCR_BACKUP_DIR", home.join("backups"))
            .env("GROK_HOME", home.join("grok"))
            .env("NO_COLOR", "1")
            .env("CLICOLOR", "0")
            .env("CCR_LOG_LEVEL", "off")
            .env("COLUMNS", "120");
        command
    }

    fn run(&self, args: &[&str]) -> Output {
        let output = self.command(args).output().unwrap();
        assert_clean(&output);
        output
    }

    fn run_success(&self, args: &[&str]) -> Output {
        let output = self.run(args);
        assert!(output.status.success(), "{}", text(&output.stderr));
        output
    }

    fn codex_login(&self, with_email: bool) {
        let id_token = if with_email {
            "eyJhbGciOiJub25lIn0.eyJlbWFpbCI6InRlYWNoZXJAZXhhbXBsZS50ZXN0Iiwic3ViIjoidGVhY2hlciIsImNoYXRncHRfdXNlcl9pZCI6InRlYWNoZXIifQ.signature"
        } else {
            "eyJhbGciOiJub25lIn0.eyJzdWIiOiJ0ZWFjaGVyIiwiY2hhdGdwdF91c2VyX2lkIjoidGVhY2hlciJ9.signature"
        };
        self.write("codex/auth.json", json!({"OPENAI_API_KEY":null,"tokens":{"id_token":id_token,"access_token":SECRET,"refresh_token":SECRET,"account_id":"acct-teacher"},"last_refresh":"2026-01-01T00:00:00Z"}).to_string().as_bytes());
    }

    #[cfg(not(target_os = "macos"))]
    fn claude_login(&self, with_email: bool) {
        self.write("claude/.credentials.json", json!({"claudeAiOauth":{"accessToken":SECRET,"refreshToken":SECRET,"expiresAt":"2099-01-01T00:00:00Z","subscriptionType":"pro","rateLimitTier":null,"scopes":null}}).to_string().as_bytes());
        if with_email {
            self.write("claude-state.json", json!({"oauthAccount":{"accountUuid":"teacher-uuid","emailAddress":"teacher@example.test"},"unknown":"preserve"}).to_string().as_bytes());
        }
    }

    fn grok_login(&self) {
        self.write("grok/auth.json", json!({(SCOPE):{"auth_mode":"oidc","oidc_issuer":"https://auth.x.ai","oidc_client_id":"client","key":SECRET,"user_id":"teacher","create_time":"2026-01-01T00:00:00Z"}}).to_string().as_bytes());
    }

    fn cancel_delete(&self, platform: &str) -> Output {
        let mut child = self
            .command(&[platform, "auth", "delete", "teacher"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(b"n\n").unwrap();
        let output = child.wait_with_output().unwrap();
        assert_clean(&output);
        assert!(output.status.success());
        assert!(text(&output.stdout).contains("已取消删除"));
        assert!(!text(&output.stdout).contains("成功:"));
        assert!(!text(&output.stdout).contains("下一步"));
        output
    }
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).unwrap()
}

fn assert_clean(output: &Output) {
    for bytes in [&output.stdout, &output.stderr] {
        let value = text(bytes);
        assert!(!value.contains(SECRET), "credential leaked");
        assert!(!value.contains('\u{1b}'), "ANSI in captured output");
        for label in [
            "[OK]", "[INFO]", "[WARN]", "[ERR]", "[STEP]", "[FAIL]", "[SKIP]",
        ] {
            assert!(!value.contains(label), "legacy label {label}");
        }
    }
}

#[test]
fn codex_save_result_fields_and_suggestion_preserve_runtime() {
    let f = Fixture::new();
    f.codex_login(true);
    let before = f.read("codex/auth.json");
    let output = f.run_success(&[
        "codex",
        "auth",
        "save",
        "teacher",
        "--description",
        "课程\n第二行",
    ]);
    assert_eq!(
        text(&output.stdout),
        "成功: 已保存账号 teacher\n  描述: 课程\n    第二行\n  邮箱: tea***@example.test\n\n下一步\n  查看账号\n    ccr codex auth list\n"
    );
    assert!(output.stderr.is_empty());
    assert_eq!(f.read("codex/auth.json"), before);
    assert!(
        f.0.path()
            .join("ccr/platforms/codex/auth/teacher.json")
            .exists()
    );
}

#[test]
fn codex_save_missing_fields_and_overwrite_command_handles_leading_hyphen() {
    for name in ["teacher", "-teacher"] {
        let f = Fixture::new();
        f.codex_login(false);
        let output = f.run_success(&["codex", "auth", "save", "--", name]);
        assert_eq!(
            text(&output.stdout),
            format!("成功: 已保存账号 {name}\n\n下一步\n  查看账号\n    ccr codex auth list\n")
        );
        let snapshot = f.read(&format!("ccr/platforms/codex/auth/{name}.json"));
        let duplicate = f.run_success(&["codex", "auth", "save", "--", name]);
        assert!(text(&duplicate.stderr).starts_with("错误: 保存失败:"));
        assert!(text(&duplicate.stdout).contains("覆盖会替换该账号已保存的凭据快照"));
        let command = format!("ccr codex auth save --force -- {name}");
        assert!(text(&duplicate.stdout).contains(&command));
        assert_eq!(
            f.read(&format!("ccr/platforms/codex/auth/{name}.json")),
            snapshot
        );
        let arguments: Vec<_> = command.split_whitespace().skip(1).collect();
        f.run_success(&arguments);
    }
}

#[test]
fn codex_unlogged_and_unsupported_keep_legacy_zero_exit_and_error_stream() {
    let f = Fixture::new();
    let unlogged = f.run_success(&["codex", "auth", "save", "teacher"]);
    assert_eq!(text(&unlogged.stderr), "错误: 未登录 Codex\n");
    assert_eq!(
        text(&unlogged.stdout),
        "\n下一步\n  登录 Codex\n    codex login\n"
    );
    f.write(
        "codex/config.toml",
        b"cli_auth_credentials_store = \"keyring\"\n",
    );
    let unsupported = f.run_success(&["codex", "auth", "save", "teacher"]);
    assert!(text(&unsupported.stderr).contains("暂不支持 CCR 保存账号"));
    let stdout = text(&unsupported.stdout);
    assert!(stdout.contains("  凭据存储: keyring"));
    assert!(stdout.contains("cli_auth_credentials_store 切换为 file"));
    assert!(!stdout.contains("成功:"));
    assert!(
        !f.0.path()
            .join("ccr/platforms/codex/auth/teacher.json")
            .exists()
    );
}

#[test]
fn codex_current_list_and_json_remain_state_specific() {
    let f = Fixture::new();
    f.codex_login(true);
    let unsaved = text(&f.run_success(&["codex", "auth", "current"]).stdout);
    assert!(unsaved.contains("警告: 当前登录尚未保存"));
    assert!(unsaved.contains("ccr codex auth save --help"));
    f.run_success(&["codex", "auth", "save", "teacher"]);
    let list = text(&f.run_success(&["codex", "auth", "list"]).stdout);
    assert!(list.contains("共 1 个已保存账号"));
    assert!(!list.contains("成功:"));
    assert!(list.contains("ccr codex auth switch --help"));
    assert!(!list.contains("<名称>"));
    let output = f.run_success(&["codex", "auth", "current", "--json"]);
    let dto: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(dto["current_auth_info"]["email"], "teacher@example.test");
    assert_eq!(dto["runtime_summary"]["current_auth_name"], "teacher");
    assert_eq!(dto["auth_state"]["status"], "valid");
    assert!(!text(&output.stdout).contains("下一步"));
    assert!(output.stderr.is_empty());
    f.write(
        "codex/auth.json",
        json!({"OPENAI_API_KEY":SECRET}).to_string().as_bytes(),
    );
    let api = text(&f.run_success(&["codex", "auth", "current"]).stdout);
    assert!(api.contains("  认证模式: API Key"));
    assert!(!api.contains("save --help"));
}

#[test]
fn codex_cancel_and_empty_export_have_no_success_result() {
    let f = Fixture::new();
    let empty = text(
        &f.run_success(&["codex", "auth", "export", "--no-secrets"])
            .stdout,
    );
    assert!(empty.starts_with("没有已保存的账号可导出\n"));
    assert!(!empty.contains("成功:"));
    f.codex_login(true);
    f.run_success(&["codex", "auth", "save", "teacher"]);
    let before = f.read("ccr/platforms/codex/auth_registry.toml");
    let runtime = f.read("codex/auth.json");
    f.cancel_delete("codex");
    assert_eq!(f.read("ccr/platforms/codex/auth_registry.toml"), before);
    assert_eq!(f.read("codex/auth.json"), runtime);
}

#[test]
#[cfg(target_os = "windows")]
fn codex_switch_running_process_warning_preserves_result_and_exit_code() {
    const PROBE: &str = "CCR_C2_RUNNING_WARNING_PROBE";
    if let Some(ready_path) = std::env::var_os(PROBE) {
        fs::write(ready_path, b"ready").unwrap();
        std::thread::sleep(std::time::Duration::from_secs(10));
        return;
    }

    struct WarningProcess(std::process::Child);
    impl Drop for WarningProcess {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }

    let f = Fixture::new();
    f.codex_login(true);
    f.run_success(&["codex", "auth", "save", "teacher"]);
    let runtime = f.read("codex/auth.json");
    let snapshot = f.read("ccr/platforms/codex/auth/teacher.json");
    let probe_exe = f.0.path().join("codex.exe");
    let ready_path = f.0.path().join("warning-probe-ready");
    fs::copy(std::env::current_exe().unwrap(), &probe_exe).unwrap();
    let mut probe = WarningProcess(
        Command::new(probe_exe)
            .args([
                "--exact",
                "output_presentation::codex_switch_running_process_warning_preserves_result_and_exit_code",
                "--nocapture",
            ])
            .env(PROBE, &ready_path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while !ready_path.exists() {
        assert!(
            probe.0.try_wait().unwrap().is_none(),
            "probe exited before readiness"
        );
        assert!(
            std::time::Instant::now() < deadline,
            "probe readiness timeout"
        );
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    let output = f.run_success(&["codex", "auth", "switch", "teacher"]);
    let stdout = text(&output.stdout);
    assert!(
        stdout.contains("警告: 检测到 Codex 进程正在运行"),
        "{stdout}"
    );
    assert!(stdout.contains(&probe.0.id().to_string()), "{stdout}");
    assert!(stdout.contains("切换账号可能导致正在运行的 Codex 会话出现问题"));
    assert!(stdout.contains("成功: 已切换到账号 teacher"));
    assert!(stdout.contains("ccr codex auth current"));
    assert!(!text(&output.stderr).contains("错误:"));
    let mut expected_runtime: Value = serde_json::from_slice(&runtime).unwrap();
    assert_eq!(expected_runtime["OPENAI_API_KEY"], Value::Null);
    expected_runtime
        .as_object_mut()
        .unwrap()
        .remove("OPENAI_API_KEY");
    assert_eq!(
        serde_json::from_slice::<Value>(&f.read("codex/auth.json")).unwrap(),
        expected_runtime
    );
    assert_eq!(f.read("ccr/platforms/codex/auth/teacher.json"), snapshot);
    assert!(
        probe.0.try_wait().unwrap().is_none(),
        "switch terminated the probe"
    );
}

#[test]
#[cfg(not(target_os = "macos"))]
fn claude_save_duplicate_cancel_and_switch_warning_preserve_boundaries() {
    let f = Fixture::new();
    f.claude_login(true);
    let runtime = f.read("claude/.credentials.json");
    let state = f.read("claude-state.json");
    let save = text(&f.run_success(&["claude", "auth", "save", "teacher"]).stdout);
    assert!(
        save.starts_with("成功: 已保存 Claude 官方账号 teacher\n  邮箱: tea***@example.test\n")
    );
    assert_eq!(save.matches("下一步").count(), 1);
    assert!(!save.contains("switch"));
    let duplicate = f.run_success(&["claude", "auth", "save", "teacher"]);
    assert!(text(&duplicate.stderr).contains("保存失败"));
    assert!(text(&duplicate.stdout).contains("ccr claude auth save --force -- teacher"));
    let registry = f.read("ccr/platforms/claude/auth_registry.toml");
    f.cancel_delete("claude");
    assert_eq!(f.read("ccr/platforms/claude/auth_registry.toml"), registry);
    let switched = f
        .command(&["claude", "auth", "switch", "teacher"])
        .env("ANTHROPIC_API_KEY", SECRET)
        .output()
        .unwrap();
    assert_clean(&switched);
    assert!(switched.status.success());
    let stdout = text(&switched.stdout);
    assert!(stdout.starts_with("成功: 已切换到 Claude 官方账号 teacher"));
    assert!(stdout.contains("警告: 仍存在 CCR 不会自动清理的认证来源"));
    assert!(stdout.contains("potential"));
    assert!(stdout.contains("ccr claude auth current"));
    assert_eq!(
        serde_json::from_slice::<Value>(&f.read("claude/.credentials.json")).unwrap(),
        serde_json::from_slice::<Value>(&runtime).unwrap()
    );
    assert_eq!(f.read("claude-state.json"), state);
}

#[test]
#[cfg(not(target_os = "macos"))]
fn claude_empty_and_missing_email_use_neutral_fields_and_save_help() {
    let f = Fixture::new();
    let empty = text(&f.run_success(&["claude", "auth", "list"]).stdout);
    assert!(empty.contains("尚未保存任何官方账号快照"));
    assert!(!empty.contains("成功:"));
    assert!(empty.contains("claude login"));
    f.claude_login(false);
    let current = text(&f.run_success(&["claude", "auth", "current"]).stdout);
    assert!(current.contains("警告: 当前官方订阅登录尚未保存"));
    assert!(current.contains("ccr claude auth save --help"));
    let save = text(&f.run_success(&["claude", "auth", "save", "teacher"]).stdout);
    assert!(!save.contains("邮箱:"));
    let output = f.run_success(&["claude", "auth", "current", "--json"]);
    let dto: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(dto.get("runtime_summary").is_some());
    assert!(dto.get("current_auth_info").is_some());
    assert!(output.stderr.is_empty());
}

#[test]
fn grok_mutation_cancel_failure_and_json_preserve_state_and_streams() {
    let f = Fixture::new();
    f.grok_login();
    let runtime = f.read("grok/auth.json");
    let save = f.run_success(&["grok", "auth", "save", "teacher"]);
    assert_eq!(
        text(&save.stdout),
        "成功: 已保存 Grok 账号 teacher\n\n下一步\n  查看账号\n    ccr grok auth list\n"
    );
    assert_eq!(f.read("grok/auth.json"), runtime);
    let store = f.read("ccr/platforms/grok/auth/accounts.json");
    f.cancel_delete("grok");
    assert_eq!(f.read("ccr/platforms/grok/auth/accounts.json"), store);
    let failed = f.run(&["grok", "auth", "switch", "missing"]);
    assert!(!failed.status.success());
    assert!(text(&failed.stderr).contains("错误:"));
    assert!(!text(&failed.stdout).contains("成功:"));
    assert_eq!(f.read("grok/auth.json"), runtime);
    let mut command = f.command(&["grok", "auth", "save", "teacher", "--force", "--json"]);
    command.env("CLICOLOR_FORCE", "1");
    let output = command.output().unwrap();
    let dto: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        dto,
        json!({"name":"teacher","cancelled":false,"outgoing_saved":false,"warnings":[]})
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn auth_off_noop_is_neutral_and_json_shape_is_preserved() {
    let f = Fixture::new();
    for platform in ["codex", "grok"] {
        let human = f.run_success(&[platform, "auth", "off"]);
        assert!(text(&human.stdout).starts_with("当前没有可清除的官方凭据文件"));
        assert!(!text(&human.stdout).contains("成功:"));
        let dto: Value =
            serde_json::from_slice(&f.run_success(&[platform, "auth", "off", "--json"]).stdout)
                .unwrap();
        assert_eq!(dto, json!({"ok":true,"changed":false,"path":"file"}));
    }
}

#[test]
fn codex_provider_key_does_not_offer_oauth_save() {
    let f = Fixture::new();
    f.write("codex/config.toml", b"cli_auth_credentials_store = \"file\"\nmodel_provider = \"fixture\"\n[model_providers.fixture]\nname = \"fixture\"\nbase_url = \"https://provider.example.test/v1\"\nwire_api = \"responses\"\nenv_key = \"CCR_OUTPUT_FIXTURE_API_KEY\"\nrequires_openai_auth = false\n");
    f.write(
        "codex/auth.json",
        json!({"CCR_OUTPUT_FIXTURE_API_KEY":SECRET})
            .to_string()
            .as_bytes(),
    );
    let output = f
        .command(&["codex", "auth", "current"])
        .env("CCR_OUTPUT_FIXTURE_API_KEY", SECRET)
        .output()
        .unwrap();
    assert_clean(&output);
    assert!(output.status.success());
    let stdout = text(&output.stdout);
    assert!(
        stdout.contains("  认证模式: Provider Key (CCR_OUTPUT_FIXTURE_API_KEY)"),
        "{stdout}"
    );
    assert!(stdout.contains("Provider Key 模式无需保存账号"));
    assert!(!stdout.contains("save --help"));
}

#[test]
fn codex_rename_update_json_keep_dto_messages_and_optional_description() {
    let f = Fixture::new();
    f.codex_login(true);
    f.run_success(&["codex", "auth", "save", "teacher"]);
    let renamed: Value = serde_json::from_slice(
        &f.run_success(&["codex", "auth", "rename", "teacher", "tutor", "--json"])
            .stdout,
    )
    .unwrap();
    assert_eq!(
        renamed,
        json!({"ok":true,"old_name":"teacher","new_name":"tutor","message":"已重命名 Codex Auth 'teacher' -> 'tutor'"})
    );
    let updated: Value = serde_json::from_slice(
        &f.run_success(&[
            "codex",
            "auth",
            "update",
            "tutor",
            "--description",
            "课程",
            "--json",
        ])
        .stdout,
    )
    .unwrap();
    assert_eq!(
        updated,
        json!({"ok":true,"name":"tutor","description":"课程","message":"已更新 Codex auth 'tutor' 的描述"})
    );
    let cleared: Value = serde_json::from_slice(
        &f.run_success(&[
            "codex",
            "auth",
            "update",
            "tutor",
            "--clear-description",
            "--json",
        ])
        .stdout,
    )
    .unwrap();
    assert!(cleared.get("description").is_none());
}

#[test]
#[cfg(not(target_os = "macos"))]
fn claude_overwrite_suggestion_safely_parses_leading_hyphen_name() {
    let f = Fixture::new();
    f.claude_login(false);
    f.run_success(&["claude", "auth", "save", "--", "-teacher"]);
    let duplicate = f.run_success(&["claude", "auth", "save", "--", "-teacher"]);
    let command = "ccr claude auth save --force -- -teacher";
    assert!(text(&duplicate.stdout).contains(command));
    let args: Vec<_> = command.split_whitespace().skip(1).collect();
    f.run_success(&args);
}

#[test]
fn clean_preview_fields_suggestion_and_empty_result_are_neutral() {
    let f = Fixture::new();
    let directory =
        f.0.path()
            .join("a-long-planning-directory-which-must-remain-complete");
    fs::create_dir_all(&directory).unwrap();
    let planning_file = directory.join("task_plan.md");
    fs::write(&planning_file, b"synthetic planning content").unwrap();
    let output = f
        .command(&["clean", "planfiles", "--dry-run"])
        .current_dir(&directory)
        .env("COLUMNS", "40")
        .output()
        .unwrap();
    assert_clean(&output);
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = text(&output.stdout);
    assert!(stdout.contains(&format!("  扫描目录: {}", directory.display())));
    assert!(stdout.contains("  命中: task_plan.md"));
    assert!(stdout.contains("  命中数量: 1 个"));
    assert!(stdout.contains("\n下一步\n  执行实际清理\n    ccr clean\n"));
    assert!(!stdout.contains("成功:"));
    assert!(!stdout.contains("警告:"));
    assert_eq!(
        fs::read(&planning_file).unwrap(),
        b"synthetic planning content"
    );
    let empty_dir = f.0.path().join("empty-planning-directory");
    fs::create_dir_all(&empty_dir).unwrap();
    let empty = f
        .command(&["clean", "planfiles", "--dry-run"])
        .current_dir(&empty_dir)
        .output()
        .unwrap();
    assert_clean(&empty);
    assert!(empty.status.success());
    assert!(text(&empty.stdout).contains("没有找到需要清理的规划文件"));
    assert!(!text(&empty.stdout).contains("成功:"));
    assert!(!text(&empty.stdout).contains("下一步"));
}

#[test]
fn dumb_cli_startup_disables_decorations_under_forced_color() {
    let f = Fixture::new();
    let output = f
        .command(&["clean", "planfiles", "--dry-run"])
        .current_dir(f.0.path())
        .env("TERM", "dumb")
        .env("CLICOLOR_FORCE", "1")
        .output()
        .unwrap();
    assert_clean(&output);
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert!(text(&output.stdout).contains("清理规划文件"));
}

#[test]
fn platform_counts_keep_json_and_stdout_contracts() {
    let f = Fixture::new();
    f.write("ccr/config.toml", b"[claude]\nenabled = true\n");
    let list = f.run_success(&["platform", "list"]);
    let stdout = text(&list.stdout);
    let platform_count = ccr_cli::platforms::PlatformRegistry::new()
        .list_platform_info()
        .len();
    assert!(
        stdout.contains(&format!("Found {platform_count} platforms")),
        "{stdout}"
    );
    assert!(!stdout.contains("成功:"));
    assert!(stdout.contains("\n下一步\n  查看运行时状态\n    ccr current\n"));
    let list_json: Value =
        serde_json::from_slice(&f.run_success(&["platform", "list", "--json"]).stdout).unwrap();
    let platforms = list_json["platforms"].as_array().unwrap();
    assert_eq!(platforms.len(), platform_count);
    assert!(
        platforms
            .iter()
            .any(|platform| platform["name"] == "claude" && platform["enabled"] == true)
    );
}

#[test]
fn doctor_failure_keeps_json_stdout_and_read_only_contracts() {
    let f = Fixture::new();
    f.write("ccr/config.toml", b"[claude]\nenabled = true\n");
    f.write("claude/settings.json", b"invalid synthetic json");
    let config_before = f.read("ccr/config.toml");
    let runtime_before = f.read("claude/settings.json");
    let json_output = f.run(&["doctor", "--platform", "claude", "--json"]);
    assert_eq!(json_output.status.code(), Some(1));
    assert!(json_output.stderr.is_empty());
    let report: Value = serde_json::from_slice(&json_output.stdout).unwrap();
    let human = f.run(&["doctor", "--platform", "claude", "--verbose"]);
    assert_eq!(human.status.code(), Some(1));
    assert!(human.stderr.is_empty());
    let stdout = text(&human.stdout);
    assert!(report["summary"]["failed"].as_u64().unwrap() > 0);
    for check in report["checks"].as_array().unwrap() {
        let prefix = match check["status"].as_str().unwrap() {
            "ok" => "成功:",
            "warn" => "警告:",
            "fail" => "错误:",
            "skip" => "跳过:",
            status => panic!("unexpected diagnostic status: {status}"),
        };
        assert!(stdout.contains(&format!("{prefix} {}", check["summary"].as_str().unwrap())));
    }
    assert_eq!(f.read("ccr/config.toml"), config_before);
    assert_eq!(f.read("claude/settings.json"), runtime_before);
    assert!(!f.0.path().join("locks").exists());
    assert!(!f.0.path().join("ccr/logs").exists());
}
