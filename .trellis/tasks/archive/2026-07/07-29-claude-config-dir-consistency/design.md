# 设计:Claude 运行时路径单一解析

## 归属

在 `ccr-config` 新增 `ClaudeRuntimePaths`,因为它是 Claude 配置域的跨消费者路径模型:ccr-cli 与 Tauri 都已依赖 `ccr-config`;放入 `ccr-core` 会把平台领域知识下沉到基础设施层,放入 `ccr-types` 则会让纯 wire/data crate 负责文件系统环境。

建议字段:

- `config_dir`
- `settings_file`
- `credentials_file`
- `state_file` (`.claude.json`)
- `backups_dir`

## 解析契约

`resolve_with(home, env_getter)` 是可测试纯入口,`from_env()` 只负责取得 home 与进程 env。

1. `config_dir`: `CLAUDE_CONFIG_DIR` > `<home>/.claude`。
2. `settings_file`: `CCR_SETTINGS_PATH` > `<config_dir>/settings.json`。
3. `credentials_file`: `<config_dir>/.credentials.json`。
4. `state_file`: `CLAUDE_JSON_PATH` > 当设置 `CLAUDE_CONFIG_DIR` 时 `<config_dir>/.claude.json` > `<home>/.claude.json`。
5. `backups_dir`: `CCR_BACKUP_DIR` > `<config_dir>/backups`。

路径只做词法展开/拼接,不 canonicalize 不存在的目标。Windows 支持盘符、反斜杠和 `%NAME%` 展开;未知 `%NAME%` 保留原文并由调用点后续 I/O 报错,避免静默写错目录。非 Windows 不解释 `%...%`。

## 消费者迁移

- `SettingsManager::with_default` 与 Claude 分支 `get_platform_paths` 使用共享 settings/backups。
- `ClaudeAuthService::new` 保存 `ClaudeRuntimePaths`,credentials/settings/state 都由其派生。
- `ClaudePlatform` 与 doctor 不再各自解析 `CLAUDE_JSON_PATH`。
- Tauri `LocalEnvironment::resolve_config_path("claude", ...)` 使用 `config_dir`;`settings_raw.rs` 的 Claude settings 路径同源。
- `ClaudeMcpContext` 的用户状态文件使用共享 `state_file`;项目级 `.mcp.json` / `.claude/settings*.json` 仍按 project root,不受 user config dir 影响。

本任务不迁移 session observer、usage 扫描、skills、system prompts、sync 展示文案等非认证/运行时设置路径;这些路径是否应跟随 `CLAUDE_CONFIG_DIR` 需单独产品审计,避免把 C7 扩成全仓目录迁移。

## 数据与兼容

不搬迁已有文件,只让后续读写命中 Claude Code 当前选择的目录。未设置任何覆盖变量时,路径逐字节保持现状。`CCR_SETTINGS_PATH` 只覆盖 settings 文件,不会意外改变 credentials/state 根目录。

## 测试

- 纯解析矩阵:默认、每个覆盖变量、组合优先级、Windows `%USERPROFILE%`、带空格/反斜杠路径。
- CLI fixture:profile/auth/doctor 对同一临时 config dir 可见。
- Tauri local:read/write config 与 settings_raw 命中共享路径。
- 状态路径:复现 2.1.220 探针位置,并验证 `CLAUDE_JSON_PATH` 优先。

## 回滚

共享类型是无持久化 schema 的内部 API。若某一消费者迁移失败,修复该消费者;不得重新复制 env 优先级逻辑。
