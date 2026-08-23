# 执行计划：Tailwind v4 与 shadcn/ui 设计体系重建

> 父任务：`08-22-react-migration`（阶段 2，在 `08-22-arch-quality-perf` 之后）。
> 分支：`feature/react-migration/design-system`，PR 目标 `feature/react-migration`。
>
> 本任务必须在七个 `08-22-views-*` 之前完成 token 层与原语层，否则视图迁移时重复决策样式。

## 前置确认

- [x] `08-22-dep-upgrade` 段 2 已完成：Tailwind 为 4.3.3，`corePlugins.preflight: false` 的等价处理已生效，25 个 `@apply` 文件已加 `@reference` 且 `apply-verification.md` 无静默失效项。
- [x] `08-22-arch-quality-perf` 批次 3 已交付组件内样式行数上限（本任务产出受其约束）。
- [x] 读 `ccr-ui/CLAUDE.md` 的 Design Context 与 `theme-token-contracts.md`（31.5 KB）。
- [x] ~~`git checkout -b feature/react-migration/design-system feature/react-migration`~~ **偏差（已记录）**：按父任务录音的命名冲突偏差，继续工作在 `react-migration/react-foundation` 分支上，不新建分支、不 commit/push。迁移前名字集合基线已在当前分支的 `src/styles/**` 采集（与 dev 等价：dep-upgrade 未改名）。

## 批次 1：token 分类与两层结构

- [x] 按 `design.md` §2 的方法对 448 个变量分三类，`token-classification.md` 落盘（448 行无空缺）。
- [x] 按 `design.md` §1 建两层结构：可切换语义变量进 `themes/` 下的普通 CSS 变量，常量 token 进 `@theme`，`@theme inline` 只放指向第 1 层的映射。
- [x] **核对 token 名不变**：比对范围为 `src/styles/**`（不是 `tokens.css` 单文件——第 1 层变量会在批次 2 移出该文件）。迁移前在 `dev` 上采一次基线名字集合，迁移后再采一次，两集合相等（AC13、`design.md` §2 末段）。4,097 处 `var(--)` 引用与契约断言依赖这些名字。
- [x] `chart-colors.css`（5 变量）同步迁移，与 `usage-chart-stability-contracts.md`、`apexcharts-style-contract.smoke.test.ts` 的耦合逐项核对（PRD Notes）。

验证：`bun run build` 成功；切换 `data-theme` 后工具类生效的颜色随之变化（手动一次 + 批次 5 的自动断言）。

### 批次 1 落位决策（与父任务设计的三处偏差说明）

1. **第 1 层变量物理上仍留在 `tokens.css`**，未移入 `themes/`。原因：`theme-contrast-contract` / `apple-glass-surface-contract` / `theme-bootstrap` 三个 smoke 测试直接读 `tokens.css` 文本并按已知选择器清单解析（`KNOWN_SELECTOR_PATTERN` 只接受 `tokens.css` 顶层块形态），把第 1 层变量移出会破坏这些契约，且批次 2 目录分层就是本设计文档规划的动作。决策：批次 1 完成「第 2 层映射 + 分类表 + 名字集合不变」；第 1 层变量的文件级移动在批次 2 随 `theme-token-contracts.md` 重建（批次 8）一并处理，届时同步放宽三个测试的选择器清单。设计文档 §3 的最终落位不变。
2. **`@theme inline` 映射块在 `core.css` 内**（未新建 `tokens.css` 聚合文件）。理由：`core.css` 已是主入口且已承载同类映射，`tokens.css` 若加 `@import` 自引用会破坏三个测试的文本断言。批次 1 在既有 `@theme inline` 块中补齐了组件实际使用但此前缺失的 4 个工具类可达语义色映射：`--color-accent-danger`、`--color-accent-info`、`--color-accent-primary-hover`、`--color-border-accent`。这些名字都在 448 名集合内（accent-danger/info 来自 theme.css 兼容桥），无新增 token 名。
3. **常量 token 的物理落位**：`tokens.css` 的 `:root` 常量块已具备「单上下文、全主题同值」形态，被 `@theme`（非 inline）或 `@theme inline` 引用；本轮不复制进 `@theme`，避免值双写漂移。常量类判定已完整记入 `token-classification.md`，批次 2 落位时按表迁移。

### 批次 1 证据

**分类表**：`.trellis/tasks/08-22-design-system/token-classification.md`，448 行，0 未分类（脚本 `.trellis/tasks/08-22-design-system/classify-tokens.mjs` 生成；按唯一名统计：可切换语义变量 87、常量 token 117、计算/别名 token 74）。

**名字集合比对（AC13，范围 `src/styles/**`）**：

```bash
# before（迁移前基线，与本任务改动前等价）
cd ccr-ui/src/styles && find . -name '*.css' -print0 | xargs -0 rg -o --no-filename -e '--[a-z0-9-]+\s*:' \
  | sed -E 's/.*(--[a-z0-9-]+)\s*:.*/\1/' | sort -u > .trellis/tasks/08-22-design-system/token-names-before.txt
# after（批次 1 完成后）
cd ccr-ui/src/styles && find . -name '*.css' -print0 | xargs -0 rg -o --no-filename -e '--[a-z0-9-]+\s*:' \
  | sed -E 's/.*(--[a-z0-9-]+)\s*:.*/\1/' | sort -u > .trellis/tasks/08-22-design-system/token-names-after.txt
diff .trellis/tasks/08-22-design-system/token-names-before.txt .trellis/tasks/08-22-design-system/token-names-after.txt
# → 无输出（两集合相等），各 426 个唯一名。tokens.css 内 448 个定义点/278 个唯一名。
```

**工具类可达性验证（dist 产物）**：`.hover\:bg-accent-primary-hover:hover{background-color:var(--color-accent-primary-hover)}`、`.bg-accent-danger{background-color:var(--color-danger)}`、`.hover\:border-border-accent:hover{border-color:var(--color-border-accent)}` 等规则生成，全部引用第 1 层运行时变量而非内联字面量。

**chart-colors 耦合核对**：`apexcharts-style-contract.smoke.test.ts` 只读 `src/utils/apexChartsCore.ts` 与 apexcharts dist 样式表，不读 `chart-colors.css`；`chart-colors.css` 的 5 个变量本就是 `var()` 别名到第 1 层，符合两层结构，无需改动。`usage-chart-stability-contracts.md` §5 的 ApexCharts 双路径契约未受影响。

**验证命令与退出码**：

| 命令 | 退出码 | 结果 |
| --- | --- | --- |
| `bun run build` | 0 | ✓ 构建成功 |
| `bun run test:smoke` | 0 | 60 files / 294 tests 全绿（含 theme-switch 新增用例与三个 style-coupled 契约） |
| `bun run lint:style` | 0 | stylelint 无告警 |
| `bun run lint:ci` | 0 | eslint + stylelint + check:style-lines 全绿 |
| `just frontend-check-quick` | 0 | type-check + lint:ci + i18n + smoke 全绿 |
| `bun x vitest run tests/theme-switch.smoke.test.tsx` | 0 | 主题切换用例通过 |

**theme-switch smoke 测试（新增 `ccr-ui/tests/theme-switch.smoke.test.tsx`）**：jsdom 不解析普通属性内 `var()` 链，故分两段验证完整链条——(1) 断言 `@theme inline` 映射存在且工具类规则引用第 1 层变量（非内联字面量）；(2) 注入 `tokens.css` 后断言 `:root` 上 `--color-bg-surface-rgb` 在 light/dark/clay 三态下分别等于 tokens.css 锚点值（`251 252 253` / `34 36 42` / `254 250 242`）且互不相同。通过即证明「切换 `data-theme` 后工具类生效的颜色随之变化」。

## 批次 2：styles 目录分层

- [x] 按 `design.md` §3 把 18 个文件落位到 `base/` / `themes/` / `components/` / `utilities/` 与根。
- [x] 四个页面级样式文件逐个判定归属：`codex-auth-shared.css`、`home.css`、`profiles-page.css`、`checkin-shared.css`。判定记录落盘。
- [x] 空目录填充或删除（AC3）。
- [x] 主入口的 `@import` 顺序确定，三层 CSS 加载语义（`shell-critical` / `deferred-decorations` / `deferred-interactive`）保留。

验证：`ls src/styles/{base,components,themes,utilities}` 无空目录（AC3）；首屏 CSS 只含 `shell-critical` 层（`08-22-arch-quality-perf` 的 `code-splitting.md` 约定）。

### 批次 2 落位决策（一处偏差 + 一处顺带落地）

1. **`themes/` 目录本批次移除（AC3）**。批次 1 落位决策 1 已记录：三个 smoke 契约
   （`theme-contrast-contract` / `apple-glass-surface-contract` / `theme-bootstrap`）直接解析
   `tokens.css` 文本与选择器（`KNOWN_SELECTOR_PATTERN` 只接受 tokens.css 顶层块形态），
   `theme-switch.smoke.test.tsx` 亦断言 tokens.css 内 `--color-bg-surface-rgb` 锚点。
   批次 2 保持 `tokens.css` 为第 1 层可切换变量的定义点，`themes/` 无内容故按 AC3 移除；
   批次 8（`theme-token-contracts.md` 重建）随 `design.md` §1 的目标结构重建 `themes/`，
   届时同步放宽三个 smoke 测试的选择器清单。设计文档 §3 的最终落位不变。
   `animations/` 空目录同理移除，批次 7 判定后按需重建。
2. **三层 CSS 加载的 React 落地顺带完成**。`code-splitting.md` §3.1 记录 React 侧
   `deferred-*` 两层「尚无等价加载点，归 08-22-design-system」。批次 2 新增
   `src/utils/deferredStyles.ts`（首帧后 `scheduleAfterPaint` 载 `deferred-interactive`，
   空闲 `scheduleWhenIdle` 载 `deferred-decorations`，`<link data-style="deferred-*">` 幂等挂载），
   `main.tsx` 调用；旧 Vue `main.ts` 的挂载形态等价。`fonts.css` 无导入点属现状（旧 Vue
   经 `/fonts/...` URL 直载字体子集，不经该文件），未在本批次改动。

### 批次 2 证据

**落位表（old → new）**：完整表见 `page-styles-disposition.md` §1。摘要：
`base.css`→`base/base.css`、`fonts.css`→`base/fonts.css`、`home.css`→`components/home.css`、
`codex-auth-shared.css`→`components/codex-auth-shared.css`、
`profiles-page.css`→`components/profiles-page.css`、`checkin-shared.css`→`components/checkin-shared.css`、
`utilities.css`→`utilities/utilities.css`；`tokens.css` / `theme.css` / `chart-colors.css` /
`core.css` / `index.css` / `shell-critical.css` / `deferred-*` / `backgrounds.css` /
`animations.css` / `components/surfaces.css` 留根或原位（surfaces.css 原已在 `components/`）。
全部 `git mv`，无文件改名、无变量改名、无值变更。

**四个页面级文件判定（`page-styles-disposition.md` §2）**：`home.css`（单域局部 token）、
`codex-auth-shared.css`（单域多组件共享，`code-splitting.md` 明示需留首屏）、
`checkin-shared.css`（单域多组件共享）、`profiles-page.css`（多路由共享，三平台 profiles 共用）
——按「被多路由共享的进 components/」标准，批次 2 全部落 `components/`；
`features/<域>/.module.css` 归阶段 5 视图子任务（features/ 目录尚不存在，消费方均为死 .vue）。

**空目录（AC3）**：`base/`、`components/`、`utilities/` 均有内容；`themes/`、`animations/`
空目录已移除。`find src/styles -type d` 无空目录。

**`@import` 顺序**：`index.css` 首屏链 = `core.css` → `components/checkin-shared.css` →
`components/codex-auth-shared.css`；`core.css` 链 = theme(theme) → utilities(utilities) →
tokens → theme → base/base(layer base) → components/home(layer components) →
shell-critical(layer components) → components/surfaces(layer components)；
`deferred-interactive.css` = animations → utilities/utilities → chart-colors（+ `@tailwind components`）。
层序语义（theme < base < components < utilities）与 @layer 结构不变。

**三层 CSS 加载（dist 产物核对）**：`dist/index.html` 只含一个
`<link rel="stylesheet" href="/assets/index-*.css">`（首屏）；`deferred-interactive-*.css`（20.27 kB）
与 `deferred-decorations-*.css`（1.61 kB）为独立产物，经 `deferredStyles.ts` 的 `?url` 惰性导入。
首屏 index css 含 `.loading-spinner`（shell-critical）且不含 deferred 专属内容
（glass-panel / gradient-shift / float-gentle 均 0 命中）；deferred-interactive 含
utilities/animations/chart-colors，deferred-decorations 含 backgrounds。

**名字集合比对（AC13）**：批次 2 后重跑
`rg -o -- '--[a-z0-9-]+\s*:' src/styles --glob '*.css'` → 426 个唯一名，
`diff` 对 `token-names-before.txt` **无输出**（移动不改名）。

**测试/配置路径更新（仅路径，断言未动）**：`tests/apple-glass-surface-contract.smoke.test.ts`
两处（`checkin-shared.css` → `components/checkin-shared.css`、`utilities.css` →
`utilities/utilities.css`）；`.stylelintrc.json` 的 checkin-shared 覆盖路径；三个死 .vue
的 `@/styles/profiles-page.css` → `@/styles/components/profiles-page.css`。
`tokens.css` / `core.css` / `deferred-decorations.css` 路径未变，对应断言原样保留。

**验证命令与退出码**：

| 命令 | 退出码 | 结果 |
| --- | --- | --- |
| `bun run build` | 0 | ✓ 构建成功，deferred 两层独立产物 |
| `bun run test:smoke` | 0 | 60 files / 294 tests 全绿（含三个 tokens.css 解析契约与 theme-switch） |
| `bun run lint:style` | 0 | stylelint 无告警 |
| `bun run lint:ci` | 0 | eslint + stylelint + check:style-lines 全绿 |
| `bun run check:bundle-budget` | 0 | PASS，largest-lazy 为 deferred-decorations 0.07 KiB |
| `just frontend-check-quick` | 0 | type-check + lint:ci + i18n + smoke 全绿 |

## 批次 3：原语层

- [x] 手写原语普查：按 `design.md` §6 的特征 `rg` 出 Dropdown / Tooltip / Popover / Tabs / Accordion / Combobox 的手写实现与调用点，`adhoc-primitives.md` 落盘（27 个手写实现：Dropdown 6、Popover 3、Tabs 11、Accordion 3、Combobox 2、Tooltip 2，含 §10 替换映射）。
- [x] 原语层落位到 `src/ui/`（父任务 `design.md` §2）：9 类 Radix 原语（dialog/popover/dropdown-menu/tooltip/tabs/combobox/select/switch/checkbox）+ `cn.ts` + 桶导出。`src/components/ui/` 16 个 .vue 为死代码，随阶段 5 迁移消失。
- [x] 接入 shadcn/ui，覆盖 9 类。依赖：@radix-ui/react-{dialog,popover,dropdown-menu,tooltip,select,switch,checkbox} + tailwind-merge；core.css 补 --radius-2xl/3xl/full 映射防 Tailwind 默认字面量绕过 token。
- [x] 每类原语写一个消费示例（AC4）：`tests/ui-primitives.smoke.test.tsx` 9 个行为用例（Dialog Esc 关闭、Tabs 面板切换、Select 选项选择等）全绿。
- [x] 16 个现有原语逐个核对现有用法后确认判定，`primitive-disposition.md` 落盘（AC6）：16 行无空缺，消费方计数经 grep 实测（SIcon 128、PageShell/PageHeader 44、Button 38 等）；三处对 §6 初判的修正已记录（Breadcrumb/NavItem/IconWrapper 实际 0 消费者）。
- [x] 判定为「保留并改消费新 token」的原语改写完成——**偏差记录**：16 个原语均为 .vue 死代码，改写发生在各自 React 移植时（shell-port / views 阶段），不在本批次执行；`primitive-disposition.md` 即为查表依据。
- [x] 判定为「shadcn/ui 替换」的原语，其调用点改动由视图子任务执行，本批次只提供替换映射（`adhoc-primitives.md` §10）。

### 批次 3 证据

| 命令 | 退出码 | 结果 |
| --- | --- | --- |
| `bun run type-check` | 0 | ✓ |
| `bun run lint:ci` | 0 | eslint + stylelint + check:style-lines 全绿 |
| `vitest run --config vitest.smoke.config.ts tests/ui-primitives.smoke.test.tsx` | 0 | 9/9 通过 |
| `just frontend-check-quick` | 0 | 61 文件 / 303 测试全绿（新增 1 文件 9 用例） |
| `src/ui/*.tsx` 硬编码扫描 | — | px 字面量 / rgba() / hex 均为 0（token 工具类承载） |

注：直接 `bun x vitest run <file>` 不带 `--config vitest.smoke.config.ts` 会落入 node 环境导致 jsdom 全局缺失（MouseEvent/window undefined），属调用方式问题而非代码缺陷。

验证：`bun run type-check`；9 类原语各有示例可渲染。

## 批次 4：弹层收口

- [x] Dialog 作为唯一底座，四项行为（焦点陷阱、Esc、滚动锁定、层级）只有一处实现。
- [x] `BaseModal` API 适配器落地。**判定：适配器保留**（见下「批次 4 落位决策」）。
- [x] 13 个自实现 `fixed inset-0` 弹层的替换方案记录，交由各视图子任务执行（`adhoc-primitives.md` §8 清单 + §10 替换映射，入口统一为 `src/ui/base-modal.tsx` 适配器）。
- [x] smoke 测试断言四项行为只有一处实现（AC5）：`tests/overlay-single-implementation.smoke.test.ts` 3 用例。

### 批次 4 落位决策与实现发现

1. **适配器保留（design.md §7 末段的判定）**：适配器仅做 API 翻译（`modelValue`/`onUpdateModelValue`/`onClose`/`onOpened`/`header`/`footer`/`ref.close()` → Radix 受控形态）加一个拖拽阈值判定，不含四项弹层行为的任何实现，复杂度远低于改 33 个调用点，判定为保留。33 个 `.vue` 调用点在阶段 5 迁移时按文件头「Vue → React API 映射」查表转换。
2. **Radix Dialog 1.1.23 的三个实测行为**（适配器与测试均按实测行为编写）：
   - `DialogContent` 无 `onPointerUpOutside` prop（前一轮实现误用导致 type-check 失败）；
   - `deferPointerDownOutside: true`：外部 pointerdown（button 0）的判定推迟到随后的 `click` 才派发 `onPointerDownOutside`，custom event 只携带原始 pointerdown——拖拽阈值所需的 pointerup 坐标由适配器在打开期间用 document 级监听自行记录；
   - Content 不再输出 `aria-modal` 属性（dist 内无该字符串，modal 语义由 `role="dialog"` + Radix 焦点管理承载），测试按实际行为断言并在用例内注明核实结论。
3. **负向用例的有效性**：拖拽超阈值 / `closeOnBackdrop=false` / `persistent` 三个负向用例都在正向用例（同序列关闭成功）证明路径可达之后执行，避免「事件未到达、断言空过」的假阴性。Radix 的 document 级 pointerdown 监听在 `setTimeout(0)` 后注册，测试用 `settleRadixOutsideDetection()`（让出一个宏任务）保证探测就绪。
4. **复杂度上限（2c 的 max 16）触发拆分**：`BaseModal` 首版复杂度 23 超限，拆出 `ModalHeader` / `ModalBody` / `ModalFooterBar` / `ModalCloseButton` 子组件与 `isBackdropClick` 判定函数后通过。

### 批次 4 证据

改动：新增 `src/ui/base-modal.tsx`（适配器）、`tests/base-modal-adapter.smoke.test.tsx`（13 用例：Esc 三态、遮罩点击三态 + 阈值、滚动锁定锁定/解除、aria 接线、onOpened、ref.close()、header/footer 插槽映射、showClose）、`tests/overlay-single-implementation.smoke.test.ts`（3 用例：`src/ui` 无滚动锁定/Esc/焦点陷阱自实现、无 `onPointerUpOutside` 回归、层级 token 在 styles 层有定义）；`src/ui/index.ts` 补桶导出。临时探针文件（`probe-radix.smoke.test.tsx`、`probe-out.txt`）已删除。

| 命令 | 退出码 | 结果 |
| --- | --- | --- |
| `bun run type-check` | 0 | ✓ |
| `bun run lint:ci` | 0 | eslint（含复杂度 max 16）+ stylelint + check:style-lines 全绿 |
| `vitest run --config vitest.smoke.config.ts tests/base-modal-adapter.smoke.test.tsx` | 0 | 13/13 通过 |
| `vitest run --config vitest.smoke.config.ts tests/overlay-single-implementation.smoke.test.ts` | 0 | 3/3 通过 |
| `bun run test:smoke` | 0 | 63 文件 / 319 测试全绿（批次 3 为 61/303，新增 2 文件 16 用例） |
| `just frontend-check-quick` | 0 | type-check + lint:ci + i18n + smoke 全绿 |

## 批次 5：主题配置域

- [x] 按 `design.md` §10 让 `FlavorMode` 与 `AccentMode` 值域可扩展：新增成员只需改类型联合 + `FLAVOR_MODES`/`ACCENT_MODES` 加一项 + 第 1 层变量加一组定义。结构性保证由测试断言（见证据用例 3：三个 `data-*` 属性的写入点在 `src` 内只有 `themeBootstrap.ts`，组件侧无成员级代码依赖）。
- [x] 加一个测试用的新 flavor 与新 accent，验证界面正确响应（AC7）：`tests/theme-domain-extension.smoke.test.tsx` 用例 1、2（注入测试值 `ink-test` / `sage-test` 的第 1 层定义块，断言属性切换后 `--color-bg-surface-rgb`、`--color-accent-primary` 重解析且工具类仍引用运行时变量）。测试内的联合 cast 模拟「联合已扩展、调用点照旧」。
- [x] `themeBootstrap` 的自定义 accent 输入所需的变量结构就位：`applyCustomAccent` / `clearCustomAccent` / `CustomAccentDefinition` / `CUSTOM_ACCENT_VARIABLE_FAMILY`（8 变量族，与 `[data-accent='clay']` 块集合一致，明暗两块由单一主色推导 hover/active/glow/contrast/border）。接线归 `08-22-shell-port` R6。用例 4 断言整族写入、明暗分别生效、非法输入拒绝、清除后恢复。
- [x] 存储键 `ccr-theme` 等的旧值可正常解析：既有 `tests/theme-bootstrap.smoke.test.ts` 已覆盖（`migrates legacy flavor/accent values`、`writes migrated values back`、first-paint IIFE 对齐与非法值回退共 9 个用例），本批次无新增改动，引为证据。

### 批次 5 证据

改动：`src/utils/themeBootstrap.ts`（+129 行至 470 行，max-lines 500 内）新增自定义 accent 原语；新增 `tests/theme-domain-extension.smoke.test.tsx`（4 用例）。

| 命令 | 退出码 | 结果 |
| --- | --- | --- |
| `bun run type-check` | 0 | ✓ |
| `bun run lint:ci` | 0 | ✓ |
| `vitest run --config vitest.smoke.config.ts tests/theme-domain-extension.smoke.test.tsx` | 0 | 4/4 通过 |
| `bun run test:smoke` | 0 | 64 文件 / 323 测试全绿（批次 4 为 63/319） |
| `just frontend-check-quick` | 0 | 全绿 |

## 批次 6：CSS 侧硬编码收口

- [ ] `.css` 内 290 处 px 与 102 处 hex 映射到 token。
- [ ] `hardcode-mapping.md` 落盘：常见字面量到 token 名的查表映射，供七个视图子任务使用。
- [ ] `hardcode-exemptions.md` 落盘：图表与画布等确需字面量的场景逐条登记。
- [ ] `0.75rem` 字号例外保留并在映射表中标注。

`.tsx` 内的 2,591 处（1,639 px + 932 rgba + 20 hex）不在本批次，随各视图迁移收口。该部分归 AC12，由父任务视图门核对，不是本任务交付门的准出条件。本任务只保证 `src/styles/**` 侧归零与映射表可用（AC1、AC2）。

## 批次 7：动画与 reduced motion

- [ ] `animations.css` 580 行按 `design.md` §8 逐段判定，`animation-disposition.md` 落盘（含起止行、选择器、动画类型、判定、理由）。
- [ ] 逐段检查属性重叠，确认无同一元素同一属性由 CSS 与 motion 双驱动。
- [ ] `src/styles/animations/` 空目录填充或删除。
- [ ] 按 `design.md` §9 把 reduced motion 收敛到一处，`@media` 兜底的去留记录判定。
- [ ] `prefers-reduced-motion` 下动效降级生效（AC8 的一半）。

## 批次 8：契约重写与验证

- [ ] `theme-token-contracts.md`（31.5 KB）重写。与 `08-22-test-contract-rebuild` 协同（PRD Notes：该文档不宜独立完成）。
- [ ] 保留 `0.75rem` 字号例外的说明（R10、AC9）。
- [ ] 保留三层主题模型语义说明。
- [ ] `brand-asset-pipeline.md`（4.4 KB）同步核对，是否受本任务影响。
- [ ] 明暗对比度检查：每个语义色对计算 WCAG 对比度，与迁移前同名 token 对比（AC8）。
- [ ] token 单点生效验证用例（`design.md` §12），断言 3 个域同时变化（AC11）。

验证：`rg '\.vue|<script setup|scoped' .trellis/spec/ccr-ui/frontend/theme-token-contracts.md` 无匹配（AC9）；`bun run lint:style` 退出码 0（AC10）。

## 验证命令

| 时机           | 命令                                                       |
| -------------- | ---------------------------------------------------------- |
| 每批次后       | `bun run type-check`、`bun run lint:style`                 |
| 批次 1–2 后    | `bun run build`                                            |
| 批次 3–5、7 后 | `bun run test:smoke`                                       |
| 交付前         | `just frontend-check-quick`、`bun run check:bundle-budget` |

## 交付门（父任务约束门的另一半）

- [ ] AC1–AC11 与 AC13 全部满足。
- [ ] **AC12 不在本门**：`.tsx` 侧的 px 与 `rgba()` 归零由七个视图子任务执行、父任务视图门核对。本任务的责任止于 `src/styles/**` 归零 + `hardcode-mapping.md` 可用。
- [ ] 七份记录落盘：`token-classification.md`、`primitive-disposition.md`、`adhoc-primitives.md`、`hardcode-mapping.md`、`hardcode-exemptions.md`、`animation-disposition.md`、四个页面级样式文件的归属判定。
- [ ] token 名集合迁移前后一致，比对范围 `src/styles/**`（批次 1 的核对项、AC13）。
- [ ] 9 类 shadcn/ui 原语在 `src/ui/` 下可用，各有消费示例。
- [ ] `theme-token-contracts.md` 重写完成。
- [ ] 无 `Neko` / `anime` / `purple-tech` / `guofeng` 相关命名、色板或组件语义（R9）。

## 回滚点

| 批次 | 回滚方式                                                                  |
| ---- | ------------------------------------------------------------------------- |
| 1    | token 两层结构回滚。风险最高的一步——4,097 处 `var()` 引用依赖它。单独提交 |
| 2    | 目录移动。单独提交，`@import` 顺序一并回滚                                |
| 3–4  | 原语与弹层。可按原语粒度分多次提交                                        |
| 5、7 | 主题域与动画，各自单独提交                                                |
| 6    | CSS 侧硬编码收口。逐文件提交，可精确回退                                  |
| 8    | 契约文档，revert 无代码影响                                               |

## 协同点

| 编号 | 内容                                              | 对方                          | 时机      |
| ---- | ------------------------------------------------- | ----------------------------- | --------- |
| K    | `animations.css` 逐段去留与 `motion` 引入协同     | `08-22-dep-upgrade`           | 批次 7    |
| H    | `ui/` 原语接口与 `MasterDetailLayout` 共同定稳    | `08-22-shell-port`            | 批次 3 后 |
| —    | `theme-token-contracts.md` 重写需协同，不独立完成 | `08-22-test-contract-rebuild` | 批次 8    |
| —    | `hardcode-mapping.md` 是七个视图子任务的查表依据  | 七个视图子任务                | 批次 6 后 |
| —    | `primitive-disposition.md` 的替换映射供调用点改写 | 七个视图子任务                | 批次 3 后 |
| —    | 组件内样式行数上限约束本任务产出                  | `08-22-arch-quality-perf`     | 全程      |
| —    | `chart-colors.css` 的耦合项                       | `08-22-views-usage`           | 批次 1    |
