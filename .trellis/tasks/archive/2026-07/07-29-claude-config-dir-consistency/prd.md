# CLAUDE_CONFIG_DIR 路径解析统一

## Goal

修复 C7:统一 Claude 配置目录解析,消除设置 `CLAUDE_CONFIG_DIR` 的用户"profile 与 auth 各写各的目录"的失效场景。父任务序列第 2 号——为后续 credentials 加固(统一写路径)打底。

> 2026-07-29 修订:原 C8(`.claude.json` 并发)已拆出为独立任务 `07-29-claude-json-write-strategy`——ccr 侧文件锁无法约束 Claude Code 进程,原"加锁防第三方丢字段"的验收标准不可实现,本任务不再承载。

## 问题(源码已核实)

`CLAUDE_CONFIG_DIR` 会整体改变 Claude Code 的配置目录(官方文档确认,含 `settings.json` 与 `.credentials.json`),但 ccr 三方解析不一致:

| 组件 | 识别 `CLAUDE_CONFIG_DIR`? | 位置 |
|---|---|---|
| `ClaudeAuthService` | 部分 | `claude_auth_service.rs:162-167`(`claude_dir` 识别,但 `.claude.json` 仍回落到 home) |
| `SettingsManager::with_default` | ❌ 只认 `CCR_SETTINGS_PATH` | `crates/ccr-cli/src/managers/settings.rs:72-88`(硬编码 `~/.claude/settings.json`) |
| UI `resolve_config_path` | ❌ | `ccr-ui/src-tauri/src/platform/local.rs:127-144`(硬编码) |

后果:设置了 `CLAUDE_CONFIG_DIR` 时,`profile use` 写错目录的 settings.json,`auth switch` 读写对目录——清理逻辑对不上真正生效的文件,Claude Code 实际读到的 env 与 ccr 认为的完全脱节。

## Requirements

- R1:在 `ccr-config` 提供单一 `ClaudeRuntimePaths` 解析器。`settings.json` 顺序为 `CCR_SETTINGS_PATH`(开发覆盖)> `CLAUDE_CONFIG_DIR/settings.json` > `~/.claude/settings.json`;备份目录顺序为 `CCR_BACKUP_DIR` > `<config_dir>/backups`;凭据固定为 `<config_dir>/.credentials.json`;状态文件顺序为 `CLAUDE_JSON_PATH`(开发覆盖)> `CLAUDE_CONFIG_DIR/.claude.json` > `~/.claude.json`。`SettingsManager`、`ClaudeAuthService`、UI `resolve_config_path` 三处统一调用。
- R2:Windows 路径语义(反斜杠、`%USERPROFILE%` 展开、盘符)在解析函数内统一处理,补 Windows 用例。
- R3:env 读取需可测试注入(避免测试间 `std::env` 竞争,注意仓库 `--test-threads=1` 约定的由来)。
- R4:路径解析权威契约写入 ccr-config backend spec,ccr-cli consumer spec 只链接并记录调用边界;当前不存在 ccr-ui backend spec,不得为单一条款伪造新层,由 Tauri 测试证明消费一致性。

## Acceptance Criteria

- [ ] 设置 `CLAUDE_CONFIG_DIR=<tmp>` 后:`profile use`、`auth switch`、UI 读写、doctor 全部作用于 `<tmp>/settings.json` 与 `<tmp>/.credentials.json`,互相可见。
- [ ] 同一场景下元数据/诊断读取 `<tmp>/.claude.json`;`CLAUDE_JSON_PATH` 显式覆盖仍优先。
- [ ] 未设置时行为与现状完全一致(回归零破坏)。
- [ ] Windows 路径用例通过。
- [ ] `just lint-strict` + `just test` 通过;UI 侧过 `just frontend-check-quick`。
- [ ] 相关 spec 条款同步更新。

## Notes

- 依赖:在 `07-29-claude-authmode-consistency` 之后 start;`07-29-claude-credentials-hardening` 依赖本任务的统一解析函数。
- Planning status:`design.md` 已定夺 ccr-config 归属与注入方式,`implement.md`/JSONL 已就绪;第 1 个子任务完成后 start。
- 2026-07-29 本机 Claude Code `2.1.220` 探针确认:全新 `CLAUDE_CONFIG_DIR` 首次运行会在该目录创建 `.claude.json`,而不是写回默认 home;该证据纳入路径契约。
