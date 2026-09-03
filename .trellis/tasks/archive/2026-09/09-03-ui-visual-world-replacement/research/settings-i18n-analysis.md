# 设置页与 i18n 深度分析（2025-09-03）

## 设置页组件地图

| 部分 | 文件 | 关键行 |
|---|---|---|
| 页面根/hero/布局 | `ccr-ui/src/features/configs/AppSettingsView.tsx` | hero 44-66，栏目卡 67-82，分区 83-124 |
| Theme/Flavor/Typography 面板 | `features/configs/settings/AppearanceSection.tsx` | Theme 74-93，Flavor 94-116，Typography 118-194 |
| Language 面板 | `features/configs/settings/LanguageSection.tsx` | 9-45 |
| Workbench 面板 | `features/configs/settings/ShellSection.tsx` | 32-107 |
| Diagnostics 面板 | `features/configs/settings/DiagnosticsSection.tsx` | 10-37 |
| 展示组件（无文案） | 同目录 `NavButton/ThemeOption/FlavorCard/ChoiceButton/SettingsSwitchRow` | — |
| 状态 hook | `features/configs/hooks/useAppSettings.ts` | 见下 |
| 分区定义 | `features/configs/lib/settingsModel.ts` | `SETTINGS_SECTIONS` L96-101 |
| 样式 | `features/configs/styles/app-settings.css` | — |
| 路由 | `shell/routeCatalog.ts:17-19`（`deferLocaleHydration: true`） | — |

## 中英文切换失效的根因（两个独立机制叠加）

i18n 架构（`src/i18n/index.ts`）：i18next + react-i18next，`zh-CN|en-US`，默认 zh-CN，持久化 `localStorage['ccr-ui-locale']`；boot 包同步加载 + 完整包懒加载（`ensureLocaleLoaded` L122-136）。Settings 词条在 boot 包里就有（`bootMessages.ts:509` zh / `:1123` en），**不是缺 key**。

三条翻译路径：`translate()`（L93，原始单例，非响应式）、`useAppT()`（L99，响应式）、`tt(zh,en)`（L96，内联字面量）。`setLocale()`（L138）只 `changeLanguage` + 持久化，不刷新页面。

**(a) 中文语言包里故意写了英文值**（"editorial eyebrow" 风格，重挂载也还在）：
- `locales/zh-CN.ts:2501` `settings.eyebrow: 'Shell Preferences'`
- L2515/2523/2530/2539/2555/2564/2581：`Appearance/Theme/Flavor/Typography/Language/Shell/Diagnostics`
- L2511-2512 `settings.summary.runtimeDesktop: 'Desktop Runtime'` / `runtimeWeb: 'Web Preview'` → hero meta chip

**(b) 空依赖 useMemo 缓存了英文文案**（左侧栏目卡英文副标题）：
- `useAppSettings.ts:181-184`：`const sections = useMemo(() => SETTINGS_SECTIONS.map(... t(titleKey), t(captionKey) ...), [])` —— 空依赖数组，挂载时按当时语言算一次，永不重算。切换后卡片仍显示 en-US 文案（"Theme and visual tone" 等），而 zh-CN 同 key 有中文值（`zh-CN.ts:2518,2558,2566,2584`）。
- 页面其余部分会重渲染（MainLayout 通过 `useShellT` 订阅，整棵 Outlet 子树随 `languageChanged` 重渲染），所以呈现"中文标题 + 英文 eyebrow + 英文栏目卡"的混合态。

设置页组件内**没有**硬编码英文 JSX；问题 = 数据 (a) + 记忆化 (b)。

## 系统性评估

- 侧边导航翻译正常（`MainLayoutNav.tsx:56`，响应式 `t`）。
- 空依赖 `useMemo(...t()...)` 全仓库扫描：**仅设置页这一处**，机制上是孤例。
- **英文 eyebrow 是全 app 约定**：zh-CN 里 29 处英文 eyebrow（`Operations Monitor` L1635、`Claude Profiles` L1236、`Codex Command Center` L3346、`Usage Insight` L5224、Grok 集群 L4707-5163 等），另有 `Live Log`/`Event Health`/`Warning`/`Base URL` 等零散标签。
- `tt(zh,en)` 内联字面量全仓库 471 处，求值是实时的，无此问题。

**用户已拍板：全局中文化**——29 处英文 eyebrow 与零散英文标签全部翻译为中文。

## 左下角设置坞

- 文件：`ccr-ui/src/shell/MainLayoutChrome.tsx:74-97` —— `NavLink to="/settings"`（`data-testid="settings-dock-link"`，`.settings-dock`，`/settings` 上带 `--active`）。
- 文案：标题 `t('nav.settings')`（L85）；meta 行 = `themeLabel · flavorLabel · localeLabel · APP_VERSION_LABEL`（L86-94），计算于 `MainLayout.tsx:77-85`（响应式 `useShellT()`），版本来自 `config/appMeta.ts:9`。
- 样式：`shell/shell.css:199-251`（`.settings-dock`、`__icon` 2rem 方块 `:214-224`、`__title` 0.875rem/600 `:234-238`、`__meta` **0.6875rem** flex-wrap 点分隔 `:240-246`、`__version` mono `:248-250`）。
- 问题：窄侧栏下 meta 行换行凌乱；纯导航无其他行为；视觉层级与排版粗糙（标题/meta/版本混排一行点分隔串）。

## 测试现状与缺口

- `tests/i18n.test.cjs`（1027 行）：存在性/体量/命名空间/占位符，**语言无关**——zh 里写英文也能过。
- `tests/i18n/settings-i18n.smoke.test.ts`：11 个 `settings.*` key 双语言"非空"断言，不查翻译质量。
- `tests/i18n/i18n-runtime.smoke.test.ts`：`setLocale` 免刷新切换/持久化/插值/兜底。
- `scripts/check-i18n.mjs`：en/zh 叶 key 集合对齐（`EXPECTED_LEAF_COUNT = 4404`）+ 未用/缺失 key 扫描。
- **缺口**：没有任何门禁保证 zh-CN 的值真的是中文；没有测试在设置页上做真实语言切换。两种故障模式都能穿过所有现有门禁 → 需要新增回归测试。

## 修复方向

1. `useAppSettings.ts:181-184` 的 sections 改为语言感知（用 `useAppT()` 并依赖解析后的 locale，或去掉 memo）。
2. zh-CN.ts 中 `settings.*.eyebrow`、`settings.summary.runtime*` 及全 app 29 处英文 eyebrow 翻译为中文。
3. 将"组件内禁止对 `t()` 结果做空依赖 memo，统一 `useAppT()`"写入 spec（`react-rerender-discipline.md` 或新 i18n 契约）。
4. 新增测试：zh-CN 值中文断言（设置域）+ 设置页 live 切换回归。
