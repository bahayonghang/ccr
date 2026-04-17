//! Agent 注册表：抽象各 AI CLI 的 skills 目录约定。
//!
//! 当前仅实现 ccr-ui 已支持的 5 家（Claude Code / Codex / Gemini / Droid / OpenCode）。
//! 通过 `trait AgentLocator` 预留 42 agent 扩展位 — skill-hub 的完整注册表
//! 可在未来按需迁入，无需重构调用方。
//!
//! ## 扩展新 agent
//! 1. 新建 `agents/my_agent.rs` 并实现 `AgentLocator`
//! 2. 在 `AgentId` 枚举添加变体
//! 3. 在 `all_agents()` 的静态数组追加 `&MyAgentLocator`
//! 4. 补齐 `AgentId::from_str` 与 `as_kebab` 的分支

mod claude_code;
mod codex;
mod droid;
mod gemini;
mod opencode;

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use claude_code::ClaudeCodeLocator;
use codex::CodexLocator;
use droid::DroidLocator;
use gemini::GeminiLocator;
use opencode::OpenCodeLocator;

/// 支持的 AI CLI agent。`Unknown` 兜底 frontmatter `agent:` 无法解析的情况。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentId {
    ClaudeCode,
    Codex,
    Gemini,
    Droid,
    OpenCode,
    Unknown,
}

impl AgentId {
    /// 从 frontmatter `agent:` 字段宽松解析：大小写 / 连字符 / 常见别名都可接受。
    /// 未知输入归为 [`AgentId::Unknown`]。**故意不实现 `FromStr`**（避免 Result 语义）。
    pub fn parse(raw: &str) -> Self {
        let s = raw.trim().to_lowercase();
        match s.as_str() {
            "claude-code" | "claude" | "claudecode" => Self::ClaudeCode,
            "codex" => Self::Codex,
            "gemini" | "gemini-cli" => Self::Gemini,
            "droid" => Self::Droid,
            "opencode" | "open-code" => Self::OpenCode,
            _ => Self::Unknown,
        }
    }

    /// 序列化为前端协议使用的 kebab-case id。
    pub fn as_kebab(&self) -> &'static str {
        match self {
            Self::ClaudeCode => "claude-code",
            Self::Codex => "codex",
            Self::Gemini => "gemini",
            Self::Droid => "droid",
            Self::OpenCode => "opencode",
            Self::Unknown => "unknown",
        }
    }
}

/// 统一定位 agent 的 skills 目录。
///
/// 所有实现都必须是**纯函数**（无 I/O、无环境变量副作用），
/// 方便在扫描器里并发调用、在测试里注入假 `home` / `project_root`。
pub trait AgentLocator: Send + Sync {
    /// 唯一 id。
    fn id(&self) -> AgentId;
    /// UI 展示名称 (English)。
    fn display_name(&self) -> &'static str;
    /// 单 emoji 图标。
    fn icon(&self) -> &'static str;
    /// 全局 skills 目录 (用户级)。一个 agent 可能有多条 (如 Windsurf 同时有
    /// `.codeium` 和 `.windsurf`)。
    fn global_paths(&self, home: &Path) -> Vec<PathBuf>;
    /// 项目级 skills 目录。部分 agent (如 Codex / Gemini) 仅有全局，返回空 Vec。
    fn project_paths(&self, project_root: &Path) -> Vec<PathBuf>;
}

/// 所有内置 agent 的静态注册表。**顺序稳定**，前端按此顺序展示。
pub fn all_agents() -> &'static [&'static dyn AgentLocator] {
    static AGENTS: &[&dyn AgentLocator] = &[
        &ClaudeCodeLocator,
        &CodexLocator,
        &GeminiLocator,
        &DroidLocator,
        &OpenCodeLocator,
    ];
    AGENTS
}

/// 按 `AgentId` 查找 locator。`AgentId::Unknown` 始终返回 `None`。
pub fn locator_for(id: AgentId) -> Option<&'static dyn AgentLocator> {
    all_agents().iter().copied().find(|a| a.id() == id)
}
