# Web API 参考

本页仅覆盖 `ccr web` 暴露的 legacy HTTP 路由。主图形界面请看 [UI 概览](/guide/ui-overview)。

## 启动方式

```bash
ccr web
ccr web --host 127.0.0.1 --port 19527 --no-browser
```

当前默认值：

- Host：`127.0.0.1`
- Port：`19527`

## 路由分组

### 静态页面

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/` | Legacy 页面入口 |
| GET | `/style.css` | 静态样式 |
| GET | `/script.js` | 静态脚本 |

### 配置管理

| 方法 | 路径 |
|------|------|
| GET | `/api/configs` |
| POST | `/api/switch` |
| POST | `/api/config` |
| GET | `/api/config/{name}` |
| PUT | `/api/config/{name}` |
| DELETE | `/api/config/{name}` |
| PATCH | `/api/config/{name}/enable` |
| PATCH | `/api/config/{name}/disable` |
| POST | `/api/export` |
| POST | `/api/import` |

### Codex 配置

| 方法 | 路径 |
|------|------|
| GET | `/api/codex/profiles` |
| POST | `/api/codex/profiles` |
| PUT | `/api/codex/profiles/{name}` |
| DELETE | `/api/codex/profiles/{name}` |

### 系统与设置

| 方法 | 路径 |
|------|------|
| GET | `/api/history` |
| POST | `/api/validate` |
| POST | `/api/clean` |
| GET | `/api/settings` |
| GET | `/api/settings/backups` |
| POST | `/api/settings/restore` |
| GET | `/api/system` |
| POST | `/api/reload` |

### 统计与成本

| 方法 | 路径 |
|------|------|
| GET | `/api/stats/provider-usage` |
| GET | `/api/stats/cost/summary` |
| GET | `/api/stats/cost/details` |
| GET | `/api/stats/cost/export` |
| GET | `/api/stats/cost/by-model` |
| GET | `/api/budget/status` |
| POST | `/api/budget/set` |
| POST | `/api/budget/reset` |
| GET | `/api/pricing/list` |
| POST | `/api/pricing/set` |
| DELETE | `/api/pricing/remove/{model}` |
| POST | `/api/pricing/reset` |

### 平台与同步

| 方法 | 路径 |
|------|------|
| GET | `/api/platforms` |
| POST | `/api/platforms/switch` |
| GET | `/api/sync/status` |
| POST | `/api/sync/config` |
| POST | `/api/sync/push` |
| POST | `/api/sync/pull` |

## 最小示例

### 获取当前配置列表

```bash
curl http://127.0.0.1:19527/api/configs
```

### 切换平台

```bash
curl -X POST http://127.0.0.1:19527/api/platforms/switch
```

### 获取系统信息

```bash
curl http://127.0.0.1:19527/api/system
```

## 说明

- 本页不记录不存在的 `/api/provider-health/*` 路由。
- 路由事实源以 `crates/ccr/src/web/server.rs` 为准。
- 若你要看“页面怎么用”，回到 [UI 概览](/guide/ui-overview)；若你要看“命令怎么启服务”，回到 [`ccr web`](/reference/commands/web)。
