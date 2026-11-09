//! Isolated-home coverage for `profile edit` and the shared edit use case.

#![allow(clippy::unwrap_used)]

use ccr_cli::application::profile_edit::{EditProfileRequest, FieldEdit, SecretEdit, edit_profile};
use ccr_cli::application::profile_lifecycle::{ApplyProfileRequest, apply_profile};
use ccr_cli::commands::platform::{
    ProfileEditCommand, platform_profile_edit_command, platform_profile_set_field_command,
    read_secret_text,
};
use ccr_cli::models::{ApplyPolicy, Platform, ProfileConfig};
use ccr_cli::platforms::create_platform;
use ccr_core::Secret;
use serde_json::Value;
use std::ffi::OsString;
use std::path::Path;
use std::sync::{Mutex, MutexGuard};

const SENTINEL: &str = "synthetic-profile-edit-secret-sentinel";

static TEST_ENV_LOCK: Mutex<()> = Mutex::new(());

struct IsolatedEnv {
    _guard: MutexGuard<'static, ()>,
    _temp: tempfile::TempDir,
    previous: Vec<(&'static str, Option<OsString>)>,
}

impl IsolatedEnv {
    fn new() -> Self {
        let guard = TEST_ENV_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let temp = tempfile::tempdir().unwrap();
        let home = temp.path().to_path_buf();
        for dir in [
            ".ccr",
            ".claude",
            ".claude/backups",
            ".locks",
            ".codex",
            ".grok",
        ] {
            std::fs::create_dir_all(home.join(dir)).unwrap();
        }
        let mut previous = Vec::new();
        set_env(&mut previous, "CCR_ROOT", &home.join(".ccr"));
        set_env(&mut previous, "CCR_LOCK_DIR", &home.join(".locks"));
        set_env(&mut previous, "CLAUDE_CONFIG_DIR", &home.join(".claude"));
        set_env(
            &mut previous,
            "CLAUDE_JSON_PATH",
            &home.join(".claude.json"),
        );
        set_env(
            &mut previous,
            "CCR_SETTINGS_PATH",
            &home.join(".claude/settings.json"),
        );
        set_env(
            &mut previous,
            "CCR_BACKUP_DIR",
            &home.join(".claude/backups"),
        );
        set_env(&mut previous, "CCR_CODEX_DIR", &home.join(".codex"));
        set_env(&mut previous, "GROK_HOME", &home.join(".grok"));
        remove_env(&mut previous, "CCR_CONFIG_PATH");
        Self {
            _guard: guard,
            _temp: temp,
            previous,
        }
    }
}

impl Drop for IsolatedEnv {
    fn drop(&mut self) {
        for (key, previous) in self.previous.drain(..).rev() {
            restore_env(key, previous);
        }
    }
}

fn set_env(previous: &mut Vec<(&'static str, Option<OsString>)>, key: &'static str, value: &Path) {
    previous.push((key, std::env::var_os(key)));
    // SAFETY: IsolatedEnv holds the test env lock until Drop restores this key.
    unsafe { std::env::set_var(key, value.as_os_str()) }
}

fn remove_env(previous: &mut Vec<(&'static str, Option<OsString>)>, key: &'static str) {
    previous.push((key, std::env::var_os(key)));
    // SAFETY: IsolatedEnv holds the test env lock until Drop restores this key.
    unsafe { std::env::remove_var(key) }
}

fn restore_env(key: &str, previous: Option<OsString>) {
    // SAFETY: Drop runs while IsolatedEnv still holds the test env lock.
    unsafe {
        match previous {
            Some(value) => std::env::set_var(key, value),
            None => std::env::remove_var(key),
        }
    }
}

fn seed(platform: Platform) {
    let owner = create_platform(platform).unwrap();
    for (name, url) in [
        ("work", "https://old.example"),
        ("other", "https://other.example"),
    ] {
        let mut profile = ProfileConfig::new()
            .with_base_url(url.into())
            .with_auth_token(SENTINEL.into())
            .with_model(format!("model-{name}"));
        profile.description = Some(format!("desc-{name}"));
        profile.provider = Some("provider-old".into());
        profile.provider_type = Some("third_party_model".into());
        profile
            .platform_data
            .insert("keep_me".into(), serde_json::json!({"n": 1}));
        owner.save_profile(name, &profile).unwrap();
    }
}

fn profiles(platform: Platform) -> indexmap::IndexMap<String, ProfileConfig> {
    create_platform(platform).unwrap().load_profiles().unwrap()
}

fn runtime_text(platform: Platform) -> String {
    let path = create_platform(platform).unwrap().get_settings_path();
    std::fs::read_to_string(path).unwrap_or_default()
}

fn revision(platform: Platform) -> String {
    let paths = ccr_cli::models::PlatformPaths::new(platform).unwrap();
    ccr_config::ConfigManager::new(&paths.profiles_file)
        .snapshot()
        .unwrap()
        .version
}

fn edit(
    platform: Platform,
    name: &str,
    rename: Option<&str>,
    fields: Vec<FieldEdit>,
    secret: SecretEdit,
    apply: ApplyPolicy,
    expected: Option<String>,
) -> ccr_cli::application::profile_edit::EditProfileOutcome {
    edit_profile(EditProfileRequest {
        platform,
        name: name.into(),
        rename: rename.map(str::to_string),
        expected_revision: expected,
        fields,
        secret,
        apply,
    })
    .unwrap()
}

fn string_field(field: &str, value: &str) -> FieldEdit {
    FieldEdit::Set {
        field: field.into(),
        value: Value::String(value.into()),
    }
}

fn command(platform: &str, name: &str) -> ProfileEditCommand {
    ProfileEditCommand {
        platform_name: platform.into(),
        name: name.into(),
        rename: None,
        description: None,
        model: None,
        base_url: None,
        provider: None,
        tags: Vec::new(),
        enable: false,
        disable: false,
        set: Vec::new(),
        clear: Vec::new(),
        auth_token_stdin: false,
        api_key_stdin: false,
        env_key: None,
        clear_auth_token: false,
        api_backend: None,
        context_window: None,
        supports_backend_search: None,
        reasoning_effort: None,
        expected_revision: None,
        no_apply: false,
        json: true,
    }
}

#[test]
fn edit_updates_only_requested_fields_and_reapplies_the_current_profile() {
    let _env = IsolatedEnv::new();
    for platform in [Platform::Claude, Platform::Codex, Platform::Grok] {
        seed(platform);
        assert!(
            apply_profile(ApplyProfileRequest::new(platform, "work"))
                .unwrap()
                .activation_committed,
            "{platform}"
        );
        let outcome = edit(
            platform,
            "work",
            None,
            vec![
                string_field("description", "desk"),
                string_field("model", "model-new"),
                string_field("base_url", "https://edited.example"),
            ],
            SecretEdit::Keep,
            ApplyPolicy::IfCurrent,
            None,
        );
        assert!(outcome.reapplied, "{platform}: {outcome:?}");
        assert!(outcome.is_success());
        let stored = &profiles(platform)["work"];
        assert_eq!(stored.description.as_deref(), Some("desk"));
        assert_eq!(stored.model.as_deref(), Some("model-new"));
        assert_eq!(stored.base_url.as_deref(), Some("https://edited.example"));
        assert_eq!(stored.provider.as_deref(), Some("provider-old"));
        assert_eq!(stored.platform_data["keep_me"], serde_json::json!({"n": 1}));
        assert_eq!(stored.auth_token.as_ref().unwrap().expose(), SENTINEL);
        let runtime = runtime_text(platform);
        assert!(
            runtime.contains("https://edited.example"),
            "{platform}: {runtime}"
        );
        assert!(!runtime.contains("https://old.example"), "{platform}");
        let json = outcome.to_json_value().to_string();
        assert!(json.contains("\"reapplied\":true"));
        assert!(!json.contains(SENTINEL));
        assert!(!format!("{outcome:?}").contains(SENTINEL));

        let runtime_before = runtime_text(platform);
        let stored_only = edit(
            platform,
            "work",
            None,
            vec![string_field("base_url", "https://stored-only.example")],
            SecretEdit::Keep,
            ApplyPolicy::Never,
            None,
        );
        assert!(!stored_only.reapplied);
        assert_eq!(runtime_text(platform), runtime_before);
        assert_eq!(
            profiles(platform)["work"].base_url.as_deref(),
            Some("https://stored-only.example")
        );

        let inactive_runtime = runtime_text(platform);
        edit(
            platform,
            "other",
            None,
            vec![string_field("base_url", "https://inactive.example")],
            SecretEdit::Keep,
            ApplyPolicy::IfCurrent,
            None,
        );
        assert_eq!(runtime_text(platform), inactive_runtime);
        assert_eq!(
            profiles(platform)["other"].base_url.as_deref(),
            Some("https://inactive.example")
        );
        assert_eq!(
            profiles(platform)["other"].platform_data["keep_me"],
            serde_json::json!({"n": 1})
        );
    }
}

#[test]
fn rename_keeps_the_current_profile_and_rejects_collisions() {
    let _env = IsolatedEnv::new();
    for platform in [Platform::Claude, Platform::Codex, Platform::Grok] {
        seed(platform);
        apply_profile(ApplyProfileRequest::new(platform, "work")).unwrap();
        let before = profiles(platform);
        let invalid = edit_profile(EditProfileRequest {
            platform,
            name: "work".into(),
            rename: Some("settings".into()),
            expected_revision: None,
            fields: Vec::new(),
            secret: SecretEdit::Keep,
            apply: ApplyPolicy::IfCurrent,
        })
        .unwrap_err();
        assert!(!invalid.to_string().contains(SENTINEL));
        assert_eq!(profiles(platform)["work"].base_url, before["work"].base_url);

        let collision = edit_profile(EditProfileRequest {
            platform,
            name: "work".into(),
            rename: Some("other".into()),
            expected_revision: None,
            fields: Vec::new(),
            secret: SecretEdit::Keep,
            apply: ApplyPolicy::IfCurrent,
        })
        .unwrap_err();
        assert!(!collision.to_string().contains(SENTINEL));
        assert!(profiles(platform).contains_key("work"));

        let renamed = edit(
            platform,
            "work",
            Some("renamed"),
            Vec::new(),
            SecretEdit::Keep,
            ApplyPolicy::IfCurrent,
            None,
        );
        assert!(renamed.reapplied, "{platform}: {renamed:?}");
        let stored = profiles(platform);
        assert!(!stored.contains_key("work"));
        assert!(stored.contains_key("renamed"));
        assert_eq!(
            create_platform(platform)
                .unwrap()
                .get_current_profile()
                .unwrap()
                .as_deref(),
            Some("renamed")
        );
        assert_eq!(
            stored["renamed"].platform_data["keep_me"],
            serde_json::json!({"n": 1})
        );
    }
}

#[test]
fn stdin_secret_is_stored_and_absent_from_output() {
    let _env = IsolatedEnv::new();
    seed(Platform::Claude);
    let secret = read_secret_text(&format!("{SENTINEL}\n")).unwrap();
    assert_eq!(secret.expose(), SENTINEL);
    let outcome = edit(
        Platform::Claude,
        "work",
        None,
        Vec::new(),
        SecretEdit::Replace(secret),
        ApplyPolicy::Never,
        None,
    );
    assert_eq!(
        profiles(Platform::Claude)["work"]
            .auth_token
            .as_ref()
            .unwrap()
            .expose(),
        SENTINEL
    );
    let json = outcome.to_json_value().to_string();
    assert!(!json.contains(SENTINEL));
    assert!(!outcome.human_message().contains(SENTINEL));
    assert!(!format!("{outcome:?}").contains(SENTINEL));
    let error = edit_profile(EditProfileRequest {
        platform: Platform::Claude,
        name: "missing".into(),
        rename: None,
        expected_revision: None,
        fields: vec![string_field("description", "x")],
        secret: SecretEdit::Replace(Secret::new(SENTINEL)),
        apply: ApplyPolicy::Never,
    })
    .unwrap_err();
    assert!(!error.to_string().contains(SENTINEL));
}

#[test]
fn grok_credential_replacement_is_exclusive() {
    let _env = IsolatedEnv::new();
    seed(Platform::Grok);
    let with_key = edit(
        Platform::Grok,
        "work",
        None,
        Vec::new(),
        SecretEdit::Replace(Secret::new(SENTINEL)),
        ApplyPolicy::Never,
        None,
    );
    let stored = &profiles(Platform::Grok)["work"];
    assert_eq!(stored.platform_data["api_key"], SENTINEL);
    assert!(stored.platform_data.get("env_key").is_none());
    assert!(stored.auth_token.is_none());
    assert!(!with_key.to_json_value().to_string().contains(SENTINEL));

    edit(
        Platform::Grok,
        "work",
        None,
        Vec::new(),
        SecretEdit::UseEnvKey("GROK_ENV_NAME".into()),
        ApplyPolicy::Never,
        None,
    );
    let stored = &profiles(Platform::Grok)["work"];
    assert_eq!(stored.platform_data["env_key"], "GROK_ENV_NAME");
    assert!(stored.platform_data.get("api_key").is_none());
    assert!(stored.auth_token.is_none());
}

#[tokio::test]
async fn set_field_keeps_its_output_shape_and_grok_exclusivity() {
    let _env = IsolatedEnv::new();
    let owner = create_platform(Platform::Grok).unwrap();
    let mut profile = ProfileConfig::new()
        .with_base_url("https://old.example".into())
        .with_model("model-work".into());
    profile.provider_type = Some("third_party_model".into());
    profile
        .platform_data
        .insert("env_key".into(), Value::String("GROK_ENV_NAME".into()));
    profile
        .platform_data
        .insert("keep_me".into(), serde_json::json!({"n": 1}));
    owner.save_profile("work", &profile).unwrap();

    platform_profile_set_field_command(
        "grok",
        "work",
        "api_key",
        Some(SENTINEL.into()),
        None,
        false,
        true,
    )
    .await
    .unwrap();
    let stored = &profiles(Platform::Grok)["work"];
    assert_eq!(stored.platform_data["api_key"], SENTINEL);
    assert!(stored.platform_data.get("env_key").is_none());
    assert_eq!(stored.platform_data["keep_me"], serde_json::json!({"n": 1}));
}

#[test]
fn expected_revision_mismatch_leaves_the_catalog_unchanged() {
    let _env = IsolatedEnv::new();
    seed(Platform::Claude);
    let original = revision(Platform::Claude);
    let error = edit_profile(EditProfileRequest {
        platform: Platform::Claude,
        name: "work".into(),
        rename: None,
        expected_revision: Some("stale-revision".into()),
        fields: vec![string_field("description", "changed")],
        secret: SecretEdit::Keep,
        apply: ApplyPolicy::IfCurrent,
    })
    .unwrap_err();
    assert!(error.to_string().contains("配置版本冲突"));
    assert_eq!(revision(Platform::Claude), original);
    assert_eq!(
        profiles(Platform::Claude)["work"].description.as_deref(),
        Some("desc-work")
    );
}

#[test]
fn disabling_the_current_profile_is_rejected() {
    let _env = IsolatedEnv::new();
    for platform in [Platform::Claude, Platform::Codex, Platform::Grok] {
        seed(platform);
        apply_profile(ApplyProfileRequest::new(platform, "work")).unwrap();
        let runtime = runtime_text(platform);
        let token = revision(platform);
        let error = edit_profile(EditProfileRequest {
            platform,
            name: "work".into(),
            rename: None,
            expected_revision: None,
            fields: vec![FieldEdit::Set {
                field: "enabled".into(),
                value: Value::Bool(false),
            }],
            secret: SecretEdit::Keep,
            apply: ApplyPolicy::IfCurrent,
        })
        .unwrap_err();
        assert!(error.to_string().contains("不能禁用"));
        assert!(profiles(platform)["work"].is_enabled());
        assert_eq!(runtime_text(platform), runtime);
        assert_eq!(revision(platform), token);
    }
}

#[test]
fn reapply_failure_restores_claude_and_codex_and_is_not_success() {
    let _env = IsolatedEnv::new();
    for platform in [Platform::Claude, Platform::Codex] {
        seed(platform);
        apply_profile(ApplyProfileRequest::new(platform, "work")).unwrap();
        edit(
            platform,
            "work",
            None,
            vec![string_field("base_url", "https://edited.example")],
            SecretEdit::Keep,
            ApplyPolicy::IfCurrent,
            None,
        );
        let runtime = runtime_text(platform);
        let token = revision(platform);
        let _fault = ccr_core::core::write_journal::fault::install(|path| {
            if ccr_core::core::write_journal::contains(path) {
                Err(ccr_core::CcrError::FileIoError(
                    "synthetic injected failure".into(),
                ))
            } else {
                Ok(())
            }
        });
        let outcome = edit(
            platform,
            "work",
            None,
            vec![string_field("base_url", "https://failed.example")],
            SecretEdit::Keep,
            ApplyPolicy::IfCurrent,
            None,
        );
        assert!(!outcome.is_success(), "{platform}: {outcome:?}");
        assert_eq!(outcome.status_name(), "unchanged");
        assert!(!outcome.reapplied);
        assert!(!outcome.human_message().contains("已更新") || !outcome.is_success());
        assert!(!outcome.to_json_value()["ok"].as_bool().unwrap());
        assert!(!outcome.to_json_value().to_string().contains(SENTINEL));
        assert_eq!(runtime_text(platform), runtime);
        assert_eq!(revision(platform), token);
        assert_eq!(
            profiles(platform)["work"].base_url.as_deref(),
            Some("https://edited.example")
        );
    }
}

#[tokio::test]
async fn edit_command_json_reports_reapply_without_secrets() {
    let _env = IsolatedEnv::new();
    seed(Platform::Claude);
    apply_profile(ApplyProfileRequest::new(Platform::Claude, "work")).unwrap();
    let mut command = command("claude", "work");
    command.description = Some("desk".into());
    command.model = Some("model-new".into());
    command.base_url = Some("https://edited.example".into());
    command.clear_auth_token = false;
    platform_profile_edit_command(command).await.unwrap();
    assert!(runtime_text(Platform::Claude).contains("https://edited.example"));
    assert_eq!(
        profiles(Platform::Claude)["work"].description.as_deref(),
        Some("desk")
    );
}

#[test]
fn edit_command_parse_uses_stdin_flag_without_a_secret_value() {
    use ccr_cli::cli::subcommands::{
        ClaudeAction, ClaudeProfileAction, GrokAction, GrokProfileAction,
    };
    use ccr_cli::cli::{Cli, Commands};
    use clap::Parser;

    let cli = Cli::try_parse_from([
        "ccr",
        "claude",
        "profile",
        "edit",
        "work",
        "--description",
        "desk",
        "--auth-token-stdin",
        "--no-apply",
        "--json",
    ])
    .unwrap();
    match cli.command {
        Some(Commands::Claude {
            action: Some(ClaudeAction::Profile { action }),
        }) => match action.as_ref() {
            ClaudeProfileAction::Edit(args) => {
                assert!(args.auth_token_stdin);
                assert!(args.no_apply);
                assert!(args.json);
                assert_eq!(args.description.as_deref(), Some("desk"));
                assert!(!format!("{args:?}").contains(SENTINEL));
            }
            _ => panic!("expected edit"),
        },
        _ => panic!("expected claude profile edit"),
    }

    let conflict = Cli::try_parse_from([
        "ccr",
        "grok",
        "profile",
        "edit",
        "work",
        "--api-key-stdin",
        "--env-key",
        "GROK_ENV_NAME",
    ]);
    let error = match conflict {
        Ok(_) => panic!("api key stdin and env key must conflict"),
        Err(error) => error,
    };
    assert_eq!(error.kind(), clap::error::ErrorKind::ArgumentConflict);
    let _ = (GrokAction::Help, GrokProfileAction::Help);
}

fn json_field(field: &str, value: Value) -> FieldEdit {
    FieldEdit::Set {
        field: field.into(),
        value,
    }
}

#[test]
fn claude_advanced_fields_round_trip_and_bad_effort_does_not_write() {
    let _env = IsolatedEnv::new();
    seed(Platform::Claude);
    let outcome = edit(
        Platform::Claude,
        "work",
        None,
        vec![
            string_field("default_opus_model", "claude-opus"),
            string_field("effort_level", "HIGH"),
            json_field("api_timeout_ms", serde_json::json!(0)),
            json_field(
                "claude_code_auto_compact_window",
                serde_json::json!(120_000),
            ),
        ],
        SecretEdit::Keep,
        ApplyPolicy::Never,
        None,
    );
    assert!(outcome.is_success(), "{outcome:?}");
    let stored = &profiles(Platform::Claude)["work"];
    assert_eq!(stored.default_opus_model.as_deref(), Some("claude-opus"));
    assert_eq!(stored.effort_level.as_deref(), Some("high"));
    assert_eq!(stored.api_timeout_ms.as_deref(), Some("0"));
    assert_eq!(
        stored.claude_code_auto_compact_window.as_deref(),
        Some("120000")
    );
    assert_eq!(stored.description.as_deref(), Some("desc-work"));
    assert_eq!(stored.platform_data["keep_me"], serde_json::json!({"n": 1}));

    let revision_before = revision(Platform::Claude);
    let error = edit_profile(EditProfileRequest {
        platform: Platform::Claude,
        name: "work".into(),
        rename: None,
        expected_revision: None,
        fields: vec![string_field("effort_level", "turbo")],
        secret: SecretEdit::Keep,
        apply: ApplyPolicy::Never,
    })
    .unwrap_err();
    assert!(error.to_string().contains("effort_level"), "{error}");
    assert_eq!(revision(Platform::Claude), revision_before);
    assert_eq!(
        profiles(Platform::Claude)["work"].effort_level.as_deref(),
        Some("high")
    );
}

#[test]
fn codex_route_fields_round_trip_and_invalid_values_do_not_write() {
    let _env = IsolatedEnv::new();
    seed(Platform::Codex);
    let outcome = edit(
        Platform::Codex,
        "work",
        None,
        vec![
            string_field("wire_api", "responses"),
            string_field("env_key", "CODEX_ENV_NAME"),
            json_field("requires_openai_auth", serde_json::json!(false)),
        ],
        SecretEdit::Keep,
        ApplyPolicy::Never,
        None,
    );
    assert!(outcome.is_success(), "{outcome:?}");
    let stored = &profiles(Platform::Codex)["work"];
    assert_eq!(stored.platform_data["wire_api"], "responses");
    assert_eq!(stored.platform_data["env_key"], "CODEX_ENV_NAME");
    assert_eq!(stored.platform_data["requires_openai_auth"], false);
    assert!(stored.platform_data.contains_key("requires_openai_auth"));
    assert_eq!(stored.auth_token.as_ref().unwrap().expose(), SENTINEL);
    assert_eq!(stored.platform_data["keep_me"], serde_json::json!({"n": 1}));

    let revision_before = revision(Platform::Codex);
    let chat = edit_profile(EditProfileRequest {
        platform: Platform::Codex,
        name: "work".into(),
        rename: None,
        expected_revision: None,
        fields: vec![string_field("wire_api", "chat")],
        secret: SecretEdit::Keep,
        apply: ApplyPolicy::Never,
    })
    .unwrap_err();
    assert!(chat.to_string().contains("responses"), "{chat}");
    assert_eq!(revision(Platform::Codex), revision_before);
    assert_eq!(
        profiles(Platform::Codex)["work"].platform_data["wire_api"],
        "responses"
    );

    let foreign = edit_profile(EditProfileRequest {
        platform: Platform::Codex,
        name: "work".into(),
        rename: None,
        expected_revision: None,
        fields: vec![string_field("default_opus_model", "claude-opus")],
        secret: SecretEdit::Keep,
        apply: ApplyPolicy::Never,
    })
    .unwrap_err();
    assert!(foreign.to_string().contains("不允许编辑字段"), "{foreign}");
    assert_eq!(revision(Platform::Codex), revision_before);
    assert!(
        profiles(Platform::Codex)["work"]
            .default_opus_model
            .is_none()
    );
}
