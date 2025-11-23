use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub description: Option<String>,
    pub path: String,
    pub instruction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillRepository {
    pub name: String,
    pub url: String,
    pub branch: String,
    pub description: Option<String>,
}

impl Default for SkillRepository {
    fn default() -> Self {
        Self {
            name: "official".to_string(),
            url: "https://github.com/anthropics/anthropic-quickstarts".to_string(),
            branch: "main".to_string(),
            description: Some("Anthropic Official Quickstarts".to_string()),
        }
    }
}
