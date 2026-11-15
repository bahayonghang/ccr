# `init` - 已弃用的顶层初始化命令

> [!WARNING]
> **已弃用并已隐藏（7.x 兼容）**：顶层 `ccr init [--force]` 在 CCR `7.x` 中**已弃用**且在 `--help` 中**已隐藏**。调用时会向 `stderr` 输出弃用警告（不污染 `stdout`），计划于 `8.0` 移除。

## 规范替代命令

| 场景 | 规范替代命令 |
|---|---|
| 初始化 Claude Profile 目录库并注册平台 | `ccr claude profile init [--json]` |
| 初始化 Codex Profile 目录库并注册平台 | `ccr codex profile init [--json]` |
| 初始化 Grok Profile 目录库并注册平台 | `ccr grok profile init [--json]` |
| 初始化当前仓库的 Git、工作流引擎（`mini-trellis` / `trellis`）与 `ref/` | `ccr project init [--workflow mini-trellis\|trellis]` |

```bash
ccr claude profile init
ccr codex profile init
ccr grok profile init
ccr project init
```

## 关键行为差异

- **按平台独立建库且不覆盖已有配置**：旧 `ccr init` 初始化旧式全局目录（传 `--force` 时覆盖现有配置）；规范命令 `ccr <claude|codex|grok> profile init` 幂等地从内置非激活模板（`current_config = ""`）创建 `~/.ccr/platforms/<platform>/profiles.db` 并登记 `~/.ccr/config.toml`，目录已存在时保持原样不变。
- **不生成驻留 `profiles.toml` 也不改动宿主运行态**：`profile init` 仅建库与注册，不会在平台目录留下活的 `profiles.toml`，也不会修改宿主设置文件；支持 `--json` 输出 `{ ok, platform, profiles_file, created, registered }`。项目级工作流引导由 [`project init`](./project-init) 独立负责。

## 相关页面

- [`claude`](./claude)
- [`codex`](./codex)
- [`grok`](./grok)
- [`project init`](./project-init)
- [`platform`](./platform)
