use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::CredentialStoreKind;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeMatchStatus {
    Match,
    Missing,
    Mismatch,
    NotApplicable,
    Unsupported,
}

impl RuntimeMatchStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Match => "match",
            Self::Missing => "missing",
            Self::Mismatch => "mismatch",
            Self::NotApplicable => "not_applicable",
            Self::Unsupported => "unsupported",
        }
    }

    pub fn is_drift(self) -> bool {
        matches!(self, Self::Missing | Self::Mismatch)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProviderAuthValidity {
    NotChecked,
}

impl ProviderAuthValidity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NotChecked => "not_checked",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CodexRuntimeAuthSource {
    AuthJsonOpenAiApiKey,
    AuthJsonChatgptTokens,
    Environment { variable: String },
    EnvironmentInvalid,
    KeyringUnreadable,
    AutoUnreadable,
    None,
}

impl CodexRuntimeAuthSource {
    pub fn label(&self) -> String {
        match self {
            Self::AuthJsonOpenAiApiKey => "auth_json:OPENAI_API_KEY".to_string(),
            Self::AuthJsonChatgptTokens => "auth_json:tokens".to_string(),
            Self::Environment { variable } => format!("env:{variable}"),
            Self::EnvironmentInvalid => "env:invalid_name".to_string(),
            Self::KeyringUnreadable => "keyring:unreadable".to_string(),
            Self::AutoUnreadable => "auto:unreadable".to_string(),
            Self::None => "none".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexEnvironmentPresence {
    pub variable: String,
    pub is_set: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CodexRuntimeIssue {
    RegistryPointerMissing,
    ProfilesPointerMissing,
    ProfilePointerMismatch,
    ProfileNotFound { profile: String },
    RouteMismatch,
    CredentialMissing,
    CredentialMismatch,
    CredentialUnsupported,
    EnvironmentOverride { variable: String },
    CodexHomeMismatch,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexRuntimeDiagnostic {
    pub registry_path: PathBuf,
    pub profiles_path: PathBuf,
    pub config_path: PathBuf,
    pub auth_path: PathBuf,
    pub registry_profile: Option<String>,
    pub profiles_file_profile: Option<String>,
    pub resolved_profile: Option<String>,
    pub runtime_provider_id: Option<String>,
    pub runtime_provider_name: Option<String>,
    pub base_url: Option<String>,
    pub wire_api: Option<String>,
    pub credential_store: CredentialStoreKind,
    pub auth_source: CodexRuntimeAuthSource,
    pub profile_status: RuntimeMatchStatus,
    pub route_status: RuntimeMatchStatus,
    pub credential_status: RuntimeMatchStatus,
    pub provider_auth_validity: ProviderAuthValidity,
    pub environment: Vec<CodexEnvironmentPresence>,
    pub issues: Vec<CodexRuntimeIssue>,
    pub repairable: bool,
}

impl CodexRuntimeDiagnostic {
    pub fn runtime_consistency(&self) -> RuntimeMatchStatus {
        let statuses = [
            self.profile_status,
            self.route_status,
            self.credential_status,
        ];

        if statuses.contains(&RuntimeMatchStatus::Mismatch) {
            RuntimeMatchStatus::Mismatch
        } else if statuses.contains(&RuntimeMatchStatus::Missing) {
            RuntimeMatchStatus::Missing
        } else if statuses.contains(&RuntimeMatchStatus::Unsupported) {
            RuntimeMatchStatus::Unsupported
        } else if self.profile_status == RuntimeMatchStatus::NotApplicable {
            RuntimeMatchStatus::NotApplicable
        } else {
            RuntimeMatchStatus::Match
        }
    }

    pub fn has_local_drift(&self) -> bool {
        self.profile_status.is_drift()
            || self.route_status.is_drift()
            || self.credential_status.is_drift()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_consistency_prioritizes_mismatch_then_missing() {
        let mut diagnostic = empty_diagnostic();
        assert_eq!(diagnostic.runtime_consistency(), RuntimeMatchStatus::Match);

        diagnostic.credential_status = RuntimeMatchStatus::Missing;
        assert_eq!(
            diagnostic.runtime_consistency(),
            RuntimeMatchStatus::Missing
        );

        diagnostic.route_status = RuntimeMatchStatus::Mismatch;
        assert_eq!(
            diagnostic.runtime_consistency(),
            RuntimeMatchStatus::Mismatch
        );
    }

    fn empty_diagnostic() -> CodexRuntimeDiagnostic {
        CodexRuntimeDiagnostic {
            registry_path: PathBuf::from("registry.toml"),
            profiles_path: PathBuf::from("profiles.toml"),
            config_path: PathBuf::from("config.toml"),
            auth_path: PathBuf::from("auth.json"),
            registry_profile: Some("test".to_string()),
            profiles_file_profile: Some("test".to_string()),
            resolved_profile: Some("test".to_string()),
            runtime_provider_id: Some("custom".to_string()),
            runtime_provider_name: Some("test".to_string()),
            base_url: Some("https://example.com".to_string()),
            wire_api: Some("responses".to_string()),
            credential_store: CredentialStoreKind::File,
            auth_source: CodexRuntimeAuthSource::AuthJsonOpenAiApiKey,
            profile_status: RuntimeMatchStatus::Match,
            route_status: RuntimeMatchStatus::Match,
            credential_status: RuntimeMatchStatus::Match,
            provider_auth_validity: ProviderAuthValidity::NotChecked,
            environment: Vec::new(),
            issues: Vec::new(),
            repairable: false,
        }
    }
}
