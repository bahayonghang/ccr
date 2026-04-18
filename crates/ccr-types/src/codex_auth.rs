//! Codex Auth Types
//!
//! Shared types for Codex authentication management.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Value, json};

/// TUI login state
///
/// Represents the current login status for Codex authentication.
/// Unknown state payloads are preserved for forward compatibility.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoginState {
    /// Not logged in (auth.json does not exist)
    NotLoggedIn,
    /// Logged in via OAuth but not saved to registry
    LoggedInUnsaved,
    /// Logged in via OAuth and saved (account name)
    LoggedInSaved(String),
    /// Active via OPENAI_API_KEY (not an OAuth login)
    ApiKeyActive,
    /// Active via provider environment key (not an OAuth login)
    ProviderKeyActive { env_key: String },
    /// Unknown/forward-compatible state from newer backend
    Unknown { type_name: String, raw: Value },
}

impl Serialize for LoginState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = match self {
            LoginState::NotLoggedIn => json!({ "type": "NotLoggedIn" }),
            LoginState::LoggedInUnsaved => json!({ "type": "LoggedInUnsaved" }),
            LoginState::LoggedInSaved(account_name) => {
                json!({ "type": "LoggedInSaved", "account_name": account_name })
            }
            LoginState::ApiKeyActive => json!({ "type": "ApiKeyActive" }),
            LoginState::ProviderKeyActive { env_key } => {
                json!({ "type": "ProviderKeyActive", "env_key": env_key })
            }
            LoginState::Unknown { raw, .. } => raw.clone(),
        };
        value.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for LoginState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        let map = value
            .as_object()
            .ok_or_else(|| {
                serde::de::Error::custom("invalid type for LoginState: expected object")
            })?
            .clone();

        let type_name = map
            .get("type")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                serde::de::Error::custom("invalid LoginState: expected string field `type`")
            })?
            .to_string();

        match type_name.as_str() {
            "NotLoggedIn" => Ok(LoginState::NotLoggedIn),
            "LoggedInUnsaved" => Ok(LoginState::LoggedInUnsaved),
            "LoggedInSaved" => {
                let account_name = map
                    .get("account_name")
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        serde::de::Error::custom(
                            "invalid LoginState::LoggedInSaved: expected string field `account_name`",
                        )
                    })?
                    .to_string();
                Ok(LoginState::LoggedInSaved(account_name))
            }
            "ApiKeyActive" => Ok(LoginState::ApiKeyActive),
            "ProviderKeyActive" => {
                let env_key = map
                    .get("env_key")
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        serde::de::Error::custom(
                            "invalid LoginState::ProviderKeyActive: expected string field `env_key`",
                        )
                    })?
                    .to_string();
                Ok(LoginState::ProviderKeyActive { env_key })
            }
            _ => Ok(LoginState::Unknown {
                type_name,
                raw: Value::Object(map),
            }),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_login_state_serialization() {
        // NotLoggedIn
        let json = serde_json::to_string(&LoginState::NotLoggedIn).unwrap();
        assert!(json.contains("\"type\":\"NotLoggedIn\""));

        // LoggedInUnsaved
        let json = serde_json::to_string(&LoginState::LoggedInUnsaved).unwrap();
        assert!(json.contains("\"type\":\"LoggedInUnsaved\""));

        // LoggedInSaved
        let json = serde_json::to_string(&LoginState::LoggedInSaved("test".to_string())).unwrap();
        assert!(json.contains("\"type\":\"LoggedInSaved\""));
        assert!(json.contains("\"account_name\":\"test\""));

        // ApiKeyActive
        let json = serde_json::to_string(&LoginState::ApiKeyActive).unwrap();
        assert!(json.contains("\"type\":\"ApiKeyActive\""));

        // ProviderKeyActive
        let json = serde_json::to_string(&LoginState::ProviderKeyActive {
            env_key: "MISTRAL_API_KEY".to_string(),
        })
        .unwrap();
        assert!(json.contains("\"type\":\"ProviderKeyActive\""));
        assert!(json.contains("\"env_key\":\"MISTRAL_API_KEY\""));
    }

    #[test]
    fn test_login_state_deserialization() {
        let state: LoginState = serde_json::from_str(r#"{"type":"NotLoggedIn"}"#).unwrap();
        assert_eq!(state, LoginState::NotLoggedIn);

        let state: LoginState =
            serde_json::from_str(r#"{"type":"LoggedInSaved","account_name":"myaccount"}"#).unwrap();
        assert_eq!(state, LoginState::LoggedInSaved("myaccount".to_string()));

        let state: LoginState = serde_json::from_str(r#"{"type":"ApiKeyActive"}"#).unwrap();
        assert_eq!(state, LoginState::ApiKeyActive);

        let state: LoginState =
            serde_json::from_str(r#"{"type":"ProviderKeyActive","env_key":"MISTRAL_API_KEY"}"#)
                .unwrap();
        assert_eq!(
            state,
            LoginState::ProviderKeyActive {
                env_key: "MISTRAL_API_KEY".to_string()
            }
        );
    }

    #[test]
    fn test_login_state_unknown_deserialization_preserves_raw() {
        let raw = r#"{"type":"LoggedInFromCloud","account_name":"cloud","region":"us-east-1"}"#;
        let state: LoginState = serde_json::from_str(raw).unwrap();
        match state {
            LoginState::Unknown { type_name, raw } => {
                assert_eq!(type_name, "LoggedInFromCloud");
                assert_eq!(raw.get("region").and_then(Value::as_str), Some("us-east-1"));
            }
            _ => panic!("expected unknown login state"),
        }
    }

    #[test]
    fn test_login_state_unknown_serialization_passthrough() {
        let state = LoginState::Unknown {
            type_name: "LoggedInFromCloud".to_string(),
            raw: json!({
                "type": "LoggedInFromCloud",
                "account_name": "cloud",
                "region": "us-east-1"
            }),
        };
        let serialized = serde_json::to_string(&state).unwrap();
        assert!(serialized.contains("\"type\":\"LoggedInFromCloud\""));
        assert!(serialized.contains("\"region\":\"us-east-1\""));
    }
}
