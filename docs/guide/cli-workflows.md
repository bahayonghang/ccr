# CLI 工作流

本页按任务组织 CCR 的核心 CLI 路径。当前推荐的 auth/profile 主路径是显式的 Claude Runtime 与 Codex Runtime，而不是旧的“全局当前平台”。

## 工作流 1：初始化并确认运行时状态

```bash
ccr init
ccr current
ccr platform list
```

适用场景：
- 首次初始化 `~/.ccr/`
- 确认 Claude / Codex 当前 runtime 是否就绪
- 查看平台注册表中的已启用平台与 `current_profile`

> `ccr platform list` 现在是注册表兼容视图；实际运行时状态请看 `ccr current`。

## 工作流 2：Claude Runtime / Profile

```bash
ccr add
ccr claude profile list
ccr claude profile switch <name>
ccr claude profile current
ccr claude profile off
```

适用场景：
- 为 Claude Code 写入 `~/.claude/settings.json`
- 在 official auth runtime 与 profile/API-key runtime 之间切换
- 退出 profile mode，回到 Claude official auth runtime

## 工作流 3：Codex Runtime / Profile

```bash
ccr codex auth current
ccr codex profile list
ccr codex profile switch <name>
ccr codex profile current
ccr codex profile off
```

适用场景：
- 保留 Codex official auth 登录态，同时切换第三方 profile
- 在 `~/.codex/config.toml` / `~/.codex/auth.json` 间应用或退出 profile runtime
- 明确区分 `codex auth`（账号）与 `codex profile`（运行时路由）

## 工作流 4：校验与诊断

```bash
ccr current --verbose
ccr validate
ccr doctor
```

当前行为：
- `ccr current --verbose` 显示双 runtime 总览 + registry 目标信息
- `ccr validate` 分别检查 Claude / Codex 的 profile-auth 状态
- `ccr doctor` 默认面向已配置的 Claude / Codex runtime target

## 工作流 5：同步、历史与清理

```bash
ccr history -l 50
ccr sync config
ccr sync all status
ccr clean backups --days 30 --dry-run
```

## 工作流 6：Codex 多账号与 OpenCode 迁移

```bash
ccr codex auth save work
ccr codex auth list
ccr codex auth switch work
ccr opencode auth import-codex --dry-run
```

## 工作流 7：进入图形界面

```bash
ccr ui -p 15173 --backend-port 38081
ccr
```

- `ccr ui`：推荐的图形入口
- `ccr`：默认终端交互入口（TUI）

## 迁移速查表

| 旧命令 | 当前做法 |
|---|---|
| `ccr switch <name>` | `ccr claude profile switch <name>` 或 `ccr codex profile switch <name>` |
| `ccr <name>` | 同上；快捷入口已退休 |
| `ccr platform switch <platform>` | auth/profile 路由已退休；改用显式 runtime/profile 命令 |
| `ccr platform current` | `ccr current` |
| `ccr platform profile ...` | `ccr claude profile ...` / `ccr codex profile ...` |

## 相关页面

- [`快速开始`](/guide/quick-start)
- [`配置模型`](/guide/configuration)
- [`入口选择`](/guide/entrypoints)
- [`命令参考`](/reference/commands/)
