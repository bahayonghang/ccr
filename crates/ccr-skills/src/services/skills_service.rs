use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// 创建一个在 Windows 上隐藏控制台窗口的 Command。
/// 防止 Tauri 桌面应用中生成可见的 cmd.exe / git.exe 子进程窗口。
fn silent_command(program: &str) -> Command {
    let cmd = Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let mut cmd = cmd;
        cmd.creation_flags(CREATE_NO_WINDOW);
        cmd
    }
    #[cfg(not(windows))]
    {
        cmd
    }
}

use blake3::Hasher;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::models::skill::Skill;
use crate::models::skills::{
    MarketplaceListResponse, MarketplaceSkill, NpxPlatformSupport, NpxStatus, SkillContent,
    SkillDescriptor, SkillFileContent, SkillFileEntry, SkillInstallMeta, SkillInstallMode,
    SkillInstallStrategy, SkillInstallationRecord, SkillLifecycleSummary, SkillOperationResponse,
    SkillOrigin, SkillPlatformConfig, SkillPlatformSummary, SkillRecord, SkillSourceHealth,
    SkillSourceRecord, SkillSourceSkillRecord, SkillSourceType, SkillTargetRecord,
    SkillTargetStatus, SkillsInstallCommandPreview, SkillsInstallRequest,
    SkillsInstallReviewResponse, SkillsInstallReviewSource, SkillsInstallReviewTarget,
    SkillsInventoryQuery, SkillsInventoryResponse, SkillsNpxCapabilities,
    SkillsOnboardingCandidate, SkillsSourceManifest, SkillsSyncRequest,
};
use ccr_core::core::atomic_writer::AtomicWriter;
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::http::HTTP_CLIENT;

const DEFAULT_MARKETPLACE_URL: &str = "https://skills.sh/api/trending";
const BUNDLED_FEATURED_SKILLS_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../ccr/assets/featured-skills.json"
));
const SKILL_META_FILENAME: &str = ".skill-meta.json";
const SOURCES_FILENAME: &str = "sources.json";
const PLATFORMS_FILENAME: &str = "platforms.toml";
const SUMMARY_CACHE_FILENAME: &str = "skills-summary-cache.json";

#[derive(Debug, Clone)]
pub struct SkillsService {
    home_dir: PathBuf,
    ccr_skills_dir: PathBuf,
    sources_path: PathBuf,
    platforms_path: PathBuf,
    summary_cache_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct SkillPlatformsConfigFile {
    #[serde(default)]
    platforms: Vec<SkillPlatformConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct SkillSourcesFile {
    #[serde(default)]
    sources: Vec<SkillSourceEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SkillSourceEntry {
    id: String,
    #[serde(rename = "type")]
    source_type: SkillSourceType,
    location: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    checkout_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_synced_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct SkillSummaryCacheFile {
    #[serde(default)]
    entries: HashMap<String, SkillSummaryCacheEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SkillSummaryCacheEntry {
    skill_id: String,
    installation_id: String,
    name: String,
    description: Option<String>,
    category: Option<String>,
    tags: Vec<String>,
    version: Option<String>,
    author: Option<String>,
    origin: SkillOrigin,
    source_label: Option<String>,
    source_ref: Option<String>,
    install_group_id: Option<String>,
    content_hash: String,
    installed_at: Option<i64>,
    skill_mtime_ms: u64,
    meta_mtime_ms: Option<u64>,
}

#[derive(Debug, Clone)]
struct ScannedInstallation {
    skill_id: String,
    installation_id: String,
    name: String,
    description: Option<String>,
    category: Option<String>,
    tags: Vec<String>,
    version: Option<String>,
    author: Option<String>,
    origin: SkillOrigin,
    source_label: Option<String>,
    source_ref: Option<String>,
    installed_at: Option<i64>,
    install_path: PathBuf,
    platform_id: String,
    platform_name: String,
}

#[derive(Debug, Clone)]
struct SkillAggregate {
    id: String,
    name: String,
    description: Option<String>,
    category: Option<String>,
    tags: Vec<String>,
    version: Option<String>,
    author: Option<String>,
    origin: SkillOrigin,
    source_label: Option<String>,
    source_ref: Option<String>,
    installations: Vec<ScannedInstallation>,
}

#[derive(Debug, Clone)]
struct InstallPayload {
    requested_name: String,
    dir_name: String,
    raw: String,
    origin: SkillOrigin,
    source_url: Option<String>,
    source_repo_id: Option<String>,
    install_group_id: Option<String>,
}

#[derive(Debug, Clone)]
struct InstallationTarget {
    platform: SkillPlatformConfig,
    skill_dir: PathBuf,
}

impl SkillsService {
    pub fn new() -> Result<Self> {
        let home_dir =
            dirs::home_dir().ok_or_else(|| CcrError::ConfigError("无法获取用户主目录".into()))?;
        let ccr_skills_dir = home_dir.join(".ccr").join("skills");
        let sources_path = ccr_skills_dir.join(SOURCES_FILENAME);
        let platforms_path = ccr_skills_dir.join(PLATFORMS_FILENAME);
        let summary_cache_path = dirs::cache_dir()
            .unwrap_or_else(|| ccr_skills_dir.clone())
            .join("ccr")
            .join(SUMMARY_CACHE_FILENAME);

        Ok(Self {
            home_dir,
            ccr_skills_dir,
            sources_path,
            platforms_path,
            summary_cache_path,
        })
    }

    pub fn default_platforms() -> Vec<SkillPlatformConfig> {
        let platform = |id: &str,
                        display_name: &str,
                        relative_path: &str,
                        shared_dir_group: Option<&str>,
                        install_strategy: SkillInstallStrategy,
                        npx_agent_key: Option<&str>,
                        category: Option<&str>,
                        sort_order: usize| SkillPlatformConfig {
            id: id.into(),
            display_name: display_name.into(),
            relative_path: relative_path.into(),
            shared_dir_group: shared_dir_group.map(str::to_string),
            install_strategy,
            npx_agent_key: npx_agent_key.map(str::to_string),
            category: category.map(str::to_string),
            capabilities: vec!["skills".into()],
            sort_order,
        };

        vec![
            platform(
                "claude-code",
                "Claude Code",
                ".claude/skills",
                None,
                SkillInstallStrategy::ManagedCopy,
                Some("claude-code"),
                Some("primary"),
                10,
            ),
            platform(
                "codex",
                "Codex",
                ".codex/skills",
                None,
                SkillInstallStrategy::ManagedCopy,
                Some("codex"),
                Some("primary"),
                20,
            ),
            platform(
                "gemini",
                "Antigravity CLI",
                ".gemini/antigravity-cli/skills",
                None,
                SkillInstallStrategy::ManagedCopy,
                Some("antigravity"),
                Some("primary"),
                30,
            ),
            platform(
                "qwen",
                "Qwen",
                ".qwen/skills",
                None,
                SkillInstallStrategy::ManagedCopy,
                None,
                Some("primary"),
                40,
            ),
            platform(
                "qoder",
                "Qoder",
                ".qoder/skills",
                None,
                SkillInstallStrategy::ManagedCopy,
                None,
                Some("primary"),
                50,
            ),
            platform(
                "droid",
                "Droid",
                ".factory/skills",
                None,
                SkillInstallStrategy::ManagedCopy,
                None,
                Some("primary"),
                60,
            ),
            platform(
                "opencode",
                "OpenCode",
                ".config/opencode/skills",
                None,
                SkillInstallStrategy::ManagedCopy,
                Some("opencode"),
                Some("primary"),
                70,
            ),
            platform(
                "cursor",
                "Cursor",
                ".cursor/skills",
                None,
                SkillInstallStrategy::ManagedCopy,
                None,
                Some("ecosystem"),
                100,
            ),
            platform(
                "amp",
                "Amp",
                ".config/agents/skills",
                Some("agents-shared"),
                SkillInstallStrategy::ManagedCopy,
                None,
                Some("ecosystem"),
                110,
            ),
            platform(
                "kimi-cli",
                "Kimi Code CLI",
                ".config/agents/skills",
                Some("agents-shared"),
                SkillInstallStrategy::ManagedCopy,
                None,
                Some("ecosystem"),
                120,
            ),
            platform(
                "augment",
                "Augment",
                ".augment/rules",
                None,
                SkillInstallStrategy::ManagedCopy,
                None,
                Some("ecosystem"),
                130,
            ),
            platform(
                "openclaw",
                "OpenClaw",
                ".openclaw/skills",
                None,
                SkillInstallStrategy::ManagedCopy,
                None,
                Some("ecosystem"),
                140,
            ),
            platform(
                "cline",
                "Cline",
                ".cline/skills",
                None,
                SkillInstallStrategy::ManagedCopy,
                None,
                Some("ecosystem"),
                150,
            ),
            platform(
                "codebuddy",
                "CodeBuddy",
                ".codebuddy/skills",
                None,
                SkillInstallStrategy::ManagedCopy,
                None,
                Some("ecosystem"),
                160,
            ),
            platform(
                "command-code",
                "Command Code",
                ".commandcode/skills",
                None,
                SkillInstallStrategy::ManagedCopy,
                None,
                Some("ecosystem"),
                170,
            ),
            platform(
                "continue",
                "Continue",
                ".continue/skills",
                None,
                SkillInstallStrategy::ManagedCopy,
                None,
                Some("ecosystem"),
                180,
            ),
            platform(
                "crush",
                "Crush",
                ".config/crush/skills",
                None,
                SkillInstallStrategy::ManagedCopy,
                None,
                Some("ecosystem"),
                190,
            ),
            platform(
                "junie",
                "Junie",
                ".junie/skills",
                None,
                SkillInstallStrategy::ManagedCopy,
                None,
                Some("ecosystem"),
                200,
            ),
            platform(
                "iflow",
                "iFlow CLI",
                ".iflow/skills",
                None,
                SkillInstallStrategy::ManagedCopy,
                None,
                Some("ecosystem"),
                210,
            ),
            platform(
                "kiro",
                "Kiro CLI",
                ".kiro/skills",
                None,
                SkillInstallStrategy::ManagedCopy,
                None,
                Some("ecosystem"),
                220,
            ),
            platform(
                "kode",
                "Kode",
                ".kode/skills",
                None,
                SkillInstallStrategy::ManagedCopy,
                None,
                Some("ecosystem"),
                230,
            ),
            platform(
                "mcpjam",
                "MCPJam",
                ".mcpjam/skills",
                None,
                SkillInstallStrategy::ManagedCopy,
                None,
                Some("ecosystem"),
                240,
            ),
            platform(
                "mistral-vibe",
                "Mistral Vibe",
                ".vibe/skills",
                None,
                SkillInstallStrategy::ManagedCopy,
                None,
                Some("ecosystem"),
                250,
            ),
            platform(
                "mux",
                "Mux",
                ".mux/skills",
                None,
                SkillInstallStrategy::ManagedCopy,
                None,
                Some("ecosystem"),
                260,
            ),
            platform(
                "openclaude",
                "OpenClaude IDE",
                ".openclaude/skills",
                None,
                SkillInstallStrategy::ManagedCopy,
                None,
                Some("ecosystem"),
                270,
            ),
            platform(
                "openhands",
                "OpenHands",
                ".openhands/skills",
                None,
                SkillInstallStrategy::ManagedCopy,
                None,
                Some("ecosystem"),
                280,
            ),
            platform(
                "pi",
                "Pi",
                ".pi/agent/skills",
                None,
                SkillInstallStrategy::ManagedCopy,
                None,
                Some("ecosystem"),
                290,
            ),
            platform(
                "qoderwork",
                "QoderWork",
                ".qoderwork/skills",
                None,
                SkillInstallStrategy::ManagedCopy,
                None,
                Some("ecosystem"),
                300,
            ),
            platform(
                "qwen-code",
                "Qwen Code",
                ".qwen/skills",
                Some("qwen-shared"),
                SkillInstallStrategy::ManagedCopy,
                None,
                Some("ecosystem"),
                310,
            ),
            platform(
                "trae",
                "Trae",
                ".trae/skills",
                None,
                SkillInstallStrategy::ManagedCopy,
                None,
                Some("ecosystem"),
                320,
            ),
            platform(
                "trae-cn",
                "Trae CN",
                ".trae-cn/skills",
                None,
                SkillInstallStrategy::ManagedCopy,
                None,
                Some("ecosystem"),
                330,
            ),
            platform(
                "zencoder",
                "Zencoder",
                ".zencoder/skills",
                None,
                SkillInstallStrategy::ManagedCopy,
                None,
                Some("ecosystem"),
                340,
            ),
            platform(
                "neovate",
                "Neovate",
                ".neovate/skills",
                None,
                SkillInstallStrategy::ManagedCopy,
                None,
                Some("ecosystem"),
                350,
            ),
            platform(
                "pochi",
                "Pochi",
                ".pochi/skills",
                None,
                SkillInstallStrategy::ManagedCopy,
                None,
                Some("ecosystem"),
                360,
            ),
            platform(
                "adal",
                "AdaL",
                ".adal/skills",
                None,
                SkillInstallStrategy::ManagedCopy,
                None,
                Some("ecosystem"),
                370,
            ),
            platform(
                "kilo-code",
                "Kilo Code",
                ".kilocode/skills",
                None,
                SkillInstallStrategy::ManagedCopy,
                None,
                Some("ecosystem"),
                380,
            ),
            platform(
                "roo-code",
                "Roo Code",
                ".roo/skills",
                None,
                SkillInstallStrategy::ManagedCopy,
                None,
                Some("ecosystem"),
                390,
            ),
            platform(
                "goose",
                "Goose",
                ".config/goose/skills",
                None,
                SkillInstallStrategy::ManagedCopy,
                None,
                Some("ecosystem"),
                400,
            ),
            platform(
                "gemini-cli",
                "Gemini CLI legacy/shared",
                ".gemini/skills",
                Some("gemini-shared"),
                SkillInstallStrategy::ManagedCopy,
                Some("gemini-cli"),
                Some("ecosystem"),
                410,
            ),
            platform(
                "github-copilot",
                "GitHub Copilot",
                ".copilot/skills",
                None,
                SkillInstallStrategy::ManagedCopy,
                None,
                Some("ecosystem"),
                420,
            ),
            platform(
                "clawdbot",
                "Clawdbot",
                ".clawdbot/skills",
                None,
                SkillInstallStrategy::ManagedCopy,
                None,
                Some("ecosystem"),
                430,
            ),
            platform(
                "windsurf",
                "Windsurf",
                ".codeium/windsurf/skills",
                None,
                SkillInstallStrategy::ManagedCopy,
                None,
                Some("ecosystem"),
                440,
            ),
            platform(
                "moltbot",
                "MoltBot",
                ".moltbot/skills",
                None,
                SkillInstallStrategy::ManagedCopy,
                None,
                Some("ecosystem"),
                450,
            ),
        ]
    }

    fn sanitize_platforms(platforms: Vec<SkillPlatformConfig>) -> Vec<SkillPlatformConfig> {
        let mut seen = HashSet::new();
        let mut sanitized = platforms
            .into_iter()
            .filter(|platform| seen.insert(platform.id.to_ascii_lowercase()))
            .collect::<Vec<_>>();
        sanitized.sort_by_key(|platform| platform.sort_order);
        sanitized
    }

    #[allow(dead_code)]
    fn save_platforms(&self, platforms: &[SkillPlatformConfig]) -> Result<()> {
        self.ensure_runtime_dirs()?;
        let payload = toml::to_string_pretty(&SkillPlatformsConfigFile {
            platforms: platforms.to_vec(),
        })
        .map_err(|e| CcrError::ConfigError(e.to_string()))?;
        AtomicWriter::new(&self.platforms_path).write_string(&payload)
    }

    fn ensure_runtime_dirs(&self) -> Result<()> {
        fs::create_dir_all(&self.ccr_skills_dir).map_err(CcrError::IoError)?;
        if let Some(parent) = self.summary_cache_path.parent() {
            fs::create_dir_all(parent).map_err(CcrError::IoError)?;
        }
        Ok(())
    }

    pub fn load_platforms(&self) -> Result<Vec<SkillPlatformConfig>> {
        if !self.platforms_path.exists() {
            return Ok(Self::default_platforms());
        }

        let raw = fs::read_to_string(&self.platforms_path).map_err(CcrError::IoError)?;
        let parsed: SkillPlatformsConfigFile =
            toml::from_str(&raw).map_err(|e| CcrError::ConfigError(e.to_string()))?;
        let mut platforms = if parsed.platforms.is_empty() {
            Self::default_platforms()
        } else {
            Self::sanitize_platforms(parsed.platforms)
        };

        if platforms
            .windows(2)
            .any(|pair| pair[0].sort_order > pair[1].sort_order)
        {
            platforms.sort_by_key(|platform| platform.sort_order);
        }

        if platforms.is_empty() {
            platforms = Self::default_platforms();
        }

        Ok(platforms)
    }

    fn load_sources(&self) -> Result<Vec<SkillSourceEntry>> {
        if !self.sources_path.exists() {
            return Ok(Vec::new());
        }

        let raw = fs::read_to_string(&self.sources_path).map_err(CcrError::IoError)?;
        let parsed: SkillSourcesFile = serde_json::from_str(&raw).map_err(CcrError::JsonError)?;
        Ok(parsed.sources)
    }

    fn save_sources(&self, sources: &[SkillSourceEntry]) -> Result<()> {
        self.ensure_runtime_dirs()?;
        let payload = serde_json::to_string_pretty(&SkillSourcesFile {
            sources: sources.to_vec(),
        })
        .map_err(CcrError::JsonError)?;
        AtomicWriter::new(&self.sources_path).write_string(&payload)
    }

    fn load_summary_cache(&self) -> HashMap<String, SkillSummaryCacheEntry> {
        let raw = match fs::read_to_string(&self.summary_cache_path) {
            Ok(raw) => raw,
            Err(_) => return HashMap::new(),
        };
        serde_json::from_str::<SkillSummaryCacheFile>(&raw)
            .map(|cache| cache.entries)
            .unwrap_or_default()
    }

    fn save_summary_cache(&self, entries: &HashMap<String, SkillSummaryCacheEntry>) -> Result<()> {
        self.ensure_runtime_dirs()?;
        let payload = serde_json::to_string_pretty(&SkillSummaryCacheFile {
            entries: entries.clone(),
        })
        .map_err(CcrError::JsonError)?;
        AtomicWriter::new(&self.summary_cache_path).write_string(&payload)
    }

    fn platform_dir(&self, platform: &SkillPlatformConfig) -> PathBuf {
        self.home_dir.join(&platform.relative_path)
    }

    fn meta_path(skill_dir: &Path) -> PathBuf {
        skill_dir.join(SKILL_META_FILENAME)
    }

    fn skill_path(skill_dir: &Path) -> PathBuf {
        skill_dir.join("SKILL.md")
    }

    fn resolve_installation_file_path(
        install_path: &Path,
        requested_path: &str,
    ) -> Result<PathBuf> {
        let requested = Path::new(requested_path);
        if requested.has_root() || requested.is_absolute() {
            return Err(CcrError::ValidationError(
                "技能文件路径必须位于安装目录内".to_string(),
            ));
        }

        let canonical_install_path = install_path.canonicalize().map_err(CcrError::IoError)?;
        let canonical_target = canonical_install_path
            .join(requested)
            .canonicalize()
            .map_err(CcrError::IoError)?;

        if !canonical_target.starts_with(&canonical_install_path) {
            return Err(CcrError::ValidationError(
                "技能文件路径必须位于安装目录内".to_string(),
            ));
        }

        Ok(canonical_target)
    }

    fn read_install_meta(skill_dir: &Path) -> SkillInstallMeta {
        let meta_path = Self::meta_path(skill_dir);
        fs::read_to_string(meta_path)
            .ok()
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or_default()
    }

    fn file_mtime_ms(path: &Path) -> Option<u64> {
        path.metadata()
            .ok()?
            .modified()
            .ok()?
            .duration_since(std::time::UNIX_EPOCH)
            .ok()
            .map(|duration| duration.as_millis() as u64)
    }

    fn normalize_skill_name(name: &str) -> String {
        let normalized = name
            .trim()
            .to_lowercase()
            .chars()
            .map(|ch| {
                if ch.is_ascii_alphanumeric() {
                    ch
                } else if ch == '-' || ch == '_' {
                    '-'
                } else {
                    ' '
                }
            })
            .collect::<String>();

        normalized
            .split_whitespace()
            .filter(|segment| !segment.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    }

    fn hash_content(content: &str) -> String {
        let mut hasher = Hasher::new();
        hasher.update(content.as_bytes());
        hasher.finalize().to_hex().to_string()
    }

    fn slugify_dir_name(name: &str) -> String {
        let normalized = Self::normalize_skill_name(name);
        if normalized.is_empty() {
            "skill".into()
        } else {
            normalized
        }
    }

    fn cache_key(platform_id: &str, skill_dir: &Path) -> String {
        format!("{platform_id}::{}", skill_dir.to_string_lossy())
    }

    fn installation_id(platform_id: &str, skill_dir: &Path) -> String {
        format!(
            "ins_{}",
            &Self::hash_content(&format!("{platform_id}::{}", skill_dir.to_string_lossy()))[..16]
        )
    }

    fn now_ts() -> i64 {
        Utc::now().timestamp_millis()
    }

    fn source_ref_from_meta(meta: &SkillInstallMeta) -> Option<String> {
        meta.source_repo_id
            .clone()
            .or_else(|| meta.source_url.clone())
            .filter(|value| !value.trim().is_empty())
    }

    fn source_label_from_meta(meta: &SkillInstallMeta) -> Option<String> {
        meta.source_repo_id
            .clone()
            .or_else(|| meta.source_url.clone())
            .filter(|value| !value.trim().is_empty())
    }

    fn build_group_key(
        install_group_id: Option<&str>,
        origin: &SkillOrigin,
        source_ref: Option<&str>,
        normalized_name: &str,
        content_hash: &str,
    ) -> String {
        if let Some(group_id) = install_group_id
            && !group_id.trim().is_empty()
        {
            return group_id.to_string();
        }

        if *origin != SkillOrigin::Unknown
            && let Some(source_ref) = source_ref
            && !source_ref.trim().is_empty()
        {
            return format!(
                "src:{}:{}:{}",
                serde_json::to_string(origin).unwrap_or_else(|_| "\"unknown\"".into()),
                source_ref,
                normalized_name
            );
        }

        format!("hash:{normalized_name}:{content_hash}")
    }

    fn derived_skill_id(group_key: &str) -> String {
        if group_key.starts_with("sg_") {
            return group_key.to_string();
        }
        format!("sg_{}", &Self::hash_content(group_key)[..20])
    }

    fn read_skill_descriptor(skill_dir: &Path) -> Result<SkillDescriptor> {
        let raw = fs::read_to_string(Self::skill_path(skill_dir)).map_err(CcrError::IoError)?;
        let (metadata, description) = Skill::parse_with_fallback(&raw);
        let fallback_name = skill_dir
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("skill")
            .to_string();
        let name = Self::extract_frontmatter_name(&raw).unwrap_or(fallback_name);
        let install_meta = Self::read_install_meta(skill_dir);

        Ok(SkillDescriptor {
            skill_dir: skill_dir.to_path_buf(),
            name,
            description,
            metadata,
            raw,
            install_meta,
        })
    }

    fn extract_frontmatter_name(raw: &str) -> Option<String> {
        let trimmed = raw.trim();
        let after_prefix = trimmed.strip_prefix("---")?;
        let end_idx = after_prefix.find("---")?;
        let frontmatter = &after_prefix[..end_idx];
        for line in frontmatter.lines() {
            let line = line.trim();
            if let Some((key, value)) = line.split_once(':')
                && key.trim().eq_ignore_ascii_case("name")
            {
                let value = value.trim().trim_matches('"').trim_matches('\'');
                if !value.is_empty() {
                    return Some(value.to_string());
                }
            }
        }
        None
    }

    fn enumerate_skill_dirs(base: &Path) -> Vec<PathBuf> {
        if !base.is_dir() {
            return Vec::new();
        }

        let mut dirs = fs::read_dir(base)
            .map(|entries| {
                entries
                    .flatten()
                    .map(|entry| entry.path())
                    .filter(|path| path.is_dir() && Self::skill_path(path).exists())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        dirs.sort();
        dirs
    }

    fn write_skill_installation(
        &self,
        target: &InstallationTarget,
        raw: &str,
        meta: &SkillInstallMeta,
        force: bool,
    ) -> Result<()> {
        if target.skill_dir.exists() && !force {
            return Err(CcrError::ResourceAlreadyExists(format!(
                "技能目录已存在: {}",
                target.skill_dir.to_string_lossy()
            )));
        }

        fs::create_dir_all(&target.skill_dir).map_err(CcrError::IoError)?;
        AtomicWriter::new(Self::skill_path(&target.skill_dir)).write_string(raw)?;
        let meta_raw = serde_json::to_string_pretty(meta).map_err(CcrError::JsonError)?;
        AtomicWriter::new(Self::meta_path(&target.skill_dir)).write_string(&meta_raw)?;
        Ok(())
    }

    fn persist_install_meta(&self, skill_dir: &Path, meta: &SkillInstallMeta) -> Result<()> {
        let payload = serde_json::to_string_pretty(meta).map_err(CcrError::JsonError)?;
        AtomicWriter::new(Self::meta_path(skill_dir)).write_string(&payload)
    }

    fn invalidate_cache_paths<I>(&self, skill_dirs: I)
    where
        I: IntoIterator<Item = PathBuf>,
    {
        let mut cache = self.load_summary_cache();
        let remove_set: HashSet<String> = skill_dirs
            .into_iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect();
        cache.retain(|key, _| !remove_set.iter().any(|path| key.ends_with(path)));
        let _ = self.save_summary_cache(&cache);
    }

    fn scan_platform_installations(
        &self,
        platform: &SkillPlatformConfig,
        cache: &HashMap<String, SkillSummaryCacheEntry>,
        next_cache: &mut HashMap<String, SkillSummaryCacheEntry>,
    ) -> Result<(Vec<ScannedInstallation>, SkillPlatformSummary)> {
        let platform_dir = self.platform_dir(platform);
        let detected = platform_dir.is_dir();
        let skill_dirs = Self::enumerate_skill_dirs(&platform_dir);
        let mut installations = Vec::with_capacity(skill_dirs.len());

        for skill_dir in skill_dirs {
            let cache_key = Self::cache_key(&platform.id, &skill_dir);
            let skill_mtime_ms = Self::file_mtime_ms(&Self::skill_path(&skill_dir)).unwrap_or(0);
            let meta_mtime_ms = Self::file_mtime_ms(&Self::meta_path(&skill_dir));

            if let Some(cached) = cache.get(&cache_key)
                && cached.skill_mtime_ms == skill_mtime_ms
                && cached.meta_mtime_ms == meta_mtime_ms
            {
                next_cache.insert(cache_key.clone(), cached.clone());
                installations.push(ScannedInstallation {
                    skill_id: cached.skill_id.clone(),
                    installation_id: cached.installation_id.clone(),
                    name: cached.name.clone(),
                    description: cached.description.clone(),
                    category: cached.category.clone(),
                    tags: cached.tags.clone(),
                    version: cached.version.clone(),
                    author: cached.author.clone(),
                    origin: cached.origin.clone(),
                    source_label: cached.source_label.clone(),
                    source_ref: cached.source_ref.clone(),
                    installed_at: cached.installed_at,
                    install_path: skill_dir.clone(),
                    platform_id: platform.id.clone(),
                    platform_name: platform.display_name.clone(),
                });
                continue;
            }

            let descriptor = Self::read_skill_descriptor(&skill_dir)?;
            let content_hash = descriptor
                .install_meta
                .content_hash
                .clone()
                .unwrap_or_else(|| Self::hash_content(&descriptor.raw));
            let normalized_name = Self::normalize_skill_name(&descriptor.name);
            let source_ref = Self::source_ref_from_meta(&descriptor.install_meta);
            let group_key = Self::build_group_key(
                descriptor.install_meta.install_group_id.as_deref(),
                &descriptor.install_meta.origin,
                source_ref.as_deref(),
                &normalized_name,
                &content_hash,
            );
            let skill_id = Self::derived_skill_id(&group_key);
            let installation_id = Self::installation_id(&platform.id, &skill_dir);
            let source_label = Self::source_label_from_meta(&descriptor.install_meta);

            next_cache.insert(
                cache_key,
                SkillSummaryCacheEntry {
                    skill_id: skill_id.clone(),
                    installation_id: installation_id.clone(),
                    name: descriptor.name.clone(),
                    description: descriptor.description.clone(),
                    category: descriptor.metadata.category.clone(),
                    tags: descriptor.metadata.tags.clone(),
                    version: descriptor.metadata.version.clone(),
                    author: descriptor.metadata.author.clone(),
                    origin: descriptor.install_meta.origin.clone(),
                    source_label: source_label.clone(),
                    source_ref: source_ref.clone(),
                    install_group_id: descriptor.install_meta.install_group_id.clone(),
                    content_hash: content_hash.clone(),
                    installed_at: descriptor.install_meta.installed_at,
                    skill_mtime_ms,
                    meta_mtime_ms,
                },
            );

            installations.push(ScannedInstallation {
                skill_id,
                installation_id,
                name: descriptor.name,
                description: descriptor.description,
                category: descriptor.metadata.category,
                tags: descriptor.metadata.tags,
                version: descriptor.metadata.version,
                author: descriptor.metadata.author,
                origin: descriptor.install_meta.origin,
                source_label,
                source_ref,
                installed_at: descriptor.install_meta.installed_at,
                install_path: skill_dir,
                platform_id: platform.id.clone(),
                platform_name: platform.display_name.clone(),
            });
        }

        let summary = SkillPlatformSummary {
            id: platform.id.clone(),
            display_name: platform.display_name.clone(),
            global_skills_dir: platform_dir.to_string_lossy().to_string(),
            detected,
            installed_count: installations.len(),
            shared_dir_group: platform.shared_dir_group.clone(),
            install_strategy: platform.install_strategy.clone(),
            npx_agent_key: platform.npx_agent_key.clone(),
            category: platform.category.clone(),
            capabilities: platform.capabilities.clone(),
            sort_order: platform.sort_order,
        };

        Ok((installations, summary))
    }

    fn aggregate_installations(
        &self,
        platforms: &[SkillPlatformConfig],
    ) -> Result<(Vec<SkillRecord>, Vec<SkillPlatformSummary>)> {
        let cache = self.load_summary_cache();
        let mut next_cache = HashMap::new();
        let mut aggregates: HashMap<String, SkillAggregate> = HashMap::new();
        let mut platform_summaries = Vec::with_capacity(platforms.len());

        for platform in platforms {
            let (installations, summary) =
                self.scan_platform_installations(platform, &cache, &mut next_cache)?;
            platform_summaries.push(summary);

            for installation in installations {
                let aggregate = aggregates
                    .entry(installation.skill_id.clone())
                    .or_insert_with(|| SkillAggregate {
                        id: installation.skill_id.clone(),
                        name: installation.name.clone(),
                        description: installation.description.clone(),
                        category: installation.category.clone(),
                        tags: installation.tags.clone(),
                        version: installation.version.clone(),
                        author: installation.author.clone(),
                        origin: installation.origin.clone(),
                        source_label: installation.source_label.clone(),
                        source_ref: installation.source_ref.clone(),
                        installations: Vec::new(),
                    });

                if aggregate.description.is_none() {
                    aggregate.description = installation.description.clone();
                }
                if aggregate.category.is_none() {
                    aggregate.category = installation.category.clone();
                }
                if aggregate.version.is_none() {
                    aggregate.version = installation.version.clone();
                }
                if aggregate.author.is_none() {
                    aggregate.author = installation.author.clone();
                }
                if aggregate.source_label.is_none() {
                    aggregate.source_label = installation.source_label.clone();
                }
                if aggregate.source_ref.is_none() {
                    aggregate.source_ref = installation.source_ref.clone();
                }
                if aggregate.origin == SkillOrigin::Unknown
                    && installation.origin != SkillOrigin::Unknown
                {
                    aggregate.origin = installation.origin.clone();
                }

                let mut tag_set: HashSet<String> = aggregate.tags.iter().cloned().collect();
                for tag in &installation.tags {
                    if tag_set.insert(tag.clone()) {
                        aggregate.tags.push(tag.clone());
                    }
                }

                aggregate.installations.push(installation);
            }
        }

        self.save_summary_cache(&next_cache)?;

        let mut skills = aggregates
            .into_values()
            .map(|mut aggregate| {
                aggregate
                    .installations
                    .sort_by(|left, right| left.platform_name.cmp(&right.platform_name));
                let primary_id = aggregate
                    .installations
                    .first()
                    .map(|installation| installation.installation_id.clone());
                let install_count = aggregate.installations.len();
                let editable_installations = aggregate
                    .installations
                    .iter()
                    .map(|installation| installation.installation_id.clone())
                    .collect::<Vec<_>>();
                let installations = aggregate
                    .installations
                    .iter()
                    .map(|installation| SkillInstallationRecord {
                        is_primary: primary_id
                            .as_ref()
                            .is_some_and(|primary| primary == &installation.installation_id),
                        id: installation.installation_id.clone(),
                        platform_id: installation.platform_id.clone(),
                        platform_name: installation.platform_name.clone(),
                        install_path: installation.install_path.to_string_lossy().to_string(),
                        install_mode: SkillInstallMode::Copy,
                        installed_at: installation.installed_at,
                    })
                    .collect::<Vec<_>>();
                let targets = aggregate
                    .installations
                    .iter()
                    .map(|installation| SkillTargetRecord {
                        id: installation.installation_id.clone(),
                        platform_id: installation.platform_id.clone(),
                        platform_name: installation.platform_name.clone(),
                        target_path: installation.install_path.to_string_lossy().to_string(),
                        sync_mode: SkillInstallMode::Copy,
                        status: SkillTargetStatus::Ok,
                        synced_at: installation.installed_at,
                        last_error: None,
                        is_primary: primary_id
                            .as_ref()
                            .is_some_and(|primary| primary == &installation.installation_id),
                    })
                    .collect::<Vec<_>>();
                let last_synced_at = targets.iter().filter_map(|target| target.synced_at).max();
                let healthy_target_count = targets
                    .iter()
                    .filter(|target| matches!(target.status, SkillTargetStatus::Ok))
                    .count();
                SkillRecord {
                    id: aggregate.id,
                    name: aggregate.name,
                    description: aggregate.description,
                    category: aggregate.category,
                    tags: aggregate.tags,
                    version: aggregate.version,
                    author: aggregate.author,
                    origin: aggregate.origin,
                    source_label: aggregate.source_label.clone(),
                    source_ref: aggregate.source_ref.clone(),
                    install_count,
                    editable_installations,
                    installations,
                    targets,
                    lifecycle: SkillLifecycleSummary {
                        source_ref: aggregate.source_ref,
                        source_label: aggregate.source_label,
                        source_revision: None,
                        content_hash: None,
                        last_synced_at,
                        has_errors: healthy_target_count != install_count,
                        target_count: install_count,
                        healthy_target_count,
                    },
                }
            })
            .collect::<Vec<_>>();

        skills.sort_by_key(|skill| skill.name.to_lowercase());
        Ok((skills, platform_summaries))
    }

    fn apply_inventory_query(
        &self,
        mut skills: Vec<SkillRecord>,
        query: Option<&SkillsInventoryQuery>,
    ) -> Vec<SkillRecord> {
        let Some(query) = query else {
            return skills;
        };

        if let Some(platform_id) = query.platform.as_deref() {
            skills.retain(|skill| {
                skill
                    .installations
                    .iter()
                    .any(|installation| installation.platform_id == platform_id)
            });
        }

        if let Some(origin) = query.origin.as_ref() {
            skills.retain(|skill| &skill.origin == origin);
        }

        if let Some(source_id) = query.source_id.as_deref() {
            skills.retain(|skill| {
                skill.source_ref.as_deref() == Some(source_id)
                    || skill.source_label.as_deref() == Some(source_id)
            });
        }

        if let Some(q) = query.q.as_deref() {
            let q = q.trim().to_lowercase();
            if !q.is_empty() {
                skills.retain(|skill| {
                    skill.name.to_lowercase().contains(&q)
                        || skill
                            .description
                            .as_deref()
                            .unwrap_or_default()
                            .to_lowercase()
                            .contains(&q)
                        || skill
                            .category
                            .as_deref()
                            .unwrap_or_default()
                            .to_lowercase()
                            .contains(&q)
                        || skill
                            .author
                            .as_deref()
                            .unwrap_or_default()
                            .to_lowercase()
                            .contains(&q)
                        || skill.tags.iter().any(|tag| tag.to_lowercase().contains(&q))
                });
            }
        }

        skills
    }

    pub fn inventory(
        &self,
        query: Option<SkillsInventoryQuery>,
    ) -> Result<SkillsInventoryResponse> {
        let platforms = self.load_platforms()?;
        let (skills, platform_summaries) = self.aggregate_installations(&platforms)?;
        let skills = self.apply_inventory_query(skills, query.as_ref());
        let total = skills.len();

        Ok(SkillsInventoryResponse {
            skills,
            platforms: platform_summaries,
            total,
        })
    }

    pub fn detail(&self, skill_id: &str) -> Result<SkillRecord> {
        self.inventory(None)?
            .skills
            .into_iter()
            .find(|skill| skill.id == skill_id)
            .ok_or_else(|| CcrError::ResourceNotFound(format!("Skill '{skill_id}' not found")))
    }

    fn resolve_installation_path(
        &self,
        skill_id: &str,
        installation_id: Option<&str>,
    ) -> Result<(SkillRecord, SkillInstallationRecord)> {
        let skill = self.detail(skill_id)?;
        let installation = if let Some(installation_id) = installation_id {
            skill
                .installations
                .iter()
                .find(|installation| installation.id == installation_id)
                .cloned()
        } else {
            skill
                .installations
                .iter()
                .find(|installation| installation.is_primary)
                .cloned()
                .or_else(|| skill.installations.first().cloned())
        }
        .ok_or_else(|| {
            CcrError::ResourceNotFound(format!("Installation for '{skill_id}' not found"))
        })?;

        Ok((skill, installation))
    }

    pub fn content_get(
        &self,
        skill_id: &str,
        installation_id: Option<&str>,
    ) -> Result<SkillContent> {
        let (skill, installation) = self.resolve_installation_path(skill_id, installation_id)?;
        let raw = fs::read_to_string(Self::skill_path(Path::new(&installation.install_path)))
            .map_err(CcrError::IoError)?;

        Ok(SkillContent {
            skill_id: skill.id,
            installation_id: installation.id,
            name: skill.name,
            description: skill.description,
            category: skill.category,
            tags: skill.tags,
            content: raw.clone(),
            raw,
            skill_dir: installation.install_path,
        })
    }

    pub fn files_list(
        &self,
        skill_id: &str,
        installation_id: Option<&str>,
    ) -> Result<Vec<SkillFileEntry>> {
        let (_, installation) = self.resolve_installation_path(skill_id, installation_id)?;
        let install_path = PathBuf::from(&installation.install_path);
        let mut entries = Vec::new();

        for entry in WalkDir::new(&install_path) {
            let Ok(entry) = entry else {
                continue;
            };
            if entry.path() == install_path {
                continue;
            }
            let Ok(relative) = entry.path().strip_prefix(&install_path) else {
                continue;
            };
            let Ok(metadata) = entry.metadata() else {
                continue;
            };
            entries.push(SkillFileEntry {
                path: relative.to_string_lossy().replace('\\', "/"),
                size: if metadata.is_file() {
                    metadata.len()
                } else {
                    0
                },
                is_dir: metadata.is_dir(),
            });
        }

        entries.sort_by(|left, right| left.path.cmp(&right.path));
        Ok(entries)
    }

    pub fn file_get(
        &self,
        skill_id: &str,
        installation_id: Option<&str>,
        path: &str,
    ) -> Result<SkillFileContent> {
        let (_, installation) = self.resolve_installation_path(skill_id, installation_id)?;
        let install_path = PathBuf::from(&installation.install_path);
        let target = Self::resolve_installation_file_path(&install_path, path)?;
        let content = fs::read_to_string(&target).map_err(CcrError::IoError)?;
        Ok(SkillFileContent {
            skill_id: skill_id.to_string(),
            installation_id: installation.id,
            path: path.to_string(),
            content,
        })
    }

    pub fn onboarding_candidates(&self) -> Result<Vec<SkillsOnboardingCandidate>> {
        let skills = self.inventory(None)?.skills;
        let candidates = skills
            .into_iter()
            .filter(|skill| skill.origin == SkillOrigin::Unknown || skill.source_ref.is_none())
            .map(|skill| SkillsOnboardingCandidate {
                skill_id: skill.id,
                name: skill.name,
                platform_ids: skill
                    .installations
                    .iter()
                    .map(|installation| installation.platform_id.clone())
                    .collect(),
                installation_ids: skill
                    .installations
                    .iter()
                    .map(|installation| installation.id.clone())
                    .collect(),
                installation_paths: skill
                    .installations
                    .iter()
                    .map(|installation| installation.install_path.clone())
                    .collect(),
                reason: if skill.source_ref.is_none() {
                    "missing_source".to_string()
                } else {
                    "unknown_origin".to_string()
                },
            })
            .collect();
        Ok(candidates)
    }

    pub fn content_save(
        &self,
        skill_id: &str,
        installation_id: &str,
        raw: &str,
    ) -> Result<SkillContent> {
        let (_, installation) = self.resolve_installation_path(skill_id, Some(installation_id))?;
        let install_path = PathBuf::from(&installation.install_path);
        AtomicWriter::new(Self::skill_path(&install_path)).write_string(raw)?;

        let mut meta = Self::read_install_meta(&install_path);
        let now = Self::now_ts();
        meta.install_group_id = Some(
            meta.install_group_id
                .clone()
                .unwrap_or_else(|| skill_id.to_string()),
        );
        meta.install_mode = SkillInstallMode::Copy;
        meta.content_hash = Some(Self::hash_content(raw));
        meta.updated_at = Some(now);
        if meta.installed_at.is_none() {
            meta.installed_at = Some(now);
        }
        self.persist_install_meta(&install_path, &meta)?;
        self.invalidate_cache_paths([install_path.clone()]);

        self.content_get(skill_id, Some(installation_id))
    }

    fn source_checkout_dir(&self, source_id: &str) -> PathBuf {
        self.ccr_skills_dir.join("sources").join(source_id)
    }

    fn run_git(args: &[&str], cwd: Option<&Path>) -> Result<String> {
        let mut command = silent_command("git");
        command.args(args);
        if let Some(cwd) = cwd {
            command.current_dir(cwd);
        }

        let output = command
            .output()
            .map_err(|e| CcrError::ExternalCommandError(format!("执行 git 命令失败: {e}")))?;

        if !output.status.success() {
            return Err(CcrError::ExternalCommandError(
                String::from_utf8_lossy(&output.stderr).trim().to_string(),
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    fn read_source_manifest(root: &Path) -> SkillsSourceManifest {
        let manifest_path = root.join("skills.toml");
        fs::read_to_string(manifest_path)
            .ok()
            .and_then(|raw| toml::from_str(&raw).ok())
            .unwrap_or_default()
    }

    fn resolve_source_root(root: &Path) -> PathBuf {
        let manifest = Self::read_source_manifest(root);
        if let Some(skills_dir) = manifest.skills_dir
            && !skills_dir.trim().is_empty()
        {
            let candidate = root.join(skills_dir);
            if candidate.is_dir() {
                return candidate;
            }
        }

        let fallback = root.join("skills");
        if fallback.is_dir() {
            fallback
        } else {
            root.to_path_buf()
        }
    }

    fn source_base_path(source: &SkillSourceEntry) -> PathBuf {
        match source.source_type {
            SkillSourceType::Git => source
                .checkout_path
                .as_deref()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from(&source.location)),
            SkillSourceType::Local => PathBuf::from(&source.location),
        }
    }

    fn source_record(&self, source: &SkillSourceEntry) -> SkillSourceRecord {
        let base_path = Self::source_base_path(source);
        let skills_root = Self::resolve_source_root(&base_path);
        let manifest = Self::read_source_manifest(&base_path);
        let skills = Self::enumerate_skill_dirs(&skills_root)
            .into_iter()
            .filter_map(|skill_dir| {
                let descriptor = Self::read_skill_descriptor(&skill_dir).ok()?;
                Some(SkillSourceSkillRecord {
                    id: Self::slugify_dir_name(&descriptor.name),
                    name: descriptor.name,
                    description: descriptor.description,
                    category: descriptor.metadata.category,
                    tags: descriptor.metadata.tags,
                    install_ref: skill_dir.to_string_lossy().to_string(),
                })
            })
            .collect::<Vec<_>>();
        let skill_count = skills.len();

        let health = if !base_path.exists() {
            SkillSourceHealth::Missing
        } else if skills_root.exists() {
            SkillSourceHealth::Ok
        } else {
            SkillSourceHealth::Error
        };

        SkillSourceRecord {
            id: source.id.clone(),
            source_type: source.source_type.clone(),
            name: manifest.name.unwrap_or_else(|| {
                if source.source_type == SkillSourceType::Git {
                    source.location.clone()
                } else {
                    base_path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("Local Source")
                        .to_string()
                }
            }),
            description: manifest.description,
            location: source.location.clone(),
            skills_root: skills_root.to_string_lossy().to_string(),
            skill_count,
            last_synced_at: source.last_synced_at.clone(),
            health,
            skills,
        }
    }

    fn resolve_source_skill_dir(
        &self,
        source: &SkillSourceEntry,
        skill_name: Option<&str>,
    ) -> Result<PathBuf> {
        let base_path = Self::source_base_path(source);
        if !base_path.exists() {
            return Err(CcrError::ResourceNotFound(format!(
                "Source '{}' is missing",
                source.id
            )));
        }

        let skills_root = Self::resolve_source_root(&base_path);
        let skill_dirs = Self::enumerate_skill_dirs(&skills_root);
        if let Some(skill_name) = skill_name {
            let normalized = Self::normalize_skill_name(skill_name);
            skill_dirs
                .into_iter()
                .find(|dir| {
                    dir.file_name()
                        .and_then(|name| name.to_str())
                        .map(Self::normalize_skill_name)
                        .is_some_and(|name| name == normalized)
                })
                .ok_or_else(|| {
                    CcrError::ResourceNotFound(format!(
                        "Skill '{}' not found in source '{}'",
                        skill_name, source.id
                    ))
                })
        } else if skill_dirs.len() == 1 {
            Ok(skill_dirs[0].clone())
        } else {
            Err(CcrError::ConfigError(format!(
                "Source '{}' contains multiple skills, source_skill_id is required",
                source.id
            )))
        }
    }

    fn resolve_local_skill_dir(
        &self,
        source_ref: &str,
        skill_name: Option<&str>,
    ) -> Result<PathBuf> {
        let source_path = PathBuf::from(source_ref);
        if source_path.is_file()
            && source_path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.eq_ignore_ascii_case("SKILL.md"))
        {
            return source_path
                .parent()
                .map(Path::to_path_buf)
                .ok_or_else(|| CcrError::ConfigError("无效的 SKILL.md 路径".into()));
        }

        if Self::skill_path(&source_path).exists() {
            return Ok(source_path);
        }

        let source = SkillSourceEntry {
            id: "local".into(),
            source_type: SkillSourceType::Local,
            location: source_ref.into(),
            checkout_path: None,
            last_synced_at: None,
        };
        self.resolve_source_skill_dir(&source, skill_name)
    }

    async fn fetch_github_skill_payload(
        &self,
        source_ref: &str,
        skill_name: Option<&str>,
        origin: SkillOrigin,
    ) -> Result<InstallPayload> {
        if source_ref.contains("raw.githubusercontent.com") {
            let raw = HTTP_CLIENT
                .get(source_ref)
                .send()
                .await
                .map_err(|e| CcrError::NetworkError(e.to_string()))?
                .text()
                .await
                .map_err(|e| CcrError::NetworkError(e.to_string()))?;
            let descriptor_name = Self::extract_frontmatter_name(&raw)
                .or_else(|| skill_name.map(ToString::to_string))
                .unwrap_or_else(|| "skill".into());
            return Ok(InstallPayload {
                requested_name: descriptor_name.clone(),
                dir_name: Self::slugify_dir_name(&descriptor_name),
                raw,
                origin,
                source_url: Some(source_ref.to_string()),
                source_repo_id: None,
                install_group_id: None,
            });
        }

        let parsed = Self::parse_github_reference(source_ref, skill_name)?;
        let raw_url = format!(
            "https://raw.githubusercontent.com/{}/{}/{}/SKILL.md",
            parsed.owner, parsed.repo, parsed.skill_path
        );
        let raw = HTTP_CLIENT
            .get(&raw_url)
            .send()
            .await
            .map_err(|e| CcrError::NetworkError(e.to_string()))?
            .text()
            .await
            .map_err(|e| CcrError::NetworkError(e.to_string()))?;

        let descriptor_name =
            Self::extract_frontmatter_name(&raw).unwrap_or_else(|| parsed.skill_name.clone());

        Ok(InstallPayload {
            requested_name: descriptor_name.clone(),
            dir_name: Self::slugify_dir_name(&parsed.skill_name),
            raw,
            origin,
            source_url: Some(parsed.repo_url),
            source_repo_id: Some(format!("{}/{}", parsed.owner, parsed.repo)),
            install_group_id: None,
        })
    }

    fn parse_github_reference(
        source_ref: &str,
        skill_name: Option<&str>,
    ) -> Result<ParsedGithubRef> {
        let source_ref = source_ref.trim().trim_end_matches('/');
        if source_ref.is_empty() {
            return Err(CcrError::ConfigError("GitHub source is empty".into()));
        }

        let trimmed = if let Some(value) = source_ref.strip_prefix("https://github.com/") {
            value
        } else if let Some(value) = source_ref.strip_prefix("http://github.com/") {
            value
        } else {
            source_ref
        };

        let parts = trimmed.split('/').collect::<Vec<_>>();
        if parts.len() < 2 {
            return Err(CcrError::ConfigError(format!(
                "Invalid GitHub source: {source_ref}"
            )));
        }

        let owner = parts[0].to_string();
        let repo_and_skill = parts[1];
        let (repo, skill_from_repo) = if let Some((repo, skill)) = repo_and_skill.split_once('@') {
            (repo.to_string(), Some(skill.to_string()))
        } else {
            (repo_and_skill.to_string(), None)
        };
        let skill_name = skill_name
            .map(ToString::to_string)
            .or(skill_from_repo)
            .unwrap_or_else(|| repo.clone());
        let skill_path = Self::slugify_dir_name(&skill_name);

        Ok(ParsedGithubRef {
            owner: owner.clone(),
            repo: repo.clone(),
            repo_url: format!("https://github.com/{owner}/{repo}"),
            skill_name,
            skill_path,
        })
    }

    pub fn sources_list(&self) -> Result<Vec<SkillSourceRecord>> {
        let mut sources = self
            .load_sources()?
            .into_iter()
            .map(|source| self.source_record(&source))
            .collect::<Vec<_>>();
        sources.sort_by_key(|source| source.name.to_lowercase());
        Ok(sources)
    }

    pub fn source_add_local(&self, path: &str) -> Result<SkillSourceRecord> {
        let source_path = PathBuf::from(path);
        if !source_path.exists() {
            return Err(CcrError::ResourceNotFound(format!(
                "Local source '{}' not found",
                path
            )));
        }

        let mut sources = self.load_sources()?;
        let location = source_path.to_string_lossy().to_string();
        if let Some(existing) = sources.iter().find(|source| source.location == location) {
            return Ok(self.source_record(existing));
        }

        let source = SkillSourceEntry {
            id: format!("src_{}", &Self::hash_content(&location)[..16]),
            source_type: SkillSourceType::Local,
            location,
            checkout_path: None,
            last_synced_at: Some(Utc::now().to_rfc3339()),
        };
        sources.push(source.clone());
        self.save_sources(&sources)?;
        Ok(self.source_record(&source))
    }

    pub fn source_add_git(&self, url: &str) -> Result<SkillSourceRecord> {
        self.ensure_runtime_dirs()?;
        let mut sources = self.load_sources()?;
        if let Some(existing) = sources.iter().find(|source| source.location == url) {
            return Ok(self.source_record(existing));
        }

        let source_id = format!("src_{}", &Self::hash_content(url)[..16]);
        let checkout_path = self.source_checkout_dir(&source_id);
        if !checkout_path.exists() {
            if let Some(parent) = checkout_path.parent() {
                fs::create_dir_all(parent).map_err(CcrError::IoError)?;
            }
            let checkout_str = checkout_path.to_string_lossy().to_string();
            Self::run_git(&["clone", url, &checkout_str], None)?;
        }

        let source = SkillSourceEntry {
            id: source_id,
            source_type: SkillSourceType::Git,
            location: url.to_string(),
            checkout_path: Some(checkout_path.to_string_lossy().to_string()),
            last_synced_at: Some(Utc::now().to_rfc3339()),
        };
        sources.push(source.clone());
        self.save_sources(&sources)?;
        Ok(self.source_record(&source))
    }

    pub fn source_sync(&self, source_id: &str) -> Result<SkillSourceRecord> {
        let mut sources = self.load_sources()?;
        let source_index = sources
            .iter()
            .position(|source| source.id == source_id)
            .ok_or_else(|| CcrError::ResourceNotFound(format!("Source '{source_id}' not found")))?;

        let source_snapshot = sources[source_index].clone();
        if source_snapshot.source_type == SkillSourceType::Git {
            let checkout = Self::source_base_path(&source_snapshot);
            if !checkout.exists() {
                return Err(CcrError::ResourceNotFound(format!(
                    "Source checkout missing: {}",
                    checkout.to_string_lossy()
                )));
            }
            Self::run_git(&["pull", "--ff-only"], Some(&checkout))?;
        }

        sources[source_index].last_synced_at = Some(Utc::now().to_rfc3339());
        let updated = sources[source_index].clone();
        self.save_sources(&sources)?;
        Ok(self.source_record(&updated))
    }

    pub fn source_remove(&self, source_id: &str) -> Result<()> {
        let mut sources = self.load_sources()?;
        let removed = sources
            .iter()
            .find(|source| source.id == source_id)
            .cloned()
            .ok_or_else(|| CcrError::ResourceNotFound(format!("Source '{source_id}' not found")))?;

        sources.retain(|source| source.id != source_id);
        self.save_sources(&sources)?;

        if removed.source_type == SkillSourceType::Git {
            let checkout = Self::source_base_path(&removed);
            if checkout.exists() {
                fs::remove_dir_all(checkout).map_err(CcrError::IoError)?;
            }
        }

        Ok(())
    }

    fn resolve_install_targets(
        &self,
        target_platforms: &[String],
        dir_name: &str,
    ) -> Result<Vec<InstallationTarget>> {
        let platforms = self.load_platforms()?;
        let requested = if target_platforms.is_empty() {
            return Err(CcrError::ConfigError("target_platforms 不能为空".into()));
        } else {
            target_platforms.iter().collect::<HashSet<_>>()
        };

        let mut targets = Vec::new();
        for platform in platforms {
            if requested.contains(&platform.id) {
                let skill_dir = self.platform_dir(&platform).join(dir_name);
                targets.push(InstallationTarget {
                    platform,
                    skill_dir,
                });
            }
        }

        if targets.is_empty() {
            return Err(CcrError::ResourceNotFound(
                "No matching target platforms".into(),
            ));
        }

        Ok(targets)
    }

    fn platform_detected(&self, platform: &SkillPlatformConfig) -> bool {
        self.platform_dir(platform).is_dir()
    }

    fn read_description_from_raw(raw: &str) -> Option<String> {
        let (_, description) = Skill::parse_with_fallback(raw);
        description
    }

    fn build_npx_install_command(
        &self,
        request: &SkillsInstallRequest,
        payload: &InstallPayload,
        targets: &[InstallationTarget],
        skip_confirmation: bool,
    ) -> Result<String> {
        let mut parts = vec![
            "npx".to_string(),
            "skills".to_string(),
            "add".to_string(),
            request.source_ref.clone(),
        ];

        let selected_skills = if request.selected_skills.is_empty() {
            request
                .source_skill_id
                .clone()
                .into_iter()
                .collect::<Vec<_>>()
        } else {
            request.selected_skills.clone()
        };

        let agent_keys = targets
            .iter()
            .filter_map(|target| target.platform.npx_agent_key.clone())
            .collect::<Vec<_>>();

        let scope = request.scope.as_deref().unwrap_or("global");
        if scope.eq_ignore_ascii_case("global") {
            parts.push("--global".into());
        }

        if request.copy_mode.unwrap_or(true) {
            parts.push("--copy".into());
        }

        if request.all_mode.unwrap_or(false) {
            parts.push("--all".into());
        } else {
            if !agent_keys.is_empty() {
                parts.push("--agent".into());
                parts.extend(agent_keys);
            }
            if !selected_skills.is_empty() {
                parts.push("--skill".into());
                parts.extend(selected_skills);
            } else if request.source_skill_id.is_none() {
                parts.push("--skill".into());
                parts.push(payload.dir_name.clone());
            }
        }

        if skip_confirmation {
            parts.push("--yes".into());
        }

        Ok(parts.join(" "))
    }

    fn run_npx_install_command(&self, command: &str) -> Result<()> {
        let output = if cfg!(windows) {
            silent_command("cmd").args(["/C", command]).output()
        } else {
            silent_command("sh").args(["-lc", command]).output()
        }
        .map_err(|error| CcrError::ExternalCommandError(error.to_string()))?;

        if output.status.success() {
            return Ok(());
        }

        Err(CcrError::ExternalCommandError({
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            if stderr.is_empty() {
                String::from_utf8_lossy(&output.stdout).trim().to_string()
            } else {
                stderr
            }
        }))
    }

    fn find_existing_skill_id(
        &self,
        origin: &SkillOrigin,
        source_ref: Option<&str>,
        name: &str,
    ) -> Result<Option<String>> {
        let inventory = self.inventory(None)?;
        let normalized_name = Self::normalize_skill_name(name);
        Ok(inventory.skills.into_iter().find_map(|skill| {
            let same_source = match (source_ref, skill.source_ref.as_deref()) {
                (Some(left), Some(right)) => left == right,
                _ => false,
            };
            let same_name = Self::normalize_skill_name(&skill.name) == normalized_name;
            if &skill.origin == origin && same_source && same_name {
                Some(skill.id)
            } else {
                None
            }
        }))
    }

    async fn resolve_install_payload(
        &self,
        request: &SkillsInstallRequest,
    ) -> Result<InstallPayload> {
        match request.source_kind.as_str() {
            "marketplace" => {
                self.fetch_github_skill_payload(
                    &request.source_ref,
                    request.source_skill_id.as_deref(),
                    SkillOrigin::Marketplace,
                )
                .await
            }
            "github" => {
                self.fetch_github_skill_payload(
                    &request.source_ref,
                    request.source_skill_id.as_deref(),
                    SkillOrigin::Github,
                )
                .await
            }
            "npx" => {
                self.fetch_github_skill_payload(
                    &request.source_ref,
                    request.source_skill_id.as_deref(),
                    SkillOrigin::Npx,
                )
                .await
            }
            "local" => {
                let skill_dir = self.resolve_local_skill_dir(
                    &request.source_ref,
                    request.source_skill_id.as_deref(),
                )?;
                let descriptor = Self::read_skill_descriptor(&skill_dir)?;
                Ok(InstallPayload {
                    requested_name: descriptor.name.clone(),
                    dir_name: Self::slugify_dir_name(&descriptor.name),
                    raw: descriptor.raw,
                    origin: SkillOrigin::Local,
                    source_url: None,
                    source_repo_id: None,
                    install_group_id: descriptor.install_meta.install_group_id,
                })
            }
            "source" => {
                let sources = self.load_sources()?;
                let source = sources
                    .iter()
                    .find(|source| source.id == request.source_ref)
                    .ok_or_else(|| {
                        CcrError::ResourceNotFound(format!(
                            "Source '{}' not found",
                            request.source_ref
                        ))
                    })?;
                let skill_dir =
                    self.resolve_source_skill_dir(source, request.source_skill_id.as_deref())?;
                let descriptor = Self::read_skill_descriptor(&skill_dir)?;
                Ok(InstallPayload {
                    requested_name: descriptor.name.clone(),
                    dir_name: Self::slugify_dir_name(&descriptor.name),
                    raw: descriptor.raw,
                    origin: if source.source_type == SkillSourceType::Git {
                        SkillOrigin::Repo
                    } else {
                        SkillOrigin::Local
                    },
                    source_url: if source.source_type == SkillSourceType::Git {
                        Some(source.location.clone())
                    } else {
                        None
                    },
                    source_repo_id: Some(source.id.clone()),
                    install_group_id: descriptor.install_meta.install_group_id,
                })
            }
            other => Err(CcrError::ConfigError(format!(
                "Unsupported source_kind: {other}"
            ))),
        }
    }

    pub async fn install(&self, request: SkillsInstallRequest) -> Result<SkillOperationResponse> {
        let mut payload = self.resolve_install_payload(&request).await?;
        if payload.install_group_id.is_none() {
            payload.install_group_id = self.find_existing_skill_id(
                &payload.origin,
                payload
                    .source_repo_id
                    .as_deref()
                    .or(payload.source_url.as_deref()),
                &payload.requested_name,
            )?;
        }
        if payload.install_group_id.is_none() {
            let group_key = Self::build_group_key(
                None,
                &payload.origin,
                payload
                    .source_repo_id
                    .as_deref()
                    .or(payload.source_url.as_deref()),
                &Self::normalize_skill_name(&payload.requested_name),
                &Self::hash_content(&payload.raw),
            );
            payload.install_group_id = Some(Self::derived_skill_id(&group_key));
        }

        let targets = self.resolve_install_targets(&request.target_platforms, &payload.dir_name)?;
        let now = Self::now_ts();
        let content_hash = Self::hash_content(&payload.raw);
        let mut touched_paths = Vec::new();
        let mut results = Vec::new();

        let npx_enabled = request.source_kind.eq_ignore_ascii_case("npx")
            && self.npx_status().available
            && request.copy_mode.unwrap_or(true);

        let (npx_targets, direct_targets): (Vec<_>, Vec<_>) = if npx_enabled {
            targets.into_iter().partition(|target| {
                target.platform.npx_agent_key.is_some() && self.platform_detected(&target.platform)
            })
        } else {
            (Vec::new(), targets)
        };

        if !npx_targets.is_empty() {
            let command = self.build_npx_install_command(&request, &payload, &npx_targets, true)?;
            let command_result = self.run_npx_install_command(&command);
            let command_error = command_result.err().map(|error| error.to_string());

            for target in &npx_targets {
                let meta = SkillInstallMeta {
                    install_group_id: payload.install_group_id.clone(),
                    origin: SkillOrigin::Npx,
                    source_url: Some(
                        payload
                            .source_url
                            .clone()
                            .unwrap_or_else(|| request.source_ref.clone()),
                    ),
                    source_repo_id: payload.source_repo_id.clone(),
                    install_mode: SkillInstallMode::Copy,
                    installed_at: Some(now),
                    updated_at: Some(now),
                    content_hash: Some(content_hash.clone()),
                };

                let result = if command_error.is_none() {
                    if target.skill_dir.exists() {
                        self.persist_install_meta(&target.skill_dir, &meta)
                    } else {
                        Err(CcrError::ResourceNotFound(format!(
                            "npx install target not found: {}",
                            target.skill_dir.to_string_lossy()
                        )))
                    }
                } else {
                    Err(CcrError::ExternalCommandError(
                        command_error
                            .clone()
                            .unwrap_or_else(|| "npx install failed".into()),
                    ))
                };

                if result.is_ok() {
                    touched_paths.push(target.skill_dir.clone());
                }
                results.push(crate::models::skills::SkillOperationResult {
                    agent: target.platform.id.clone(),
                    ok: result.is_ok(),
                    message: result.err().map(|error| error.to_string()),
                });
            }
        }

        for target in direct_targets {
            let meta = SkillInstallMeta {
                install_group_id: payload.install_group_id.clone(),
                origin: payload.origin.clone(),
                source_url: payload.source_url.clone(),
                source_repo_id: payload.source_repo_id.clone(),
                install_mode: SkillInstallMode::Copy,
                installed_at: Some(now),
                updated_at: Some(now),
                content_hash: Some(content_hash.clone()),
            };

            let result = self.write_skill_installation(&target, &payload.raw, &meta, request.force);
            if result.is_ok() {
                touched_paths.push(target.skill_dir.clone());
            }
            results.push(crate::models::skills::SkillOperationResult {
                agent: target.platform.id,
                ok: result.is_ok(),
                message: result.err().map(|error| error.to_string()),
            });
        }

        if !touched_paths.is_empty() {
            self.invalidate_cache_paths(touched_paths);
        }

        Ok(SkillOperationResponse { results })
    }

    pub async fn prepare_install(
        &self,
        request: SkillsInstallRequest,
    ) -> Result<SkillsInstallReviewResponse> {
        let payload = self.resolve_install_payload(&request).await?;
        let targets = self.resolve_install_targets(&request.target_platforms, &payload.dir_name)?;
        let npx = if request.source_kind.eq_ignore_ascii_case("npx") {
            Some(self.npx_capabilities()?)
        } else {
            None
        };

        let mut warnings = Vec::new();
        if request.source_kind.eq_ignore_ascii_case("npx") {
            if npx.as_ref().is_some_and(|cap| !cap.status.available) {
                warnings.push(
                    "npx is unavailable. CCR will use the internal managed-copy install flow."
                        .into(),
                );
            }

            let fallback_targets = targets
                .iter()
                .filter(|target| target.platform.npx_agent_key.is_none())
                .map(|target| target.platform.display_name.clone())
                .collect::<Vec<_>>();
            if !fallback_targets.is_empty() {
                warnings.push(format!(
                    "Direct npx install is not supported for {}. CCR will materialize copies for those targets.",
                    fallback_targets.join(", ")
                ));
            }
        }

        let mut command_previews = Vec::new();
        if request.source_kind.eq_ignore_ascii_case("npx")
            && npx.as_ref().is_some_and(|cap| cap.status.available)
        {
            let supported_targets = targets
                .iter()
                .filter(|target| target.platform.npx_agent_key.is_some())
                .cloned()
                .collect::<Vec<_>>();
            if !supported_targets.is_empty() {
                command_previews.push(SkillsInstallCommandPreview {
                    kind: "npx".into(),
                    label: "Official npx install".into(),
                    command: self.build_npx_install_command(
                        &request,
                        &payload,
                        &supported_targets,
                        false,
                    )?,
                    platforms: supported_targets
                        .iter()
                        .map(|target| target.platform.id.clone())
                        .collect(),
                });
            }
        }

        command_previews.push(SkillsInstallCommandPreview {
            kind: "ccr".into(),
            label: "CCR managed install".into(),
            command: format!(
                "skills_confirm_install {} -> {}",
                request.source_kind, payload.dir_name
            ),
            platforms: targets
                .iter()
                .map(|target| target.platform.id.clone())
                .collect(),
        });

        Ok(SkillsInstallReviewResponse {
            source: SkillsInstallReviewSource {
                source_kind: request.source_kind,
                source_ref: request.source_ref,
                source_skill_id: request
                    .source_skill_id
                    .or_else(|| request.selected_skills.first().cloned()),
                resolved_name: payload.requested_name,
                resolved_dir_name: payload.dir_name,
                origin: payload.origin,
                description: Self::read_description_from_raw(&payload.raw),
            },
            targets: targets
                .into_iter()
                .map(|target| SkillsInstallReviewTarget {
                    platform_id: target.platform.id.clone(),
                    platform_name: target.platform.display_name.clone(),
                    detected: self.platform_detected(&target.platform),
                    target_path: target.skill_dir.to_string_lossy().to_string(),
                    shared_dir_group: target.platform.shared_dir_group.clone(),
                    install_strategy: target.platform.install_strategy.clone(),
                    direct_npx_supported: target.platform.npx_agent_key.is_some(),
                    npx_agent_key: target.platform.npx_agent_key.clone(),
                })
                .collect(),
            warnings,
            command_previews,
            npx,
        })
    }

    pub async fn sync(&self, request: SkillsSyncRequest) -> Result<SkillOperationResponse> {
        let (_, installation) =
            self.resolve_installation_path(&request.skill_id, request.installation_id.as_deref())?;
        let installation_path = PathBuf::from(&installation.install_path);
        let raw =
            fs::read_to_string(Self::skill_path(&installation_path)).map_err(CcrError::IoError)?;
        let descriptor = Self::read_skill_descriptor(&installation_path)?;
        let targets = self.resolve_install_targets(
            &request.target_platforms,
            &Self::slugify_dir_name(&descriptor.name),
        )?;
        let now = Self::now_ts();
        let content_hash = Self::hash_content(&raw);
        let mut touched_paths = Vec::new();
        let mut results = Vec::new();

        for target in targets {
            let mut meta = descriptor.install_meta.clone();
            meta.install_group_id = Some(request.skill_id.clone());
            meta.install_mode = SkillInstallMode::Copy;
            meta.updated_at = Some(now);
            if meta.installed_at.is_none() {
                meta.installed_at = Some(now);
            }
            meta.content_hash = Some(content_hash.clone());

            let target_dir = if target.platform.id == installation.platform_id {
                installation_path.clone()
            } else {
                target.skill_dir.clone()
            };
            let target = InstallationTarget {
                platform: target.platform,
                skill_dir: target_dir.clone(),
            };

            let result = self.write_skill_installation(&target, &raw, &meta, true);
            if result.is_ok() {
                touched_paths.push(target_dir);
            }
            results.push(crate::models::skills::SkillOperationResult {
                agent: target.platform.id,
                ok: result.is_ok(),
                message: result.err().map(|error| error.to_string()),
            });
        }

        if !touched_paths.is_empty() {
            self.invalidate_cache_paths(touched_paths);
        }

        Ok(SkillOperationResponse { results })
    }

    pub fn remove_installation(
        &self,
        skill_id: &str,
        installation_id: &str,
    ) -> Result<SkillOperationResponse> {
        let (_, installation) = self.resolve_installation_path(skill_id, Some(installation_id))?;
        let install_path = PathBuf::from(&installation.install_path);
        fs::remove_dir_all(&install_path).map_err(CcrError::IoError)?;
        self.invalidate_cache_paths([install_path]);

        Ok(SkillOperationResponse {
            results: vec![crate::models::skills::SkillOperationResult {
                agent: installation.platform_id,
                ok: true,
                message: None,
            }],
        })
    }

    pub fn remove_skill(&self, skill_id: &str) -> Result<SkillOperationResponse> {
        let skill = self.detail(skill_id)?;
        let mut touched_paths = Vec::new();
        let mut results = Vec::new();

        for installation in skill.installations {
            let path = PathBuf::from(&installation.install_path);
            let result = fs::remove_dir_all(&path).map_err(CcrError::IoError);
            if result.is_ok() {
                touched_paths.push(path);
            }
            results.push(crate::models::skills::SkillOperationResult {
                agent: installation.platform_id,
                ok: result.is_ok(),
                message: result.err().map(|error| error.to_string()),
            });
        }

        if !touched_paths.is_empty() {
            self.invalidate_cache_paths(touched_paths);
        }

        Ok(SkillOperationResponse { results })
    }

    pub async fn marketplace_list(
        &self,
        query: Option<String>,
        page: usize,
        page_size: usize,
    ) -> Result<MarketplaceListResponse> {
        let items = if let Some(query_text) = query.as_deref() {
            let query_text = query_text.trim();
            if query_text.is_empty() {
                self.load_bundled_featured_marketplace_items()?
            } else {
                self.search_marketplace_items(query_text).await?
            }
        } else {
            self.fetch_trending_marketplace_items().await?
        };

        let total = items.len();
        let page = page.max(1);
        let page_size = page_size.max(1);
        let start = (page - 1) * page_size;
        let items = items.into_iter().skip(start).take(page_size).collect();

        Ok(MarketplaceListResponse {
            items,
            total,
            page,
            page_size,
            cached: false,
        })
    }

    async fn fetch_trending_marketplace_items(&self) -> Result<Vec<MarketplaceSkill>> {
        let response = HTTP_CLIENT.get(DEFAULT_MARKETPLACE_URL).send().await;
        match response {
            Ok(response) if response.status().is_success() => {
                let raw = response
                    .text()
                    .await
                    .map_err(|e| CcrError::NetworkError(e.to_string()))?;
                match serde_json::from_str::<serde_json::Value>(&raw) {
                    Ok(payload) => {
                        let raw_items = payload
                            .get("skills")
                            .and_then(|value| value.as_array())
                            .cloned()
                            .or_else(|| payload.as_array().cloned())
                            .unwrap_or_default();
                        let items = raw_items
                            .into_iter()
                            .filter_map(|item| Self::parse_marketplace_item(&item))
                            .collect::<Vec<_>>();
                        if items.is_empty() {
                            self.load_bundled_featured_marketplace_items()
                        } else {
                            Ok(items)
                        }
                    }
                    Err(_) => self.load_bundled_featured_marketplace_items(),
                }
            }
            _ => self.load_bundled_featured_marketplace_items(),
        }
    }

    async fn search_marketplace_items(&self, query: &str) -> Result<Vec<MarketplaceSkill>> {
        let url = format!(
            "https://skills.sh/api/search?q={}&limit=50",
            urlencoding::encode(query)
        );
        let response = HTTP_CLIENT
            .get(url)
            .send()
            .await
            .map_err(|e| CcrError::NetworkError(e.to_string()))?;
        let payload: serde_json::Value = response
            .json()
            .await
            .map_err(|e| CcrError::NetworkError(e.to_string()))?;
        Ok(payload
            .get("skills")
            .and_then(|value| value.as_array())
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|item| {
                let source = item.get("source")?.as_str()?.to_string();
                let name = item
                    .get("skillId")
                    .or_else(|| item.get("name"))
                    .and_then(|value| value.as_str())
                    .map(ToString::to_string);
                let mut segments = source.split('/');
                let owner = segments.next()?.to_string();
                let repo = segments.next()?.to_string();
                let package = match name.as_deref() {
                    Some(skill_name) if !skill_name.is_empty() => format!("{source}@{skill_name}"),
                    _ => source.clone(),
                };
                Some(MarketplaceSkill {
                    package,
                    owner,
                    repo,
                    skill: name,
                    skills_sh_url: format!("https://skills.sh/{source}"),
                    description: None,
                    author_avatar: None,
                    stars: None,
                })
            })
            .collect())
    }

    fn load_bundled_featured_marketplace_items(&self) -> Result<Vec<MarketplaceSkill>> {
        let payload: serde_json::Value =
            serde_json::from_str(BUNDLED_FEATURED_SKILLS_JSON).map_err(CcrError::JsonError)?;
        Ok(payload
            .get("skills")
            .and_then(|value| value.as_array())
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|item| {
                let source_url = item.get("source_url")?.as_str()?;
                let trimmed = source_url
                    .trim()
                    .trim_start_matches("https://github.com/")
                    .trim_end_matches('/');
                let repo_root = trimmed.split("/tree/").next().unwrap_or(trimmed);
                let mut segments = repo_root.split('/');
                let owner = segments.next()?.to_string();
                let repo = segments.next()?.to_string();
                let skill_name = item
                    .get("name")
                    .and_then(|value| value.as_str())?
                    .to_string();
                Some(MarketplaceSkill {
                    package: format!("{owner}/{repo}@{skill_name}"),
                    owner,
                    repo,
                    skill: Some(skill_name),
                    skills_sh_url: format!("https://skills.sh/{repo_root}"),
                    description: item
                        .get("summary")
                        .and_then(|value| value.as_str())
                        .map(ToString::to_string),
                    author_avatar: None,
                    stars: item.get("stars").and_then(|value| value.as_u64()),
                })
            })
            .collect())
    }

    pub async fn marketplace_detail(&self, package_id: &str) -> Result<MarketplaceSkill> {
        let list = self.marketplace_list(None, 1, usize::MAX / 2).await?;
        list.items
            .into_iter()
            .find(|item| item.package == package_id)
            .ok_or_else(|| CcrError::ResourceNotFound(format!("Package '{package_id}' not found")))
    }

    fn parse_marketplace_item(item: &serde_json::Value) -> Option<MarketplaceSkill> {
        let package = item.get("package")?.as_str()?.to_string();
        let owner = item
            .get("owner")
            .and_then(|value| value.as_str())
            .map(ToString::to_string)
            .or_else(|| package.split('/').next().map(ToString::to_string))?;
        let repo = item
            .get("repo")
            .and_then(|value| value.as_str())
            .map(ToString::to_string)
            .or_else(|| {
                package
                    .split('/')
                    .nth(1)
                    .map(|value| value.split('@').next().unwrap_or(value).to_string())
            })?;
        let skill = item
            .get("skill")
            .and_then(|value| value.as_str())
            .map(ToString::to_string)
            .or_else(|| package.split('@').nth(1).map(ToString::to_string));
        let skills_sh_url = item
            .get("skills_sh_url")
            .or_else(|| item.get("skillsShUrl"))
            .and_then(|value| value.as_str())
            .map(ToString::to_string)
            .unwrap_or_else(|| format!("https://skills.sh/{}", package));

        Some(MarketplaceSkill {
            package,
            owner,
            repo,
            skill,
            skills_sh_url,
            description: item
                .get("description")
                .and_then(|value| value.as_str())
                .map(ToString::to_string),
            author_avatar: item
                .get("author_avatar")
                .or_else(|| item.get("authorAvatar"))
                .and_then(|value| value.as_str())
                .map(ToString::to_string),
            stars: item.get("stars").and_then(|value| value.as_u64()),
        })
    }

    pub fn npx_status(&self) -> NpxStatus {
        let version_output = if cfg!(windows) {
            silent_command("cmd")
                .args(["/C", "npx", "--version"])
                .output()
        } else {
            silent_command("sh").args(["-c", "npx --version"]).output()
        };

        let Ok(version_output) = version_output else {
            return NpxStatus {
                available: false,
                version: None,
                path: None,
            };
        };

        if !version_output.status.success() {
            return NpxStatus {
                available: false,
                version: None,
                path: None,
            };
        }

        let path_output = if cfg!(windows) {
            silent_command("cmd")
                .args(["/C", "where", "npx"])
                .output()
                .ok()
        } else {
            silent_command("sh").args(["-c", "which npx"]).output().ok()
        };

        NpxStatus {
            available: true,
            version: Some(
                String::from_utf8_lossy(&version_output.stdout)
                    .trim()
                    .to_string(),
            ),
            path: path_output.and_then(|output| {
                if output.status.success() {
                    Some(
                        String::from_utf8_lossy(&output.stdout)
                            .lines()
                            .next()?
                            .trim()
                            .to_string(),
                    )
                } else {
                    None
                }
            }),
        }
    }

    pub fn npx_capabilities(&self) -> Result<SkillsNpxCapabilities> {
        let status = self.npx_status();
        let supported_platforms = self
            .load_platforms()?
            .into_iter()
            .map(|platform| NpxPlatformSupport {
                platform_id: platform.id.clone(),
                platform_name: platform.display_name.clone(),
                supported: platform.npx_agent_key.is_some(),
                agent_key: platform.npx_agent_key.clone(),
                reason: if platform.npx_agent_key.is_some() {
                    None
                } else {
                    Some("CCR fallback required".into())
                },
            })
            .collect::<Vec<_>>();

        Ok(SkillsNpxCapabilities {
            status,
            supported_platforms,
            supported_flags: vec![
                "--agent".into(),
                "--skill".into(),
                "--global".into(),
                "--copy".into(),
                "--all".into(),
                "--list".into(),
                "--yes".into(),
                "--full-depth".into(),
            ],
            package_manager: "npx".into(),
        })
    }

    pub fn watch_paths(&self) -> Result<Vec<PathBuf>> {
        let mut paths = self
            .load_platforms()?
            .into_iter()
            .map(|platform| self.platform_dir(&platform))
            .collect::<Vec<_>>();

        for source in self.load_sources()? {
            paths.push(Self::source_base_path(&source));
        }

        paths.sort();
        paths.dedup();
        Ok(paths)
    }
}

#[derive(Debug, Clone)]
struct ParsedGithubRef {
    owner: String,
    repo: String,
    repo_url: String,
    skill_name: String,
    skill_path: String,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use ccr_core::core::error::CcrError;
    use std::future::Future;
    use std::task::{Context, Poll, Waker};
    use tempfile::tempdir;

    fn run_ready<F: Future>(future: F) -> F::Output {
        let mut context = Context::from_waker(Waker::noop());
        let mut future = std::pin::pin!(future);
        match future.as_mut().poll(&mut context) {
            Poll::Ready(output) => output,
            Poll::Pending => panic!("offline skills operation unexpectedly yielded"),
        }
    }

    fn build_test_service(root: &Path) -> SkillsService {
        SkillsService {
            home_dir: root.to_path_buf(),
            ccr_skills_dir: root.join(".ccr").join("skills"),
            sources_path: root.join(".ccr").join("skills").join(SOURCES_FILENAME),
            platforms_path: root.join(".ccr").join("skills").join(PLATFORMS_FILENAME),
            summary_cache_path: root.join(".cache").join(SUMMARY_CACHE_FILENAME),
        }
    }

    fn create_test_skill(root: &Path, name: &str) -> (SkillsService, String, String) {
        let service = build_test_service(root);
        let codex_platform = SkillsService::default_platforms()
            .into_iter()
            .find(|platform| platform.id == "codex")
            .unwrap();
        let skill_dir = root.join(codex_platform.relative_path).join(name);
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            format!("---\nname: {name}\ndescription: demo skill\n---\n\n# {name}\n"),
        )
        .unwrap();

        let inventory = service.inventory(None).unwrap();
        let skill = inventory
            .skills
            .into_iter()
            .find(|skill| skill.name == name)
            .unwrap();
        let installation = skill.installations.first().cloned().unwrap();
        (service, skill.id, installation.id)
    }

    #[test]
    fn build_group_key_prefers_explicit_install_group_id() {
        let key = SkillsService::build_group_key(
            Some("sg_existing"),
            &SkillOrigin::Marketplace,
            Some("owner/repo"),
            "demo",
            "hash123",
        );
        assert_eq!(key, "sg_existing");
    }

    #[test]
    fn parse_github_reference_supports_repo_skill_suffix() {
        let parsed = SkillsService::parse_github_reference("owner/repo@demo-skill", None).unwrap();
        assert_eq!(parsed.owner, "owner");
        assert_eq!(parsed.repo, "repo");
        assert_eq!(parsed.skill_name, "demo-skill");
        assert_eq!(parsed.skill_path, "demo-skill");
    }

    #[test]
    fn resolve_source_root_prefers_manifest_skills_dir() {
        let temp = tempdir().unwrap();
        let repo_root = temp.path().join("repo");
        fs::create_dir_all(repo_root.join("catalog")).unwrap();
        fs::write(
            repo_root.join("skills.toml"),
            "name = \"demo\"\nskills_dir = \"catalog\"\n",
        )
        .unwrap();

        let resolved = SkillsService::resolve_source_root(&repo_root);
        assert_eq!(resolved, repo_root.join("catalog"));
    }

    #[test]
    fn load_platforms_keeps_custom_iflow_entries() {
        let temp = tempdir().unwrap();
        let service = build_test_service(temp.path());
        fs::create_dir_all(service.ccr_skills_dir.clone()).unwrap();
        fs::write(
            &service.platforms_path,
            r#"
[[platforms]]
id = "codex"
display_name = "Codex"
relative_path = ".agents/skills"

[[platforms]]
id = "iflow"
display_name = "iFlow"
relative_path = ".iflow/skills"
"#,
        )
        .unwrap();

        let platforms = service.load_platforms().unwrap();
        assert_eq!(platforms.len(), 2);
        assert_eq!(platforms[0].id, "codex");
        assert_eq!(platforms[1].id, "iflow");

        let persisted = fs::read_to_string(&service.platforms_path).unwrap();
        assert!(persisted.contains("iflow"));
    }

    #[test]
    fn onboarding_candidates_work_after_inventory_scan_for_legacy_skill() {
        let temp = tempdir().unwrap();
        let service = build_test_service(temp.path());
        let codex_platform = SkillsService::default_platforms()
            .into_iter()
            .find(|platform| platform.id == "codex")
            .unwrap();
        let skill_dir = temp
            .path()
            .join(codex_platform.relative_path)
            .join("legacy-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: Legacy Skill\ndescription: demo skill\n---\n\n# Legacy Skill\n",
        )
        .unwrap();

        let inventory = service.inventory(None).unwrap();
        assert_eq!(inventory.skills.len(), 1);

        let candidates = service.onboarding_candidates().unwrap();
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].reason, "missing_source");
        assert_eq!(candidates[0].name, "Legacy Skill");
    }

    #[test]
    fn file_get_rejects_parent_traversal_outside_installation() {
        let temp = tempdir().unwrap();
        let (service, skill_id, installation_id) = create_test_skill(temp.path(), "demo-skill");
        fs::write(
            temp.path()
                .join(".codex")
                .join("skills")
                .join("outside.txt"),
            "secret",
        )
        .unwrap();

        let error = service
            .file_get(&skill_id, Some(&installation_id), "../outside.txt")
            .unwrap_err();

        assert!(matches!(error, CcrError::ValidationError(_)));
    }

    #[test]
    fn file_get_rejects_absolute_path_requests() {
        let temp = tempdir().unwrap();
        let (service, skill_id, installation_id) = create_test_skill(temp.path(), "demo-skill");
        let outside = temp.path().join("outside.txt");
        fs::write(&outside, "secret").unwrap();

        let error = service
            .file_get(
                &skill_id,
                Some(&installation_id),
                outside.to_string_lossy().as_ref(),
            )
            .unwrap_err();

        assert!(matches!(error, CcrError::ValidationError(_)));
    }

    #[test]
    fn file_get_reads_files_within_installation_root() {
        let temp = tempdir().unwrap();
        let (service, skill_id, installation_id) = create_test_skill(temp.path(), "demo-skill");
        let nested_dir = temp
            .path()
            .join(".codex")
            .join("skills")
            .join("demo-skill")
            .join("docs");
        fs::create_dir_all(&nested_dir).unwrap();
        fs::write(nested_dir.join("guide.md"), "hello").unwrap();

        let content = service
            .file_get(&skill_id, Some(&installation_id), "docs/guide.md")
            .unwrap();

        assert_eq!(content.path, "docs/guide.md");
        assert_eq!(content.content, "hello");
    }

    #[test]
    fn local_source_install_sync_and_remove_lifecycle() {
        let temp = tempdir().unwrap();
        let service = build_test_service(temp.path());
        let source_root = temp.path().join("source-repo");
        let skill_dir = source_root.join("catalog").join("demo-skill");
        fs::create_dir_all(skill_dir.join("docs")).unwrap();
        fs::write(
            source_root.join("skills.toml"),
            "name = \"Local Catalog\"\ndescription = \"offline fixture\"\nskills_dir = \"catalog\"\n",
        )
        .unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: Demo Skill\ndescription: lifecycle fixture\ncategory: testing\ntags: [offline, lifecycle]\n---\n\n# Demo Skill\n",
        )
        .unwrap();
        fs::write(skill_dir.join("docs").join("guide.md"), "source guide").unwrap();

        let source = service
            .source_add_local(source_root.to_string_lossy().as_ref())
            .unwrap();
        assert_eq!(source.name, "Local Catalog");
        assert_eq!(source.skill_count, 1);
        assert_eq!(source.skills[0].id, "demo-skill");
        assert_eq!(service.sources_list().unwrap().len(), 1);
        assert_eq!(service.source_sync(&source.id).unwrap().skill_count, 1);

        let request = SkillsInstallRequest {
            source_kind: "source".into(),
            source_ref: source.id.clone(),
            source_skill_id: Some("demo-skill".into()),
            selected_skills: Vec::new(),
            target_platforms: vec!["codex".into(), "claude-code".into()],
            force: false,
            scope: Some("global".into()),
            copy_mode: Some(true),
            all_mode: Some(false),
        };
        let review = run_ready(service.prepare_install(request.clone())).unwrap();
        assert_eq!(review.source.resolved_name, "Demo Skill");
        assert_eq!(review.targets.len(), 2);
        assert_eq!(review.command_previews.len(), 1);

        let installed = run_ready(service.install(request)).unwrap();
        assert!(installed.results.iter().all(|result| result.ok));
        assert_eq!(installed.results.len(), 2);

        let inventory = service
            .inventory(Some(SkillsInventoryQuery {
                platform: Some("codex".into()),
                origin: Some(SkillOrigin::Local),
                q: Some("lifecycle".into()),
                source_id: Some(source.id.clone()),
            }))
            .unwrap();
        assert_eq!(inventory.total, 1);
        let skill = inventory.skills.into_iter().next().unwrap();
        assert_eq!(skill.name, "Demo Skill");
        assert_eq!(skill.installations.len(), 2);

        let detail = service.detail(&skill.id).unwrap();
        let codex = detail
            .installations
            .iter()
            .find(|installation| installation.platform_id == "codex")
            .unwrap();
        let content = service.content_get(&skill.id, Some(&codex.id)).unwrap();
        assert!(content.raw.contains("lifecycle fixture"));
        let files = service.files_list(&skill.id, Some(&codex.id)).unwrap();
        assert!(files.iter().any(|entry| entry.path == "SKILL.md"));
        assert_eq!(
            service
                .file_get(&skill.id, Some(&codex.id), "SKILL.md")
                .unwrap()
                .content,
            content.raw
        );

        let updated_raw = "---\nname: Demo Skill\ndescription: updated fixture\n---\n\n# Updated\n";
        let updated = service
            .content_save(&skill.id, &codex.id, updated_raw)
            .unwrap();
        assert_eq!(updated.raw, updated_raw);

        let synced = run_ready(service.sync(SkillsSyncRequest {
            skill_id: skill.id.clone(),
            installation_id: Some(codex.id.clone()),
            target_platforms: vec!["codex".into(), "gemini".into()],
            force: true,
        }))
        .unwrap();
        assert!(synced.results.iter().all(|result| result.ok));
        let detail = service.detail(&skill.id).unwrap();
        assert_eq!(detail.installations.len(), 3);

        let gemini = detail
            .installations
            .iter()
            .find(|installation| installation.platform_id == "gemini")
            .unwrap();
        assert!(
            service
                .remove_installation(&skill.id, &gemini.id)
                .unwrap()
                .results[0]
                .ok
        );
        assert_eq!(service.detail(&skill.id).unwrap().installations.len(), 2);
        assert!(
            service
                .remove_skill(&skill.id)
                .unwrap()
                .results
                .iter()
                .all(|result| result.ok)
        );
        assert!(service.inventory(None).unwrap().skills.is_empty());

        service.source_remove(&source.id).unwrap();
        assert!(service.sources_list().unwrap().is_empty());
    }

    #[test]
    fn local_skill_install_filters_inventory_and_rejects_duplicates() {
        let temp = tempdir().unwrap();
        let service = build_test_service(temp.path());
        let source_dir = temp.path().join("standalone-skill");
        fs::create_dir_all(&source_dir).unwrap();
        fs::write(
            source_dir.join("SKILL.md"),
            "---\nname: Standalone Skill\ndescription: offline install fixture\ncategory: automation\nauthor: CCR Team\ntags: [local, testing]\n---\n\n# Standalone Skill\n",
        )
        .unwrap();

        let request = SkillsInstallRequest {
            source_kind: "local".into(),
            source_ref: source_dir.to_string_lossy().to_string(),
            source_skill_id: None,
            selected_skills: Vec::new(),
            target_platforms: vec!["codex".into()],
            force: false,
            scope: None,
            copy_mode: None,
            all_mode: None,
        };
        let review = run_ready(service.prepare_install(request.clone())).unwrap();
        assert_eq!(review.source.resolved_name, "Standalone Skill");
        assert_eq!(review.source.origin, SkillOrigin::Local);
        assert_eq!(review.targets.len(), 1);

        let installed = run_ready(service.install(request.clone())).unwrap();
        assert!(installed.results[0].ok);
        let duplicate = run_ready(service.install(request)).unwrap();
        assert!(!duplicate.results[0].ok);
        assert!(
            duplicate.results[0]
                .message
                .as_deref()
                .is_some_and(|message| message.contains("技能目录已存在"))
        );

        for query in ["standalone", "offline", "automation", "CCR Team", "testing"] {
            let inventory = service
                .inventory(Some(SkillsInventoryQuery {
                    q: Some(query.into()),
                    ..Default::default()
                }))
                .unwrap();
            assert_eq!(inventory.total, 1, "query {query} should match");
        }
        assert_eq!(
            service
                .inventory(Some(SkillsInventoryQuery {
                    platform: Some("codex".into()),
                    origin: Some(SkillOrigin::Local),
                    q: Some("   ".into()),
                    source_id: None,
                }))
                .unwrap()
                .total,
            1
        );
        assert_eq!(
            service
                .inventory(Some(SkillsInventoryQuery {
                    platform: Some("missing".into()),
                    ..Default::default()
                }))
                .unwrap()
                .total,
            0
        );
        assert_eq!(
            service
                .inventory(Some(SkillsInventoryQuery {
                    origin: Some(SkillOrigin::Github),
                    ..Default::default()
                }))
                .unwrap()
                .total,
            0
        );
    }

    #[test]
    fn local_git_source_clone_pull_and_remove_lifecycle() {
        let temp = tempdir().unwrap();
        let service = build_test_service(temp.path());
        let origin = temp.path().join("origin");
        let skill_dir = origin.join("skills").join("git-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: Git Skill\ndescription: first revision\n---\n\n# Git Skill\n",
        )
        .unwrap();
        SkillsService::run_git(&["init"], Some(&origin)).unwrap();
        SkillsService::run_git(
            &["config", "user.email", "tests@example.invalid"],
            Some(&origin),
        )
        .unwrap();
        SkillsService::run_git(&["config", "user.name", "CCR Tests"], Some(&origin)).unwrap();
        SkillsService::run_git(&["add", "."], Some(&origin)).unwrap();
        SkillsService::run_git(&["commit", "-m", "initial"], Some(&origin)).unwrap();

        let source = service
            .source_add_git(origin.to_string_lossy().as_ref())
            .unwrap();
        assert_eq!(source.source_type, SkillSourceType::Git);
        assert_eq!(source.skill_count, 1);
        assert_eq!(source.skills[0].name, "Git Skill");
        let duplicate = service
            .source_add_git(origin.to_string_lossy().as_ref())
            .unwrap();
        assert_eq!(duplicate.id, source.id);

        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: Git Skill\ndescription: second revision\n---\n\n# Git Skill\n",
        )
        .unwrap();
        SkillsService::run_git(&["add", "."], Some(&origin)).unwrap();
        SkillsService::run_git(&["commit", "-m", "update"], Some(&origin)).unwrap();
        let synced = service.source_sync(&source.id).unwrap();
        assert_eq!(synced.health, SkillSourceHealth::Ok);
        let checkout = service.source_checkout_dir(&source.id);
        assert!(
            fs::read_to_string(checkout.join("skills/git-skill/SKILL.md"))
                .unwrap()
                .contains("second revision")
        );

        service.source_remove(&source.id).unwrap();
        assert!(!checkout.exists());
        assert!(service.sources_list().unwrap().is_empty());
    }

    #[test]
    fn install_validation_reports_offline_request_errors() {
        let temp = tempdir().unwrap();
        let service = build_test_service(temp.path());
        let skill_dir = temp.path().join("demo");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "---\nname: Demo\n---\n").unwrap();

        let base = SkillsInstallRequest {
            source_kind: "local".into(),
            source_ref: skill_dir.to_string_lossy().to_string(),
            source_skill_id: None,
            selected_skills: Vec::new(),
            target_platforms: vec!["codex".into()],
            force: false,
            scope: None,
            copy_mode: None,
            all_mode: None,
        };

        let mut request = base.clone();
        request.source_kind = "unsupported".into();
        assert!(matches!(
            run_ready(service.prepare_install(request)).unwrap_err(),
            CcrError::ConfigError(_)
        ));

        let mut request = base.clone();
        request.target_platforms.clear();
        assert!(matches!(
            run_ready(service.prepare_install(request)).unwrap_err(),
            CcrError::ConfigError(_)
        ));

        let mut request = base.clone();
        request.target_platforms = vec!["missing".into()];
        assert!(matches!(
            run_ready(service.prepare_install(request)).unwrap_err(),
            CcrError::ResourceNotFound(_)
        ));

        let mut request = base;
        request.source_ref = temp.path().join("missing").to_string_lossy().to_string();
        assert!(matches!(
            run_ready(service.prepare_install(request)).unwrap_err(),
            CcrError::ResourceNotFound(_)
        ));
        assert!(matches!(
            service.source_add_local(temp.path().join("absent").to_string_lossy().as_ref()),
            Err(CcrError::ResourceNotFound(_))
        ));
        assert!(matches!(
            service.source_sync("missing"),
            Err(CcrError::ResourceNotFound(_))
        ));
        assert!(matches!(
            service.source_remove("missing"),
            Err(CcrError::ResourceNotFound(_))
        ));
    }

    #[test]
    fn npx_command_builder_covers_selected_and_all_modes() {
        let temp = tempdir().unwrap();
        let service = build_test_service(temp.path());
        let payload = InstallPayload {
            requested_name: "Demo Skill".into(),
            dir_name: "demo-skill".into(),
            raw: "---\nname: Demo Skill\n---\n".into(),
            origin: SkillOrigin::Npx,
            source_url: Some("https://github.com/owner/repo".into()),
            source_repo_id: Some("owner/repo".into()),
            install_group_id: None,
        };
        let targets = service
            .resolve_install_targets(&["codex".into(), "claude-code".into()], "demo-skill")
            .unwrap();
        let mut request = SkillsInstallRequest {
            source_kind: "npx".into(),
            source_ref: "owner/repo".into(),
            source_skill_id: None,
            selected_skills: vec!["alpha".into(), "beta".into()],
            target_platforms: vec!["codex".into(), "claude-code".into()],
            force: false,
            scope: Some("project".into()),
            copy_mode: Some(false),
            all_mode: Some(false),
        };

        let selected = service
            .build_npx_install_command(&request, &payload, &targets, true)
            .unwrap();
        assert!(selected.starts_with("npx skills add owner/repo"));
        assert!(selected.contains("--agent"));
        assert!(selected.contains("--skill alpha beta"));
        assert!(selected.ends_with("--yes"));
        assert!(!selected.contains("--global"));
        assert!(!selected.contains("--copy"));

        request.scope = Some("global".into());
        request.copy_mode = Some(true);
        request.all_mode = Some(true);
        let all = service
            .build_npx_install_command(&request, &payload, &targets, false)
            .unwrap();
        assert!(all.contains("--global"));
        assert!(all.contains("--copy"));
        assert!(all.ends_with("--all"));
        assert!(!all.contains("--agent"));
        assert!(!all.contains("--skill"));
    }

    #[test]
    fn bundled_marketplace_fallback_and_parser_are_stable() {
        let temp = tempdir().unwrap();
        let service = build_test_service(temp.path());
        let items = service.load_bundled_featured_marketplace_items().unwrap();
        assert!(!items.is_empty());
        assert!(items.iter().all(|item| !item.owner.is_empty()));

        let parsed = SkillsService::parse_marketplace_item(&serde_json::json!({
            "package": "owner/repo@demo",
            "stars": 42,
            "description": "fixture"
        }))
        .unwrap();
        assert_eq!(parsed.owner, "owner");
        assert_eq!(parsed.repo, "repo");
        assert_eq!(parsed.skill.as_deref(), Some("demo"));
        assert_eq!(parsed.stars, Some(42));
    }
}
