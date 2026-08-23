# 技术设计：Tailwind v4 与 shadcn/ui 设计体系重建

> 父任务：`08-22-react-migration`。样式承载决策见父任务 `design.md` §6，动画决策见 §9。本文件写 token 两层结构、原语判定方法与硬编码收口方案。

## 1. token 的两层结构（本任务最关键的设计点）

Tailwind v4 的 `@theme` 与 `@theme inline` 行为不同：

- `@theme` 定义的变量，工具类通过 `var(--x)` 引用，变量本身输出到 `:root`。
- `@theme inline` 定义的变量，其值被内联进工具类，不产生额外一层 `var()` 间接。

本仓需要运行时切换主题（`data-theme` / `data-flavor` / `data-accent` 三层模型），因此 token 必须分两层：

```css
/* 第 1 层：可切换的语义变量。普通 CSS 变量，不在 @theme 内 */
:root {
  --surface-1: <暖中性亮色>;
}
[data-theme="dark"] {
  --surface-1: <石墨深色>;
}
[data-flavor="clay"] {
  --surface-1: <clay 变体>;
}
[data-accent="clay"] {
  --accent-1: <clay 强调色>;
}

/* 第 2 层：Tailwind 命名空间，指向第 1 层 */
@theme inline {
  --color-surface-1: var(--surface-1);
  --color-accent-1: var(--accent-1);
}
```

结果：`bg-surface-1` 展开为 `background-color: var(--surface-1)`，运行时随 `data-theme` 切换重解析。

若把可切换的值直接写进 `@theme`（非 inline），主题切换需要覆盖 Tailwind 自己生成的变量，层叠关系变复杂；若写进 `@theme inline` 但值是字面量，则工具类内联死值，主题切换失效。两层结构是唯一同时满足「工具类可用」与「运行时可切换」的形态。

448 个变量按此分两层的分类方法见第 2 节。

**两个集合的命名**（PRD Scope 与 AC13 用同一组名字，避免三处口径不一致）：

| 名称                       | 内容                                          | 落位                             |
| -------------------------- | --------------------------------------------- | -------------------------------- |
| 稳定语义变量集合           | 第 1 层的可切换语义变量                       | `src/styles/themes/` 普通 CSS 变量 |
| Tailwind namespace 映射集合 | 第 2 层，值形如 `var(--<语义变量>)`           | `@theme inline`                  |
| 常量 token 集合            | 全主题同值                                    | `@theme`（非 inline）            |

三个集合的变量名并集 == 迁移前 `tokens.css` 的 448 个名字（AC13）。因此名字集合的比对**不能只查 `tokens.css` 单文件**——第 1 层已移出该文件。比对范围为 `src/styles/**`，见第 2 节末段。

## 2. 448 个变量的分类

`src/styles/tokens.css` 现 26.7 KB / 448 变量。分三类：

| 类             | 判据                                                                     | 落位                                                 |
| -------------- | ------------------------------------------------------------------------ | ---------------------------------------------------- |
| 可切换语义变量 | 在 `[data-theme]` / `[data-flavor]` / `[data-accent]` 选择器下有不同取值 | 第 1 层（普通 CSS 变量）                             |
| 常量 token     | 全主题同值（间距、圆角、字号、字重、时长、层级 z-index）                 | 直接进 `@theme`（非 inline），由 Tailwind 生成工具类 |
| 计算 token     | 值由其他变量计算得出（`calc()`、`color-mix()`）                          | 跟随其输入变量的类别                                 |

分类方法：对每个变量名 `rg` 其在 `tokens.css` 与 `themes/` 下的全部定义点，出现在 2 个以上选择器下即为可切换。

分类表落盘为 `token-classification.md`，448 行，无未分类项。

**4,097 处 `var(--)` 引用的 token 名不变**（父任务 `design.md` §6）。这是本任务的硬约束：分类只改变量的定义位置，不改变量名。名字一改，4,097 处引用与 `theme-token-contracts.md` 的断言同时失效。

**名字集合的比对范围是 `src/styles/**`，不是 `tokens.css` 单文件。** 第 1 层变量在批次 2 移入 `themes/`，只查 `tokens.css` 会把「已移动」误判为「已删除」。比对命令的形态：

```
rg -o -- '--[a-z0-9-]+\s*:' src/styles --glob '*.css' | 抽变量名 | sort -u
```

迁移前在 `dev` 上跑一次作为基线，迁移后在 `src/styles/**` 上跑一次，两个集合相等（AC13）。

## 3. `src/styles/` 分层落位

现状 18 个文件 / 4,026 行，另有 4 个空目录（`base/`、`components/`、`themes/`、`utilities/`）。

目标落位：

| 目录          | 内容                                                                                                                                                                                |
| ------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `base/`       | reset（现 `base.css`，`preflight: false` 下自带）、根字号、文档级默认                                                                                                               |
| `themes/`     | 第 1 层可切换语义变量，按 `data-theme` / `data-flavor` / `data-accent` 分文件                                                                                                       |
| `components/` | 页面级与组件级共享样式。三个按页面聚合的文件在此判定：`codex-auth-shared.css`（14.8 KB）、`home.css`（41 变量）、`profiles-page.css`（28 变量）、`checkin-shared.css`（1,136 字节） |
| `utilities/`  | `@utility` 定义与自定义工具类（含 `08-22-dep-upgrade` 段 2 迁来的 1 处 `addComponents` plugin，若判定为工具类形态）                                                                 |
| 根            | 主入口（`@import` 顺序）、`tokens.css`（`@theme` 部分）、`chart-colors.css`                                                                                                         |

四个页面级样式文件的判定标准：其规则是否只服务单一路由。只服务单一路由的进对应 `features/<域>/` 下的 `.module.css`；被多路由共享的进 `components/`。判定逐文件记录。

空目录填充或删除，不保留空目录（R3、AC3）。

## 4. 组件内样式的处理

24,434 行局部样式，覆盖 139 / 185 个组件。承载方式为「Tailwind 工具类为主，残余进 CSS Modules」（父任务 `design.md` §6）。

「残余」的判定标准（本任务定，七个视图子任务执行）：

| 进 `.module.css`                                  | 进工具类                                                   |
| ------------------------------------------------- | ---------------------------------------------------------- |
| 后代与兄弟选择器（`>`、`+`、`~`、空格组合）       | 单元素的属性设置                                           |
| `@keyframes` 与 `animation` 简写                  | 布局、间距、颜色、字体、边框、阴影                         |
| 伪元素内容（`::before` / `::after` 的 `content`） | 伪类状态（`hover:` / `focus:` / `disabled:` 有工具类前缀） |
| 媒体查询中的复杂重排                              | 响应式断点（`sm:` / `md:` 前缀可表达）                     |
| 第三方库注入的类名覆盖（ApexCharts、CodeMirror）  | —                                                          |

约束：单组件的 `.module.css` 行数不超过其 `.tsx` 行数（`08-22-arch-quality-perf` 批次 3 的检查脚本强制）。

`@apply` 在 `.module.css` 内需 `@reference`（`08-22-dep-upgrade` §3 已处理 25 个现有文件；本任务与视图子任务新增的 `.module.css` 同样需要）。判定：新增 `.module.css` 尽量不用 `@apply`——它在 CSS Modules 内的收益低于直接写 CSS 属性，且多一个静默失效面。

## 5. 硬编码收口

| 类型                         | 数量  | 映射目标                 |
| ---------------------------- | ----- | ------------------------ |
| `.vue` 内 px                 | 1,639 | 间距 / 字号 / 圆角 token |
| `.vue` 内 `rgba()` / `rgb()` | 932   | 颜色与材质 token         |
| `.vue` 内 hex                | 20    | 同上                     |
| `.css` 内 px                 | 290   | 同上                     |
| `.css` 内 hex                | 102   | 同上                     |

**分批策略**：按 `views/` 域分批，与七个视图子任务的批次对齐（PRD Notes）。本任务只负责 `.css` 内的 392 处（290 px + 102 hex）与 token 映射表；`.vue` 内的 2,591 处随各视图迁移收口，由各视图子任务的 AC 检查（其 AC4 / AC9 / AC11 项）。

**映射表**：本任务产出 `hardcode-mapping.md`，把常见的字面量值映射到 token 名。例如 `1px` → 边框 token、`8px` / `12px` / `16px` → 间距阶、`rgba(0,0,0,0.08)` → 阴影 token。该表是视图子任务的查表依据，避免七个子任务各自决定映射。

**豁免登记**：图表与画布确需字面量的场景逐个登记（PRD Scope）。登记格式：文件、行、字面量值、原因。落盘为 `hardcode-exemptions.md`。

`theme-token-contracts.md` 已登记的 `0.75rem` 字号例外（Profiles 共享层密集元信息，低于 Label 下限 `0.8125rem` 一档）在新体系中保留（R10、AC9）。

## 6. 原语层

### 16 个现有原语的判定

`AsyncStatePanel`、`Badge`、`Breadcrumb`、`Button`、`Card`、`EmptyState`、`IconWrapper`、`Input`、`NavItem`、`PageHeader`、`PageShell`、`PillToggleGroup`、`SIcon`、`Sparkline`、`Spinner`、`StatTile`（合计 2,201 行）。

判定标准：

| 判定                 | 条件                                                                                                                                |
| -------------------- | ----------------------------------------------------------------------------------------------------------------------------------- |
| shadcn/ui 替换       | shadcn/ui 有对应原语且行为覆盖现有用法（`Badge`、`Button`、`Card`、`Input`、`Breadcrumb` 属此类）                                   |
| 保留并改消费新 token | 本仓特有的组合或语义（`PageShell`、`PageHeader`、`NavItem`、`AsyncStatePanel`、`EmptyState`、`StatTile`、`PillToggleGroup` 属此类） |
| 保留不变             | 与样式体系无关的纯渲染封装（`SIcon`、`IconWrapper`、`Spinner`、`Sparkline` 属此类，`Sparkline` 需核对是否消费 `chart-colors.css`）  |

上表的归类为初判，实施时逐个核对现有用法后确认。判定表落盘为 `primitive-disposition.md`，16 行无空缺（AC6）。

### 9 类 shadcn/ui 原语接入

Dialog、Popover、DropdownMenu、Tooltip、Tabs、Combobox、Select、Switch、Checkbox。

现状缺口：`Dropdown` / `Tooltip` / `Popover` / `Tabs` / `Accordion` / `Combobox` 命名文件各 0 个，对应交互由手写 div 承担。**接入前需先普查定位**：`rg` 手写实现的特征（`aria-expanded`、`role="tablist"`、`role="menu"`、`onMouseEnter` 配 `position: absolute` 的组合），产出手写实现清单与其调用点。清单落盘为 `adhoc-primitives.md`。

清单是视图子任务的替换依据。不做普查则手写实现在迁移时被逐个照搬成 React 版本，缺口保持不变。

每类原语需一个消费示例（AC4）。示例放在 `ui/` 下的 `__examples__` 或 smoke 测试内，不放业务代码。

## 7. 弹层收口

现状：33 个文件引用 `BaseModal.vue`；13 个文件自行实现 `fixed inset-0`；18 个 `*Modal.vue` + 5 个 `*Dialog.vue`。

目标：焦点陷阱、Esc 关闭、滚动锁定、层级管理只有一处实现（AC5）。

方案：shadcn/ui 的 Dialog 作为唯一底座（其底层为 Radix UI，四项行为由 Radix 提供）。`BaseModal` 的 API 形态在 Dialog 之上包一层适配器保留，使 33 个调用点的改动面最小。13 个自实现的弹层改为走该适配器。

适配器的存在理由是调用点数量（33 处），不是行为差异。若实施时发现 `BaseModal` 的 API 与 Dialog 差异过大导致适配器复杂度超过直接改 33 个调用点，取消适配器。该判定记录。

18 个 `*Modal` + 5 个 `*Dialog` 的具体组件由各视图子任务迁移，本任务只提供底座与适配器。

## 8. 动画（与 motion 协同）

`animations.css` 580 行的逐段判定（父任务 `design.md` §9、协同点 K）。

分段方法：按 CSS 规则块的语义分段，每段记录：起止行、选择器、动画类型、判定、理由。

| 判定        | 条件                                                                             |
| ----------- | -------------------------------------------------------------------------------- |
| 交给 motion | 进出场动画（元素挂载 / 卸载时触发）、布局位移动画                                |
| 保留 CSS    | 装饰性持续动画（背景、光效）、纯 `:hover` / `:focus` 过渡、`@keyframes` 定义本身 |

**硬约束**：不允许 CSS 动画与 motion 对同一元素的同一属性并存。判定时逐段检查该段影响的属性是否与 motion 接管的属性重叠。

12 处 Vue `Transition` → `AnimatePresence`（卸载动画由其接管）。其中落在 `08-22-shell-port` 范围内的部分由该任务改写（其 R5），本任务提供约定。

`src/styles/animations/` 空目录：若保留 CSS 的段落有多个文件规模，填充；否则删除。

判定结果落盘为 `animation-disposition.md`。

## 9. reduced motion 收敛

现状两处逻辑：散在多个组件的 `@media (prefers-reduced-motion)`，以及 `useAnimationVisibility.ts`。

目标：单一实现。方案为一个 hook 读系统偏好，同时驱动 motion 的 reduced motion 行为与 CSS 侧的一个根级 data 属性（如 `data-reduced-motion`），CSS 侧的降级规则统一挂在该属性下，不再散写 `@media`。

该方案下 `@media (prefers-reduced-motion)` 只出现一次——在设置根 data 属性的地方，或完全由 JS 读取。两条路径二选一，选择标准是首屏无 JS 时降级是否需要生效。若需要，保留一处 `@media` 兜底。

`useAnimationVisibility.ts` 的其余职责（若有超出 reduced motion 的部分）由 `08-22-state-logic-port` 的 composable 迁移覆盖，本任务只处理其 reduced motion 部分。

## 10. 主题配置域可扩展

现状值域：`FlavorMode = 'neutral' | 'clay'`、`AccentMode = 'clay'`、`DEFAULT_FLAVOR = 'neutral'`、`DEFAULT_ACCENT = 'clay'`。

可扩展的含义（R7）：新增一个 flavor 或 accent 只需两处改动——类型联合加一个成员，`themes/` 下加一组第 1 层变量定义。不需要改任何组件。

`themeBootstrap` 支持自定义 accent 输入：接受一个颜色值而非枚举成员，运行时写入第 1 层的 `--accent-*` 变量。该能力的接线在 `08-22-shell-port`（其 R6），本任务提供变量结构。

存储键 `ccr-theme` 等视觉偏好键的读写兼容保留，旧值可正常解析（Scope）。

## 11. 视觉方向约束

遵循 `ccr-ui/CLAUDE.md` 的 Design Context：品牌气质 `克制 / 准确 / 编辑式`，Anthropic-like 编辑式工作台，暖中性色表面，高对比排版，克制的材质深度。

禁止引入或延续 `Neko` / `anime` / `purple-tech` / `guofeng` 分支（R9）。这些是待移除的历史方向。

明暗两套主题对比度不低于迁移前（R8、AC8）。验证方式：对每个语义色对（前景 / 背景）计算 WCAG 对比度，与迁移前的同名 token 对比。

## 12. token 单点生效验证（AC11）

「改一处 token 值可同时影响所有消费点」的验证用例：

1. 选一个被 3 个以上域消费的 token（如 `--surface-1`）。
2. 在测试中改其值。
3. 断言 3 个不同域的组件渲染出的计算样式同时变化。

该用例是本任务「降低改样式成本」这一目标的唯一可执行证据。

## 13. 未决项

- 448 个变量的具体分类结果，按第 2 节的方法测量后确定。
- 16 个原语的最终判定，按第 6 节的初判逐个核对后确定。
- `BaseModal` 适配器是否保留，按第 7 节末段的判定。
- reduced motion 的 `@media` 兜底是否保留，按第 9 节末段的判定。
- 四个页面级样式文件的归属，按第 3 节末段的标准逐文件判定。
