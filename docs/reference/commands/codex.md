# codex - Codex 运行时与多账号管理

`ccr codex` 是 Codex 平台的专项命令组，当前重点能力包括：

- `ccr codex auth ...`：official auth 多账号管理
- `ccr codex profile ...`：runtime/profile 路由管理
- `ccr codex sync-history ...`：修复 provider namespace 切换后的历史可见性

## 常用命令

```bash
ccr codex auth current
ccr codex auth list
ccr codex profile list
ccr codex profile switch <name>
ccr codex profile current
ccr codex profile off
```

## `auth` 与 `profile` 的区别

| 命令组 | 作用 |
|---|---|
| `ccr codex auth ...` | 保存、切换、导出、导入 official auth 账号 |
| `ccr codex profile ...` | 把某个 CCR profile 应用到 Codex runtime，或退出回到 official auth runtime |

## `profile` 当前支持面

- `list`
- `current`
- `switch <name>`
- `off`
- `create`
- `set-field`
- `enable`
- `disable`
- `delete`

## `sync-history`

保留原有用途：修复 `openai` / `custom` provider namespace 切换后，旧历史在 Codex CLI / App 中不可见的问题。

常用模式：

```bash
# 保持旧行为：显式写入某个 provider，默认只处理最近 7 天
ccr codex sync-history --provider custom --dry-run
ccr codex sync-history --provider openai

# 新 bridge 模式：把 openai/custom/缺失 provider 历史桥接到当前 runtime provider
ccr codex sync-history --bridge official-custom --dry-run
ccr codex sync-history --bridge official-custom --all-history

# 诊断 provider、SQLite、preview、cwd、Desktop 首屏限制与 encrypted_content
ccr codex sync-history status
```

补充约束：

- `--provider` 继续保持兼容行为；未指定 provider 时仍读取当前 `~/.codex/config.toml`。
- `--bridge official-custom` 会根据当前 runtime 决定目标：官方/隐式 OpenAI 为 `openai`，第三方 profile 为 `custom`。
- `--all-history` 取消 7 天过滤；普通模式默认仍只处理最近 7 天。
- bridge / all-history 的 SQLite 修复默认只碰 `openai`、`custom` 与缺失 provider；需要额外 provider 时使用可重复的 `--include-provider <name>`。
- 写入前会备份 rollout 首行、`state_5.sqlite` 与 `.codex-global-state.json`；`--dry-run` 只输出计划，不写文件。
- `encrypted_content` 只做统计和警告，不解密、不重加密，也不修改消息正文或文件 mtime。
