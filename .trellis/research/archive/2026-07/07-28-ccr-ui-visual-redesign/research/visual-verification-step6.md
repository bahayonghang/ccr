# Step 6 视觉核验证据 — dark + neutral（PRD AC6）

- 日期：2026-07-28
- 环境：`cd ccr-ui && bun run dev`（vite dev server，`http://127.0.0.1:15173`，`vite.config.ts` 固定端口 15173），Playwright MCP 驱动 Chromium，viewport 1004×946 CSS px
- 运行形态：web preview（无 Tauri 后端），Profiles 数据加载失败属预期，与配色无关
- 偏好注入（spec `theme-token-contracts.md` §3 契约）：先 `browser_navigate` 到 `/`，`browser_evaluate` 写入
  `localStorage: ccr-theme=dark, ccr-flavor=neutral, ccr-accent=clay`，然后 `location.reload()`，再断言 dataset，最后才记录计算样式与截图。

## 1. 路由清单与 dataset 断言

| 路由 | data-theme | data-flavor | data-resolved-flavor | data-accent |
|---|---|---|---|---|
| `/`（Overview） | dark ✓ | neutral ✓ | neutral ✓ | clay ✓ |
| `/claude-code/profiles` | dark ✓ | neutral ✓ | neutral ✓ | clay ✓ |
| `/settings` | dark ✓ | neutral ✓ | neutral ✓ | clay ✓ |

三条路由 `document.documentElement.dataset` 完全一致，无回退到默认 light/clay。

## 2. 关键计算样式（dark + neutral 下实测）

语义令牌（`getComputedStyle(document.documentElement)`）：

- `--color-bg-base: #131316`（全不透明）
- `--color-bg-elevated: #1a1b1f`（全不透明）
- `--color-bg-surface: #22242a`（全不透明）
- `--color-text-primary: #f2f3f5`（实心）
- `--color-text-secondary: #c9ccd3`（实心）
- `--color-border-subtle: rgb(235 238 245 / 14%)`、`--color-border-strong: rgb(235 238 245 / 34%)`

元素级（三条路由一致）：

- `body`：`background-color: rgb(19, 19, 22)`（=#131316 不透明），`color: rgb(242, 243, 245)`（=#f2f3f5 实心）
- `.claude-background`（fixed 全局背景层）：`rgb(19, 19, 22)` 不透明，`backdrop-filter: none` —— 诊断前 34% premium-blue halo + 16% pink 洗色带已不存在
- `.sidebar-glass`（fixed 侧栏）：`rgb(26, 27, 31)` 不透明，`backdrop-filter: none`
- `.topbar-glass`（sticky 顶栏）：`rgb(26, 27, 31)` 不透明，`backdrop-filter: none`
- `.titlebar-shell`（fixed 标题栏）：`rgb(34, 36, 42)` 不透明，`backdrop-filter: none`
- 主内容卡片（Overview `main` 内实测）：`section.dashboard-*` 背景 `rgba(26, 27, 31, 0.98)`（≥98% 符合 surface-card 契约方向），内部 metric  chip `rgba(34, 36, 42, 0.62–0.68)` 叠加在近不透明父层与不透明底层之上，全页 `backdrop-filter: none`，视觉上无雾化

## 3. 截图

均存于 `ccr-ui/.tmp/`：

- `step6-overview-dark-neutral.png` — `/` Overview 整页
- `step6-profiles-dark-neutral.png` — `/claude-code/profiles` 整页
- `step6-settings-dark-neutral.png` — `/settings` 视口（外观段）
- `step6-settings-flavor-section.png` — `/settings` 滚动至「界面语调」flavor 区（补充证据，见 §5 残留 1）

## 4. 目检结论（对照 diagnosis.md 诊断前症状）

- **背景 halo / 洗色带已消失**：全局背景层为纯 `rgb(19,19,22)` 平面暗色，三张整页截图无任何径向光晕、顶部洗色带或噪点层。
- **卡片边界清晰**：Profiles 页统计卡、设置页 header 卡 / 导航卡 / 主题选项卡均有可见的细分隔边框（14–22% alpha 亮边），卡片与底色层级分明，无边界模糊。
- **文本实心**：主标题与正文为 `#f2f3f5` 实心近白，次要文本 `#c9ccd3` 实心灰，无发灰半透明文本（诊断前 stage 文本半透明问题未复现）。
- **按钮不发白**：Profiles「添加 Profile」、Overview「确认 Web 限制」均为实心 clay accent 填充（实心橙陶色），暗色下未发白发灰。
- 整页无残留白雾感、无纯白块。

结论：**AC6 通过** —— dark + neutral 下三条目标路由无雾、边界清晰、文本实心。

## 5. 残留问题（仅记录，未在本步修复）

1. **`.app-settings-nav`（Settings 页内 sticky 导航）仍有透出**：容器背景 `rgba(0,0,0,0)`、`backdrop-filter: none`；非激活按钮背景 `rgba(26, 27, 31, 0.72)`、激活按钮 `rgba(232, 131, 91, 0.1)`。滚动时下方内容文字会透出导航条（见 `step6-settings-flavor-section.png` 顶部「界面语调」描述文字与导航卡交叠）。按玻璃三档预算契约，页内 sticky 工具面应走 inline 档或不透明 surface；建议子任务 C 重写 Settings UI 时一并处理。
2. **flavor 选项列表仍是旧 7 项**（暖陶/纸面/石墨/Latte/Frappé/Macchiato/Mocha），等子任务 C 重写；Step 6 新增的 `neutral`/`catppuccin` i18n 键已就位，旧 UI 因无匹配选项不会渲染 `undefined` caption（`flavorStatusLabel` 仅当 `flavor.value === option.value` 时才拼 resolved label）。
3. **`settings.appearance.flavor.description` 及 latte/frappe/macchiato/mocha 描述仍是旧自适应文案**（深色映射 Frappé 等），与新体系（catppuccin: light→latte, dark→mocha）语义不符，按任务约定保留，待子任务 C 统一清理。
4. Profiles 页「加载 Claude Profiles 失败」为 web preview 缺少 Tauri `invoke` 的预期错误（console 1 error），非配色问题。

## 6. Console 状态

- `/` 与 `/settings`：0 error（各 1 条与配色无关的既有 warning）。
- `/claude-code/profiles`：1 error = `Failed to load Claude profiles: Cannot read properties of undefined (reading 'invoke')`（web preview 无 Tauri 后端，预期）。
