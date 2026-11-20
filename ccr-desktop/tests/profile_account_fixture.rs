//! Explicit Windows fixture for the native Profiles → Accounts acceptance run.
#![cfg(windows)]

use ccr_cli::application::{platform_roots::PlatformRoots, profile_lifecycle::ApplyProfileRequest};
use ccr_config::{CcsConfig, ConfigSection, Platform};
use ccr_gpui::runtime::{ConfigSource, profiles::context};
use serde_json::json;
use std::{fs, path::Path, time::Duration};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn profiles(source: &ConfigSource, platform: Platform) -> Result {
    let owner = context(source, platform)?;
    let mut catalog = CcsConfig {
        default_config: "A-Primary".into(),
        current_config: String::new(),
        settings: Default::default(),
        sections: Default::default(),
    };
    for i in 0..27 {
        let name = match i {
            0 => "A-Primary".into(),
            1 => "B-Preview".into(),
            2 => "C-Apply".into(),
            3 => "D-Disabled".into(),
            _ => format!("Lab-{i:02}"),
        };
        let provider = ["North", "West", "East"][i % 3];
        let mut section = ConfigSection {
            provider: Some(provider.into()),
            base_url: Some(format!(
                "https://{}.example.invalid/v1",
                provider.to_lowercase()
            )),
            auth_token: Some(ccr_core::Secret::new(format!("fixture-only-{name}"))),
            model: (i != 4).then(|| format!("demo-model-{i:02}")),
            enabled: Some(i != 3),
            tags: Some(if i % 3 == 0 {
                vec!["demo".into(), "qa".into(), "local".into(), "team".into()]
            } else {
                vec!["demo".into()]
            }),
            ..Default::default()
        };
        if platform == Platform::Codex {
            section
                .other
                .insert("auth_mode".into(), "openai_api_key".into());
        }
        catalog.sections.insert(name, section);
    }
    owner.run(|| owner.manager().save(&catalog))?;
    owner.apply(ApplyProfileRequest::new(platform, "A-Primary"))?;
    Ok(())
}

fn write_json(path: &Path, value: serde_json::Value) -> Result {
    fs::create_dir_all(path.parent().ok_or("fixture parent")?)?;
    fs::write(path, serde_json::to_vec_pretty(&value)?)?;
    Ok(())
}

fn accounts(root: &Path) -> Result {
    let roots = PlatformRoots::fixture(&root.join("desktop-platform-fixture"))?;
    let grok = ccr_cli::services::GrokAuthService::from_roots(roots.clone());
    let scope = "https://auth.x.ai::fixture";
    for name in ["Account-B", "Account-A"] {
        let marker = format!("synthetic-only-{name}");
        write_json(
            &roots.claude.credentials_file,
            json!({"claudeAiOauth": {
                "accessToken": marker, "refreshToken": format!("refresh-{marker}"),
                "expiresAt": "2099-01-01T00:00:00Z", "scopes": ["user:profile"]
            }}),
        )?;
        write_json(
            &roots.claude.state_file,
            json!({"oauthAccount": {
                "accountUuid": name, "emailAddress": format!("{name}@example.invalid")
            }}),
        )?;
        roots.claude_auth().save_current(name, None, false)?;
        write_json(
            &roots.codex_runtime.join("auth.json"),
            json!({"OPENAI_API_KEY": marker}),
        )?;
        roots.codex_auth().save_current(name, None, false)?;
        write_json(
            &roots.grok_runtime.join("auth.json"),
            json!({(scope): {
                "auth_mode":"oidc", "oidc_issuer":"https://auth.x.ai", "oidc_client_id":"fixture",
                "key":marker, "user_id":name, "create_time":"2026-01-01T00:00:00Z"
            }}),
        )?;
        grok.save_current(name, scope, &grok.read_snapshot()?.revision, false)?;
    }
    Ok(())
}

#[test]
#[ignore = "creates a new synthetic Windows native QA catalog on explicit request"]
fn prepare_profile_account_fixture() -> Result {
    let output = Path::new(env!("CARGO_MANIFEST_DIR")).join("../outputs/desktop");
    let root = std::path::PathBuf::from(std::env::var("CCR_NATIVE_FIXTURE_ROOT")?);
    let parent = root.parent().ok_or("fixture parent")?.canonicalize()?;
    assert!(root.is_absolute() && parent.starts_with(output.canonicalize()?));
    fs::create_dir(&root)?;
    let source = ConfigSource::Fixture {
        profiles: root.join("profiles.toml"),
        registry: root.join("config.toml"),
        delay: Duration::ZERO,
    };
    for platform in [Platform::Claude, Platform::Codex, Platform::Grok] {
        profiles(&source, platform)?;
    }
    accounts(&root)?;
    write_json(
        &root.join("desktop-gpui/preferences.json"),
        json!({
            "version": 1, "environment": "local", "theme": "light", "language": "zh",
            "confirm_quit": false, "window": {"x": 60, "y": 60, "width": 1280, "height": 800, "maximized": false}
        }),
    )
}
