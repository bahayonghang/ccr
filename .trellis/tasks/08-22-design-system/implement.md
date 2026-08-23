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

- [ ] 按 `design.md` §3 把 18 个文件落位到 `base/` / `themes/` / `components/` / `utilities/` 与根。
- [ ] 四个页面级样式文件逐个判定归属：`codex-auth-shared.css`、`home.css`、`profiles-page.css`、`checkin-shared.css`。判定记录落盘。
- [ ] 空目录填充或删除（AC3）。
- [ ] 主入口的 `@import` 顺序确定，三层 CSS 加载语义（`shell-critical` / `deferred-decorations` / `deferred-interactive`）保留。

验证：`ls src/styles/{base,components,themes,utilities}` 无空目录（AC3）；首屏 CSS 只含 `shell-critical` 层（`08-22-arch-quality-perf` 的 `code-splitting.md` 约定）。

## 批次 3：原语层

- [ ] 手写原语普查：按 `design.md` §6 的特征 `rg` 出 Dropdown / Tooltip / Popover / Tabs / Accordion / Combobox 的手写实现与调用点，`adhoc-primitives.md` 落盘。
- [ ] 原语层落位到 `src/ui/`（父任务 `design.md` §2）。`src/components/ui/` 迁移后不再存在。
- [ ] 接入 shadcn/ui，覆盖 9 类：Dialog、Popover、DropdownMenu、Tooltip、Tabs、Combobox、Select、Switch、Checkbox。
- [ ] 每类原语写一个消费示例（AC4），放 `src/ui/__examples__` 或 smoke 测试内。
- [ ] 16 个现有原语逐个核对现有用法后确认判定，`primitive-disposition.md` 落盘（AC6）。
- [ ] 判定为「保留并改消费新 token」的原语改写完成。
- [ ] 判定为「shadcn/ui 替换」的原语，其调用点改动由视图子任务执行，本批次只提供替换映射。

验证：`bun run type-check`；9 类原语各有示例可渲染。

## 批次 4：弹层收口

- [ ] Dialog 作为唯一底座，四项行为（焦点陷阱、Esc、滚动锁定、层级）只有一处实现。
- [ ] `BaseModal` API 适配器落地。若其复杂度超过直接改 33 个调用点，取消适配器并记录判定（`design.md` §7 末段）。
- [ ] 13 个自实现 `fixed inset-0` 弹层的替换方案记录，交由各视图子任务执行。
- [ ] smoke 测试断言四项行为只有一处实现（AC5）。

## 批次 5：主题配置域

- [ ] 按 `design.md` §10 让 `FlavorMode` 与 `AccentMode` 值域可扩展：新增成员只需改类型联合 + 加一组第 1 层变量。
- [ ] 加一个测试用的新 flavor 与新 accent，验证界面正确响应（AC7）。
- [ ] `themeBootstrap` 的自定义 accent 输入所需的变量结构就位。接线归 `08-22-shell-port` R6。
- [ ] 存储键 `ccr-theme` 等的旧值可正常解析，写一个断言。

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
