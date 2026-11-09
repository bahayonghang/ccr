//! `ccr <platform> profile edit` argument assembly and presentation.

use crate::application::profile_edit::{
    EditProfileOutcome, EditProfileRequest, FieldEdit, SecretEdit, edit_profile,
};
use crate::cli::subcommands::profile_args::ProfileEditActionArgs;
use crate::commands::common::print_next_steps;
use crate::models::{ApplyPolicy, Platform};
use ccr_core::Secret;
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::logging::ColorOutput;
use serde_json::Value;
use std::io::Read;

#[derive(Debug, Clone)]
pub struct ProfileEditCommand {
    pub platform_name: String,
    pub name: String,
    pub rename: Option<String>,
    pub description: Option<String>,
    pub model: Option<String>,
    pub base_url: Option<String>,
    pub provider: Option<String>,
    pub tags: Vec<String>,
    pub enable: bool,
    pub disable: bool,
    pub set: Vec<String>,
    pub clear: Vec<String>,
    pub auth_token_stdin: bool,
    pub api_key_stdin: bool,
    pub env_key: Option<String>,
    pub clear_auth_token: bool,
    pub api_backend: Option<String>,
    pub context_window: Option<u64>,
    pub supports_backend_search: Option<bool>,
    pub reasoning_effort: Option<String>,
    pub expected_revision: Option<String>,
    pub no_apply: bool,
    pub json: bool,
}

impl ProfileEditCommand {
    pub fn from_common(platform_name: &str, args: ProfileEditActionArgs) -> Self {
        Self {
            platform_name: platform_name.to_string(),
            name: args.name,
            rename: args.rename,
            description: args.description,
            model: args.model,
            base_url: args.base_url,
            provider: args.provider,
            tags: args.tags,
            enable: args.enable,
            disable: args.disable,
            set: args.set,
            clear: args.clear,
            auth_token_stdin: args.auth_token_stdin,
            api_key_stdin: false,
            env_key: None,
            clear_auth_token: args.clear_auth_token,
            api_backend: None,
            context_window: None,
            supports_backend_search: None,
            reasoning_effort: None,
            expected_revision: args.expected_revision,
            no_apply: args.no_apply,
            json: args.json,
        }
    }
}

pub async fn platform_profile_edit_command(command: ProfileEditCommand) -> Result<()> {
    let platform = parse_edit_platform(&command.platform_name)?;
    let secret = resolve_secret(&command)?;
    let request = build_edit_request(&command, platform, secret)?;
    let outcome = edit_profile(request)?;
    render_edit_outcome(&outcome, command.json)?;
    if outcome.is_success() {
        Ok(())
    } else {
        Err(CcrError::ConfigError(outcome.failure_message().into()))
    }
}

pub fn read_secret_text(text: &str) -> Result<Secret> {
    let mut value = text.to_string();
    if value.ends_with('\n') {
        value.pop();
        if value.ends_with('\r') {
            value.pop();
        }
    }
    if value.is_empty() {
        return Err(CcrError::ValidationError("密钥不能为空".into()));
    }
    Ok(Secret::new(value))
}

fn parse_edit_platform(platform_name: &str) -> Result<Platform> {
    let platform = platform_name
        .parse::<Platform>()
        .map_err(|_| CcrError::PlatformNotFound(platform_name.to_string()))?;
    if matches!(
        platform,
        Platform::Claude | Platform::Codex | Platform::Grok
    ) {
        Ok(platform)
    } else {
        Err(CcrError::PlatformNotSupported(format!(
            "{platform} 目前不支持 profile edit"
        )))
    }
}

fn resolve_secret(command: &ProfileEditCommand) -> Result<SecretEdit> {
    if command.auth_token_stdin || command.api_key_stdin {
        return Ok(SecretEdit::Replace(read_stdin_secret()?));
    }
    if command.clear_auth_token {
        return Ok(SecretEdit::Clear);
    }
    if let Some(env_key) = &command.env_key {
        return Ok(SecretEdit::UseEnvKey(env_key.clone()));
    }
    Ok(SecretEdit::Keep)
}

fn read_stdin_secret() -> Result<Secret> {
    if std::io::IsTerminal::is_terminal(&std::io::stdin()) {
        let value = rpassword::prompt_password("请输入密钥: ")
            .map_err(|error| CcrError::ValidationError(format!("无法读取密钥: {error}")))?;
        return read_secret_text(&value);
    }
    let mut buffer = String::new();
    std::io::stdin().read_to_string(&mut buffer)?;
    read_secret_text(&buffer)
}

fn build_edit_request(
    command: &ProfileEditCommand,
    platform: Platform,
    secret: SecretEdit,
) -> Result<EditProfileRequest> {
    if command.enable && command.disable {
        return Err(CcrError::ValidationError(
            "--enable 与 --disable 不能同时使用".into(),
        ));
    }
    let mut fields = Vec::new();
    push_string(&mut fields, "description", command.description.clone())?;
    push_string(&mut fields, "model", command.model.clone())?;
    push_string(&mut fields, "base_url", command.base_url.clone())?;
    push_string(&mut fields, "provider", command.provider.clone())?;
    if !command.tags.is_empty() {
        push_field(
            &mut fields,
            FieldEdit::Set {
                field: "tags".into(),
                value: Value::Array(command.tags.iter().cloned().map(Value::String).collect()),
            },
        )?;
    }
    if command.enable {
        push_field(
            &mut fields,
            FieldEdit::Set {
                field: "enabled".into(),
                value: Value::Bool(true),
            },
        )?;
    }
    if command.disable {
        push_field(
            &mut fields,
            FieldEdit::Set {
                field: "enabled".into(),
                value: Value::Bool(false),
            },
        )?;
    }
    push_string(&mut fields, "api_backend", command.api_backend.clone())?;
    if let Some(context_window) = command.context_window {
        push_field(
            &mut fields,
            FieldEdit::Set {
                field: "context_window".into(),
                value: Value::Number(context_window.into()),
            },
        )?;
    }
    if let Some(enabled) = command.supports_backend_search {
        push_field(
            &mut fields,
            FieldEdit::Set {
                field: "supports_backend_search".into(),
                value: Value::Bool(enabled),
            },
        )?;
    }
    if let Some(effort) = &command.reasoning_effort {
        let field = if platform == Platform::Codex {
            "model_reasoning_effort"
        } else {
            "reasoning_effort"
        };
        push_string(&mut fields, field, Some(effort.clone()))?;
    }
    for raw in &command.set {
        let (field, value) = parse_set_argument(raw)?;
        reject_inline_secret(&field)?;
        push_string(&mut fields, &field, Some(value))?;
    }
    for field in &command.clear {
        let field = field.trim();
        if field.is_empty() {
            return Err(CcrError::ValidationError("字段名不能为空".into()));
        }
        push_field(
            &mut fields,
            FieldEdit::Clear {
                field: field.to_string(),
            },
        )?;
    }
    Ok(EditProfileRequest {
        platform,
        name: command.name.clone(),
        rename: command.rename.clone(),
        expected_revision: command.expected_revision.clone(),
        fields,
        secret,
        apply: if command.no_apply {
            ApplyPolicy::Never
        } else {
            ApplyPolicy::IfCurrent
        },
    })
}

fn parse_set_argument(raw: &str) -> Result<(String, String)> {
    let (field, value) = raw
        .split_once('=')
        .ok_or_else(|| CcrError::ValidationError("--set 需要 field=value 形式".into()))?;
    let field = field.trim();
    if field.is_empty() {
        return Err(CcrError::ValidationError("字段名不能为空".into()));
    }
    Ok((field.to_string(), value.to_string()))
}

fn reject_inline_secret(field: &str) -> Result<()> {
    if matches!(field, "auth_token" | "api_key") {
        Err(CcrError::ValidationError(
            "密钥请使用标准输入参数，不要把密钥写进命令参数".into(),
        ))
    } else {
        Ok(())
    }
}

fn push_string(fields: &mut Vec<FieldEdit>, field: &str, value: Option<String>) -> Result<()> {
    if let Some(value) = value {
        push_field(
            fields,
            FieldEdit::Set {
                field: field.to_string(),
                value: Value::String(value),
            },
        )?;
    }
    Ok(())
}

fn push_field(fields: &mut Vec<FieldEdit>, edit: FieldEdit) -> Result<()> {
    if fields.iter().any(|existing| existing.name() == edit.name()) {
        return Err(CcrError::ValidationError(format!(
            "字段 '{}' 重复",
            edit.name()
        )));
    }
    fields.push(edit);
    Ok(())
}

fn render_edit_outcome(outcome: &EditProfileOutcome, json: bool) -> Result<()> {
    if json {
        println!(
            "{}",
            serde_json::to_string(&outcome.to_json_value()).map_err(CcrError::JsonError)?
        );
        return Ok(());
    }
    if !outcome.is_success() {
        return Ok(());
    }
    for warning in &outcome.warnings {
        ColorOutput::warning(warning_text(warning));
    }
    ColorOutput::success(&outcome.human_message());
    let command = format!("ccr {} profile current", outcome.platform.short_name());
    print_next_steps(&[("查看当前 profile", command.as_str())]);
    Ok(())
}

fn warning_text(code: &str) -> &'static str {
    match code {
        "usage_count_failed" => "使用次数未能更新",
        "history_failed" => "操作历史未能记录",
        "auth_registry_failed" => "认证注册表未能更新",
        "analytics_failed" => "提供商统计未能记录",
        "operation_record_failed" => "操作记录未能保存",
        "ancillary_pending" => "附加步骤尚未完成",
        "activation_failed" => "重新应用未能完成",
        "interrupted" => "操作被中断",
        _ => "附加步骤未完成",
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn command(platform: &str) -> ProfileEditCommand {
        ProfileEditCommand {
            platform_name: platform.into(),
            name: "work".into(),
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
            json: false,
        }
    }

    #[test]
    fn read_secret_text_strips_one_trailing_newline_and_rejects_empty() {
        let secret = read_secret_text("synthetic-profile-edit-secret-sentinel\n").unwrap();
        assert_eq!(secret.expose(), "synthetic-profile-edit-secret-sentinel");
        assert!(read_secret_text("\n").is_err());
    }

    #[test]
    fn set_rejects_inline_secrets() {
        let mut command = command("claude");
        command.set.push("auth_token=synthetic-secret".into());
        let error = build_edit_request(&command, Platform::Claude, SecretEdit::Keep).unwrap_err();
        assert!(error.to_string().contains("标准输入"));
        assert!(!error.to_string().contains("synthetic-secret"));
    }

    #[test]
    fn codex_reasoning_effort_uses_model_field() {
        let mut command = command("codex");
        command.reasoning_effort = Some("HIGH".into());
        let request = build_edit_request(&command, Platform::Codex, SecretEdit::Keep).unwrap();
        assert_eq!(request.fields[0].name(), "model_reasoning_effort");
    }
}
