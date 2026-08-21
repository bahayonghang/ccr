# Tauri Command Inventory

> Generated from `commands/handler_registry.rs`; do not edit manually.

- Base commands: 334
- Windows commands: 342
- Base modules: 37

- Capability metadata: 334/334
- Generated typed commands: 271/334 (81.14%)

- Exact input/output type declarations: 271/271

| Module | Title | Platform | Commands | Default risk | Schema |
| --- | --- | --- | ---: | --- | --- |
| `config` | 配置管理 | base | 12 | `local_mutation` | `generated` |
| `settings_raw` | 配置源文件 | base | 6 | `secret_mutation` | `legacy_json` |
| `system_prompts` | 系统提示词 | base | 4 | `local_mutation` | `generated` |
| `sync` | 同步 | base | 17 | `network_mutation` | `generated` |
| `claude` | Claude Code | base | 28 | `secret_mutation` | `generated` |
| `claude_profiles` | Claude Code Profiles | base | 10 | `secret_mutation` | `generated` |
| `claude_auth` | Claude Auth | base | 6 | `secret_mutation` | `generated` |
| `codex` | Codex | base | 44 | `secret_mutation` | `generated` |
| `codex_auth` | Codex Auth | base | 18 | `secret_mutation` | `generated` |
| `codex_model_providers` | Codex Model Providers | base | 3 | `secret_mutation` | `generated` |
| `gemini` | Gemini | base | 11 | `secret_mutation` | `generated` |
| `grok` | Grok Build | base | 15 | `secret_mutation` | `generated` |
| `opencode` | OpenCode | base | 16 | `secret_mutation` | `generated` |
| `checkin` | CheckIn | base | 25 | `network_mutation` | `legacy_json` |
| `system_info` | 系统信息 | base | 2 | `read_only` | `generated` |
| `system` | 系统 | base | 1 | `read_only` | `legacy_json` |
| `converter` | 转换器 | base | 1 | `read_only` | `generated` |
| `ui_state` | UI 状态 | base | 6 | `local_mutation` | `generated` |
| `waf` | WAF | base | 4 | `network_mutation` | `legacy_json` |
| `unified_mcp` | 统一 MCP | base | 4 | `secret_mutation` | `legacy_json` |
| `events` | 事件查询 | base | 4 | `read_only` | `generated` |
| `environment` | 环境管理 | base | 4 | `local_mutation` | `generated` |
| `environment_legacy` | 环境动态探测 | base | 2 | `local_mutation` | `legacy_json` |
| `ssh` | SSH | base | 13 | `network_mutation` | `generated` |
| `builtin_prompts` | 内置提示词 | base | 3 | `read_only` | `generated` |
| `pricing` | 定价管理 | base | 4 | `local_mutation` | `legacy_json` |
| `mcp_presets` | MCP 预设 | base | 7 | `network_mutation` | `legacy_json` |
| `usage_v2` | Usage V2 | base | 17 | `read_only` | `generated` |
| `command_exec` | 命令执行 | base | 6 | `process_execution` | `generated` |
| `checkin_extended` | 签到扩展 | base | 7 | `network_mutation` | `legacy_json` |
| `config_extended` | 配置扩展 | base | 2 | `local_mutation` | `legacy_json` |
| `exit_confirm` | 退出确认 | base | 2 | `local_mutation` | `generated` |
| `shell` | Desktop Shell | base | 10 | `process_execution` | `generated` |
| `system_extended_legacy` | 系统更新 | base | 1 | `process_execution` | `legacy_json` |
| `system_extended` | CLI 版本探测 | base | 2 | `process_execution` | `generated` |
| `install` | llmusage 安装流程 | base | 8 | `process_execution` | `generated` |
| `claude_observer` | Claude Observer | base | 9 | `read_only` | `generated` |
| `wsl` | WSL | windows | 8 | `process_execution` | `legacy_json` |
