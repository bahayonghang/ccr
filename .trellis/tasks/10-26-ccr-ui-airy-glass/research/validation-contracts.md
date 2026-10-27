# Research: 轻盈玻璃迁移的验证与兼容契约

- Query: 现有主题、材质、首帧、可访问性和性能测试能否证明 airy-glass 迁移安全；哪些断言必须迁移，哪些行为必须冻结？
- Scope: internal，静态只读研究；未运行产品测试、浏览器、桌面应用或安装依赖。
- Date: 2026-10-26

## Findings

### 1. 设计方向变更需要显式迁移契约

| 证据 | 当前事实 | 迁移含义 |
| --- | --- | --- |
| `ccr-ui/DESIGN.md:244` | 同屏最多 1 个 backdrop-filter，仅 floating tier 可模糊，禁止滚动内容玻璃与嵌套玻璃 | workspace 玻璃会改变材质角色分配；须同步设计说明、令牌与测试，不能沿用旧文档声称满足新需求 |
| `ccr-ui/DESIGN.md:254` | 产品表面圆角上限 12px | 如仅外层工作区增大圆角，必须限定角色，不把全局 radius 替换成大圆角 |
| `ccr-ui/DESIGN.md:410` | 禁紫色/紫蓝渐变 | 与用户指定 airy-glass 的淡紫背景有直接方向冲突；用户请求支持迁移方向，后续审阅应确认具体适配方案 |
| `.trellis/spec/ccr-ui/frontend/theme-token-contracts.md:278` | 同屏最多 1 个真实 blur，floating only | 当前可执行守卫并没有证明这一预算 |
| `ccr-ui/src/styles/tokens.css:490`、`ccr-ui/tests/theme/apple-glass-surface-contract.smoke.test.ts:157` | 令牌注释和测试仍写“同屏 backdrop-filter 元素 ≤ 3” | **已确认规范漂移**：测试只查注释文本，并非 DOM 实际计数。应改成一致预算，并补运行时可见层计数 |

推荐：只允许一个专用工作区玻璃层；普通表格、输入、日志、代码编辑器、sticky 表头、弹窗保持实底。不要将 `--surface-workspace-*` 直接改为玻璃，因其为共享别名；先审计全部消费者，必要时增加唯一外壳专用语义别名。若采用此方案，floating 模态也应取消 blur，避免同时两个真实模糊层。无 blur 支持和减少透明度时必须实底回退。

### 2. 外观状态兼容必须保持

- `ccr-ui/src/utils/themeBootstrap.ts:1`：`ThemeMode = light | dark | system`；flavor 为 `neutral | clay`，持久化 accent 为 `clay`。`system` 是偏好，渲染主题最终为 light/dark。
- `themeBootstrap.ts:7`：存储键分别为 `ccr-theme`、`ccr-flavor`、`ccr-accent`，三轴不能折叠为单个“新风格”开关。
- `themeBootstrap.ts:32`：旧 flavor 和 accent 有迁移映射；尤其 `amber → clay` 已存在，不能把 clay 改名 amber 再制造歧义。
- `themeBootstrap.ts:269` / `:285`：持久化迁移仅对已有存储写回，空存储不应被新默认值污染。
- `themeBootstrap.ts:332` / `:421`：runtime custom accent 是独立覆盖态，不加入持久化枚举；支持 light/dark 配色、整族 CSS 变量覆盖、非法 hex 不改 DOM 并返回 false、清除后恢复枚举 accent。
- `themeBootstrap.ts:395` 附近自定义 accent 的文字色是亮度启发式，不等于任意自定义颜色均已通过 WCAG；静态 contrast suite 只验证 `clay`，不能夸大覆盖。
- `ccr-ui/tests/theme/theme-bootstrap.smoke.test.ts:72` 起覆盖 system 跟随、迁移、独立持久化、空存储、flavor 不随主题改变；这些不是视觉旧债，禁止删改为迎合新样式。
- `ccr-ui/tests/theme/theme-domain-extension.smoke.test.tsx:37` 起覆盖主题轴单写点及 runtime custom 扩展。保持这类结构契约。
- 字体偏好独立于主题：`ccr-ui/tests/theme/font-preferences.smoke.test.ts`；沿用 sans/brand/mono 三轨及用户字体覆盖，不能为了 airy-glass 顺带更换字体机制。

建议验证矩阵：light/dark × neutral/clay 共 4 个基础组合；另加 system 首帧和运行时系统切换、旧持久化值迁移、空存储、有效/无效 custom accent、字体存在/缺失/重置。预置 localStorage 后必须断言 `data-theme` / `data-flavor` / `data-resolved-flavor` / `data-accent`，不能只凭截图标签认为已覆盖。

### 3. 首帧和原生窗口有独立配色来源

- `ccr-ui/index.html:12` 起内联 IIFE 在加载应用 CSS 前解析主题/迁移/字体；`:56` / `:61` 是硬编码 warm-paper / warm-black boot 背景。新主题必须同时更新 boot 视觉并保留迁移逻辑，避免首帧闪旧色。
- `ccr-ui/src/utils/nativeWindowAppearance.ts:7` 保存 macOS 原生窗口底色 `#EEF4FF` / `#07121F`；`:12` 只同步 macOS；`:26` 同时调用 setTheme / setBackgroundColor。与当前 DESIGN 的背景已不是同一色值，纳入首帧视觉核查。
- `ccr-ui/tests/theme/native-window-appearance.smoke.test.ts:45` 是 mock 测试，固定浅色期望。若迁移窗口底色需同步该值级断言；保留 macOS-only 与 lazy import 行为。
- `ccr-ui/src-tauri/tauri.conf.json:61` 主窗口 1200×800、最小 800×600，`transparent: false`。本次 CSS 玻璃不要求开启 OS 透明窗、vibrancy 或修改权限/CSP。
- 浏览器验证不能证明系统标题栏、窗口 resize、DPI、原生首帧、tray popup 的实际合成效果。原生验收需独立记录 PASS / FAIL / NOT_RUN。

### 4. 对比度守卫强，但不是玻璃合成验证

`ccr-ui/tests/theme/theme-contrast-contract.smoke.test.ts`：

- `:39` 固定 4 个 theme × flavor 组合；`:34` accent 仅 clay。
- `:65` 静态解析只取顶层规则；不执行浏览器 cascade/layout，也跳过 @media 嵌套。
- `:271` 支持 alpha blend，但目前主要用于 border；并未模拟 workspace 上薄荷—淡紫背景经过 blur 后的不同采样位置。
- `:293` 锁 `--color-bg-*`、文本、stage、card、workspace 完全不透明。
- `:324` primary ≥12:1、secondary ≥7:1、muted ≥4.5:1，基准是 `--color-bg-surface`。
- `:361` border-default ≥1.2:1；`:383` accent/contrast ≥3.5:1。
- `:396` 再扫描文本/表面令牌，禁止 alpha。

保持现有阈值，不以“更轻盈”为由降低。优先保留现有实底 surface 别名，只让新的工作区外壳角色透明；新增实测或可复现合成计算，覆盖背景两端、渐变中间、边缘、内容滚动位置以及明暗/降级状态。普通文字目标至少 4.5:1，重要控制边界/焦点至少 3:1；现有 stronger token gates 继续独立执行。不能拿边框 1.2:1 规则当作所有可交互边界合规证明。

### 5. 需有意迁移的测试与不能动的保障

| 类型 | 文件/锚点 | 建议 |
| --- | --- | --- |
| 旧材质角色 | `tests/theme/apple-glass-surface-contract.smoke.test.ts:116` | 更新 shell/workspace/modal 角色断言；保留禁嵌套、禁旧 utility、reduced-transparency 回退；新增可见 blur 实际计数 |
| 固定圆角 | `tests/theme/token-consolidation.smoke.test.ts:65`、`:88` | 新角色如需圆角扩展，定向登记，不能删除整套 token 守卫或放任所有组件大圆角 |
| 固定外观预览 | `tests/theme/flavor-preview-consistency.smoke.test.ts:11` | 同步 `src/features/configs/lib/flavorPreview.ts` 四组色值，保持预览与真实 token 一致 |
| 配色/实底 | `tests/theme/theme-contrast-contract.smoke.test.ts:293` | 保持文字/列表/card/输入相关实底断言；仅为批准的唯一外壳角色新增合成验证 |
| 硬编码控制 | `tests/theme/apple-glass-surface-contract.smoke.test.ts:39`、`.stylelintrc.json:42` | 不因允许淡紫背景而解除组件硬编码调色板限制；用语义 token 表达颜色 |
| 结构/偏好 | `tests/theme/theme-bootstrap.smoke.test.ts`、`theme-switch.smoke.test.tsx`、`theme-domain-extension.smoke.test.tsx`、`font-preferences.smoke.test.ts` | 保持行为断言 |
| 动效 | `tests/theme/reduced-motion.smoke.test.tsx`、`src/utils/reducedMotion.ts` | `data-reduced-motion` 单读取轨道继续保持；不要添加第二套 matchMedia 状态 |
| 模态/安全 | `tests/quality/overlay-single-implementation.smoke.test.ts`、`tests/ui/confirm-interaction.smoke.test.tsx` | BaseModal 单实现、焦点/关闭/确认门保持 |
| 路由/状态 | `tests/shell/cache-route.smoke.test.ts`、`main-layout-route-presence.smoke.test.ts`、`event-bridge-leak.smoke.test.tsx` | 不改路由生命周期、滚动恢复、订阅/清理等行为来完成视觉重构 |

所有测试路径以上除特指源码外均相对 `ccr-ui/`。

### 6. 性能工具存在，但基线需先校准

- `ccr-ui/scripts/perf/_lib.mjs:8` 固定 1800×1125；`:46` 描述连续三轮 RSD ≤15%；`:105` headless Chromium；`:115` 桌面 CDP 是另一条显式连接路径。
- `scripts/perf/perf-list-scroll.mjs:18` 固定注入 500 条监控日志，通过 `data-testid="monitoring-log-row"` 找容器，测 rAF 与 >33ms / >50ms 掉帧。`:4` 的 Vue/唯一虚拟列表历史说明已陈旧，不能证明当前 React 大列表覆盖。实际行数和滚动距离必须验证，不能把 500 条监控当 10,000 条虚拟列表。
- 该脚本 `measureScroll` 先解构返回值，再访问 `stamps.error`；页面返回 `{error}` 时 stamps 为 undefined，可能报二次异常。若用于本任务验收，应先作最小 harness 校准并记录失败路径，而不是默认结果可靠。
- `scripts/perf/perf-route-switch.mjs:17` 采样表有 `/mcp-manager`、`/slash-commands`、`/budget`、`/pricing` 等历史入口。`:57` 监听 `#app`（当前 `index.html:82` 仍有此节点）；`:86` 使用 pushState+popstate，注释仍称 vue-router。需对照当前 routeCatalog，确认每个场景加载目标页面且排除 error/redirect 假样本；目前没有运行证据证明失效或有效。
- `scripts/perf/perf-form-input.mjs`、`perf-chart-update.mjs`、`perf-log-stream.mjs`、`soak-persist.mjs` 可作为工具入口，但本研究未逐一验证其适用性。
- 必须保留同一机器、运行模式、浏览器/WebView版本、窗口/DPI、数据量、路由、主题的 before/after 原始结果；至少三轮，不混合 native 与 headless 结果。修改前没有成功基线就不能宣称“无性能回退”。

### 7. 建议验收路径（未来实施时运行，本轮全部 NOT_RUN）

工作目录 `ccr-ui/`：

```powershell
bun run type-check
bun run lint
bunx vitest run --config vitest.smoke.config.ts tests/theme/apple-glass-surface-contract.smoke.test.ts tests/theme/theme-contrast-contract.smoke.test.ts tests/theme/theme-bootstrap.smoke.test.ts tests/theme/theme-switch.smoke.test.tsx tests/theme/theme-domain-extension.smoke.test.tsx tests/theme/token-consolidation.smoke.test.ts tests/theme/flavor-preview-consistency.smoke.test.ts tests/theme/font-preferences.smoke.test.ts tests/theme/reduced-motion.smoke.test.tsx tests/theme/native-window-appearance.smoke.test.ts tests/quality/overlay-single-implementation.smoke.test.ts tests/ui/confirm-interaction.smoke.test.tsx
bun run test
bun run test:smoke --coverage
bun run build
bun run dev:web -- --host 127.0.0.1 --strictPort
```

`vitest.smoke.config.ts:38` 保持文件并行，`:55` lines coverage ≥70%，CSS 被排除，因此覆盖率通过不代表视觉正确。针对实际变更的域再加 profiles、usage、shell 等原有行为 smoke。

视觉矩阵应包含默认 1200×800、最小 800×600、宽屏 1800×1125，以及 Sidebar modal 断点两侧；4 个基础主题组合；默认/悬停/按压/键盘焦点/禁用/错误/加载/空数据；中英文长文本；弹窗、下拉与 sticky/滚动叠加；减少动效、减少透明度、无 blur 回退。浏览器截图与计算样式证据单独落盘，无法操作时明确 NOT_RUN。

根目录 `just ui-check` 委托 `ccr-ui/justfile` 的 `check`（`:747`），包含 cargo check + 前端 lint/type/smoke，但不等于 build、coverage 或视觉验收全部通过。根 `just frontend-check`（`:813`）还含 docs 等完整前端门；`just ci` 用于最终交付/跨模块验收，不能以之代替目标浏览器证据。部分根门包含 `bun install --frozen-lockfile`（`justfile:759`），执行前应检查安装边界，不把检查名误认为纯只读。Rust 测试沿用默认并行及 `--skip export_bindings`。

## Files Found / Related Specs

- `.trellis/spec/ccr-ui/frontend/theme-token-contracts.md`：主题、材质、字体、boot 与 reduced-motion 的主契约。
- `.trellis/spec/ccr-ui/frontend/usage-chart-stability-contracts.md`：图表配置稳定性与 CSS 双路径；视觉迁移不得更改数据/挂载策略。
- `.trellis/spec/ccr-ui/frontend/confirm-interaction-contracts.md`：确认门与危险操作语义。
- `.trellis/spec/ccr-ui/frontend/profiles-page-contracts.md`：Profiles 共享骨架/编辑序列化边界。
- `.trellis/spec/ccr-ui/frontend/development-resource-contracts.md`：预览进程、warmup 与 worker 资源边界。
- `ccr-ui/package.json:7`：真实 scripts 入口；依赖显示 React 19、Tailwind 4、Tauri 2、Playwright。

## External References

无外部搜索；本文件结论仅来自当前 checkout 的代码、配置与本地规范，不声称外部库最新版本行为。

## Caveats / Not Found

- 未运行任何上述命令、截图或原生验收；不存在本轮测试通过数、性能提升幅度或视觉 PASS。
- 源码正则可以保护令牌写法，不能证明真实级联、可见 blur 数量、alpha 合成与可读性。
- `native-window-appearance` 是 mock 覆盖，不能充当 macOS 实机证据。
- 基线与验收门不得直接复用其他项目（如 Markbridge）的窗口尺寸、列表数据量或历史通过数。
