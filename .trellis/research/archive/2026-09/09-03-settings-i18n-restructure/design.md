# Design — 设置页重构与全局中文化

> 方向契约：行情终端。craft-floor 绝对禁令适用于本页：**heading 上方的 kicker/eyebrow 一律删除**，标题自己说话。本任务 = i18n 修复（根因）+ 设置页终端化重构 + 回归测试 + locale 死键清理（自 overview 子任务移交）。

## 1. 根因修复

**(a) stale memo**：`features/configs/hooks/useAppSettings.ts:181-184` 的 `useMemo(() => ..., [])` 改为语言感知：组件内使用 `useAppT()`（`src/i18n/index.ts:99` 响应式 hook），memo 依赖数组加入解析后的 locale（或直接去掉 memo——sections 只有 4 项，计算成本为零；优先去 memo，最简单且永不陈旧）。

**(b) zh-CN 英文值中文化**：两个装载层都要改——`i18n/locales/zh-CN.ts`（全量包）与 `i18n/bootMessages.ts`（首屏包，settings 词条在 `:509` 起有一份拷贝）。已知清单（实施时以扫描为准补全）：
- settings 域：`settings.eyebrow` 'Shell Preferences'（zh-CN.ts:2501）、七个分区 eyebrow（L2515-2581：Appearance/Theme/Flavor/Typography/Language/Shell/Diagnostics）、`settings.summary.runtimeDesktop/runtimeWeb`（L2511-2512）
- 全局其余 eyebrow：`zh-CN.ts` L1236（Claude Profiles）、L1635（Operations Monitor）、L1656（Live Log）、L1672（Event Health）、L3346（Codex Command Center）、L4707-5163（Grok 集群：Readiness/Next actions/Model defaults/CLI runtime/Worktrees/Custom models 等）、L5224（Usage Insight）
- 零散英文标签：`Base URL`（L1007）、`Warning`（L1690）等
- 扫描方法：zh-CN.ts 中 value 为纯拉丁串且 key 匹配 `eyebrow|summary|runtime` 等模式的条目逐一评审；误伤 guard：产品名（Claude Code/Codex/MCP/API/Base URL 这类技术名词的保留与否按中文技术写作惯例——`Base URL` 保留属正常，评审逐条定）。
- 同步英文侧不需要改（en-US 原值不动）；改的是 zh-CN 的 value。

**(c) 死键清理**（overview 移交）：`dashboard.usage.peakLabel/hoverHint/metricSelectLabel/metricPlatforms` 从 zh-CN.ts + en-US.ts + bootMessages.ts（若有）删除；`EXPECTED_LEAF_COUNT` 两处（`scripts/check-i18n.mjs`、`tests/i18n.test.cjs`）4407 → 4403 与本任务新增/删除合并结算。

## 2. 设置页终端化重构（`AppSettingsView.tsx` + `app-settings.css` + 分区组件）

- **Hero**：删除 eyebrow kicker（`settings.eyebrow` 元素整个移除；该 key 双语言一并删除并结算计数）。标题「全局设置」直接承担；meta chips（Desktop Runtime / v7.3.0 / 跟随系统·深色模式 / 中文 / 312px）重排为等宽状态读出串：mono + `tabular-nums` + 发丝分隔，不要 chip 盒子。
- **分区选择列**（Appearance/Language/Workbench/Diagnostics）：卡片 → 命令行列表：icon + 名称 + 说明一行，`border-bottom` 发丝分隔；选中态 = `--color-bg-overlay` 表面 + 左侧琥珀 tick（2px），不用重边框/发光。说明文字用修复后的响应式翻译。
- **Theme 三选卡 / Flavor 双卡**：保留卡片形态（选择器语义），但选中态收敛为琥珀发丝边 + tick 标记，去掉厚重 fill；Flavor 预览色块已在 token 任务同步新值（`flavorPreview.ts`），此处只确认渲染。
- **Typography / Workbench / Diagnostics 分区**：结构不动，排版对齐新纪律（label 层级、发丝分隔、行距）。
- craft-floor 其他适用项：无装饰填充、无嵌套卡、状态色只住标记点。

## 3. 回归测试（新增）

- `tests/i18n/` 新增断言：settings 域 + 全部 `*.eyebrow` 残留 key 的 zh-CN 值含 CJK 字符（防止"zh 里写英文"回归）；可推广为全局 eyebrow 模式断言。
- `tests/configs/` 新增 live 切换 smoke：渲染 `AppSettingsView` → `setLocale('en-US')` → `setLocale('zh-CN')` 往返，断言分区选择列文案与 hero meta 无英文残留、无陈旧缓存。
- 现有门禁保持绿：`bun run test:i18n`、`check-i18n.mjs`（计数结算后）。

## 4. spec 沉淀

- `react-rerender-discipline.md` 增补条款：组件内禁止对 `t()` 输出做空依赖 memo；统一 `useAppT()`；locale 切换必须触发重渲染的断言方式。
- `frontend/index.md` 登记该条款引用。

## 边界与不做

- 只重构设置页本身的 eyebrow 元素；其他页面的 eyebrow 元素本任务只翻译值、不删元素（全 app 元素级清理是后续 harden 议题，记入 PRD Notes）。
- 不改 i18n 架构（boot + 懒加载两层保持）；不动 `tt(zh,en)` 的 471 处内联字面量。
- locale 文件是本任务独占改动面；其他子任务不得并行改 `zh-CN.ts`/`en-US.ts`/`bootMessages.ts`。

## 验证

```bash
cd ccr-ui && bun run test:i18n && bun run scripts/check-i18n.mjs
cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/i18n/ tests/configs/
cd ccr-ui && bun run type-check && bun run lint && bun run test && bun run build
```
