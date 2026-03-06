//! Codex Auth Types
//!
//! Shared types for Codex authentication management.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Value, json};

const TOKEN_UNKNOWN_LABEL: &str = "Unknown";

/// Token freshness status
///
/// Indicates how recently the token was refreshed.
/// Serializes to CamelCase to match frontend TypeScript types.
/// Unknown values are preserved for forward compatibility.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenFreshness {
    /// Fresh (< 1 day)
    Fresh,
    /// Stale (1-7 days)
    Stale,
    /// Old (> 7 days)
    Old,
    /// Unknown or unrecognized value (keeps raw string)
    Unknown(String),
}

impl TokenFreshness {
    /// Canonical unknown value used by current system.
    pub fn unknown() -> Self {
        Self::Unknown(TOKEN_UNKNOWN_LABEL.to_string())
    }

    /// Get display icon
    pub fn icon(&self) -> &'static str {
        match self {
            TokenFreshness::Fresh => "🟢",
            TokenFreshness::Stale => "🟡",
            TokenFreshness::Old => "🔴",
            TokenFreshness::Unknown(_) => "⚪",
        }
    }

    /// Get description text
    pub fn description(&self) -> &'static str {
        match self {
            TokenFreshness::Fresh => "Token 状态良好",
            TokenFreshness::Stale => "Token 可能需要刷新",
            TokenFreshness::Old => "Token 可能已过期，建议重新登录",
            TokenFreshness::Unknown(_) => "无法确定 Token 状态",
        }
    }
}

impl Serialize for TokenFreshness {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            TokenFreshness::Fresh => serializer.serialize_str("Fresh"),
            TokenFreshness::Stale => serializer.serialize_str("Stale"),
            TokenFreshness::Old => serializer.serialize_str("Old"),
            TokenFreshness::Unknown(raw) => serializer.serialize_str(raw),
        }
    }
}

impl<'de> Deserialize<'de> for TokenFreshness {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        match raw.as_str() {
            "Fresh" => Ok(TokenFreshness::Fresh),
            "Stale" => Ok(TokenFreshness::Stale),
            "Old" => Ok(TokenFreshness::Old),
            _ => Ok(TokenFreshness::Unknown(raw)),
        }
    }
}

/// TUI login state
///
/// Represents the current login status for Codex authentication.
/// Unknown state payloads are preserved for forward compatibility.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoginState {
    /// Not logged in (auth.json does not exist)
    NotLoggedIn,
    /// Logged in but not saved
    LoggedInUnsaved,
    /// Logged in and saved (account name)
    LoggedInSaved(String),
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
    fn test_token_freshness_serialization() {
        // Test CamelCase serialization
        assert_eq!(
            serde_json::to_string(&TokenFreshness::Fresh).unwrap(),
            "\"Fresh\""
        );
        assert_eq!(
            serde_json::to_string(&TokenFreshness::Stale).unwrap(),
            "\"Stale\""
        );
        assert_eq!(
            serde_json::to_string(&TokenFreshness::Old).unwrap(),
            "\"Old\""
        );
        assert_eq!(
            serde_json::to_string(&TokenFreshness::unknown()).unwrap(),
            "\"Unknown\""
        );
        assert_eq!(
            serde_json::to_string(&TokenFreshness::Unknown("VeryFresh".to_string())).unwrap(),
            "\"VeryFresh\""
        );
    }

    #[test]
    fn test_token_freshness_deserialization() {
        assert_eq!(
            serde_json::from_str::<TokenFreshness>("\"Fresh\"").unwrap(),
            TokenFreshness::Fresh
        );
        assert_eq!(
            serde_json::from_str::<TokenFreshness>("\"Stale\"").unwrap(),
            TokenFreshness::Stale
        );
        assert_eq!(
            serde_json::from_str::<TokenFreshness>("\"VeryFresh\"").unwrap(),
            TokenFreshness::Unknown("VeryFresh".to_string())
        );
    }

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
    }

    #[test]
    fn test_login_state_deserialization() {
        let state: LoginState = serde_json::from_str(r#"{"type":"NotLoggedIn"}"#).unwrap();
        assert_eq!(state, LoginState::NotLoggedIn);

        let state: LoginState =
            serde_json::from_str(r#"{"type":"LoggedInSaved","account_name":"myaccount"}"#).unwrap();
        assert_eq!(state, LoginState::LoggedInSaved("myaccount".to_string()));
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

    #[test]
    fn test_token_freshness_icon() {
        assert_eq!(TokenFreshness::Fresh.icon(), "🟢");
        assert_eq!(TokenFreshness::Stale.icon(), "🟡");
        assert_eq!(TokenFreshness::Old.icon(), "🔴");
        assert_eq!(TokenFreshness::unknown().icon(), "⚪");
    }

    #[test]
    fn test_token_freshness_description() {
        assert!(!TokenFreshness::Fresh.description().is_empty());
        assert!(!TokenFreshness::Stale.description().is_empty());
        assert!(!TokenFreshness::Old.description().is_empty());
        assert!(!TokenFreshness::unknown().description().is_empty());
    }
}
