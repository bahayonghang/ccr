# Usage Dashboard 浏览器测试桩 —— Harness Notes

- **产物**: `tauri-shim.js`（同目录）+ 本文档
- **用途**: Playwright `page.addInitScript()` 在 app bundle 之前注入，伪造 Tauri v2 IPC，使 Usage Dashboard 在纯浏览器（vite dev，无 Tauri）里完整运行
- **契约来源**: `@tauri-apps/api` v2.11.0（`node_modules/@tauri-apps/api/{core,event,window}.js`）
- **验证**: `node --check tauri-shim.js` 通过；见文末“自测结果”
- **日期基准**: 所有日期止于 `2026-07-07`（确定性硬编码）
- **变体**: `localStorage['ccr-usage-fixture-variant']` = `'healthy'`（默认）| `'stale'`

---

## (a) 挂载 / 筛选时的命令调用清单与顺序

直接导航到 `/usage` 时，按时间线触发（`[事件]`=事件插件注册，`[桩]`=显式桩，`[null]`=惰性兜底）：

### 应用外壳启动（usage store 之前）

| 顺序 | 触发点                                                                                              | 命令                                                               | 处理     |
| ---- | --------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------ | -------- |
| 1    | `main.ts` → `showCurrentWindowIfTauri()` → `getCurrentWindow()`                                     | 读 `__TAURI_INTERNALS__.metadata.currentWindow.label`（无 invoke） | metadata |
| 2    | `main.ts` → `win.show()`                                                                            | `plugin:window                                                     | show`    | `[null]` |
| 3    | `App.vue` onMounted → `win.listen('shell:navigate')`                                                | `plugin:event                                                      | listen`  | `[事件]` |
| 4    | `MainLayout` → `useMainLayoutShell` onMounted → `shellPreferencesStore.hydrateRuntimePreferences()` | `shell_get_preferences`                                            | `[桩]`   |
| 5    | `BackendStatusBanner` onMounted → `useBackendHealth().resume()`（轮询，`intervalMs` 30s/300s）      | `health_check`                                                     | `[桩]`   |
| 6    | `EnvironmentSwitcher`（`v-if=isTauri && !isMobileSidebar`）onMounted → `fetchEnvironments()`        | `list_environments`                                                | `[桩]`   |

> `EnvironmentSwitcher` 只在桌面视口渲染（`window.matchMedia('(max-width: 1023px)')` 为 false）。Playwright 默认视口 1280×720 会渲染它。

### Usage Dashboard（`useUsageDashboardState` onMounted → `store.initializeDashboard`）

| 顺序 | 调用链                                                                                   | 命令                        | 处理    |
| ---- | ---------------------------------------------------------------------------------------- | --------------------------- | ------- |
| 7    | `fetchAll` → `ensureUsageSnapshotListener()` → `listen('usage:snapshot-updated')`        | `plugin:event               | listen` | `[事件]` |
| 8    | `fetchAll` → `refreshUsageCapabilities()`                                                | `get_usage_capabilities_v2` | `[桩]`  |
| 9    | `fetchAll`（`USE_DASHBOARD_API` 默认 true）                                              | `get_usage_dashboard_v2`    | `[桩]`  |
| 10   | `fetchAll` → `scheduleHeatmapLoad`（`LAZY_HEATMAP_LOAD` 默认 true，requestIdleCallback） | `get_usage_heatmap_v2`      | `[桩]`  |

- **`get_usage_dashboard_v2` 参数形状**（camelCase，来自 `api/domains/stats.ts`）：
  `{ platform, provider, startDate, endDate, heatmapDays: 365, includeHeatmap: false }`
  桩按 `args.includeHeatmap` 决定 `heatmap` 字段：默认 false → 返回 `heatmap: null`，热力图单独走命令 10。
- **`get_usage_heatmap_v2` 参数**：`{ platform, days: 365 }`（`HEATMAP_DAYS = 365`）。
- **无自举导入**：`initializeDashboard` 在 `summary.total_requests > 0` 时提前返回。fixture `total_requests = 48000`，因此**不会**触发 `start_usage_import_job_v2` 及其事件流。
- **自动刷新**：`startDashboardAutoRefresh({ immediate: false })` 不立即请求；此后每 30s（`REFRESH_INTERVAL`）重复命令 9（`reason: 'auto-refresh-core'`，`includeHeatmap: false`）。

### 筛选变更（平台 / 时间范围）

- `onFilterChange` → `store.setFilters()` → **300ms 防抖**（`FILTER_DEBOUNCE_MS`）→ `fetchAll({ reason: 'filter', includeHeatmap: false })` → 命令 9（+ 惰性命令 10）。
- 若 `activeTab === 'logs'`，额外 `loadLogs('reset')`（见 (c)）。

### 非聚合回退路径（仅当 `VITE_USAGE_DASHBOARD_AGGREGATED_API=0`，默认关闭）

此时命令 9 被替换为并发 4 条：`get_usage_summary_v2` / `get_usage_trends_v2` / `get_usage_by_model_v2` / `get_usage_by_project_v2`。四者的桩已全部就绪。`get_usage_by_provider_v2` 由 `store.fetchProviderStats()` 提供，聚合路径下 provider_stats 来自命令 9，启动时不单独调用；桩仍已备好以防手动触发。

---

## (b) 驱动 ops-cockpit staleness / degraded 呈现的 fixture 字段

`views/usage/usageOpsCockpit.ts` `buildUsageOpsCockpit` 的读取优先级（**snapshot 优先于 archive**）：

```
readiness     = snapshot.readiness   ?? archive.readiness
freshness     = snapshot.freshness   ?? archive.freshness
source_health = snapshot.source_health ?? archive.source_health
state         = (importing || loading) ? 'syncing' : (readiness.state ?? 'empty')
```

数据落定后 `importing`/`loading` 均为 false，因此**顶层座舱状态直接等于 `readiness.state`**。各呈现元素的控制字段：

| 呈现元素                                          | 驱动字段                                                                             | healthy                                | stale                                         |
| ------------------------------------------------- | ------------------------------------------------------------------------------------ | -------------------------------------- | --------------------------------------------- |
| 座舱横幅 title/tone（“Usage data is stale” 警告） | `readiness.state`                                                                    | `ready`（tone success）                | `stale`（tone warning）                       |
| Readiness 健康项 value / 主按钮                   | `readiness.state` + `readiness.next_action`                                          | `ready` / `null` → 无按钮              | `stale` / `refresh_usage` → “Refresh usage”   |
| Freshness 健康项 value/tone                       | `freshness.state`                                                                    | `fresh`（success）                     | `stale`（warning）                            |
| Freshness 明细（“{time} · {n}d ago”）             | `freshness.latest_completed_at` + `freshness.age_seconds`                            | `2026-07-07T09:30:00Z` / `1800`（30m） | `2026-07-03T09:30:00Z` / `345600`（4d）       |
| Source health 健康项 **L/M/D 在档/缺失/已删**     | `archive.live_sources` / `missing_sources` / `deleted_sources`                       | `2053 / 2829 / 0`                      | `2053 / 2829 / 0`（**两变体相同**）           |
| Source health 健康项 tone                         | `archive.missing_sources>0 \|\| deleted_sources>0` → warning                         | warning（missing=2829）                | warning                                       |
| Source 徽章（`sourceItems`）逐源 tone             | 每个 `source_health[].state`（`live`→success，`degraded`→warning，`missing`→danger） | 全 `live`                              | `claude:live, codex:degraded, gemini:missing` |
| Snapshot cache 健康项（“Generated {time}”）       | `snapshot.generated_at`                                                              | `2026-07-07T10:00:00Z`                 | `2026-07-03T10:00:00Z`                        |
| Drilldown 健康项                                  | `snapshot.drilldown.dimensions`                                                      | `[platform, model, project, source]`   | 同左                                          |

**关键结论：staleness 完全由服务端（即 fixture）提供的枚举字段驱动，前端不重算。** `formatDateTime` 只做 `new Date(v).toLocaleString(locale)`；`formatAge` 只格式化给定的 `age_seconds`；`usageDiagnostics.ts` 用 `logsRecords[0].recorded_at` 也只 `toLocaleString`。因此时间戳硬编码安全、确定性。两变体的 L/M/D 计数刻意保持一致，只切换 `readiness.state` / `freshness.state` / 逐源 `source_health[].state` / `generated_at`。

> 精确匹配任务描述：healthy = 在档 2053 / 缺失 2829 / 已删 0 且所有 source `live`；stale = 相同计数 + `readiness.state='stale'` + `freshness.state='stale'`（age 4 天） + 逐源降级（codex degraded / gemini missing） + `generated_at` 早 4 天。

---

## (c) 日志分页调用

- **命令**：`get_usage_logs_v2`，参数 `{ query: UsageLogsQuery }`。
- **query 形状**（`stores/usageDashboardPayload.ts` `buildUsageLogsQuery`）：
  `{ platform, model, start_date, end_date, page, page_size, cursor, include_total, mode: 'cursor' }`。
- **触发**：
  - 切到 logs 标签 / 筛选变更 → `loadLogs('reset')` → `fetchLogs('reset')`：`page=1, cursor=undefined, include_total=true`。
  - `nextLogsPage()` → `fetchLogs('next')`：`cursor = 上一页 next_cursor`，`include_total=false`。
  - `prevLogsPage()` → `fetchLogs('prev')`：`cursor` 取自 `logsCursorStack`。
- **桩行为**：cursor 编码 offset（首页 `cursor=undefined`→offset 0），每页固定 **20** 条，`next_cursor = String(offset+20)`（无更多则 `null`），首页 `include_total=true` → `total=60`。产出 **3 页 × 20，total 60**，与 `normalizePaginatedLogs` 一致（`mode: 'cursor'`，翻页靠 `next_cursor`）。
- **model 过滤**：`query.model` 存在时先按 `record.model === query.model` 过滤再分页并重算 `total`（如 `gpt-5` → total 12）。可用于练习 logs-filter 骨架。

> 注意：store 的 `logsPageSize` 默认 50，桩固定返回 `page_size: 20`。cursor 模式的 `canNextLogs` 只看 `next_cursor`，翻页正常到 3 页；`logsTotalPages`（`ceil(60/50)=2`）仅在 offset 模式显示时才相关，此处不影响。

---

## (d) 以非 null 形状打桩的命令及原因

| 命令                                                                                          | 返回形状                                                                                                        | 原因（消费方会解构 / 遍历，null 会崩）                                                                                                                                           |
| --------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `list_environments` / `refresh_environments`                                                  | `[{id,name,env_type,is_active,description}]`                                                                    | `EnvironmentSwitcher.vue`：`environments.value = await listEnvironments()` 后 `computed(() => environments.value.find(e => e.is_active))`；返回 null 会在渲染时 `null.find` 抛错 |
| `get_current_environment`                                                                     | 同上单对象                                                                                                      | 部分消费方读 `env.env_type`/`name`                                                                                                                                               |
| `health_check`                                                                                | `{ status: 'healthy', database: true }`                                                                         | `useBackendHealth`：`result.status === 'healthy' && result.database !== false`；healthy 使红色降级横幅（`BackendStatusBanner`）不显示                                            |
| `shell_get_preferences`                                                                       | `{ confirm_before_exit, close_to_tray, open_panel_on_tray_click, tray_panel:{placement_mode,manual_position} }` | `hydrateRuntimePreferences` 读 `preferences.confirm_before_exit` 等；虽有 try/catch 兜底，打桩避免走 catch 分支                                                                  |
| `get_usage_capabilities_v2`                                                                   | `CapabilityReport`（全 `supported:true`，`schema_version:14`）                                                  | store 读 `features.overview/sync_json_events/provider_breakdown`；`overview.supported=false` 会切到“unsupported”面板                                                             |
| `get_usage_dashboard_v2`                                                                      | `UsageDashboardResponse`（见 (b)）                                                                              | 主数据源，`applyDashboardPayload` 全字段解构                                                                                                                                     |
| `get_usage_heatmap_v2`                                                                        | `{ data: { 'YYYY-MM-DD': n } }` × 365                                                                           | `heatmap.value.data` 被遍历                                                                                                                                                      |
| `get_usage_logs_v2`                                                                           | `PaginatedLogsDto`（见 (c)）                                                                                    | `normalizePaginatedLogs` 解构 `records/total/next_cursor/mode`                                                                                                                   |
| `get_usage_summary_v2` / `_trends_v2` / `_by_model_v2` / `_by_project_v2` / `_by_provider_v2` | 对应 DTO / DTO[]                                                                                                | 非聚合回退路径与 `fetchProviderStats` 用                                                                                                                                         |

事件插件命令（`plugin:event|listen` / `unlisten`）：注册/注销监听器并返回数字 id（事件永不真正 emit，但调用不能 reject）。其余 `plugin:*`（`window|show` 等）与未知命令走惰性兜底 `Promise.resolve(null)`。

---

## (e) 已知局限

1. **仅覆盖 `/usage` 页面**。惰性兜底对所有未打桩命令返回 `null`；若测试导航到 Claude/Codex/CheckIn/Config 等页面，那些页面解构 null 会崩。要扩展需按需补桩。
2. **事件不真正触发**。没有 `usage:snapshot-updated`、`usage:job-*` 事件流；导入任务链路（`start_usage_import_job_v2` + 事件）未被练习——但因 fixture 已有数据，本就不触发。
3. **时间戳相对 `2026-07-07` 硬编码**。座舱不按 `Date.now()` 重算 staleness，故安全且确定；但 stale 的“4 天前”只相对 2026-07-07 成立。若将来有代码把 `generated_at` 与 `Date.now()` 比较，需改为 `Date.now() - offset` 运行时计算。
4. **`page_size` 不一致（显示细节）**。store 请求 50，桩固定服务 20/页；cursor 翻页正常，仅 offset 模式的 `logsTotalPages` 显示值不同（见 (c)）。
5. **provider stats 双路径**。聚合路径 provider_stats 来自 `get_usage_dashboard_v2`；`get_usage_by_provider_v2` 桩已备但启动不命中。
6. **`transformCallback` id 为单调自增**（非 crypto 随机），对测试无影响。
7. **无法确信是否需打桩的命令**：无。usage 仪表盘启动/筛选/日志路径涉及的命令均已显式打桩或经事件插件处理；其余启动命令（`shell_get_preferences`/`health_check`/`list_environments`）已按解构需求给非 null 形状。若后续把 `EnvironmentSwitcher` 之外的启动组件纳入，需重新核对。

---

## 自测结果（`node` 内跑 fixture 逻辑）

两变体均通过：

- `total_tokens = 12527400000`（格式化应为 `12.53B`）；`total_cost_usd = 26114.04` == model_stats cost 合计；`total_requests = 48000` == model_stats request 合计；model_stats token 合计 == `total_tokens`（三者完全自洽）。
- trends 30 天 `2026-06-08 … 2026-07-07`；model_stats 5 / project_stats 6 / provider_stats 3 / source_stats 3。
- dashboard `heatmap = null`（includeHeatmap=false）；`get_usage_heatmap_v2` 返回 365 天。
- archive L/M/D `2053/2829/0`（两变体一致）。
- healthy：readiness `ready` / freshness `fresh` / source 全 `live` / generated_at `2026-07-07`。
- stale：readiness `stale` / freshness `stale`（age 345600）/ source `claude:live, codex:degraded, gemini:missing` / generated_at `2026-07-03`。
- logs：p1 20 条 total 60 next `20`；p2 20 条 next `40`；p3 20 条 next `null`；model=`gpt-5` → total 12 全匹配。
- `list_environments` 为数组且 active=`local`；`health_check` healthy；capabilities `overview.supported=true` schema 14；`plugin:event|listen`→id，`unlisten`→null，未知命令→null；`__TAURI__`/`__TAURI_INTERNALS__` 均在 window 上，`metadata.currentWindow.label='main'`。
