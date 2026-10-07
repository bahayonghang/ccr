//! Local OAuth identity association. JWT signatures are not verified here.

use crate::models::{CodexAuthJson, CodexAuthTokens};
use serde_json::Value;
use std::fmt;

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct OAuthIdentity {
    user_id: String,
    account_id: String,
}

impl fmt::Debug for OAuthIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("OAuthIdentity([REDACTED])")
    }
}

impl OAuthIdentity {
    pub(crate) fn key(&self) -> String {
        format!("{}::{}", self.user_id, self.account_id)
    }

    pub(crate) fn from_tokens(tokens: &CodexAuthTokens) -> Option<Self> {
        let claims: Vec<Value> = [tokens.id_token.as_deref(), tokens.access_token.as_deref()]
            .into_iter()
            .flatten()
            .filter_map(decode_claims)
            .collect();
        let mut user_id = None;
        for claim in &claims {
            for scope in [Some(claim), claim.get("https://api.openai.com/auth")]
                .into_iter()
                .flatten()
            {
                if let Some(value) = nonempty(scope.get("chatgpt_user_id").and_then(Value::as_str))
                {
                    if user_id.is_some_and(|old| old != value) {
                        return None;
                    }
                    user_id = Some(value);
                }
            }
        }
        let account_id = token_account_id(tokens);
        Some(Self {
            user_id: user_id?.to_string(),
            account_id: account_id?,
        })
    }
}

pub(crate) fn token_account_id(tokens: &CodexAuthTokens) -> Option<String> {
    nonempty(tokens.account_id.as_deref())
        .map(str::to_string)
        .or_else(|| {
            [tokens.access_token.as_deref(), tokens.id_token.as_deref()]
                .into_iter()
                .flatten()
                .filter_map(decode_claims)
                .find_map(|claim| account_claim(&claim).map(str::to_string))
        })
}

/// Import rejects contradictions that ordinary incomplete-identity reads retain as unknown.
pub(crate) fn has_conflicting_claims(tokens: &CodexAuthTokens) -> bool {
    let claims: Vec<Value> = [tokens.id_token.as_deref(), tokens.access_token.as_deref()]
        .into_iter()
        .flatten()
        .filter_map(decode_claims)
        .collect();
    let mut user = None;
    let mut account = nonempty(tokens.account_id.as_deref());
    for claim in &claims {
        for scope in [Some(claim), claim.get("https://api.openai.com/auth")]
            .into_iter()
            .flatten()
        {
            if let Some(value) = nonempty(scope.get("chatgpt_user_id").and_then(Value::as_str)) {
                if user.is_some_and(|previous| previous != value) {
                    return true;
                }
                user = Some(value);
            }
            for key in ["chatgpt_account_id", "account_id"] {
                if let Some(value) = nonempty(scope.get(key).and_then(Value::as_str)) {
                    if account.is_some_and(|previous| previous != value) {
                        return true;
                    }
                    account = Some(value);
                }
            }
        }
    }
    false
}

pub(crate) fn identity_from_auth(auth: &CodexAuthJson) -> Option<OAuthIdentity> {
    if nonempty(auth.openai_api_key.as_deref()).is_some() {
        return None;
    }
    OAuthIdentity::from_tokens(auth.tokens.as_ref()?)
}

fn nonempty(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

fn decode_claims(token: &str) -> Option<Value> {
    let mut parts = token.trim().split('.');
    parts.next()?;
    let payload = parts.next()?;
    parts.next()?;
    if parts.next().is_some() {
        return None;
    }
    serde_json::from_slice(&crate::utils::decode_base64url(payload)?).ok()
}

fn account_claim(claim: &Value) -> Option<&str> {
    [Some(claim), claim.get("https://api.openai.com/auth")]
        .into_iter()
        .flatten()
        .find_map(|scope| {
            ["chatgpt_account_id", "account_id"]
                .into_iter()
                .find_map(|key| nonempty(scope.get(key).and_then(Value::as_str)))
        })
}

#[cfg(test)]
pub(crate) fn test_jwt(claims: Value) -> String {
    use base64::Engine;
    format!(
        "header.{}.signature",
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(claims.to_string())
    )
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use serde_json::json;

    fn tokens(id: Value, access: Value, account: Option<&str>) -> CodexAuthTokens {
        CodexAuthTokens {
            id_token: Some(test_jwt(id)),
            access_token: Some(test_jwt(access)),
            refresh_token: Some("synthetic-refresh".into()),
            account_id: account.map(str::to_string),
        }
    }

    #[test]
    fn complete_identity_uses_both_token_and_claim_scopes() {
        for scope in [
            json!({"chatgpt_user_id":" user-a ","chatgpt_account_id":" workspace "}),
            json!({"https://api.openai.com/auth":{"chatgpt_user_id":"user-a","chatgpt_account_id":"workspace"}}),
        ] {
            for (id, access) in [(scope.clone(), json!({})), (json!({}), scope)] {
                let identity = OAuthIdentity::from_tokens(&tokens(id, access, None)).unwrap();
                assert_eq!(identity.key(), "user-a::workspace");
                assert_eq!(format!("{identity:?}"), "OAuthIdentity([REDACTED])");
            }
        }
        let identity = OAuthIdentity::from_tokens(&tokens(
            json!({"chatgpt_user_id":"u"}),
            json!({"chatgpt_account_id":"jwt-workspace"}),
            Some("explicit"),
        ))
        .unwrap();
        assert_eq!(identity.key(), "u::explicit");
    }

    #[test]
    fn incomplete_or_conflicting_claims_have_no_identity() {
        for claim in [
            json!({}),
            json!({"sub":"u","email":"u@example.test"}),
            json!({"chatgpt_user_id":" "}),
            json!({"chatgpt_user_id":42}),
            json!({"chatgpt_user_id":"u","https://api.openai.com/auth":{"chatgpt_user_id":"v"}}),
        ] {
            assert!(
                OAuthIdentity::from_tokens(&tokens(claim, json!({}), Some("workspace"))).is_none()
            );
        }
        assert!(
            OAuthIdentity::from_tokens(&tokens(
                json!({"chatgpt_user_id":"u"}),
                json!({"chatgpt_user_id":"v"}),
                Some("workspace")
            ))
            .is_none()
        );
        assert!(
            OAuthIdentity::from_tokens(&tokens(json!({"chatgpt_user_id":"u"}), json!({}), None))
                .is_none()
        );
        for malformed in ["opaque", "a.!.c", "a.e30.c", "a.e30.c.d"] {
            let mut value = tokens(json!({}), json!({}), Some("workspace"));
            value.id_token = Some(malformed.into());
            assert!(OAuthIdentity::from_tokens(&value).is_none());
        }
    }
}
