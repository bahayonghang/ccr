//! 命令执行模块 — CCR CLI 命令白名单执行。

use std::collections::HashMap;
use std::io;
use std::process::Stdio;
use std::sync::LazyLock;
use std::time::Instant;

use crate::process::tokio_command;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::{Mutex, mpsc};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

const EVENT_COMMAND_JOB_PROGRESS: &str = "commands:job-progress";
const EVENT_COMMAND_JOB_FINISHED: &str = "commands:job-finished";
const EVENT_COMMAND_JOB_CANCELLED: &str = "commands:job-cancelled";

/// 允许执行的 CCR 子命令白名单
const ALLOWED_COMMANDS: &[&str] = &[
    "list",
    "switch",
    "add",
    "delete",
    "rename",
    "duplicate",
    "show",
    "validate",
    "export",
    "import",
    "history",
    "version",
    "help",
    "backup",
    "restore",
    "diff",
    "status",
];

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CommandArgSchema {
    pub name: &'static str,
    pub label: &'static str,
    #[serde(rename = "type")]
    pub arg_type: &'static str,
    pub required: bool,
    pub placeholder: Option<&'static str>,
    pub source: Option<&'static str>,
    pub description: &'static str,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CommandFlagSchema {
    pub name: &'static str,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<&'static str>,
    pub label: &'static str,
    pub description: &'static str,
    #[serde(rename = "type")]
    pub flag_type: &'static str,
    pub takes_value: bool,
    pub default_value: Option<&'static str>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CommandInfo {
    pub name: &'static str,
    pub path: Vec<&'static str>,
    pub title: &'static str,
    pub description: &'static str,
    pub usage: &'static str,
    pub examples: Vec<&'static str>,
    pub category: &'static str,
    pub risk: &'static str,
    pub executable: bool,
    pub requires_confirmation: bool,
    pub args: Vec<CommandArgSchema>,
    pub flags: Vec<CommandFlagSchema>,
    pub aliases: Vec<&'static str>,
    pub related_route: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CommandExecutionRequest {
    command: String,
    args: Vec<String>,
    confirmation_token: Option<String>,
    background_job: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CommandPolicy {
    info: CommandInfo,
    allow_background_job: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParsedFlagValue {
    Inline,
    NextArg,
}

impl CommandExecutionRequest {
    fn foreground(
        command: String,
        args: Option<Vec<String>>,
        confirmation_token: Option<String>,
    ) -> Self {
        Self {
            command,
            args: args.unwrap_or_default(),
            confirmation_token,
            background_job: false,
        }
    }

    fn background(
        command: String,
        args: Option<Vec<String>>,
        confirmation_token: Option<String>,
    ) -> Self {
        Self {
            command,
            args: args.unwrap_or_default(),
            confirmation_token,
            background_job: true,
        }
    }
}

impl CommandPolicy {
    fn from_command(command: &str) -> Result<Self, String> {
        let catalog = command_catalog();
        let info = catalog
            .into_iter()
            .find(|entry| entry.name == command)
            .ok_or_else(|| command_not_allowed_error(command))?;

        if !info.executable || !ALLOWED_COMMANDS.contains(&info.name) {
            return Err(command_not_allowed_error(command));
        }

        Ok(Self {
            allow_background_job: true,
            info,
        })
    }

    fn validate(&self, request: &CommandExecutionRequest) -> Result<(), String> {
        if request.background_job && !self.allow_background_job {
            return Err(format!(
                "命令 '{}' 不允许作为后台任务执行。",
                request.command
            ));
        }

        self.validate_confirmation(request)?;
        self.validate_args(&request.args)
    }

    fn validate_confirmation(&self, request: &CommandExecutionRequest) -> Result<(), String> {
        if !self.info.requires_confirmation {
            return Ok(());
        }

        let expected = confirmation_token_for(&self.info);
        match request.confirmation_token.as_deref() {
            Some(token) if token == expected => Ok(()),
            _ => Err(format!(
                "命令 '{}' 需要桌面确认后才能执行。",
                self.info.name
            )),
        }
    }

    fn validate_args(&self, args: &[String]) -> Result<(), String> {
        let required_positional_count = self.info.args.iter().filter(|arg| arg.required).count();
        let max_positional_count = self.info.args.len();
        let mut positional_count = 0usize;
        let mut index = 0usize;

        while let Some(raw_arg) = args.get(index) {
            if raw_arg.starts_with('-') {
                let (flag_name, value_kind) = split_flag_arg(raw_arg)?;
                let flag = self.find_flag(flag_name).ok_or_else(|| {
                    format!(
                        "命令 '{}' 不允许参数 '{}'。",
                        self.info.name, flag_name
                    )
                })?;

                if flag.takes_value {
                    match value_kind {
                        Some(ParsedFlagValue::Inline) => {}
                        Some(ParsedFlagValue::NextArg) => {
                            return Err(format!(
                                "命令 '{}' 的参数 '{}' 缺少值。",
                                self.info.name, flag_name
                            ));
                        }
                        None => {
                            let value = args.get(index + 1).ok_or_else(|| {
                                format!(
                                    "命令 '{}' 的参数 '{}' 缺少值。",
                                    self.info.name, flag_name
                                )
                            })?;
                            if value.starts_with('-') {
                                return Err(format!(
                                    "命令 '{}' 的参数 '{}' 缺少值。",
                                    self.info.name, flag_name
                                ));
                            }
                            index += 1;
                        }
                    }
                } else if value_kind.is_some() {
                    return Err(format!(
                        "命令 '{}' 的布尔参数 '{}' 不接受值。",
                        self.info.name, flag_name
                    ));
                }
            } else {
                positional_count += 1;
                if positional_count > max_positional_count {
                    return Err(format!(
                        "命令 '{}' 不接受额外位置参数 '{}'。",
                        self.info.name, raw_arg
                    ));
                }
            }

            index += 1;
        }

        if positional_count < required_positional_count {
            return Err(format!(
                "命令 '{}' 缺少必需位置参数。需要 {} 个，收到 {} 个。",
                self.info.name, required_positional_count, positional_count
            ));
        }

        Ok(())
    }

    fn find_flag(&self, name: &str) -> Option<&CommandFlagSchema> {
        self.info
            .flags
            .iter()
            .find(|flag| flag.name == name || flag.aliases.contains(&name))
    }
}

fn text_arg(
    name: &'static str,
    label: &'static str,
    required: bool,
    placeholder: Option<&'static str>,
    source: Option<&'static str>,
    description: &'static str,
) -> CommandArgSchema {
    CommandArgSchema {
        name,
        label,
        arg_type: "text",
        required,
        placeholder,
        source,
        description,
    }
}

fn path_arg(
    name: &'static str,
    label: &'static str,
    required: bool,
    placeholder: Option<&'static str>,
    description: &'static str,
) -> CommandArgSchema {
    CommandArgSchema {
        name,
        label,
        arg_type: "path",
        required,
        placeholder,
        source: None,
        description,
    }
}

fn bool_flag(
    name: &'static str,
    label: &'static str,
    description: &'static str,
) -> CommandFlagSchema {
    bool_flag_with_aliases(name, vec![], label, description)
}

fn bool_flag_with_aliases(
    name: &'static str,
    aliases: Vec<&'static str>,
    label: &'static str,
    description: &'static str,
) -> CommandFlagSchema {
    CommandFlagSchema {
        name,
        aliases,
        label,
        description,
        flag_type: "boolean",
        takes_value: false,
        default_value: None,
    }
}

fn value_flag_with_aliases(
    name: &'static str,
    aliases: Vec<&'static str>,
    label: &'static str,
    description: &'static str,
    value_type: &'static str,
    default_value: Option<&'static str>,
) -> CommandFlagSchema {
    CommandFlagSchema {
        name,
        aliases,
        label,
        description,
        flag_type: value_type,
        takes_value: true,
        default_value,
    }
}

fn command_catalog() -> Vec<CommandInfo> {
    vec![
        CommandInfo {
            name: "status",
            path: vec!["status"],
            title: "Status overview",
            description: "Inspect the current Claude/Codex runtime status without changing configuration.",
            usage: "ccr status --json",
            examples: vec!["ccr status", "ccr status --json"],
            category: "read",
            risk: "safe",
            executable: true,
            requires_confirmation: false,
            args: vec![],
            flags: vec![
                bool_flag(
                    "--json",
                    "JSON output",
                    "Return a machine-readable status payload.",
                ),
                bool_flag(
                    "--verbose",
                    "Verbose diagnostics",
                    "Include legacy diagnostic details.",
                ),
            ],
            aliases: vec!["current", "show"],
            related_route: None,
        },
        CommandInfo {
            name: "list",
            path: vec!["list"],
            title: "List configurations",
            description: "List saved CCR configurations.",
            usage: "ccr list",
            examples: vec!["ccr list"],
            category: "read",
            risk: "safe",
            executable: true,
            requires_confirmation: false,
            args: vec![],
            flags: vec![],
            aliases: vec!["ls"],
            related_route: Some("/configs"),
        },
        CommandInfo {
            name: "validate",
            path: vec!["validate"],
            title: "Validate configuration",
            description: "Validate CCR configuration and managed settings files.",
            usage: "ccr validate",
            examples: vec!["ccr validate"],
            category: "diagnostic",
            risk: "safe",
            executable: true,
            requires_confirmation: false,
            args: vec![],
            flags: vec![],
            aliases: vec![],
            related_route: None,
        },
        CommandInfo {
            name: "history",
            path: vec!["history"],
            title: "Audit history",
            description: "Inspect recent CCR configuration operations.",
            usage: "ccr history -l 20",
            examples: vec!["ccr history", "ccr history -l 50 -t switch"],
            category: "diagnostic",
            risk: "safe",
            executable: true,
            requires_confirmation: false,
            args: vec![],
            flags: vec![
                value_flag_with_aliases(
                    "--limit",
                    vec!["-l"],
                    "Limit",
                    "Maximum number of history entries.",
                    "number",
                    Some("20"),
                ),
                value_flag_with_aliases(
                    "--type",
                    vec!["-t"],
                    "Operation type",
                    "Filter by operation type.",
                    "text",
                    None,
                ),
            ],
            aliases: vec![],
            related_route: None,
        },
        CommandInfo {
            name: "version",
            path: vec!["version"],
            title: "Version details",
            description: "Show the installed CCR version.",
            usage: "ccr version",
            examples: vec!["ccr version"],
            category: "read",
            risk: "safe",
            executable: true,
            requires_confirmation: false,
            args: vec![],
            flags: vec![],
            aliases: vec!["ver"],
            related_route: None,
        },
        CommandInfo {
            name: "help",
            path: vec!["help"],
            title: "Command help",
            description: "Show CCR help for a command path.",
            usage: "ccr help <command>",
            examples: vec!["ccr help", "ccr help switch"],
            category: "read",
            risk: "safe",
            executable: true,
            requires_confirmation: false,
            args: vec![text_arg(
                "path",
                "Command path",
                false,
                Some("switch"),
                None,
                "Optional command path to inspect.",
            )],
            flags: vec![],
            aliases: vec![],
            related_route: None,
        },
        CommandInfo {
            name: "show",
            path: vec!["show"],
            title: "Show current status",
            description: "Compatibility alias for the current status view.",
            usage: "ccr show",
            examples: vec!["ccr show", "ccr show --json"],
            category: "read",
            risk: "safe",
            executable: true,
            requires_confirmation: false,
            args: vec![],
            flags: vec![bool_flag(
                "--json",
                "JSON output",
                "Return a machine-readable status payload.",
            )],
            aliases: vec!["current", "status"],
            related_route: None,
        },
        CommandInfo {
            name: "diff",
            path: vec!["diff"],
            title: "Compare configurations",
            description: "Compare two saved configurations.",
            usage: "ccr diff <left> <right>",
            examples: vec!["ccr diff default work"],
            category: "diagnostic",
            risk: "safe",
            executable: true,
            requires_confirmation: false,
            args: vec![
                text_arg(
                    "left",
                    "Left config",
                    true,
                    Some("default"),
                    Some("configs"),
                    "First configuration name.",
                ),
                text_arg(
                    "right",
                    "Right config",
                    true,
                    Some("work"),
                    Some("configs"),
                    "Second configuration name.",
                ),
            ],
            flags: vec![],
            aliases: vec![],
            related_route: Some("/configs"),
        },
        CommandInfo {
            name: "export",
            path: vec!["export"],
            title: "Export configuration",
            description: "Export CCR configuration to a file.",
            usage: "ccr export --no-secrets",
            examples: vec!["ccr export", "ccr export -o ccr-config.toml --no-secrets"],
            category: "read",
            risk: "safe",
            executable: true,
            requires_confirmation: false,
            args: vec![],
            flags: vec![
                value_flag_with_aliases(
                    "--output",
                    vec!["-o"],
                    "Output file",
                    "Destination TOML file path.",
                    "path",
                    None,
                ),
                bool_flag(
                    "--no-secrets",
                    "Exclude secrets",
                    "Omit API keys and other sensitive values.",
                ),
            ],
            aliases: vec![],
            related_route: Some("/configs"),
        },
        CommandInfo {
            name: "switch",
            path: vec!["switch"],
            title: "Switch configuration",
            description: "Switch Claude/Codex settings to a saved CCR configuration.",
            usage: "ccr switch <config>",
            examples: vec!["ccr switch default"],
            category: "write",
            risk: "writes_config",
            executable: true,
            requires_confirmation: false,
            args: vec![text_arg(
                "config_name",
                "Configuration",
                true,
                Some("default"),
                Some("configs"),
                "Configuration name from the CCR config list.",
            )],
            flags: vec![],
            aliases: vec![],
            related_route: Some("/configs"),
        },
        CommandInfo {
            name: "rename",
            path: vec!["rename"],
            title: "Rename configuration",
            description: "Rename a saved CCR configuration.",
            usage: "ccr rename <old> <new>",
            examples: vec!["ccr rename default work"],
            category: "write",
            risk: "writes_config",
            executable: true,
            requires_confirmation: false,
            args: vec![
                text_arg(
                    "old_name",
                    "Current name",
                    true,
                    Some("default"),
                    Some("configs"),
                    "Existing configuration name.",
                ),
                text_arg(
                    "new_name",
                    "New name",
                    true,
                    Some("work"),
                    None,
                    "New configuration name.",
                ),
            ],
            flags: vec![],
            aliases: vec![],
            related_route: Some("/configs"),
        },
        CommandInfo {
            name: "duplicate",
            path: vec!["duplicate"],
            title: "Duplicate configuration",
            description: "Copy an existing CCR configuration to a new name.",
            usage: "ccr duplicate <source> <target>",
            examples: vec!["ccr duplicate default sandbox"],
            category: "write",
            risk: "writes_config",
            executable: true,
            requires_confirmation: false,
            args: vec![
                text_arg(
                    "source",
                    "Source",
                    true,
                    Some("default"),
                    Some("configs"),
                    "Configuration to copy.",
                ),
                text_arg(
                    "target",
                    "Target",
                    true,
                    Some("sandbox"),
                    None,
                    "New configuration name.",
                ),
            ],
            flags: vec![],
            aliases: vec![],
            related_route: Some("/configs"),
        },
        CommandInfo {
            name: "backup",
            path: vec!["backup"],
            title: "Create backup",
            description: "Create a backup of managed CCR configuration files.",
            usage: "ccr backup",
            examples: vec!["ccr backup"],
            category: "write",
            risk: "writes_config",
            executable: true,
            requires_confirmation: false,
            args: vec![],
            flags: vec![],
            aliases: vec![],
            related_route: None,
        },
        CommandInfo {
            name: "delete",
            path: vec!["delete"],
            title: "Delete configuration",
            description: "Delete a saved CCR configuration. Requires explicit confirmation in the UI.",
            usage: "ccr delete <config>",
            examples: vec!["ccr delete old-config --force"],
            category: "danger",
            risk: "destructive",
            executable: true,
            requires_confirmation: true,
            args: vec![text_arg(
                "config_name",
                "Configuration",
                true,
                Some("old-config"),
                Some("configs"),
                "Configuration name to delete.",
            )],
            flags: vec![bool_flag_with_aliases(
                "--force",
                vec!["-f"],
                "Skip CLI confirmation",
                "Run non-interactively after UI confirmation.",
            )],
            aliases: vec![],
            related_route: Some("/configs"),
        },
        CommandInfo {
            name: "import",
            path: vec!["import"],
            title: "Import configuration",
            description: "Import configuration from a TOML file. Replace mode can overwrite existing data.",
            usage: "ccr import <file> --merge --backup",
            examples: vec!["ccr import ccr-config.toml --merge --backup"],
            category: "danger",
            risk: "destructive",
            executable: true,
            requires_confirmation: true,
            args: vec![path_arg(
                "input",
                "Input file",
                true,
                Some("ccr-config.toml"),
                "TOML file to import.",
            )],
            flags: vec![
                bool_flag_with_aliases(
                    "--merge",
                    vec!["-m"],
                    "Merge",
                    "Merge imported configs into existing configs.",
                ),
                bool_flag_with_aliases(
                    "--backup",
                    vec!["-b"],
                    "Backup first",
                    "Create a backup before importing.",
                ),
                bool_flag_with_aliases(
                    "--force",
                    vec!["-f"],
                    "Skip CLI confirmation",
                    "Run non-interactively after UI confirmation.",
                ),
            ],
            aliases: vec![],
            related_route: Some("/configs"),
        },
        CommandInfo {
            name: "restore",
            path: vec!["restore"],
            title: "Restore backup",
            description: "Restore configuration from a backup. Requires explicit confirmation in the UI.",
            usage: "ccr restore <backup>",
            examples: vec!["ccr restore ccr-backup-20260524"],
            category: "danger",
            risk: "destructive",
            executable: true,
            requires_confirmation: true,
            args: vec![text_arg(
                "backup",
                "Backup id/path",
                true,
                Some("ccr-backup-20260524"),
                None,
                "Backup identifier or path to restore.",
            )],
            flags: vec![],
            aliases: vec![],
            related_route: None,
        },
        CommandInfo {
            name: "add",
            path: vec!["add"],
            title: "Add configuration",
            description: "Interactive CLI flow. Use the configuration page until a non-interactive schema is available.",
            usage: "ccr add",
            examples: vec!["ccr add"],
            category: "preview",
            risk: "interactive",
            executable: false,
            requires_confirmation: false,
            args: vec![],
            flags: vec![],
            aliases: vec![],
            related_route: Some("/configs"),
        },
        CommandInfo {
            name: "clean",
            path: vec!["clean"],
            title: "Clean maintenance",
            description: "Maintenance flows need explicit non-interactive targets before desktop execution.",
            usage: "ccr clean planfiles --dry-run",
            examples: vec![
                "ccr clean planfiles --dry-run",
                "ccr clean backups --days 30 --dry-run",
            ],
            category: "preview",
            risk: "preview_only",
            executable: false,
            requires_confirmation: true,
            args: vec![],
            flags: vec![],
            aliases: vec![],
            related_route: None,
        },
        CommandInfo {
            name: "platform",
            path: vec!["platform"],
            title: "Platform management",
            description: "Preview-only shortcut to platform management surfaces.",
            usage: "ccr platform <action>",
            examples: vec!["ccr platform list", "ccr platform current"],
            category: "preview",
            risk: "preview_only",
            executable: false,
            requires_confirmation: false,
            args: vec![],
            flags: vec![],
            aliases: vec![],
            related_route: Some("/settings"),
        },
        CommandInfo {
            name: "codex",
            path: vec!["codex"],
            title: "Codex management",
            description: "Use the Codex workspace pages for auth, MCP, profiles, and settings.",
            usage: "ccr codex <action>",
            examples: vec!["ccr codex auth list"],
            category: "preview",
            risk: "preview_only",
            executable: false,
            requires_confirmation: false,
            args: vec![],
            flags: vec![],
            aliases: vec![],
            related_route: Some("/codex"),
        },
        CommandInfo {
            name: "claude",
            path: vec!["claude"],
            title: "Claude management",
            description: "Use the Claude Code workspace pages for auth, MCP, agents, plugins, and settings.",
            usage: "ccr claude <action>",
            examples: vec!["ccr claude auth list"],
            category: "preview",
            risk: "preview_only",
            executable: false,
            requires_confirmation: false,
            args: vec![],
            flags: vec![],
            aliases: vec![],
            related_route: Some("/claude-code"),
        },
        CommandInfo {
            name: "opencode",
            path: vec!["opencode"],
            title: "OpenCode management",
            description: "Use the OpenCode workspace pages for provider and auth management.",
            usage: "ccr opencode <action>",
            examples: vec!["ccr opencode auth import-codex --dry-run"],
            category: "preview",
            risk: "preview_only",
            executable: false,
            requires_confirmation: false,
            args: vec![],
            flags: vec![],
            aliases: vec![],
            related_route: Some("/opencode"),
        },
        CommandInfo {
            name: "skills",
            path: vec!["skills"],
            title: "Skills management",
            description: "Use the Skills migration bridge for scanning and installation workflows.",
            usage: "ccr skills <action>",
            examples: vec!["ccr skills list"],
            category: "preview",
            risk: "preview_only",
            executable: false,
            requires_confirmation: false,
            args: vec![],
            flags: vec![],
            aliases: vec![],
            related_route: Some("/skills"),
        },
        CommandInfo {
            name: "prompts",
            path: vec!["prompts"],
            title: "Prompt presets",
            description: "Prompt preset management is routed through dedicated configuration surfaces.",
            usage: "ccr prompts <action>",
            examples: vec!["ccr prompts list"],
            category: "preview",
            risk: "preview_only",
            executable: false,
            requires_confirmation: false,
            args: vec![],
            flags: vec![],
            aliases: vec![],
            related_route: Some("/slash-commands"),
        },
        CommandInfo {
            name: "pricing",
            path: vec!["pricing"],
            title: "Pricing data",
            description: "Use the pricing page for model pricing source-of-truth and refresh actions.",
            usage: "ccr pricing <action>",
            examples: vec!["ccr pricing list"],
            category: "preview",
            risk: "preview_only",
            executable: false,
            requires_confirmation: false,
            args: vec![],
            flags: vec![],
            aliases: vec![],
            related_route: Some("/pricing"),
        },
        CommandInfo {
            name: "stats",
            path: vec!["stats"],
            title: "Usage statistics",
            description: "Use the usage dashboard for llmusage-backed statistics and cost views.",
            usage: "ccr stats <action>",
            examples: vec!["ccr stats cost --today"],
            category: "preview",
            risk: "preview_only",
            executable: false,
            requires_confirmation: false,
            args: vec![],
            flags: vec![],
            aliases: vec![],
            related_route: Some("/usage"),
        },
        CommandInfo {
            name: "budget",
            path: vec!["budget"],
            title: "Budget management",
            description: "Use the budget page for cost budget status and changes.",
            usage: "ccr budget <action>",
            examples: vec!["ccr budget status"],
            category: "preview",
            risk: "preview_only",
            executable: false,
            requires_confirmation: false,
            args: vec![],
            flags: vec![],
            aliases: vec![],
            related_route: Some("/budget"),
        },
        CommandInfo {
            name: "sessions",
            path: vec!["sessions"],
            title: "Session management",
            description: "Use the sessions page for indexed AI CLI session history.",
            usage: "ccr sessions <action>",
            examples: vec!["ccr sessions list"],
            category: "preview",
            risk: "preview_only",
            executable: false,
            requires_confirmation: false,
            args: vec![],
            flags: vec![],
            aliases: vec![],
            related_route: Some("/sessions"),
        },
    ]
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CommandJobStatus {
    Queued,
    Running,
    Success,
    Failed,
    Cancelled,
    Unavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommandJobSnapshot {
    pub job_id: String,
    pub command: String,
    pub args: Vec<String>,
    pub status: CommandJobStatus,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub duration_ms: Option<u64>,
    pub exit_code: Option<i32>,
    pub stdout_lines: Vec<String>,
    pub stderr_lines: Vec<String>,
    pub system_lines: Vec<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StartCommandJobResponse {
    pub job_id: String,
    pub snapshot: CommandJobSnapshot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputChannel {
    Stdout,
    Stderr,
    System,
}

#[derive(Debug)]
struct OutputEvent {
    channel: OutputChannel,
    line: String,
}

#[derive(Default)]
struct CommandJobRegistry {
    jobs: Mutex<HashMap<String, CommandJobSnapshot>>,
    cancel_tokens: Mutex<HashMap<String, CancellationToken>>,
}

static COMMAND_JOBS: LazyLock<CommandJobRegistry> = LazyLock::new(CommandJobRegistry::default);

fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

fn elapsed_ms(started: Instant) -> u64 {
    started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64
}

impl CommandJobSnapshot {
    fn queued(job_id: String, command: String, args: Vec<String>) -> Self {
        Self {
            job_id,
            command,
            args,
            status: CommandJobStatus::Queued,
            started_at: now_rfc3339(),
            finished_at: None,
            duration_ms: None,
            exit_code: None,
            stdout_lines: Vec::new(),
            stderr_lines: Vec::new(),
            system_lines: vec!["Job queued".to_string()],
            error: None,
        }
    }

    fn mark_running(&mut self) {
        self.status = CommandJobStatus::Running;
        self.system_lines.push("Process started".to_string());
    }

    fn push_line(&mut self, channel: OutputChannel, line: String) {
        match channel {
            OutputChannel::Stdout => self.stdout_lines.push(line),
            OutputChannel::Stderr => self.stderr_lines.push(line),
            OutputChannel::System => self.system_lines.push(line),
        }
    }

    fn mark_terminal(
        &mut self,
        status: CommandJobStatus,
        started: Instant,
        exit_code: Option<i32>,
        error: Option<String>,
    ) {
        self.status = status;
        self.finished_at = Some(now_rfc3339());
        self.duration_ms = Some(elapsed_ms(started));
        self.exit_code = exit_code;
        self.error = error;
    }
}

fn confirmation_token_for(command: &CommandInfo) -> String {
    format!("desktop-confirm:{}", command.name)
}

fn command_not_allowed_error(command: &str) -> String {
    let catalog = command_catalog();
    let allowed = catalog
        .iter()
        .filter(|entry| entry.executable)
        .map(|entry| entry.name)
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "命令 '{}' 不允许从桌面命令面板直接执行。可直接执行的命令: {}",
        command, allowed
    )
}

fn split_flag_arg(raw_arg: &str) -> Result<(&str, Option<ParsedFlagValue>), String> {
    if !raw_arg.starts_with('-') || raw_arg == "-" {
        return Err(format!("无效参数 '{}'", raw_arg));
    }

    if let Some((flag, value)) = raw_arg.split_once('=') {
        if flag.is_empty() || flag == "-" || flag == "--" {
            return Err(format!("无效参数 '{}'", raw_arg));
        }
        Ok((
            flag,
            Some(if value.is_empty() {
                ParsedFlagValue::NextArg
            } else {
                ParsedFlagValue::Inline
            }),
        ))
    } else if raw_arg.ends_with('=') {
        Ok((raw_arg.trim_end_matches('='), Some(ParsedFlagValue::NextArg)))
    } else {
        Ok((raw_arg, None))
    }
}

/// 校验子命令请求是否允许从桌面命令面板直接执行。
fn validate_command_request(request: &CommandExecutionRequest) -> Result<(), String> {
    let policy = CommandPolicy::from_command(&request.command)?;
    policy.validate(request)
}

#[cfg(test)]
fn validate_command(command: &str) -> Result<(), String> {
    validate_command_request(&CommandExecutionRequest::foreground(
        command.to_string(),
        None,
        None,
    ))
}

#[cfg(test)]
fn split_output_lines(output: &str) -> Vec<String> {
    output
        .lines()
        .map(str::trim_end)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

async fn insert_job(snapshot: CommandJobSnapshot, cancel_token: CancellationToken) {
    COMMAND_JOBS
        .jobs
        .lock()
        .await
        .insert(snapshot.job_id.clone(), snapshot.clone());
    COMMAND_JOBS
        .cancel_tokens
        .lock()
        .await
        .insert(snapshot.job_id, cancel_token);
}

async fn update_job<F>(job_id: &str, updater: F) -> Option<CommandJobSnapshot>
where
    F: FnOnce(&mut CommandJobSnapshot),
{
    let mut jobs = COMMAND_JOBS.jobs.lock().await;
    let snapshot = jobs.get_mut(job_id)?;
    updater(snapshot);
    Some(snapshot.clone())
}

async fn get_job(job_id: &str) -> Option<CommandJobSnapshot> {
    COMMAND_JOBS.jobs.lock().await.get(job_id).cloned()
}

async fn remove_cancel_token(job_id: &str) {
    COMMAND_JOBS.cancel_tokens.lock().await.remove(job_id);
}

async fn emit_job_snapshot(app_handle: &AppHandle, event: &str, snapshot: &CommandJobSnapshot) {
    if let Err(error) = app_handle.emit(event, snapshot.clone()) {
        tracing::warn!(event, ?error, job_id = %snapshot.job_id, "Failed to emit command job event");
    }
}

async fn update_and_emit<F>(app_handle: &AppHandle, event: &str, job_id: &str, updater: F)
where
    F: FnOnce(&mut CommandJobSnapshot),
{
    if let Some(snapshot) = update_job(job_id, updater).await {
        emit_job_snapshot(app_handle, event, &snapshot).await;
    }
}

async fn stream_reader<R>(reader: R, channel: OutputChannel, tx: mpsc::UnboundedSender<OutputEvent>)
where
    R: tokio::io::AsyncRead + Unpin,
{
    let mut lines = BufReader::new(reader).lines();
    loop {
        match lines.next_line().await {
            Ok(Some(line)) => {
                if tx.send(OutputEvent { channel, line }).is_err() {
                    break;
                }
            }
            Ok(None) => break,
            Err(error) => {
                let _ = tx.send(OutputEvent {
                    channel: OutputChannel::System,
                    line: format!("Output stream read error: {error}"),
                });
                break;
            }
        }
    }
}

async fn run_command_job(app_handle: AppHandle, job_id: String, cancel_token: CancellationToken) {
    let started = Instant::now();
    let Some(initial) = get_job(&job_id).await else {
        return;
    };

    update_and_emit(&app_handle, EVENT_COMMAND_JOB_PROGRESS, &job_id, |job| {
        job.mark_running();
    })
    .await;

    let mut cmd = tokio_command("ccr");
    cmd.arg(&initial.command)
        .args(&initial.args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = match cmd.spawn() {
        Ok(child) => child,
        Err(error) => {
            let (status, message) = if error.kind() == io::ErrorKind::NotFound {
                (
                    CommandJobStatus::Unavailable,
                    "CCR 二进制未找到，请确认已安装并在 PATH 中".to_string(),
                )
            } else {
                (CommandJobStatus::Failed, format!("执行失败: {error}"))
            };
            update_and_emit(&app_handle, EVENT_COMMAND_JOB_FINISHED, &job_id, |job| {
                job.push_line(OutputChannel::System, message.clone());
                job.mark_terminal(status, started, None, Some(message));
            })
            .await;
            remove_cancel_token(&job_id).await;
            return;
        }
    };

    let (tx, mut rx) = mpsc::unbounded_channel::<OutputEvent>();
    if let Some(stdout) = child.stdout.take() {
        tauri::async_runtime::spawn(stream_reader(stdout, OutputChannel::Stdout, tx.clone()));
    }
    if let Some(stderr) = child.stderr.take() {
        tauri::async_runtime::spawn(stream_reader(stderr, OutputChannel::Stderr, tx.clone()));
    }
    drop(tx);

    let mut wait_finished = false;
    let mut final_code: Option<i32> = None;
    let mut cancel_requested = false;

    while !wait_finished || !rx.is_closed() {
        tokio::select! {
            event = rx.recv(), if !rx.is_closed() => {
                if let Some(event) = event {
                    update_and_emit(&app_handle, EVENT_COMMAND_JOB_PROGRESS, &job_id, |job| {
                        job.push_line(event.channel, event.line);
                    }).await;
                }
            }
            status = child.wait(), if !wait_finished => {
                match status {
                    Ok(status) => {
                        final_code = Some(status.code().unwrap_or(-1));
                    }
                    Err(error) => {
                        update_and_emit(&app_handle, EVENT_COMMAND_JOB_PROGRESS, &job_id, |job| {
                            job.push_line(OutputChannel::System, format!("Process wait error: {error}"));
                        }).await;
                        final_code = Some(-1);
                    }
                }
                wait_finished = true;
            }
            _ = cancel_token.cancelled(), if !cancel_requested && !wait_finished => {
                cancel_requested = true;
                let kill_result = child.kill().await;
                update_and_emit(&app_handle, EVENT_COMMAND_JOB_CANCELLED, &job_id, |job| {
                    if let Err(error) = kill_result {
                        job.push_line(OutputChannel::System, format!("Cancel signal failed: {error}"));
                    } else {
                        job.push_line(OutputChannel::System, "Cancel signal sent".to_string());
                    }
                    job.mark_terminal(
                        CommandJobStatus::Cancelled,
                        started,
                        None,
                        Some("Command cancelled".to_string()),
                    );
                }).await;
            }
        }

        if wait_finished && rx.is_closed() {
            break;
        }
    }

    remove_cancel_token(&job_id).await;

    if cancel_requested {
        return;
    }

    let exit_code = final_code.unwrap_or(-1);
    let success = exit_code == 0;
    let event = if success {
        EVENT_COMMAND_JOB_FINISHED
    } else {
        EVENT_COMMAND_JOB_FINISHED
    };
    update_and_emit(&app_handle, event, &job_id, |job| {
        job.mark_terminal(
            if success {
                CommandJobStatus::Success
            } else {
                CommandJobStatus::Failed
            },
            started,
            Some(exit_code),
            if success {
                None
            } else {
                Some(format!("Command exited with code {exit_code}"))
            },
        );
    })
    .await;
}

/// 执行白名单内的 CCR CLI 子命令并返回输出
///
/// 返回 `{ success, stdout, stderr, output, error, exit_code, duration_ms }`
#[tauri::command]
pub async fn execute_ccr_command(
    command: String,
    args: Option<Vec<String>>,
    confirmation_token: Option<String>,
) -> Result<Value, String> {
    let request = CommandExecutionRequest::foreground(command, args, confirmation_token);
    validate_command_request(&request)?;

    let started = Instant::now();
    let mut cmd = tokio_command("ccr");
    cmd.arg(&request.command).args(&request.args);

    let output = cmd.output().await.map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            "CCR 二进制未找到，请确认已安装并在 PATH 中".to_string()
        } else {
            format!("执行失败: {e}")
        }
    })?;

    let exit_code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    Ok(serde_json::json!({
        "success": output.status.success(),
        "stdout": stdout,
        "stderr": stderr,
        "output": stdout,
        "error": stderr,
        "exit_code": exit_code,
        "duration_ms": elapsed_ms(started),
    }))
}

/// 启动一个 app session 内可跟踪、可取消的 CCR CLI 后台任务。
#[tauri::command]
pub async fn start_ccr_command_job(
    app_handle: AppHandle,
    command: String,
    args: Option<Vec<String>>,
    confirmation_token: Option<String>,
) -> Result<Value, String> {
    let request = CommandExecutionRequest::background(command, args, confirmation_token);
    validate_command_request(&request)?;

    let job_id = format!("ccr-command-{}", Uuid::new_v4());
    let snapshot = CommandJobSnapshot::queued(
        job_id.clone(),
        request.command,
        request.args,
    );
    let cancel_token = CancellationToken::new();
    insert_job(snapshot.clone(), cancel_token.clone()).await;

    tauri::async_runtime::spawn(run_command_job(app_handle, job_id.clone(), cancel_token));

    serde_json::to_value(StartCommandJobResponse { job_id, snapshot })
        .map_err(|e| format!("Serialization error: {e}"))
}

#[tauri::command]
pub async fn get_ccr_command_job_status(job_id: String) -> Result<Value, String> {
    let snapshot = get_job(&job_id)
        .await
        .ok_or_else(|| format!("Command job '{}' not found", job_id))?;

    serde_json::to_value(snapshot).map_err(|e| format!("Serialization error: {e}"))
}

#[tauri::command]
pub async fn cancel_ccr_command_job(
    app_handle: AppHandle,
    job_id: String,
) -> Result<Value, String> {
    if let Some(token) = COMMAND_JOBS
        .cancel_tokens
        .lock()
        .await
        .get(&job_id)
        .cloned()
    {
        token.cancel();
    }

    let snapshot = update_job(&job_id, |job| {
        if !matches!(
            job.status,
            CommandJobStatus::Success
                | CommandJobStatus::Failed
                | CommandJobStatus::Cancelled
                | CommandJobStatus::Unavailable
        ) {
            job.push_line(OutputChannel::System, "Cancel requested".to_string());
            job.status = CommandJobStatus::Cancelled;
            job.finished_at = Some(now_rfc3339());
            job.error = Some("Command cancelled".to_string());
        }
    })
    .await
    .ok_or_else(|| format!("Command job '{}' not found", job_id))?;

    emit_job_snapshot(&app_handle, EVENT_COMMAND_JOB_CANCELLED, &snapshot).await;
    serde_json::to_value(snapshot).map_err(|e| format!("Serialization error: {e}"))
}

/// 返回 CCR 命令目录和桌面执行元数据
///
/// 返回 `CommandInfo[]`，其中 `executable=false` 的条目只能作为预览/跳转。
#[tauri::command]
pub async fn list_ccr_commands() -> Result<Value, String> {
    serde_json::to_value(command_catalog()).map_err(|e| format!("Serialization error: {e}"))
}

/// 执行 `ccr help <command>` 并返回帮助文本
#[tauri::command]
pub async fn get_ccr_command_help(command: String) -> Result<Value, String> {
    CommandPolicy::from_command(&command)?;

    let output = tokio_command("ccr")
        .args(["help", &command])
        .output()
        .await
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                "CCR 二进制未找到，请确认已安装并在 PATH 中".to_string()
            } else {
                format!("执行失败: {e}")
            }
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    Ok(serde_json::json!({
        "command": command,
        "help": stdout,
        "stderr": stderr,
        "success": output.status.success(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_command_rejects_non_whitelisted_command() {
        assert!(validate_command("status").is_ok());
        let error = validate_command("platform").expect_err("platform must stay blocked");
        assert!(error.contains("不允许"));
        assert!(
            validate_command("add").is_err(),
            "interactive commands must stay blocked"
        );
    }

    #[test]
    fn command_policy_allows_safe_command_with_declared_flag() {
        let request = CommandExecutionRequest::foreground(
            "status".to_string(),
            Some(vec!["--json".to_string()]),
            None,
        );

        assert!(validate_command_request(&request).is_ok());
    }

    #[test]
    fn command_policy_rejects_unknown_flag_before_process_spawn() {
        let request = CommandExecutionRequest::foreground(
            "status".to_string(),
            Some(vec!["--delete-everything".to_string()]),
            None,
        );

        let error = validate_command_request(&request).expect_err("unknown flags are blocked");
        assert!(error.contains("不允许参数"));
        assert!(error.contains("--delete-everything"));
    }

    #[test]
    fn command_policy_rejects_extra_positional_args() {
        let request = CommandExecutionRequest::foreground(
            "status".to_string(),
            Some(vec!["surprise".to_string()]),
            None,
        );

        let error =
            validate_command_request(&request).expect_err("extra positional args are blocked");
        assert!(error.contains("不接受额外位置参数"));
    }

    #[test]
    fn command_policy_rejects_missing_required_arg() {
        let request = CommandExecutionRequest::foreground("delete".to_string(), Some(vec![]), None);

        let error =
            validate_command_request(&request).expect_err("required args must be enforced");
        assert!(error.contains("需要桌面确认") || error.contains("缺少必需位置参数"));
    }

    #[test]
    fn command_policy_rejects_destructive_command_without_confirmation() {
        let request = CommandExecutionRequest::foreground(
            "delete".to_string(),
            Some(vec!["old".to_string()]),
            None,
        );

        let error =
            validate_command_request(&request).expect_err("destructive commands need confirmation");
        assert!(error.contains("需要桌面确认"));
    }

    #[test]
    fn command_policy_allows_destructive_command_with_confirmation_and_declared_force_flag() {
        let request = CommandExecutionRequest::foreground(
            "delete".to_string(),
            Some(vec!["old".to_string(), "--force".to_string()]),
            Some("desktop-confirm:delete".to_string()),
        );

        assert!(validate_command_request(&request).is_ok());
    }

    #[test]
    fn command_policy_rejects_force_flag_without_confirmation() {
        let request = CommandExecutionRequest::foreground(
            "delete".to_string(),
            Some(vec!["old".to_string(), "--force".to_string()]),
            None,
        );

        let error = validate_command_request(&request).expect_err("--force must stay gated");
        assert!(error.contains("需要桌面确认"));
    }

    #[test]
    fn command_policy_applies_same_checks_to_background_jobs() {
        let request = CommandExecutionRequest::background(
            "import".to_string(),
            Some(vec![
                "ccr-config.toml".to_string(),
                "--merge".to_string(),
                "--backup".to_string(),
                "-f".to_string(),
            ]),
            Some("desktop-confirm:import".to_string()),
        );

        assert!(validate_command_request(&request).is_ok());

        let missing_confirmation = CommandExecutionRequest::background(
            "import".to_string(),
            Some(vec!["ccr-config.toml".to_string(), "--merge".to_string()]),
            None,
        );
        let error = validate_command_request(&missing_confirmation)
            .expect_err("job path cannot be looser than sync path");
        assert!(error.contains("需要桌面确认"));
    }

    #[test]
    fn command_policy_rejects_value_flags_without_values() {
        let request = CommandExecutionRequest::foreground(
            "history".to_string(),
            Some(vec!["--limit".to_string()]),
            None,
        );

        let error = validate_command_request(&request).expect_err("value flags need values");
        assert!(error.contains("缺少值"));
    }

    #[test]
    fn command_policy_accepts_short_aliases_declared_in_catalog() {
        let request = CommandExecutionRequest::foreground(
            "history".to_string(),
            Some(vec!["-l".to_string(), "5".to_string(), "-t=switch".to_string()]),
            None,
        );

        assert!(validate_command_request(&request).is_ok());
    }

    #[test]
    fn command_catalog_marks_execution_boundaries_with_metadata() {
        let catalog = command_catalog();
        let delete = catalog
            .iter()
            .find(|command| command.name == "delete")
            .expect("delete metadata exists");
        assert_eq!(delete.risk, "destructive");
        assert!(delete.executable);
        assert!(delete.requires_confirmation);
        assert!(
            delete
                .args
                .iter()
                .any(|arg| arg.name == "config_name" && arg.required)
        );

        let platform = catalog
            .iter()
            .find(|command| command.name == "platform")
            .expect("platform preview metadata exists");
        assert_eq!(platform.risk, "preview_only");
        assert!(!platform.executable);
        assert_eq!(platform.related_route, Some("/settings"));

        for command in catalog.iter().filter(|command| command.executable) {
            assert!(
                ALLOWED_COMMANDS.contains(&command.name),
                "{} is executable but missing from the process whitelist",
                command.name
            );
        }
    }

    #[test]
    fn split_output_lines_preserves_channel_text_without_blank_noise() {
        assert_eq!(
            split_output_lines("first\n\nsecond\r\n"),
            vec!["first".to_string(), "second".to_string()]
        );
    }

    #[test]
    fn command_job_snapshot_serializes_required_status_values() {
        let snapshot = CommandJobSnapshot::queued(
            "job-1".to_string(),
            "status".to_string(),
            vec!["--json".to_string()],
        );
        let value = serde_json::to_value(snapshot).expect("snapshot serializes");

        assert_eq!(value["job_id"], "job-1");
        assert_eq!(value["command"], "status");
        assert_eq!(value["args"][0], "--json");
        assert_eq!(value["status"], "queued");
        assert!(value["stdout_lines"].is_array());
        assert!(value["stderr_lines"].is_array());
        assert!(value["system_lines"].is_array());
    }

    #[test]
    fn terminal_snapshot_keeps_non_zero_exit_as_failed_with_output() {
        let started = Instant::now();
        let mut snapshot =
            CommandJobSnapshot::queued("job-2".to_string(), "validate".to_string(), Vec::new());

        snapshot.push_line(OutputChannel::Stdout, "partial stdout".to_string());
        snapshot.push_line(OutputChannel::Stderr, "validation failed".to_string());
        snapshot.mark_terminal(
            CommandJobStatus::Failed,
            started,
            Some(2),
            Some("Command exited with code 2".to_string()),
        );

        assert_eq!(snapshot.status, CommandJobStatus::Failed);
        assert_eq!(snapshot.exit_code, Some(2));
        assert_eq!(snapshot.stdout_lines, vec!["partial stdout"]);
        assert_eq!(snapshot.stderr_lines, vec!["validation failed"]);
        assert!(snapshot.duration_ms.is_some());
        assert!(snapshot.error.as_deref().unwrap_or_default().contains('2'));
    }
}
