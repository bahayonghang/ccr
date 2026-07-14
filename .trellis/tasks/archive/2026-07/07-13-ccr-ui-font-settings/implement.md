# Implement — ccr-ui 字体设置与 fallback

工作目录：`ccr-ui/`。先看 prd.md → design.md，再按序执行。每步给出验证。

## 前置

- [ ] 分支：`feature/ccr-ui-font-settings`（从 `dev` 切）。
- [ ] 确认 open question「预设字体清单」已定稿（design.md §4）。

## 有序清单

### A. CSS 基座（tokens.css）

- [ ] 在 `ccr-ui/src/styles/tokens.css` 字体系统块，新增 `--font-sans-base/--font-brand-base/--font-mono-base`（= 现有字面栈），并把 `--font-sans/brand/mono` 改为 `var(--font-*-base)`。
- 验证：`rg -n "font-sans-base" src/styles/tokens.css`；grep 现有 `var(--font-sans)` 消费端无需改动。

### B. 字体偏好 util（新文件）

- [ ] 新建 `ccr-ui/src/utils/fontPreferences.ts`：
  - 常量：`FONT_UI_STORAGE_KEY='ccr-font-ui'`、`FONT_CODE_STORAGE_KEY='ccr-font-code'`、`MAX_FONT_NAME_LEN=64`。
  - `sanitizeFontFamily(input): string`（design §3 规则）。
  - `readStoredUiFont()/readStoredCodeFont()`、`persistUiFont()/persistCodeFont()`（localStorage，try/catch 静默降级，对齐 themeBootstrap 风格）。
  - `applyFontsToDocument(ui, code)`：净化后非空则 `setProperty` 组合值，空则 `removeProperty`（sans+brand 走 ui，mono 走 code）。
- 验证：`just frontend-typecheck` 局部通过。

### C. Store 接线（shellPreferences.ts）

- [ ] `uiFont/codeFont` ref + 初始化；`initializeTheme()` 末尾调用 `applyFontsToDocument`。
- [ ] `setUiFont/setCodeFont`（净化→persist→apply→更新 ref），并在 `return` 暴露。
- 验证：typecheck 通过；无对既有 theme/flavor/accent 行为的回归。

### D. 设置 UI（AppSettingsView.vue）

- [ ] 外观区 accent 卡片后新增「字体」卡片：两行控件（下拉 + 自定义输入 + 预览条），`data-testid`：`settings-font-ui` / `settings-font-code` / 对应预览。
- [ ] 绑定 `storeToRefs(shellPreferencesStore)` 的 `uiFont/codeFont` 与 `setUiFont/setCodeFont`；预设清单 + 「自定义…」切换逻辑；预览条 `:style="{ fontFamily: ... }"`。
- [ ] （可选）hero summary pill 增加当前界面字体标签。
- 验证：`npm run dev` 手测切换即时生效、回退、重置。

### E. FOUC 引导脚本（index.html）

- [ ] 在「主题预初始化」IIFE 内追加字体读取 + `clean()` + `setProperty`（design §5）。
- 验证：刷新无字体跳变。

### F. i18n（zh-CN.ts / en-US.ts）

- [ ] `settings.appearance.typography.*`：eyebrow/title/description、uiLabel/uiDescription、codeLabel/codeDescription、systemDefault、custom、previewSampleUi/previewSampleCode。zh/en 双语齐备。
- 验证：`tests/settings-i18n.smoke.test.ts` 断言新增关键 key（两 locale 均 truthy）。

### G. 测试

- [ ] 新建 `tests/font-preferences.smoke.test.ts`：净化规则（引号/分号/花括号/尖括号/超长）、read/persist 往返、`applyFontsToDocument` 覆盖与移除（断言 `:root` inline `--font-sans/brand/mono`）、空值=系统默认。
- [ ] 扩展 `tests/theme-bootstrap.smoke.test.ts` 引导脚本用例：预置 `ccr-font-ui/ccr-font-code`，执行脚本后断言 `document.documentElement.style.getPropertyValue('--font-sans')` 含用户字体且以 `var(--font-sans-base)` 结尾；含恶意输入被净化。
- [ ] `tests/settings-i18n.smoke.test.ts` requiredPaths 增补字体 key。
- 验证：`cd ccr-ui && bun run test`（或 `just frontend-check-quick`）。

## 验证命令（收尾门槛）

- [ ] `just fmt-check`
- [ ] `just frontend-check-quick`（typecheck + lint + smoke）
- [ ] `npm run dev` 人工走查：界面字体改本机存在字体（生效）→ 改不存在字体（回退无豆腐块）→ 系统默认（还原）→ 代码字体独立生效 → 刷新保持、首帧无跳变。

## 风险文件 / 回滚点

- `src/styles/tokens.css`：抽 `-base` 若拼错栈会全局掉字体 → 改后立即 `npm run dev` 目视。
- `index.html` 引导脚本：语法错误会拖垮首帧初始化 → 保持在既有 IIFE 内、小步追加、跑引导脚本 smoke 用例。
- 回滚：还原 tokens.css 三行 + 移除新增文件/分支/卡片/i18n/测试；localStorage 冗余键无害。

## task.py start 前检查

- [ ] prd/design/implement 三件套齐备并经用户评审。
- [ ] 预设字体清单定稿。
- [ ] 与用户确认无需触碰 Tauri 后端（已确认：不碰）。
