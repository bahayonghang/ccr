//! Per-platform profile editor fields. Labels resolve at render time.

use ccr_cli::models::Platform;

use super::catalog::{self, ModelOption};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EditorMode {
    Create,
    Edit,
}

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
    /// Create-wizard auth/credential mode selector.
    AuthMode(&'static [&'static str]),
    /// Grok edit credential mode. The value is not itself a catalog field.
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

const CLAUDE_AUTH_MODES: &[&str] = &["api_key", "subscription"];
const CODEX_AUTH_MODES: &[&str] = &[
    "provider_bearer_token",
    "provider_env_key",
    "openai_api_key",
    "openai_chatgpt",
    "no_auth",
];
const GROK_CREATE_MODES: &[&str] = &["api_key", "env_key", "session"];
const PROVIDER_TYPES: &[&str] = &["official_relay", "third_party_model"];
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

pub(crate) fn create_specs(platform: Platform) -> Vec<FieldSpec> {
    match platform {
        Platform::Claude => vec![
            FieldSpec {
                key: "auth_mode",
                kind: FieldKind::AuthMode(CLAUDE_AUTH_MODES),
                group: FieldGroup::General,
            },
            text("name"),
            text("description"),
            text("base_url"),
            FieldSpec {
                key: "auth_token",
                kind: FieldKind::Secret,
                group: FieldGroup::General,
            },
            model_field("model", catalog::CLAUDE, FieldGroup::General),
            model_field("small_fast_model", catalog::CLAUDE, FieldGroup::General),
            text("provider"),
            FieldSpec {
                key: "provider_type",
                kind: FieldKind::Choice(PROVIDER_TYPES),
                group: FieldGroup::General,
            },
            text("account"),
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
            FieldSpec {
                key: "advanced",
                kind: FieldKind::Advanced,
                group: FieldGroup::General,
            },
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
        ],
        Platform::Codex => vec![
            FieldSpec {
                key: "auth_mode",
                kind: FieldKind::AuthMode(CODEX_AUTH_MODES),
                group: FieldGroup::General,
            },
            text("name"),
            text("description"),
            text("base_url"),
            text("env_key"),
            FieldSpec {
                key: "auth_token",
                kind: FieldKind::Secret,
                group: FieldGroup::General,
            },
            model_field("model", catalog::CODEX, FieldGroup::General),
            model_field("small_fast_model", catalog::CODEX, FieldGroup::General),
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
            text("provider"),
            FieldSpec {
                key: "provider_type",
                kind: FieldKind::Choice(PROVIDER_TYPES),
                group: FieldGroup::General,
            },
            text("account"),
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
        ],
        Platform::Grok => vec![
            FieldSpec {
                key: "auth_mode",
                kind: FieldKind::AuthMode(GROK_CREATE_MODES),
                group: FieldGroup::General,
            },
            text("name"),
            text("description"),
            text("base_url"),
            FieldSpec {
                key: "api_key",
                kind: FieldKind::Secret,
                group: FieldGroup::General,
            },
            text("env_key"),
            model_field("model", catalog::GROK, FieldGroup::General),
            FieldSpec {
                key: "api_backend",
                kind: FieldKind::Choice(API_BACKEND),
                group: FieldGroup::General,
            },
            FieldSpec {
                key: "reasoning_effort",
                kind: FieldKind::Choice(GROK_REASONING),
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
            text("provider"),
            FieldSpec {
                key: "provider_type",
                kind: FieldKind::Choice(PROVIDER_TYPES),
                group: FieldGroup::General,
            },
            text("account"),
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
        ],
        _ => specs(platform),
    }
}

pub(crate) fn platform_label(platform: Platform) -> &'static str {
    match platform {
        Platform::Claude => "Claude",
        Platform::Codex => "Codex",
        Platform::Grok => "Grok",
        other => other.display_name(),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CreateAuthMode {
    ClaudeApiKey,
    ClaudeSubscription,
    CodexProviderBearerToken,
    CodexProviderEnvKey,
    CodexOpenAiApiKey,
    CodexOpenAiChatgpt,
    CodexNoAuth,
    GrokApiKey,
    GrokEnvKey,
    GrokSession,
}

impl CreateAuthMode {
    pub(crate) fn default_for(platform: Platform) -> Self {
        match platform {
            Platform::Claude => Self::ClaudeApiKey,
            Platform::Codex => Self::CodexProviderBearerToken,
            Platform::Grok => Self::GrokApiKey,
            _ => Self::ClaudeApiKey,
        }
    }

    pub(crate) fn parse(platform: Platform, raw: &str) -> Self {
        match (platform, raw.trim()) {
            (Platform::Claude, "subscription") => Self::ClaudeSubscription,
            (Platform::Claude, _) => Self::ClaudeApiKey,
            (Platform::Codex, "provider_env_key") => Self::CodexProviderEnvKey,
            (Platform::Codex, "openai_api_key") => Self::CodexOpenAiApiKey,
            (Platform::Codex, "openai_chatgpt") => Self::CodexOpenAiChatgpt,
            (Platform::Codex, "no_auth") => Self::CodexNoAuth,
            (Platform::Codex, _) => Self::CodexProviderBearerToken,
            (Platform::Grok, "env_key") => Self::GrokEnvKey,
            (Platform::Grok, "session") => Self::GrokSession,
            (Platform::Grok, _) => Self::GrokApiKey,
            (_, _) => Self::default_for(platform),
        }
    }

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::ClaudeApiKey => "api_key",
            Self::ClaudeSubscription => "subscription",
            Self::CodexProviderBearerToken => "provider_bearer_token",
            Self::CodexProviderEnvKey => "provider_env_key",
            Self::CodexOpenAiApiKey => "openai_api_key",
            Self::CodexOpenAiChatgpt => "openai_chatgpt",
            Self::CodexNoAuth => "no_auth",
            Self::GrokApiKey => "api_key",
            Self::GrokEnvKey => "env_key",
            Self::GrokSession => "session",
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::ClaudeApiKey => crate::tui_text!("API key (api_key)", "API 令牌 (api_key)"),
            Self::ClaudeSubscription => {
                crate::tui_text!("Subscription (subscription)", "官方订阅 (subscription)")
            }
            Self::CodexProviderBearerToken => crate::tui_text!(
                "Bearer token (provider_bearer_token)",
                "Bearer 令牌 (provider_bearer_token)"
            ),
            Self::CodexProviderEnvKey => crate::tui_text!(
                "Env key (provider_env_key)",
                "环境变量密钥 (provider_env_key)"
            ),
            Self::CodexOpenAiApiKey => crate::tui_text!(
                "OpenAI API key (openai_api_key)",
                "OpenAI 密钥 (openai_api_key)"
            ),
            Self::CodexOpenAiChatgpt => crate::tui_text!(
                "ChatGPT login (openai_chatgpt)",
                "ChatGPT 登录态 (openai_chatgpt)"
            ),
            Self::CodexNoAuth => {
                crate::tui_text!("No auth (no_auth)", "免认证端点 (no_auth)")
            }
            Self::GrokApiKey => {
                crate::tui_text!("Inline API key (api_key)", "内联密钥 (api_key)")
            }
            Self::GrokEnvKey => {
                crate::tui_text!("Env variable (env_key)", "环境变量 (env_key)")
            }
            Self::GrokSession => {
                crate::tui_text!("Official session (session)", "官方会话 (session)")
            }
        }
    }

    pub(crate) fn emits_auth_mode_field(self) -> bool {
        match self {
            Self::ClaudeApiKey
            | Self::ClaudeSubscription
            | Self::CodexProviderBearerToken
            | Self::CodexProviderEnvKey
            | Self::CodexOpenAiApiKey
            | Self::CodexOpenAiChatgpt
            | Self::CodexNoAuth => true,
            Self::GrokApiKey | Self::GrokEnvKey | Self::GrokSession => false,
        }
    }

    pub(crate) fn secret_field_key(self) -> Option<&'static str> {
        match self {
            Self::ClaudeApiKey
            | Self::CodexProviderBearerToken
            | Self::CodexProviderEnvKey
            | Self::CodexOpenAiApiKey => Some("auth_token"),
            Self::GrokApiKey => Some("api_key"),
            Self::ClaudeSubscription
            | Self::CodexOpenAiChatgpt
            | Self::CodexNoAuth
            | Self::GrokEnvKey
            | Self::GrokSession => None,
        }
    }

    pub(crate) fn uses_grok_env_key_secret(self) -> bool {
        match self {
            Self::GrokEnvKey => true,
            Self::ClaudeApiKey
            | Self::ClaudeSubscription
            | Self::CodexProviderBearerToken
            | Self::CodexProviderEnvKey
            | Self::CodexOpenAiApiKey
            | Self::CodexOpenAiChatgpt
            | Self::CodexNoAuth
            | Self::GrokApiKey
            | Self::GrokSession => false,
        }
    }

    pub(crate) fn conflicts_with_official_relay(self) -> bool {
        match self {
            Self::CodexProviderBearerToken
            | Self::CodexProviderEnvKey
            | Self::CodexNoAuth
            | Self::GrokApiKey
            | Self::GrokEnvKey => true,
            Self::ClaudeApiKey
            | Self::ClaudeSubscription
            | Self::CodexOpenAiApiKey
            | Self::CodexOpenAiChatgpt
            | Self::GrokSession => false,
        }
    }

    pub(crate) fn requires_base_url_for_third_party_model(self) -> bool {
        match self {
            Self::CodexOpenAiApiKey | Self::CodexOpenAiChatgpt => true,
            Self::ClaudeApiKey
            | Self::ClaudeSubscription
            | Self::CodexProviderBearerToken
            | Self::CodexProviderEnvKey
            | Self::CodexNoAuth
            | Self::GrokApiKey
            | Self::GrokEnvKey
            | Self::GrokSession => false,
        }
    }

    pub(crate) fn is_key_visible(self, key: &str) -> bool {
        match self {
            Self::ClaudeApiKey => true,
            Self::ClaudeSubscription => !matches!(key, "base_url" | "auth_token" | "provider_type"),
            Self::CodexProviderBearerToken | Self::CodexOpenAiApiKey => key != "env_key",
            Self::CodexProviderEnvKey => true,
            Self::CodexOpenAiChatgpt | Self::CodexNoAuth => {
                !matches!(key, "auth_token" | "env_key")
            }
            Self::GrokApiKey => key != "env_key",
            Self::GrokEnvKey => key != "api_key",
            Self::GrokSession => {
                !matches!(key, "base_url" | "api_key" | "env_key" | "provider_type")
            }
        }
    }

    pub(crate) fn is_field_visible(self, spec: FieldSpec, advanced_open: bool) -> bool {
        if spec.kind == FieldKind::Advanced {
            return true;
        }
        if spec.group == FieldGroup::Advanced && !advanced_open {
            return false;
        }
        self.is_key_visible(spec.key)
    }

    pub(crate) fn is_field_required(self, key: &str) -> bool {
        if key == "name" {
            return true;
        }
        match self {
            Self::ClaudeApiKey => matches!(key, "base_url" | "auth_token"),
            Self::ClaudeSubscription => false,
            Self::CodexProviderBearerToken => matches!(key, "base_url" | "auth_token"),
            Self::CodexProviderEnvKey => matches!(key, "base_url" | "env_key" | "auth_token"),
            Self::CodexOpenAiApiKey => key == "auth_token",
            Self::CodexOpenAiChatgpt => false,
            Self::CodexNoAuth => key == "base_url",
            Self::GrokApiKey => matches!(key, "base_url" | "api_key" | "model"),
            Self::GrokEnvKey => matches!(key, "base_url" | "env_key" | "model"),
            Self::GrokSession => false,
        }
    }
}

pub(crate) fn label(key: &str) -> &'static str {
    match key {
        "auth_mode" => crate::tui_text!("auth mode", "认证模式"),
        "name" => crate::tui_text!("name", "名称"),
        "description" => crate::tui_text!("description", "描述"),
        "model" => crate::tui_text!("model", "模型"),
        "base_url" => crate::tui_text!("base URL", "地址"),
        "auth_token" => crate::tui_text!("secret", "密钥"),
        "provider" => crate::tui_text!("provider", "提供商"),
        "provider_type" => crate::tui_text!("provider type", "提供商类型"),
        "account" => crate::tui_text!("account", "账号"),
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

pub(crate) fn auth_mode_label(platform: Platform, value: &str) -> &'static str {
    CreateAuthMode::parse(platform, value).label()
}

pub(crate) fn choice_value_label(key: &str, value: &str) -> Option<&'static str> {
    match (key, value) {
        ("provider_type", "official_relay") => Some(crate::tui_text!(
            "official_relay",
            "官方中转 (official_relay)"
        )),
        ("provider_type", "third_party_model") => Some(crate::tui_text!(
            "third_party_model",
            "第三方模型 (third_party_model)"
        )),
        _ => None,
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
