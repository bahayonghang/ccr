# 主题 Token 体系与壳层深度分析（2025-09-03）

## Token 架构

唯一权威：`ccr-ui/src/styles/tokens.css`（719 行）。导入链：`styles/index.css` → `core.css`（Tailwind v4）→ `tokens.css`、`theme.css`（遗留别名桥）、`chart-colors.css`。

`<html>` 上三个正交属性：
- `data-theme = light|dark`（`system` 经 `prefers-color-scheme` 解析）+ 冗余 `.dark` 类（`themeBootstrap.ts:86-87`）
- `data-flavor = neutral|clay`（中性为 `:root` 默认，clay 覆盖）
- `data-accent = clay`（值域已塌缩为单一强调色）

变量组：surface（`--color-bg-base/-elevated/-surface/-overlay` + rgb 三件套）、border、text、accent（`--color-accent-primary` 族 + `--color-accent-secondary` 暖沙）、semantic（success/warning/danger/info 各带 glow/contrast/tint）、platform（6 平台 × 4 角色变体，`tokens.css:107-137` / dark `249-279`）、stage 语义层、间距/圆角/排版/阴影、玻璃材质契约（`--material-glass-*` → `--surface-shell/workspace/card/modal/status`，`tokens.css:427-501`）。

**深色表面值**：
- 中性深（`[data-theme='dark']`，`tokens.css:161-170`）：`#131316 / #1a1b1f / #22242a / #2c2f37` —— **冷灰**
- 暖陶深（`[data-theme='dark'][data-flavor='clay']`，`tokens.css:635-643`）：`#17120f / #221b18 / #2a221e / #342b26` —— 暖棕
- 深色强调色 `#e8835b`（`tokens.css:200`）

**"配色不对"的技术根源**：中性深色是冷灰蓝调，与暖 clay 强调色气质冲突；用户两张截图恰好分别是 clay 暖棕（首页）与中性冷灰（设置页），观感割裂。用户已拍板：**中性深色向暖棕微调**，两底色族在深色下同源。

## Flavor 切换链路

`AppearanceSection.tsx` → `shellPreferences.setFlavor()`（`shell/stores/shellPreferences.ts:181-190`）→ 持久化 `ccr-flavor` + `applyFlavorToDocument` 写 `data-flavor`（`utils/themeBootstrap.ts:103-112`）。首帧 IIFE 在 `ccr-ui/index.html:10-48` 同步。flavor 持久化无 bug。

## 发现的遗留/隐患

1. **自定义强调色死代码**：`applyCustomAccent/clearCustomAccent`（`themeBootstrap.ts:421-453`）无调用方；`data-accent='custom'` 会被下次 `applyAccentToDocument` 抹掉，`index.html:17` 白名单强制回 `clay`。
2. **启动 Loader 颜色不符**：`index.html:60-62` 硬编 `#app-loader` 深色底 `#000000` + 蓝色 `#2997ff` 转圈，与真实深色表面/强调色不符，深色启动有可见闪色。
3. **`data-resolved-flavor` 残留**：恒等于 `data-flavor`（`resolveFlavorMode` 忽略 theme 入参，`themeBootstrap.ts:67-72`）。
4. **Antigravity token 孤儿**（详见 overview 分析）：两个蓝色争夺同一平台身份，重构必须二选一。
5. **双深色标记**：`.dark` 类与 `[data-theme='dark']` 并存，Tailwind 只读属性（`core.css:37`），可能漂移。

## 壳层布局

- 根：`shell/MainLayout.tsx:90-170` —— `.layout-shell` flex 行、h-screen；侧栏 + `main.content-main`（含顶栏与 `.content-scroll-area`，`p-4 sm:p-6`，`overflow-y-auto`）。
- 侧栏组件：`shell/MainLayoutChrome.tsx:24-100`；样式 `.sidebar-glass`（`shell/shell.css:108-117`，`--surface-shell-bg` = `--color-bg-elevated`）。宽度内联样式，store 约束 **默认 240px，200-480 夹取**（`shellPreferences.ts:55-57`），拖拽手柄 `MainLayoutChrome.tsx:48-61`。
- 导航分区：`config/mainLayoutShell.ts:14-80` —— dashboard / workspace（配置中心）/ modules（平台）/ tools（工具）。
- 图表色映射：纯 CSS 属性选择器 `features/usage/styles/dashboard-usage-movement.css:192-206`（antigravity → gemini 蓝，见上）；通用 5 槽位图表 ramp：`styles/chart-colors.css:9-24`（`--chart-color-0..4`）。

## 相关既有 spec

- `.trellis/spec/ccr-ui/frontend/theme-token-contracts.md`
- `.trellis/spec/ccr-ui/frontend/layering-contracts.md`
- `.trellis/spec/ccr-ui/frontend/react-rerender-discipline.md`
