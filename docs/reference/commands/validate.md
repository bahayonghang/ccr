# `validate` - 跨平台只读配置验证

`ccr validate` 以纯只读方式检查全局注册表以及 Claude、Codex、Grok 三个平台的 SQLite Profile 目录（`profiles.db`）、认证模式与宿主运行态配置文件。命令绝不初始化文件、修复 `current` 指针、创建备份或生成日志目录。

## 用法

```bash
ccr validate
ccr validate --json
```

## 检查范围与确定性顺序

诊断固定按 **`registry -> Claude -> Codex -> Grok`** 顺序收集结果：

- **全局注册表（`registry`）**：`<CCR_ROOT>/config.toml`（默认 `~/.ccr/config.toml`）。
- **平台 Profile 目录**：`<CCR_ROOT>/platforms/<claude|codex|grok>/profiles.db`（由 SQLite 目录库承载；若存在未完成迁移或语法损坏的历史配置，同样以只读方式分类报告而不做写盘迁移）。
- **Claude 运行态**：按 `ClaudeRuntimePaths` 环境变量优先级解析 `settings.json`。
- **Codex 运行态**：按 Codex 配置目录解析 `config.toml` 与凭据状态。
- **Grok 运行态**：`$GROK_HOME/config.toml`（默认 `~/.grok/config.toml`）。

每个平台均调用与 `profile switch` 相同的领域校验器：
- Claude `subscription` 模式不要求 API-key 字段；`api_key` 模式按 Claude 凭据与端点规则校验。
- Codex 按所选 `openai_chatgpt`、`openai_api_key`、`provider_env_key`、`provider_bearer` 或 `no_auth` 模式校验。
- Grok 分别校验官方会话模型选择器与第三方 `api_key` / `env_key` 配置。

非激活的禁用 Profile 可合法保留并报告 `disabled` 警告；但被标记为 `current` 的禁用 Profile 会报告 `disabled` 错误（退出码 `90`）。Claude 按有效目录标记优先于有效注册表标记的顺序选取候选，若标记悬空或冲突但存在有效候选则仅报告 `warning/invalid` 而不写盘修复；显式标记指向不存在的目标且无领域允许的回退候选时返回 `62`。运行态文件检查不依赖 Profile 是否存在，已有运行态 JSON/TOML 损坏或不可读时仍会报告错误。

## `--json` 输出契约（`DiagnosticReport`）

传入 `--json` 时，标准输出打印结构化 `DiagnosticReport` JSON（进程退出码规则与终端文本模式完全一致）：

```json
{
  "checks": [
    {
      "platform": null,
      "target": "registry",
      "severity": "info",
      "category": "valid",
      "message": "registry: valid.",
      "exit_code": 0
    },
    {
      "platform": "claude",
      "target": "profiles",
      "severity": "info",
      "category": "valid",
      "message": "profiles: valid.",
      "exit_code": 0
    }
  ]
}
```

`DiagnosticCheck` 字段说明：
- `platform`：`"claude" | "codex" | "grok" | null`（全局 `registry` 检查项为 `null`）。
- `target`：被检查对象标识（如 `"registry"`、`"profiles"`、`"runtime"` 或具体 Profile 名称）。
- `severity`：`"info" | "warning" | "error"`。
- `category`：`"valid" | "missing" | "corrupt" | "unreadable" | "invalid" | "disabled" | "inactive" | "runtime"`。
- `message`：脱敏诊断说明（绝不包含完整 token、原始配置正文或被拒绝的敏感字段值）。
- `exit_code`：对应 `CcrError` 退出码（`info` 与 `warning` 为 `0`）。

## 分类与退出码

| 状态 | 分类（`category`） | 退出码 |
|---|---|---|
| 全部合法，或仅有警告 | `valid`、`missing`、`inactive`、`disabled` | `0` |
| Profile 领域校验失败或当前激活项处于禁用状态 | `invalid`、`disabled` | `90` |
| Profile 目录语法或字段类型损坏 | `corrupt` | `14` |
| Profile 或运行态文件读取失败（权限拒绝、共享冲突、目录占位等） | `unreadable` | `51` |
| 已存在的运行态 JSON / TOML 语法损坏 | `corrupt` | `40` / `41` |
| 显式 `current` 目标不存在且无合法回退候选 | `invalid` | `62` |

多个错误同时出现时，按 `registry -> Claude -> Codex -> Grok` 顺序返回第一个 `error` 的退出码；仅含 `info` / `warning` 时返回 `0`。`missing` 仅代表路径不存在，权限拒绝或读取异常归类为 `unreadable`（`51`）。

## 相关页面

- [`doctor`](./doctor) — 环境依赖、运行态认证、冲突与可选 `--online` 探活检查
- [`current`](./current) — Claude 与 Codex 运行态总览
- [`claude`](./claude) / [`codex`](./codex) / [`grok`](./grok) — 各平台修复与切换命令（Gemini / Droid 仅保留兼容适配器，无 Profile CLI 表面）
