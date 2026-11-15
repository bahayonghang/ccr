# `current` - 运行时状态总览

`ccr current` 并列展示 Claude Runtime 与 Codex Runtime 的实时状态摘要，不再使用单一全局“当前平台”模型。命令同时提供可见别名 **`status`**（推荐简写）与 **`show`**。

## 用法与可见别名

```bash
ccr current [--verbose | --json]
ccr status [--verbose | --json]
ccr show [--verbose | --json]
```

`--verbose` 与 `--json` 互斥；`ccr status`、`ccr show` 与 `ccr current` 的参数及输出完全一致。如需查看单平台详情（含 Grok），也可分别运行 `ccr claude profile current [--json]`、`ccr codex profile current [--json]` 或 `ccr grok profile current [--json]`。

## 输出模型

默认终端输出：
- Claude Runtime 状态卡片
- Codex Runtime 状态卡片
- 每个平台的当前 Profile、Provider、认证来源与健康度摘要

`--verbose` 额外展示：
- 注册表目标信息
- 各平台目录与宿主配置路径
- 当前 Profile 完整非敏感字段
- 环境变量与设置覆盖诊断细节

`--json` 输出结构：
- `schema_version`
- `generated_at`
- `claude`
- `codex`

> 顶层不再输出历史全局字段 `current_platform`。

## 适用场景

```bash
ccr status
ccr current --verbose
ccr status --json
```

- 快速确认 Claude 与 Codex 各自处于官方登录模式还是第三方 Profile 模式
- 在排查环境覆盖或切换后核对生效路径与认证状态
- 为桌面端、VS Code 扩展或自动化脚本提供统一结构化运行态快照

## 相关页面

- [`claude`](./claude) / [`codex`](./codex) / [`grok`](./grok)
- [`platform`](./platform)
- [`validate`](./validate)
- [迁移指南](/reference/migration)
