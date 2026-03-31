use serde::{Deserialize, Serialize};

use crate::models::skill::SkillMetadata;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum SkillOrigin {
    Marketplace,
    Github,
    Repo,
    Local,
    Npx,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum SkillInstallMode {
    #[default]
    Copy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SkillSourceType {
    Git,
    Local,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SkillSourceHealth {
    Ok,
    Error,
    Missing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillPlatformConfig {
    pub id: String,
    pub display_name: String,
    pub relative_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillPlatformSummary {
    pub id: String,
    pub display_name: String,
    pub global_skills_dir: String,
    pub detected: bool,
    pub installed_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillInstallMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install_group_id: Option<String>,
    #[serde(default)]
    pub origin: SkillOrigin,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_repo_id: Option<String>,
    #[serde(default)]
    pub install_mode: SkillInstallMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub installed_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInstallationRecord {
    pub id: String,
    pub platform_id: String,
    pub platform_name: String,
    pub install_path: String,
    pub install_mode: SkillInstallMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub installed_at: Option<i64>,
    pub is_primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillRecord {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(default)]
    pub origin: SkillOrigin,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_ref: Option<String>,
    pub install_count: usize,
    #[serde(default)]
    pub installations: Vec<SkillInstallationRecord>,
    #[serde(default)]
    pub editable_installations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsInventoryResponse {
    #[serde(default)]
    pub skills: Vec<SkillRecord>,
    #[serde(default)]
    pub platforms: Vec<SkillPlatformSummary>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSourceRecord {
    pub id: String,
    #[serde(rename = "type")]
    pub source_type: SkillSourceType,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub location: String,
    pub skills_root: String,
    pub skill_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_synced_at: Option<String>,
    pub health: SkillSourceHealth,
    #[serde(default)]
    pub skills: Vec<SkillSourceSkillRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSourceSkillRecord {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub install_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillContent {
    pub skill_id: String,
    pub installation_id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub raw: String,
    pub content: String,
    pub skill_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillsInventoryQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<SkillOrigin>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub q: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsInstallRequest {
    pub source_kind: String,
    pub source_ref: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_skill_id: Option<String>,
    #[serde(default)]
    pub target_platforms: Vec<String>,
    #[serde(default)]
    pub force: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsSyncRequest {
    pub skill_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub installation_id: Option<String>,
    #[serde(default)]
    pub target_platforms: Vec<String>,
    #[serde(default)]
    pub force: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillOperationResult {
    pub agent: String,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillOperationResponse {
    #[serde(default)]
    pub results: Vec<SkillOperationResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillsSourceManifest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skills_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceSkill {
    pub package: String,
    pub owner: String,
    pub repo: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill: Option<String>,
    pub skills_sh_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author_avatar: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stars: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceListResponse {
    #[serde(default)]
    pub items: Vec<MarketplaceSkill>,
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
    pub cached: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpxStatus {
    pub available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SkillDescriptor {
    #[allow(dead_code)]
    pub skill_dir: std::path::PathBuf,
    pub name: String,
    pub description: Option<String>,
    pub metadata: SkillMetadata,
    pub raw: String,
    pub install_meta: SkillInstallMeta,
}
