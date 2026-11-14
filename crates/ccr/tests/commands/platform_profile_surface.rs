#![allow(clippy::unwrap_used)]

use serde_json::Value;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use tempfile::TempDir;

struct PlatformProfileSurfaceFixture {
    _temp_dir: TempDir,
    home: PathBuf,
    root: PathBuf,
}

impl PlatformProfileSurfaceFixture {
    fn new() -> Self {
        let temp_dir = tempfile::tempdir().unwrap();
        let home = temp_dir.path().join("home");
        let root = home.join(".ccr");
        fs::create_dir_all(&home).unwrap();
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
        command.env("CCR_DATA_DIR", &self.root);
        command.env("CCR_CODEX_DIR", self.home.join(".codex"));
        command.env("CODEX_HOME", self.home.join(".codex"));
        command.env("CLAUDE_CONFIG_DIR", self.home.join(".claude"));
        command.env("CLAUDE_JSON_PATH", self.home.join(".claude.json"));
        command.env(
            "CCR_SETTINGS_PATH",
            self.home.join(".claude").join("settings.json"),
        );
        command.env("CCR_BACKUP_DIR", self.home.join(".backups"));
        command.env("CCR_LOCK_DIR", self.home.join(".locks"));
        command.env("GROK_HOME", self.home.join(".grok"));
        command.env("HOME", &self.home);
        command.env("USERPROFILE", &self.home);
        command.env("NO_COLOR", "1");
        command.env("CLICOLOR", "0");
        command.env("COLUMNS", "120");
        command.env("CCR_LOG_LEVEL", "off");
        command
    }

    fn run_with_stdin(&self, args: &[&str], stdin_bytes: &[u8]) -> Output {
        let mut child = self
            .command()
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(stdin_bytes).unwrap();
        child.wait_with_output().unwrap()
    }

    fn run_forced_color(&self, args: &[&str]) -> Output {
        let mut cmd = self.command();
        cmd.env_remove("NO_COLOR");
        cmd.env("CLICOLOR", "1");
        cmd.env("CLICOLOR_FORCE", "1");
        cmd.args(args).output().unwrap()
    }

    fn run_output(&self, args: &[&str]) -> Output {
        self.command().args(args).output().unwrap()
    }

    fn run_json(&self, args: &[&str]) -> (Output, Value) {
        let output = self.run_output(args);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let json = serde_json::from_str::<Value>(&stdout).unwrap();
        (output, json)
    }
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "status={:?}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn claude_and_codex_profile_crud_surfaces_accept_supported_platforms() {
    let fixture = PlatformProfileSurfaceFixture::new();

    let (claude_output, claude_json) = fixture.run_json(&[
        "claude",
        "profile",
        "create",
        "work",
        "--description",
        "Work profile",
        "--base-url",
        "https://anthropic-proxy.example.com",
        "--auth-token",
        "sk-work",
        "--model",
        "claude-sonnet-4-20250514",
        "--json",
    ]);
    assert_success(&claude_output);
    assert_eq!(claude_json["platform"], "claude");
    assert_eq!(claude_json["name"], "work");

    let (codex_output, codex_json) = fixture.run_json(&[
        "codex",
        "profile",
        "create",
        "team",
        "--provider",
        "openai",
        "--provider-type",
        "official_relay",
        "--model",
        "gpt-5",
        "--json",
    ]);
    assert_success(&codex_output);
    assert_eq!(codex_json["platform"], "codex");
    assert_eq!(codex_json["name"], "team");
}

#[test]
fn legacy_platform_profile_gemini_reports_migration_instead_of_mutating() {
    let fixture = PlatformProfileSurfaceFixture::new();

    let output = fixture.run_output(&["platform", "profile", "create", "gemini", "team"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success(), "{stderr}");
    assert!(stderr.contains("legacy"), "{stderr}");
    assert!(stderr.contains("current_platform"), "{stderr}");
    assert!(stderr.contains("ccr claude profile"), "{stderr}");
    assert!(stderr.contains("ccr codex profile"), "{stderr}");
}

fn contains_long_flag(help: &str, flag: &str) -> bool {
    help.split(|ch: char| ch.is_whitespace() || ch == ',' || ch == '[')
        .any(|token| token == flag)
}

#[test]
fn profile_create_and_delete_help_match_secret_and_active_protection_contracts() {
    let fixture = PlatformProfileSurfaceFixture::new();

    for platform in ["claude", "codex"] {
        let out = fixture.run_output(&[platform, "profile", "create", "--help"]);
        assert_success(&out);
        let help = String::from_utf8_lossy(&out.stdout);
        assert!(contains_long_flag(&help, "--auth-token-stdin"), "{help}");
        assert!(contains_long_flag(&help, "--expected-revision"), "{help}");
        assert!(contains_long_flag(&help, "--set"), "{help}");
        assert!(
            !contains_long_flag(&help, "--auth-token"),
            "{platform} create --help must hide --auth-token:\n{help}"
        );
    }

    let grok_out = fixture.run_output(&["grok", "profile", "create", "--help"]);
    assert_success(&grok_out);
    let grok_help = String::from_utf8_lossy(&grok_out.stdout);
    assert!(
        contains_long_flag(&grok_help, "--api-key-stdin"),
        "{grok_help}"
    );
    assert!(grok_help.contains("auth-token-stdin"), "{grok_help}");
    assert!(
        contains_long_flag(&grok_help, "--expected-revision"),
        "{grok_help}"
    );
    assert!(contains_long_flag(&grok_help, "--set"), "{grok_help}");
    assert!(
        !contains_long_flag(&grok_help, "--api-key"),
        "grok create --help must hide --api-key:\n{grok_help}"
    );
    assert!(
        !contains_long_flag(&grok_help, "--auth-token"),
        "grok create --help must hide --auth-token alias:\n{grok_help}"
    );

    for platform in ["claude", "codex", "grok"] {
        let del_out = fixture.run_output(&[platform, "profile", "delete", "--help"]);
        assert_success(&del_out);
        let del_help = String::from_utf8_lossy(&del_out.stdout);
        assert!(del_help.contains("--force"), "{del_help}");
        assert!(
            del_help.contains("Allow deleting the currently active profile"),
            "{del_help}"
        );
        assert!(
            !del_help.contains("disabling"),
            "{platform} delete --help leaked disable copy:\n{del_help}"
        );
    }
}

#[test]
fn profile_create_stdin_secret_channel_persists_inactive_first_profile_cas_and_exit_64() {
    let fixture = PlatformProfileSurfaceFixture::new();
    let claude_runtime = fixture.home.join(".claude").join("settings.json");
    let codex_runtime = fixture.home.join(".codex").join("config.toml");
    let grok_runtime = fixture.home.join(".grok").join("config.toml");

    // 1. Claude stdin create: inactive on first create, runtime untouched, duplicate exits 64.
    let claude_secret = "sk-claude-stdin-secret-sentinel-9182";
    let claude_out = fixture.run_with_stdin(
        &[
            "claude",
            "profile",
            "create",
            "work",
            "--base-url",
            "https://anthropic-proxy.example.com",
            "--model",
            "claude-sonnet-4-20250514",
            "--auth-token-stdin",
            "--set",
            "effort_level=HIGH",
            "--json",
        ],
        format!("{claude_secret}\r\n").as_bytes(),
    );
    assert_success(&claude_out);
    let claude_stdout = String::from_utf8_lossy(&claude_out.stdout);
    let claude_stderr = String::from_utf8_lossy(&claude_out.stderr);
    assert!(!claude_stdout.contains(claude_secret));
    assert!(!claude_stderr.contains(claude_secret));
    assert!(
        claude_stderr.is_empty(),
        "unexpected stderr: {claude_stderr}"
    );
    let claude_json: Value = serde_json::from_str(&claude_stdout).unwrap();
    assert_eq!(claude_json["ok"], true);
    assert_eq!(claude_json["platform"], "claude");
    assert_eq!(claude_json["name"], "work");
    assert_eq!(claude_json["current_profile"], Value::Null);
    assert!(
        claude_json["revision"]
            .as_str()
            .is_some_and(|r| !r.is_empty())
    );
    assert!(!claude_runtime.exists());

    let dup_out = fixture.run_with_stdin(
        &[
            "claude",
            "profile",
            "create",
            "work",
            "--base-url",
            "https://anthropic-proxy.example.com",
            "--auth-token-stdin",
        ],
        format!("{claude_secret}\n").as_bytes(),
    );
    assert_eq!(
        dup_out.status.code(),
        Some(64),
        "duplicate create must return ResourceAlreadyExists exit code 64, stderr={}",
        String::from_utf8_lossy(&dup_out.stderr)
    );

    // 2. Codex CAS mismatch before first create leaves zero profile_secrets.json.
    let codex_secrets_path = fixture
        .root
        .join("platforms")
        .join("codex")
        .join("profile_secrets.json");
    let stale_codex = fixture.run_with_stdin(
        &[
            "codex",
            "profile",
            "create",
            "stale-attempt",
            "--auth-mode",
            "openai_api_key",
            "--auth-token-stdin",
            "--expected-revision",
            "nonexistent-revision",
        ],
        b"sk-codex-stale-sentinel\n",
    );
    assert!(!stale_codex.status.success());
    assert!(!codex_secrets_path.exists());

    // Codex stdin create (openai_api_key + provider_env_key with --set env_key=RELAY_KEY).
    let codex_secret = "sk-codex-stdin-secret-sentinel-7364";
    let codex_out = fixture.run_with_stdin(
        &[
            "codex",
            "profile",
            "create",
            "api-team",
            "--auth-mode",
            "openai_api_key",
            "--model",
            "gpt-5",
            "--auth-token-stdin",
            "--expected-revision",
            "",
            "--json",
        ],
        format!("{codex_secret}\n").as_bytes(),
    );
    assert_success(&codex_out);
    let codex_json: Value =
        serde_json::from_slice(&codex_out.stdout).expect("valid codex create json");
    assert_eq!(codex_json["current_profile"], Value::Null);
    let codex_rev = codex_json["revision"].as_str().unwrap().to_string();
    assert!(!codex_rev.is_empty());
    assert!(!codex_runtime.exists());
    assert!(codex_secrets_path.is_file());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(&codex_secrets_path)
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600);
    }
    let secrets_raw = fs::read_to_string(&codex_secrets_path).unwrap();
    assert!(secrets_raw.contains(codex_secret));
    let _env = crate::setup_ccr_test_env();
    let codex_catalog = ccr_cli::managers::ConfigManager::new(
        fixture
            .root
            .join("platforms")
            .join("codex")
            .join("profiles.toml"),
    )
    .load()
    .unwrap();
    assert!(codex_catalog.current_config.is_empty());
    assert!(codex_catalog.sections["api-team"].auth_token.is_none());
    drop(_env);

    let codex_env_secret = "sk-codex-env-relay-sentinel-5511";
    let codex_env_out = fixture.run_with_stdin(
        &[
            "codex",
            "profile",
            "create",
            "env-relay",
            "--base-url",
            "https://relay.example.com/v1",
            "--auth-mode",
            "provider_env_key",
            "--set",
            "env_key=RELAY_ENV_KEY",
            "--auth-token-stdin",
            "--expected-revision",
            &codex_rev,
            "--json",
        ],
        format!("{codex_env_secret}\n").as_bytes(),
    );
    assert_success(&codex_env_out);

    // 3. Grok stdin create via both --api-key-stdin and --auth-token-stdin alias.
    let grok_secret_1 = "xai-grok-stdin-secret-sentinel-1101";
    let grok_out_1 = fixture.run_with_stdin(
        &[
            "grok",
            "profile",
            "create",
            "relay-a",
            "--base-url",
            "https://api.x.ai/v1",
            "--model",
            "grok-4",
            "--api-key-stdin",
            "--json",
        ],
        format!("{grok_secret_1}\n").as_bytes(),
    );
    assert_success(&grok_out_1);
    let grok_json_1: Value = serde_json::from_slice(&grok_out_1.stdout).unwrap();
    assert_eq!(grok_json_1["current_profile"], Value::Null);
    assert!(!grok_runtime.exists());

    let grok_secret_2 = "xai-grok-alias-secret-sentinel-2202";
    let grok_out_2 = fixture.run_with_stdin(
        &[
            "grok",
            "profile",
            "create",
            "relay-b",
            "--base-url",
            "https://api.x.ai/v1",
            "--model",
            "grok-4",
            "--auth-token-stdin",
            "--json",
        ],
        format!("{grok_secret_2}\n").as_bytes(),
    );
    assert_success(&grok_out_2);

    // Grok --set env_key=... is rejected cleanly telling the caller to use --env-key.
    let grok_set_env_err = fixture.run_output(&[
        "grok",
        "profile",
        "create",
        "relay-bad-env",
        "--base-url",
        "https://api.x.ai/v1",
        "--model",
        "grok-4",
        "--set",
        "env_key=GROK_SET_ENV_SENTINEL_7788",
    ]);
    assert!(!grok_set_env_err.status.success());
    let grok_set_env_stderr = String::from_utf8_lossy(&grok_set_env_err.stderr);
    assert!(
        grok_set_env_stderr.contains("--env-key"),
        "{grok_set_env_stderr}"
    );
    assert!(
        !grok_set_env_stderr.contains("GROK_SET_ENV_SENTINEL_7788"),
        "{grok_set_env_stderr}"
    );

    // 4. Empty / whitespace-only stdin is rejected with non-zero exit.
    for empty_input in [&b""[..], &b"   \r\n"[..]] {
        let bad = fixture.run_with_stdin(
            &[
                "claude",
                "profile",
                "create",
                "empty-secret",
                "--base-url",
                "https://anthropic-proxy.example.com",
                "--auth-token-stdin",
            ],
            empty_input,
        );
        assert!(!bad.status.success());
    }
}

#[test]
fn profile_create_plaintext_flags_warn_on_stderr_without_leaking_secret_or_breaking_json() {
    let fixture = PlatformProfileSurfaceFixture::new();

    let cases: &[(&[&str], &str, &str)] = &[
        (
            &[
                "claude",
                "profile",
                "create",
                "legacy-claude",
                "--base-url",
                "https://anthropic-proxy.example.com",
                "--auth-token",
                "SENTINEL_PLAIN_CLAUDE_991",
                "--json",
            ],
            "SENTINEL_PLAIN_CLAUDE_991",
            "--auth-token-stdin",
        ),
        (
            &[
                "codex",
                "profile",
                "create",
                "legacy-codex",
                "--auth-mode",
                "openai_api_key",
                "--auth-token",
                "SENTINEL_PLAIN_CODEX_992",
                "--json",
            ],
            "SENTINEL_PLAIN_CODEX_992",
            "--auth-token-stdin",
        ),
        (
            &[
                "grok",
                "profile",
                "create",
                "legacy-grok-api",
                "--base-url",
                "https://api.x.ai/v1",
                "--model",
                "grok-4",
                "--api-key",
                "SENTINEL_PLAIN_GROK_993",
                "--json",
            ],
            "SENTINEL_PLAIN_GROK_993",
            "--api-key-stdin",
        ),
        (
            &[
                "grok",
                "profile",
                "create",
                "legacy-grok-alias",
                "--base-url",
                "https://api.x.ai/v1",
                "--model",
                "grok-4",
                "--auth-token",
                "SENTINEL_PLAIN_GROK_994",
                "--json",
            ],
            "SENTINEL_PLAIN_GROK_994",
            "--api-key-stdin",
        ),
    ];

    for (args, sentinel, expected_hint) in cases {
        let out = fixture.run_forced_color(args);
        assert_success(&out);
        let stdout = String::from_utf8_lossy(&out.stdout);
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            !stdout.contains('\u{1b}'),
            "stdout JSON must not contain ANSI escapes: {stdout}"
        );
        assert!(!stdout.contains(sentinel), "stdout leaked secret");
        assert!(!stderr.contains(sentinel), "stderr leaked secret: {stderr}");
        assert!(
            stderr.contains(expected_hint),
            "expected deprecation hint `{expected_hint}` in stderr, got: {stderr}"
        );
        let parsed: Value = serde_json::from_str(&stdout).unwrap();
        assert_eq!(parsed["ok"], true);
        assert!(parsed["revision"].as_str().is_some_and(|r| !r.is_empty()));
    }
}

#[test]
fn profile_list_and_switch_json_emit_pure_receipts_across_claude_codex_and_grok() {
    let fixture = PlatformProfileSurfaceFixture::new();
    fs::create_dir_all(fixture.home.join(".codex")).unwrap();
    fs::write(
        fixture.home.join(".codex").join("config.toml"),
        "cli_auth_credentials_store = \"file\"\n",
    )
    .unwrap();

    // Missing catalogs report revision: null in list --json.
    for platform in ["claude", "codex", "grok"] {
        let out = fixture.run_forced_color(&[platform, "profile", "list", "--json"]);
        assert_success(&out);
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(!stdout.contains('\u{1b}'), "{stdout}");
        let list_json: Value = serde_json::from_str(&stdout).unwrap();
        assert_eq!(
            list_json["revision"],
            Value::Null,
            "{platform}: {list_json}"
        );
        assert_eq!(list_json["current_profile"], Value::Null);
    }

    // Create two profiles per platform via stdin.
    let claude_secret = "sk-switch-claude-sentinel-401";
    for name in ["alpha", "beta"] {
        assert_success(&fixture.run_with_stdin(
            &[
                "claude",
                "profile",
                "create",
                name,
                "--base-url",
                "https://anthropic-proxy.example.com",
                "--model",
                "claude-sonnet-4-20250514",
                "--auth-token-stdin",
                "--json",
            ],
            format!("{claude_secret}-{name}\n").as_bytes(),
        ));
    }

    let codex_secret = "sk-switch-codex-sentinel-402";
    for name in ["alpha", "beta"] {
        assert_success(&fixture.run_with_stdin(
            &[
                "codex",
                "profile",
                "create",
                name,
                "--auth-mode",
                "openai_api_key",
                "--model",
                "gpt-5",
                "--auth-token-stdin",
                "--json",
            ],
            format!("{codex_secret}-{name}\n").as_bytes(),
        ));
    }

    let grok_secret = "xai-switch-grok-sentinel-403";
    for name in ["alpha", "beta"] {
        assert_success(&fixture.run_with_stdin(
            &[
                "grok",
                "profile",
                "create",
                name,
                "--base-url",
                "https://api.x.ai/v1",
                "--model",
                "grok-4",
                "--api-key-stdin",
                "--json",
            ],
            format!("{grok_secret}-{name}\n").as_bytes(),
        ));
    }

    for (platform, secret_prefix) in [
        ("claude", claude_secret),
        ("codex", codex_secret),
        ("grok", grok_secret),
    ] {
        let list_before = fixture.run_forced_color(&[platform, "profile", "list", "--json"]);
        assert_success(&list_before);
        let list_before_json: Value = serde_json::from_slice(&list_before.stdout).unwrap();
        assert!(
            list_before_json["revision"]
                .as_str()
                .is_some_and(|rev| !rev.is_empty())
        );

        let first_switch =
            fixture.run_forced_color(&[platform, "profile", "switch", "alpha", "--json"]);
        assert_success(&first_switch);
        let first_stdout = String::from_utf8_lossy(&first_switch.stdout);
        let first_stderr = String::from_utf8_lossy(&first_switch.stderr);
        assert!(!first_stdout.contains('\u{1b}'), "{first_stdout}");
        assert!(!first_stdout.contains("下一步"), "{first_stdout}");
        assert!(!first_stdout.contains(secret_prefix));
        assert!(!first_stderr.contains(secret_prefix));
        assert_eq!(
            first_stdout.trim_end().lines().count(),
            1,
            "switch --json must emit a single-line receipt: {first_stdout}"
        );
        let first_json: Value = serde_json::from_str(&first_stdout).unwrap();
        assert_eq!(first_json["ok"], true);
        assert_eq!(first_json["platform"], platform);
        assert_eq!(first_json["profile"], "alpha");
        assert_eq!(first_json["previous_profile"], Value::Null);
        assert_eq!(first_json["status"], "applied");
        assert!(
            first_json["revision"]
                .as_str()
                .is_some_and(|r| !r.is_empty())
        );
        assert_eq!(first_json["warnings"], serde_json::json!([]));

        let second_switch =
            fixture.run_forced_color(&[platform, "profile", "switch", "beta", "--json"]);
        assert_success(&second_switch);
        let second_json: Value = serde_json::from_slice(&second_switch.stdout).unwrap();
        assert_eq!(second_json["ok"], true);
        assert_eq!(second_json["platform"], platform);
        assert_eq!(second_json["profile"], "beta");
        assert_eq!(second_json["previous_profile"], "alpha");
        assert_eq!(second_json["status"], "applied");

        let list_after = fixture.run_forced_color(&[platform, "profile", "list", "--json"]);
        assert_success(&list_after);
        let list_after_json: Value = serde_json::from_slice(&list_after.stdout).unwrap();
        assert_eq!(list_after_json["current_profile"], "beta");
        assert_eq!(list_after_json["revision"], second_json["revision"]);
    }

    // When switch --json fails to commit activation (e.g. corrupt runtime settings.json),
    // stdout still receives a single-line JSON receipt (ok: false) before non-zero exit.
    fs::write(
        fixture.home.join(".claude").join("settings.json"),
        "{corrupt-json",
    )
    .unwrap();
    let failed_switch =
        fixture.run_forced_color(&["claude", "profile", "switch", "alpha", "--json"]);
    assert!(!failed_switch.status.success());
    let failed_stdout = String::from_utf8_lossy(&failed_switch.stdout);
    let failed_stderr = String::from_utf8_lossy(&failed_switch.stderr);
    assert_eq!(
        failed_stdout.trim_end().lines().count(),
        1,
        "failed switch --json must still emit a single-line JSON receipt: {failed_stdout}"
    );
    assert!(!failed_stdout.contains(claude_secret));
    assert!(!failed_stderr.contains(claude_secret));
    let failed_json: Value = serde_json::from_str(&failed_stdout).unwrap();
    assert_eq!(failed_json["ok"], false);
    assert_eq!(failed_json["platform"], "claude");
    assert_eq!(failed_json["profile"], "alpha");
    assert_eq!(failed_json["status"], "unchanged");
    assert_eq!(
        failed_json["warnings"],
        serde_json::json!(["activation_failed"])
    );
    assert!(
        failed_json["revision"]
            .as_str()
            .is_some_and(|rev| !rev.is_empty())
    );
}
