# React 侧性能补测（2026-08-24）

对照：`08-22-arch-quality-perf/perf-baseline.md`（Vue，2026-08-23）。
原始日志：scratch `C:\Users\lyh\AppData\Local\Temp\grok-goal-7a92e603621f\implementer\perf-*.log`。

## 场景 1 大表单输入

方法同基线：200 字符，input → 下一帧 rAF。React 选择器改为 `input[name="maxOutputTokens"]` / `input[name="model"]`（Vue 的 `.claude-settings-control` 类名已不存在）。

打包 WebView2（`http://tauri.localhost`，CDP 9222），3 次，RSD ≤ 8.5%：

| 页面 | Vue P50 / P95 | React P50 / P95 |
| --- | --- | --- |
| AppSettingsView | 4.57 / 9.4 | 3.1 / 6.2 |
| ClaudeCodeSettingsView | 4.23 / 9.3 | 3.23 / 6.37 |
| CodexSettingsView | 4.20 / 9.37 | 3.17 / 6.37 |

Web `http://127.0.0.1:15173`：AppSettings P50 6.5 / P95 13.53（RSD 2.7% / 1.9%）。Claude/Codex 无 IPC 时不渲染表单，选择器超时。场景 1 以打包桌面为准。

## 场景 2 列表滚动

web Vite，注入 500 行监控日志，8×600px 滚动：

| 指标 | Vue | React |
| --- | --- | --- |
| 行数 | 500 | 500 |
| fpsMean | 60.09 | 60.09 |
| fpsP95 | 63.43 | 63.7 |
| 掉帧 >33ms / >50ms | 0 / 0 | 0 / 0 |

10k 行仍未达（HistoryList 仍依赖 IPC）。与 Vue 同一替代口径。

## 场景 3 日志流

React 跑在 headless Chromium `--web`（Vite 可 `import('/src/utils/logger.ts')`）。打包产物无该 Vite 模块 URL，不能用同一注入管道。

5 分钟 ×3 + 1 次弃用预热，堆斜率 0 B/s，最终 500 行，fpsMean 57（RSD 0.2%）。Vue 桌面 WebView2 fpsMean 143.10、斜率 8900 B/s。FPS 绝对值不可比（60Hz vsync vs 未限帧 WebView2）。主指标「稳态下流不涨堆」成立。

## 场景 4 图表

打包 `/usage`，CDP。run1 画布未稳定（n=4）。run2 / run3 范围与主题均为 n=20：

| | Vue P50 / P95 | React run2 | React run3 |
| --- | --- | --- | --- |
| 范围切换 | 5.2 / 346.4 | 13.4 / 15.7 | 12.9 / 14.7 |
| 主题切换 | 31.3 / 45.9 | 7.7 / 9.3 | 7.5 / 15.4 |

范围 P50 高于 Vue；范围 P95 与主题耗时低于 Vue。三跑聚合 RSD 被 run1 拉高，不采用。不单开优化任务。

## 场景 5 路由切换

web Vite，29 路由 ×5。聚合 mount P50 5.9 ms（Vue 12.0），settle P50 155.9 ms（Vue 155.3）。settle 与基线同一量级。`/monitoring` settle RSD 20.2%、`/skills` 16%、`/configs` 15.6%，其余多数 ≤15%。与 Vue 快路由 rAF 噪声同类，settle 主指标可用。

## 启动 / 首屏

| | Vue（tauri dev CDP） | React（打包 tauri.localhost） |
| --- | --- | --- |
| DCL `/` `/settings` `/usage` `/configs` | 57 / 53 / 52 / 52 | 55 / 41 / 44 / 44 |
| FCP 同上 | 28 / 32 / 28 / 36 | 48 / 32 / 40 / 36 |

`measure-vite-route.mjs --route=/ --browser`：`serverReadyAndWarmMs=544`（Vue 12101）。本轮依赖缓存已热，不能当冷启动绝对值。

JSON：scratch `perf-startup-desktop.json`；Vite 测量：`measure-vite-route-home.log`。
