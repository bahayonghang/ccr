# 批次 2 落位与页面级样式判定

> 所属任务：`08-22-design-system` 批次 2（styles 目录分层）。
> 依据：`design.md` §3 分层落位表 + §3 末段页面级文件判定标准。
> 采集日期：2026-08-23，分支 `react-migration/react-foundation`，未提交。
> 约束执行：无变量改名、无值变更（纯移动 + 导入路径更新）；`themes/` 第 1 层变量不搬（见 §3 决策）。

## 1. 落位表（old → new，18 个文件）

| 旧路径 | 新路径 | 归属说明 |
| --- | --- | --- |
| `src/styles/index.css` | 不动（根） | 主入口，同步首屏加载 |
| `src/styles/core.css` | 不动（根） | 主入口，承载 `@import` 链与 `@theme inline`/`@utility` 定义 |
| `src/styles/tokens.css` | 不动（根） | 第 1 层可切换语义变量 + 常量 token 定义点（批次 1 决策保留，见 §3） |
| `src/styles/theme.css` | 不动（根） | 旧短名兼容桥（`@layer base` 内 `:root`），与 tokens.css 伴生，批次 8 契约重建时一并评估 |
| `src/styles/chart-colors.css` | 不动（根） | 设计 §3 根目录项 |
| `src/styles/shell-critical.css` | 不动（根） | 首屏 shell-critical 层（由 core.css `layer(components)` 引入） |
| `src/styles/deferred-interactive.css` | 不动（根） | 三层加载的 deferred 聚合器（批次 2 起由 React 侧惰性加载，见 §5） |
| `src/styles/deferred-decorations.css` | 不动（根） | 同上（decorations 层） |
| `src/styles/backgrounds.css` | 不动（根） | 装饰层内容（由 deferred-decorations.css 引入），非 base/themes/components/utilities 任一形态 |
| `src/styles/animations.css` | 不动（根） | 动画层内容（由 deferred-interactive.css 引入）；逐段判定归批次 7 |
| `src/styles/base.css` | `src/styles/base/base.css` | 设计 §3 base/：reset + 文档级默认 |
| `src/styles/fonts.css` | `src/styles/base/fonts.css` | 设计 §3 base/：根字号/字体声明（当前无任何导入点，纯定义文件；旧 Vue 主程经 `/fonts/...` URL 直载字体子集，不经本文件） |
| `src/styles/components/surfaces.css` | 不动（components/） | `addComponents` 插件迁移产物（`08-22-dep-upgrade` 段 2），判定为组件类形态（见 §4） |
| `src/styles/home.css` | `src/styles/components/home.css` | 页面级判定：单域局部 token（见 §2） |
| `src/styles/codex-auth-shared.css` | `src/styles/components/codex-auth-shared.css` | 页面级判定：单域共享样式（见 §2） |
| `src/styles/profiles-page.css` | `src/styles/components/profiles-page.css` | 页面级判定：多路由共享（见 §2） |
| `src/styles/checkin-shared.css` | `src/styles/components/checkin-shared.css` | 页面级判定：多组件共享（见 §2） |
| `src/styles/utilities.css` | `src/styles/utilities/utilities.css` | 设计 §3 utilities/：自定义工具类（deferred 加载） |

`git mv` 保留历史；所有文件保持原 basename，无文件重命名。

## 2. 四个页面级样式文件的逐文件判定

判定标准（`design.md` §3 末段）：规则只服务单一路由 → 进对应 `features/<域>/.module.css`；
被多路由共享 → 进 `components/`。`features/` 目录尚不存在（阶段 5 视图子任务创建），
且四个文件的当前消费方均为 `.vue`（死代码，React 侧尚未接入）。故批次 2 统一落 `components/`，
阶段 5 归属作为 disposition 记录。

| 文件 | 规模 | 服务域 | 是否单路由 | 批次 2 落位 | 阶段 5 归处 | 理由 |
| --- | --- | --- | --- | --- | --- | --- |
| `codex-auth-shared.css` | 639 行 / 14.8 KB | codex auth（CodexAuthView + Accounts/Providers tab + 各 Modal） | 单域多组件 | `components/` | `features/codex/.module.css`（若届时 CodexAuth 视图路由级懒加载；注意其首屏全局层约束，见下） | `code-splitting.md` §3.1 明确「codex-auth-shared.css 属全局样式层，仍应进入首屏」，批次 2 必须保留 index.css 首屏导入。阶段 5 若 CodexAuth 路由懒加载，可随视图子任务改 `features/codex/` 并按需首屏 |
| `home.css` | 73 行 / 41 个 `--home-*` 变量 | home（HomeView + components/home/*） | 单域 | `components/` | `features/home/.module.css` | 全部为 `--home-*` 局部 token，无全局类；当前经 core.css `layer(components)` 首屏引入（home 是首屏页）。阶段 5 视图子任务将其收进 home 域模块 |
| `profiles-page.css` | 217 行 / 28 个 `--cp-*` 变量 | profiles（Claude / Codex / Grok 三个 profiles 视图共享） | 多路由共享（单域三平台） | `components/` | `features/profiles/.module.css`（或保持 components/，视三平台是否共享 shell 而定） | 判据「被多路由共享的进 components/」命中。当前仅被 3 个 `.vue`（死代码）导入，React 未接入；`.vue` 导入路径已同步更新以保持树一致 |
| `checkin-shared.css` | 24 行 / 1.1 KB | checkin（Tab 面板/表格/空状态 + 账号仪表盘） | 单域多组件 | `components/` | `features/checkin/.module.css`（若视图子任务需路由级 scoping；当前首屏共享，也可能保持 components/） | 仅 `.checkin-surface-card` / `.checkin-badge-pill` 两个共享表面配方类；由 index.css 首屏导入。阶段 5 视图子任务决定收进 checkin 域或保留 components/ |

## 3. `themes/` 目录决策

**批次 2 不移第 1 层变量，`themes/` 目录移除（AC3）。**

- 依据：批次 1 落位决策 1（`implement.md`）已记录——`theme-contrast-contract` /
  `apple-glass-surface-contract` / `theme-bootstrap` 三个 smoke 测试直接解析
  `tokens.css` 文本（`KNOWN_SELECTOR_PATTERN` 只接受 tokens.css 顶层块形态），
  `theme-switch.smoke.test.tsx` 同样断言 tokens.css 内 `--color-bg-surface-rgb` 锚点。
  把第 1 层可切换变量物理移出 `tokens.css` 会破坏这 4 个测试（已复核 4 个测试源码确认）。
- 决策：`tokens.css` 保持为第 1 层变量定义点；`themes/` 目录本批次无内容，
  按 AC3「空目录填充或删除」移除；批次 8（`theme-token-contracts.md` 重建）随
  `design.md` §1 的目标结构重建 `themes/`，届时同步放宽三个 smoke 测试的选择器清单。
- 设计文档 §3 的最终落位不变（themes/ 最终承载按 data-theme / data-flavor / data-accent 分文件的第 1 层变量）。

**`animations/` 空目录**（批次 7 的范围）：批次 2 同样按 AC3 移除；批次 7 判定后按需重建。

## 4. `@utility` 与 `addComponents` 的落位判定

- **4 处 `@utility` 定义（`transition-interactive`、`duration-fast`、`duration-normal`、`duration-slow`）留在 `core.css`（主入口）**。
  理由：它们是首屏工具类可达性所必需（旧 Vue 组件在 `@apply`/类名中引用，v4 扫描期需定义在场；
  移入 deferred 的 `utilities/` 会改变产出时机，且 `transition-interactive` 被 shell 级组件使用）。
  `utilities/` 目录承载的是「自定义工具类文件」`utilities.css`（deferred 加载），与设计 §3 的
  utilities/ 描述「@utility 定义与自定义工具类」有出入，判定记录：@utility 定义归主入口，
  utilities/ 归自定义工具类文件。
- **`addComponents` 插件迁移**（`08-22-dep-upgrade` 段 2 迁来）已是 `components/surfaces.css`。
  判定为**组件类形态**（非工具类）：其头部注释已记录「addComponents 产出的是组件类——
  固定多属性整体、不与变体组合；须保持工具类可覆盖组件类的层序语义」。故不迁入 `utilities/`。

## 5. 三层 CSS 加载语义（批次 2 落地方案）

- 现状（React）：`main.tsx` 只同步 `import './styles/index.css'`，`deferred-interactive.css` /
  `deferred-decorations.css` **无加载点**（`code-splitting.md` §3.1 记录的三层语义缺口）。
- 批次 2 落地：在 React 侧补 deferred 加载器（`src/utils/deferredStyles.ts`，由 `main.tsx` 调用）：
  - 首帧后（`scheduleAfterPaint`）：惰性挂载 `deferred-interactive.css?url`（交互层，优先级高）；
  - 空闲时（`scheduleWhenIdle`）：惰性挂载 `deferred-decorations.css?url`（装饰层）。
  - 挂载方式与旧 Vue `main.ts` 一致：`<link rel="stylesheet" data-style="deferred-*">`，幂等。
- 语义保持：首屏 CSS（index.css 产物）只含 `shell-critical` 层；`deferred-*` 两层为独立产物、
  首帧后才进 DOM。构建产物核对见 `implement.md` 批次 2 证据。

## 6. 本次测试/配置路径更新（仅路径，不改断言）

| 文件 | 变更 |
| --- | --- |
| `ccr-ui/tests/apple-glass-surface-contract.smoke.test.ts` L19 | `'../src/styles/checkin-shared.css'` → `'../src/styles/components/checkin-shared.css'` |
| `ccr-ui/tests/apple-glass-surface-contract.smoke.test.ts` L183 | `readFile('src/styles/utilities.css')` → `readFile('src/styles/utilities/utilities.css')` |
| `ccr-ui/.stylelintrc.json` L31 | `"src/styles/checkin-shared.css"` → `"src/styles/components/checkin-shared.css"` |
| `ccr-ui/src/views/ClaudeCodeProfilesView.vue` L361 | `@/styles/profiles-page.css` → `@/styles/components/profiles-page.css` |
| `ccr-ui/src/views/CodexProfilesView.vue` L372 | 同上 |
| `ccr-ui/src/views/grok/GrokProfilesView.vue` L397 | 同上 |

`tokens.css` / `core.css` / `deferred-decorations.css` 路径不变（留在根），对应测试断言原样保留。
