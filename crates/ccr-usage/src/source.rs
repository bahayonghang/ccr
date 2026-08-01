use serde::{Deserialize, Serialize};

#[derive(
    Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    #[serde(alias = "claude-code", alias = "claude code")]
    Claude,
    #[default]
    #[serde(alias = "openai-codex", alias = "openai codex")]
    Codex,
    #[serde(
        alias = "gemini",
        alias = "gemini-cli",
        alias = "gemini cli",
        alias = "google-gemini",
        alias = "google gemini"
    )]
    Antigravity,
    #[serde(alias = "open-code", alias = "open code")]
    Opencode,
    #[serde(alias = "kimi-code", alias = "kimi code")]
    KimiCode,
    #[serde(alias = "oh-my-pi", alias = "oh my pi", alias = "omp")]
    Pi,
    #[serde(alias = "grok-build", alias = "grok build")]
    Grok,
}

impl SourceKind {
    pub const ALL: [Self; 7] = [
        Self::Claude,
        Self::Codex,
        Self::Opencode,
        Self::Antigravity,
        Self::KimiCode,
        Self::Pi,
        Self::Grok,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::Antigravity => "antigravity",
            Self::Opencode => "opencode",
            Self::KimiCode => "kimi_code",
            Self::Pi => "pi",
            Self::Grok => "grok",
        }
    }

    /// Schema 13 renamed persisted `gemini` rows to `antigravity`.
    pub fn storage_key(self, schema_version: i64) -> &'static str {
        if self == Self::Antigravity && schema_version < 13 {
            "gemini"
        } else {
            self.as_str()
        }
    }

    pub fn parse_id(raw: &str) -> Option<Self> {
        match raw.trim().to_lowercase().as_str() {
            "claude" | "claude-code" | "claude code" => Some(Self::Claude),
            "codex" | "openai-codex" | "openai codex" => Some(Self::Codex),
            "antigravity" | "gemini" | "gemini-cli" | "gemini cli" | "google-gemini"
            | "google gemini" => Some(Self::Antigravity),
            "opencode" | "open-code" | "open code" => Some(Self::Opencode),
            "kimi_code" | "kimi-code" | "kimi code" => Some(Self::KimiCode),
            "pi" | "oh-my-pi" | "oh my pi" | "omp" => Some(Self::Pi),
            "grok" | "grok-build" | "grok build" => Some(Self::Grok),
            _ => None,
        }
    }
}

impl std::fmt::Display for SourceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

pub fn parse_source_filter(raw: &str) -> Option<SourceKind> {
    match raw.trim().to_lowercase().as_str() {
        "" | "all" | "*" => None,
        value => SourceKind::parse_id(value),
    }
}

pub fn canonical_source_id(raw: Option<&str>) -> Option<String> {
    raw.and_then(parse_source_filter)
        .map(|source| source.as_str().to_string())
}

pub fn platform_scope_label(raw: Option<&str>) -> String {
    canonical_source_id(raw).unwrap_or_else(|| "all".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_filter_accepts_ccr_aliases() {
        assert_eq!(
            canonical_source_id(Some("Claude Code")).as_deref(),
            Some("claude")
        );
        assert_eq!(
            canonical_source_id(Some("openai-codex")).as_deref(),
            Some("codex")
        );
        assert_eq!(
            canonical_source_id(Some("gemini-cli")).as_deref(),
            Some("antigravity")
        );
        assert_eq!(
            canonical_source_id(Some("Open Code")).as_deref(),
            Some("opencode")
        );
        assert_eq!(canonical_source_id(Some("all")), None);
        assert_eq!(
            canonical_source_id(Some("Kimi Code")).as_deref(),
            Some("kimi_code")
        );
        assert_eq!(canonical_source_id(Some("oh-my-pi")).as_deref(), Some("pi"));
        assert_eq!(
            canonical_source_id(Some("Grok Build")).as_deref(),
            Some("grok")
        );
    }

    #[test]
    fn antigravity_storage_key_tracks_schema_13_cutover() {
        assert_eq!(SourceKind::Antigravity.storage_key(10), "gemini");
        assert_eq!(SourceKind::Antigravity.storage_key(12), "gemini");
        assert_eq!(SourceKind::Antigravity.storage_key(13), "antigravity");
        assert_eq!(SourceKind::Antigravity.storage_key(19), "antigravity");
    }

    #[test]
    fn serde_accepts_all_canonical_source_ids() {
        for source in SourceKind::ALL {
            let encoded = format!("\"{}\"", source.as_str());
            let decoded: SourceKind =
                serde_json::from_str(&encoded).expect("canonical source id should deserialize");
            assert_eq!(decoded, source);
            assert_eq!(
                serde_json::to_string(&source).expect("source should serialize"),
                encoded
            );
        }
    }
}
