# nextTick 登记（08-22-views-sync-tools）

| 原调用点 | 意图 | 改写 |
| --- | --- | --- |
| `src/views/MonitoringView.vue` `await nextTick()` 后滚到日志底部 | 等 DOM 行插入后再设 `scrollTop` | 改为 `useEffect` 依赖 `filteredLogs.length`，直接读 `ref` 容器滚动。 |

本批次其余文件（Sync / Commands / MCP Manager / tray / SSH / WSL / editor）无 `nextTick`。
