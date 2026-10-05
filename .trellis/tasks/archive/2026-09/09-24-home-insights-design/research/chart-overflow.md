# 首屏用量图越界：复现与原因

## 结论

用户截图中的越界来自已安装桌面包的旧前端代码，不是 dev 分支当前代码的缺陷。dev 分支当前代码在 1920×1080 下图表高度为 256px（= 16rem），符合 `DESIGN.md` 的 `clamp(10rem, 26vh, 16rem)`。

frontend 子任务 R1 不需要修改代码。验收方式改为：用当前 dev 分支重新构建桌面包（`just tauri-build`）后，在桌面端确认图表高度不超过 16rem。

## 证据

1. 截图特征与旧 CSS 一致。截图中柱体有圆角、网格线为虚线。当前 CSS（`ccr-ui/src/features/usage/styles/dashboard-usage-movement.css`）中柱体无圆角，网格线为 `solid`。圆角（`border-radius: var(--radius-sm) var(--radius-sm) 0 0`）与 `dashed` 网格线只存在于 `51ea4b1b^`（`git show 51ea4b1b^:ccr-ui/src/features/usage/styles/dashboard-usage-movement.css` 第 161、189 行）。
2. 旧 CSS 的高度链路不受限：
   - `.dashboard-usage { height: 100% }`（旧文件第 5 行）
   - `.dashboard-usage__chart { flex: 1; min-height: var(--space-32) }`（旧文件第 131-141 行），无上限
   - `.dashboard-lower { align-items: stretch }`（旧 `dashboard-view.css` 第 175 行）
   结果：用量面板被拉伸到右侧栏（行动队列 + 事件流）的高度，图表占满剩余空间。
3. `51ea4b1b`（2026-09-03，"替换 UI 视觉世界为「行情终端」"）把图表改为 `height: clamp(10rem, 26vh, 16rem)`，删除 `flex: 1` 与 `height: 100%`，`.dashboard-lower` 改为 `align-items: start`。
4. 版本号不能区分两个构建：`Cargo.toml` 在 `8de2cfed`（2026-09-02，`chore(release): v7.3.0`）设为 7.3.0，之后的提交未改版本号，所以旧构建和新构建都显示 "CCR UI v7.3.0"。

## 复现（当前 dev 代码）

脚本：本目录 `chart-overflow.mjs`（Playwright，`addInitScript` 注入模拟的 `window.__TAURI_INTERNALS__`，`get_home_usage_overview_v2` 返回 7 天中 2 天有请求的数据，与截图一致）。

步骤：

1. `cd ccr-ui && bun run dev:web -- --host 127.0.0.1 --strictPort`
2. 把脚本复制到 `ccr-ui/` 下运行 `bun <script>`（需要解析 `ccr-ui/node_modules/playwright`）。

1920×1080、DPR 1 的测量结果：

| 元素 | 高度 |
| --- | --- |
| `.dashboard-usage__chart` | 256px（计算样式 `256px`，`overflow: hidden`） |
| `.dashboard-usage` | 453px |
| `.dashboard-lower` / `.dashboard-rail` | 539px |
| 最高柱 `.dashboard-usage-stack` | 235px（`--stack-height: 100%`，减去 20px 坐标轴留白） |

`.dashboard-usage__chart` 规则已加载（`document.styleSheets` 中可找到该选择器）。

## 副作用记录（不在本任务范围）

- 模拟运行时控制台出现 `Cannot read properties of undefined (reading 'unregisterListener')`。原因：模拟对象未实现 `__TAURI_EVENT_PLUGIN_INTERNALS__`。这是测试脚本的限制，不是产品缺陷。
