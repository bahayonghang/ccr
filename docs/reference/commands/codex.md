# codex - Codex 多账号管理

`ccr codex` 是 Codex 平台的专项命令组，当前重点能力包括 `auth` 与 `sync-history`。

## 用法

```bash
ccr codex
ccr codex auth <ACTION> [OPTIONS]
ccr codex sync-history --provider <ID> [--keep <N>] [--max-age-days <DAYS>] [--dry-run] [--codex-home <PATH>]
ccr codex sync-history status
ccr codex sync-history restore <BACKUP_DIR> [--restore-state]
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
| `export` | 导出账号到加密 JSON |
| `import` | 从 JSON 导入账号（自动检测加密/明文） |

### `ccr codex sync-history`

修复 Codex 在官方 `openai` 与第三方 `custom` 等 provider namespace 间切换后，历史会话在 CLI / App 中不可见的问题。

行为范围：

- 改写 `~/.codex/sessions` 与 `~/.codex/archived_sessions` 中 rollout 首行的 `session_meta.payload.model_provider`
- 同步 `~/.codex/state_5.sqlite` 中 `threads.model_provider`
- 保守补齐 `.codex-global-state.json` 中缺失的侧边栏项目项
- 在 `.codex/backups_state/sync-history/` 下创建可恢复备份
- 默认只处理最近 7 天会话；可用 `--max-age-days` 调整窗口
- 改写 rollout 后保留原始 mtime，避免 Codex Resume 的 `Updated` 排序被批量刷新

支持的子命令：

| 子命令 | 说明 |
|--------|------|
| `sync-history --provider <ID>` | 同步到显式指定的目标 provider；root `model_provider` 缺失时必须传 `--provider` |
| `sync-history --dry-run` | 只预览 rollout / SQLite / sidebar 将修改的数量，不创建备份、不写状态 |
| `sync-history status` | 查看当前 Codex runtime provider、rollout / SQLite 分布、最近 7 天 provider 分布 |
| `sync-history restore <backup-dir>` | 默认只按 manifest 恢复 rollout provider 字段 |
| `sync-history restore <backup-dir> --restore-state` | 同时恢复旧 `state_5.sqlite` 和全局状态，可能覆盖备份后新增线程元数据 |
| `sync-history prune-backups` | 清理旧备份 |

常见示例：

```bash
# 先预览最近 7 天将被改写的会话
ccr codex sync-history --provider custom --dry-run

# URL+Key profile 在 CCR 中的运行时 provider 是 custom
# 想让最近 openai 会话在 URL+Key profile 下可见，用 custom
ccr codex sync-history --provider custom

# 想恢复官方 profile 视图，用 openai
ccr codex sync-history --provider openai

# 查看当前状态
ccr codex sync-history status

# 普通恢复只回滚 rollout manifest 中记录的 provider 字段
ccr codex sync-history restore C:\Users\you\.codex\backups_state\sync-history\20260409T101530123Z

# 需要完整回到旧快照时才恢复 SQLite / 全局状态
ccr codex sync-history restore C:\Users\you\.codex\backups_state\sync-history\20260409T101530123Z --restore-state

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

# 导入导出（包含敏感信息时自动加密）
ccr codex auth export
ccr codex auth export --no-secrets
ccr codex auth import --replace
```

### 导出加密

当导出包含敏感信息（OAuth Token、API Key）时，系统会自动提示设置密码并使用 AES-256-GCM + Argon2id 加密。

**加密方案：**

| 项目 | 说明 |
|------|------|
| 加密算法 | AES-256-GCM（认证加密） |
| 密钥派生 | Argon2id（64MB / 3 iterations / 1 并行度） |
| 导出格式 | JSON 信封：可读头（版本、时间、账号数）+ 加密 payload |
| AAD 保护 | 信封头字段绑定为 GCM 认证数据，防止元数据篡改 |
| 向后兼容 | 导入时自动识别旧版明文文件 |

**导出文件格式（v2.0 加密信封）：**

```json
{
  "version": "2.0",
  "format": "encrypted",
  "exported_at": "2026-04-15T12:00:00Z",
  "account_count": 5,
  "encryption": {
    "algorithm": "aes-256-gcm",
    "kdf": "argon2id",
    "kdf_params": { "m_cost": 65536, "t_cost": 3, "p_cost": 1 },
    "salt": "<base64>",
    "nonce": "<base64>"
  },
  "encrypted_payload": "<base64>"
}
```

无需解密即可读取的信息：导出时间、账号数量、加密参数。
账号的所有敏感数据（Token、API Key）全部在 `encrypted_payload` 中加密保护。

## 将已保存账号迁移到 OpenCode

如果你已经在 CCR 中保存了一批 Codex 账号，并希望让 OpenCode 也能直接切换这些账号，请使用 `opencode` 命令组：

```bash
# 先预览可迁移账号
ccr opencode auth import-codex --dry-run

# 再导入兼容账号
ccr opencode auth import-codex
```

迁移保证：

- 只读取 CCR 已保存的 Codex 账号，不读取未保存的运行时登录态
- 只导入兼容的 ChatGPT OAuth 账号
- 不覆盖已有 OpenCode 账号
- 不切换当前 OpenCode 运行时登录
- 会按原因报告跳过项

## 何时使用

- 一个开发者维护多个 GitHub / Codex 登录身份
- 团队共享机器，需要显式切换账号
- 需要导入导出 Codex 登录状态做迁移或备份（跨设备传输自动加密保护）

## 相关文档

- [`opencode`](./opencode)
- [平台支持](/reference/platforms/)
- [UI 模块地图](/guide/ui-modules)
