# Design — 设置坞（左下角）排版与样式优化

> 方向契约：行情终端——等宽状态读出、发丝分隔、琥珀只住激活标记。体量小，一个组件 + 一段 CSS。

## 现状问题

`MainLayoutChrome.tsx:74-97` + `shell/shell.css:199-251`：meta 行是 0.6875rem 点分隔长串（`theme · flavor · locale · version`），flex-wrap 在窄侧栏凌乱换行；视觉层级平（标题/meta/版本同一重量）；active/hover 态与新世界的纪律不一致。

## 方案

1. **结构**（`MainLayoutChrome.tsx` 的 settings-dock 块）：
   - 行 1：图标 + 「设置」标题 + 版本号右对齐（mono、muted，`v7.3.0`）。
   - 行 2：meta 状态串 = `跟随系统·深色` / `中性` / `中文` 三项，mono + `tabular-nums` + `·` 分隔，**单行 nowrap + text-overflow: ellipsis + title 悬浮全文**。
   - 版本号从 meta 串移入行 1，meta 只剩三项状态，缩短一半。
2. **样式**（`shell/shell.css:199-251`）：
   - active（`/settings` 路由）：`--color-bg-overlay` 表面 + 左侧 2px 琥珀 tick（与设置页命令行列表同一语法）；非 active 保持 `--surface-card-bg` + subtle border。
   - hover：border 升到 `--color-border-default`，不做位移/发光。
   - focus-visible：`--glow-primary` 焦点环（既有 token）。
   - 200px 下限宽度（`shellPreferences.ts:55-57`）与 480px 上限两端布局整齐：图标 2rem 固定，右列 min-width:0 保证 ellipsis 生效。
3. **文案修正**（随本任务的 locale 窗口一起结算）：flavor 描述在新世界下失实——`中性` 已转暖纸灰而非"不带冷暖倾向"。zh-CN 与 en-US 的 `settings.appearance.flavor*` 描述改为与新 token 一致的表述（中性=暖纸灰、暖陶=暖陶棕纸面）；bootMessages.ts 若含同 key 同步。
4. **行为不变**：NavLink to="/settings"、`data-testid="settings-dock-link"`、meta 计算（`MainLayout.tsx:77-85` 响应式）不动。

## 验证

- `bun run type-check && bun run lint && bun run test`（相关 shell smoke 适配更新若有断言）。
- 视觉：dev:web 下侧栏 200px / 480px 截图；`/settings` 路由 active 态截图。
