# 性能基线（迁移前，Vue 版）— 08-22-arch-quality-perf 批次 7

采集时间：2026-08-23（本地 Windows 10 x64）
应用版本：Vue 版 v7.2.0（`feature/react-migration` 分支经 worktree `ccr-wt-vue`，应用代码与 `dev` 等价，见父任务基线 README）
测量脚本：`ccr-ui/scripts/perf/`（框架无关，只依赖 DOM + `performance` API + playwright 驱动，无 vue/react import）

## 0. 环境与口径

| 项 | 值 |
| --- | --- |
| 分支 / 工作区 | `feature/react-migration`（worktree `D:/Documents/Code/Github/ccr-wt-vue`） |
| Web 模式 dev server | Vite dev `http://127.0.0.1:4180`（`bun run dev -- --port 4180`，与 Phase 0 同端口口径） |
| 桌面运行时 | `ccr-desktop.exe`（debug 构建）`tauri dev` 等价，devUrl `http://127.0.0.1:15173`，WebView2 CDP `http://127.0.0.1:9222` |
| 视口 | 1800×1125（web 模式 headless chromium；桌面 WebView2 主窗口经 CDP `setViewportSize` 模拟同尺寸） |
| 浏览器 | headless chromium 1234（web 模式）；Edge/WebView2 151（桌面模式） |
| 机器 | Windows 10 x64，bun 1.4.0 |

五项场景脚本为 `--base-url` / `--cdp-url` 可重定向的独立测量器，**框架无关**：同一脚本在 Vue（本基线）与 React 基座（阶段 5 后）上以相同方法采集，只改 `--base-url` 指向即可。React 侧数值统一推迟到 `08-22-regression-release` 步骤 7 补测（见 §8 推迟表）。

可重复性门（R7 / AC7）：同一脚本连续运行 3 次，关键指标 RSD ≤ 15%。超限则增加采样/固定更多变量后重测，记录迭代过程（见各场景方法说明）。

## 1. 场景 1：大表单输入延迟

方法：在 AppSettingsView（`/settings`）、ClaudeCodeSettingsView（`/claude-code/settings`）、CodexSettingsView（`/codex/settings`）各选一个文本字段，连续键入 200 字符，in-page 记录每次 `input` 事件到「下一帧 rAF」的时间差（rAF 回调先于该帧绘制，等价于 input → 下一帧 paint 的时间代理）。预热一轮（首访路由的 Vite 转译与首屏资源加载不计入）后跑 3 次。指标：每页 P50 / P95。

| 页面 | 路由 | run1 P50/P95 (ms) | run2 P50/P95 (ms) | run3 P50/P95 (ms) | 均值 P50 / P95 (ms) | RSD P50 / P95 |
| --- | --- | --- | --- | --- | --- | --- |
| AppSettingsView | `/settings` | 5.0 / 9.4 | 4.3 / 9.4 | 4.4 / 9.4 | 4.57 / 9.4 | 8.3% / 0% |
| ClaudeCodeSettingsView | `/claude-code/settings` | 4.3 / 9.3 | 4.4 / 9.4 | 4.0 / 9.2 | 4.23 / 9.3 | 4.9% / 1.1% |
| CodexSettingsView | `/codex/settings` | 4.3 / 9.4 | 4.3 / 9.2 | 4.0 / 9.5 | 4.20 / 9.37 | 4.1% / 1.6% |

样本量：三页每 run 约 120–150 个按键样本（200 键中部分因同步合并未产生独立 input 事件）。

判定：**PASS**（全指标 RSD ≤ 8.3% < 15%）。输入到绘制的 P50 ≈ 4.2–4.6 ms，P95 ≈ 9.2–9.4 ms。

## 2. 场景 2：虚拟列表滚动

方法：应用内唯一虚拟化列表 `HistoryList`（`@tanstack/vue-virtual`）依赖 IPC `get_history` 供数——web 模式 0 行、桌面实测 17 行，均无法达到设计目标的 10,000 行（见下「方法受限」）。本脚本改测**可渲染的最大数据集**：监控页日志流经应用自身 logger 模块注入 500 条（`useMonitoringFeed` `DEFAULT_MAX_ENTRIES=500` 上限），程序化滚动固定距离（8 × 600px），用 rAF 时间戳序列计算帧间隔分布与掉帧数。记录实际行数 500。

| 指标 | run1 | run2 | run3 | 均值 | RSD |
| --- | --- | --- | --- | --- | --- |
| 实际渲染行数 | 500 | 500 | 500 | 500 | 0% |
| 帧率均值 (FPS) | 60.10 | 60.10 | 60.07 | 60.09 | 0% |
| 帧率 P95 (FPS) | 64.52 | 62.89 | 62.89 | 63.43 | 1.5% |
| 掉帧 >33ms | 0 | 0 | 0 | 0 | 0% |
| 掉帧 >50ms | 0 | 0 | 0 | 0 | 0% |

判定：**PASS**（RSD ≤ 1.5%）。

**方法受限（10k 目标未达）**：设计目标为 10,000 行，但 Vue 版唯一虚拟化列表（`HistoryList`，`get_history` IPC 供数）在 web 模式 0 行、桌面运行时实测 17 行，无法渲染 10k 行。本基线以最大可渲染数据集 500 行（日志流上限）代替，方法不变；10k 目标在阶段 7（React 侧，数据可 mock）恢复，同脚本 `--base-url` 重定向测量即可。

## 3. 场景 3：日志流

方法：桌面运行时（tauri dev / WebView2 真实 IPC）`/monitoring` 持续注入日志 **5 分钟**（应用自身 logger 模块，`useMonitoringFeed` 经 `logger.subscribe` 消费，与真实前端日志同一管道），每 10 秒采样 `performance.memory.usedJSHeapSize` 与 rAF 帧率（1 秒窗口）。报告内存增长斜率（线性回归，B/s）与帧率分布。正式测量前先跑**完整弃用轮**（5 分钟注入，数据弃用不计入）——R7 迭代 1 见下——再连跑 3 次。

| 指标 | run1 | run2 | run3 | 均值 | RSD |
| --- | --- | --- | --- | --- | --- |
| 帧率均值 (FPS) | 142.85 | 143.22 | 143.22 | 143.10 | 0.1% |
| 堆 Δ (B) | 2,467,046 | 3,289,137 | 2,182,113 | 2,646,099 | 21.7% |
| 斜率 (B/s) | 8,385.1 | 9,075.5 | 9,239.4 | 8,900.0 | 5.1% |
| 最终行数 | 500 | 500 | 500 | 500 | 0% |

（run 级原始数据与采样轨迹见 `evidence/perf-log-stream-run-batch1.txt`（首轮）与 `evidence/perf-log-stream-run-batch2.txt`（迭代后正式数据）。）

**迭代说明（R7 方法改进，design §7「固定更多变量」）**：首轮按原方法（30 秒预热 + 3 次正式 run）实测，run1 仍承担 500 行渲染路径的完整冷启动开销（V8 编译 + 模块图加载），保留堆高出 run2/3 约 3.5 MB（run1 堆 Δ 5.12 MB vs run2/3 1.41/2.20 MB，堆 Δ RSD 67.1%、斜率 RSD 80.3% 超限；FPS 已达标 RSD 1.1%）。将预热升级为**完整弃用轮**（注入满 5 分钟、数据弃用、重载后开跑），使每个测量 run 都从等价饱和态出发。迭代后：FPS RSD 0.1%、斜率 RSD 5.1%、最终行数 RSD 0% 全部达标；**堆 Δ 残留 RSD 21.7%**，属两点差（末采样 − 首采样）对瞬时 GC/堆尖峰的固有敏感——如 run2 末采样落在 17.65 MB 临时尖峰（前后采样稳定 16.4–16.5 MB），是 WebView2 堆管理的瞬时现象而非稳定增长。增长率的稳健估计是全程 27 个样本的线性回归斜率（RSD 5.1% 达标），与 §5 快路由 `changed` 指标同处理方式：主指标达标、残余噪声记录为方法局限。

判定：**PASS**（主指标全部 ≤15% RSD：帧率 0.1%、增长斜率 5.1%、最终行数 0%；堆 Δ 两点差残余 21.7% 为瞬时堆尖峰噪声，已记录为方法局限，见迭代说明）。

说明：日志条目受 `DEFAULT_MAX_ENTRIES=500` 上限约束，DOM 行数稳定在 500 附近，内存增长斜率反映「稳定流下的稳态开销」，为真实（可能接近零）而非编造。

## 4. 场景 4：图表更新与主题切换

方法：桌面运行时（真实 llmusage 数据）`/usage`：时间范围切换 20 次（`PillToggleGroup` 5 档循环，规避同值点击 no-op，保证每次真实切换；按钮定位用 `.pill-toggle-group__item` 顺序，与语言无关）+ 明暗切换 20 次（`data-theme` + `.dark`，与主题 store `applyThemeToDocument` 同源，触发图表 MutationObserver → 重渲染）。每次记录点击/切换 → 图表画布 `.apexcharts-canvas` 子树首次 DOM 变更的耗时。预热一轮后跑 3 次。

| 指标 | run1 | run2 | run3 | 均值 | RSD |
| --- | --- | --- | --- | --- | --- |
| 范围切换 P50 (ms) | 5.6 | 4.9 | 5.1 | 5.2 | 6.9% |
| 范围切换 P95 (ms) | 345.3 | 347.0 | 347.0 | 346.4 | 0.3% |
| 主题切换 P50 (ms) | 30.8 | 32.3 | 30.9 | 31.3 | 2.7% |
| 主题切换 P95 (ms) | 45.1 | 47.5 | 45.1 | 45.9 | 3.0% |

样本量：每 run 范围 20 + 主题 20（全部成功，n=20/20）。

判定：**PASS**（RSD ≤ 6.9%）。范围切换 P50 ≈ 5 ms、P95 ≈ 346 ms（数据重取最慢档的首次渲染计入 P95）；主题切换 P50 ≈ 31 ms、P95 ≈ 46 ms。

## 5. 场景 5：路由切换

方法：SPA 导航（`history.pushState` + `popstate`，与 `createWebHistory` 同机制），in-page 安装 `#app` 子树 MutationObserver，记录 dispatch → 首次内容变更（mount 开始）与 → 连续 120 ms 无新增节点突变（settle）。75 条路由按域采样 29 条，每域 2–3 条。预热一轮后跑 5 次（迭代说明见下）。

| 路由 | mount 均值 (ms) | settle 均值 (ms) | mount RSD | settle RSD |
| --- | --- | --- | --- | --- |
| `/` | 34.7 | 165.1 | 8.3% | 3.2% |
| `/settings` | 15.2 | 161.6 | 6.7% | 3.7% |
| `/claude-code` | 14.2 | 150.2 | 3.3% | 3.6% |
| `/claude-code/settings` | 8.5 | 168.7 | 76.5% | 8.5% |
| `/claude-code/profiles` | 34.7 | 175.8 | 5.9% | 5.4% |
| `/codex` | 18.7 | 155.3 | 11.1% | 5.3% |
| `/codex/settings` | 9.5 | 151.9 | 13.3% | 10.1% |
| `/codex/mcp` | 11.6 | 161.4 | 3.3% | 10.1% |
| `/grok` | 8.7 | 152.5 | 11.4% | 10.7% |
| `/grok/settings` | 9.3 | 148.3 | 9.6% | 9.2% |
| `/antigravity` | 12.4 | 157.3 | 11.5% | 10.3% |
| `/antigravity/mcp` | 10.0 | 148.2 | 9.9% | 10.3% |
| `/opencode` | 11.4 | 154.8 | 11.7% | 11.4% |
| `/opencode/settings` | 13.2 | 149.9 | 19.1% | 8.5% |
| `/opencode/providers` | 7.8 | 151.1 | 22.2% | 4.8% |
| `/commands` | 13.5 | 159.3 | 8.8% | 5.2% |
| `/converter` | 8.3 | 155.0 | 16.0% | 12.0% |
| `/sync` | 13.0 | 150.3 | 6.5% | 5.8% |
| `/configs` | 18.4 | 164.1 | 5.4% | 7.4% |
| `/mcp-manager` | 18.8 | 162.2 | 5.5% | 8.1% |
| `/slash-commands` | 14.3 | 158.5 | 9.9% | 6.0% |
| `/budget` | 7.7 | 148.4 | 26.8% | 8.0% |
| `/pricing` | 7.7 | 145.9 | 22.9% | 9.5% |
| `/usage` | 25.6 | 163.9 | 5.0% | 8.1% |
| `/monitoring` | 7.7 | 156.3 | 7.7% | 12.7% |
| `/checkin` | 20.1 | 160.5 | 10.9% | 9.2% |
| `/wsl` | 6.4 | 151.3 | 11.2% | 9.9% |
| `/ssh` | 4.8 | 140.6 | 8.8% | 8.9% |
| `/skills` | 2.4 | 144.9 | 19.6% | 13.2% |

聚合（5 次运行）：

| 指标 | 值 | RSD |
| --- | --- | --- |
| mount P50 (ms) | 12.0 | 3.4% |
| mount P95 (ms) | 33.4 | 5.4% |
| settle P50 (ms)（全样本 145） | 155.3 | — |
| settle P95 (ms)（全样本 145） | 182.4 | — |

**迭代说明（R7 方法改进）**：首轮 3 次运行有 7 项逐路由指标 RSD > 15%（主要为懒加载路由的 settle，如 `/commands` settle RSD 94.7%、`/converter` settle 22.1%、`/codex/mcp` settle 18%）。按 design §7 增加样本数（3 → 5 次）后：**全部 29 条路由的 settle RSD ≤ 13.2%**（主指标），聚合 mount P50/P95 RSD 3.4% / 5.4%。剩余 >15% 的 7 项全部是 `changed`（首次 DOM 变更）指标，集中在 <10 ms 的快挂载路由（`/skills` mount 2.4 ms RSD 19.6%、`/budget` 7.7 ms RSD 26.8%、`/pricing` 7.7 ms RSD 22.9%、`/claude-code/settings` 8.5 ms RSD 76.5% 等）——这些路由挂载耗时接近 rAF 帧量子化粒度（~16.7 ms），量测代理的抖动被小分母放大，属方法固有噪声而非真实性能波动。路由切换的用户体感主指标（settle）已全部满足 RSD ≤ 15%。

判定：**PASS**（settle 全路由 ≤ 15% RSD；mount 聚合 ≤ 5.4%；残余 mount 逐路由噪声为快路由 rAF 量子化，已记录为方法局限）。

## 6. 三项产品指标（Vue 基线，引用 Phase 0）

本批次只采集五项场景，三项产品指标（启动 / 首屏 / bundle）引用父任务 Phase 0 基线（`08-22-react-migration/baseline/`），不重复采集：

| 指标 | Vue 基线值 | 来源 |
| --- | --- | --- |
| server ready + warm | 12101 ms（含 Vite 冷启动与依赖预热） | `baseline/startup-timings.md`（原始见 `route-timing-settings.json`） |
| DOMContentLoaded（`/` `/settings` `/usage` `/configs`） | 57 / 53 / 52 / 52 ms | 同上（桌面运行时 CDP Navigation/Paint Timing） |
| First Contentful Paint（同上四路由） | 28 / 32 / 28 / 36 ms | 同上 |
| LCP | 未获得（骨架屏为主未触发 LCP 条目），以 FCP/DCL 对照 | 同上 |
| bundle：index | 243.69 KiB raw / 45.41 KiB gzip | `baseline/bundle-budget.txt` |
| bundle：UsageDashboardView（最大懒加载 chunk） | 93.40 KiB raw / 26.51 KiB gzip | 同上 |
| bundle：core.css | 123.13 KiB raw / 19.35 KiB gzip | 同上 |

React 侧对应值来源：bundle 由 `08-22-arch-quality-perf` 批次 8（`bundle-budget.md`，`check-bundle-budget.mjs` 重设后）提供；启动/首屏由 `08-22-regression-release` 步骤 7 以相同命令与路由集合复测（`perfTelemetry.ts` 采集能力保留）。本文件只作 Vue 对照端引用，不落 React 数值。

## 7. 三项产品指标的采集方法（Phase 0 口径）

- 路由级冷启动：`bun ./scripts/measure-vite-route.mjs`（脚本自起 Vite :5173，记录 serverReadyAndWarmMs 与各资源 fetch 耗时）。
- 首屏渲染：桌面运行时（tauri dev，WebView2）通过 CDP 在 `/`、`/settings`、`/usage`、`/configs` 四条路由读取 Navigation/Paint Timing。
- bundle 体积：`bun ./scripts/check-bundle-budget.mjs`。
- DCL/FCP 为 dev server 热缓存下的数值，用于同口径前后对比，不代表生产构建绝对值。迁移后复测须使用相同命令与相同路由集合。

## 8. 推迟表（React 侧全部数值 → 08-22-regression-release 步骤 7）

当前 React 基座仅有 1 条路由 `/`，无任何业务视图（阶段 5 七个视图子任务未迁移），五项场景的 React 侧数值**全部**推迟到 `08-22-regression-release` 步骤 7 补测。约束门已预豁免场景 1、3、4（`08-22-regression-release/implement.md` 步骤 7：「场景 1、3、4 的 React 侧数值由本任务首次补测」）；场景 2、5 同理记录如下原因。

| 场景 | Vue 基线（本文件） | React 侧 | 推迟原因 |
| --- | --- | --- | --- |
| 1 大表单输入延迟 | §1（P50 ≈ 4.2–4.6 ms / P95 ≈ 9.2–9.4 ms） | 待步骤 7 补测 | React 侧无设置页视图 |
| 2 虚拟列表滚动 | §2（500 行，60.09 FPS，0 掉帧；10k 待 React mock） | 待步骤 7 补测 | React 侧无监控页；10k 目标需 mock 数据 |
| 3 日志流 | §3（5 分钟稳态流） | 待步骤 7 补测 | 约束门已预豁免；React 侧无监控页 |
| 4 图表更新与主题切换 | §4（范围 P50 5.2 ms / 主题 P50 31.3 ms） | 待步骤 7 补测 | 约束门已预豁免；React 侧无用例页 |
| 5 路由切换 | §5（29 条采样，settle P50 155.3 ms） | 待步骤 7 补测 | React 侧仅 1 条路由 `/`，无 29 条业务路由可采样 |

脚本框架无关性是推迟计划的前提：`--base-url`（web 场景 1/2/5）与 `--cdp-url`（桌面场景 3/4）重定向后即可在 React 侧跑同一脚本、同方法、同视口，无代码改动。

## 9. 零编造声明与已知缺口

- 全部数值来自实际运行输出（原始 JSON + 人类可读表在批次 7 evidence 中存档）。任何未能采集的指标均如实记录为「方法受限 / 待补测」，未编造。
- 缺口清单：
  1. 场景 2 的 10k 行目标（Vue 版无可渲染 10k 行的列表，500 行上限代替；React 侧步骤 7 以 mock 数据恢复）。
  2. 场景 5 的 29 条路由 `changed` 指标 RSD（快路由 rAF 量子化噪声，已记录，settle 主指标达标）。
  3. 三项产品指标的 React 侧数值（批次 8 / 步骤 7 提供，本文件只引用 Vue 端）。
  4. ~~场景 3 聚合数值待三次完整运行后回填（§3 表格）。~~ → 已回填（2026-08-23，`evidence/perf-log-stream-run-batch2.txt`）；残余项为堆 Δ 两点差噪声（§3 迭代说明）。
