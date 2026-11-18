#![allow(clippy::unwrap_used)]

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use tempfile::TempDir;

struct LegacyRoutingFixture {
    _temp_dir: TempDir,
    home: PathBuf,
    root: PathBuf,
}

impl LegacyRoutingFixture {
    fn new() -> Self {
        let temp_dir = crate::command_tempdir().unwrap();
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

fn assert_unknown_subcommand_error(output: Output, subcommand: &str) {
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "expected Clap parse failure\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        stderr
    );
    assert!(stderr.contains("unrecognized subcommand"), "{stderr}");
    assert!(stderr.contains(subcommand), "{stderr}");
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
fn removed_platform_state_and_profile_commands_fail_with_clap_parse_error() {
    let fixture = LegacyRoutingFixture::new();

    assert_unknown_subcommand_error(
        fixture.run_output(&["platform", "switch", "codex"]),
        "switch",
    );
    assert_unknown_subcommand_error(fixture.run_output(&["platform", "current"]), "current");
    assert_unknown_subcommand_error(fixture.run_output(&["platform", "info", "claude"]), "info");
    assert_unknown_subcommand_error(fixture.run_output(&["platform", "init", "grok"]), "init");
    assert_unknown_subcommand_error(
        fixture.run_output(&["platform", "profile", "create", "gemini", "team"]),
        "profile",
    );
}

#[test]
fn hidden_legacy_commands_emit_deprecation_only_on_stderr_and_skip_readonly_logs() {
    let fixture = LegacyRoutingFixture::new();

    // 1. Read-only `ccr list` on missing config: stderr warning only, zero warning leak on stdout, zero ~/.ccr/logs.
    let missing_list = fixture.run_output(&["list"]);
    assert!(!missing_list.status.success());
    let missing_stdout = String::from_utf8_lossy(&missing_list.stdout);
    let missing_stderr = String::from_utf8_lossy(&missing_list.stderr);
    assert!(
        !missing_stdout.contains("已弃用") && !missing_stdout.contains("ccr claude profile list"),
        "deprecation warning leaked to stdout: {missing_stdout}"
    );
    assert!(
        missing_stderr.contains("警告:") && missing_stderr.contains("ccr claude profile list"),
        "{missing_stderr}"
    );
    assert!(
        !fixture.root.join("logs").exists(),
        "read-only `ccr list` must not create ~/.ccr/logs"
    );

    // Seed Claude profiles via canonical CLI without creating logs yet.
    let seed_main = fixture.run_with_stdin(
        &[
            "claude",
            "profile",
            "create",
            "main",
            "--base-url",
            "https://anthropic-main.example.com",
            "--model",
            "claude-sonnet-4-20250514",
            "--auth-token-stdin",
            "--json",
        ],
        b"sk-claude-main-secret-0000\n",
    );
    assert!(seed_main.status.success(), "{:?}", seed_main);
    let switch_main = fixture.run_output(&["claude", "profile", "switch", "main"]);
    assert!(switch_main.status.success(), "{:?}", switch_main);
    let seed = fixture.run_with_stdin(
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
            "--json",
        ],
        b"sk-claude-work-secret-1111\n",
    );
    assert!(seed.status.success(), "{:?}", seed);
    let _ = fs::remove_dir_all(fixture.root.join("logs"));

    // 2. Read-only `ccr list` and `ccr export --stdout` on existing catalog: zero ~/.ccr/logs!
    let ok_list = fixture.run_output(&["list"]);
    assert!(ok_list.status.success());
    let list_stdout = String::from_utf8_lossy(&ok_list.stdout);
    let list_stderr = String::from_utf8_lossy(&ok_list.stderr);
    assert!(!list_stdout.contains("已弃用"), "{list_stdout}");
    assert!(
        list_stderr.contains("ccr claude profile list"),
        "{list_stderr}"
    );
    assert!(
        !fixture.root.join("logs").exists(),
        "read-only `ccr list` must not create ~/.ccr/logs"
    );

    let export_out = fixture.run_output(&["export", "--platform", "claude", "--stdout"]);
    assert!(export_out.status.success());
    let export_stdout = String::from_utf8_lossy(&export_out.stdout);
    let export_stderr = String::from_utf8_lossy(&export_out.stderr);
    assert!(!export_stdout.contains("已弃用"), "{export_stdout}");
    assert!(
        export_stderr.contains("ccr claude profile export"),
        "{export_stderr}"
    );
    assert!(
        !fixture.root.join("logs").exists(),
        "read-only `ccr export --stdout` must not create ~/.ccr/logs"
    );
    let parsed_flat: toml::Table =
        toml::from_str(&export_stdout).expect("legacy `ccr export --stdout` must be pure TOML");
    assert!(
        parsed_flat.get("schema_version").is_none(),
        "{export_stdout}"
    );
    assert!(parsed_flat.get("profiles").is_none(), "{export_stdout}");
    assert!(parsed_flat.get("work").is_some(), "{export_stdout}");

    // 3. Remaining hidden commands emit deprecation warning ONLY to stderr, never stdout.
    for (args, expected_hint) in [
        (
            &["disable", "work", "--force"][..],
            "ccr claude profile disable",
        ),
        (&["enable", "work"][..], "ccr claude profile enable"),
        (&["optimize"][..], "profiles.db"),
        (&["clear", "--force"][..], "ccr claude profile off"),
        (
            &["delete", "work", "--force"][..],
            "ccr claude profile delete",
        ),
    ] {
        let out = fixture.run_output(args);
        assert!(
            out.status.success(),
            "expected {args:?} to succeed:\nstdout:{}\nstderr:{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        let stdout = String::from_utf8_lossy(&out.stdout);
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            !stdout.contains("已弃用"),
            "{args:?} leaked deprecation to stdout:\n{stdout}"
        );
        assert!(
            stderr.contains(expected_hint),
            "{args:?} missing stderr hint `{expected_hint}`:\n{stderr}"
        );
    }

    // `ccr add` with empty stdin exits non-zero or cancels, still emitting deprecation only on stderr.
    let add_out = fixture.run_with_stdin(&["add"], b"\n");
    let add_stdout = String::from_utf8_lossy(&add_out.stdout);
    let add_stderr = String::from_utf8_lossy(&add_out.stderr);
    assert!(!add_stdout.contains("已弃用"), "{add_stdout}");
    assert!(
        add_stderr.contains("ccr claude profile create"),
        "{add_stderr}"
    );

    // `ccr init --force` on a fresh fixture emits deprecation only on stderr.
    let fresh = LegacyRoutingFixture::new();
    let init_out = fresh.run_output(&["init", "--force"]);
    assert!(init_out.status.success());
    let init_stdout = String::from_utf8_lossy(&init_out.stdout);
    let init_stderr = String::from_utf8_lossy(&init_out.stderr);
    assert!(!init_stdout.contains("已弃用"), "{init_stdout}");
    assert!(
        init_stderr.contains("ccr claude profile init"),
        "{init_stderr}"
    );
}

#[test]
fn hidden_export_and_import_enforce_grok_redaction_codex_sidecar_and_replace_vs_merge() {
    let fixture = LegacyRoutingFixture::new();

    // 1. Grok `--no-secrets` via legacy `ccr export` must strip `api_key`.
    let grok_secret = "xai-grok-top-secret-998877";
    let grok_create = fixture.run_with_stdin(
        &[
            "grok",
            "profile",
            "create",
            "relay",
            "--base-url",
            "https://api.example.com/v1",
            "--model",
            "grok-4",
            "--api-key-stdin",
            "--json",
        ],
        format!("{grok_secret}\n").as_bytes(),
    );
    assert!(grok_create.status.success());

    let grok_redacted =
        fixture.run_output(&["export", "--platform", "grok", "--no-secrets", "--stdout"]);
    assert!(grok_redacted.status.success());
    let grok_stdout = String::from_utf8_lossy(&grok_redacted.stdout);
    let grok_stderr = String::from_utf8_lossy(&grok_redacted.stderr);
    assert!(!grok_stdout.contains(grok_secret), "{grok_stdout}");
    assert!(!grok_stderr.contains(grok_secret), "{grok_stderr}");
    assert!(!grok_stdout.contains("api_key"), "{grok_stdout}");
    let grok_tbl: toml::Table = toml::from_str(&grok_stdout).unwrap();
    assert!(grok_tbl.get("relay").is_some(), "{grok_stdout}");

    // 2. Codex sidecar secret overlaid in legacy `ccr export --platform codex --stdout`.
    let codex_secret = "sk-codex-sidecar-secret-5566";
    let codex_create = fixture.run_with_stdin(
        &[
            "codex",
            "profile",
            "create",
            "proxy",
            "--base-url",
            "https://codex-proxy.example.com/v1",
            "--model",
            "gpt-5",
            "--provider",
            "custom",
            "--provider-type",
            "third_party_model",
            "--auth-mode",
            "openai_api_key",
            "--auth-token-stdin",
            "--json",
        ],
        format!("{codex_secret}\n").as_bytes(),
    );
    assert!(codex_create.status.success());
    let codex_full = fixture.run_output(&["export", "--platform", "codex", "--stdout"]);
    assert!(codex_full.status.success());
    let codex_full_stdout = String::from_utf8_lossy(&codex_full.stdout);
    let codex_tbl: toml::Table = toml::from_str(&codex_full_stdout).unwrap();
    assert_eq!(
        codex_tbl["proxy"]["auth_token"].as_str(),
        Some(codex_secret)
    );

    // 3. Legacy `ccr import`:
    //    - Rejects masked secret placeholder
    //    - Rejects non-TTY Replace without `--force` / `-y`
    //    - Without `--merge` (`--force`): replaces catalog
    //    - With `--merge`: merges with overwrite
    for name in ["alpha", "beta"] {
        let out = fixture.run_with_stdin(
            &[
                "claude",
                "profile",
                "create",
                name,
                "--base-url",
                "https://old.example.com",
                "--model",
                "claude-sonnet-4-20250514",
                "--auth-token-stdin",
                "--json",
            ],
            b"sk-initial-token-1234\n",
        );
        assert!(out.status.success());
    }

    let work_dir = fixture.home.join("work");
    fs::create_dir_all(&work_dir).unwrap();

    // Default filename export check (`ccs_config_export_*.toml`)
    let file_export = fixture
        .command()
        .current_dir(&work_dir)
        .args(["export", "--platform", "claude"])
        .output()
        .unwrap();
    assert!(file_export.status.success());
    let exported_files: Vec<_> = fs::read_dir(&work_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("ccs_config_export_")
        })
        .collect();
    assert_eq!(exported_files.len(), 1);

    // Masked secret rejection
    let masked_file = work_dir.join("masked.toml");
    fs::write(
        &masked_file,
        "[alpha]\nbase_url = \"https://new.example.com\"\nauth_token = \"sk-ant-...1234\"\nmodel = \"claude-sonnet-4-20250514\"\n",
    )
    .unwrap();
    let masked_import = fixture.run_output(&[
        "import",
        masked_file.to_str().unwrap(),
        "--platform",
        "claude",
        "--force",
    ]);
    assert!(!masked_import.status.success());

    // Replace file containing ONLY `beta` (updated URL) and `gamma` (new) -> `alpha` must be removed!
    let replace_file = work_dir.join("replace.toml");
    fs::write(
        &replace_file,
        "[beta]\nbase_url = \"https://replaced.example.com\"\nauth_token = \"sk-replaced-9999\"\nmodel = \"claude-sonnet-4-20250514\"\n\n[gamma]\nbase_url = \"https://gamma.example.com\"\nauth_token = \"sk-gamma-8888\"\nmodel = \"claude-sonnet-4-20250514\"\n",
    )
    .unwrap();

    // Non-TTY Replace without `--force` or `-y` is rejected cleanly.
    let no_force = fixture.run_output(&[
        "import",
        replace_file.to_str().unwrap(),
        "--platform",
        "claude",
    ]);
    assert!(!no_force.status.success());

    // Replace with `--force` succeeds and removes `alpha`.
    let ok_replace = fixture.run_output(&[
        "import",
        replace_file.to_str().unwrap(),
        "--platform",
        "claude",
        "--force",
    ]);
    assert!(
        ok_replace.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&ok_replace.stderr)
    );
    assert!(String::from_utf8_lossy(&ok_replace.stderr).contains("ccr claude profile import"));

    let after_replace = fixture.run_output(&["claude", "profile", "list", "--json"]);
    assert!(after_replace.status.success());
    let list_json: serde_json::Value = serde_json::from_slice(&after_replace.stdout).unwrap();
    let names_after_replace: Vec<&str> = list_json["profiles"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|p| p["name"].as_str())
        .collect();
    assert_eq!(names_after_replace, vec!["beta", "gamma"]);

    // Merge (`--merge`) overwrites same-name `beta` while retaining `gamma` and adding `delta`.
    let merge_file = work_dir.join("merge.toml");
    fs::write(
        &merge_file,
        "[beta]\nbase_url = \"https://merged-overwrite.example.com\"\nauth_token = \"sk-merged-7777\"\nmodel = \"claude-sonnet-4-20250514\"\n\n[delta]\nbase_url = \"https://delta.example.com\"\nauth_token = \"sk-delta-6666\"\nmodel = \"claude-sonnet-4-20250514\"\n",
    )
    .unwrap();
    let ok_merge = fixture.run_output(&[
        "import",
        merge_file.to_str().unwrap(),
        "--merge",
        "--platform",
        "claude",
    ]);
    assert!(
        ok_merge.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&ok_merge.stderr)
    );
    let after_merge = fixture.run_output(&["claude", "profile", "list", "--json"]);
    let merge_json: serde_json::Value = serde_json::from_slice(&after_merge.stdout).unwrap();
    let profiles_arr = merge_json["profiles"].as_array().unwrap();
    let names_after_merge: Vec<&str> = profiles_arr
        .iter()
        .filter_map(|p| p["name"].as_str())
        .collect();
    assert_eq!(names_after_merge, vec!["beta", "gamma", "delta"]);
    let verify_export = fixture.run_output(&["claude", "profile", "export", "--stdout"]);
    assert!(verify_export.status.success());
    let verify_toml: toml::Value =
        toml::from_str(&String::from_utf8_lossy(&verify_export.stdout)).unwrap();
    assert_eq!(
        verify_toml["profiles"]["beta"]["base_url"].as_str(),
        Some("https://merged-overwrite.example.com")
    );
}

#[test]
fn legacy_non_tri_platform_import_never_echoes_secrets_and_guards_replace() {
    let fixture = LegacyRoutingFixture::new();
    let work_dir = fixture._temp_dir.path().join("gemini_import");
    fs::create_dir_all(&work_dir).unwrap();

    // 1. Malformed TOML on `--platform gemini` must never echo inline secret literals.
    let sentinel = "sk-gemini-never-echo-9999";
    let bad_file = work_dir.join("malformed_gemini.toml");
    fs::write(
        &bad_file,
        format!("[work]\nbase_url = \"https://gemini.example.com\"\nauth_token = \"{sentinel}\n"),
    )
    .unwrap();

    let bad_import = fixture.run_output(&[
        "import",
        bad_file.to_str().unwrap(),
        "--platform",
        "gemini",
        "--force",
    ]);
    assert!(!bad_import.status.success());
    let bad_stdout = String::from_utf8_lossy(&bad_import.stdout);
    let bad_stderr = String::from_utf8_lossy(&bad_import.stderr);
    assert!(
        !bad_stdout.contains(sentinel) && !bad_stderr.contains(sentinel),
        "malformed TOML leaked secret! stdout={bad_stdout}, stderr={bad_stderr}"
    );

    // 2. Valid TOML on `--platform gemini` in non-TTY Replace mode without `--force` is rejected cleanly without stdout prompt.
    let valid_file = work_dir.join("valid_gemini.toml");
    fs::write(
        &valid_file,
        "[work]\nbase_url = \"https://gemini.example.com\"\nauth_token = \"sk-gemini-ok-1234\"\n",
    )
    .unwrap();
    let unconfirmed = fixture.run_output(&[
        "import",
        valid_file.to_str().unwrap(),
        "--platform",
        "gemini",
    ]);
    assert!(!unconfirmed.status.success());
    let unconfirmed_stdout = String::from_utf8_lossy(&unconfirmed.stdout);
    assert!(
        !unconfirmed_stdout.contains("确认执行 Replace 操作"),
        "Replace prompt must never write to stdout: {unconfirmed_stdout}"
    );

    // 3. `--platform gemini --force` succeeds cleanly.
    let confirmed = fixture.run_output(&[
        "import",
        valid_file.to_str().unwrap(),
        "--platform",
        "gemini",
        "--force",
    ]);
    assert!(
        confirmed.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&confirmed.stderr)
    );
}
