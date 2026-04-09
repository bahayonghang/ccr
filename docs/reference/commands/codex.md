# codex - Codex 多账号管理

`ccr codex` 是 Codex 平台的专项命令组，当前重点能力包括 `auth` 与 `sync-history`。

## 用法

```bash
ccr codex
ccr codex auth <ACTION> [OPTIONS]
ccr codex sync-history [--provider <ID>] [--keep <N>] [--codex-home <PATH>]
ccr codex sync-history status
ccr codex sync-history restore <BACKUP_DIR>
ccr codex sync-history prune-backups [--keep <N>]
```

## 当前支持的子命令

### `ccr codex`

不带子命令时，会进入 Codex 相关的默认交互路径；在启用 TUI 特性时，可作为 Codex 账号管理入口。

### `ccr codex auth`

| 子命令 | 说明 |
|--------|------|
| `save <name>` | 保存当前 `~/.codex/auth.json` 为命名账号 |
| `list` | 列出已保存账号 |
| `switch <name>` | 切换到指定账号 |
| `delete <name>` | 删除已保存账号 |
| `current` | 显示当前账号信息 |
| `export` | 导出账号到 JSON |
| `import` | 从 JSON 导入账号 |

### `ccr codex sync-history`

修复 Codex 在官方 `openai` 与第三方 `custom` 等 provider namespace 间切换后，历史会话在 CLI / App 中不可见的问题。

行为范围：

- 改写 `~/.codex/sessions` 与 `~/.codex/archived_sessions` 中 rollout 首行的 `session_meta.payload.model_provider`
- 同步 `~/.codex/state_5.sqlite` 中 `threads.model_provider`
- 保守补齐 `.codex-global-state.json` 中缺失的侧边栏项目项
- 在 `.codex/backups_state/sync-history/` 下创建可恢复备份

支持的子命令：

| 子命令 | 说明 |
|--------|------|
| `sync-history` | 同步到当前 root `model_provider`，或 `--provider` 指定的目标 provider |
| `sync-history status` | 查看当前 provider 与 rollout / SQLite 分布 |
| `sync-history restore <backup-dir>` | 从指定备份恢复 |
| `sync-history prune-backups` | 清理旧备份 |

常见示例：

```bash
# 同步到当前 ~/.codex/config.toml 的根级 model_provider
ccr codex sync-history

# 显式同步到 custom
ccr codex sync-history --provider custom

# 查看当前状态
ccr codex sync-history status

# 恢复某次备份
ccr codex sync-history restore C:\Users\you\.codex\backups_state\sync-history\20260409T101530123Z

# 只保留最近 3 份备份
ccr codex sync-history prune-backups --keep 3
```

## 常见示例

```bash
# 保存当前登录
ccr codex auth save work

# 带描述和到期时间
ccr codex auth save personal -d "Personal GitHub account" --expires-at 2026-02-01T00:00:00Z

# 查看与切换
ccr codex auth list
ccr codex auth switch work
ccr codex auth current

# 导入导出
ccr codex auth export --no-secrets
ccr codex auth import --replace
```

## 何时使用

- 一个开发者维护多个 GitHub / Codex 登录身份
- 团队共享机器，需要显式切换账号
- 需要导入导出 Codex 登录状态做迁移或备份

## 相关文档

- [平台支持](/reference/platforms/)
- [UI 模块地图](/guide/ui-modules)
