//! 命令执行模块 — CCR CLI 命令白名单执行。

use std::collections::{HashMap, VecDeque};
use std::ffi::OsString;
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant};

use crate::process::{ProcessDescriptor, ProcessGateway, read_bounded_line};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio::io::BufReader;
use tokio::sync::{Mutex, mpsc};
use tokio_util::sync::CancellationToken;
use ts_rs::TS;
use uuid::Uuid;

const EVENT_COMMAND_JOB_PROGRESS: &str = "commands:job-progress";
const EVENT_COMMAND_JOB_FINISHED: &str = "commands:job-finished";
const EVENT_COMMAND_JOB_CANCELLED: &str = "commands:job-cancelled";

const COMMAND_JOB_MAX_JOBS: usize = 64;
const COMMAND_JOB_TTL_SECS: u64 = 30 * 60;
const COMMAND_JOB_MAX_LINES_PER_CHANNEL: usize = 500;
const COMMAND_JOB_MAX_BYTES_PER_CHANNEL: usize = 512 * 1024;
const COMMAND_JOB_OUTPUT_CHANNEL_CAPACITY: usize = 256;
const COMMAND_JOB_OUTPUT_BATCH_LINES: usize = 50;
const COMMAND_JOB_OUTPUT_BATCH_INTERVAL: Duration = Duration::from_millis(100);
const COMMAND_JOB_TERMINATION_GRACE: Duration = Duration::from_secs(5);

/// 允许执行的 CCR 子命令白名单
const ALLOWED_COMMANDS: &[&str] = &[
    "list",
    "switch",
    "add",
    "rename",
    "duplicate",
    "show",
    "validate",
    "export",
    "history",
    "version",
    "help",
    "backup",
    "diff",
    "status",
];

#[derive(Debug, Clone, Serialize, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/types/generated/command_exec/")]
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

#[derive(Debug, Clone, Serialize, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/types/generated/command_exec/")]
pub struct CommandFlagSchema {
    pub name: &'static str,
    pub aliases: Vec<&'static str>,
    pub label: &'static str,
    pub description: &'static str,
    #[serde(rename = "type")]
    pub flag_type: &'static str,
    pub takes_value: bool,
    pub default_value: Option<&'static str>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/types/generated/command_exec/")]
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
                    format!("命令 '{}' 不允许参数 '{}'。", self.info.name, flag_name)
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
                                format!("命令 '{}' 的参数 '{}' 缺少值。", self.info.name, flag_name)
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
            description: "Use the typed desktop configuration API to delete a saved CCR configuration.",
            usage: "Config domain: delete_config(name)",
            examples: vec!["delete_config({ name: 'old-config' })"],
            category: "danger",
            risk: "typed_only",
            executable: false,
            requires_confirmation: false,
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
            description: "Use the typed desktop configuration API to import TOML content.",
            usage: "Config domain: import_config(content, mode, backup)",
            examples: vec!["import_config({ content, mode: 'merge', backup: true })"],
            category: "danger",
            risk: "typed_only",
            executable: false,
            requires_confirmation: false,
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
            description: "Restore is not exposed through generic desktop command passthrough.",
            usage: "Dedicated restore command pending typed service boundary",
            examples: vec![],
            category: "danger",
            risk: "typed_only",
            executable: false,
            requires_confirmation: false,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../src/types/generated/command_exec/")]
pub enum CommandJobStatus {
    Queued,
    Running,
    Success,
    Failed,
    Cancelled,
    CleanupFailed,
    Unavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TS)]
#[ts(export, export_to = "../../src/types/generated/command_exec/")]
pub struct CommandJobSnapshot {
    pub job_id: String,
    pub command: String,
    pub args: Vec<String>,
    pub status: CommandJobStatus,
    pub started_at: String,
    pub finished_at: Option<String>,
    #[ts(as = "Option<f64>")]
    pub duration_ms: Option<u64>,
    pub exit_code: Option<i32>,
    #[ts(as = "Vec<String>")]
    pub stdout_lines: VecDeque<String>,
    #[ts(as = "Vec<String>")]
    pub stderr_lines: VecDeque<String>,
    #[ts(as = "Vec<String>")]
    pub system_lines: VecDeque<String>,
    pub truncated: bool,
    pub dropped_lines: usize,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/command_exec/")]
pub struct StartCommandJobResponse {
    pub job_id: String,
    pub snapshot: CommandJobSnapshot,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export, export_to = "../../src/types/generated/command_exec/")]
pub enum OutputChannel {
    Stdout,
    Stderr,
    System,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/command_exec/")]
pub struct CommandJobDelta {
    pub job_id: String,
    #[ts(as = "f64")]
    pub seq: u64,
    pub channel: OutputChannel,
    pub lines: Vec<String>,
    pub dropped_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub status: Option<CommandJobStatus>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/command_exec/")]
pub struct CommandExecutionResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub output: String,
    pub error: String,
    pub exit_code: i32,
    #[ts(as = "f64")]
    pub duration_ms: u64,
    pub timed_out: bool,
    pub truncated: bool,
    pub stdout_bytes: usize,
    pub stderr_bytes: usize,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(transparent)]
#[ts(export, export_to = "../../src/types/generated/command_exec/")]
pub struct CommandCatalog(pub Vec<CommandInfo>);

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/command_exec/")]
pub struct CommandHelpResponse {
    pub command: String,
    pub help: String,
    pub stderr: String,
    pub success: bool,
    pub timed_out: bool,
    pub truncated: bool,
}

#[derive(Debug)]
struct OutputEvent {
    channel: OutputChannel,
    lines: Vec<String>,
    dropped_count: usize,
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

fn sorted_prune_candidates(
    jobs: &HashMap<String, CommandJobSnapshot>,
    include_terminal_only: bool,
) -> Vec<String> {
    let mut candidates = jobs
        .values()
        .filter(|job| !include_terminal_only || job.is_terminal())
        .map(|job| {
            (
                job.finished_datetime()
                    .or_else(|| job.started_datetime())
                    .unwrap_or_else(Utc::now),
                job.job_id.clone(),
            )
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
    candidates.into_iter().map(|(_, job_id)| job_id).collect()
}

fn prune_expired_jobs_locked(
    jobs: &mut HashMap<String, CommandJobSnapshot>,
    now: DateTime<Utc>,
) -> Vec<String> {
    let expired = jobs
        .iter()
        .filter(|(_, snapshot)| snapshot.is_expired(now))
        .map(|(job_id, _)| job_id.clone())
        .collect::<Vec<_>>();
    for job_id in &expired {
        jobs.remove(job_id);
    }
    expired
}

fn prune_to_capacity_locked(
    jobs: &mut HashMap<String, CommandJobSnapshot>,
    capacity: usize,
) -> Vec<String> {
    let mut removed = Vec::new();
    if capacity == 0 {
        removed.extend(jobs.keys().cloned().collect::<Vec<_>>());
        jobs.clear();
        return removed;
    }

    if jobs.len() <= capacity {
        return removed;
    }

    for job_id in sorted_prune_candidates(jobs, true) {
        if jobs.len() <= capacity {
            break;
        }
        if jobs.remove(&job_id).is_some() {
            removed.push(job_id);
        }
    }

    removed
}

async fn remove_cancel_tokens(job_ids: &[String]) {
    if job_ids.is_empty() {
        return;
    }

    let mut cancel_tokens = COMMAND_JOBS.cancel_tokens.lock().await;
    for job_id in job_ids {
        cancel_tokens.remove(job_id);
    }
}

async fn prune_jobs_by_policy() -> usize {
    let removed = {
        let mut jobs = COMMAND_JOBS.jobs.lock().await;
        let mut removed = prune_expired_jobs_locked(&mut jobs, Utc::now());
        removed.extend(prune_to_capacity_locked(&mut jobs, COMMAND_JOB_MAX_JOBS));
        removed
    };

    remove_cancel_tokens(&removed).await;
    removed.len()
}

pub(crate) async fn prune_command_jobs() -> usize {
    prune_jobs_by_policy().await
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
            stdout_lines: VecDeque::new(),
            stderr_lines: VecDeque::new(),
            system_lines: VecDeque::from(["Job queued".to_string()]),
            truncated: false,
            dropped_lines: 0,
            error: None,
        }
    }

    fn mark_running(&mut self) {
        self.status = CommandJobStatus::Running;
        self.push_line(OutputChannel::System, "Process started".to_string());
    }

    fn push_line(&mut self, channel: OutputChannel, line: String) {
        match channel {
            OutputChannel::Stdout => push_capped_line(
                &mut self.stdout_lines,
                &mut self.truncated,
                &mut self.dropped_lines,
                line,
            ),
            OutputChannel::Stderr => push_capped_line(
                &mut self.stderr_lines,
                &mut self.truncated,
                &mut self.dropped_lines,
                line,
            ),
            OutputChannel::System => push_capped_line(
                &mut self.system_lines,
                &mut self.truncated,
                &mut self.dropped_lines,
                line,
            ),
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

    fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            CommandJobStatus::Success
                | CommandJobStatus::Failed
                | CommandJobStatus::Cancelled
                | CommandJobStatus::CleanupFailed
                | CommandJobStatus::Unavailable
        )
    }

    fn finished_datetime(&self) -> Option<DateTime<Utc>> {
        DateTime::parse_from_rfc3339(self.finished_at.as_deref()?)
            .ok()
            .map(|value| value.with_timezone(&Utc))
    }

    fn started_datetime(&self) -> Option<DateTime<Utc>> {
        DateTime::parse_from_rfc3339(&self.started_at)
            .ok()
            .map(|value| value.with_timezone(&Utc))
    }

    fn is_expired(&self, now: DateTime<Utc>) -> bool {
        if !self.is_terminal() {
            return false;
        }

        self.finished_datetime().is_some_and(|finished_at| {
            now.signed_duration_since(finished_at).num_seconds() >= COMMAND_JOB_TTL_SECS as i64
        })
    }
}

fn channel_size_bytes(lines: &VecDeque<String>) -> usize {
    lines.iter().map(|line| line.len()).sum()
}

fn truncate_line_to_bytes(line: String) -> (String, bool) {
    if line.len() <= COMMAND_JOB_MAX_BYTES_PER_CHANNEL {
        return (line, false);
    }

    (append_truncation_marker(line), true)
}

fn append_truncation_marker(mut line: String) -> String {
    const MARKER: &str = "…";
    let budget = COMMAND_JOB_MAX_BYTES_PER_CHANNEL.saturating_sub(MARKER.len());
    let mut end = 0usize;
    for (index, ch) in line.char_indices() {
        let next = index + ch.len_utf8();
        if next > budget {
            break;
        }
        end = next;
    }

    line.truncate(end);
    if COMMAND_JOB_MAX_BYTES_PER_CHANNEL >= MARKER.len() {
        line.push_str(MARKER);
    }
    line
}

fn cancellation_terminal_state(
    cleanup_error: Option<String>,
) -> (CommandJobStatus, Option<String>) {
    match cleanup_error {
        Some(error) => (CommandJobStatus::CleanupFailed, Some(error)),
        None => (
            CommandJobStatus::Cancelled,
            Some("Command cancelled".to_string()),
        ),
    }
}

fn push_capped_line(
    lines: &mut VecDeque<String>,
    truncated: &mut bool,
    dropped_lines: &mut usize,
    line: String,
) {
    let (line, line_was_truncated) = truncate_line_to_bytes(line);
    if line_was_truncated {
        *truncated = true;
    }

    lines.push_back(line);
    while lines.len() > COMMAND_JOB_MAX_LINES_PER_CHANNEL
        || channel_size_bytes(lines) > COMMAND_JOB_MAX_BYTES_PER_CHANNEL
    {
        if lines.is_empty() {
            break;
        }
        lines.pop_front();
        *truncated = true;
        *dropped_lines += 1;
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
        Ok((
            raw_arg.trim_end_matches('='),
            Some(ParsedFlagValue::NextArg),
        ))
    } else {
        Ok((raw_arg, None))
    }
}

async fn verify_ccr_sidecar_version() -> Result<(), String> {
    let descriptor = ProcessDescriptor::ccr_version_probe();
    let output = ProcessGateway::execute(&descriptor, &[OsString::from("--version")]).await?;
    if output.timed_out {
        return Err("ccr_version_probe_timeout".to_string());
    }
    if !output.status.success() {
        return Err(format!(
            "ccr_version_probe_failed: exit code {:?}",
            output.status.code()
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let actual = parse_ccr_version_output(&stdout)
        .ok_or_else(|| "ccr_version_probe_invalid_output".to_string())?;
    let expected = env!("CARGO_PKG_VERSION");
    if actual != expected {
        return Err(format!(
            "ccr_version_mismatch: desktop {expected}, sidecar {actual}"
        ));
    }
    Ok(())
}

fn parse_ccr_version_output(output: &str) -> Option<&str> {
    output
        .split_whitespace()
        .find(|part| part.chars().next().is_some_and(|ch| ch.is_ascii_digit()))
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

async fn insert_job(
    snapshot: CommandJobSnapshot,
    cancel_token: CancellationToken,
) -> Result<(), String> {
    let removed = {
        let mut jobs = COMMAND_JOBS.jobs.lock().await;
        let mut removed = prune_expired_jobs_locked(&mut jobs, Utc::now());
        if jobs.len() >= COMMAND_JOB_MAX_JOBS {
            removed.extend(prune_to_capacity_locked(
                &mut jobs,
                COMMAND_JOB_MAX_JOBS.saturating_sub(1),
            ));
        }

        if jobs.len() >= COMMAND_JOB_MAX_JOBS {
            return Err(format!(
                "后台命令任务已达到上限（{} 个），请等待正在运行的任务完成后重试。",
                COMMAND_JOB_MAX_JOBS
            ));
        }

        jobs.insert(snapshot.job_id.clone(), snapshot.clone());
        removed
    };
    remove_cancel_tokens(&removed).await;
    COMMAND_JOBS
        .cancel_tokens
        .lock()
        .await
        .insert(snapshot.job_id, cancel_token);
    Ok(())
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
    prune_jobs_by_policy().await;
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

fn try_send_output_batch(
    tx: &mpsc::Sender<OutputEvent>,
    channel: OutputChannel,
    batch: &mut Vec<String>,
    dropped: &AtomicUsize,
) -> bool {
    if batch.is_empty() {
        return true;
    }

    let event = OutputEvent {
        channel,
        lines: std::mem::take(batch),
        dropped_count: dropped.swap(0, Ordering::AcqRel),
    };
    match tx.try_send(event) {
        Ok(()) => true,
        Err(mpsc::error::TrySendError::Full(event)) => {
            dropped.fetch_add(
                event.lines.len().saturating_add(event.dropped_count),
                Ordering::Relaxed,
            );
            true
        }
        Err(mpsc::error::TrySendError::Closed(_)) => false,
    }
}

async fn stream_reader<R>(
    reader: R,
    channel: OutputChannel,
    tx: mpsc::Sender<OutputEvent>,
    dropped: Arc<AtomicUsize>,
) where
    R: tokio::io::AsyncRead + Unpin,
{
    let mut reader = BufReader::new(reader);
    let mut batch = Vec::with_capacity(COMMAND_JOB_OUTPUT_BATCH_LINES);
    let mut interval = tokio::time::interval(COMMAND_JOB_OUTPUT_BATCH_INTERVAL);
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    interval.tick().await;

    loop {
        tokio::select! {
            next = read_bounded_line(&mut reader, COMMAND_JOB_MAX_BYTES_PER_CHANNEL) => match next {
                Ok(Some(line)) => {
                    let line = if line.truncated {
                        dropped.fetch_add(1, Ordering::Relaxed);
                        append_truncation_marker(line.text)
                    } else {
                        line.text
                    };
                    if batch.len() < COMMAND_JOB_OUTPUT_BATCH_LINES {
                        batch.push(line);
                    } else {
                        dropped.fetch_add(1, Ordering::Relaxed);
                    }
                }
                Ok(None) => {
                    let _ = try_send_output_batch(&tx, channel, &mut batch, &dropped);
                    break;
                }
                Err(error) => {
                    let _ = try_send_output_batch(&tx, channel, &mut batch, &dropped);
                    let mut error_batch = vec![format!("Output stream read error: {error}")];
                    let _ = try_send_output_batch(
                        &tx,
                        OutputChannel::System,
                        &mut error_batch,
                        &dropped,
                    );
                    break;
                }
            },
            _ = interval.tick() => {
                if !try_send_output_batch(&tx, channel, &mut batch, &dropped) {
                    break;
                }
            }
        }
    }
}

async fn apply_output_event(
    app_handle: &AppHandle,
    job_id: &str,
    sequence: &AtomicU64,
    event: OutputEvent,
) {
    let lines = event.lines;
    let emitted_lines = lines.clone();
    let dropped_count = event.dropped_count;
    if update_job(job_id, |job| {
        for line in lines {
            job.push_line(event.channel, line);
        }
        if dropped_count > 0 {
            job.truncated = true;
            job.dropped_lines = job.dropped_lines.saturating_add(dropped_count);
        }
    })
    .await
    .is_none()
    {
        return;
    }

    let delta = CommandJobDelta {
        job_id: job_id.to_string(),
        seq: sequence.fetch_add(1, Ordering::Relaxed),
        channel: event.channel,
        lines: emitted_lines,
        dropped_count,
        status: None,
    };
    if let Err(error) = app_handle.emit(EVENT_COMMAND_JOB_PROGRESS, delta) {
        tracing::warn!(?error, %job_id, "Failed to emit command job delta");
    }
}

async fn run_command_job(app_handle: AppHandle, job_id: String, cancel_token: CancellationToken) {
    let started = Instant::now();
    let sequence = AtomicU64::new(0);
    let Some(initial) = get_job(&job_id).await else {
        return;
    };

    update_job(&job_id, |job| {
        job.mark_running();
    })
    .await;
    let started_delta = CommandJobDelta {
        job_id: job_id.clone(),
        seq: sequence.fetch_add(1, Ordering::Relaxed),
        channel: OutputChannel::System,
        lines: vec!["Process started".to_string()],
        dropped_count: 0,
        status: Some(CommandJobStatus::Running),
    };
    if let Err(error) = app_handle.emit(EVENT_COMMAND_JOB_PROGRESS, started_delta) {
        tracing::warn!(?error, %job_id, "Failed to emit command job start delta");
    }

    if let Err(message) = verify_ccr_sidecar_version().await {
        update_and_emit(&app_handle, EVENT_COMMAND_JOB_FINISHED, &job_id, |job| {
            job.push_line(OutputChannel::System, message.clone());
            job.mark_terminal(CommandJobStatus::Unavailable, started, None, Some(message));
        })
        .await;
        remove_cancel_token(&job_id).await;
        return;
    }

    let descriptor = ProcessDescriptor::ccr_command();
    let mut cmd = match ProcessGateway::command(&descriptor) {
        Ok(command) => command,
        Err(message) => {
            update_and_emit(&app_handle, EVENT_COMMAND_JOB_FINISHED, &job_id, |job| {
                job.push_line(OutputChannel::System, message.clone());
                job.mark_terminal(CommandJobStatus::Unavailable, started, None, Some(message));
            })
            .await;
            remove_cancel_token(&job_id).await;
            return;
        }
    };
    cmd.arg(&initial.command)
        .args(&initial.args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = match ProcessGateway::spawn(cmd, &descriptor, cancel_token.clone(), Vec::new())
    {
        Ok(child) => child,
        Err(message) => {
            update_and_emit(&app_handle, EVENT_COMMAND_JOB_FINISHED, &job_id, |job| {
                job.push_line(OutputChannel::System, message.clone());
                job.mark_terminal(CommandJobStatus::Failed, started, None, Some(message));
            })
            .await;
            remove_cancel_token(&job_id).await;
            return;
        }
    };

    let (tx, mut rx) = mpsc::channel::<OutputEvent>(COMMAND_JOB_OUTPUT_CHANNEL_CAPACITY);
    if let Some(stdout) = child.take_stdout() {
        tauri::async_runtime::spawn(stream_reader(
            stdout,
            OutputChannel::Stdout,
            tx.clone(),
            Arc::new(AtomicUsize::new(0)),
        ));
    }
    if let Some(stderr) = child.take_stderr() {
        tauri::async_runtime::spawn(stream_reader(
            stderr,
            OutputChannel::Stderr,
            tx.clone(),
            Arc::new(AtomicUsize::new(0)),
        ));
    }
    drop(tx);

    let mut wait_finished = false;
    let mut final_code: Option<i32> = None;
    let mut cancel_requested = false;
    let mut timed_out = false;
    let mut cleanup_error: Option<String> = None;
    let deadline = tokio::time::sleep(descriptor.timeout());
    tokio::pin!(deadline);

    while !wait_finished || !rx.is_closed() || !rx.is_empty() {
        tokio::select! {
            event = rx.recv(), if !rx.is_closed() || !rx.is_empty() => {
                if let Some(event) = event {
                    apply_output_event(&app_handle, &job_id, &sequence, event).await;
                }
            }
            status = child.wait(), if !wait_finished => {
                match status {
                    Ok(status) => {
                        final_code = Some(status.code().unwrap_or(-1));
                    }
                    Err(error) => {
                        cleanup_error = Some(format!("process_wait_failed: {error}"));
                        final_code = Some(-1);
                    }
                }
                wait_finished = true;
            }
            _ = cancel_token.cancelled(), if !cancel_requested && !wait_finished => {
                cancel_requested = true;
                match child.terminate_tree(COMMAND_JOB_TERMINATION_GRACE).await {
                    Ok(status) => final_code = Some(status.code().unwrap_or(-1)),
                    Err(error) => cleanup_error = Some(format!("process_tree_cleanup_failed: {error}")),
                }
                wait_finished = true;
            }
            _ = &mut deadline, if !wait_finished => {
                timed_out = true;
                match child.terminate_tree(COMMAND_JOB_TERMINATION_GRACE).await {
                    Ok(status) => final_code = Some(status.code().unwrap_or(-1)),
                    Err(error) => cleanup_error = Some(format!("process_tree_cleanup_failed: {error}")),
                }
                wait_finished = true;
            }
        }
    }

    remove_cancel_token(&job_id).await;

    if cancel_requested {
        let (status, error) = cancellation_terminal_state(cleanup_error);
        update_and_emit(&app_handle, EVENT_COMMAND_JOB_CANCELLED, &job_id, |job| {
            job.mark_terminal(status, started, final_code, error);
        })
        .await;
        return;
    }

    if timed_out {
        let status = if cleanup_error.is_some() {
            CommandJobStatus::CleanupFailed
        } else {
            CommandJobStatus::Failed
        };
        let error = cleanup_error.or_else(|| Some("process_timeout".to_string()));
        update_and_emit(&app_handle, EVENT_COMMAND_JOB_FINISHED, &job_id, |job| {
            job.mark_terminal(status, started, final_code, error);
        })
        .await;
        return;
    }

    let exit_code = final_code.unwrap_or(-1);
    let success = exit_code == 0 && cleanup_error.is_none();
    update_and_emit(&app_handle, EVENT_COMMAND_JOB_FINISHED, &job_id, |job| {
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
                cleanup_error.or_else(|| Some(format!("Command exited with code {exit_code}")))
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
) -> Result<CommandExecutionResult, String> {
    let request = CommandExecutionRequest::foreground(command, args, confirmation_token);
    validate_command_request(&request)?;

    verify_ccr_sidecar_version().await?;
    let descriptor = ProcessDescriptor::ccr_command();
    let command_args = std::iter::once(OsString::from(&request.command))
        .chain(request.args.iter().map(OsString::from))
        .collect::<Vec<_>>();
    let output = ProcessGateway::execute(&descriptor, &command_args).await?;

    let exit_code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let truncated = output.stdout_truncated || output.stderr_truncated;

    Ok(CommandExecutionResult {
        success: output.status.success() && !output.timed_out && !truncated,
        output: stdout.clone(),
        error: stderr.clone(),
        stdout,
        stderr,
        exit_code,
        duration_ms: output.duration.as_millis().min(u128::from(u64::MAX)) as u64,
        timed_out: output.timed_out,
        truncated,
        stdout_bytes: output.stdout_bytes,
        stderr_bytes: output.stderr_bytes,
    })
}

/// 启动一个 app session 内可跟踪、可取消的 CCR CLI 后台任务。
#[tauri::command]
pub async fn start_ccr_command_job(
    app_handle: AppHandle,
    command: String,
    args: Option<Vec<String>>,
    confirmation_token: Option<String>,
) -> Result<StartCommandJobResponse, String> {
    let request = CommandExecutionRequest::background(command, args, confirmation_token);
    validate_command_request(&request)?;

    let job_id = format!("ccr-command-{}", Uuid::new_v4());
    let snapshot = CommandJobSnapshot::queued(job_id.clone(), request.command, request.args);
    let cancel_token = CancellationToken::new();
    insert_job(snapshot.clone(), cancel_token.clone()).await?;

    tauri::async_runtime::spawn(run_command_job(app_handle, job_id.clone(), cancel_token));

    Ok(StartCommandJobResponse { job_id, snapshot })
}

#[tauri::command]
pub async fn get_ccr_command_job_status(job_id: String) -> Result<CommandJobSnapshot, String> {
    let snapshot = get_job(&job_id)
        .await
        .ok_or_else(|| format!("Command job '{}' not found", job_id))?;

    Ok(snapshot)
}

#[tauri::command]
pub async fn cancel_ccr_command_job(
    _app_handle: AppHandle,
    job_id: String,
) -> Result<CommandJobSnapshot, String> {
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
                | CommandJobStatus::CleanupFailed
                | CommandJobStatus::Unavailable
        ) {
            job.push_line(OutputChannel::System, "Cancel requested".to_string());
        }
    })
    .await
    .ok_or_else(|| format!("Command job '{}' not found", job_id))?;

    Ok(snapshot)
}

/// 返回 CCR 命令目录和桌面执行元数据
///
/// 返回 `CommandInfo[]`，其中 `executable=false` 的条目只能作为预览/跳转。
#[tauri::command]
pub async fn list_ccr_commands() -> Result<CommandCatalog, String> {
    Ok(CommandCatalog(command_catalog()))
}

/// 执行 `ccr help <command>` 并返回帮助文本
#[tauri::command]
pub async fn get_ccr_command_help(command: String) -> Result<CommandHelpResponse, String> {
    CommandPolicy::from_command(&command)?;

    verify_ccr_sidecar_version().await?;
    let output = ProcessGateway::execute(
        &ProcessDescriptor::ccr_command(),
        &[OsString::from("help"), OsString::from(&command)],
    )
    .await?;

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    Ok(CommandHelpResponse {
        command,
        help: stdout,
        stderr,
        success: output.status.success()
            && !output.timed_out
            && !output.stdout_truncated
            && !output.stderr_truncated,
        timed_out: output.timed_out,
        truncated: output.stdout_truncated || output.stderr_truncated,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncWriteExt;

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
    fn command_policy_rejects_destructive_passthrough_commands_even_with_confirmation() {
        for command in ["delete", "import", "restore"] {
            let request = CommandExecutionRequest::foreground(
                command.to_string(),
                Some(vec!["target".to_string(), "--force".to_string()]),
                Some(format!("desktop-confirm:{command}")),
            );

            let error = validate_command_request(&request)
                .expect_err("destructive commands must use typed Tauri APIs");
            assert!(
                error.contains("不允许"),
                "{command} should be rejected by the generic process policy: {error}"
            );
        }
    }

    #[test]
    fn command_policy_rejects_destructive_passthrough_for_background_jobs() {
        for command in ["delete", "import", "restore"] {
            let request = CommandExecutionRequest::background(
                command.to_string(),
                Some(vec!["target".to_string()]),
                Some(format!("desktop-confirm:{command}")),
            );

            let error = validate_command_request(&request)
                .expect_err("job path cannot re-enable destructive passthrough");
            assert!(
                error.contains("不允许"),
                "{command} should be rejected by the generic job policy: {error}"
            );
        }
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
            Some(vec![
                "-l".to_string(),
                "5".to_string(),
                "-t=switch".to_string(),
            ]),
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
        assert_eq!(delete.risk, "typed_only");
        assert!(!delete.executable);
        assert!(!delete.requires_confirmation);
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

        for command_name in ["delete", "import", "restore"] {
            let command = catalog
                .iter()
                .find(|entry| entry.name == command_name)
                .expect("destructive command metadata exists");
            assert_eq!(command.risk, "typed_only");
            assert!(
                !command.executable,
                "{command_name} must use typed config commands instead of process passthrough"
            );
            assert!(
                !ALLOWED_COMMANDS.contains(&command.name),
                "{command_name} must not be in the generic process whitelist"
            );
        }
    }

    #[test]
    fn parse_ccr_version_output_extracts_semver_token() {
        assert_eq!(parse_ccr_version_output("ccr 6.3.0\n"), Some("6.3.0"));
        assert_eq!(parse_ccr_version_output("6.3.0"), Some("6.3.0"));
        assert_eq!(parse_ccr_version_output("ccr"), None);
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
        assert_eq!(value["truncated"], false);
        assert_eq!(value["dropped_lines"], 0);
    }

    #[test]
    fn cleanup_failure_never_claims_cancelled_status() {
        let (status, error) = cancellation_terminal_state(Some("access denied".to_string()));

        assert_eq!(status, CommandJobStatus::CleanupFailed);
        assert_eq!(error.as_deref(), Some("access denied"));
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
        assert_eq!(
            snapshot.stdout_lines,
            VecDeque::from(["partial stdout".to_string()])
        );
        assert_eq!(
            snapshot.stderr_lines,
            VecDeque::from(["validation failed".to_string()])
        );
        assert!(snapshot.duration_ms.is_some());
        assert!(snapshot.error.as_deref().unwrap_or_default().contains('2'));
    }

    #[test]
    fn command_job_snapshot_caps_output_lines_per_channel() {
        let mut snapshot =
            CommandJobSnapshot::queued("job-lines".to_string(), "status".to_string(), Vec::new());

        for index in 0..=COMMAND_JOB_MAX_LINES_PER_CHANNEL {
            snapshot.push_line(OutputChannel::Stdout, format!("line {index}"));
        }

        assert_eq!(
            snapshot.stdout_lines.len(),
            COMMAND_JOB_MAX_LINES_PER_CHANNEL
        );
        assert_eq!(snapshot.stdout_lines[0], "line 1");
        assert!(snapshot.truncated);
        assert_eq!(snapshot.dropped_lines, 1);
    }

    #[test]
    fn command_job_snapshot_caps_single_line_bytes() {
        let mut snapshot =
            CommandJobSnapshot::queued("job-bytes".to_string(), "status".to_string(), Vec::new());
        let long_line = "x".repeat(COMMAND_JOB_MAX_BYTES_PER_CHANNEL + 64);

        snapshot.push_line(OutputChannel::Stderr, long_line);

        assert_eq!(snapshot.stderr_lines.len(), 1);
        assert!(snapshot.stderr_lines[0].ends_with('…'));
        assert!(snapshot.stderr_lines[0].len() <= COMMAND_JOB_MAX_BYTES_PER_CHANNEL);
        assert!(snapshot.truncated);
        assert_eq!(snapshot.dropped_lines, 0);
    }

    #[test]
    fn command_job_ttl_prunes_only_terminal_jobs() {
        let now = Utc::now();
        let old_finished_at =
            (now - chrono::Duration::seconds(COMMAND_JOB_TTL_SECS as i64 + 1)).to_rfc3339();
        let mut jobs = HashMap::new();
        let mut terminal =
            CommandJobSnapshot::queued("terminal".to_string(), "status".to_string(), Vec::new());
        terminal.status = CommandJobStatus::Success;
        terminal.finished_at = Some(old_finished_at.clone());
        let mut running =
            CommandJobSnapshot::queued("running".to_string(), "status".to_string(), Vec::new());
        running.status = CommandJobStatus::Running;
        running.finished_at = Some(old_finished_at);
        jobs.insert(terminal.job_id.clone(), terminal);
        jobs.insert(running.job_id.clone(), running);

        let removed = prune_expired_jobs_locked(&mut jobs, now);

        assert_eq!(removed, vec!["terminal".to_string()]);
        assert!(!jobs.contains_key("terminal"));
        assert!(jobs.contains_key("running"));
    }

    #[test]
    fn command_job_capacity_prunes_oldest_terminal_snapshots_first() {
        let mut jobs = HashMap::new();
        for index in 0..3 {
            let mut snapshot = CommandJobSnapshot::queued(
                format!("job-{index}"),
                "status".to_string(),
                Vec::new(),
            );
            snapshot.status = CommandJobStatus::Success;
            snapshot.finished_at =
                Some((Utc::now() + chrono::Duration::seconds(index as i64)).to_rfc3339());
            jobs.insert(snapshot.job_id.clone(), snapshot);
        }

        let removed = prune_to_capacity_locked(&mut jobs, 1);

        assert_eq!(removed, vec!["job-0".to_string(), "job-1".to_string()]);
        assert_eq!(jobs.len(), 1);
        assert!(jobs.contains_key("job-2"));
    }

    #[test]
    fn command_job_capacity_keeps_active_snapshots() {
        let mut jobs = HashMap::new();
        for index in 0..3 {
            let mut snapshot = CommandJobSnapshot::queued(
                format!("active-{index}"),
                "status".to_string(),
                Vec::new(),
            );
            snapshot.status = CommandJobStatus::Running;
            jobs.insert(snapshot.job_id.clone(), snapshot);
        }

        let removed = prune_to_capacity_locked(&mut jobs, 1);

        assert!(removed.is_empty());
        assert_eq!(jobs.len(), 3);
    }

    #[tokio::test]
    async fn stalled_output_consumer_stays_bounded_and_reports_drops() {
        let (mut writer, reader) = tokio::io::duplex(64 * 1024);
        let (tx, mut rx) = mpsc::channel(1);
        let dropped = Arc::new(AtomicUsize::new(0));
        let reader_task = tokio::spawn(stream_reader(reader, OutputChannel::Stdout, tx, dropped));
        let payload = (0..1_000)
            .map(|index| format!("line-{index}\n"))
            .collect::<String>();

        writer
            .write_all(payload.as_bytes())
            .await
            .expect("write flood");
        writer.shutdown().await.expect("close writer");
        reader_task.await.expect("reader task");

        let event = rx.recv().await.expect("one bounded batch");
        assert_eq!(event.lines.len(), COMMAND_JOB_OUTPUT_BATCH_LINES);
        assert_eq!(event.dropped_count, 1_000 - COMMAND_JOB_OUTPUT_BATCH_LINES);
        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn unterminated_output_line_is_bounded_and_reported() {
        let (mut writer, reader) = tokio::io::duplex(64 * 1024);
        let (tx, mut rx) = mpsc::channel(1);
        let reader_task = tokio::spawn(stream_reader(
            reader,
            OutputChannel::Stdout,
            tx,
            Arc::new(AtomicUsize::new(0)),
        ));
        let writer_task = tokio::spawn(async move {
            writer
                .write_all(&vec![b'x'; COMMAND_JOB_MAX_BYTES_PER_CHANNEL * 2])
                .await
                .unwrap();
            writer.shutdown().await.unwrap();
        });

        writer_task.await.unwrap();
        reader_task.await.unwrap();
        let event = rx.recv().await.unwrap();

        assert_eq!(event.lines.len(), 1);
        assert!(event.lines[0].len() <= COMMAND_JOB_MAX_BYTES_PER_CHANNEL);
        assert!(event.lines[0].ends_with('…'));
        assert_eq!(event.dropped_count, 1);
    }
}
