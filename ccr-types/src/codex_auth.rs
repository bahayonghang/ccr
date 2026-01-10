//! Codex Auth Types
//!
//! Shared types for Codex authentication management.

use serde::{Deserialize, Serialize};

/// Token freshness status
///
/// Indicates how recently the token was refreshed.
/// Serializes to CamelCase to match frontend TypeScript types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenFreshness {
    /// Fresh (< 1 day)
    Fresh,
    /// Stale (1-7 days)
    Stale,
    /// Old (> 7 days)
    Old,
    /// Unknown (cannot parse time)
    Unknown,
}

impl TokenFreshness {
    /// Get display icon
    pub fn icon(&self) -> &'static str {
        match self {
            TokenFreshness::Fresh => "🟢",
            TokenFreshness::Stale => "🟡",
            TokenFreshness::Old => "🔴",
            TokenFreshness::Unknown => "⚪",
        }
    }

    /// Get description text
    pub fn description(&self) -> &'static str {
        match self {
            TokenFreshness::Fresh => "Token 状态良好",
            TokenFreshness::Stale => "Token 可能需要刷新",
            TokenFreshness::Old => "Token 可能已过期，建议重新登录",
            TokenFreshness::Unknown => "无法确定 Token 状态",
        }
    }
}

/// TUI login state
///
/// Represents the current login status for Codex authentication.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "account_name")]
pub enum LoginState {
    /// Not logged in (auth.json does not exist)
    NotLoggedIn,
    /// Logged in but not saved
    LoggedInUnsaved,
    /// Logged in and saved (account name)
    LoggedInSaved(String),
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
            serde_json::to_string(&TokenFreshness::Unknown).unwrap(),
            "\"Unknown\""
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
    fn test_token_freshness_icon() {
        assert_eq!(TokenFreshness::Fresh.icon(), "🟢");
        assert_eq!(TokenFreshness::Stale.icon(), "🟡");
        assert_eq!(TokenFreshness::Old.icon(), "🔴");
        assert_eq!(TokenFreshness::Unknown.icon(), "⚪");
    }

    #[test]
    fn test_token_freshness_description() {
        assert!(!TokenFreshness::Fresh.description().is_empty());
        assert!(!TokenFreshness::Stale.description().is_empty());
        assert!(!TokenFreshness::Old.description().is_empty());
        assert!(!TokenFreshness::Unknown.description().is_empty());
    }
}
