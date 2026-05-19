//! Claude Observer 模块
//!
//! 提供 Claude Code 本地 JSONL 转写文件的 tool-call 级解析与查询能力。
//! 与 llmusage_adapter 互补：后者承担 token/cost 维度的事实，本模块仅承担
//! 「工具调用事件」的事实链。两者通过 (session_id, ts) 关联但不强绑。
//!
//! 模块边界：
//! - `pricing` —— Anthropic 价目表（来自 vibe-observer），仅做版本暴露与
//!   未来重定价兜底，不参与 llmusage 的 value 计算。
//! - `jsonl`   —— 单行 JSONL 解析，输出 `ParsedToolEvent`。
//! - `scanner` —— `~/.claude/projects/**.jsonl` 增量扫描，写入
//!   `ccr_db::database::repositories::claude_tool_calls_repo`。
//! - `subscription` —— 订阅模式 / 月费的 user_settings 包装。
//!
//! 调用方一律走 `crate::claude_observer::<sub>::<item>` 全路径，本模块不再
//! 维护 re-export 表面，避免 P2 阶段那种 `#[allow(unused_imports)]` 散落噪音。

pub mod jsonl;
pub mod pricing;
pub mod scanner;
pub mod subscription;
