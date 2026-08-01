# 前端平台页面模式(ccr-ui)

> 调研日期 2026-08-01。栈:Vue 3.5 `<script setup>` + TS + Vue Router 4 + Pinia + Tailwind + vue-i18n + Tauri v2 IPC。前端(含 src-tauri)当前 **0 处 grok 代码**。

## 1. 路由与导航

- 路由:`src/router/index.ts` 手写 routes,挂 `MainLayout` children;meta 约定 `depth`(1 首页 / 2 子页,驱动过渡方向)、`group`(面包屑)、`cache: true` 需配 `cacheKey` + 组件 `defineOptions({name})`。参考 Codex:首页 L122-126,子页 L329-378。
- 侧边栏:`src/config/mainLayoutShell.ts` — `mainLayoutNavSections`(modules 组,`{to, labelKey:'nav.xxx', icon, iconClass:'text-platform-*'}`)、`mainLayoutRouteTitleMap`(route name→标题 key)、`mainLayoutGroupTitleMap`(group→section 标题)。
- 平台内子导航:`src/config/moduleSubnav.ts` 的 `moduleSubnavMap`(key=module 名,`{labelKey, href, icon, localOnly?}[]`);页面顶部 `<ModuleSubnav module="grok" />`。
- 平台色:`tailwind.config.ts:73-75` `platform.claude|codex|gemini` → CSS var `--color-platform-*-rgb`(主题 CSS 各加一条)。
- 能力表(可选):`src/config/platformCapabilities.ts`。

## 2. 平台首页两种模板

- **静态 Hub 型**(`ClaudeCodeView.vue` 929 行 / `GeminiCliView.vue`):Hero(eyebrow+标题+CTA+终端卡)、功能标签、核心/扩展模块卡格、常用命令 copy 列表(`copyText`)、外部资源。无自有 API。
- **数据仪表盘型**(`CodexView.vue` 935 行,推荐):逻辑在 `src/composables/useCodexDashboard.ts` — 模块级缓存 + TTL(30s/60s)+ inflight 去重;数据源 `getCodexDashboardOverview({force})` + `getCliVersion({tool, timeoutMs:1500, force})`(`api/runtime/system.ts`,`CliVersionEntry.status: ok|timeout|error|not_installed`);派生 `readinessItems`(tone success|warning|danger|neutral)、`nextActions`(前 3)、`primaryAction`、`compactInventory`;View 侧 `onMounted`/`onActivated` 都 `refresh(false)`,skeleton + `EmptyState` 兜底。
- 通用组件:`@/components/ui/{Button,Card,SIcon,EmptyState}.vue`,`Card variant="glass"`。

## 3. Profiles 页骨架(Claude 938 行 / Codex 1043 行,同构)

- 共享组件 `src/components/profiles/`:`ProfilesHeader`、`ProfilesStatStrip`、`ProfilesQuickRail`(钉选+最近)、`ProfilesToolbar`(搜索/状态/标签/排序/card-list)、`ProfilesSection`、`ProfileListRow`、`ProfilesInspector`、`ProfilesCommandPalette`(⌘K)、`ProfilesRawEditorPanel`(props `getRaw`/`saveRaw`)、`ProfileDiffRows`。样式 `@/styles/profiles-page.css`(`cp-*`,platform 色 `--cp-icon-color`)。契约:`.trellis/spec/ccr-ui/frontend/profiles-page-contracts.md`。
- 平台特有:编辑弹窗(`components/codex/CodexProfileEditorModal.vue`)、卡片(`ProfileCard`)、表单序列化 `utils/codexProfileEditor.ts`(create/fill/buildRequest)、descriptor 工厂 `utils/codexProfiles.ts`(`createXRowDescriptor(t)`/`createXInspectorDescriptor(t)`/`createXDiffFields(t)`)、过滤 `useCodexProfilesFilter`。
- 共享 composables:`useConfirmAction`(ConfirmModal 状态机)、`useProfilesQuickSwitch({platform})`(localStorage 按平台 key)、`useProfilesHotkeys`(`/`、⌘K、⌘1-9、Esc)。
- 交互(确认优先,非乐观更新):apply/delete/rename 先 `openConfirmDialog`(apply 用 `buildProfileDiff` 展示当前→目标 diff);`pendingAction` 行内 busy + `rowsDisabled` 全局锁;成功后 `markWrite()` + `loadProfiles({preserveData:true})` 整体重载;toast 走 `useUIStore().showSuccess/showError`;错误文案 `getErrorMessage(error, fallback)`;删除确认带备份 footnote。
- 环境门:`getCurrentEnvironment()` → 非 local 禁用源码编辑。

## 4. Settings 页双模式(Claude 1420 行 JSON / Codex 1065 行 TOML,同构)

- `tabs` = N 个可视化 tab(`v-show`)+ `source` tab(`v-if` 渲染共享 `src/components/editor/ConfigSourcePanel.vue`,`language="toml"`,props `get-raw`/`save-raw`/`list-layers`,events `saved`/`close`/`dirty-change`)。
- 可视化表单:`reactive` form + 嵌套字段 `computed({get,set})` 代理(CodexSettingsView L699-813 范本);头部单个 Save,`handleSave` 手工组装 payload → `updateCodexConfig(payload)`;可视化表单无脏检查,只有 saving 态。
- ConfigSourcePanel 内建:挂载先弹明文警告(拒绝即 close);`dirty` 上报;离开确认(父组件 changeTab + Panel 自身 `onBeforeRouteLeave`);保存带 token,返回 `status: saved|conflict|invalid|unsupported_environment`(conflict 重载横幅、invalid 行列 errorMarker,类型在 `api/domains/configRawTypes.ts`);`listLayers()` 分层展示。契约:`.trellis/spec/ccr-ui/frontend/raw-config-editor-contracts.md`。

## 5. API 层规则(有边界测试强制)

- `src/api/tauri.ts` ❄️ 冻结门面禁止新增;新 wrapper **唯一合法位置** `src/api/domains/<domain>.ts`,由 `src/api/index.ts` `export * as grokApi` 暴露。spec:`.trellis/spec/ccr-ui/frontend/api-facade-boundary.md`(smoke test 拦截)。
- `invokeRuntime.ts` 统一封装:查 `COMMAND_MANIFEST`,user_gesture 命令自动附 confirmationToken。
- `src/api/generated/` 由 handler_registry 再生,"do not edit"。
- 类型:手写 `src/types/grok.ts` + `src/types/index.ts` barrel;ts-rs 生成物在 `src/types/generated/`。

## 6. i18n

- `src/i18n/locales/zh-CN.ts` + `en-US.ts`(必须双边同步,`scripts/check-i18n.mjs` + `tests/i18n.test.cjs` 拦截)。
- 推荐 Codex 新式聚合命名:单一顶层 `grok: {}` 含 `overview/dashboard/profiles/settings/status/actions/states`;共享命名空间复用 `common.*`、`nav.*`(加 `nav.grok`)、`settingsRaw.*`、`profilesRaw.*`。

## 7. 新增 Grok 三页面文件清单(前端)

必改:`src/views/GrokView.vue`、`GrokProfilesView.vue`、`GrokSettingsView.vue`(新建);`src/router/index.ts`;`src/config/mainLayoutShell.ts`(三个 map);`src/config/moduleSubnav.ts`;`src/api/domains/grok.ts`;`src/api/index.ts`;`src/types/grok.ts` + `src/types/index.ts`;i18n 两语言文件。
配套新建:`src/utils/grokProfiles.ts`、`src/utils/grokProfileEditor.ts`、`src/composables/useGrokProfilesFilter.ts`、`src/composables/useGrokDashboard.ts`、`src/components/grok/{GrokProfileEditorModal,GrokProfileCard}.vue`。
视情况:`tailwind.config.ts` + 主题 CSS 加 `platform.grok` 色;`platformCapabilities.ts`;keep-alive cacheKey。
验证:`just frontend-check-quick`(含 i18n parity 与 api-facade-boundary 测试)。
