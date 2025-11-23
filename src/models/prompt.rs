use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptPreset {
    pub name: String,
    pub description: Option<String>,
    pub target_file: PromptTarget,
    pub content: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PromptTarget {
    Claude, // CLAUDE.md
    Agents, // AGENTS.md
    Gemini, // GEMINI.md
}

impl PromptTarget {
    pub fn filename(&self) -> &str {
        match self {
            PromptTarget::Claude => "CLAUDE.md",
            PromptTarget::Agents => "AGENTS.md",
            PromptTarget::Gemini => "GEMINI.md",
        }
    }
}

impl std::fmt::Display for PromptTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PromptTarget::Claude => write!(f, "claude"),
            PromptTarget::Agents => write!(f, "agents"),
            PromptTarget::Gemini => write!(f, "gemini"),
        }
    }
}

impl std::str::FromStr for PromptTarget {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "claude" => Ok(PromptTarget::Claude),
            "agents" => Ok(PromptTarget::Agents),
            "gemini" => Ok(PromptTarget::Gemini),
            _ => Err(format!("Invalid prompt target: {}", s)),
        }
    }
}
