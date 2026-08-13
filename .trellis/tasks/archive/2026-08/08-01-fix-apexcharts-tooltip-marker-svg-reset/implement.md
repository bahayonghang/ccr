# Implementation Plan

## Preconditions

- 保持任务 `status=planning`，直到用户明确批准本次最终规划摘要。
- 实施前运行 `trellis-before-dev`，重新加载 ccr-ui frontend 与 Usage Chart Stability
  contracts。
- 保留工作区既有 `.trellis/.version` 改动及其他用户变更。

## 1. Capture Baseline

- [x] 确认 WebView2 CDP 指向真实 `/usage` 页面。
- [x] 运行 probe normal 模式，保存 `healthy=true` 的结构化输出。
- [x] 运行 `--block-runtime-css`，确认未修复基线为 `healthy=false`、runtime style 不存在、
  giant marker > 0，并确认脚本退出后页面恢复正常。

Rollback point：无产品代码改动。

## 2. Add The Independent CSS Path

- [x] 在 `ccr-ui/src/utils/apexChartsCore.ts` 的第三方 imports 旁添加
  `import 'apexcharts/dist/apexcharts.css'`。
- [x] 保留现有模块化 type/feature imports 和默认 runtime style injection；不改 chart
  options、Vite manualChunks 或全局 reset。

Rollback point：删除单个 CSS import 即恢复原行为。

## 3. Add The Contract Test

- [x] 新建 `ccr-ui/tests/apexcharts-style-contract.smoke.test.ts`。
- [x] 断言装配入口包含唯一的完整 CSS import，并保留 area/line/bar/donut/heatmap/legend
  imports。
- [x] 读取 `apexcharts/dist/apexcharts.css`，断言 tooltip 绝对定位、series group 初始
  隐藏、marker host 12px 和 marker SVG 100% 的关键规则。
- [x] 测试失败信息应明确提示依赖升级后的 style contract drift，而非使用宽泛 snapshot。

Rollback point：测试与 CSS import 同批回滚。

## 4. Narrow Verification

- [x] `cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/apexcharts-style-contract.smoke.test.ts`
- [x] `cd ccr-ui && bun run type-check`
- [x] `cd ccr-ui && bun run lint`
- [x] `cd ccr-ui && bun run build`
- [x] 检查 production CSS asset 确实包含 marker/group 关键规则，且 charts JS chunk/现有
  懒加载边界没有重复模块警告。
- [x] 检查包含 ApexCharts 规则的 CSS asset 位于图表动态 import 的 preload 依赖中，且未被
  写成 `index.html` 的首屏直链样式。

任一失败即停在本步骤修复；不以 probe 或源码断言替代 production build。

## 5. Runtime And Visual Verification

- [x] 启动任务自有的 Tauri debug/WebView2，记录 PID/端口，避免占用或停止用户已有服务。
- [x] probe normal 模式必须 `healthy=true`。
- [x] probe `--block-runtime-css` 模式也必须 `healthy=true`，且输出
  `runtimeStyle.present=false`、构建管理的 rule source > 0、giant marker = 0。
- [x] 在 Usage donut 首次加载与 hover 后检查 tooltip 为紧凑绝对定位浮层，card 高度不变。
- [x] 抽查一个 axis chart 页面；确认图表、tooltip/legend 和主题仍正常。
- [x] 留存一张故障注入后仍正常的最终截图；不保留无判别力的中间截图。

## 6. Full Frontend Gate And Diff Review

- [x] `just frontend-check-quick`
- [x] `just frontend-check`（共享 ApexCharts 入口影响多个 frontend 页面，因此不作条件跳过）。
- [x] `git diff --check`
- [x] 检查最终 diff 只包含装配入口、focused test 和任务规划/研究产物。
- [x] 清理本任务启动的浏览器、CDP、Vite/Tauri 进程；不停止用户原有 5173/5174/5199。

## 7. Completion Evidence

- [x] 在任务记录中列出实际运行命令、exit code、probe 关键字段和截图路径。
- [x] 明确写出“故障机制已确认，现场触发源仍未知”；不把 fault injection 描述为自然复现。
- [x] `bun run build:with-budget` 当前未修改产品代码的基线已因入口 JS 超预算而失败；若实施
  期间重跑，必须与本任务结果分开记录，不得写成新增 CSS 导致的回归或伪造为通过。
- [x] 进入 Trellis quality check、spec update、commit 和 finish-work 流程；未经另行授权不 push。

实际命令、结构化探针字段、产物检查和验收矩阵见 `verification.md`。
