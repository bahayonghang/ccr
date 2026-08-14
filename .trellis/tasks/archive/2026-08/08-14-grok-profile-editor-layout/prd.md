# 优化 Grok Profile 编辑器滚动与排版

## Goal

让 Grok 添加/编辑 profile 弹窗在桌面视口内完整可用：第三方长表单能向下滚动，页脚操作始终可达，排版与 Claude/Codex 编辑器同属一套 `pe-*` 工作台语言。

用户价值：第三方字段多，当前弹窗被视口裁切且无法滚动，Enabled 与保存不可达；修滚动并理清分段后，创建/编辑才能一次完成。

## Background

用户在桌面暗色主题下打开「添加 Third party」，内容超出窗口下沿，底部 Enabled 与保存区被裁切。这是 `08-01-grok-ui-profiles` 落地后的外壳缺陷，不是字段或 patch 契约缺陷。

已确认决策（2026-08-14）：对齐 Claude/Codex 编辑器壳（限高 + `pe-scroll` + 段导航 + 粘性页脚 + `pe-*` 节奏），不只开 `BaseModal.scrollable`，也不做更深字段 IA。

## Confirmed Facts

- 复现面是 `GrokProfileEditorModal`，由 `GrokProfilesView` 打开。第三方创建态最长，官方更短，同一模态。
- `BaseModal` 默认 `scrollable: false`，面板 `overflow-hidden` 但无限高；打开时锁定 `document.body` 滚动。`.pe-scroll` 的内部滚动只在祖先限高时生效。
- Claude/Codex 用 `pe-shell max-h-[calc(90vh-9rem)] overflow-hidden` + 内层 `pe-scroll` + `pe-nav` + 壳内 `pe-footer`，不用 `scrollable`。
- Grok 已消费 `pe-*`，但缺限高、段导航、校验跳转；Tags/Enabled 无分段标题；页脚走 BaseModal `#footer`。
- 字段与 write-only patch 由 `profiles-page-contracts.md` 场景三冻结。现有 editor smoke 只锁契约，不锁滚动。
- 视觉方向保持 Anthropic-like 编辑式工作台与 `pe-*` / `--cp-*`。禁止新平行令牌或紫色科技感装饰。

## Requirements

### R1 — 滚动与可达

- 第三方创建/编辑在常见桌面高度（≤900px 高）下，表单主体可垂直滚动。
- 标题、关闭、Cancel/Save 在滚动时保持可见且可点。
- 滚轮、触控板、键盘（Tab / 方向键 / PageDown）都能到达 Tags、Enabled 与页脚。
- 官方短表单不得出现无内容的空滚动条或被压扁的页脚。
- 打开模态不得在锁页面滚动的同时把超高内容裁死。

### R2 — 对齐 Claude/Codex 编辑器壳

- 分段阅读顺序：身份 → 连接/凭据（仅第三方）→ 运行时 → 标签/启用。官方导航不出现连接段。
- 提供段导航；校验失败时顶部汇总条可跳到对应分段。
- 分段标题、字段标签、控件、弱化状态盒的间距与对齐跟 Claude/Codex `pe-*` 同一套节奏。
- Tags/Enabled 有独立分段标题，不再贴在运行时段底下无标题收尾。
- 继续用 `pe-*` 与主题 token；不新增 `--editor-*` 或平台私有色板。
- 明暗主题、窄桌面（约 1280×720）与标准桌面都要能读完并保存。
- 不改字段集、校验规则、既有 i18n 语义；允许新增段导航/跳转/状态段文案。

### R3 — 契约与回归

- `buildGrokPatch`、凭据互斥、`base_url_display` 不回写、官方只读 kind、Local-only 关闭写入口，行为不变。
- 现有 Grok Profiles / editor smoke 继续成立；补最小滚动/分段回归。

## Acceptance Criteria

- [ ] 第三方添加/编辑：视口 1280×720 与 1440×900，暗/亮主题下，主体可滚到 Tags/Enabled，Cancel/Save 始终可见可点。
- [ ] 官方添加/编辑：短表单完整可见；导航无连接段；无多余滚动条，页脚不被裁切。
- [ ] 滚轮、触控板、键盘都能到达被裁切字段；焦点不掉出模态。
- [ ] 段导航可跳转；校验失败时汇总条能跳到对应分段。
- [ ] 排版可扫读：分段层级清楚；双列在窄宽下不错位；控件继续走 `pe-input` / `pe-select` / `pe-panel`。
- [ ] 未改 patch/凭据/kind 契约；`tests/grok-profile-editor.smoke.test.ts` 与 `tests/grok-profiles-view.smoke.test.ts` 通过。
- [ ] `cd ccr-ui && bun run type-check && bun run lint` 通过；触及 i18n 时跑 `bun run test:i18n`。
- [ ] 视觉走查走 web preview（`bun run dev:web -- --host 127.0.0.1 --strictPort`，`http://127.0.0.1:5173/`）。Tauri-only 保存不作为本任务视觉失败。

## Out of Scope

- 不改 Grok profile 字段、patch 语义、校验规则、DTO。
- 不改列表页骨架、卡片、Inspector、快捷键。
- 不改 Claude/Codex 编辑器，除非共享 `pe-*` 的无回归修复。
- 不做源码编辑器、provider 模板、新设计体系、可选字段折叠/逐步披露。
- 不把官方/第三方拆成两个模态。
- 不启用 `BaseModal.scrollable` 作为主方案（会与 `pe-scroll` 叠出双滚动）。
