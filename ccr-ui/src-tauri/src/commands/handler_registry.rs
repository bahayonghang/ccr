//! Domain command registry for the Tauri invoke handler.
// The registry is the single source for both command metadata and the
// `tauri::generate_handler!` command list. Keep new commands inside the
// smallest matching domain module instead of expanding `commands::mod` again.

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CommandRisk {
    ReadOnly,
    LocalMutation,
    SecretMutation,
    NetworkMutation,
    ProcessExecution,
    Destructive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CommandAuthorization {
    LocalUser,
    SecretAccess,
    SystemCapability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CommandConfirmation {
    None,
    UserGesture,
    OpaqueCapability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CommandConcurrency {
    Parallel,
    ModuleExclusive,
    Singleton,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CommandAudit {
    MetadataOnly,
    Redacted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CommandSchema {
    Generated,
    LegacyJson,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CommandPlatform {
    Base,
    Windows,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CommandModule {
    pub(crate) key: &'static str,
    pub(crate) title: &'static str,
    pub(crate) commands: &'static [&'static str],
    pub(crate) wire_contracts: &'static [Option<CommandWireContract>],
    pub(crate) default_risk: CommandRisk,
    pub(crate) schema: CommandSchema,
    pub(crate) platform: CommandPlatform,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CommandWireContract {
    pub(crate) input_type: &'static str,
    pub(crate) output_type: &'static str,
    #[cfg(test)]
    pub(crate) client_declaration: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) struct CommandDescriptor {
    pub(crate) id: &'static str,
    pub(crate) handler_path: &'static str,
    pub(crate) module: &'static str,
    pub(crate) title: &'static str,
    pub(crate) platform: CommandPlatform,
    pub(crate) risk: CommandRisk,
    pub(crate) input_schema: CommandSchema,
    pub(crate) output_schema: CommandSchema,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) input_type: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) output_type: Option<&'static str>,
    pub(crate) timeout_ms: u64,
    pub(crate) concurrency: CommandConcurrency,
    pub(crate) confirmation: CommandConfirmation,
    pub(crate) authorization: CommandAuthorization,
    pub(crate) audit: CommandAudit,
}

impl CommandDescriptor {
    fn from_module(
        module: &'static CommandModule,
        command_index: usize,
        handler_path: &'static str,
    ) -> Self {
        let id = handler_path.rsplit("::").next().unwrap_or(handler_path);
        let risk = effective_risk(module.default_risk, id);
        let (timeout_ms, concurrency, confirmation, authorization, audit) =
            capability_policy(risk, module.default_risk, id);
        let wire_contract = module.wire_contracts[command_index];

        Self {
            id,
            handler_path,
            module: module.key,
            title: module.title,
            platform: module.platform,
            risk,
            input_schema: module.schema,
            output_schema: module.schema,
            input_type: wire_contract.map(|contract| contract.input_type),
            output_type: wire_contract.map(|contract| contract.output_type),
            timeout_ms,
            concurrency,
            confirmation,
            authorization,
            audit,
        }
    }

    pub(crate) fn is_typed(self) -> bool {
        self.input_schema == CommandSchema::Generated
            && self.output_schema == CommandSchema::Generated
    }

    #[cfg(test)]
    pub(crate) fn has_exact_wire_types(self) -> bool {
        self.input_type.is_some() && self.output_type.is_some()
    }
}

fn effective_risk(default_risk: CommandRisk, command: &str) -> CommandRisk {
    const DESTRUCTIVE_PREFIXES: &[&str] = &[
        "clear_", "clean_", "delete_", "remove_", "reset_", "restore_",
    ];
    const READ_PREFIXES: &[&str] = &[
        "check_",
        "detect_",
        "env_detect_",
        "export_",
        "get_",
        "health_",
        "is_",
        "list_",
        "preview_",
        "probe_",
        "read_",
        "validate_",
    ];

    if DESTRUCTIVE_PREFIXES
        .iter()
        .any(|prefix| command_has_action(command, prefix))
    {
        CommandRisk::Destructive
    } else if READ_PREFIXES
        .iter()
        .any(|prefix| command_has_action(command, prefix))
    {
        CommandRisk::ReadOnly
    } else {
        default_risk
    }
}

fn command_has_action(command: &str, prefix: &str) -> bool {
    let action = prefix.trim_end_matches('_');
    command == action
        || command.starts_with(prefix)
        || command.ends_with(&format!("_{action}"))
        || command.contains(&format!("_{action}_"))
}

fn capability_policy(
    risk: CommandRisk,
    default_risk: CommandRisk,
    command: &str,
) -> (
    u64,
    CommandConcurrency,
    CommandConfirmation,
    CommandAuthorization,
    CommandAudit,
) {
    let (timeout_ms, concurrency, confirmation, mut authorization, mut audit) = match risk {
        CommandRisk::ReadOnly => (
            15_000,
            CommandConcurrency::Parallel,
            CommandConfirmation::None,
            CommandAuthorization::LocalUser,
            CommandAudit::MetadataOnly,
        ),
        CommandRisk::LocalMutation => (
            30_000,
            CommandConcurrency::ModuleExclusive,
            CommandConfirmation::UserGesture,
            CommandAuthorization::LocalUser,
            CommandAudit::MetadataOnly,
        ),
        CommandRisk::SecretMutation => (
            30_000,
            CommandConcurrency::ModuleExclusive,
            CommandConfirmation::UserGesture,
            CommandAuthorization::SecretAccess,
            CommandAudit::Redacted,
        ),
        CommandRisk::NetworkMutation => (
            120_000,
            CommandConcurrency::ModuleExclusive,
            CommandConfirmation::UserGesture,
            CommandAuthorization::SecretAccess,
            CommandAudit::Redacted,
        ),
        CommandRisk::ProcessExecution => (
            120_000,
            CommandConcurrency::Singleton,
            if command == "llmusage_install_execute" {
                CommandConfirmation::OpaqueCapability
            } else {
                CommandConfirmation::UserGesture
            },
            CommandAuthorization::SystemCapability,
            CommandAudit::Redacted,
        ),
        CommandRisk::Destructive => (
            60_000,
            CommandConcurrency::ModuleExclusive,
            CommandConfirmation::UserGesture,
            CommandAuthorization::SecretAccess,
            CommandAudit::Redacted,
        ),
    };

    match default_risk {
        CommandRisk::SecretMutation | CommandRisk::NetworkMutation | CommandRisk::Destructive => {
            authorization = CommandAuthorization::SecretAccess;
            audit = CommandAudit::Redacted;
        }
        CommandRisk::ProcessExecution => {
            authorization = CommandAuthorization::SystemCapability;
            audit = CommandAudit::Redacted;
        }
        CommandRisk::ReadOnly | CommandRisk::LocalMutation => {}
    }

    (timeout_ms, concurrency, confirmation, authorization, audit)
}

pub(crate) fn command_descriptors() -> impl Iterator<Item = CommandDescriptor> {
    COMMAND_MODULES
        .iter()
        .chain(WINDOWS_COMMAND_MODULES)
        .flat_map(|module| {
            module
                .commands
                .iter()
                .enumerate()
                .map(|(index, command)| CommandDescriptor::from_module(module, index, command))
        })
}

pub(crate) fn command_descriptor(command: &str) -> Option<CommandDescriptor> {
    command_descriptors().find(|descriptor| descriptor.id == command)
}

macro_rules! command_wire_contract {
    () => {
        None
    };
    ($input_type:literal, $output_type:literal, $client_declaration:literal) => {
        Some(CommandWireContract {
            input_type: $input_type,
            output_type: $output_type,
            #[cfg(test)]
            client_declaration: $client_declaration,
        })
    };
}

macro_rules! define_command_registry {
    (
        $(
            $key:ident: $title:literal [$risk:ident, $schema:ident] => [
                $($command:path $(=> [$input_type:literal, $output_type:literal, $client_declaration:literal])?),* $(,)?
            ]
        ),* $(,)?
    ) => {
        pub(crate) const COMMAND_MODULES: &[CommandModule] = &[
            $(
                CommandModule {
                    key: stringify!($key),
                    title: $title,
                    commands: &[$(stringify!($command)),*],
                    wire_contracts: &[$(command_wire_contract!($($input_type, $output_type, $client_declaration)?)),*],
                    default_risk: CommandRisk::$risk,
                    schema: CommandSchema::$schema,
                    platform: CommandPlatform::Base,
                },
            )*
        ];

        #[cfg(not(target_os = "windows"))]
        pub fn generate_handler() -> impl Fn(tauri::ipc::Invoke) -> bool {
            debug_assert!(command_registry_is_well_formed());
            let handler: Box<dyn Fn(tauri::ipc::Invoke) -> bool + Send + Sync> = Box::new(tauri::generate_handler![
                $(
                    $($command,)*
                )*
            ]);
            move |invoke: tauri::ipc::Invoke| {
                if audit_invoke(&invoke) {
                    handler(invoke)
                } else {
                    invoke
                        .resolver
                        .reject("command is not registered in the capability manifest");
                    true
                }
            }
        }

        #[cfg(target_os = "windows")]
        pub fn generate_handler() -> impl Fn(tauri::ipc::Invoke) -> bool {
            debug_assert!(command_registry_is_well_formed());
            let handler: Box<dyn Fn(tauri::ipc::Invoke) -> bool + Send + Sync> = Box::new(tauri::generate_handler![
                $(
                    $($command,)*
                )*
                super::wsl::wsl_list_distros,
                super::wsl::wsl_refresh_distros,
                super::wsl::wsl_clear_cache,
                super::wsl::wsl_cache_status,
                super::wsl::wsl_read_config,
                super::wsl::wsl_write_config,
                super::wsl::wsl_detect_cli,
                super::wsl::wsl_sync_config,
            ]);
            move |invoke: tauri::ipc::Invoke| {
                if audit_invoke(&invoke) {
                    handler(invoke)
                } else {
                    invoke
                        .resolver
                        .reject("command is not registered in the capability manifest");
                    true
                }
            }
        }
    };
}

fn audit_invoke(invoke: &tauri::ipc::Invoke) -> bool {
    let command = invoke.message.command();
    if let Some(descriptor) = command_descriptor(command) {
        tracing::debug!(
            command = descriptor.id,
            module = descriptor.module,
            risk = ?descriptor.risk,
            audit = ?descriptor.audit,
            typed = descriptor.is_typed(),
            "tauri command accepted by capability manifest"
        );
        true
    } else {
        tracing::warn!(
            command,
            "tauri command rejected: missing capability metadata"
        );
        false
    }
}

define_command_registry! {
    config: "配置管理" [LocalMutation, Generated] => [
        super::config::list_configs => ["void", "ConfigInfo[]", "export const listConfigsTyped = (): Promise<ConfigInfo[]> => invoke('list_configs')\n"],
        super::config::switch_config => ["string", "string", "export const switchConfigTyped = (name: string): Promise<string> => invoke('switch_config', { name })\n"],
        super::config::add_config => ["AddConfigInput", "string", "export const addConfigTyped = (input: AddConfigInput): Promise<string> => invoke('add_config', input)\n"],
        super::config::delete_config => ["string", "string", "export const deleteConfigTyped = (name: string): Promise<string> => invoke('delete_config', { name, confirmationToken: confirmationTokenFor('delete_config') })\n"],
        super::config::rename_config => ["{ oldName: string; newName: string }", "string", "export const renameConfigTyped = (oldName: string, newName: string): Promise<string> => invoke('rename_config', { oldName, newName })\n"],
        super::config::duplicate_config => ["{ source: string; target: string }", "string", "export const duplicateConfigTyped = (source: string, target: string): Promise<string> => invoke('duplicate_config', { source, target })\n"],
        super::config::validate_configs => ["void", "string", "export const validateConfigsTyped = (): Promise<string> => invoke('validate_configs')\n"],
        super::config::import_config => ["ImportConfigInput", "ImportResult", "export const importConfigTyped = (input: ImportConfigInput): Promise<ImportResult> => invoke('import_config', { content: input.content, mode: input.mode ?? 'merge', backup: input.backup ?? true, confirmationToken: confirmationTokenFor('import_config') })\n"],
        super::config::restore_config => ["string", "string", "export const restoreConfigTyped = (backupPath: string): Promise<string> => invoke('restore_config', { backupPath, confirmationToken: confirmationTokenFor('restore_config') })\n"],
        super::config::export_config => ["boolean | undefined", "ExportResult", "export const exportConfigTyped = (includeSecrets = false): Promise<ExportResult> => invoke('export_config', { includeSecrets })\n"],
        super::config::get_history => ["number | undefined", "HistoryEntry[]", "export const getHistoryTyped = (limit = 100): Promise<HistoryEntry[]> => invoke('get_history', { limit })\n"],
        super::config::clear_history => ["void", "string", "export const clearHistoryTyped = (): Promise<string> => invoke('clear_history')\n"],
    ],
    settings_raw: "配置源文件" [SecretMutation, LegacyJson] => [
        super::settings_raw::claude_get_settings_raw_text,
        super::settings_raw::claude_save_settings_raw_text,
        super::settings_raw::codex_get_config_raw_text,
        super::settings_raw::codex_save_config_raw_text,
        super::settings_raw::claude_list_settings_layers,
        super::settings_raw::codex_list_config_layers,
    ],
    system_prompts: "系统提示词" [LocalMutation, Generated] => [
        super::system_prompts::system_prompts_list => ["string", "OpenJsonValueDto", "export const listSystemPrompts = (platform: string): Promise<OpenJsonValueDto> => invoke('system_prompts_list', { platform })\n"],
        super::system_prompts::system_prompts_get => ["{ platform: string; id: string }", "OpenJsonValueDto", "export const getSystemPrompt = (platform: string, id: string): Promise<OpenJsonValueDto> => invoke('system_prompts_get', { platform, id })\n"],
        super::system_prompts::system_prompts_save => ["{ platform: string; id: string; content: string; token: string }", "OpenJsonValueDto", "export const saveSystemPrompt = (platform: string, id: string, content: string, token: string): Promise<OpenJsonValueDto> => invoke('system_prompts_save', { platform, id, content, token })\n"],
        super::system_prompts::system_prompts_create => ["{ platform: string; id: string }", "OpenJsonValueDto", "export const createSystemPrompt = (platform: string, id: string): Promise<OpenJsonValueDto> => invoke('system_prompts_create', { platform, id })\n"],
    ],
    sync: "同步" [NetworkMutation, Generated] => [
        super::sync::sync_push => ["boolean | undefined", "SyncOperationResult", "export const syncPush = (force?: boolean): Promise<SyncOperationResult> => invoke('sync_push', { force })\n"],
        super::sync::sync_pull => ["boolean | undefined", "SyncOperationResult", "export const syncPull = (force?: boolean): Promise<SyncOperationResult> => invoke('sync_pull', { force })\n"],
        super::sync::list_sync_assets => ["void", "SyncAssetInfo[]", "export const listSyncAssets = (): Promise<SyncAssetInfo[]> => invoke('list_sync_assets')\n"],
        super::sync::sync_push_asset => ["SyncAssetOperationInput", "SyncOperationResult", "export const syncPushAsset = (payload: SyncAssetOperationInput): Promise<SyncOperationResult> => invoke('sync_push_asset', { payload })\n"],
        super::sync::sync_pull_asset => ["SyncAssetOperationInput", "SyncOperationResult", "export const syncPullAsset = (payload: SyncAssetOperationInput): Promise<SyncOperationResult> => invoke('sync_pull_asset', { payload })\n"],
        super::sync::sync_asset => ["SyncAssetOperationInput", "SyncOperationResult", "export const syncAsset = (payload: SyncAssetOperationInput): Promise<SyncOperationResult> => invoke('sync_asset', { payload })\n"],
        super::sync::sync_all_assets => ["SyncAllAssetsInput", "SyncOperationResult", "export const syncAllAssets = (payload: SyncAllAssetsInput = {}): Promise<SyncOperationResult> => invoke('sync_all_assets', { payload })\n"],
        super::sync::sync_push_folder => ["{ id: string; force?: boolean }", "SyncOperationResult", "export const syncPushFolder = (id: string, force?: boolean): Promise<SyncOperationResult> => invoke('sync_push_folder', { id, force })\n"],
        super::sync::sync_pull_folder => ["{ id: string; force?: boolean }", "SyncOperationResult", "export const syncPullFolder = (id: string, force?: boolean): Promise<SyncOperationResult> => invoke('sync_pull_folder', { id, force })\n"],
        super::sync::sync_status => ["void", "SyncStatusInfo", "export const syncStatus = (): Promise<SyncStatusInfo> => invoke('sync_status')\n"],
        super::sync::list_sync_folders => ["void", "SyncFolderInfo[]", "export const listSyncFolders = (): Promise<SyncFolderInfo[]> => invoke('list_sync_folders')\n"],
        super::sync::add_sync_folder => ["AddSyncFolderInput", "SyncFolderInfo", "export const addSyncFolder = (input: AddSyncFolderInput): Promise<SyncFolderInfo> => invoke('add_sync_folder', input)\n"],
        super::sync::update_sync_folder => ["UpdateSyncFolderInput", "SyncFolderInfo", "export const updateSyncFolder = (input: UpdateSyncFolderInput): Promise<SyncFolderInfo> => invoke('update_sync_folder', input)\n"],
        super::sync::delete_sync_folder => ["string", "SyncOperationResult", "export const deleteSyncFolder = (id: string): Promise<SyncOperationResult> => invoke('delete_sync_folder', { id })\n"],
        super::sync::set_webdav_config => ["WebDavConfigInput", "WebDavConfigDetails", "export const setWebdavConfig = (payload: WebDavConfigInput): Promise<WebDavConfigDetails> => invoke('set_webdav_config', { payload })\n"],
        super::sync::test_webdav_config => ["WebDavConfigInput", "WebDavTestResult", "export const testWebdavConfig = (payload: WebDavConfigInput): Promise<WebDavTestResult> => invoke('test_webdav_config', { payload })\n"],
        super::sync::clear_webdav_config => ["void", "void", "export const clearWebdavConfig = (): Promise<void> => invoke('clear_webdav_config')\n"],
    ],
    claude: "Claude Code" [SecretMutation, Generated] => [
        super::claude::claude_get_settings => ["void", "OpenJsonValueDto", "export const getClaudeSettings = (): Promise<OpenJsonValueDto> => invoke('claude_get_settings')\n"],
        super::claude::claude_update_settings => ["OpenJsonValueDto", "OpenJsonValueDto", "export const updateClaudeSettings = (settings: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('claude_update_settings', { settings })\n"],
        super::claude::claude_list_mcp_servers => ["void", "OpenJsonValueDto", "export const listClaudeMcpServers = (): Promise<OpenJsonValueDto> => invoke('claude_list_mcp_servers')\n"],
        super::claude::claude_add_mcp_server => ["{ name: string; config: OpenJsonValueDto; scope?: string }", "OpenJsonValueDto", "export const addClaudeMcpServer = (name: string, config: OpenJsonValueDto, scope?: string): Promise<OpenJsonValueDto> => invoke('claude_add_mcp_server', { name, config, scope })\n"],
        super::claude::claude_update_mcp_server => ["{ name: string; config: OpenJsonValueDto; scope?: string }", "OpenJsonValueDto", "export const updateClaudeMcpServer = (name: string, config: OpenJsonValueDto, scope?: string): Promise<OpenJsonValueDto> => invoke('claude_update_mcp_server', { name, config, scope })\n"],
        super::claude::claude_delete_mcp_server => ["{ name: string; scope?: string }", "string", "export const deleteClaudeMcpServer = (name: string, scope?: string): Promise<string> => invoke('claude_delete_mcp_server', { name, scope })\n"],
        super::claude::claude_list_agents => ["void", "OpenJsonValueDto", "export const listClaudeAgents = (): Promise<OpenJsonValueDto> => invoke('claude_list_agents')\n"],
        super::claude::claude_add_agent => ["{ name: string; config: OpenJsonValueDto }", "OpenJsonValueDto", "export const addClaudeAgent = (name: string, config: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('claude_add_agent', { name, config })\n"],
        super::claude::claude_update_agent => ["{ name: string; config: OpenJsonValueDto }", "OpenJsonValueDto", "export const updateClaudeAgent = (name: string, config: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('claude_update_agent', { name, config })\n"],
        super::claude::claude_delete_agent => ["string", "string", "export const deleteClaudeAgent = (name: string): Promise<string> => invoke('claude_delete_agent', { name })\n"],
        super::claude::claude_list_slash_commands => ["void", "OpenJsonValueDto", "export const listClaudeSlashCommands = (): Promise<OpenJsonValueDto> => invoke('claude_list_slash_commands')\n"],
        super::claude::claude_add_slash_command => ["{ name: string; config: OpenJsonValueDto }", "OpenJsonValueDto", "export const addClaudeSlashCommand = (name: string, config: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('claude_add_slash_command', { name, config })\n"],
        super::claude::claude_update_slash_command => ["{ name: string; config: OpenJsonValueDto }", "OpenJsonValueDto", "export const updateClaudeSlashCommand = (name: string, config: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('claude_update_slash_command', { name, config })\n"],
        super::claude::claude_delete_slash_command => ["string", "string", "export const deleteClaudeSlashCommand = (name: string): Promise<string> => invoke('claude_delete_slash_command', { name })\n"],
        super::claude::claude_list_plugins => ["void", "OpenJsonValueDto", "export const listClaudePlugins = (): Promise<OpenJsonValueDto> => invoke('claude_list_plugins')\n"],
        super::claude::claude_add_plugin => ["{ name: string; config: OpenJsonValueDto }", "OpenJsonValueDto", "export const addClaudePlugin = (name: string, config: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('claude_add_plugin', { name, config })\n"],
        super::claude::claude_update_plugin => ["{ name: string; config: OpenJsonValueDto }", "OpenJsonValueDto", "export const updateClaudePlugin = (name: string, config: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('claude_update_plugin', { name, config })\n"],
        super::claude::claude_delete_plugin => ["string", "string", "export const deleteClaudePlugin = (name: string): Promise<string> => invoke('claude_delete_plugin', { name })\n"],
        super::claude::claude_get_output_styles => ["void", "OpenJsonValueDto", "export const getClaudeOutputStyles = (): Promise<OpenJsonValueDto> => invoke('claude_get_output_styles')\n"],
        super::claude::claude_update_output_styles => ["OpenJsonValueDto", "OpenJsonValueDto", "export const updateClaudeOutputStyles = (styles: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('claude_update_output_styles', { styles })\n"],
        super::claude::claude_get_statusline => ["void", "OpenJsonValueDto", "export const getClaudeStatusline = (): Promise<OpenJsonValueDto> => invoke('claude_get_statusline')\n"],
        super::claude::claude_update_statusline => ["OpenJsonValueDto", "OpenJsonValueDto", "export const updateClaudeStatusline = (config: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('claude_update_statusline', { config })\n"],
        super::claude::claude_list_hooks => ["void", "OpenJsonValueDto", "export const listClaudeHooks = (): Promise<OpenJsonValueDto> => invoke('claude_list_hooks')\n"],
        super::claude::claude_update_hooks => ["OpenJsonValueDto", "OpenJsonValueDto", "export const updateClaudeHooks = (hooks: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('claude_update_hooks', { hooks })\n"],
        super::claude::claude_get_budgets => ["void", "OpenJsonValueDto", "export const getClaudeBudgets = (): Promise<OpenJsonValueDto> => invoke('claude_get_budgets')\n"],
        super::claude::claude_update_budgets => ["OpenJsonValueDto", "OpenJsonValueDto", "export const updateClaudeBudgets = (budgets: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('claude_update_budgets', { budgets })\n"],
        super::claude::claude_list_prompts => ["void", "OpenJsonValueDto", "export const listClaudePrompts = (): Promise<OpenJsonValueDto> => invoke('claude_list_prompts')\n"],
        super::claude::claude_update_prompts => ["OpenJsonValueDto", "OpenJsonValueDto", "export const updateClaudePrompts = (prompts: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('claude_update_prompts', { prompts })\n"],
    ],
    claude_profiles: "Claude Code Profiles" [SecretMutation, Generated] => [
        super::claude::claude_list_profiles => ["void", "OpenJsonValueDto", "export const listClaudeProfiles = (): Promise<OpenJsonValueDto> => invoke('claude_list_profiles')\n"],
        super::claude::claude_get_profile => ["string", "OpenJsonValueDto", "export const getClaudeProfile = (name: string): Promise<OpenJsonValueDto> => invoke('claude_get_profile', { name })\n"],
        super::claude::claude_add_profile => ["OpenJsonValueDto", "OpenJsonValueDto", "export const addClaudeProfile = (request: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('claude_add_profile', { request })\n"],
        super::claude::claude_update_profile => ["{ name: string; request: OpenJsonValueDto }", "OpenJsonValueDto", "export const updateClaudeProfile = (name: string, request: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('claude_update_profile', { name, request })\n"],
        super::claude::claude_delete_profile => ["string", "OpenJsonValueDto", "export const deleteClaudeProfile = (name: string): Promise<OpenJsonValueDto> => invoke('claude_delete_profile', { name })\n"],
        super::claude::claude_apply_profile => ["string", "OpenJsonValueDto", "export const applyClaudeProfile = (name: string): Promise<OpenJsonValueDto> => invoke('claude_apply_profile', { name })\n"],
        super::claude::claude_export_profiles => ["boolean", "OpenJsonValueDto", "export const exportClaudeProfiles = (includeSecrets: boolean): Promise<OpenJsonValueDto> => invoke('claude_export_profiles', { includeSecrets })\n"],
        super::claude::claude_get_profiles_raw => ["void", "OpenJsonValueDto", "export const getClaudeProfilesRaw = (): Promise<OpenJsonValueDto> => invoke('claude_get_profiles_raw')\n"],
        super::claude::claude_save_profiles_raw => ["{ content: string; token: string; force: boolean }", "OpenJsonValueDto", "export const saveClaudeProfilesRaw = (content: string, token: string, force: boolean): Promise<OpenJsonValueDto> => invoke('claude_save_profiles_raw', { content, token, force })\n"],
    ],
    claude_auth: "Claude Auth" [SecretMutation, Generated] => [
        super::claude::claude_list_auth_accounts => ["void", "ClaudeAuthListResponse", "export const listClaudeAuthAccounts = (): Promise<ClaudeAuthListResponse> =>\n  invoke('claude_list_auth_accounts')\n\n"],
        super::claude::claude_get_auth_current => ["void", "ClaudeAuthCurrentResponse", "export const getClaudeAuthCurrent = (): Promise<ClaudeAuthCurrentResponse> =>\n  invoke('claude_get_auth_current')\n\n"],
        super::claude::claude_save_auth => ["ClaudeAuthSaveRequest", "ClaudeAuthActionResponse", "export const saveClaudeAuth = (request: ClaudeAuthSaveRequest): Promise<ClaudeAuthActionResponse> =>\n  invoke('claude_save_auth', {\n    name: request.name,\n    description: request.description ?? null,\n    force: request.force ?? false,\n  })\n\n"],
        super::claude::claude_switch_auth => ["string", "ClaudeAuthActionResponse", "export const switchClaudeAuth = (name: string): Promise<ClaudeAuthActionResponse> =>\n  invoke('claude_switch_auth', { name })\n\n"],
        super::claude::claude_delete_auth => ["string", "ClaudeAuthActionResponse", "export const deleteClaudeAuth = (name: string): Promise<ClaudeAuthActionResponse> =>\n  invoke('claude_delete_auth', { name })\n"],
    ],
    codex: "Codex" [SecretMutation, Generated] => [
        super::codex::codex_list_profiles => ["void", "OpenJsonValueDto", "export const listCodexProfiles = (): Promise<OpenJsonValueDto> => invoke('codex_list_profiles')\n"],
        super::codex::codex_list_models => ["void", "OpenJsonValueDto", "export const listCodexModels = (): Promise<OpenJsonValueDto> => invoke('codex_list_models')\n"],
        super::codex::codex_add_profile => ["{ name: string; config: OpenJsonValueDto }", "OpenJsonValueDto", "export const addCodexProfile = (name: string, config: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('codex_add_profile', { name, config })\n"],
        super::codex::codex_update_profile => ["{ name: string; config: OpenJsonValueDto }", "OpenJsonValueDto", "export const updateCodexProfile = (name: string, config: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('codex_update_profile', { name, config })\n"],
        super::codex::codex_delete_profile => ["string", "OpenJsonValueDto", "export const deleteCodexProfile = (name: string): Promise<OpenJsonValueDto> => invoke('codex_delete_profile', { name })\n"],
        super::codex::codex_get_profile_env => ["string", "OpenJsonValueDto", "export const getCodexProfileEnv = (name: string): Promise<OpenJsonValueDto> => invoke('codex_get_profile_env', { name })\n"],
        super::codex::codex_apply_profile => ["string", "OpenJsonValueDto", "export const applyCodexProfile = (name: string): Promise<OpenJsonValueDto> => invoke('codex_apply_profile', { name })\n"],
        super::codex::codex_export_profiles => ["boolean", "OpenJsonValueDto", "export const exportCodexProfiles = (includeSecrets: boolean): Promise<OpenJsonValueDto> => invoke('codex_export_profiles', { includeSecrets })\n"],
        super::codex::codex_get_profiles_raw => ["void", "OpenJsonValueDto", "export const getCodexProfilesRaw = (): Promise<OpenJsonValueDto> => invoke('codex_get_profiles_raw')\n"],
        super::codex::codex_save_profiles_raw => ["{ content: string; token: string; force: boolean }", "OpenJsonValueDto", "export const saveCodexProfilesRaw = (content: string, token: string, force: boolean): Promise<OpenJsonValueDto> => invoke('codex_save_profiles_raw', { content, token, force })\n"],
        super::codex::codex_get_settings => ["void", "OpenJsonValueDto", "export const getCodexSettings = (): Promise<OpenJsonValueDto> => invoke('codex_get_settings')\n"],
        super::codex::codex_update_settings => ["OpenJsonValueDto", "OpenJsonValueDto", "export const updateCodexSettings = (settings: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('codex_update_settings', { settings })\n"],
        super::codex::codex_list_mcp_servers => ["void", "OpenJsonValueDto", "export const listCodexMcpServers = (): Promise<OpenJsonValueDto> => invoke('codex_list_mcp_servers')\n"],
        super::codex::codex_add_mcp_server => ["{ name: string; config: OpenJsonValueDto }", "OpenJsonValueDto", "export const addCodexMcpServer = (name: string, config: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('codex_add_mcp_server', { name, config })\n"],
        super::codex::codex_update_mcp_server => ["{ name: string; config: OpenJsonValueDto }", "OpenJsonValueDto", "export const updateCodexMcpServer = (name: string, config: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('codex_update_mcp_server', { name, config })\n"],
        super::codex::codex_delete_mcp_server => ["string", "string", "export const deleteCodexMcpServer = (name: string): Promise<string> => invoke('codex_delete_mcp_server', { name })\n"],
        super::codex::codex_list_agents => ["CodexAgentContextRequest | undefined", "OpenJsonValueDto", "export const listCodexAgents = (context?: CodexAgentContextRequest): Promise<OpenJsonValueDto> => invoke('codex_list_agents', { context })\n"],
        super::codex::codex_add_agent => ["{ name: string; config: OpenJsonValueDto; context?: CodexAgentContextRequest }", "OpenJsonValueDto", "export const addCodexAgent = (name: string, config: OpenJsonValueDto, context?: CodexAgentContextRequest): Promise<OpenJsonValueDto> => invoke('codex_add_agent', { name, config, context })\n"],
        super::codex::codex_update_agent => ["{ name: string; config: OpenJsonValueDto; context?: CodexAgentContextRequest }", "OpenJsonValueDto", "export const updateCodexAgent = (name: string, config: OpenJsonValueDto, context?: CodexAgentContextRequest): Promise<OpenJsonValueDto> => invoke('codex_update_agent', { name, config, context })\n"],
        super::codex::codex_delete_agent => ["{ name: string; context?: CodexAgentContextRequest }", "string", "export const deleteCodexAgent = (name: string, context?: CodexAgentContextRequest): Promise<string> => invoke('codex_delete_agent', { name, context })\n"],
        super::codex::codex_rename_agent => ["{ name: string; newName: string; context?: CodexAgentContextRequest }", "OpenJsonValueDto", "export const renameCodexAgent = (name: string, newName: string, context?: CodexAgentContextRequest): Promise<OpenJsonValueDto> => invoke('codex_rename_agent', { name, newName, context })\n"],
        super::codex::codex_copy_agent => ["{ name: string; sourceContext?: CodexAgentContextRequest; targetContext?: CodexAgentContextRequest; targetName?: string }", "OpenJsonValueDto", "export const copyCodexAgent = (name: string, sourceContext?: CodexAgentContextRequest, targetContext?: CodexAgentContextRequest, targetName?: string): Promise<OpenJsonValueDto> => invoke('codex_copy_agent', { name, sourceContext, targetContext, targetName })\n"],
        super::codex::codex_validate_agent_toml => ["{ name: string; context?: CodexAgentContextRequest }", "OpenJsonValueDto", "export const validateCodexAgentToml = (name: string, context?: CodexAgentContextRequest): Promise<OpenJsonValueDto> => invoke('codex_validate_agent_toml', { name, context })\n"],
        super::codex::codex_list_agent_sources => ["void", "OpenJsonValueDto", "export const listCodexAgentSources = (): Promise<OpenJsonValueDto> => invoke('codex_list_agent_sources')\n"],
        super::codex::codex_add_agent_source => ["string", "OpenJsonValueDto", "export const addCodexAgentSource = (url: string): Promise<OpenJsonValueDto> => invoke('codex_add_agent_source', { request: { url } })\n"],
        super::codex::codex_remove_agent_source => ["string", "void", "export const removeCodexAgentSource = (sourceId: string): Promise<void> => invoke('codex_remove_agent_source', { sourceId })\n"],
        super::codex::codex_sync_agent_source => ["string", "OpenJsonValueDto", "export const syncCodexAgentSource = (sourceId: string): Promise<OpenJsonValueDto> => invoke('codex_sync_agent_source', { sourceId })\n"],
        super::codex::codex_get_agent_source_catalog => ["string", "OpenJsonValueDto", "export const getCodexAgentSourceCatalog = (sourceId: string): Promise<OpenJsonValueDto> => invoke('codex_get_agent_source_catalog', { sourceId })\n"],
        super::codex::codex_install_source_agent => ["CodexAgentSourceInstallRequest", "OpenJsonValueDto", "export const installCodexSourceAgent = (request: CodexAgentSourceInstallRequest): Promise<OpenJsonValueDto> => invoke('codex_install_source_agent', { request })\n"],
        super::codex::codex_sync_source_install => ["CodexAgentSourceSyncRequest", "OpenJsonValueDto", "export const syncCodexSourceInstall = (request: CodexAgentSourceSyncRequest): Promise<OpenJsonValueDto> => invoke('codex_sync_source_install', { request })\n"],
        super::codex::codex_accept_local_source_install => ["string", "OpenJsonValueDto", "export const acceptLocalCodexSourceInstall = (installId: string): Promise<OpenJsonValueDto> => invoke('codex_accept_local_source_install', { request: { installId } })\n"],
        super::codex::codex_untrack_source_install => ["string", "OpenJsonValueDto", "export const untrackCodexSourceInstall = (installId: string): Promise<OpenJsonValueDto> => invoke('codex_untrack_source_install', { request: { installId } })\n"],
        super::codex::codex_list_sessions => ["{ limit?: number; query?: string }", "OpenJsonValueDto", "export const listCodexSessions = (limit?: number, query?: string): Promise<OpenJsonValueDto> => invoke('codex_list_sessions', { limit, query })\n"],
        super::codex::codex_get_session_detail => ["{ filePath: string; messageLimit?: number }", "OpenJsonValueDto", "export const getCodexSessionDetail = (filePath: string, messageLimit?: number): Promise<OpenJsonValueDto> => invoke('codex_get_session_detail', { filePath, messageLimit })\n"],
        super::codex::codex_export_session => ["{ filePath: string; maxMessages?: number }", "OpenJsonValueDto", "export const exportCodexSession = (filePath: string, maxMessages?: number): Promise<OpenJsonValueDto> => invoke('codex_export_session', { filePath, maxMessages })\n"],
        super::codex::codex_clone_session => ["string", "OpenJsonValueDto", "export const cloneCodexSession = (filePath: string): Promise<OpenJsonValueDto> => invoke('codex_clone_session', { filePath })\n"],
        super::codex::codex_delete_session => ["string", "OpenJsonValueDto", "export const deleteCodexSession = (filePath: string): Promise<OpenJsonValueDto> => invoke('codex_delete_session', { filePath })\n"],
        super::codex::codex_get_usage => ["boolean | undefined", "OpenJsonValueDto", "export const getCodexUsage = (force?: boolean): Promise<OpenJsonValueDto> => invoke('codex_get_usage', { force })\n"],
        super::codex::codex_get_dashboard_overview => ["boolean | undefined", "OpenJsonValueDto", "export const getCodexDashboardOverview = (force?: boolean): Promise<OpenJsonValueDto> => invoke('codex_get_dashboard_overview', { force })\n"],
        super::codex::codex_get_dashboard_usage_summary => ["boolean | undefined", "OpenJsonValueDto", "export const getCodexDashboardUsageSummary = (force?: boolean): Promise<OpenJsonValueDto> => invoke('codex_get_dashboard_usage_summary', { force })\n"],
        super::codex::codex_get_tray_snapshot => ["boolean | undefined", "OpenJsonValueDto", "export const getCodexTraySnapshot = (force?: boolean): Promise<OpenJsonValueDto> => invoke('codex_get_tray_snapshot', { force })\n"],
        super::codex::codex_get_all_quotas => ["void", "OpenJsonValueDto", "export const getCodexAllQuotas = (): Promise<OpenJsonValueDto> => invoke('codex_get_all_quotas')\n"],
        super::codex::codex_get_quota => ["string", "OpenJsonValueDto", "export const getCodexQuota = (account: string): Promise<OpenJsonValueDto> => invoke('codex_get_quota', { account })\n"],
    ],
    codex_auth: "Codex Auth" [SecretMutation, Generated] => [
        super::codex::codex_list_auth_accounts => ["void", "CodexAuthListResponse", "export const listCodexAuthAccounts = (): Promise<CodexAuthListResponse> => invoke('codex_list_auth_accounts')\n"],
        super::codex::codex_get_auth_current => ["void", "CodexAuthCurrentResponse", "export const getCodexAuthCurrent = (): Promise<CodexAuthCurrentResponse> => invoke('codex_get_auth_current')\n"],
        super::codex::codex_save_auth => ["CodexAuthSaveRequest", "CodexAuthActionResponse", "export const saveCodexAuth = (request: CodexAuthSaveRequest): Promise<CodexAuthActionResponse> =>\n  invoke('codex_save_auth', { name: request.name, description: request.description ?? null, force: request.force ?? false })\n"],
        super::codex::codex_switch_auth => ["string", "CodexAuthActionResponse", "export const switchCodexAuth = (name: string): Promise<CodexAuthActionResponse> => invoke('codex_switch_auth', { name })\n"],
        super::codex::codex_delete_auth => ["string", "CodexAuthActionResponse", "export const deleteCodexAuth = (name: string): Promise<CodexAuthActionResponse> => invoke('codex_delete_auth', { name })\n"],
        super::codex::codex_rename_auth => ["{ oldName: string; newName: string; force?: boolean }", "CodexAuthRenameResponse", "export const renameCodexAuth = (oldName: string, newName: string, force = false): Promise<CodexAuthRenameResponse> =>\n  invoke('codex_rename_auth', { oldName, newName, force })\n"],
        super::codex::codex_detect_process => ["void", "CodexAuthProcessResponse", "export const detectCodexProcess = (): Promise<CodexAuthProcessResponse> => invoke('codex_detect_process')\n"],
        super::codex::codex_oauth_login_start => ["void", "CodexOAuthStartResponse", "export const codexOAuthLoginStart = (): Promise<CodexOAuthStartResponse> => invoke('codex_oauth_login_start')\n"],
        super::codex::codex_oauth_login_completed => ["{ loginId: string; preferredAccountName?: string | null }", "CodexAuthMutationResponse", "export const codexOAuthLoginCompleted = (loginId: string, preferredAccountName?: string | null): Promise<CodexAuthMutationResponse> =>\n  invoke('codex_oauth_login_completed', { loginId, preferredAccountName: preferredAccountName ?? null })\n"],
        super::codex::codex_oauth_login_cancel => ["string | null | undefined", "void", "export const codexOAuthLoginCancel = (loginId?: string | null): Promise<void> =>\n  invoke('codex_oauth_login_cancel', { loginId: loginId ?? null })\n"],
        super::codex::codex_oauth_submit_callback_url => ["{ loginId: string; callbackUrl: string }", "void", "export const codexOAuthSubmitCallbackUrl = (loginId: string, callbackUrl: string): Promise<void> =>\n  invoke('codex_oauth_submit_callback_url', { loginId, callbackUrl })\n"],
        super::codex::codex_is_oauth_port_in_use => ["void", "boolean", "export const codexIsOAuthPortInUse = (): Promise<boolean> => invoke('codex_is_oauth_port_in_use')\n"],
        super::codex::codex_release_oauth_port => ["void", "OAuthPortReleaseReport", "export const codexReleaseOAuthPort = (): Promise<OAuthPortReleaseReport> => invoke('codex_release_oauth_port')\n"],
        super::codex::codex_open_external_url => ["string", "void", "export const codexOpenExternalUrl = (url: string): Promise<void> => invoke('codex_open_external_url', { url })\n"],
        super::codex::codex_import_auth_payload => ["CodexAuthImportPayload", "CodexAuthMutationResponse", "export const codexImportAuthPayload = (payload: CodexAuthImportPayload): Promise<CodexAuthMutationResponse> =>\n  invoke('codex_import_auth_payload', { payload })\n"],
        super::codex::codex_import_auth_from_local => ["string | null | undefined", "CodexAuthMutationResponse", "export const codexImportAuthFromLocal = (preferredAccountName?: string | null): Promise<CodexAuthMutationResponse> =>\n  invoke('codex_import_auth_from_local', { preferredAccountName: preferredAccountName ?? null })\n"],
        super::codex::codex_add_auth_with_api_key => ["CodexApiKeyAddPayload", "CodexAuthMutationResponse", "export const codexAddAuthWithApiKey = (payload: CodexApiKeyAddPayload): Promise<CodexAuthMutationResponse> =>\n  invoke('codex_add_auth_with_api_key', { payload })\n"],
    ],
    codex_model_providers: "Codex Model Providers" [SecretMutation, Generated] => [
        super::codex::codex_list_model_providers => ["void", "CodexModelProvidersResponse", "export const codexListModelProviders = (): Promise<CodexModelProvidersResponse> => invoke('codex_list_model_providers')\n"],
        super::codex::codex_save_model_provider => ["CodexModelProviderUpsertPayload", "CodexModelProviderSaveResponse", "export const codexSaveModelProvider = (payload: CodexModelProviderUpsertPayload): Promise<CodexModelProviderSaveResponse> =>\n  invoke('codex_save_model_provider', { payload })\n"],
        super::codex::codex_delete_model_provider => ["string", "CodexModelProviderDeleteResponse", "export const codexDeleteModelProvider = (providerId: string): Promise<CodexModelProviderDeleteResponse> =>\n  invoke('codex_delete_model_provider', { providerId })\n"],
    ],
    gemini: "Gemini" [SecretMutation, Generated] => [
        super::gemini::gemini_get_settings => ["void", "OpenJsonValueDto", "export const getGeminiSettings = (): Promise<OpenJsonValueDto> => invoke('gemini_get_settings')\n"],
        super::gemini::gemini_update_settings => ["OpenJsonValueDto", "OpenJsonValueDto", "export const updateGeminiSettings = (settings: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('gemini_update_settings', { settings })\n"],
        super::gemini::gemini_list_mcp_servers => ["void", "OpenJsonValueDto", "export const listGeminiMcpServers = (): Promise<OpenJsonValueDto> => invoke('gemini_list_mcp_servers')\n"],
        super::gemini::gemini_add_mcp_server => ["{ name: string; config: OpenJsonValueDto }", "OpenJsonValueDto", "export const addGeminiMcpServer = (name: string, config: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('gemini_add_mcp_server', { name, config })\n"],
        super::gemini::gemini_update_mcp_server => ["{ name: string; config: OpenJsonValueDto }", "OpenJsonValueDto", "export const updateGeminiMcpServer = (name: string, config: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('gemini_update_mcp_server', { name, config })\n"],
        super::gemini::gemini_delete_mcp_server => ["string", "string", "export const deleteGeminiMcpServer = (name: string): Promise<string> => invoke('gemini_delete_mcp_server', { name })\n"],
        super::gemini::gemini_list_slash_commands => ["void", "OpenJsonValueDto", "export const listGeminiSlashCommands = (): Promise<OpenJsonValueDto> => invoke('gemini_list_slash_commands')\n"],
        super::gemini::gemini_add_slash_command => ["{ name: string; config: OpenJsonValueDto }", "OpenJsonValueDto", "export const addGeminiSlashCommand = (name: string, config: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('gemini_add_slash_command', { name, config })\n"],
        super::gemini::gemini_update_slash_command => ["{ name: string; config: OpenJsonValueDto }", "OpenJsonValueDto", "export const updateGeminiSlashCommand = (name: string, config: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('gemini_update_slash_command', { name, config })\n"],
        super::gemini::gemini_delete_slash_command => ["string", "string", "export const deleteGeminiSlashCommand = (name: string): Promise<string> => invoke('gemini_delete_slash_command', { name })\n"],
        super::gemini::gemini_list_extensions => ["void", "OpenJsonValueDto", "export const listGeminiExtensions = (): Promise<OpenJsonValueDto> => invoke('gemini_list_extensions')\n"],
    ],
    opencode: "OpenCode" [SecretMutation, Generated] => [
        super::opencode::opencode_get_settings => ["void", "OpenJsonValueDto", "export const getOpenCodeSettings = (): Promise<OpenJsonValueDto> => invoke('opencode_get_settings')\n"],
        super::opencode::opencode_update_settings => ["OpenJsonValueDto", "OpenJsonValueDto", "export const updateOpenCodeSettings = (settings: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('opencode_update_settings', { settings })\n"],
        super::opencode::opencode_get_tui_settings => ["void", "OpenJsonValueDto", "export const getOpenCodeTuiSettings = (): Promise<OpenJsonValueDto> => invoke('opencode_get_tui_settings')\n"],
        super::opencode::opencode_update_tui_settings => ["OpenJsonValueDto", "OpenJsonValueDto", "export const updateOpenCodeTuiSettings = (settings: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('opencode_update_tui_settings', { settings })\n"],
        super::opencode::opencode_get_keybindings => ["void", "OpenJsonValueDto", "export const getOpenCodeKeybindings = (): Promise<OpenJsonValueDto> => invoke('opencode_get_keybindings')\n"],
        super::opencode::opencode_update_keybindings => ["OpenJsonValueDto", "OpenJsonValueDto", "export const updateOpenCodeKeybindings = (keybindings: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('opencode_update_keybindings', { keybindings })\n"],
        super::opencode::opencode_list_themes => ["void", "OpenCodeThemeRecord[]", "export const listOpenCodeThemes = (): Promise<OpenCodeThemeRecord[]> => invoke('opencode_list_themes')\n"],
        super::opencode::opencode_list_agents => ["void", "OpenJsonValueDto", "export const listOpenCodeAgents = (): Promise<OpenJsonValueDto> => invoke('opencode_list_agents')\n"],
        super::opencode::opencode_add_agent => ["OpenJsonValueDto", "OpenJsonValueDto", "export const addOpenCodeAgent = (config: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('opencode_add_agent', { config })\n"],
        super::opencode::opencode_update_agent => ["OpenJsonValueDto", "OpenJsonValueDto", "export const updateOpenCodeAgent = (config: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('opencode_update_agent', { config })\n"],
        super::opencode::opencode_delete_agent => ["{ name: string; context?: OpenJsonValueDto }", "string", "export const deleteOpenCodeAgent = (name: string, context?: OpenJsonValueDto): Promise<string> => invoke('opencode_delete_agent', { name, context })\n"],
        super::opencode::opencode_list_commands => ["void", "OpenJsonValueDto", "export const listOpenCodeCommands = (): Promise<OpenJsonValueDto> => invoke('opencode_list_commands')\n"],
        super::opencode::opencode_add_command => ["OpenJsonValueDto", "OpenJsonValueDto", "export const addOpenCodeCommand = (config: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('opencode_add_command', { config })\n"],
        super::opencode::opencode_update_command => ["OpenJsonValueDto", "OpenJsonValueDto", "export const updateOpenCodeCommand = (config: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('opencode_update_command', { config })\n"],
        super::opencode::opencode_delete_command => ["{ name: string; context?: OpenJsonValueDto }", "string", "export const deleteOpenCodeCommand = (name: string, context?: OpenJsonValueDto): Promise<string> => invoke('opencode_delete_command', { name, context })\n"],
        super::opencode::opencode_list_local_plugins => ["void", "OpenCodePluginFileRecord[]", "export const listOpenCodeLocalPlugins = (): Promise<OpenCodePluginFileRecord[]> => invoke('opencode_list_local_plugins')\n"],
    ],
    checkin: "CheckIn" [NetworkMutation, LegacyJson] => [
        super::checkin::list_providers,
        super::checkin::add_provider,
        super::checkin::update_provider,
        super::checkin::delete_provider,
        super::checkin::test_provider_connection,
        super::checkin::list_accounts,
        super::checkin::add_account,
        super::checkin::update_account,
        super::checkin::delete_account,
        super::checkin::batch_delete_accounts,
        super::checkin::execute_checkin,
        super::checkin::batch_checkin,
        super::checkin::start_checkin_job,
        super::checkin::get_checkin_job_status,
        super::checkin::get_checkin_records,
        super::checkin::get_balance,
        super::checkin::get_balance_history,
        super::checkin::get_balance_stats,
        super::checkin::export_checkin_data,
        super::checkin::export_checkin_stats,
        super::checkin::execute_cdk_recharge,
        super::checkin::get_cdk_history,
        super::checkin::list_waf_cookies,
        super::checkin::add_waf_cookie,
        super::checkin::delete_waf_cookie,
    ],
    system_info: "系统信息" [ReadOnly, Generated] => [
        super::system::get_system_info => ["void", "SystemInfo", "export const getSystemInfo = (): Promise<SystemInfo> => invoke('get_system_info')\n"],
        super::system::check_version => ["void", "VersionInfo", "export const checkVersion = (): Promise<VersionInfo> => invoke('check_version')\n"],
    ],
    system: "系统" [ReadOnly, LegacyJson] => [
        super::system::health_check,
    ],
    converter: "转换器" [ReadOnly, Generated] => [
        super::converter::convert_config => ["ConverterRequestDto", "ConvertResult", "export const convertConfig = (request: ConverterRequestDto): Promise<ConvertResult> =>\n  invoke('convert_config', { request })\n"],
    ],
    ui_state: "UI 状态" [LocalMutation, Generated] => [
        super::ui_state::get_favorites => ["void", "FavoriteCommandDto[]", "export const getFavorites = (): Promise<FavoriteCommandDto[]> => invoke('get_favorites')\n"],
        super::ui_state::add_favorite => ["{ command: string; args: string[]; displayName?: string | null; module: string }", "FavoriteCommandDto", "export const addFavorite = (command: string, args: string[], displayName: string | null | undefined, module: string): Promise<FavoriteCommandDto> =>\n  invoke('add_favorite', { command, args, displayName: displayName ?? null, module })\n"],
        super::ui_state::remove_favorite => ["string", "boolean", "export const removeFavorite = (id: string): Promise<boolean> => invoke('remove_favorite', { id })\n"],
        super::ui_state::get_recent_items => ["number | undefined", "CommandHistoryDto[]", "export const getRecentItems = (limit?: number): Promise<CommandHistoryDto[]> => invoke('get_recent_items', { limit })\n"],
        super::ui_state::add_recent_item => ["{ command: string; args: string[]; success: boolean; durationMs: number }", "CommandHistoryDto", "export const addRecentItem = (command: string, args: string[], success: boolean, durationMs: number): Promise<CommandHistoryDto> =>\n  invoke('add_recent_item', { command, args, success, durationMs })\n"],
        super::ui_state::clear_recent_items => ["void", "string", "export const clearRecentItems = (): Promise<string> => invoke('clear_recent_items')\n"],
    ],
    waf: "WAF" [NetworkMutation, LegacyJson] => [
        super::waf::open_waf_login,
        super::waf::get_waf_cookie_status,
        super::waf::validate_waf_cookie_for_account,
        super::waf::waf_deliver_cookie,
    ],
    unified_mcp: "统一 MCP" [SecretMutation, LegacyJson] => [
        super::unified_mcp::unified_list_mcp_servers,
        super::unified_mcp::unified_add_mcp_server,
        super::unified_mcp::unified_update_mcp_server,
        super::unified_mcp::unified_delete_mcp_server,
    ],
    events: "事件查询" [ReadOnly, Generated] => [
        super::system::get_recent_events => ["number | undefined", "EventLogEntryDto[]", "export const getRecentEvents = (count?: number): Promise<EventLogEntryDto[]> => invoke('get_recent_events', { count })\n"],
        super::system::get_monitoring_feed => ["MonitoringFeedQueryDto | undefined", "MonitoringEntryDto[]", "export const getMonitoringFeed = (query: MonitoringFeedQueryDto = {}): Promise<MonitoringEntryDto[]> => invoke('get_monitoring_feed', { query })\n"],
        super::system::append_frontend_logs => ["FrontendLogInputDto[]", "void", "export const appendFrontendLogs = (entries: FrontendLogInputDto[]): Promise<void> => invoke('append_frontend_logs', { entries })\n"],
        super::system::get_runtime_metrics => ["void", "RuntimeMetricsResponse", "export const getRuntimeMetrics = (): Promise<RuntimeMetricsResponse> => invoke('get_runtime_metrics')\n"],
    ],
    environment: "环境管理" [LocalMutation, Generated] => [
        super::environment::list_environments => ["void", "EnvironmentInfo[]", "export const listEnvironments = (): Promise<EnvironmentInfo[]> => invoke('list_environments')\n"],
        super::environment::get_current_environment => ["void", "EnvironmentInfo", "export const getCurrentEnvironment = (): Promise<EnvironmentInfo> => invoke('get_current_environment')\n"],
        super::environment::switch_environment => ["string", "EnvironmentInfo", "export const switchEnvironment = (envId: string): Promise<EnvironmentInfo> => invoke('switch_environment', { envId })\n"],
        super::environment::refresh_environments => ["boolean | undefined", "EnvironmentInfo[]", "export const refreshEnvironments = (forceRefresh?: boolean): Promise<EnvironmentInfo[]> => invoke('refresh_environments', { forceRefresh })\n"],
    ],
    environment_legacy: "环境动态探测" [LocalMutation, LegacyJson] => [
        super::environment::env_list_platforms,
        super::environment::env_detect_cli,
    ],
    ssh: "SSH" [NetworkMutation, Generated] => [
        super::ssh::ssh_list_hosts => ["void", "SshHostConfigDto[]", "export const sshListHosts = (): Promise<SshHostConfigDto[]> => invoke('ssh_list_hosts')\n"],
        super::ssh::ssh_add_host => ["AddSshHostRequest", "SshHostConfigDto", "export const sshAddHost = (host: AddSshHostRequest): Promise<SshHostConfigDto> => invoke('ssh_add_host', { host })\n"],
        super::ssh::ssh_connect => ["SshConnectInput", "SshConnectionState", "export const sshConnect = (input: SshConnectInput): Promise<SshConnectionState> => invoke('ssh_connect', input)\n"],
        super::ssh::ssh_reconnect => ["SshConnectInput", "SshConnectionState", "export const sshReconnect = (input: SshConnectInput): Promise<SshConnectionState> => invoke('ssh_reconnect', input)\n"],
        super::ssh::ssh_disconnect => ["void", "SshConnectionState", "export const sshDisconnect = (): Promise<SshConnectionState> => invoke('ssh_disconnect')\n"],
        super::ssh::ssh_get_connection_state => ["string | undefined", "SshConnectionStateResponse", "export const sshGetConnectionState = (envId?: string): Promise<SshConnectionStateResponse> => invoke('ssh_get_connection_state', { envId })\n"],
        super::ssh::ssh_probe_host_fingerprint => ["SshProbeFingerprintRequest", "SshFingerprintProbeResult", "export const sshProbeHostFingerprint = (request: SshProbeFingerprintRequest): Promise<SshFingerprintProbeResult> => invoke('ssh_probe_host_fingerprint', { request })\n"],
        super::ssh::ssh_confirm_host_fingerprint => ["string", "void", "export const sshConfirmHostFingerprint = (challengeId: string): Promise<void> => invoke('ssh_confirm_host_fingerprint', { request: { challenge_id: challengeId } })\n"],
        super::ssh::ssh_read_config => ["SshReadConfigInput", "string", "export const sshReadConfig = (input: SshReadConfigInput): Promise<string> => invoke('ssh_read_config', input)\n"],
        super::ssh::ssh_write_config => ["SshWriteConfigInput", "void", "export const sshWriteConfig = (input: SshWriteConfigInput): Promise<void> => invoke('ssh_write_config', input)\n"],
        super::ssh::ssh_detect_cli => ["string", "SshCliStatusDto[]", "export const sshDetectCli = (envId: string): Promise<SshCliStatusDto[]> => invoke('ssh_detect_cli', { envId })\n"],
        super::ssh::ssh_test_connection => ["string", "SshConnectResultDto", "export const sshTestConnection = (envId: string): Promise<SshConnectResultDto> => invoke('ssh_test_connection', { envId })\n"],
        super::ssh::ssh_list_keys => ["void", "SshKeyInfoDto[]", "export const sshListKeys = (): Promise<SshKeyInfoDto[]> => invoke('ssh_list_keys')\n"],
    ],
    builtin_prompts: "内置提示词" [ReadOnly, Generated] => [
        super::builtin_prompts::list_builtin_prompts => ["void", "BuiltinPromptDto[]", "export const listBuiltinPrompts = (): Promise<BuiltinPromptDto[]> => invoke('list_builtin_prompts')\n"],
        super::builtin_prompts::get_builtin_prompt => ["string", "BuiltinPromptDto | null", "export const getBuiltinPrompt = (id: string): Promise<BuiltinPromptDto | null> => invoke('get_builtin_prompt', { id })\n"],
        super::builtin_prompts::get_builtin_prompts_by_category => ["string", "BuiltinPromptDto[]", "export const getBuiltinPromptsByCategory = (category: string): Promise<BuiltinPromptDto[]> => invoke('get_builtin_prompts_by_category', { category })\n"],
    ],
    pricing: "定价管理" [LocalMutation, LegacyJson] => [
        super::pricing::set_pricing,
        super::pricing::get_pricing_list,
        super::pricing::remove_pricing,
        super::pricing::reset_pricing,
    ],
    mcp_presets: "MCP 预设" [NetworkMutation, LegacyJson] => [
        super::mcp_presets::list_mcp_presets,
        super::mcp_presets::get_mcp_preset,
        super::mcp_presets::install_mcp_preset,
        super::mcp_presets::install_mcp_preset_single,
        super::mcp_presets::list_source_mcp_servers,
        super::mcp_presets::sync_mcp_server,
        super::mcp_presets::sync_all_mcp_servers,
    ],
    usage_v2: "Usage V2" [ReadOnly, Generated] => [
        super::usage::get_usage_summary_v2 => ["UsageRangeInput", "UsageSummaryDto", "export const getUsageSummaryV2 = (platform?: string, startDate?: string, endDate?: string): Promise<UsageSummaryDto> =>\n  invoke('get_usage_summary_v2', { platform, startDate, endDate })\n"],
        super::usage::get_usage_capabilities_v2 => ["void", "CapabilityReport", "export const getUsageCapabilitiesV2 = (): Promise<CapabilityReport> => invoke('get_usage_capabilities_v2')\n"],
        super::usage::get_usage_trends_v2 => ["UsageRangeInput", "DailyTrendDto[]", "export const getUsageTrendsV2 = (platform?: string, startDate?: string, endDate?: string): Promise<DailyTrendDto[]> =>\n  invoke('get_usage_trends_v2', { platform, startDate, endDate })\n"],
        super::usage::get_usage_by_model_v2 => ["UsageRangeInput", "ModelStatDto[]", "export const getUsageByModelV2 = (platform?: string, startDate?: string, endDate?: string): Promise<ModelStatDto[]> =>\n  invoke('get_usage_by_model_v2', { platform, startDate, endDate })\n"],
        super::usage::get_usage_by_provider_v2 => ["UsageRangeInput", "ProviderBreakdownDto[]", "export const getUsageByProviderV2 = (platform?: string, startDate?: string, endDate?: string): Promise<ProviderBreakdownDto[]> =>\n  invoke('get_usage_by_provider_v2', { platform, startDate, endDate })\n"],
        super::usage::get_usage_by_project_v2 => ["UsageRangeInput", "ProjectStatDto[]", "export const getUsageByProjectV2 = (platform?: string, startDate?: string, endDate?: string): Promise<ProjectStatDto[]> =>\n  invoke('get_usage_by_project_v2', { platform, startDate, endDate })\n"],
        super::usage::get_usage_heatmap_v2 => ["{ platform?: string; days?: number }", "HeatmapResponseDto", "export const getUsageHeatmapV2 = (platform?: string, days?: number): Promise<HeatmapResponseDto> =>\n  invoke('get_usage_heatmap_v2', { platform, days })\n"],
        super::usage::get_usage_logs_v2 => ["UsageLogsQuery", "PaginatedLogsDto", "export const getUsageLogsV2 = (platformOrQuery?: string | UsageLogsQuery, page?: number, pageSize?: number, model?: string, cursor?: string, includeTotal?: boolean, mode?: 'cursor' | 'offset'): Promise<PaginatedLogsDto> => {\n  const query: UsageLogsQuery = typeof platformOrQuery === 'object'\n    ? platformOrQuery\n    : { platform: platformOrQuery, page, page_size: pageSize, model, cursor, include_total: includeTotal, mode }\n  return invoke('get_usage_logs_v2', { query })\n}\n"],
        super::usage::get_usage_dashboard_v2 => ["UsageDashboardInput", "UsageDashboardResponse", "export const getUsageDashboardV2 = (platform?: string, startDate?: string, endDate?: string, heatmapDays?: number, includeHeatmap?: boolean, provider?: string): Promise<UsageDashboardResponse> =>\n  invoke('get_usage_dashboard_v2', { platform, provider, startDate, endDate, heatmapDays, includeHeatmap })\n"],
        super::usage::get_home_usage_overview_v2 => ["number | undefined", "HomeUsageOverviewResponse", "export const getHomeUsageOverviewV2 = (days?: number): Promise<HomeUsageOverviewResponse> => invoke('get_home_usage_overview_v2', { days })\n"],
        super::usage::ensure_session_index_v2 => ["void", "StartSessionIndexJobResponse", "export const ensureSessionIndexV2 = (): Promise<StartSessionIndexJobResponse> => invoke('ensure_session_index_v2')\n"],
        super::usage::get_session_index_job_status_v2 => ["string", "SessionIndexJobSnapshot", "export const getSessionIndexJobStatusV2 = (jobId: string): Promise<SessionIndexJobSnapshot> => invoke('get_session_index_job_status_v2', { jobId })\n"],
        super::usage::start_usage_import_job_v2 => ["{ platform?: string; recentDays?: number; resetSources?: boolean }", "StartUsageImportJobResponse", "export const startUsageImportJobV2 = (platform?: string, recentDays?: number, resetSources?: boolean): Promise<StartUsageImportJobResponse> =>\n  invoke('start_usage_import_job_v2', { platform, recentDays, resetSources })\n"],
        super::usage::get_usage_import_job_status_v2 => ["string", "UsageImportJobSnapshot", "export const getUsageImportJobStatusV2 = (jobId: string): Promise<UsageImportJobSnapshot> => invoke('get_usage_import_job_status_v2', { jobId })\n"],
        super::usage::cancel_usage_import_job_v2 => ["string", "UsageImportJobSnapshot", "export const cancelUsageImportJobV2 = (jobId: string): Promise<UsageImportJobSnapshot> => invoke('cancel_usage_import_job_v2', { jobId })\n"],
        super::usage::import_usage_v2 => ["string", "UsageImportResultV2", "export const importUsageV2 = (platform: string): Promise<UsageImportResultV2> => invoke('import_usage_v2', { platform })\n"],
        super::usage::import_all_usage_v2 => ["void", "ImportAllUsageResponse", "export const importAllUsageV2 = (): Promise<ImportAllUsageResponse> => invoke('import_all_usage_v2')\n"],
    ],
    command_exec: "命令执行" [ProcessExecution, Generated] => [
        super::command_exec::execute_ccr_command => ["ExecuteCcrCommandInput", "CommandExecutionResult", "export const executeCcrCommand = (input: ExecuteCcrCommandInput): Promise<CommandExecutionResult> =>\n  invoke('execute_ccr_command', input)\n\n"],
        super::command_exec::list_ccr_commands => ["void", "CommandCatalog", "export const listCcrCommands = (): Promise<CommandCatalog> =>\n  invoke('list_ccr_commands')\n\n"],
        super::command_exec::get_ccr_command_help => ["string", "CommandHelpResponse", "export const getCcrCommandHelp = (command: string): Promise<CommandHelpResponse> =>\n  invoke('get_ccr_command_help', { command })\n\n"],
        super::command_exec::start_ccr_command_job => ["ExecuteCcrCommandInput", "StartCommandJobResponse", "export const startCcrCommandJob = (input: ExecuteCcrCommandInput): Promise<StartCommandJobResponse> =>\n  invoke('start_ccr_command_job', input)\n\n"],
        super::command_exec::get_ccr_command_job_status => ["string", "CommandJobSnapshot", "export const getCcrCommandJobStatus = (jobId: string): Promise<CommandJobSnapshot> =>\n  invoke('get_ccr_command_job_status', { jobId })\n\n"],
        super::command_exec::cancel_ccr_command_job => ["string", "CommandJobSnapshot", "export const cancelCcrCommandJob = (jobId: string): Promise<CommandJobSnapshot> =>\n  invoke('cancel_ccr_command_job', { jobId })\n"],
    ],
    checkin_extended: "签到扩展" [NetworkMutation, LegacyJson] => [
        super::checkin::list_builtin_providers,
        super::checkin::add_builtin_provider,
        super::checkin::get_checkin_account_cookies,
        super::checkin::export_checkin_config,
        super::checkin::preview_checkin_import,
        super::checkin::import_checkin_config,
        super::checkin::get_account_dashboard,
    ],
    config_extended: "配置扩展" [LocalMutation, LegacyJson] => [
        super::config::update_config,
        super::config::clean_backups,
    ],
    exit_confirm: "退出确认" [LocalMutation, Generated] => [
        super::config::get_skip_exit_confirm => ["void", "boolean", "export const getSkipExitConfirm = (): Promise<boolean> => invoke('get_skip_exit_confirm')\n"],
        super::config::set_skip_exit_confirm => ["boolean", "void", "export const setSkipExitConfirm = (skip: boolean): Promise<void> => invoke('set_skip_exit_confirm', { skip })\n"],
    ],
    shell: "Desktop Shell" [ProcessExecution, Generated] => [
        super::shell::shell_get_preferences => ["void", "DesktopShellPreferences", "export const shellGetPreferences = (): Promise<DesktopShellPreferences> => invoke('shell_get_preferences')\n"],
        super::shell::shell_set_preferences => ["DesktopShellPreferences", "DesktopShellPreferences", "export const shellSetPreferences = (preferences: DesktopShellPreferences): Promise<DesktopShellPreferences> => invoke('shell_set_preferences', { preferences })\n"],
        super::shell::shell_show_main_window => ["string | undefined", "void", "export const shellShowMainWindow = (targetRoute?: string): Promise<void> => invoke('shell_show_main_window', { targetRoute })\n"],
        super::shell::shell_request_quit => ["void", "void", "export const shellRequestQuit = (): Promise<void> => invoke('shell_request_quit')\n"],
        super::shell::shell_begin_tray_panel_drag => ["void", "void", "export const shellBeginTrayPanelDrag = (): Promise<void> => invoke('shell_begin_tray_panel_drag')\n"],
        super::shell::shell_complete_tray_panel_drag => ["TrayPanelManualPosition | null | undefined", "void", "export const shellCompleteTrayPanelDrag = (position?: TrayPanelManualPosition | null): Promise<void> => invoke('shell_complete_tray_panel_drag', { position: position ?? null })\n"],
        super::shell::shell_detect_skillport_app => ["void", "SkillportAppStatus", "export const shellDetectSkillportApp = (): Promise<SkillportAppStatus> => invoke('shell_detect_skillport_app')\n"],
        super::shell::shell_open_skillport_app => ["void", "void", "export const shellOpenSkillportApp = (): Promise<void> => invoke('shell_open_skillport_app')\n"],
        super::shell::shell_detect_skills_manage_app => ["void", "SkillportAppStatus", "export const shellDetectSkillsManageApp = (): Promise<SkillportAppStatus> => invoke('shell_detect_skills_manage_app')\n"],
        super::shell::shell_open_skills_manage_app => ["void", "void", "export const shellOpenSkillsManageApp = (): Promise<void> => invoke('shell_open_skills_manage_app')\n"],
    ],
    system_extended_legacy: "系统更新" [ProcessExecution, LegacyJson] => [
        super::system::update_ccr,
    ],
    system_extended: "CLI 版本探测" [ProcessExecution, Generated] => [
        super::system::get_cli_versions => ["CliVersionsOptions | undefined", "CliVersionsResponse", "export const getCliVersions = (options?: CliVersionsOptions): Promise<CliVersionsResponse> => invoke('get_cli_versions', { options })\n"],
        super::system::get_cli_version => ["CliVersionOptions", "CliVersionEntry", "export const getCliVersion = (options: CliVersionOptions): Promise<CliVersionEntry> => invoke('get_cli_version', { options })\n"],
    ],
    install: "llmusage 安装流程" [ProcessExecution, Generated] => [
        super::install::llmusage_install_detect => ["void", "DetectionResult", "export const llmusageInstallDetect = (): Promise<DetectionResult> => invoke('llmusage_install_detect')\n"],
        super::install::llmusage_install_probe_capabilities => ["void", "HostCapabilities", "export const llmusageInstallProbeCapabilities = (): Promise<HostCapabilities> => invoke('llmusage_install_probe_capabilities')\n"],
        super::install::llmusage_install_plan => ["{ detection: DetectionResult; capabilities: HostCapabilities }", "PlanOutcome", "export const llmusageInstallPlan = (detection: DetectionResult, capabilities: HostCapabilities): Promise<PlanOutcome> =>\n  invoke('llmusage_install_plan', { detection, capabilities })\n"],
        super::install::llmusage_install_execute => ["PlanId", "AttemptId", "export const llmusageInstallExecute = (planId: PlanId): Promise<AttemptId> => invoke('llmusage_install_execute', { planId })\n"],
        super::install::llmusage_install_cancel => ["AttemptId", "CancelResult", "export const llmusageInstallCancel = (attemptId: AttemptId): Promise<CancelResult> => invoke('llmusage_install_cancel', { attemptId })\n"],
        super::install::llmusage_install_recent => ["void", "RingBufferSnapshot", "export const llmusageInstallRecent = (): Promise<RingBufferSnapshot> => invoke('llmusage_install_recent')\n"],
        super::install::llmusage_install_manual_catalog => ["void", "ManualCatalog", "export const llmusageInstallManualCatalog = (): Promise<ManualCatalog> => invoke('llmusage_install_manual_catalog')\n"],
        super::install::llmusage_install_check => ["void", "[DetectionResult, HostCapabilities]", "export const llmusageInstallCheck = (): Promise<[DetectionResult, HostCapabilities]> => invoke('llmusage_install_check')\n"],
    ],
    claude_observer: "Claude Observer" [ReadOnly, Generated] => [
        super::claude_observer::claude_observer_get_insight => ["'today' | 'month' | 'all' | undefined", "InsightDto", "  getInsight: (range?: 'today' | 'month' | 'all'): Promise<InsightDto> => invoke('claude_observer_get_insight', { range }),\n"],
        super::claude_observer::claude_observer_daily_trend => ["number | undefined", "DailyPoint[]", "  dailyTrend: (days?: number): Promise<DailyPoint[]> => invoke('claude_observer_daily_trend', { days }),\n"],
        super::claude_observer::claude_observer_cost_breakdown => ["{ dim: 'project' | 'model'; days?: number; limit?: number }", "BreakdownRow[]", "  costBreakdown: (dim: 'project' | 'model', days?: number, limit?: number): Promise<BreakdownRow[]> => invoke('claude_observer_cost_breakdown', { dim, days, limit }),\n"],
        super::claude_observer::claude_observer_cache_stats => ["void", "CacheStatsDto", "  cacheStats: (): Promise<CacheStatsDto> => invoke('claude_observer_cache_stats'),\n"],
        super::claude_observer::claude_observer_top_sessions => ["{ limit?: number; by?: 'cost' | 'calls' }", "SessionRow[]", "  topSessions: (limit?: number, by?: 'cost' | 'calls'): Promise<SessionRow[]> => invoke('claude_observer_top_sessions', { limit, by }),\n"],
        super::claude_observer::claude_observer_tool_heatmap => ["number | undefined", "HeatmapCell[]", "  toolHeatmap: (days?: number): Promise<HeatmapCell[]> => invoke('claude_observer_tool_heatmap', { days }),\n"],
        super::claude_observer::claude_observer_top_tools => ["{ days?: number; limit?: number }", "TopToolRow[]", "  topTools: (days?: number, limit?: number): Promise<TopToolRow[]> => invoke('claude_observer_top_tools', { days, limit }),\n"],
        super::claude_observer::claude_observer_subscription_get => ["void", "SubscriptionDto", "  subscriptionGet: (): Promise<SubscriptionDto> => invoke('claude_observer_subscription_get'),\n"],
        super::claude_observer::claude_observer_subscription_set => ["{ mode: string; plan: string; monthlyUsd: number }", "SubscriptionDto", "  subscriptionSet: (mode: string, plan: string, monthlyUsd: number): Promise<SubscriptionDto> =>\n    invoke('claude_observer_subscription_set', { mode, plan, monthlyUsd }),\n"],
    ],
}

pub(crate) const WINDOWS_COMMAND_MODULES: &[CommandModule] = &[CommandModule {
    key: "wsl",
    title: "WSL",
    commands: &[
        stringify!(super::wsl::wsl_list_distros),
        stringify!(super::wsl::wsl_refresh_distros),
        stringify!(super::wsl::wsl_clear_cache),
        stringify!(super::wsl::wsl_cache_status),
        stringify!(super::wsl::wsl_read_config),
        stringify!(super::wsl::wsl_write_config),
        stringify!(super::wsl::wsl_detect_cli),
        stringify!(super::wsl::wsl_sync_config),
    ],
    wire_contracts: &[None, None, None, None, None, None, None, None],
    default_risk: CommandRisk::ProcessExecution,
    schema: CommandSchema::LegacyJson,
    platform: CommandPlatform::Windows,
}];

fn registered_command_count() -> usize {
    let base_count = COMMAND_MODULES
        .iter()
        .map(|module| module.commands.len())
        .sum::<usize>();

    #[cfg(target_os = "windows")]
    {
        base_count
            + WINDOWS_COMMAND_MODULES
                .iter()
                .map(|module| module.commands.len())
                .sum::<usize>()
    }

    #[cfg(not(target_os = "windows"))]
    {
        base_count
    }
}

fn command_registry_is_well_formed() -> bool {
    let base_modules_are_well_formed = COMMAND_MODULES.iter().all(|module| {
        !module.key.is_empty()
            && !module.title.is_empty()
            && !module.commands.is_empty()
            && module.commands.len() == module.wire_contracts.len()
    });

    #[cfg(target_os = "windows")]
    let platform_modules_are_well_formed = WINDOWS_COMMAND_MODULES.iter().all(|module| {
        !module.key.is_empty()
            && !module.title.is_empty()
            && !module.commands.is_empty()
            && module.commands.len() == module.wire_contracts.len()
    });
    #[cfg(not(target_os = "windows"))]
    let platform_modules_are_well_formed = true;

    base_modules_are_well_formed
        && platform_modules_are_well_formed
        && registered_command_count() > 0
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::fmt::Write as _;
    use std::path::PathBuf;

    use super::{
        command_descriptor, command_descriptors, command_registry_is_well_formed,
        registered_command_count, CommandAudit, CommandAuthorization, CommandConcurrency,
        CommandConfirmation, CommandDescriptor, CommandPlatform, CommandRisk, CommandSchema,
        COMMAND_MODULES, WINDOWS_COMMAND_MODULES,
    };

    #[derive(serde::Serialize)]
    struct CommandManifest {
        schema_version: u32,
        base_command_count: usize,
        windows_command_count: usize,
        typed_command_count: usize,
        exact_wire_type_count: usize,
        commands: Vec<CommandDescriptor>,
    }

    fn command_manifest() -> CommandManifest {
        let commands = command_descriptors().collect::<Vec<_>>();
        let base_command_count = commands
            .iter()
            .filter(|descriptor| descriptor.platform == CommandPlatform::Base)
            .count();
        let windows_command_count = commands.len();
        let typed_command_count = commands
            .iter()
            .filter(|descriptor| {
                descriptor.platform == CommandPlatform::Base && descriptor.is_typed()
            })
            .count();
        let exact_wire_type_count = commands
            .iter()
            .filter(|descriptor| {
                descriptor.platform == CommandPlatform::Base && descriptor.has_exact_wire_types()
            })
            .count();

        CommandManifest {
            schema_version: 2,
            base_command_count,
            windows_command_count,
            typed_command_count,
            exact_wire_type_count,
            commands,
        }
    }

    fn serialized_enum<T: serde::Serialize>(value: T) -> String {
        serde_json::to_string(&value)
            .expect("serialize command capability enum")
            .trim_matches('"')
            .to_string()
    }

    fn command_inventory_markdown() -> String {
        let manifest = command_manifest();
        let mut output = String::from(
            "# Tauri Command Inventory\n\n> Generated from `commands/handler_registry.rs`; do not edit manually.\n\n",
        );
        writeln!(output, "- Base commands: {}", manifest.base_command_count)
            .expect("write inventory count");
        writeln!(
            output,
            "- Windows commands: {}",
            manifest.windows_command_count
        )
        .expect("write inventory count");
        writeln!(output, "- Base modules: {}\n", COMMAND_MODULES.len())
            .expect("write inventory count");
        writeln!(
            output,
            "- Capability metadata: {}/{}",
            manifest.base_command_count, manifest.base_command_count
        )
        .expect("write metadata coverage");
        writeln!(
            output,
            "- Generated typed commands: {}/{} ({:.2}%)\n",
            manifest.typed_command_count,
            manifest.base_command_count,
            manifest.typed_command_count as f64 * 100.0 / manifest.base_command_count as f64
        )
        .expect("write typed coverage");
        writeln!(
            output,
            "- Exact input/output type declarations: {}/{}\n",
            manifest.exact_wire_type_count, manifest.typed_command_count
        )
        .expect("write exact wire type coverage");
        output.push_str(
            "| Module | Title | Platform | Commands | Default risk | Schema |\n| --- | --- | --- | ---: | --- | --- |\n",
        );
        for module in COMMAND_MODULES {
            writeln!(
                output,
                "| `{}` | {} | base | {} | `{}` | `{}` |",
                module.key,
                module.title,
                module.commands.len(),
                serialized_enum(module.default_risk),
                serialized_enum(module.schema)
            )
            .expect("write inventory row");
        }
        for module in WINDOWS_COMMAND_MODULES {
            writeln!(
                output,
                "| `{}` | {} | windows | {} | `{}` | `{}` |",
                module.key,
                module.title,
                module.commands.len(),
                serialized_enum(module.default_risk),
                serialized_enum(module.schema)
            )
            .expect("write inventory row");
        }
        output
    }

    fn command_manifest_json() -> String {
        let mut output =
            serde_json::to_string_pretty(&command_manifest()).expect("serialize command manifest");
        output.push('\n');
        output
    }

    fn command_capabilities_typescript() -> String {
        let manifest = command_manifest_json();
        let mut output = [
            "/* Generated from commands/handler_registry.rs; do not edit. */\n\n",
            "export type CommandRisk = 'read_only' | 'local_mutation' | 'secret_mutation' | 'network_mutation' | 'process_execution' | 'destructive'\n",
            "export type CommandSchema = 'generated' | 'legacy_json'\n",
            "export type CommandPlatform = 'base' | 'windows'\n\n",
            "export interface CommandCapability {\n",
            "  id: string\n",
            "  handler_path: string\n",
            "  module: string\n",
            "  title: string\n",
            "  platform: CommandPlatform\n",
            "  risk: CommandRisk\n",
            "  input_schema: CommandSchema\n",
            "  output_schema: CommandSchema\n",
            "  input_type?: string\n",
            "  output_type?: string\n",
            "  timeout_ms: number\n",
            "  concurrency: 'parallel' | 'module_exclusive' | 'singleton'\n",
            "  confirmation: 'none' | 'user_gesture' | 'opaque_capability'\n",
            "  authorization: 'local_user' | 'secret_access' | 'system_capability'\n",
            "  audit: 'metadata_only' | 'redacted'\n",
            "}\n\n",
            "export const COMMAND_MANIFEST = ",
        ]
        .concat();
        output.push_str(manifest.trim_end());
        output.push_str(
            &[
                " as const satisfies {\n",
                "  schema_version: number\n",
                "  base_command_count: number\n",
                "  windows_command_count: number\n",
                "  typed_command_count: number\n",
                "  exact_wire_type_count: number\n",
                "  commands: readonly CommandCapability[]\n",
                "}\n\n",
                "export type TauriCommandName = (typeof COMMAND_MANIFEST.commands)[number]['id']\n",
            ]
            .concat(),
        );
        output
    }

    fn append_module_client_declarations(output: &mut String, module_key: &str) {
        let module = COMMAND_MODULES
            .iter()
            .find(|module| module.key == module_key)
            .expect("typed client module");
        for contract in module.wire_contracts {
            let contract = contract.expect("exact wire contract for generated client");
            output.push_str(contract.client_declaration);
        }
    }

    fn command_exec_client_typescript() -> String {
        let mut output = [
            "/* Generated from commands/handler_registry.rs; do not edit. */\n\n",
            "import { invoke } from '@tauri-apps/api/core'\n",
            "import type { CommandCatalog } from '@/types/generated/command_exec/CommandCatalog'\n",
            "import type { CommandExecutionResult } from '@/types/generated/command_exec/CommandExecutionResult'\n",
            "import type { CommandHelpResponse } from '@/types/generated/command_exec/CommandHelpResponse'\n",
            "import type { CommandJobSnapshot } from '@/types/generated/command_exec/CommandJobSnapshot'\n",
            "import type { StartCommandJobResponse } from '@/types/generated/command_exec/StartCommandJobResponse'\n\n",
            "export type ExecuteCcrCommandInput = {\n",
            "  command: string\n",
            "  args?: string[]\n",
            "  confirmationToken?: string | null\n",
            "}\n\n",
        ]
        .concat();
        append_module_client_declarations(&mut output, "command_exec");
        output
    }

    fn sync_client_typescript() -> String {
        let mut output = [
            "/* Generated from commands/handler_registry.rs; do not edit. */\n\n",
            "import { invoke } from '@tauri-apps/api/core'\n",
            "import type { SyncAllAssetsInput } from '@/types/generated/sync/SyncAllAssetsInput'\n",
            "import type { SyncAssetInfo } from '@/types/generated/sync/SyncAssetInfo'\n",
            "import type { SyncAssetOperationInput } from '@/types/generated/sync/SyncAssetOperationInput'\n",
            "import type { SyncFolderInfo } from '@/types/generated/sync/SyncFolderInfo'\n",
            "import type { SyncOperationResult } from '@/types/generated/sync/SyncOperationResult'\n",
            "import type { SyncStatusInfo } from '@/types/generated/sync/SyncStatusInfo'\n",
            "import type { WebDavConfigDetails } from '@/types/generated/sync/WebDavConfigDetails'\n",
            "import type { WebDavConfigInput } from '@/types/generated/sync/WebDavConfigInput'\n",
            "import type { WebDavTestResult } from '@/types/generated/sync/WebDavTestResult'\n\n",
            "export type AddSyncFolderInput = { name: string; localPath: string; remotePath: string; description?: string }\n",
            "export type UpdateSyncFolderInput = { id: string; name?: string; enabled?: boolean; localPath?: string; remotePath?: string; description?: string }\n\n",
        ]
        .concat();
        append_module_client_declarations(&mut output, "sync");
        output
    }

    fn ssh_client_typescript() -> String {
        let mut output = [
            "/* Generated from commands/handler_registry.rs; do not edit. */\n\n",
            "import { invoke } from '@tauri-apps/api/core'\n",
            "import type { AddSshHostRequest } from '@/types/generated/ssh/AddSshHostRequest'\n",
            "import type { SshCliStatusDto } from '@/types/generated/ssh/SshCliStatusDto'\n",
            "import type { SshConnectionState } from '@/types/generated/ssh/SshConnectionState'\n",
            "import type { SshConnectionStateResponse } from '@/types/generated/ssh/SshConnectionStateResponse'\n",
            "import type { SshConnectResultDto } from '@/types/generated/ssh/SshConnectResultDto'\n",
            "import type { SshFingerprintProbeResult } from '@/types/generated/ssh/SshFingerprintProbeResult'\n",
            "import type { SshHostConfigDto } from '@/types/generated/ssh/SshHostConfigDto'\n",
            "import type { SshKeyInfoDto } from '@/types/generated/ssh/SshKeyInfoDto'\n",
            "import type { SshProbeFingerprintRequest } from '@/types/generated/ssh/SshProbeFingerprintRequest'\n\n",
            "export type SshConnectInput = { envId: string; password?: string }\n",
            "export type SshReadConfigInput = { envId: string; platform: string; path: string }\n",
            "export type SshWriteConfigInput = SshReadConfigInput & { content: string; enableBackup?: boolean }\n\n",
        ]
        .concat();
        append_module_client_declarations(&mut output, "ssh");
        output
    }

    fn claude_auth_client_typescript() -> String {
        let mut output = [
            "/* Generated from commands/handler_registry.rs; do not edit. */\n\n",
            "import { invoke } from '@tauri-apps/api/core'\n",
            "import type { ClaudeAuthActionResponse } from '@/types/generated/claude_auth/ClaudeAuthActionResponse'\n",
            "import type { ClaudeAuthCurrentResponse } from '@/types/generated/claude_auth/ClaudeAuthCurrentResponse'\n",
            "import type { ClaudeAuthListResponse } from '@/types/generated/claude_auth/ClaudeAuthListResponse'\n\n",
            "export type ClaudeAuthSaveRequest = {\n",
            "  name: string\n",
            "  description?: string | null\n",
            "  force?: boolean\n",
            "}\n\n",
        ]
        .concat();
        append_module_client_declarations(&mut output, "claude_auth");
        output
    }

    fn codex_auth_client_typescript() -> String {
        let mut output = [
            "/* Generated from commands/handler_registry.rs; do not edit. */\n\n",
            "import { invoke } from '@tauri-apps/api/core'\n",
            "import type { CodexAuthActionResponse } from '@/types/generated/codex_auth/CodexAuthActionResponse'\n",
            "import type { CodexAuthCurrentResponse } from '@/types/generated/codex_auth/CodexAuthCurrentResponse'\n",
            "import type { CodexAuthListResponse } from '@/types/generated/codex_auth/CodexAuthListResponse'\n",
            "import type { CodexAuthImportPayload } from '@/types/generated/codex_auth/CodexAuthImportPayload'\n",
            "import type { CodexAuthMutationResponse } from '@/types/generated/codex_auth/CodexAuthMutationResponse'\n",
            "import type { CodexAuthProcessResponse } from '@/types/generated/codex_auth/CodexAuthProcessResponse'\n",
            "import type { CodexAuthRenameResponse } from '@/types/generated/codex_auth/CodexAuthRenameResponse'\n",
            "import type { CodexApiKeyAddPayload } from '@/types/generated/codex_auth/CodexApiKeyAddPayload'\n",
            "import type { CodexModelProviderDeleteResponse } from '@/types/generated/codex_auth/CodexModelProviderDeleteResponse'\n",
            "import type { CodexModelProvidersResponse } from '@/types/generated/codex_auth/CodexModelProvidersResponse'\n",
            "import type { CodexModelProviderSaveResponse } from '@/types/generated/codex_auth/CodexModelProviderSaveResponse'\n",
            "import type { CodexModelProviderUpsertPayload } from '@/types/generated/codex_auth/CodexModelProviderUpsertPayload'\n\n",
            "import type { CodexOAuthStartResponse } from '@/types/generated/codex_auth/CodexOAuthStartResponse'\n",
            "import type { OAuthPortReleaseReport } from '@/types/generated/codex_auth/OAuthPortReleaseReport'\n\n",
            "export type CodexAuthSaveRequest = { name: string; description?: string; force?: boolean }\n\n",
        ]
        .concat();
        append_module_client_declarations(&mut output, "codex_auth");
        append_module_client_declarations(&mut output, "codex_model_providers");
        output
    }

    fn config_client_typescript() -> String {
        let mut output = [
            "/* Generated from commands/handler_registry.rs; do not edit. */\n\n",
            "import { invoke } from '@tauri-apps/api/core'\n",
            "import type { ConfigInfo } from '@/types/generated/config/ConfigInfo'\n",
            "import type { ExportResult } from '@/types/generated/config/ExportResult'\n",
            "import type { HistoryEntry } from '@/types/generated/config/HistoryEntry'\n",
            "import type { ImportResult } from '@/types/generated/config/ImportResult'\n\n",
            "export type AddConfigInput = {\n",
            "  name: string\n",
            "  description?: string | null\n",
            "  baseUrl: string\n",
            "  authToken: string\n",
            "  model?: string | null\n",
            "  smallFastModel?: string | null\n",
            "  provider?: string | null\n",
            "  providerType?: string | null\n",
            "  account?: string | null\n",
            "  tags?: string[] | null\n",
            "}\n",
            "export type ImportConfigInput = { content: string; mode?: string; backup?: boolean }\n\n",
            "const confirmationTokenFor = (action: 'delete_config' | 'import_config' | 'restore_config') => `desktop-confirm:${action}`\n\n",
        ]
        .concat();
        append_module_client_declarations(&mut output, "config");
        output
    }

    fn ui_state_client_typescript() -> String {
        let mut output = [
            "/* Generated from commands/handler_registry.rs; do not edit. */\n\n",
            "import { invoke } from '@tauri-apps/api/core'\n",
            "import type { CommandHistoryDto } from '@/types/generated/ui_state/CommandHistoryDto'\n",
            "import type { FavoriteCommandDto } from '@/types/generated/ui_state/FavoriteCommandDto'\n\n",
        ]
        .concat();
        append_module_client_declarations(&mut output, "ui_state");
        output
    }

    fn system_info_client_typescript() -> String {
        let mut output = [
            "/* Generated from commands/handler_registry.rs; do not edit. */\n\n",
            "import { invoke } from '@tauri-apps/api/core'\n",
            "import type { SystemInfo } from '@/types/generated/system/SystemInfo'\n",
            "import type { VersionInfo } from '@/types/generated/system/VersionInfo'\n\n",
        ]
        .concat();
        append_module_client_declarations(&mut output, "system_info");
        output
    }

    fn converter_client_typescript() -> String {
        let mut output = [
            "/* Generated from commands/handler_registry.rs; do not edit. */\n\n",
            "import { invoke } from '@tauri-apps/api/core'\n",
            "import type { ConverterRequestDto } from '@/types/generated/converter/ConverterRequestDto'\n",
            "import type { ConvertResult } from '@/types/generated/converter/ConvertResult'\n\n",
        ]
        .concat();
        append_module_client_declarations(&mut output, "converter");
        output
    }

    fn exit_confirm_client_typescript() -> String {
        let mut output = [
            "/* Generated from commands/handler_registry.rs; do not edit. */\n\n",
            "import { invoke } from '@tauri-apps/api/core'\n\n",
        ]
        .concat();
        append_module_client_declarations(&mut output, "exit_confirm");
        output
    }

    fn environment_client_typescript() -> String {
        let mut output = [
            "/* Generated from commands/handler_registry.rs; do not edit. */\n\n",
            "import { invoke } from '@tauri-apps/api/core'\n",
            "import type { EnvironmentInfo } from '@/types/generated/environment/EnvironmentInfo'\n\n",
        ]
        .concat();
        append_module_client_declarations(&mut output, "environment");
        output
    }

    fn events_client_typescript() -> String {
        let mut output = [
            "/* Generated from commands/handler_registry.rs; do not edit. */\n\n",
            "import { invoke } from '@tauri-apps/api/core'\n",
            "import type { EventLogEntryDto } from '@/types/generated/events/EventLogEntryDto'\n",
            "import type { FrontendLogInputDto } from '@/types/generated/events/FrontendLogInputDto'\n",
            "import type { MonitoringEntryDto } from '@/types/generated/events/MonitoringEntryDto'\n",
            "import type { MonitoringFeedQueryDto } from '@/types/generated/events/MonitoringFeedQueryDto'\n",
            "import type { RuntimeMetricsResponse } from '@/types/generated/events/RuntimeMetricsResponse'\n\n",
        ]
        .concat();
        append_module_client_declarations(&mut output, "events");
        output
    }

    fn shell_client_typescript() -> String {
        let mut output = [
            "/* Generated from commands/handler_registry.rs; do not edit. */\n\n",
            "import { invoke } from '@tauri-apps/api/core'\n",
            "import type { DesktopShellPreferences } from '@/types/generated/shell/DesktopShellPreferences'\n",
            "import type { SkillportAppStatus } from '@/types/generated/shell/SkillportAppStatus'\n",
            "import type { TrayPanelManualPosition } from '@/types/generated/shell/TrayPanelManualPosition'\n\n",
        ]
        .concat();
        append_module_client_declarations(&mut output, "shell");
        output
    }

    fn system_extended_client_typescript() -> String {
        let mut output = [
            "/* Generated from commands/handler_registry.rs; do not edit. */\n\n",
            "import { invoke } from '@tauri-apps/api/core'\n",
            "import type { CliVersionEntry } from '@/types/generated/system/CliVersionEntry'\n",
            "import type { CliVersionOptions } from '@/types/generated/system/CliVersionOptions'\n",
            "import type { CliVersionsOptions } from '@/types/generated/system/CliVersionsOptions'\n",
            "import type { CliVersionsResponse } from '@/types/generated/system/CliVersionsResponse'\n\n",
        ]
        .concat();
        append_module_client_declarations(&mut output, "system_extended");
        output
    }

    fn builtin_prompts_client_typescript() -> String {
        let mut output = [
            "/* Generated from commands/handler_registry.rs; do not edit. */\n\n",
            "import { invoke } from '@tauri-apps/api/core'\n",
            "import type { BuiltinPromptDto } from '@/types/generated/builtin_prompts/BuiltinPromptDto'\n\n",
        ]
        .concat();
        append_module_client_declarations(&mut output, "builtin_prompts");
        output
    }

    fn gemini_client_typescript() -> String {
        let mut output = [
            "/* Generated from commands/handler_registry.rs; do not edit. */\n\n",
            "import { invoke } from '@tauri-apps/api/core'\n",
            "import type { OpenJsonValueDto } from '@/types/generated/common/OpenJsonValueDto'\n\n",
        ]
        .concat();
        append_module_client_declarations(&mut output, "gemini");
        output
    }

    fn opencode_client_typescript() -> String {
        let mut output = [
            "/* Generated from commands/handler_registry.rs; do not edit. */\n\n",
            "import { invoke } from '@tauri-apps/api/core'\n",
            "import type { OpenJsonValueDto } from '@/types/generated/common/OpenJsonValueDto'\n",
            "import type { OpenCodePluginFileRecord } from '@/types/generated/opencode/OpenCodePluginFileRecord'\n",
            "import type { OpenCodeThemeRecord } from '@/types/generated/opencode/OpenCodeThemeRecord'\n\n",
        ]
        .concat();
        append_module_client_declarations(&mut output, "opencode");
        output
    }

    fn claude_client_typescript() -> String {
        let mut output = [
            "/* Generated from commands/handler_registry.rs; do not edit. */\n\n",
            "import { invoke } from '@tauri-apps/api/core'\n",
            "import type { OpenJsonValueDto } from '@/types/generated/common/OpenJsonValueDto'\n\n",
        ]
        .concat();
        append_module_client_declarations(&mut output, "claude");
        append_module_client_declarations(&mut output, "claude_profiles");
        output
    }

    fn codex_client_typescript() -> String {
        let mut output = [
            "/* Generated from commands/handler_registry.rs; do not edit. */\n\n",
            "import { invoke } from '@tauri-apps/api/core'\n",
            "import type { OpenJsonValueDto } from '@/types/generated/common/OpenJsonValueDto'\n\n",
            "export interface CodexAgentContextRequest { mode?: string; projectRoot?: string }\n",
            "export interface CodexAgentSourceInstallRequest { sourceId: string; agentId: string; targetName?: string | null; conflictMode?: string | null }\n",
            "export interface CodexAgentSourceSyncRequest { installId: string; force?: boolean }\n\n",
        ]
        .concat();
        append_module_client_declarations(&mut output, "codex");
        output
    }

    fn system_prompts_client_typescript() -> String {
        let mut output = [
            "/* Generated from commands/handler_registry.rs; do not edit. */\n\n",
            "import { invoke } from '@tauri-apps/api/core'\n",
            "import type { OpenJsonValueDto } from '@/types/generated/common/OpenJsonValueDto'\n\n",
        ]
        .concat();
        append_module_client_declarations(&mut output, "system_prompts");
        output
    }

    fn install_client_typescript() -> String {
        let mut output = [
            "/* Generated from commands/handler_registry.rs; do not edit. */\n\n",
            "import { invoke } from '@tauri-apps/api/core'\n",
            "import type { AttemptId } from '@/types/generated/install/AttemptId'\n",
            "import type { CancelResult } from '@/types/generated/install/CancelResult'\n",
            "import type { DetectionResult } from '@/types/generated/install/DetectionResult'\n",
            "import type { HostCapabilities } from '@/types/generated/install/HostCapabilities'\n",
            "import type { ManualCatalog } from '@/types/generated/install/ManualCatalog'\n",
            "import type { PlanId } from '@/types/generated/install/PlanId'\n",
            "import type { PlanOutcome } from '@/types/generated/install/PlanOutcome'\n",
            "import type { RingBufferSnapshot } from '@/types/generated/install/RingBufferSnapshot'\n\n",
        ]
        .concat();
        append_module_client_declarations(&mut output, "install");
        output
    }

    fn usage_v2_client_typescript() -> String {
        let mut output = [
            "/* Generated from commands/handler_registry.rs; do not edit. */\n\n",
            "import { invoke } from '@tauri-apps/api/core'\n",
            "import type { CapabilityReport } from '@/types/generated/usage/CapabilityReport'\n",
            "import type { DailyTrendDto } from '@/types/generated/usage/DailyTrendDto'\n",
            "import type { HeatmapResponseDto } from '@/types/generated/usage/HeatmapResponseDto'\n",
            "import type { HomeUsageOverviewResponse } from '@/types/generated/usage/HomeUsageOverviewResponse'\n",
            "import type { ImportAllUsageResponse } from '@/types/generated/usage/ImportAllUsageResponse'\n",
            "import type { ModelStatDto } from '@/types/generated/usage/ModelStatDto'\n",
            "import type { PaginatedLogsDto } from '@/types/generated/usage/PaginatedLogsDto'\n",
            "import type { ProjectStatDto } from '@/types/generated/usage/ProjectStatDto'\n",
            "import type { ProviderBreakdownDto } from '@/types/generated/usage/ProviderBreakdownDto'\n",
            "import type { SessionIndexJobSnapshot } from '@/types/generated/usage/SessionIndexJobSnapshot'\n",
            "import type { StartSessionIndexJobResponse } from '@/types/generated/usage/StartSessionIndexJobResponse'\n",
            "import type { StartUsageImportJobResponse } from '@/types/generated/usage/StartUsageImportJobResponse'\n",
            "import type { UsageDashboardResponse } from '@/types/generated/usage/UsageDashboardResponse'\n",
            "import type { UsageImportJobSnapshot } from '@/types/generated/usage/UsageImportJobSnapshot'\n",
            "import type { UsageImportResultV2 } from '@/types/generated/usage/UsageImportResultV2'\n",
            "import type { UsageLogsQuery } from '@/types/generated/usage/UsageLogsQuery'\n",
            "import type { UsageSummaryDto } from '@/types/generated/usage/UsageSummaryDto'\n\n",
            "export type { UsageLogsQuery }\n",
            "export type UsageRangeInput = { platform?: string; startDate?: string; endDate?: string }\n",
            "export type UsageDashboardInput = UsageRangeInput & { heatmapDays?: number; includeHeatmap?: boolean; provider?: string }\n\n",
        ]
        .concat();
        append_module_client_declarations(&mut output, "usage_v2");
        output
    }

    fn claude_observer_client_typescript() -> String {
        let mut output = [
            "/* Generated from commands/handler_registry.rs; do not edit. */\n\n",
            "import { invoke } from '@tauri-apps/api/core'\n",
            "import type { BreakdownRow } from '@/types/generated/claude_observer/BreakdownRow'\n",
            "import type { CacheStatsDto } from '@/types/generated/claude_observer/CacheStatsDto'\n",
            "import type { DailyPoint } from '@/types/generated/claude_observer/DailyPoint'\n",
            "import type { HeatmapCell } from '@/types/generated/claude_observer/HeatmapCell'\n",
            "import type { InsightDto } from '@/types/generated/claude_observer/InsightDto'\n",
            "import type { SessionRow } from '@/types/generated/claude_observer/SessionRow'\n",
            "import type { SubscriptionDto } from '@/types/generated/claude_observer/SubscriptionDto'\n",
            "import type { TopToolRow } from '@/types/generated/claude_observer/TopToolRow'\n\n",
            "export const claudeObserver = {\n",
        ]
        .concat();
        append_module_client_declarations(&mut output, "claude_observer");
        output.push_str("}\n");
        output
    }

    fn generated_artifacts() -> Vec<(PathBuf, String)> {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        vec![
            (
                root.join("docs/reference/tauri-command-inventory.md"),
                command_inventory_markdown(),
            ),
            (
                root.join("docs/en/reference/tauri-command-inventory.md"),
                command_inventory_markdown(),
            ),
            (
                root.join("ccr-ui/src/api/generated/command-manifest.json"),
                command_manifest_json(),
            ),
            (
                root.join("ccr-ui/src/api/generated/commandCapabilities.ts"),
                command_capabilities_typescript(),
            ),
            (
                root.join("ccr-ui/src/api/generated/commandExec.ts"),
                command_exec_client_typescript(),
            ),
            (
                root.join("ccr-ui/src/api/generated/sync.ts"),
                sync_client_typescript(),
            ),
            (
                root.join("ccr-ui/src/api/generated/ssh.ts"),
                ssh_client_typescript(),
            ),
            (
                root.join("ccr-ui/src/api/generated/claudeAuth.ts"),
                claude_auth_client_typescript(),
            ),
            (
                root.join("ccr-ui/src/api/generated/codexAuth.ts"),
                codex_auth_client_typescript(),
            ),
            (
                root.join("ccr-ui/src/api/generated/config.ts"),
                config_client_typescript(),
            ),
            (
                root.join("ccr-ui/src/api/generated/uiState.ts"),
                ui_state_client_typescript(),
            ),
            (
                root.join("ccr-ui/src/api/generated/systemInfo.ts"),
                system_info_client_typescript(),
            ),
            (
                root.join("ccr-ui/src/api/generated/converter.ts"),
                converter_client_typescript(),
            ),
            (
                root.join("ccr-ui/src/api/generated/exitConfirm.ts"),
                exit_confirm_client_typescript(),
            ),
            (
                root.join("ccr-ui/src/api/generated/environment.ts"),
                environment_client_typescript(),
            ),
            (
                root.join("ccr-ui/src/api/generated/events.ts"),
                events_client_typescript(),
            ),
            (
                root.join("ccr-ui/src/api/generated/shell.ts"),
                shell_client_typescript(),
            ),
            (
                root.join("ccr-ui/src/api/generated/systemExtended.ts"),
                system_extended_client_typescript(),
            ),
            (
                root.join("ccr-ui/src/api/generated/builtinPrompts.ts"),
                builtin_prompts_client_typescript(),
            ),
            (
                root.join("ccr-ui/src/api/generated/gemini.ts"),
                gemini_client_typescript(),
            ),
            (
                root.join("ccr-ui/src/api/generated/openCode.ts"),
                opencode_client_typescript(),
            ),
            (
                root.join("ccr-ui/src/api/generated/claude.ts"),
                claude_client_typescript(),
            ),
            (
                root.join("ccr-ui/src/api/generated/codex.ts"),
                codex_client_typescript(),
            ),
            (
                root.join("ccr-ui/src/api/generated/systemPrompts.ts"),
                system_prompts_client_typescript(),
            ),
            (
                root.join("ccr-ui/src/api/generated/install.ts"),
                install_client_typescript(),
            ),
            (
                root.join("ccr-ui/src/api/generated/usageV2.ts"),
                usage_v2_client_typescript(),
            ),
            (
                root.join("ccr-ui/src/api/generated/claudeObserver.ts"),
                claude_observer_client_typescript(),
            ),
        ]
    }

    #[test]
    fn command_registry_shape_matches_current_handler_surface() {
        assert_eq!(COMMAND_MODULES.len(), 36);

        #[cfg(target_os = "windows")]
        assert_eq!(registered_command_count(), 323);

        #[cfg(not(target_os = "windows"))]
        assert_eq!(registered_command_count(), 315);
    }

    #[test]
    fn command_registry_modules_are_well_formed() {
        assert!(command_registry_is_well_formed());
    }

    #[test]
    fn command_registry_paths_are_unique() {
        let mut seen = HashSet::new();

        for module in COMMAND_MODULES {
            for command in module.commands {
                assert!(
                    seen.insert(*command),
                    "duplicate command path in Tauri command registry: {command}"
                );
            }
        }

        #[cfg(target_os = "windows")]
        for module in super::WINDOWS_COMMAND_MODULES {
            for command in module.commands {
                assert!(
                    seen.insert(*command),
                    "duplicate Windows command path in Tauri command registry: {command}"
                );
            }
        }
    }

    #[test]
    fn command_capability_descriptors_are_complete_and_unique() {
        let descriptors = command_descriptors().collect::<Vec<_>>();
        assert_eq!(descriptors.len(), 323);

        let mut ids = HashSet::new();
        let mut paths = HashSet::new();
        for descriptor in &descriptors {
            assert!(
                ids.insert(descriptor.id),
                "duplicate command id: {}",
                descriptor.id
            );
            assert!(
                paths.insert(descriptor.handler_path),
                "duplicate handler path: {}",
                descriptor.handler_path
            );
            assert!(!descriptor.id.is_empty());
            assert!(!descriptor.module.is_empty());
            assert!(!descriptor.title.is_empty());
            assert!(descriptor.timeout_ms > 0);
            assert_eq!(descriptor.input_schema, descriptor.output_schema);
        }

        let manifest = command_manifest();
        assert_eq!(manifest.base_command_count, 315);
        assert_eq!(manifest.windows_command_count, 323);
        assert_eq!(manifest.typed_command_count, 252);
        assert_eq!(manifest.exact_wire_type_count, 252);

        let exact_contract_modules = descriptors
            .iter()
            .filter(|descriptor| {
                matches!(
                    descriptor.module,
                    "command_exec"
                        | "sync"
                        | "ssh"
                        | "config"
                        | "system_prompts"
                        | "claude_auth"
                        | "codex_auth"
                        | "codex_model_providers"
                        | "system_info"
                        | "converter"
                        | "ui_state"
                        | "events"
                        | "environment"
                        | "builtin_prompts"
                        | "exit_confirm"
                        | "shell"
                        | "system_extended"
                        | "gemini"
                        | "opencode"
                        | "claude"
                        | "claude_profiles"
                        | "codex"
                        | "usage_v2"
                        | "install"
                        | "claude_observer"
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(exact_contract_modules.len(), 252);
        assert!(exact_contract_modules
            .iter()
            .all(|descriptor| descriptor.has_exact_wire_types()));
    }

    #[test]
    fn command_capability_policy_covers_security_boundaries() {
        let delete_config = command_descriptor("delete_config").expect("delete config descriptor");
        assert_eq!(delete_config.risk, CommandRisk::Destructive);
        assert_eq!(delete_config.confirmation, CommandConfirmation::UserGesture);
        assert_eq!(
            delete_config.authorization,
            CommandAuthorization::SecretAccess
        );
        assert_eq!(delete_config.audit, CommandAudit::Redacted);

        let sync_push = command_descriptor("sync_push").expect("sync push descriptor");
        assert_eq!(sync_push.risk, CommandRisk::NetworkMutation);
        assert_eq!(sync_push.concurrency, CommandConcurrency::ModuleExclusive);

        let sync_list =
            command_descriptor("list_sync_assets").expect("list sync assets descriptor");
        assert_eq!(sync_list.risk, CommandRisk::ReadOnly);
        assert_eq!(sync_list.authorization, CommandAuthorization::SecretAccess);
        assert_eq!(sync_list.audit, CommandAudit::Redacted);

        let command_list =
            command_descriptor("list_ccr_commands").expect("list commands descriptor");
        assert_eq!(command_list.risk, CommandRisk::ReadOnly);
        assert_eq!(
            command_list.authorization,
            CommandAuthorization::SystemCapability
        );
        assert_eq!(command_list.audit, CommandAudit::Redacted);

        let install = command_descriptor("llmusage_install_execute")
            .expect("llmusage install execute descriptor");
        assert_eq!(install.risk, CommandRisk::ProcessExecution);
        assert_eq!(install.confirmation, CommandConfirmation::OpaqueCapability);
        assert_eq!(install.input_schema, CommandSchema::Generated);
        assert_eq!(install.output_schema, CommandSchema::Generated);

        let codex_delete =
            command_descriptor("codex_delete_session").expect("Codex delete descriptor");
        assert_eq!(codex_delete.risk, CommandRisk::Destructive);
        assert_eq!(codex_delete.audit, CommandAudit::Redacted);

        let claude_get = command_descriptor("claude_get_settings").expect("Claude read descriptor");
        assert_eq!(claude_get.risk, CommandRisk::ReadOnly);
        assert_eq!(claude_get.authorization, CommandAuthorization::SecretAccess);

        let prompts_list =
            command_descriptor("system_prompts_list").expect("system prompts list descriptor");
        assert_eq!(prompts_list.risk, CommandRisk::ReadOnly);
    }

    #[test]
    fn generated_command_exec_client_covers_the_typed_registry_module() {
        let client = command_exec_client_typescript();
        let module = COMMAND_MODULES
            .iter()
            .find(|module| module.key == "command_exec")
            .expect("command_exec module");

        assert_eq!(module.schema, CommandSchema::Generated);
        assert_eq!(client.matches("invoke('").count(), module.commands.len());
        for handler_path in module.commands {
            let command = handler_path
                .rsplit("::")
                .next()
                .expect("command handler name");
            assert!(
                client.contains(&format!("invoke('{command}'")),
                "generated client missing typed command: {command}"
            );
        }
        assert!(!client.contains("unknown"));
        assert!(!client.contains("<T"));
    }

    #[test]
    fn generated_sync_client_covers_the_typed_registry_module() {
        let client = sync_client_typescript();
        let module = COMMAND_MODULES
            .iter()
            .find(|module| module.key == "sync")
            .expect("sync module");

        assert_eq!(module.schema, CommandSchema::Generated);
        assert_eq!(client.matches("invoke('").count(), module.commands.len());
        for handler_path in module.commands {
            let command = handler_path
                .rsplit("::")
                .next()
                .expect("command handler name");
            assert!(
                client.contains(&format!("invoke('{command}'")),
                "generated client missing typed command: {command}"
            );
        }
        assert!(!client.contains("unknown"));
        assert!(!client.contains("<T"));
    }

    #[test]
    fn generated_ssh_client_covers_the_typed_registry_module() {
        let client = ssh_client_typescript();
        let module = COMMAND_MODULES
            .iter()
            .find(|module| module.key == "ssh")
            .expect("ssh module");

        assert_eq!(module.schema, CommandSchema::Generated);
        assert_eq!(client.matches("invoke('").count(), module.commands.len());
        for handler_path in module.commands {
            let command = handler_path
                .rsplit("::")
                .next()
                .expect("command handler name");
            assert!(
                client.contains(&format!("invoke('{command}'")),
                "generated client missing typed command: {command}"
            );
        }
        assert!(!client.contains("unknown"));
        assert!(!client.contains("<T"));
    }

    #[test]
    fn generated_claude_auth_client_covers_the_typed_registry_module() {
        let client = claude_auth_client_typescript();
        let module = COMMAND_MODULES
            .iter()
            .find(|module| module.key == "claude_auth")
            .expect("claude_auth module");

        assert_eq!(module.schema, CommandSchema::Generated);
        assert_eq!(client.matches("invoke('").count(), module.commands.len());
        for handler_path in module.commands {
            let command = handler_path
                .rsplit("::")
                .next()
                .expect("command handler name");
            assert!(
                client.contains(&format!("invoke('{command}'")),
                "generated client missing typed command: {command}"
            );
        }
        assert!(!client.contains("unknown"));
        assert!(!client.contains("<T"));
        assert!(!include_str!("claude_auth.rs").contains("Result<Value"));
    }

    #[test]
    fn generated_codex_auth_client_covers_typed_auth_and_provider_modules() {
        let client = codex_auth_client_typescript();
        let modules = COMMAND_MODULES
            .iter()
            .filter(|module| matches!(module.key, "codex_auth" | "codex_model_providers"))
            .collect::<Vec<_>>();
        let command_count = modules
            .iter()
            .map(|module| module.commands.len())
            .sum::<usize>();

        assert_eq!(modules.len(), 2);
        assert!(modules
            .iter()
            .all(|module| module.schema == CommandSchema::Generated));
        assert_eq!(client.matches("invoke('").count(), command_count);
        for handler_path in modules.iter().flat_map(|module| module.commands) {
            let command = handler_path
                .rsplit("::")
                .next()
                .expect("command handler name");
            assert!(
                client.contains(&format!("invoke('{command}'")),
                "generated client missing typed command: {command}"
            );
        }
        assert!(!client.contains("unknown"));
        assert!(!client.contains("<T"));
    }

    #[test]
    fn generated_config_client_covers_the_typed_registry_module() {
        let client = config_client_typescript();
        let module = COMMAND_MODULES
            .iter()
            .find(|module| module.key == "config")
            .expect("config module");

        assert_eq!(module.schema, CommandSchema::Generated);
        assert_eq!(client.matches("invoke('").count(), module.commands.len());
        for handler_path in module.commands {
            let command = handler_path
                .rsplit("::")
                .next()
                .expect("command handler name");
            assert!(
                client.contains(&format!("invoke('{command}'")),
                "generated client missing typed command: {command}"
            );
        }
        assert!(!client.contains("unknown"));
        assert!(!client.contains("<T"));
    }

    #[test]
    fn generated_claude_client_covers_core_and_profile_modules() {
        let client = claude_client_typescript();
        let modules = COMMAND_MODULES
            .iter()
            .filter(|module| matches!(module.key, "claude" | "claude_profiles"))
            .collect::<Vec<_>>();

        assert_eq!(modules.len(), 2);
        assert!(modules
            .iter()
            .all(|module| module.schema == CommandSchema::Generated));
        assert_eq!(
            client.matches("invoke('").count(),
            modules
                .iter()
                .map(|module| module.commands.len())
                .sum::<usize>()
        );
        for handler_path in modules.iter().flat_map(|module| module.commands) {
            let command = handler_path
                .rsplit("::")
                .next()
                .expect("command handler name");
            assert!(
                client.contains(&format!("invoke('{command}'")),
                "generated client missing typed command: {command}"
            );
        }
        assert!(!client.contains("unknown"));
        assert!(!client.contains("<T"));
    }

    #[test]
    fn generated_codex_client_covers_the_typed_registry_module() {
        let client = codex_client_typescript();
        let module = COMMAND_MODULES
            .iter()
            .find(|module| module.key == "codex")
            .expect("codex module");

        assert_eq!(module.schema, CommandSchema::Generated);
        assert_eq!(client.matches("invoke('").count(), module.commands.len());
        for handler_path in module.commands {
            let command = handler_path
                .rsplit("::")
                .next()
                .expect("command handler name");
            assert!(
                client.contains(&format!("invoke('{command}'")),
                "generated client missing typed command: {command}"
            );
        }
        assert!(!client.contains("unknown"));
        assert!(!client.contains("any"));
        assert!(!client.contains("<T"));
    }

    #[test]
    fn generated_small_domain_clients_cover_their_typed_registry_modules() {
        for (key, client) in [
            ("ui_state", ui_state_client_typescript()),
            ("system_info", system_info_client_typescript()),
            ("converter", converter_client_typescript()),
            ("exit_confirm", exit_confirm_client_typescript()),
            ("environment", environment_client_typescript()),
            ("events", events_client_typescript()),
            ("shell", shell_client_typescript()),
            ("system_extended", system_extended_client_typescript()),
            ("builtin_prompts", builtin_prompts_client_typescript()),
            ("gemini", gemini_client_typescript()),
            ("opencode", opencode_client_typescript()),
            ("system_prompts", system_prompts_client_typescript()),
            ("usage_v2", usage_v2_client_typescript()),
            ("install", install_client_typescript()),
            ("claude_observer", claude_observer_client_typescript()),
        ] {
            let module = COMMAND_MODULES
                .iter()
                .find(|module| module.key == key)
                .expect("typed module");

            assert_eq!(module.schema, CommandSchema::Generated);
            assert_eq!(client.matches("invoke('").count(), module.commands.len());
            for handler_path in module.commands {
                let command = handler_path
                    .rsplit("::")
                    .next()
                    .expect("command handler name");
                assert!(
                    client.contains(&format!("invoke('{command}'")),
                    "generated client missing typed command: {command}"
                );
            }
            assert!(!client.contains("unknown"));
            assert!(!client.contains("<T ="));
            assert!(!client.contains("invoke<T"));
        }
    }

    #[test]
    fn command_inventory_document_matches_registry() {
        for (path, expected) in generated_artifacts() {
            if std::env::var_os("CCR_UPDATE_COMMAND_INVENTORY").is_some() {
                std::fs::create_dir_all(path.parent().expect("inventory parent"))
                    .expect("create inventory directory");
                std::fs::write(&path, &expected).expect("write command inventory");
            }
            let actual = std::fs::read_to_string(&path).expect("read command inventory");
            assert_eq!(actual, expected, "run `just tauri-command-inventory`");
        }
    }
}
