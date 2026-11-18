#![allow(clippy::unwrap_used)]

use ccr_config::ConfigManager;
use serde_json::Value;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use tempfile::TempDir;

struct ProfileExchangeFixture {
    _temp_dir: TempDir,
    work_dir: PathBuf,
    home: PathBuf,
    root: PathBuf,
}

impl ProfileExchangeFixture {
    fn new() -> Self {
        let temp_dir = crate::command_tempdir().unwrap();
        let work_dir = temp_dir.path().join("work");
        let home = temp_dir.path().join("home");
        let root = home.join(".ccr");
        fs::create_dir_all(&work_dir).unwrap();
        fs::create_dir_all(&home).unwrap();
        // Intentionally do NOT pre-create `root` in `new_uninitialized`, only in `new`.
        fs::create_dir_all(&root).unwrap();

        Self {
            _temp_dir: temp_dir,
            work_dir,
            home,
            root,
        }
    }

    fn new_uninitialized() -> Self {
        let temp_dir = crate::command_tempdir().unwrap();
        let work_dir = temp_dir.path().join("work");
        let home = temp_dir.path().join("home");
        let root = home.join(".ccr");
        fs::create_dir_all(&work_dir).unwrap();
        fs::create_dir_all(&home).unwrap();

        Self {
            _temp_dir: temp_dir,
            work_dir,
            home,
            root,
        }
    }

    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_ccr"));
        command.current_dir(&self.work_dir);
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
        command
    }

    fn run_output(&self, args: &[&str]) -> Output {
        self.command().args(args).output().unwrap()
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

    fn run_json(&self, args: &[&str]) -> (Output, Value) {
        let output = self.run_output(args);
        assert_success(&output);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let json = serde_json::from_str::<Value>(&stdout)
            .unwrap_or_else(|e| panic!("expected valid JSON on stdout: {e}\nstdout:\n{stdout}"));
        (output, json)
    }

    fn run_json_with_stdin(&self, args: &[&str], stdin_bytes: &[u8]) -> (Output, Value) {
        let output = self.run_with_stdin(args, stdin_bytes);
        assert_success(&output);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let json = serde_json::from_str::<Value>(&stdout)
            .unwrap_or_else(|e| panic!("expected valid JSON on stdout: {e}\nstdout:\n{stdout}"));
        (output, json)
    }

    fn platform_dir(&self, platform: &str) -> PathBuf {
        self.root.join("platforms").join(platform)
    }

    fn profiles_db(&self, platform: &str) -> PathBuf {
        self.platform_dir(platform).join("profiles.db")
    }
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "expected exit 0, got {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_exit_code(output: &Output, expected: i32) {
    assert_eq!(
        output.status.code(),
        Some(expected),
        "expected exit {expected}, got {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Asserts the 6+1 bucket conservation invariants from Issue #173 / #174 / #190.
fn assert_import_bucket_invariants(report: &Value) {
    let counts = &report["counts"];
    let total_input = counts["total_input"].as_u64().unwrap();
    let total_incoming = counts["total_incoming"].as_u64().unwrap();
    let added = counts["added"].as_u64().unwrap();
    let updated = counts["updated"].as_u64().unwrap();
    let unchanged = counts["unchanged"].as_u64().unwrap();
    let renamed = counts["renamed"].as_u64().unwrap();
    let skipped = counts["skipped"].as_u64().unwrap();
    let skipped_conflict = counts["skipped_conflict"].as_u64().unwrap();
    let skipped_duplicate = counts["skipped_duplicate"].as_u64().unwrap();
    let removed = counts["removed"].as_u64().unwrap();
    let disabled_missing_secret = counts["disabled_missing_secret"].as_u64().unwrap();

    assert_eq!(total_input, total_incoming, "report={report}");
    assert_eq!(
        skipped,
        skipped_conflict + skipped_duplicate,
        "report={report}"
    );
    assert_eq!(
        added + updated + unchanged + renamed + skipped,
        total_input,
        "6 primary buckets must conserve total_input: {report}"
    );
    assert_eq!(
        removed as usize,
        report["removed_names"].as_array().unwrap().len(),
        "report={report}"
    );
    if report["mode"] == "merge" {
        assert_eq!(
            removed, 0,
            "merge mode must never remove profiles: {report}"
        );
    }
    assert_eq!(
        disabled_missing_secret as usize,
        report["missing_secrets"].as_array().unwrap().len(),
        "report={report}"
    );
}

#[test]
fn clap_flag_conflicts_exit_code_2_for_export_and_import() {
    let fixture = ProfileExchangeFixture::new_uninitialized();

    let conflict_cases: &[&[&str]] = &[
        &["claude", "profile", "export", "--stdout", "--json"],
        &["claude", "profile", "export", "--stdout", "-o", "out.toml"],
        &["codex", "profile", "export", "alpha", "--tag", "relay"],
        &["grok", "profile", "import"],
        &["claude", "profile", "import", "a.toml", "--stdin"],
        &[
            "claude",
            "profile",
            "import",
            "a.toml",
            "--replace",
            "--on-conflict",
            "overwrite",
        ],
        &[
            "codex",
            "profile",
            "import",
            "a.toml",
            "--replace",
            "--dedup",
        ],
    ];

    for args in conflict_cases {
        let out = fixture.run_output(args);
        assert_exit_code(&out, 2);
        assert!(
            !fixture.root.exists(),
            "clap failure created CCR_ROOT for {args:?}"
        );
    }
}

#[test]
fn ac1_readonly_export_and_dry_run_import_leave_zero_disk_side_effects() {
    // 1. Missing CCR_ROOT: neither logs/, .locks/, nor profiles.db may be created!
    let fixture = ProfileExchangeFixture::new_uninitialized();
    for platform in ["claude", "codex", "grok"] {
        let out = fixture.run_output(&[platform, "profile", "export"]);
        assert_exit_code(&out, 11); // ConfigMissing
        assert!(
            !fixture.root.exists(),
            "{platform} export created CCR_ROOT on missing catalog"
        );
        assert!(
            !fixture.home.join(".locks").exists(),
            "{platform} export created .locks"
        );
    }

    // Dry-run import on absent CCR_ROOT also leaves zero disk artifacts!
    let sample_toml = fixture.work_dir.join("sample.toml");
    fs::write(
        &sample_toml,
        r#"
schema_version = 1
platform = "claude"
exported_at = "2026-11-14T00:00:00Z"
secrets_mode = "redacted"

[profiles.official-sub]
description = "Claude Subscription"
auth_mode = "subscription"
enabled = true
"#,
    )
    .unwrap();
    let (dry_out, dry_json) = fixture.run_json(&[
        "claude",
        "profile",
        "import",
        sample_toml.to_str().unwrap(),
        "--dry-run",
        "--json",
    ]);
    assert!(dry_out.stderr.is_empty());
    assert_eq!(dry_json["dry_run"], true);
    assert_eq!(dry_json["changed"], true);
    assert_eq!(dry_json["counts"]["added"], 1);
    assert_import_bucket_invariants(&dry_json);
    assert!(
        !fixture.root.exists(),
        "import --dry-run must not create CCR_ROOT or logs/"
    );
    assert!(
        !fixture.home.join(".locks").exists(),
        "import --dry-run must not create .locks/"
    );

    // 2. Unmigrated legacy profiles.toml on disk: export must NOT migrate it or create profiles.db/logs!
    let legacy_dir = fixture.platform_dir("claude");
    fs::create_dir_all(&legacy_dir).unwrap();
    let legacy_toml = legacy_dir.join("profiles.toml");
    fs::write(
        &legacy_toml,
        r#"
default_config = "work"
current_config = "work"

[work]
base_url = "https://api.example.com"
auth_token = "sk-legacy-unmigrated"
"#,
    )
    .unwrap();
    let unmigrated_out = fixture.run_output(&[
        "claude",
        "profile",
        "export",
        "-o",
        fixture.work_dir.join("never.toml").to_str().unwrap(),
    ]);
    assert!(!unmigrated_out.status.success());
    assert!(
        legacy_toml.exists(),
        "readonly export must not retire legacy profiles.toml"
    );
    assert!(
        !fixture.profiles_db("claude").exists(),
        "readonly export must not create profiles.db"
    );
    assert!(
        !fixture.root.join("logs").exists(),
        "readonly export must not create logs/"
    );
    assert!(
        !fixture.work_dir.join("never.toml").exists(),
        "failed export must not create output file"
    );
}

#[test]
fn ac1_export_selection_stdout_purity_overwrite_gate_and_datetime_preservation() {
    let fixture = ProfileExchangeFixture::new();

    // Seed 3 Claude profiles via LegacyCatalog import:
    // - relay-a (tags = ["relay", "prod"], native TOML datetime `rotated_at`, usage_count = 19)
    // - relay-b (tags = ["dev"], inline secret)
    // - sub-official (tags = ["official"], subscription without inline secret)
    let seed_file = fixture.work_dir.join("seed-claude.toml");
    fs::write(
        &seed_file,
        r#"
default_config = "relay-a"
current_config = "relay-a"

[settings]
skip_confirmation = true

[relay-a]
description = "Primary Relay"
base_url = "https://relay-a.example.com/v1"
auth_token = "sk-relay-a-secret-1111"
model = "claude-sonnet-4-5"
tags = ["relay", "prod"]
usage_count = 19
rotated_at = 2026-05-01T12:30:00Z

[relay-b]
description = "Dev Relay"
base_url = "https://relay-b.example.com/v1"
auth_token = "sk-relay-b-secret-2222"
model = "claude-haiku-4-5"
tags = ["dev"]

[sub-official]
description = "Official Claude Subscription"
auth_mode = "subscription"
tags = ["official"]
"#,
    )
    .unwrap();
    let (_, import_json) = fixture.run_json(&[
        "claude",
        "profile",
        "import",
        seed_file.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(import_json["source_format"], "legacy_catalog");
    assert_eq!(import_json["counts"]["added"], 3);

    // Default export with no -o writes `./ccr-claude-profiles-<timestamp>.toml` in work_dir.
    let default_out = fixture.run_output(&["claude", "profile", "export"]);
    assert_success(&default_out);
    let generated: Vec<PathBuf> = fs::read_dir(&fixture.work_dir)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("ccr-claude-profiles-") && n.ends_with(".toml"))
        })
        .collect();
    assert_eq!(generated.len(), 1);
    let default_toml = fs::read_to_string(&generated[0]).unwrap();
    assert!(default_toml.contains("schema_version = 1"));
    assert!(default_toml.contains("platform = \"claude\""));
    assert!(default_toml.contains("secrets_mode = \"included\""));
    assert!(!default_toml.contains("current_config"));
    assert!(!default_toml.contains("default_config"));
    assert!(!default_toml.contains("skip_confirmation"));
    assert!(!default_toml.contains("usage_count"));
    // Native TOML Datetime (unquoted literal) must survive round-trip!
    assert!(
        default_toml.contains("rotated_at = 2026-05-01T12:30:00Z"),
        "expected native TOML datetime in:\n{default_toml}"
    );

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(&generated[0]).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }

    // Positional [NAME]... preserves requested order and deduplicates repeated names.
    let stdout_named = fixture.run_output(&[
        "claude",
        "profile",
        "export",
        "sub-official",
        "relay-a",
        "sub-official",
        "--stdout",
    ]);
    assert_success(&stdout_named);
    let stdout_toml = String::from_utf8_lossy(&stdout_named.stdout);
    let stderr_warn = String::from_utf8_lossy(&stdout_named.stderr);
    // stdout is 100% pure parseable TOML; warning went strictly to stderr!
    let parsed_val: toml::Value = toml::from_str(&stdout_toml)
        .unwrap_or_else(|e| panic!("--stdout must be pure TOML: {e}\n{stdout_toml}"));
    assert_eq!(parsed_val["schema_version"].as_integer(), Some(1));
    let sub_pos = stdout_toml.find("[profiles.sub-official]").unwrap();
    let relay_pos = stdout_toml.find("[profiles.relay-a]").unwrap();
    assert!(
        sub_pos < relay_pos,
        "requested NAME order must be preserved:\n{stdout_toml}"
    );
    assert!(!stdout_toml.contains("[profiles.relay-b]"));
    assert!(
        stderr_warn.contains("明文内联密钥"),
        "expected stderr warning when inline secret is exported: {stderr_warn}"
    );

    // Exporting ONLY secret-free `sub-official` emits ZERO plaintext-secret warning!
    let stdout_secret_free =
        fixture.run_output(&["claude", "profile", "export", "sub-official", "--stdout"]);
    assert_success(&stdout_secret_free);
    assert!(
        stdout_secret_free.stderr.is_empty(),
        "secret-free export must not emit false-positive secret warning: {}",
        String::from_utf8_lossy(&stdout_secret_free.stderr)
    );

    // Missing NAME returns exit 62 (ProfileNotFound) and writes no output file.
    let missing_target = fixture.work_dir.join("missing-out.toml");
    let missing_out = fixture.run_output(&[
        "claude",
        "profile",
        "export",
        "relay-a",
        "does-not-exist",
        "-o",
        missing_target.to_str().unwrap(),
    ]);
    assert_exit_code(&missing_out, 62);
    assert!(!missing_target.exists());

    // --tag OR union selection & 0-match exit 90.
    let tag_target = fixture.work_dir.join("tagged.toml");
    let (_, tag_json) = fixture.run_json(&[
        "claude",
        "profile",
        "export",
        "--tag",
        "prod",
        "--tag",
        "official",
        "-o",
        tag_target.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(tag_json["exported_count"], 2);
    assert_eq!(
        tag_json["exported_names"],
        serde_json::json!(["relay-a", "sub-official"])
    );
    assert_eq!(tag_json["contains_secrets"], true);
    let tag_json_text = serde_json::to_string(&tag_json).unwrap();
    assert!(!tag_json_text.contains("sk-relay-a-secret-1111"));

    let zero_tag_target = fixture.work_dir.join("zero-tag.toml");
    let zero_tag_out = fixture.run_output(&[
        "claude",
        "profile",
        "export",
        "--tag",
        "no-such-tag",
        "-o",
        zero_tag_target.to_str().unwrap(),
    ]);
    assert_exit_code(&zero_tag_out, 90);
    assert!(!zero_tag_target.exists());

    // Overwrite gate: existing file without -y returns exit 64 in non-TTY / --json, leaving file intact!
    let before_bytes = fs::read(&tag_target).unwrap();
    let overwrite_blocked = fixture.run_output(&[
        "claude",
        "profile",
        "export",
        "sub-official",
        "-o",
        tag_target.to_str().unwrap(),
        "--json",
    ]);
    assert_exit_code(&overwrite_blocked, 64);
    assert!(overwrite_blocked.stdout.is_empty());
    assert_eq!(fs::read(&tag_target).unwrap(), before_bytes);

    // Global -y (both prefix `ccr -y ...` and suffix `... --yes`) atomically overwrites!
    let (_, overwrite_ok_json) = fixture.run_json(&[
        "-y",
        "claude",
        "profile",
        "export",
        "sub-official",
        "-o",
        tag_target.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(overwrite_ok_json["exported_count"], 1);
    assert_eq!(overwrite_ok_json["contains_secrets"], false);

    // Directory target fails with exit 51 (FileIoError).
    let dir_target = fixture.work_dir.join("a-dir");
    fs::create_dir_all(&dir_target).unwrap();
    let dir_err = fixture.run_output(&[
        "-y",
        "claude",
        "profile",
        "export",
        "-o",
        dir_target.to_str().unwrap(),
    ]);
    assert_exit_code(&dir_err, 51);
}

#[test]
fn ac2_codex_secret_sidecar_roundtrip_and_three_platform_no_secrets_redaction() {
    let source = ProfileExchangeFixture::new();

    // 1. Create Codex profiles via official `profile create` so secrets live in `profile_secrets.json`
    //    while `profiles.db` has `auth_token = None`.
    let codex_api_secret = "sk-codex-live-secret-889900";
    let create_api = source.run_with_stdin(
        &[
            "codex",
            "profile",
            "create",
            "openai-direct",
            "--model",
            "gpt-5",
            "--auth-mode",
            "openai_api_key",
            "--auth-token-stdin",
            "--json",
        ],
        format!("{codex_api_secret}\n").as_bytes(),
    );
    assert_success(&create_api);

    let codex_env_secret = "sk-or-v1-codex-relay-776655";
    let create_relay = source.run_with_stdin(
        &[
            "codex",
            "profile",
            "create",
            "openrouter-codex",
            "--base-url",
            "https://user:pw@openrouter.ai/api/v1?api-version=2025-01&api_key=url-leak#frag",
            "--model",
            "gpt-5-mini",
            "--auth-mode",
            "provider_env_key",
            "--set",
            "env_key=OPENROUTER_API_KEY",
            "--auth-token-stdin",
            "--json",
        ],
        format!("{codex_env_secret}\n").as_bytes(),
    );
    assert_success(&create_relay);

    let create_chatgpt = source.run_output(&[
        "codex",
        "profile",
        "create",
        "chatgpt-sub",
        "--model",
        "gpt-5",
        "--auth-mode",
        "openai_chatgpt",
        "--json",
    ]);
    assert_success(&create_chatgpt);

    // Confirm `profiles.db` canonical TOML does NOT store `codex_api_secret` inline.
    let src_codex_mgr = ConfigManager::new(source.platform_dir("codex").join("profiles.toml"));
    let (canonical_db_toml, _) = src_codex_mgr.canonical_toml().unwrap().unwrap();
    assert!(!canonical_db_toml.contains(codex_api_secret));

    // Export Codex with secrets (`secrets_mode = "included"`): overlays `profile_secrets.json`!
    let codex_bundle = source.work_dir.join("codex-included.toml");
    let (_, export_codex_json) = source.run_json(&[
        "codex",
        "profile",
        "export",
        "-o",
        codex_bundle.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(export_codex_json["secrets_mode"], "included");
    assert_eq!(export_codex_json["contains_secrets"], true);
    let codex_bundle_text = fs::read_to_string(&codex_bundle).unwrap();
    assert!(codex_bundle_text.contains(codex_api_secret));
    assert!(codex_bundle_text.contains(codex_env_secret));

    // Import into a brand-new isolated target machine (`dest`) and switch to `openai-direct`!
    let dest = ProfileExchangeFixture::new();
    let (_, import_codex_json) = dest.run_json(&[
        "codex",
        "profile",
        "import",
        codex_bundle.to_str().unwrap(),
        "--json",
    ]);
    assert_import_bucket_invariants(&import_codex_json);
    assert_eq!(import_codex_json["counts"]["added"], 3);
    assert_eq!(import_codex_json["counts"]["disabled_missing_secret"], 0);

    // Verify target `profile_secrets.json` received the secrets and target `profiles.db` kept `auth_token` out!
    let dest_secrets_path = dest.platform_dir("codex").join("profile_secrets.json");
    let dest_secrets_raw = fs::read_to_string(&dest_secrets_path).unwrap();
    assert!(dest_secrets_raw.contains(codex_api_secret));
    assert!(dest_secrets_raw.contains(codex_env_secret));
    let dest_codex_mgr = ConfigManager::new(dest.platform_dir("codex").join("profiles.toml"));
    let (dest_db_toml, _) = dest_codex_mgr.canonical_toml().unwrap().unwrap();
    assert!(!dest_db_toml.contains(codex_api_secret));

    // Switching to `openai-direct` on `dest` works immediately and writes Codex runtime `auth.json`!
    let switch_codex = dest.run_output(&["codex", "profile", "switch", "openai-direct", "--json"]);
    assert_success(&switch_codex);
    let codex_auth_json = fs::read_to_string(dest.home.join(".codex").join("auth.json")).unwrap();
    assert!(codex_auth_json.contains(codex_api_secret));

    // 2. Three-platform `--no-secrets` redaction + `auth_mode` materialization:
    // Seed Claude profile WITHOUT explicit `auth_mode` (inferred `api_key` from base_url + auth_token)
    // plus URL credentials, query secret, non-secret query `?api-version=2024-10-21`, safe `max_tokens`, and extension secret!
    let claude_seed = source.work_dir.join("claude-dirty.toml");
    let claude_secret = "sk-ant-inline-secret-554433";
    let ext_bearer = "Bearer sk-ext-header-secret-998877";
    fs::write(
        &claude_seed,
        format!(
            r#"
schema_version = 1
platform = "claude"

[profiles.inferred-relay]
description = "Relay without explicit auth_mode"
base_url = "https://alice:hunter2@proxy.example.com/v1?api-version=2024-10-21&api_key=querysecret99&mode=fast#frag"
auth_token = "{claude_secret}"
model = "claude-sonnet-4-5"
max_tokens = 8192

[profiles.inferred-relay.headers]
Authorization = "{ext_bearer}"
X-Client-Name = "ccr-team"
"#
        ),
    )
    .unwrap();
    let (_, _) = source.run_json(&[
        "claude",
        "profile",
        "import",
        claude_seed.to_str().unwrap(),
        "--json",
    ]);

    let claude_redacted = source.work_dir.join("claude-redacted.toml");
    let (_, claude_redacted_json) = source.run_json(&[
        "claude",
        "profile",
        "export",
        "--no-secrets",
        "-o",
        claude_redacted.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(claude_redacted_json["secrets_mode"], "redacted");
    assert_eq!(claude_redacted_json["contains_secrets"], false);
    let claude_redacted_toml = fs::read_to_string(&claude_redacted).unwrap();
    assert!(!claude_redacted_toml.contains(claude_secret));
    assert!(!claude_redacted_toml.contains("hunter2"));
    assert!(!claude_redacted_toml.contains("querysecret99"));
    assert!(!claude_redacted_toml.contains(ext_bearer));
    assert!(!claude_redacted_toml.contains("***"));
    assert!(!claude_redacted_toml.contains("..."));
    // Materialized `auth_mode = "api_key"`, clean `base_url` with non-secret query params, and safe extensions survive!
    assert!(
        claude_redacted_toml.contains("auth_mode = \"api_key\""),
        "expected materialized auth_mode in:\n{claude_redacted_toml}"
    );
    assert!(
        claude_redacted_toml.contains(
            "base_url = \"https://proxy.example.com/v1?api-version=2024-10-21&mode=fast\""
        ),
        "expected scrubbed base_url preserving non-secret params:\n{claude_redacted_toml}"
    );
    assert!(claude_redacted_toml.contains("max_tokens = 8192"));
    assert!(claude_redacted_toml.contains("X-Client-Name = \"ccr-team\""));
    let warnings_str = serde_json::to_string(&claude_redacted_json["warnings"]).unwrap();
    assert!(warnings_str.contains("profiles.inferred-relay.base_url"));
    assert!(warnings_str.contains("profiles.inferred-relay.headers.Authorization"));
    assert!(!warnings_str.contains("querysecret99"));
    assert!(!warnings_str.contains("sk-ext-header-secret-998877"));

    // Codex `--no-secrets` preserves `auth_mode` & `env_key` and deletes `auth_token`.
    let codex_redacted = source.work_dir.join("codex-redacted.toml");
    let (_, _) = source.run_json(&[
        "codex",
        "profile",
        "export",
        "--no-secrets",
        "-o",
        codex_redacted.to_str().unwrap(),
        "--json",
    ]);
    let codex_redacted_toml = fs::read_to_string(&codex_redacted).unwrap();
    assert!(!codex_redacted_toml.contains(codex_api_secret));
    assert!(!codex_redacted_toml.contains(codex_env_secret));
    assert!(!codex_redacted_toml.contains("url-leak"));
    assert!(codex_redacted_toml.contains("env_key = \"OPENROUTER_API_KEY\""));
    assert!(codex_redacted_toml.contains("auth_mode = \"provider_env_key\""));

    // Grok `--no-secrets` deletes `api_key` and preserves `env_key`.
    let grok_secret = "xai-grok-secret-token-332211";
    let create_grok = source.run_with_stdin(
        &[
            "grok",
            "profile",
            "create",
            "xai-fast",
            "--base-url",
            "https://api.x.ai/v1",
            "--model",
            "grok-4-fast",
            "--api-key-stdin",
            "--json",
        ],
        format!("{grok_secret}\n").as_bytes(),
    );
    assert_success(&create_grok);
    let grok_redacted = source.work_dir.join("grok-redacted.toml");
    let (_, _) = source.run_json(&[
        "grok",
        "profile",
        "export",
        "--no-secrets",
        "-o",
        grok_redacted.to_str().unwrap(),
        "--json",
    ]);
    let grok_redacted_toml = fs::read_to_string(&grok_redacted).unwrap();
    assert!(!grok_redacted_toml.contains(grok_secret));
    assert!(!grok_redacted_toml.contains("api_key"));
    assert!(grok_redacted_toml.contains("model = \"grok-4-fast\""));
}

#[test]
fn ac3_masked_secret_rejection_safe_toml_error_missing_secret_lifecycle_and_local_preservation() {
    let fixture = ProfileExchangeFixture::new();

    // 1. Safe TOML syntax error: exit 14, reports line/col ONLY, NEVER echoes unclosed secret literal!
    let unclosed_secret = "sk-unclosed-ultra-sensitive-token-918273";
    let broken_toml = format!(
        "schema_version = 1\nplatform = \"claude\"\n\n[profiles.bad]\nauth_token = \"{unclosed_secret}\n"
    );
    let broken_out = fixture.run_with_stdin(
        &["claude", "profile", "import", "--stdin", "--json"],
        broken_toml.as_bytes(),
    );
    assert_exit_code(&broken_out, 14);
    assert!(broken_out.stdout.is_empty());
    let broken_stderr = String::from_utf8_lossy(&broken_out.stderr);
    assert!(
        !broken_stderr.contains(unclosed_secret),
        "TOML syntax error leaked source line secret:\n{broken_stderr}"
    );
    assert!(broken_stderr.contains("行") && broken_stderr.contains("列"));

    // 2. Legacy masked tokens & header/body mismatch rejected across all 3 formats with exit 90!
    let masked_payloads: &[(&str, &str)] = &[
        (
            "claude",
            "schema_version = 1\nplatform = \"claude\"\n[profiles.p1]\nbase_url = \"https://x.example.com\"\nauth_token = \"sk-ant-...1234\"\n",
        ),
        (
            "claude",
            "default_config = \"p1\"\ncurrent_config = \"p1\"\n[p1]\nbase_url = \"https://x.example.com\"\nauth_token = \"***\"\n",
        ),
        (
            "grok",
            "[p1]\nbase_url = \"https://api.x.ai/v1\"\nmodel = \"grok-4\"\napi_key = \"REDACTED\"\n",
        ),
        (
            "claude",
            "schema_version = 1\nplatform = \"claude\"\nsecrets_mode = \"redacted\"\n[profiles.p1]\nbase_url = \"https://x.example.com\"\nauth_token = \"sk-real-token-in-redacted-file\"\n",
        ),
    ];

    for (platform, body) in masked_payloads {
        let out = fixture.run_with_stdin(&[platform, "profile", "import", "-"], body.as_bytes());
        assert_exit_code(&out, 90);
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(!stderr.contains("sk-real-token-in-redacted-file"));
        assert!(!fixture.profiles_db(platform).exists());
    }

    // 3. Importing redacted template into empty catalog:
    //    - credential-requiring `relay-pending` enters with `enabled = false` (`missing_secrets`)
    //    - secret-free `official-sub` stays `enabled = true`
    let redacted_claude = r#"
schema_version = 1
platform = "claude"
exported_at = "2026-11-14T00:00:00Z"
secrets_mode = "redacted"

[profiles.relay-pending]
description = "Needs API key"
base_url = "https://relay.example.com/v1"
auth_mode = "api_key"
model = "claude-sonnet-4-5"
enabled = true

[profiles.official-sub]
description = "Claude Official"
auth_mode = "subscription"
enabled = true
"#;
    let (_, import_report) = fixture.run_json_with_stdin(
        &["claude", "profile", "import", "--stdin", "--json"],
        redacted_claude.as_bytes(),
    );
    assert_import_bucket_invariants(&import_report);
    assert_eq!(import_report["counts"]["added"], 2);
    assert_eq!(import_report["counts"]["disabled_missing_secret"], 1);
    assert_eq!(
        import_report["missing_secrets"],
        serde_json::json!(["relay-pending"])
    );

    // Switching or enabling `relay-pending` before supplying its secret fails with exit 90!
    let switch_fail = fixture.run_output(&["claude", "profile", "switch", "relay-pending"]);
    assert_exit_code(&switch_fail, 90);

    let enable_fail = fixture.run_output(&["claude", "profile", "enable", "relay-pending"]);
    assert_exit_code(&enable_fail, 90);
    let enable_stderr = String::from_utf8_lossy(&enable_fail.stderr);
    assert!(
        enable_stderr.contains("--auth-token-stdin"),
        "expected actionable edit guidance on enable failure:\n{enable_stderr}"
    );

    // `ccr validate --json` treats disabled credential-pending profile as Warning (exit 0), NOT Error!
    let (val_out, val_json) = fixture.run_json(&["validate", "--json"]);
    assert_success(&val_out);
    let checks = val_json["checks"].as_array().unwrap();
    assert!(
        checks.iter().all(|c| c["severity"] != "error"),
        "expected zero error checks in validate report: {val_json}"
    );
    assert!(
        checks.iter().any(|c| {
            c["platform"] == "claude"
                && c["target"] == "relay-pending"
                && c["category"] == "disabled"
                && c["severity"] == "warning"
        }),
        "expected disabled warning for relay-pending in validate report: {val_json}"
    );

    // Supplying secret via `profile edit ... --auth-token-stdin --enable` activates readiness!
    let supplied_secret = "sk-ant-freshly-supplied-654321";
    let edit_ok = fixture.run_with_stdin(
        &[
            "claude",
            "profile",
            "edit",
            "relay-pending",
            "--auth-token-stdin",
            "--enable",
            "--json",
        ],
        format!("{supplied_secret}\n").as_bytes(),
    );
    assert_success(&edit_ok);

    // Re-importing updated redacted template with `--on-conflict overwrite` preserves local `supplied_secret`!
    let updated_redacted = redacted_claude.replace("claude-sonnet-4-5", "claude-opus-4-1");
    let (_, overwrite_report) = fixture.run_json_with_stdin(
        &[
            "claude",
            "profile",
            "import",
            "-",
            "--on-conflict",
            "overwrite",
            "--json",
        ],
        updated_redacted.as_bytes(),
    );
    assert_import_bucket_invariants(&overwrite_report);
    assert_eq!(overwrite_report["counts"]["updated"], 1);
    assert_eq!(overwrite_report["counts"]["unchanged"], 1);
    assert_eq!(
        overwrite_report["preserved_local_secrets"],
        serde_json::json!(["relay-pending"])
    );
    assert!(
        overwrite_report["missing_secrets"]
            .as_array()
            .unwrap()
            .is_empty()
    );

    // Switching to `relay-pending` succeeds and writes `supplied_secret` + `claude-opus-4-1`!
    let switch_ok = fixture.run_output(&["claude", "profile", "switch", "relay-pending", "--json"]);
    assert_success(&switch_ok);
    let claude_settings =
        fs::read_to_string(fixture.home.join(".claude").join("settings.json")).unwrap();
    assert!(claude_settings.contains(supplied_secret));
    assert!(claude_settings.contains("claude-opus-4-1"));

    // `--on-conflict rename` on another redacted model delta creates `relay-pending-imported` WITHOUT stealing local secret!
    let rename_redacted = redacted_claude.replace("claude-sonnet-4-5", "claude-haiku-4-5");
    let (_, rename_report) = fixture.run_json_with_stdin(
        &[
            "claude",
            "profile",
            "import",
            "--stdin",
            "--on-conflict",
            "rename",
            "--json",
        ],
        rename_redacted.as_bytes(),
    );
    assert_import_bucket_invariants(&rename_report);
    assert_eq!(rename_report["counts"]["renamed"], 1);
    assert_eq!(
        rename_report["renamed_items"],
        serde_json::json!([{ "from": "relay-pending", "to": "relay-pending-imported" }])
    );
    assert_eq!(
        rename_report["missing_secrets"],
        serde_json::json!(["relay-pending-imported"])
    );
}

#[test]
fn ac4_formats_conflict_rename_dedup_replace_gates_idempotence_and_cas() {
    let fixture = ProfileExchangeFixture::new();

    // Cross-platform ExchangeV1 rejected with exit 90!
    let wrong_platform = r#"
schema_version = 1
platform = "grok"

[profiles.xai]
model = "grok-4"
"#;
    let cross_err = fixture.run_with_stdin(
        &["claude", "profile", "import", "--stdin"],
        wrong_platform.as_bytes(),
    );
    assert_exit_code(&cross_err, 90);

    // Unsupported schema_version rejected with exit 14!
    let future_schema =
        "schema_version = 99\nplatform = \"codex\"\n[profiles.a]\nmodel = \"gpt-5\"\n";
    let schema_err = fixture.run_with_stdin(
        &["codex", "profile", "import", "--stdin"],
        future_schema.as_bytes(),
    );
    assert_exit_code(&schema_err, 14);

    // Missing input file returns exit 51!
    let missing_file_err = fixture.run_output(&[
        "codex",
        "profile",
        "import",
        fixture.work_dir.join("nonexistent.toml").to_str().unwrap(),
    ]);
    assert_exit_code(&missing_file_err, 51);

    // CAS `--expected-revision ""` succeeds on absent catalog, then stale token fails with exit 90!
    let simplified_seed = r#"
[alpha]
description = "Alpha ChatGPT"
model = "gpt-5"
auth_mode = "openai_chatgpt"

[alpha-imported]
description = "Pre-existing imported suffix"
model = "gpt-5-mini"
auth_mode = "openai_chatgpt"
"#;
    let (_, init_report) = fixture.run_json_with_stdin(
        &[
            "codex",
            "profile",
            "import",
            "--stdin",
            "--expected-revision",
            "",
            "--json",
        ],
        simplified_seed.as_bytes(),
    );
    assert_import_bucket_invariants(&init_report);
    assert_eq!(init_report["source_format"], "legacy_simplified");
    assert_eq!(init_report["base_revision"], "");
    let rev1 = init_report["revision"].as_str().unwrap().to_string();
    assert!(!rev1.is_empty());

    // Passing `--expected-revision ""` now that catalog exists must fail with exit 90!
    let stale_cas = fixture.run_with_stdin(
        &[
            "codex",
            "profile",
            "import",
            "--stdin",
            "--expected-revision",
            "",
            "--json",
        ],
        simplified_seed.as_bytes(),
    );
    assert_exit_code(&stale_cas, 90);
    assert!(stale_cas.stdout.is_empty());

    // Idempotent re-import: `changed == false`, `backup_path == null`, `revision == rev1`, zero backup files!
    let (_, idem_report) = fixture.run_json_with_stdin(
        &[
            "codex",
            "profile",
            "import",
            "--stdin",
            "--expected-revision",
            &rev1,
            "--json",
        ],
        simplified_seed.as_bytes(),
    );
    assert_import_bucket_invariants(&idem_report);
    assert_eq!(idem_report["changed"], false);
    assert_eq!(idem_report["backup_path"], Value::Null);
    assert_eq!(idem_report["revision"], rev1);
    assert_eq!(idem_report["counts"]["unchanged"], 2);

    // `--on-conflict rename` + `--dedup`:
    // - `alpha` has different model -> `alpha` & `alpha-imported` already exist -> becomes `alpha-imported-2`!
    // - `dup-local` matches existing `alpha` spec -> skipped_duplicate (`duplicate_of: "alpha"`)
    // - `zeta-first` and `beta-second` are identical new entries in file order (`zeta-first` before `beta-second`):
    //   `zeta-first` must be added and `beta-second` must be `duplicate_of: "zeta-first"` (proving exact file order, NOT BTreeMap alphabetical order!)
    let rename_dedup_batch = r#"
schema_version = 1
platform = "codex"

[profiles.alpha]
description = "Modified Alpha"
model = "o3"
auth_mode = "openai_chatgpt"

[profiles.dup-local]
description = "Alpha ChatGPT"
model = "gpt-5"
auth_mode = "openai_chatgpt"

[profiles.zeta-first]
description = "Shared New Spec"
model = "o4-mini"
auth_mode = "openai_chatgpt"

[profiles.beta-second]
description = "Shared New Spec"
model = "o4-mini"
auth_mode = "openai_chatgpt"
"#;
    let (_, rd_report) = fixture.run_json_with_stdin(
        &[
            "codex",
            "profile",
            "import",
            "--stdin",
            "--on-conflict",
            "rename",
            "--dedup",
            "--json",
        ],
        rename_dedup_batch.as_bytes(),
    );
    assert_import_bucket_invariants(&rd_report);
    assert_eq!(rd_report["counts"]["total_input"], 4);
    assert_eq!(rd_report["counts"]["added"], 1);
    assert_eq!(rd_report["counts"]["renamed"], 1);
    assert_eq!(rd_report["counts"]["skipped_duplicate"], 2);
    assert_eq!(rd_report["added_names"], serde_json::json!(["zeta-first"]));
    assert_eq!(
        rd_report["renamed_items"],
        serde_json::json!([{ "from": "alpha", "to": "alpha-imported-2" }])
    );
    assert_eq!(
        rd_report["skipped_duplicates"],
        serde_json::json!([
            { "name": "dup-local", "duplicate_of": "alpha" },
            { "name": "beta-second", "duplicate_of": "zeta-first" }
        ])
    );
    let backup_path = PathBuf::from(rd_report["backup_path"].as_str().unwrap());
    assert!(backup_path.exists());

    // Also add a secret-bearing Codex profile `temp-secret` so we can verify `--replace` removes its entry from `profile_secrets.json`!
    let temp_secret_val = "sk-codex-temp-to-be-removed-999";
    let add_temp = fixture.run_with_stdin(
        &[
            "codex",
            "profile",
            "create",
            "temp-secret",
            "--model",
            "gpt-5",
            "--auth-mode",
            "openai_api_key",
            "--auth-token-stdin",
            "--json",
        ],
        format!("{temp_secret_val}\n").as_bytes(),
    );
    assert_success(&add_temp);
    let secrets_file = fixture.platform_dir("codex").join("profile_secrets.json");
    assert!(
        fs::read_to_string(&secrets_file)
            .unwrap()
            .contains(temp_secret_val)
    );

    // `--replace` confirmation gate:
    // Even if input file is a LegacyCatalog carrying `[settings] skip_confirmation = true`,
    // running `--replace` without `-y` via file (non-TTY / `--json`) or `--stdin` MUST fail with exit 90!
    let replace_file = fixture.work_dir.join("replace-codex.toml");
    fs::write(
        &replace_file,
        r#"
default_config = "alpha"
current_config = "alpha"

[settings]
skip_confirmation = true

[alpha]
description = "Clean Baseline Alpha"
model = "gpt-5"
auth_mode = "openai_chatgpt"
"#,
    )
    .unwrap();

    let replace_no_yes_file = fixture.run_output(&[
        "codex",
        "profile",
        "import",
        replace_file.to_str().unwrap(),
        "--replace",
        "--json",
    ]);
    assert_exit_code(&replace_no_yes_file, 90);
    assert!(replace_no_yes_file.stdout.is_empty());

    let replace_no_yes_stdin = fixture.run_with_stdin(
        &["codex", "profile", "import", "-", "--replace"],
        fs::read(&replace_file).unwrap().as_slice(),
    );
    assert_exit_code(&replace_no_yes_stdin, 90);

    // With `--yes`, `--replace` commits atomically, backs up prior catalog, and purges removed `temp-secret` from `profile_secrets.json`!
    let (_, replace_ok) = fixture.run_json(&[
        "codex",
        "profile",
        "import",
        replace_file.to_str().unwrap(),
        "--replace",
        "--yes",
        "--json",
    ]);
    assert_import_bucket_invariants(&replace_ok);
    assert_eq!(replace_ok["mode"], "replace");
    assert_eq!(replace_ok["on_conflict"], Value::Null);
    assert_eq!(replace_ok["counts"]["removed"], 4);
    assert!(
        replace_ok["removed_names"]
            .as_array()
            .unwrap()
            .contains(&Value::String("temp-secret".into()))
    );
    assert!(
        !secrets_file.exists()
            || !fs::read_to_string(&secrets_file)
                .unwrap()
                .contains(temp_secret_val),
        "--replace must purge removed profile secrets from profile_secrets.json"
    );
}

#[test]
fn ac5_active_runtime_reapply_no_apply_usage_count_and_grok_state_protection() {
    let fixture = ProfileExchangeFixture::new();

    // 1. Claude active profile: switch once (`usage_count` becomes 1), overwrite re-applies runtime WITHOUT incrementing `usage_count`,
    //    `--no-apply` skips runtime write, and active disable / `--replace` removal fails with exit 90!
    let claude_secret = "sk-claude-active-secret-4242";
    let create_claude = fixture.run_with_stdin(
        &[
            "claude",
            "profile",
            "create",
            "active-claude",
            "--base-url",
            "https://relay.example.com",
            "--model",
            "claude-sonnet-4-20250514",
            "--auth-token-stdin",
            "--json",
        ],
        format!("{claude_secret}\n").as_bytes(),
    );
    assert_success(&create_claude);
    assert_success(&fixture.run_output(&[
        "claude",
        "profile",
        "switch",
        "active-claude",
        "--json",
    ]));

    let claude_mgr = ConfigManager::new(fixture.platform_dir("claude").join("profiles.toml"));
    assert_eq!(
        claude_mgr.load().unwrap().sections["active-claude"].usage_count,
        Some(1)
    );

    // Overwrite active-claude (redacted model update -> preserves secret + re-applies runtime + keeps usage_count == 1)
    let overwrite_active_v2 = r#"
schema_version = 1
platform = "claude"
secrets_mode = "redacted"

[profiles.active-claude]
base_url = "https://relay.example.com"
auth_mode = "api_key"
model = "claude-opus-4-1"
"#;
    let (_, reapply_report) = fixture.run_json_with_stdin(
        &[
            "claude",
            "profile",
            "import",
            "--stdin",
            "--on-conflict",
            "overwrite",
            "--json",
        ],
        overwrite_active_v2.as_bytes(),
    );
    assert_import_bucket_invariants(&reapply_report);
    assert_eq!(reapply_report["reapplied_profile"], "active-claude");
    let settings_path = fixture.home.join(".claude").join("settings.json");
    assert!(
        fs::read_to_string(&settings_path)
            .unwrap()
            .contains("claude-opus-4-1")
    );
    assert_eq!(
        claude_mgr.load().unwrap().sections["active-claude"].usage_count,
        Some(1),
        "import re-apply must NOT increment usage_count"
    );

    // `--no-apply` updates catalog to `claude-haiku-4-5` while keeping runtime on `claude-opus-4-1`!
    let overwrite_active_v3 = overwrite_active_v2.replace("claude-opus-4-1", "claude-haiku-4-5");
    let (_, no_apply_report) = fixture.run_json_with_stdin(
        &[
            "claude",
            "profile",
            "import",
            "--stdin",
            "--on-conflict",
            "overwrite",
            "--no-apply",
            "--json",
        ],
        overwrite_active_v3.as_bytes(),
    );
    assert_eq!(no_apply_report["reapplied_profile"], Value::Null);
    assert_eq!(
        claude_mgr.load().unwrap().sections["active-claude"]
            .model
            .as_deref(),
        Some("claude-haiku-4-5")
    );
    assert!(
        fs::read_to_string(&settings_path)
            .unwrap()
            .contains("claude-opus-4-1"),
        "--no-apply must leave runtime settings.json untouched"
    );

    // Disabling active profile via import fails with exit 90!
    let disable_active = format!("{overwrite_active_v3}enabled = false\n");
    let disable_err = fixture.run_with_stdin(
        &[
            "claude",
            "profile",
            "import",
            "--stdin",
            "--on-conflict",
            "overwrite",
        ],
        disable_active.as_bytes(),
    );
    assert_exit_code(&disable_err, 90);

    // `--replace` removing active profile fails with exit 90!
    let without_active = r#"
schema_version = 1
platform = "claude"
secrets_mode = "redacted"

[profiles.other-sub]
auth_mode = "subscription"
"#;
    let remove_active_err = fixture.run_with_stdin(
        &[
            "claude",
            "profile",
            "import",
            "--stdin",
            "--replace",
            "--yes",
        ],
        without_active.as_bytes(),
    );
    assert_exit_code(&remove_active_err, 90);

    // 2. Grok active runtime matrix: `Active`, `Drifted`, and `UnsafeMissingEntryState`!
    let grok_key = "xai-active-grok-key-777";
    assert_success(&fixture.run_with_stdin(
        &[
            "grok",
            "profile",
            "create",
            "grok-live",
            "--base-url",
            "https://api.x.ai/v1",
            "--model",
            "grok-4-fast",
            "--api-key-stdin",
            "--json",
        ],
        format!("{grok_key}\n").as_bytes(),
    ));
    assert_success(&fixture.run_output(&["grok", "profile", "switch", "grok-live", "--json"]));

    // Mutate `~/.grok/config.toml` directly so Grok enters `Drifted { name: "grok-live" }`.
    let grok_runtime = fixture.home.join(".grok").join("config.toml");
    let drifted = fs::read_to_string(&grok_runtime)
        .unwrap()
        .replace("grok-4-fast", "grok-drifted-manual");
    fs::write(&grok_runtime, drifted).unwrap();

    let grok_other_only = r#"
schema_version = 1
platform = "grok"

[profiles.grok-other]
model = "grok-4"
"#;
    let drifted_remove_err = fixture.run_with_stdin(
        &["grok", "profile", "import", "--stdin", "--replace", "--yes"],
        grok_other_only.as_bytes(),
    );
    assert_exit_code(&drifted_remove_err, 90);

    // Remove Grok `profile_entry_config_state.json` so Grok enters `UnsafeMissingEntryState`: `--replace` MUST also block with exit 90!
    let entry_state_file = fixture
        .platform_dir("grok")
        .join("profile_entry_config_state.json");
    assert!(
        entry_state_file.exists(),
        "expected Grok profile_entry_config_state.json at {}",
        entry_state_file.display()
    );
    fs::remove_file(&entry_state_file).unwrap();

    let unsafe_replace_err = fixture.run_with_stdin(
        &["grok", "profile", "import", "--stdin", "--replace", "--yes"],
        grok_other_only.as_bytes(),
    );
    assert_exit_code(&unsafe_replace_err, 90);
}

#[test]
fn regression_symlink_and_junction_traversal_rejected_exit_51() {
    let fixture = ProfileExchangeFixture::new();
    assert_success(&fixture.run_with_stdin(
        &[
            "claude",
            "profile",
            "create",
            "relay",
            "--base-url",
            "https://relay.example.com",
            "--model",
            "claude-sonnet-4-5",
            "--auth-token-stdin",
            "--json",
        ],
        b"sk-ant-link-guard-token-123\n",
    ));

    let real_dir = fixture.work_dir.join("real_target");
    fs::create_dir_all(&real_dir).unwrap();
    let link_dir = fixture.work_dir.join("linked_dir");

    #[cfg(windows)]
    {
        let status = Command::new("cmd")
            .args([
                "/C",
                "mklink",
                "/J",
                &link_dir.display().to_string(),
                &real_dir.display().to_string(),
            ])
            .status()
            .unwrap();
        assert!(status.success(), "mklink /J failed: {status}");
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&real_dir, &link_dir).unwrap();
    }

    // 1. Export through linked/junction parent directory fails with exit 51 without writing through!
    let linked_out = link_dir.join("out.toml");
    let exp_err = fixture.run_output(&[
        "claude",
        "profile",
        "export",
        "-o",
        &linked_out.display().to_string(),
    ]);
    assert_exit_code(&exp_err, 51);
    assert!(!real_dir.join("out.toml").exists());

    // 2. Relative traversal (`linked_dir/../real_target/escaped.toml`) caught BEFORE `..` collapse!
    let escaped_rel = "linked_dir/../real_target/escaped.toml";
    let rel_err = fixture.run_output(&["claude", "profile", "export", "-o", escaped_rel]);
    assert_exit_code(&rel_err, 51);
    assert!(!real_dir.join("escaped.toml").exists());

    // 3. Existing directory target fails with exit 51!
    let dir_err = fixture.run_output(&[
        "claude",
        "profile",
        "export",
        "-o",
        &real_dir.display().to_string(),
    ]);
    assert_exit_code(&dir_err, 51);

    // 4. Import through linked/junction parent directory also fails with exit 51!
    let seeded_in = real_dir.join("in.toml");
    fs::write(
        &seeded_in,
        "schema_version = 1\nplatform = \"claude\"\n[profiles.injected]\nauth_mode = \"subscription\"\n",
    )
    .unwrap();
    let linked_in = link_dir.join("in.toml");
    let imp_err = fixture.run_output(&[
        "claude",
        "profile",
        "import",
        &linked_in.display().to_string(),
    ]);
    assert_exit_code(&imp_err, 51);
}

#[test]
fn regression_corrupt_codex_profile_secrets_returns_exit_40_and_validate_error() {
    let fixture = ProfileExchangeFixture::new();
    assert_success(&fixture.run_with_stdin(
        &[
            "codex",
            "profile",
            "create",
            "work",
            "--model",
            "gpt-5",
            "--auth-mode",
            "openai_api_key",
            "--auth-token-stdin",
            "--json",
        ],
        b"sk-codex-valid-initial-999\n",
    ));

    let sentinel = "sk-corrupt-secret-json-sentinel-4040";
    let secrets_path = fixture.platform_dir("codex").join("profile_secrets.json");
    fs::write(
        &secrets_path,
        format!("{{\"profiles\": {{\"work\": \"{sentinel}\"}}}}"),
    )
    .unwrap();

    // 1. `codex profile export` returns exit 40 without echoing secret sentinel.
    let exp_err = fixture.run_output(&["codex", "profile", "export", "--stdout"]);
    assert_exit_code(&exp_err, 40);
    assert!(!String::from_utf8_lossy(&exp_err.stderr).contains(sentinel));

    // 2. `codex profile import` returns exit 40 without echoing secret sentinel.
    let imp_err = fixture.run_with_stdin(
        &["codex", "profile", "import", "--stdin"],
        b"schema_version = 1\nplatform = \"codex\"\n[profiles.other]\nauth_mode = \"openai_chatgpt\"\n",
    );
    assert_exit_code(&imp_err, 40);
    assert!(!String::from_utf8_lossy(&imp_err.stderr).contains(sentinel));

    // 3. `ccr validate --json` reports `profile_secrets` corrupt error and exits 40!
    let val_out = fixture.run_output(&["validate", "--json"]);
    assert_exit_code(&val_out, 40);
    let val_stdout = String::from_utf8_lossy(&val_out.stdout);
    assert!(!val_stdout.contains(sentinel));
    assert!(!String::from_utf8_lossy(&val_out.stderr).contains(sentinel));
    let parsed: Value = serde_json::from_str(&val_stdout).unwrap();
    let checks = parsed["checks"].as_array().unwrap();
    assert!(
        checks.iter().any(|c| {
            c["platform"] == "codex"
                && c["target"] == "profile_secrets"
                && c["category"] == "corrupt"
                && c["severity"] == "error"
                && c["exit_code"] == 40
        }),
        "expected corrupt profile_secrets check in validate report: {val_stdout}"
    );
}

#[allow(dead_code)]
fn _path_ref(p: &Path) -> &Path {
    p
}
