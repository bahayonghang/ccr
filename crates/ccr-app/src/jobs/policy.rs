//! Shared existing desktop command catalog and request policy; Tauri wire types remain in its adapter.
use serde::Serialize;

pub const ALLOWED_COMMANDS: &[&str] = &[
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
pub struct CommandExecutionRequest {
    pub command: String,
    pub args: Vec<String>,
    pub confirmation_token: Option<String>,
    pub background_job: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandPolicy {
    pub info: CommandInfo,
    pub allow_background_job: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParsedFlagValue {
    Inline,
    NextArg,
}

impl CommandExecutionRequest {
    pub fn help(command: &str) -> Result<Self, String> {
        CommandPolicy::from_command(command)?;
        Ok(Self::foreground(
            "help".into(),
            Some(vec![command.to_owned()]),
            None,
        ))
    }

    pub fn foreground(
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

    pub fn background(
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
    pub fn from_command(command: &str) -> Result<Self, String> {
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

    pub fn validate(&self, request: &CommandExecutionRequest) -> Result<(), String> {
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

pub fn command_catalog() -> Vec<CommandInfo> {
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
            name: "grok",
            path: vec!["grok"],
            title: "Grok management",
            description: "Use the Grok workspace pages for profiles, settings, and official session logout.",
            usage: "ccr grok <action>",
            examples: vec!["ccr grok auth off"],
            category: "preview",
            risk: "preview_only",
            executable: false,
            requires_confirmation: false,
            args: vec![],
            flags: vec![],
            aliases: vec![],
            related_route: Some("/grok"),
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

pub fn confirmation_token_for(command: &CommandInfo) -> String {
    format!("desktop-confirm:{}", command.name)
}

pub fn command_not_allowed_error(command: &str) -> String {
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

pub fn split_flag_arg(raw_arg: &str) -> Result<(&str, Option<ParsedFlagValue>), String> {
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

pub fn validate_command_request(request: &CommandExecutionRequest) -> Result<(), String> {
    let policy = CommandPolicy::from_command(&request.command)?;
    policy.validate(request)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

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
}
