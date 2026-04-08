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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum SkillInstallStrategy {
    #[default]
    ManagedCopy,
    DirectCopy,
    DirectCli,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shared_dir_group: Option<String>,
    #[serde(default)]
    pub install_strategy: SkillInstallStrategy,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub npx_agent_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub sort_order: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillPlatformSummary {
    pub id: String,
    pub display_name: String,
    pub global_skills_dir: String,
    pub detected: bool,
    pub installed_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shared_dir_group: Option<String>,
    #[serde(default)]
    pub install_strategy: SkillInstallStrategy,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub npx_agent_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub sort_order: usize,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SkillTargetStatus {
    Ok,
    Pending,
    Error,
    Missing,
    Stale,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillTargetRecord {
    pub id: String,
    pub platform_id: String,
    pub platform_name: String,
    pub target_path: String,
    pub sync_mode: SkillInstallMode,
    pub status: SkillTargetStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub synced_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    pub is_primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillLifecycleSummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_revision: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_synced_at: Option<i64>,
    pub has_errors: bool,
    pub target_count: usize,
    pub healthy_target_count: usize,
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
    pub targets: Vec<SkillTargetRecord>,
    #[serde(default)]
    pub lifecycle: SkillLifecycleSummary,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillFileEntry {
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillFileContent {
    pub skill_id: String,
    pub installation_id: String,
    pub path: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsOnboardingCandidate {
    pub skill_id: String,
    pub name: String,
    #[serde(default)]
    pub platform_ids: Vec<String>,
    #[serde(default)]
    pub installation_ids: Vec<String>,
    #[serde(default)]
    pub installation_paths: Vec<String>,
    pub reason: String,
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
    pub selected_skills: Vec<String>,
    #[serde(default)]
    pub target_platforms: Vec<String>,
    #[serde(default)]
    pub force: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub copy_mode: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_mode: Option<bool>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpxPlatformSupport {
    pub platform_id: String,
    pub platform_name: String,
    pub supported: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsNpxCapabilities {
    #[serde(flatten)]
    pub status: NpxStatus,
    #[serde(default)]
    pub supported_platforms: Vec<NpxPlatformSupport>,
    #[serde(default)]
    pub supported_flags: Vec<String>,
    pub package_manager: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsInstallReviewSource {
    pub source_kind: String,
    pub source_ref: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_skill_id: Option<String>,
    pub resolved_name: String,
    pub resolved_dir_name: String,
    pub origin: SkillOrigin,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsInstallReviewTarget {
    pub platform_id: String,
    pub platform_name: String,
    pub detected: bool,
    pub target_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shared_dir_group: Option<String>,
    #[serde(default)]
    pub install_strategy: SkillInstallStrategy,
    pub direct_npx_supported: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub npx_agent_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsInstallCommandPreview {
    pub kind: String,
    pub label: String,
    pub command: String,
    #[serde(default)]
    pub platforms: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsInstallReviewResponse {
    pub source: SkillsInstallReviewSource,
    #[serde(default)]
    pub targets: Vec<SkillsInstallReviewTarget>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub command_previews: Vec<SkillsInstallCommandPreview>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub npx: Option<SkillsNpxCapabilities>,
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
