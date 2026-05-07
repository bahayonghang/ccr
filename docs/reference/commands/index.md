# 命令总览

CCR 当前 CLI 主要分成五组：运行时总览、平台级 profile/auth、数据与同步、诊断与界面、扩展与维护。

## 推荐起步顺序

```bash
ccr init
ccr current
ccr add
ccr claude profile list
ccr claude profile switch <name>
ccr validate
```

## 当前主路径

| 路径 | 说明 |
|---|---|
| [`current`](./current) | 显示 Claude Runtime / Codex Runtime 双总览 |
| [`codex`](./codex) | Codex auth、profile、sync-history |
| `ccr claude profile ...` | Claude runtime/profile 路由 |
| [`platform`](./platform) | 注册表兼容视图（主要保留 `list`） |
| [`validate`](./validate) / [`doctor`](./doctor) | 基于显式 runtime 模型做校验与体检 |

## 迁移速查表

| 旧命令 | 当前路径 |
|---|---|
| `ccr switch <name>` | `ccr claude profile switch <name>` 或 `ccr codex profile switch <name>` |
| `ccr <name>` | 快捷入口已退休 |
| `ccr platform switch <platform>` | 已退休 |
| `ccr platform current` | `ccr current` |
| `ccr platform profile ...` | `ccr claude profile ...` / `ccr codex profile ...` |

## 相关文档

- [CLI 工作流](/guide/cli-workflows)
- [配置模型](/guide/configuration)
- [迁移指南](/reference/migration)
