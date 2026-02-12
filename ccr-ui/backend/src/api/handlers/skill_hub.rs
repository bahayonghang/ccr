use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use axum::{
    Json,
    extract::{Path as AxumPath, Query},
    response::IntoResponse,
};
use once_cell::sync::Lazy;
use reqwest::Url;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::database;
use crate::database::repositories::log_repo;
use crate::models::api::ApiResponse;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillHubMarketplaceItem {
    pub package: String,
    pub owner: String,
    pub repo: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill: Option<String>,
    pub skills_sh_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillHubMarketplaceResponse {
    pub items: Vec<SkillHubMarketplaceItem>,
    pub total: usize,
    pub cached: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillHubAgentSummary {
    pub id: String,
    pub display_name: String,
    pub global_skills_dir: String,
    pub detected: bool,
    pub installed_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillHubInstalledSkill {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub skill_dir: String,
}

/// Skill installation metadata (persisted as `.skill-meta.json`)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillInstallMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install_date: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
}

/// Unified skill with platform information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedSkill {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub skill_dir: String,
    pub platform: String,
    pub platform_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    // Extended metadata fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install_date: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedSkillsResponse {
    pub skills: Vec<UnifiedSkill>,
    pub total: usize,
    pub platforms: Vec<SkillHubAgentSummary>,
}

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<usize>,
    pub page: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub limit: Option<usize>,
    pub page: Option<usize>,
}

#[derive(Clone)]
struct CacheEntry<T> {
    expires_at: Instant,
    value: T,
}

static HTTP: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .user_agent("ccr-ui-skill-hub/1.0")
        .build()
        .expect("reqwest client build failed")
});

static MARKETPLACE_CACHE: Lazy<RwLock<HashMap<String, CacheEntry<Vec<SkillHubMarketplaceItem>>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

const CACHE_TTL: Duration = Duration::from_secs(60 * 5);

pub async fn marketplace_trending(
    Query(q): Query<PaginationQuery>,
) -> ApiResponse<SkillHubMarketplaceResponse> {
    let limit = q.limit.unwrap_or(30).clamp(1, 200);
    let page = q.page.unwrap_or(1).max(1);

    marketplace_trending_impl(limit, page).await
}

pub async fn marketplace_search(
    Query(q): Query<SearchQuery>,
) -> ApiResponse<SkillHubMarketplaceResponse> {
    let query = q.q.unwrap_or_default().trim().to_string();
    let limit = q.limit.unwrap_or(30).clamp(1, 200);
    let page = q.page.unwrap_or(1).max(1);

    if query.is_empty() {
        return marketplace_trending_impl(limit, page).await;
    }

    let mut url = Url::parse("https://skills.sh/").expect("valid skills.sh url");
    url.query_pairs_mut().append_pair("q", &query);

    let cache_key = format!("search:{}", query.to_lowercase());
    match get_marketplace_items_cached(url.to_string(), &cache_key).await {
        Ok((mut items, cached)) => {
            if items.is_empty() {
                match get_marketplace_items_cached("https://skills.sh/".to_string(), "trending")
                    .await
                {
                    Ok((base_items, base_cached)) => {
                        items = filter_items(base_items, &query);
                        let total = items.len();
                        let items = paginate(items, limit, page);
                        ApiResponse::success(SkillHubMarketplaceResponse {
                            items,
                            total,
                            cached: cached && base_cached,
                        })
                    }
                    Err(err) => ApiResponse::error(format!("Failed to fetch skills.sh: {}", err)),
                }
            } else {
                let total = items.len();
                let items = paginate(items, limit, page);
                ApiResponse::success(SkillHubMarketplaceResponse {
                    items,
                    total,
                    cached,
                })
            }
        }
        Err(err) => ApiResponse::error(format!("Failed to fetch skills.sh: {}", err)),
    }
}

async fn marketplace_trending_impl(
    limit: usize,
    page: usize,
) -> ApiResponse<SkillHubMarketplaceResponse> {
    match get_marketplace_items_cached("https://skills.sh/".to_string(), "trending").await {
        Ok((items, cached)) => {
            let total = items.len();
            let items = paginate(items, limit, page);
            ApiResponse::success(SkillHubMarketplaceResponse {
                items,
                total,
                cached,
            })
        }
        Err(err) => ApiResponse::error(format!("Failed to fetch skills.sh: {}", err)),
    }
}

pub async fn list_agents() -> impl IntoResponse {
    let agents = list_supported_agents()
        .into_iter()
        .map(|agent| {
            let dir = agent_global_skills_dir(&agent.id);
            let (detected, installed_count) = match &dir {
                Some(p) => (p.exists(), count_installed_skills_fast(p)),
                None => (false, 0),
            };

            SkillHubAgentSummary {
                id: agent.id,
                display_name: agent.display_name,
                global_skills_dir: dir.as_ref().map(|p| path_to_string(p)).unwrap_or_default(),
                detected,
                installed_count,
            }
        })
        .collect::<Vec<_>>();

    ApiResponse::success(agents)
}

/// GET /skill_hub/unified - List all skills from all platforms
pub async fn list_unified_skills() -> impl IntoResponse {
    let agents = list_supported_agents();
    let mut all_skills = Vec::new();
    let mut platforms = Vec::new();

    for agent in agents {
        let dir = agent_global_skills_dir(&agent.id);
        let detected = dir.as_ref().is_some_and(|p| p.exists());

        // Single scan: list skills first, derive count from result (no double I/O)
        let skills = match &dir {
            Some(dir_path) if detected => {
                list_unified_skills_in_dir(dir_path, &agent.id, &agent.display_name)
            }
            _ => Vec::new(),
        };
        let installed_count = skills.len();
        all_skills.extend(skills);

        platforms.push(SkillHubAgentSummary {
            id: agent.id.clone(),
            display_name: agent.display_name.clone(),
            global_skills_dir: dir.as_ref().map(|p| path_to_string(p)).unwrap_or_default(),
            detected,
            installed_count,
        });
    }

    let total = all_skills.len();
    ApiResponse::success(UnifiedSkillsResponse {
        skills: all_skills,
        total,
        platforms,
    })
}

/// GET /skill_hub/unified/{platform} - List skills for a specific platform
pub async fn list_unified_skills_by_platform(
    AxumPath(platform): AxumPath<String>,
) -> impl IntoResponse {
    let agents = list_supported_agents();
    let agent = match agents.iter().find(|a| a.id == platform) {
        Some(a) => a.clone(),
        None => {
            return ApiResponse::<UnifiedSkillsResponse>::error("Unknown platform".to_string());
        }
    };

    let dir = agent_global_skills_dir(&agent.id);
    let detected = dir.as_ref().is_some_and(|p| p.exists());

    // Single scan: list skills first, derive count from result (no double I/O)
    let skills = match &dir {
        Some(dir_path) if detected => {
            list_unified_skills_in_dir(dir_path, &agent.id, &agent.display_name)
        }
        _ => Vec::new(),
    };
    let installed_count = skills.len();

    let platform_summary = SkillHubAgentSummary {
        id: agent.id.clone(),
        display_name: agent.display_name.clone(),
        global_skills_dir: dir.as_ref().map(|p| path_to_string(p)).unwrap_or_default(),
        detected,
        installed_count,
    };

    let total = skills.len();
    ApiResponse::success(UnifiedSkillsResponse {
        skills,
        total,
        platforms: vec![platform_summary],
    })
}

pub async fn list_agent_skills(AxumPath(agent): AxumPath<String>) -> impl IntoResponse {
    let dir = match agent_global_skills_dir(&agent) {
        Some(p) => p,
        None => {
            return ApiResponse::<Vec<SkillHubInstalledSkill>>::error("Unknown agent".to_string());
        }
    };

    let skills = list_installed_skills_in_dir(&dir);
    ApiResponse::success(skills)
}

#[derive(Debug, Deserialize)]
pub struct InstallRequest {
    pub package: String,
    #[serde(default)]
    pub agents: Vec<String>,
    #[serde(default)]
    pub force: bool,
}

#[derive(Debug, Deserialize)]
pub struct RemoveRequest {
    pub skill: String,
    #[serde(default)]
    pub agents: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOperationResult {
    pub agent: String,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillHubOperationResponse {
    pub results: Vec<AgentOperationResult>,
}

pub async fn install(Json(req): Json<InstallRequest>) -> impl IntoResponse {
    let (owner, repo, skill) = match parse_owner_repo_skill(&req.package) {
        Ok(v) => v,
        Err(msg) => {
            return ApiResponse::<SkillHubOperationResponse>::error(msg);
        }
    };

    let target_agents = if req.agents.is_empty() {
        list_supported_agents()
            .into_iter()
            .map(|a| a.id)
            .collect::<Vec<_>>()
    } else {
        req.agents.clone()
    };

    let mut results = Vec::new();
    for agent in target_agents {
        let Some(global_dir) = agent_global_skills_dir(&agent) else {
            results.push(AgentOperationResult {
                agent,
                ok: false,
                message: Some("Unknown agent".to_string()),
            });
            continue;
        };

        match install_skill_from_github(&owner, &repo, &skill, &global_dir, req.force).await {
            Ok(_) => {
                // Log install operation
                let log_entry = log_repo::LogEntry {
                    id: uuid::Uuid::new_v4().to_string(),
                    timestamp: chrono::Utc::now(),
                    level: "INFO".to_string(),
                    source: "SkillHub".to_string(),
                    message: format!("Installed skill '{}' to {}", req.package, agent),
                    metadata_json: Some(
                        serde_json::json!({
                            "action": "install",
                            "package": req.package,
                            "agent": agent
                        })
                        .to_string(),
                    ),
                };
                let _ = database::with_connection(|conn| log_repo::insert_log(conn, &log_entry));

                results.push(AgentOperationResult {
                    agent,
                    ok: true,
                    message: None,
                })
            }
            Err(err) => results.push(AgentOperationResult {
                agent,
                ok: false,
                message: Some(err.to_string()),
            }),
        }
    }

    ApiResponse::success(SkillHubOperationResponse { results })
}

pub async fn remove(Json(req): Json<RemoveRequest>) -> impl IntoResponse {
    let target_agents = if req.agents.is_empty() {
        list_supported_agents()
            .into_iter()
            .map(|a| a.id)
            .collect::<Vec<_>>()
    } else {
        req.agents.clone()
    };

    let mut results = Vec::new();
    for agent in target_agents {
        let Some(global_dir) = agent_global_skills_dir(&agent) else {
            results.push(AgentOperationResult {
                agent,
                ok: false,
                message: Some("Unknown agent".to_string()),
            });
            continue;
        };

        let skill_dir = global_dir.join(&req.skill);
        let resolved_dir = if skill_dir.exists() {
            Some(skill_dir)
        } else {
            find_installed_skill_dir_by_name(&global_dir, &req.skill)
        };

        let Some(resolved_dir) = resolved_dir else {
            results.push(AgentOperationResult {
                agent,
                ok: true,
                message: Some("Already removed".to_string()),
            });
            continue;
        };

        match fs::remove_dir_all(&resolved_dir) {
            Ok(_) => {
                // Log remove operation
                let log_entry = log_repo::LogEntry {
                    id: uuid::Uuid::new_v4().to_string(),
                    timestamp: chrono::Utc::now(),
                    level: "INFO".to_string(),
                    source: "SkillHub".to_string(),
                    message: format!("Removed skill '{}' from {}", req.skill, agent),
                    metadata_json: Some(
                        serde_json::json!({
                            "action": "remove",
                            "skill": req.skill,
                            "agent": agent,
                            "skill_dir": path_to_string(&resolved_dir)
                        })
                        .to_string(),
                    ),
                };
                let _ = database::with_connection(|conn| log_repo::insert_log(conn, &log_entry));

                results.push(AgentOperationResult {
                    agent,
                    ok: true,
                    message: None,
                })
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                results.push(AgentOperationResult {
                    agent,
                    ok: true,
                    message: Some("Already removed".to_string()),
                })
            }
            Err(err) => results.push(AgentOperationResult {
                agent,
                ok: false,
                message: Some(err.to_string()),
            }),
        }
    }

    ApiResponse::success(SkillHubOperationResponse { results })
}

#[derive(Clone)]
struct SupportedAgent {
    id: String,
    display_name: String,
}

fn list_supported_agents() -> Vec<SupportedAgent> {
    vec![
        SupportedAgent {
            id: "claude-code".to_string(),
            display_name: "Claude Code".to_string(),
        },
        SupportedAgent {
            id: "codex".to_string(),
            display_name: "Codex".to_string(),
        },
        SupportedAgent {
            id: "gemini".to_string(),
            display_name: "Gemini CLI".to_string(),
        },
        SupportedAgent {
            id: "qwen".to_string(),
            display_name: "Qwen".to_string(),
        },
        SupportedAgent {
            id: "iflow".to_string(),
            display_name: "iFlow".to_string(),
        },
        SupportedAgent {
            id: "droid".to_string(),
            display_name: "Droid".to_string(),
        },
    ]
}

fn agent_global_skills_dir(agent: &str) -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    match agent {
        "claude-code" => Some(home.join(".claude").join("skills")),
        "codex" => Some(home.join(".codex").join("skills")),
        "gemini" => Some(home.join(".gemini").join("skills")),
        "qwen" => Some(home.join(".qwen").join("skills")),
        "iflow" => Some(home.join(".iflow").join("skills")),
        "droid" => Some(home.join(".gemini").join("antigravity").join("skills")),
        _ => None,
    }
}

fn list_installed_skills_in_dir(dir: &Path) -> Vec<SkillHubInstalledSkill> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };

    let mut skills = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let skill_md = path.join("SKILL.md");
        if !skill_md.is_file() {
            continue;
        }

        let (name, description) = parse_skill_md_frontmatter(&skill_md).unwrap_or_else(|| {
            (
                path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                None,
            )
        });

        skills.push(SkillHubInstalledSkill {
            name,
            description,
            skill_dir: path_to_string(&path),
        });
    }

    skills.sort_by(|a, b| a.name.cmp(&b.name));
    skills
}

fn list_unified_skills_in_dir(
    dir: &Path,
    platform: &str,
    platform_name: &str,
) -> Vec<UnifiedSkill> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };

    let mut skills = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let skill_md = path.join("SKILL.md");
        if !skill_md.is_file() {
            continue;
        }

        let parsed = parse_skill_md_full(&skill_md).unwrap_or_else(|| SkillMdParsed {
            name: path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            description: None,
            category: None,
            tags: Vec::new(),
            version: None,
            author: None,
        });

        // Load persisted metadata (.skill-meta.json) if available
        let meta = load_skill_metadata(&path);

        skills.push(UnifiedSkill {
            name: parsed.name,
            description: parsed.description,
            skill_dir: path_to_string(&path),
            platform: platform.to_string(),
            platform_name: platform_name.to_string(),
            category: parsed.category,
            tags: parsed.tags,
            // version/author: prefer frontmatter, fallback to meta
            version: parsed.version.or_else(|| meta.as_ref().and_then(|m| m.version.clone())),
            author: parsed.author.or_else(|| meta.as_ref().and_then(|m| m.author.clone())),
            // source fields: from persisted metadata only
            source: meta.as_ref().and_then(|m| m.source.clone()),
            source_url: meta.as_ref().and_then(|m| m.source_url.clone()),
            install_date: meta.as_ref().and_then(|m| m.install_date),
            commit_hash: meta.as_ref().and_then(|m| m.commit_hash.clone()),
        });
    }

    skills.sort_by(|a, b| a.name.cmp(&b.name));
    skills
}

/// Lightweight skill count: only checks for subdirs containing SKILL.md,
/// does NOT parse file contents. Use this when you only need the count.
fn count_installed_skills_fast(dir: &Path) -> usize {
    let Ok(entries) = fs::read_dir(dir) else {
        return 0;
    };
    entries
        .flatten()
        .filter(|e| e.path().is_dir() && e.path().join("SKILL.md").is_file())
        .count()
}

fn find_installed_skill_dir_by_name(dir: &Path, name: &str) -> Option<PathBuf> {
    let Ok(entries) = fs::read_dir(dir) else {
        return None;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let skill_md = path.join("SKILL.md");
        if !skill_md.is_file() {
            continue;
        }

        if let Some((frontmatter_name, _)) = parse_skill_md_frontmatter(&skill_md)
            && frontmatter_name == name
        {
            return Some(path);
        }
    }

    None
}

fn parse_skill_md_frontmatter(path: &Path) -> Option<(String, Option<String>)> {
    let content = fs::read_to_string(path).ok()?;
    let mut lines = content.lines();

    if lines.next()? != "---" {
        return None;
    }

    let mut yaml_lines = Vec::new();
    for line in lines.by_ref() {
        if line == "---" {
            break;
        }
        yaml_lines.push(line);
    }

    let yaml = yaml_lines.join("\n");
    let value: serde_yaml::Value = serde_yaml::from_str(&yaml).ok()?;
    let name = value
        .get("name")
        .and_then(|v| v.as_str())
        .map(|v| v.to_string())?;
    let description = value
        .get("description")
        .and_then(|v| v.as_str())
        .map(|v| v.to_string());

    Some((name, description))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillContentResponse {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub content: String,
    pub raw: String,
    pub skill_dir: String,
}

#[derive(Debug, Deserialize)]
pub struct SkillContentQuery {
    pub skill_dir: String,
}

#[derive(Debug, Deserialize)]
pub struct SaveSkillContentRequest {
    pub skill_dir: String,
    pub content: String,
}

#[derive(Debug, Clone)]
struct SkillMdParsed {
    name: String,
    description: Option<String>,
    category: Option<String>,
    tags: Vec<String>,
    version: Option<String>,
    author: Option<String>,
}

#[derive(Debug, Clone)]
struct SkillMdWithContent {
    name: String,
    description: Option<String>,
    category: Option<String>,
    tags: Vec<String>,
    content: String,
    raw: String,
}

fn parse_skill_md_full(path: &Path) -> Option<SkillMdParsed> {
    let content = fs::read_to_string(path).ok()?;
    let mut lines = content.lines();

    if lines.next()? != "---" {
        return None;
    }

    let mut yaml_lines = Vec::new();
    for line in lines.by_ref() {
        if line == "---" {
            break;
        }
        yaml_lines.push(line);
    }

    let yaml = yaml_lines.join("\n");
    let value: serde_yaml::Value = serde_yaml::from_str(&yaml).ok()?;

    let name = value
        .get("name")
        .and_then(|v| v.as_str())
        .map(|v| v.to_string())?;

    let description = value
        .get("description")
        .and_then(|v| v.as_str())
        .map(|v| v.to_string());

    let category = value
        .get("category")
        .and_then(|v| v.as_str())
        .map(|v| v.to_string());

    let tags = value
        .get("tags")
        .and_then(|v| v.as_sequence())
        .map(|seq| {
            seq.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let version = value
        .get("version")
        .and_then(|v| v.as_str())
        .map(|v| v.to_string());

    let author = value
        .get("author")
        .and_then(|v| v.as_str())
        .map(|v| v.to_string());

    Some(SkillMdParsed {
        name,
        description,
        category,
        tags,
        version,
        author,
    })
}

fn parse_skill_md_with_content(path: &Path) -> Option<SkillMdWithContent> {
    let raw = fs::read_to_string(path).ok()?;
    let mut lines = raw.lines();

    if lines.next()? != "---" {
        // No frontmatter, treat entire file as content
        return Some(SkillMdWithContent {
            name: path
                .parent()
                .and_then(|p| p.file_name())
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            description: None,
            category: None,
            tags: Vec::new(),
            content: raw.clone(),
            raw,
        });
    }

    let mut yaml_lines = Vec::new();
    for line in lines.by_ref() {
        if line == "---" {
            break;
        }
        yaml_lines.push(line);
    }

    // Collect remaining lines as markdown body
    let content: String = lines.collect::<Vec<_>>().join("\n").trim().to_string();

    let yaml = yaml_lines.join("\n");
    let value: serde_yaml::Value = serde_yaml::from_str(&yaml).ok()?;

    let name = value
        .get("name")
        .and_then(|v| v.as_str())
        .map(|v| v.to_string())
        .unwrap_or_else(|| {
            path.parent()
                .and_then(|p| p.file_name())
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        });

    let description = value
        .get("description")
        .and_then(|v| v.as_str())
        .map(|v| v.to_string());

    let category = value
        .get("category")
        .and_then(|v| v.as_str())
        .map(|v| v.to_string());

    let tags = value
        .get("tags")
        .and_then(|v| v.as_sequence())
        .map(|seq| {
            seq.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    Some(SkillMdWithContent {
        name,
        description,
        category,
        tags,
        content,
        raw,
    })
}

/// Validate that a skill_dir is under a known agent's global skills directory.
/// Returns the canonicalized path if valid.
fn validate_skill_dir(skill_dir: &str) -> Result<PathBuf, String> {
    let requested = PathBuf::from(skill_dir);

    // Reject obvious path traversal
    let normalized = requested.to_string_lossy();
    if normalized.contains("..") {
        return Err("Invalid path: path traversal not allowed".to_string());
    }

    // Check that the path is under a known agent's skills directory
    let agents = list_supported_agents();
    for agent in &agents {
        if let Some(global_dir) = agent_global_skills_dir(&agent.id) {
            let global_str = path_to_string(&global_dir);
            if normalized.starts_with(&global_str) {
                // Valid: path is under a known skills directory
                return Ok(requested);
            }
        }
    }

    Err("Invalid path: not under any known skills directory".to_string())
}

/// GET /skill_hub/skill/content - Read SKILL.md content for a specific skill
pub async fn get_skill_content(Query(q): Query<SkillContentQuery>) -> impl IntoResponse {
    let skill_dir = match validate_skill_dir(&q.skill_dir) {
        Ok(p) => p,
        Err(msg) => {
            return ApiResponse::<SkillContentResponse>::error(msg);
        }
    };

    let skill_md = skill_dir.join("SKILL.md");
    if !skill_md.is_file() {
        return ApiResponse::<SkillContentResponse>::error(
            "SKILL.md not found in the specified directory".to_string(),
        );
    }

    match parse_skill_md_with_content(&skill_md) {
        Some(parsed) => ApiResponse::success(SkillContentResponse {
            name: parsed.name,
            description: parsed.description,
            category: parsed.category,
            tags: parsed.tags,
            content: parsed.content,
            raw: parsed.raw,
            skill_dir: path_to_string(&skill_dir),
        }),
        None => ApiResponse::<SkillContentResponse>::error("Failed to parse SKILL.md".to_string()),
    }
}

/// POST /skill_hub/skill/content - Save SKILL.md content
pub async fn save_skill_content(Json(req): Json<SaveSkillContentRequest>) -> impl IntoResponse {
    let skill_dir = match validate_skill_dir(&req.skill_dir) {
        Ok(p) => p,
        Err(msg) => {
            return ApiResponse::<()>::error(msg);
        }
    };

    let skill_md = skill_dir.join("SKILL.md");

    // Atomic write: write to temp file first, then rename
    let tmp_path = skill_dir.join(format!(".SKILL.md.tmp-{}", uuid::Uuid::new_v4()));

    if let Err(err) = fs::write(&tmp_path, &req.content) {
        return ApiResponse::<()>::error(format!("Failed to write temporary file: {}", err));
    }

    if let Err(err) = fs::rename(&tmp_path, &skill_md) {
        // Clean up temp file on rename failure
        let _ = fs::remove_file(&tmp_path);
        return ApiResponse::<()>::error(format!("Failed to save SKILL.md: {}", err));
    }

    ApiResponse::success(())
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

/// Load skill installation metadata from `.skill-meta.json`
fn load_skill_metadata(skill_dir: &Path) -> Option<SkillInstallMeta> {
    let meta_path = skill_dir.join(".skill-meta.json");
    let content = fs::read_to_string(meta_path).ok()?;
    serde_json::from_str(&content).ok()
}

/// Save skill installation metadata to `.skill-meta.json` (atomic write)
fn save_skill_metadata(skill_dir: &Path, metadata: &SkillInstallMeta) -> Result<(), String> {
    let content = serde_json::to_string_pretty(metadata)
        .map_err(|e| format!("Failed to serialize metadata: {e}"))?;
    let tmp_path = skill_dir.join(format!(".skill-meta.json.tmp-{}", uuid::Uuid::new_v4()));
    fs::write(&tmp_path, &content)
        .map_err(|e| format!("Failed to write metadata tmp: {e}"))?;
    fs::rename(&tmp_path, skill_dir.join(".skill-meta.json"))
        .map_err(|e| {
            let _ = fs::remove_file(&tmp_path);
            format!("Failed to rename metadata: {e}")
        })
}

/// Current time as Unix timestamp in milliseconds
fn current_timestamp_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

async fn get_marketplace_items_cached(
    url: String,
    cache_key: &str,
) -> anyhow::Result<(Vec<SkillHubMarketplaceItem>, bool)> {
    {
        let cache = MARKETPLACE_CACHE.read().await;
        if let Some(entry) = cache.get(cache_key)
            && Instant::now() < entry.expires_at
        {
            return Ok((entry.value.clone(), true));
        }
    }

    let html = HTTP
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    let items = parse_skills_sh_links(&html);

    {
        let mut cache = MARKETPLACE_CACHE.write().await;
        cache.insert(
            cache_key.to_string(),
            CacheEntry {
                expires_at: Instant::now() + CACHE_TTL,
                value: items.clone(),
            },
        );
    }

    Ok((items, false))
}

fn parse_skills_sh_links(html: &str) -> Vec<SkillHubMarketplaceItem> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("a[href]").expect("valid selector");

    let mut seen = HashSet::<String>::new();
    let mut items = Vec::new();

    for node in document.select(&selector) {
        let href = match node.value().attr("href") {
            Some(v) => v,
            None => continue,
        };

        let absolute = if href.starts_with("http://") || href.starts_with("https://") {
            href.to_string()
        } else if href.starts_with('/') {
            format!("https://skills.sh{}", href)
        } else {
            continue;
        };

        if !absolute.starts_with("https://skills.sh/") {
            continue;
        }

        let url = match Url::parse(&absolute) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let segments: Vec<String> = url
            .path_segments()
            .map(|s| s.filter(|p| !p.is_empty()).map(|p| p.to_string()).collect())
            .unwrap_or_default();

        if segments.len() < 2 {
            continue;
        }

        let owner = segments[0].clone();
        let repo = segments[1].clone();
        let skill = segments.get(2).cloned();

        if owner == "categories" || owner == "leaderboard" {
            continue;
        }

        let package = match &skill {
            Some(s) => format!("{}/{}@{}", owner, repo, s),
            None => format!("{}/{}", owner, repo),
        };

        let key = format!("{}|{}", package, absolute);
        if !seen.insert(key) {
            continue;
        }

        items.push(SkillHubMarketplaceItem {
            package,
            owner,
            repo,
            skill,
            skills_sh_url: absolute,
        });
    }

    items
}

fn paginate<T: Clone>(items: Vec<T>, limit: usize, page: usize) -> Vec<T> {
    let start = (page - 1).saturating_mul(limit);
    if start >= items.len() {
        return Vec::new();
    }

    let end = (start + limit).min(items.len());
    items[start..end].to_vec()
}

fn filter_items(items: Vec<SkillHubMarketplaceItem>, query: &str) -> Vec<SkillHubMarketplaceItem> {
    let q = query.to_lowercase();
    items
        .into_iter()
        .filter(|item| {
            item.package.to_lowercase().contains(&q)
                || item.owner.to_lowercase().contains(&q)
                || item.repo.to_lowercase().contains(&q)
                || item
                    .skill
                    .as_ref()
                    .map(|s| s.to_lowercase().contains(&q))
                    .unwrap_or(false)
        })
        .collect()
}

#[derive(Debug, Deserialize)]
struct GithubRepoInfo {
    default_branch: String,
}

#[derive(Debug, Clone, Deserialize)]
struct GithubContentItem {
    name: String,
    path: String,
    #[serde(rename = "type")]
    kind: String,
    download_url: Option<String>,
}

fn parse_owner_repo_skill(package: &str) -> Result<(String, String, String), String> {
    let trimmed = package.trim();
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        let url = Url::parse(trimmed).map_err(|e| e.to_string())?;
        let segments: Vec<String> = url
            .path_segments()
            .map(|s| s.filter(|p| !p.is_empty()).map(|p| p.to_string()).collect())
            .unwrap_or_default();
        if segments.len() < 3 {
            return Err(
                "Only owner/repo@skill or skills.sh/owner/repo/skill supported".to_string(),
            );
        }
        return Ok((
            segments[0].clone(),
            segments[1].clone(),
            segments[2].clone(),
        ));
    }

    let (repo_part, skill) = trimmed
        .split_once('@')
        .ok_or_else(|| "Only owner/repo@skill is supported for installation".to_string())?;
    let parts: Vec<&str> = repo_part.split('/').collect();
    if parts.len() != 2 {
        return Err("Invalid package format. Expected owner/repo@skill".to_string());
    }
    if skill.trim().is_empty() {
        return Err("Invalid package format. Missing skill name after @".to_string());
    }
    Ok((
        parts[0].to_string(),
        parts[1].to_string(),
        skill.to_string(),
    ))
}

async fn install_skill_from_github(
    owner: &str,
    repo: &str,
    skill: &str,
    global_skills_dir: &Path,
    force: bool,
) -> anyhow::Result<()> {
    fs::create_dir_all(global_skills_dir)?;
    let target_dir = global_skills_dir.join(skill);

    if target_dir.exists() {
        if !force {
            return Err(anyhow::anyhow!("Skill already installed"));
        }
        fs::remove_dir_all(&target_dir)?;
    }

    let tmp_dir = global_skills_dir.join(format!(".{}.tmp-{}", skill, uuid::Uuid::new_v4()));
    fs::create_dir_all(&tmp_dir)?;

    let result = install_skill_into_dir(owner, repo, skill, &tmp_dir).await;
    if result.is_err() {
        let _ = fs::remove_dir_all(&tmp_dir);
    }
    result?;

    if target_dir.exists() {
        fs::remove_dir_all(&target_dir)?;
    }
    fs::rename(&tmp_dir, &target_dir)?;

    // Persist installation metadata
    let meta = SkillInstallMeta {
        source: Some("marketplace".to_string()),
        source_url: Some(format!("https://skills.sh/{owner}/{repo}")),
        install_date: Some(current_timestamp_ms()),
        commit_hash: None,
        version: None,
        author: None,
    };
    let _ = save_skill_metadata(&target_dir, &meta);

    Ok(())
}

async fn install_skill_into_dir(
    owner: &str,
    repo: &str,
    skill: &str,
    target_dir: &Path,
) -> anyhow::Result<()> {
    if let Ok(()) = install_skill_via_github_contents_api(owner, repo, skill, target_dir).await {
        return Ok(());
    }

    install_skill_via_raw_fallback(owner, repo, skill, target_dir).await
}

async fn install_skill_via_github_contents_api(
    owner: &str,
    repo: &str,
    skill: &str,
    target_dir: &Path,
) -> anyhow::Result<()> {
    let default_branch = github_default_branch(owner, repo).await?;
    let base_dir = github_resolve_skill_base_dir(owner, repo, &default_branch, skill).await?;

    let base_listing = github_list_dir(owner, repo, &default_branch, &base_dir).await?;
    let skill_md_item = base_listing
        .iter()
        .find(|i| i.kind == "file" && i.name == "SKILL.md")
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("SKILL.md not found in repo"))?;

    let skill_md = github_download_text(owner, repo, &default_branch, &skill_md_item).await?;
    fs::write(target_dir.join("SKILL.md"), skill_md)?;

    for subdir in ["scripts", "references"] {
        let sub_path = format!("{}/{}", base_dir, subdir);
        if let Ok(entries) = github_list_dir(owner, repo, &default_branch, &sub_path).await {
            let out_dir = target_dir.join(subdir);
            fs::create_dir_all(&out_dir)?;
            for item in entries {
                if item.kind != "file" {
                    continue;
                }
                let content = github_download_bytes(owner, repo, &default_branch, &item).await?;
                fs::write(out_dir.join(item.name), content)?;
            }
        }
    }

    Ok(())
}

async fn install_skill_via_raw_fallback(
    owner: &str,
    repo: &str,
    skill: &str,
    target_dir: &Path,
) -> anyhow::Result<()> {
    let candidates = [
        ("main", format!("{}/SKILL.md", skill)),
        ("main", format!("skills/{}/SKILL.md", skill)),
        ("master", format!("{}/SKILL.md", skill)),
        ("master", format!("skills/{}/SKILL.md", skill)),
    ];

    for (branch, path) in candidates {
        let url = format!(
            "https://raw.githubusercontent.com/{}/{}/{}/{}",
            owner, repo, branch, path
        );
        let resp = match HTTP.get(url).send().await {
            Ok(v) => v,
            Err(_) => continue,
        };
        let resp = match resp.error_for_status() {
            Ok(v) => v,
            Err(_) => continue,
        };
        let text = match resp.text().await {
            Ok(v) => v,
            Err(_) => continue,
        };
        fs::write(target_dir.join("SKILL.md"), text)?;
        return Ok(());
    }

    Err(anyhow::anyhow!("Failed to download SKILL.md from GitHub"))
}

async fn github_default_branch(owner: &str, repo: &str) -> anyhow::Result<String> {
    let url = format!("https://api.github.com/repos/{}/{}", owner, repo);
    let info = HTTP
        .get(url)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await?
        .error_for_status()?
        .json::<GithubRepoInfo>()
        .await?;
    Ok(info.default_branch)
}

async fn github_resolve_skill_base_dir(
    owner: &str,
    repo: &str,
    branch: &str,
    skill: &str,
) -> anyhow::Result<String> {
    let root_candidate = skill.to_string();
    if let Ok(entries) = github_list_dir(owner, repo, branch, &root_candidate).await
        && entries
            .iter()
            .any(|i| i.kind == "file" && i.name == "SKILL.md")
    {
        return Ok(root_candidate);
    }

    let skills_candidate = format!("skills/{}", skill);
    if let Ok(entries) = github_list_dir(owner, repo, branch, &skills_candidate).await
        && entries
            .iter()
            .any(|i| i.kind == "file" && i.name == "SKILL.md")
    {
        return Ok(skills_candidate);
    }

    Err(anyhow::anyhow!("Unable to locate skill directory in repo"))
}

async fn github_list_dir(
    owner: &str,
    repo: &str,
    branch: &str,
    path: &str,
) -> anyhow::Result<Vec<GithubContentItem>> {
    let mut url = Url::parse(&format!(
        "https://api.github.com/repos/{}/{}/contents/{}",
        owner, repo, path
    ))?;
    url.query_pairs_mut().append_pair("ref", branch);
    let res = HTTP
        .get(url)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await?
        .error_for_status()?;
    let value = res.json::<serde_json::Value>().await?;
    let arr = value
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("Expected directory listing from GitHub contents API"))?;
    let mut items = Vec::new();
    for v in arr {
        let item: GithubContentItem = serde_json::from_value(v.clone())?;
        items.push(item);
    }
    Ok(items)
}

async fn github_download_text(
    owner: &str,
    repo: &str,
    branch: &str,
    item: &GithubContentItem,
) -> anyhow::Result<String> {
    let url = item.download_url.clone().unwrap_or_else(|| {
        format!(
            "https://raw.githubusercontent.com/{}/{}/{}/{}",
            owner, repo, branch, item.path
        )
    });
    Ok(HTTP
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?)
}

async fn github_download_bytes(
    owner: &str,
    repo: &str,
    branch: &str,
    item: &GithubContentItem,
) -> anyhow::Result<Vec<u8>> {
    let url = item.download_url.clone().unwrap_or_else(|| {
        format!(
            "https://raw.githubusercontent.com/{}/{}/{}/{}",
            owner, repo, branch, item.path
        )
    });
    Ok(HTTP
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?
        .to_vec())
}
