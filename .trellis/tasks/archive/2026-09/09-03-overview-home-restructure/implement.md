# Implement — Overview 首页重构

> 必读：`prd.md`、`design.md`、`research/overview-page-analysis.md`（父任务 research 目录）、spec 三份（usage-chart-stability / dashboard-presentation / environment-scoped-dashboard）、方向契约 surface brief。

## 执行清单

### Step 1 — 图表尺度修复（先做，可独立验证）
- [x] `dashboard-usage-movement.css`：`.dashboard-usage__chart` 去 `flex:1` 加 `height: clamp(10rem, 26vh, 16rem)`；`.dashboard-usage` 去 `height:100%`。
- [x] `dashboard-view.css`：`.dashboard-lower` 的 `align-items: stretch` → `start`；右栏 `dashboard-signal-stream.css` 的 `__list` 加 `max-height` + `overflow-y:auto`。
- [x] 验证：dev:web 下拖窗高 700px→1200px，图表高度只在 clamp 区间变化；右栏超长自滚动。（浏览器探针实测：vh624→162px、vh800→208px、vh2000→256px 封顶；窄窗 620px 行情带横向滚动）

### Step 2 — 图表终端化细节
- [x] 发丝水平网格线（3-4 条，低对比 hairline）；柱/段去圆角；指标行数字 `tabular-nums`。
- [x] 验证：`tests/dashboard/dashboard-usage-movement.smoke.test.tsx` 既有断言保持绿（百分比数学不动）。

### Step 3 — Sessions 诚实态
- [x] 按 design.md §3 决策树：先读 `src/api/generated/usageV2.ts` 与 `src-tauri/src/services/usage.rs:1126-1245`，选第一层可行方案；新增 `dashboard.*` key（双语言）+ 同步 `EXPECTED_LEAF_COUNT`。
- [x] 平台格/用量面板的 sessions 位渲染"未索引"诚实态。
- [x] 验证：web 预览（无后端）下显示诚实态而非误导性 0；如改了 Rust 跑 `bun run tauri:check && bun run tauri:test`。（决策树第一层可行：消费既有 `bootstrap.needs_session_index` + `snapshot.readiness.active_session_index`，未动 Rust）

### Step 4 — 平台行情带
- [x] `DashboardPlatformMatrix.tsx` + `dashboard-platform-matrix.css`：四卡 → 单面板内行情带（design.md §2）；sessions 数字渲染进格内；Antigravity 色已是确权色（上个任务完成），确认不被回退。
- [x] 验证：1600px 与 1280px 下带内元素不溢出；窄窗横向滚动。

### Step 5 — 页头/右栏终端化排版
- [x] 状态检查项 pills → 状态行；Event stream 日志行化；Action queue 命令行化（design.md §4）。
- [x] 验证：dev:web 截图评审；`tests/dashboard/` smoke 全绿。（亮/暗截图存 `outputs/ccr-ui/overview-restructure-{1600,dark}.png`）

### Step 6 — 收尾
- [x] `cd ccr-ui && bun run type-check && bun run lint && bun run test && bun run build`（+ Rust 动了则 `tauri:check`）。（未动 Rust；lint 需先排除未跟踪的 `.impeccable/` 设计工作流目录——eslint.config.js 加 ignore，与既有 `.tmp` ignore 同理）
- [x] spec 更新检查：`usage-chart-stability-contracts.md` 若图表高度策略变化需要补记，写进该文件。（另在 `dashboard-presentation-contracts.md` 补 sessions 诚实态契约）

## 回滚点

- Step 1/2 纯 CSS，独立可 revert；Step 3 若触发 IPC 契约变更，先停下确认再动 Rust。

## Review gates

- Step 1 完成后图表尺度必须已受控，再继续后续排版。
- 全部完成后对照父任务验收标准第 2/3/4 条。
