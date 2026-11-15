# 平台命令迁移指南

本页说明如何从早期全局平台切换与顶层单文件命令迁移到 CCR 7.x 的平台级显式命令与 `profiles.db` 存储模型。

## 核心规则

- CCR **不提供**独立的 `ccr migrate` 子命令；旧版目录在首次运行规范命令时自动完成升级。
- 已移除的 `ccr platform switch|current|info|init|profile` 子命令不再由 Clap 解析；请改用下表列出的规范命令（另见 [`platform` 命令参考](../commands/platform)）。
- 顶层 `ccr init|list|add|delete|enable|disable|clear|optimize|export|import` 在 7.x 中已隐藏并弃用（调用时向 `stderr` 输出警告，计划于 8.0 移除）。
- `Platform::auth_profile_supported()` 仅包含 `claude`、`codex` 与 `grok`。`gemini`（Antigravity CLI）与 `droid`（Factory Droid）为保留兼容适配器（`legacy_adapter`），不提供 CLI `profile` 或 `auth` 子命令。

## 历史命令与 7.x 规范命令对照表

| 历史命令（已移除 / 已退休 / 7.x 隐藏） | 当前 7.x 规范替代命令 | 说明 |
|---|---|---|
| `ccr platform init <platform>` | `ccr <claude\|codex\|grok> profile init` | 从内置模板初始化 `~/.ccr/platforms/<platform>/profiles.db` |
| `ccr platform switch <platform>` | `ccr <claude\|codex\|grok> profile switch <name>` | 目标平台直接由命令前缀显式指定 |
| `ccr platform current` | `ccr current`（别名 `status`、`show`） | 并列查看运行时状态总览 |
| `ccr platform info <platform>` | `ccr platform list [--json]` / `ccr doctor --platform <name>` | 查看注册表条目或平台深度健康状态 |
| `ccr platform profile ...` | `ccr <claude\|codex\|grok> profile <动作>` | 统一归入三平台 `profile` 命令组 |
| `ccr switch <name>` / `ccr <name>` | `ccr <claude\|codex\|grok> profile switch <name>` | 已退休，返回显式迁移提示 |
| `ccr init` | `ccr <claude\|codex\|grok> profile init`（仓库工作流用 `ccr project init`） | 7.x 隐藏兼容命令 |
| `ccr list` | `ccr <claude\|codex\|grok> profile list` | 7.x 隐藏兼容命令 |
| `ccr add` | `ccr <claude\|codex\|grok> profile create <name> ...` 或 `profile open` | 密钥请用 `--auth-token-stdin` / `--api-key-stdin` / `--env-key` |
| `ccr delete` / `enable` / `disable` | `ccr <claude\|codex\|grok> profile delete\|enable\|disable <name>` | 7.x 隐藏兼容命令 |
| `ccr clear` / `ccr optimize` | `ccr <claude\|codex\|grok> profile off` / `profile open` | 退出 Profile 模式或通过编辑器整理投影 |
| `ccr export` | `ccr <claude\|codex\|grok> profile export [--no-secrets] [-o file.toml]` | 导出 `ExchangeV1` TOML |
| `ccr import` | `ccr <claude\|codex\|grok> profile import <file.toml> [--dry-run] [--on-conflict skip\|overwrite\|rename] [--replace -y]` | 默认增量合并（旧 `ccr import` 默认整库替换） |

## 场景 1：并列初始化与管理多平台

```bash
# 初始化三大平台的 SQLite 配置库（profiles.db）
ccr claude profile init
ccr codex profile init
ccr grok profile init

# 按平台独立切换与查看，无需全局切换平台
ccr claude profile switch <claude-profile>
ccr codex profile switch <codex-profile>
ccr grok profile switch <grok-profile>
ccr current
```

> 对于 `gemini`（Antigravity CLI）和 `droid`（Factory Droid），请直接维护其宿主设置文件（或使用 `ccr ui` 的 `/antigravity` 模块），并通过 `ccr doctor --platform gemini|droid` 做只读健康检查。

## 场景 2：通过 `ExchangeV1` 导出、分享与跨环境迁移

使用 `profile export` 导出结构或完整备份，并在目标环境用 `profile import` 预演后导入：

```bash
# 导出脱敏后的 ExchangeV1 TOML（不含明文密钥）
ccr claude profile export --no-secrets -o claude-profiles.toml

# 预演导入结果，再按冲突策略增量合并到目标 profiles.db
ccr claude profile import claude-profiles.toml --dry-run
ccr claude profile import claude-profiles.toml --on-conflict skip

# 导入后通过 stdin 补齐目标环境的真实密钥并校验
printf '%s' "$ANTHROPIC_AUTH_TOKEN" | ccr claude profile edit work --auth-token-stdin
ccr validate
```

注意：
- `ccr <platform> profile import` 默认执行**增量合并**，可通过 `--on-conflict skip|overwrite|rename` 控制同名冲突；仅当显式传入 `--replace -y` 时才会整库替换。
- 跨平台复用配置思路时，模型名、端点与凭据格式需按目标平台规范重新核对。

## 场景 3：从旧版 `profiles.toml` 或 `~/.ccs_config.toml` 升级

- **Unified 旧版 `~/.ccr/platforms/<platform>/profiles.toml`**：首次运行 7.x 命令访问该平台时，CCR 会自动将其迁移为 `profiles.db` 并将原 TOML 备份到 `backups/<platform>/`，无需手工干预。
- **早期单文件 `~/.ccs_config.toml`**：将其视为参考输入，先初始化目标平台配置库，再通过 `profile create` 或 `profile open` 重建所需配置：

```bash
ccr claude profile init
printf '%s' "$ANTHROPIC_AUTH_TOKEN" | ccr claude profile create work \
  --base-url https://api.example.com \
  --auth-token-stdin
ccr claude profile list
```

## 迁移后验收清单

```bash
ccr current --verbose
ccr claude profile list
ccr codex profile list
ccr grok profile list
ccr validate
ccr doctor --all-platforms
```

检查要点：
- 目标平台的 `profiles.db` 已生成且可正常列出 Profile；
- 切换后宿主配置文件写入符合预期；
- `ccr validate` 与 `ccr doctor` 无配置或凭据异常告警。

## 回退与恢复方式

- 切回同平台另一配置：`ccr <claude|codex|grok> profile switch <previous-name>`
- 退出 CCR Profile 路由回到官方登录态：`ccr <claude|codex|grok> profile off`
- 从此前导出的 `ExchangeV1` 快照整库恢复：`ccr <claude|codex|grok> profile import snapshot.toml --replace -y`

## 相关文档

- [平台支持概览](./index)
- [Claude 平台指南](./claude)
- [Codex 平台指南](./codex)
- [Antigravity CLI 平台说明](./gemini)
- [架构说明](/reference/architecture)
