//! Per-platform profile editor fields. Labels resolve at render time.

use ccr_cli::models::Platform;

use super::catalog::{self, ModelOption};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FieldGroup {
    General,
    Advanced,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FieldKind {
    Text,
    Secret,
    Toggle,
    Choice(&'static [&'static str]),
    Number,
    Tags,
    /// Known model ids, plus one custom slot the user can type.
    Model(&'static [ModelOption]),
    /// Grok credential mode. The value is not itself a catalog field.
    Credential,
    /// Expands the advanced group. It is not submitted.
    Advanced,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct FieldSpec {
    pub key: &'static str,
    pub kind: FieldKind,
    pub group: FieldGroup,
}

const CODEX_REASONING: &[&str] = &["minimal", "low", "medium", "high", "xhigh"];
const GROK_REASONING: &[&str] = &["none", "minimal", "low", "medium", "high", "xhigh", "max"];
const CLAUDE_EFFORT: &[&str] = &["low", "medium", "high", "xhigh", "max"];
const WIRE_API: &[&str] = &["responses"];
const API_BACKEND: &[&str] = &["chat_completions", "responses", "messages"];

fn text(key: &'static str) -> FieldSpec {
    FieldSpec {
        key,
        kind: FieldKind::Text,
        group: FieldGroup::General,
    }
}

fn advanced(key: &'static str, kind: FieldKind) -> FieldSpec {
    FieldSpec {
        key,
        kind,
        group: FieldGroup::Advanced,
    }
}

fn model_field(key: &'static str, options: &'static [ModelOption], group: FieldGroup) -> FieldSpec {
    FieldSpec {
        key,
        kind: FieldKind::Model(options),
        group,
    }
}

pub(crate) fn specs(platform: Platform) -> Vec<FieldSpec> {
    let mut fields = vec![
        text("name"),
        text("description"),
        model_field("model", catalog::main_for(platform), FieldGroup::General),
        text("base_url"),
    ];
    match platform {
        Platform::Claude | Platform::Codex => fields.push(FieldSpec {
            key: "auth_token",
            kind: FieldKind::Secret,
            group: FieldGroup::General,
        }),
        _ => {}
    }
    fields.extend([
        text("provider"),
        FieldSpec {
            key: "tags",
            kind: FieldKind::Tags,
            group: FieldGroup::General,
        },
        FieldSpec {
            key: "enabled",
            kind: FieldKind::Toggle,
            group: FieldGroup::General,
        },
    ]);
    match platform {
        Platform::Claude => fields.extend([
            FieldSpec {
                key: "advanced",
                kind: FieldKind::Advanced,
                group: FieldGroup::General,
            },
            model_field("small_fast_model", catalog::CLAUDE, FieldGroup::Advanced),
            model_field(
                "default_opus_model",
                catalog::CLAUDE_OPUS,
                FieldGroup::Advanced,
            ),
            model_field(
                "default_sonnet_model",
                catalog::CLAUDE_SONNET,
                FieldGroup::Advanced,
            ),
            model_field(
                "default_haiku_model",
                catalog::CLAUDE_HAIKU,
                FieldGroup::Advanced,
            ),
            model_field(
                "default_fable_model",
                catalog::CLAUDE_FABLE,
                FieldGroup::Advanced,
            ),
            model_field("subagent_model", catalog::CLAUDE, FieldGroup::Advanced),
            advanced("effort_level", FieldKind::Choice(CLAUDE_EFFORT)),
            advanced("api_timeout_ms", FieldKind::Number),
            advanced("claude_code_auto_compact_window", FieldKind::Number),
        ]),
        Platform::Codex => fields.extend([
            FieldSpec {
                key: "model_reasoning_effort",
                kind: FieldKind::Choice(CODEX_REASONING),
                group: FieldGroup::General,
            },
            FieldSpec {
                key: "wire_api",
                kind: FieldKind::Choice(WIRE_API),
                group: FieldGroup::General,
            },
            text("env_key"),
            FieldSpec {
                key: "requires_openai_auth",
                kind: FieldKind::Toggle,
                group: FieldGroup::General,
            },
        ]),
        Platform::Grok => fields.extend([
            FieldSpec {
                key: "credential",
                kind: FieldKind::Credential,
                group: FieldGroup::General,
            },
            FieldSpec {
                key: "api_key",
                kind: FieldKind::Secret,
                group: FieldGroup::General,
            },
            text("env_key"),
            FieldSpec {
                key: "reasoning_effort",
                kind: FieldKind::Choice(GROK_REASONING),
                group: FieldGroup::General,
            },
            FieldSpec {
                key: "api_backend",
                kind: FieldKind::Choice(API_BACKEND),
                group: FieldGroup::General,
            },
            FieldSpec {
                key: "context_window",
                kind: FieldKind::Number,
                group: FieldGroup::General,
            },
            FieldSpec {
                key: "supports_backend_search",
                kind: FieldKind::Toggle,
                group: FieldGroup::General,
            },
        ]),
        _ => {}
    }
    fields
}

pub(crate) fn label(key: &str) -> &'static str {
    match key {
        "name" => crate::tui_text!("name", "名称"),
        "description" => crate::tui_text!("description", "描述"),
        "model" => crate::tui_text!("model", "模型"),
        "base_url" => crate::tui_text!("base URL", "地址"),
        "auth_token" => crate::tui_text!("secret", "密钥"),
        "provider" => crate::tui_text!("provider", "提供商"),
        "tags" => crate::tui_text!("tags", "标签"),
        "enabled" => crate::tui_text!("enabled", "启用"),
        "advanced" => crate::tui_text!("advanced", "高级"),
        "small_fast_model" => crate::tui_text!("fast model", "快速模型"),
        "default_opus_model" => crate::tui_text!("opus model", "Opus 模型"),
        "default_sonnet_model" => crate::tui_text!("sonnet model", "Sonnet 模型"),
        "default_haiku_model" => crate::tui_text!("haiku model", "Haiku 模型"),
        "default_fable_model" => crate::tui_text!("fable model", "Fable 模型"),
        "subagent_model" => crate::tui_text!("subagent model", "子代理模型"),
        "effort_level" => crate::tui_text!("effort", "思考强度"),
        "api_timeout_ms" => crate::tui_text!("timeout ms", "超时"),
        "claude_code_auto_compact_window" => crate::tui_text!("auto compact", "自动压缩"),
        "model_reasoning_effort" | "reasoning_effort" => crate::tui_text!("reasoning", "推理强度"),
        "wire_api" => crate::tui_text!("wire API", "协议"),
        "env_key" => crate::tui_text!("env key", "环境变量"),
        "requires_openai_auth" => crate::tui_text!("OpenAI auth", "需要 OpenAI 认证"),
        "credential" => crate::tui_text!("credential", "凭据"),
        "api_key" => crate::tui_text!("API key", "API 密钥"),
        "api_backend" => crate::tui_text!("backend", "后端"),
        "context_window" => crate::tui_text!("context", "上下文"),
        "supports_backend_search" => crate::tui_text!("backend search", "后端搜索"),
        _ => crate::tui_text!("field", "字段"),
    }
}

pub(crate) fn credential_label(value: &str) -> &'static str {
    match value {
        "replace" => crate::tui_text!("replace API key", "替换 API 密钥"),
        "env" => crate::tui_text!("use env key", "改用环境变量"),
        "clear" => crate::tui_text!("clear", "清除"),
        _ => crate::tui_text!("keep", "保留"),
    }
}
