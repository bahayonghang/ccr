use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "lowercase")]
pub enum SourceKind {
    Claude,
    #[default]
    Codex,
    Gemini,
    Opencode,
}

impl SourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::Gemini => "gemini",
            Self::Opencode => "opencode",
        }
    }

    pub fn parse_id(raw: &str) -> Option<Self> {
        match raw.trim().to_lowercase().as_str() {
            "claude" | "claude-code" | "claude code" => Some(Self::Claude),
            "codex" | "openai-codex" | "openai codex" => Some(Self::Codex),
            "gemini" | "gemini-cli" | "gemini cli" | "google-gemini" | "google gemini" => {
                Some(Self::Gemini)
            }
            "opencode" | "open-code" | "open code" => Some(Self::Opencode),
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
            Some("gemini")
        );
        assert_eq!(
            canonical_source_id(Some("Open Code")).as_deref(),
            Some("opencode")
        );
        assert_eq!(canonical_source_id(Some("all")), None);
    }
}
