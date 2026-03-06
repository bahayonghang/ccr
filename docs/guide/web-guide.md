# Web 指南

CCR 有两个与浏览器相关的入口，但它们不是同一等级：

| 命令 | 角色 | 推荐程度 |
|------|------|----------|
| `ccr ui` | 完整图形界面 | 默认推荐 |
| `ccr web` | Legacy 轻量 API / 兼容界面 | 仅在脚本、CI、兼容场景使用 |

## 首选：`ccr ui`

```bash
ccr ui -p 15173 --backend-port 38081
```

适合：
- 日常浏览器管理
- 平台模块导航
- usage / monitoring / skills / provider health 一类可视化能力
- 需要完整模块地图时

更多见 [`UI 概览`](/guide/ui-overview)。

## 次选：`ccr web`

```bash
ccr web --host 127.0.0.1 --port 19527 --no-browser
```

适合：
- 脚本化调用 REST API
- CI / 自动化
- 兼容旧流程

默认行为：
- 默认 host：`127.0.0.1`
- 默认 port：`19527`
- 端口占用时会尝试自动回退

## 如何选择

选择 `ccr ui`：
- 你想在浏览器中把 CCR 当作主界面使用
- 你需要完整模块导航和可视化页面

选择 `ccr web`：
- 你只需要 HTTP API
- 你在 CI、shell、远程机器上运行
- 你在兼容已有脚本

## 相关页面
- [`UI 概览`](/guide/ui-overview)
- [`UI 模块地图`](/guide/ui-modules)
- [`Web API 参考`](/reference/api)
- [`web 命令`](/reference/commands/web)
