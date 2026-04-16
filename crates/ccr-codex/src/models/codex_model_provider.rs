use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexModelProviderApiKey {
    pub id: String,
    pub name: String,
    pub api_key: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexModelProviderRecord {
    pub id: String,
    pub name: String,
    pub base_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key_url: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub api_keys: Vec<CodexModelProviderApiKey>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct CodexModelProviderStore {
    #[serde(default)]
    pub version: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub providers: Vec<CodexModelProviderRecord>,
}
