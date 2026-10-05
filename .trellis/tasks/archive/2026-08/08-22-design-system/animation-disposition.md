# animations.css 逐段判定（批次 7，协同点 K）

> 08-22-design-system 批次 7 交付物。判定标准（design.md §8）：进出场动画（挂载/卸载触发）与
> 布局位移动画交 motion；装饰性持续动画、一次性反馈动画、纯 `:hover` 过渡保留 CSS；弹层进出场
> 由批次 4 定稳的 Radix `data-state` CSS 过渡承载。硬约束：同一元素同一属性禁止 CSS 与 motion 双驱动。
>
> 判定后的 `animations.css` 已按本表执行（580 行 → 约 140 行保留段）；删除段的消费方（死 .vue）
> 在阶段 5 迁移时按「替代承载」列选择实现，不得恢复被删类名。

## 一、关键帧与工具类判定

| 原行号 | 内容 | 动画类型 | 判定 | 替代承载 / 保留理由 | 属性重叠核对 |
| --- | --- | --- | --- | --- | --- |
| 9-28 | `fade-in` / `fade-out` + `.animate-fade-*` | 进出场 | **删除** | motion `<AnimatePresence>` / `initial`+`animate` | motion 接管 opacity/transform，CSS 类已删，无双驱动 |
| 31-80 | `slide-up/down`、`slide-in-left/right` + 类 | 进出场位移 | **删除** | motion `initial={{ y: 10 }}` 等 | 同上 |
| 83-106 | `scale-in` / `scale-out` + 类 | 进出场缩放 | **删除** | motion / Radix data-state 过渡 | 同上 |
| 111-120 | `pulse-subtle` + 类 | 装饰性持续（状态指示） | **保留** | 无 motion 参与 | — |
| 123-131 | `spin` | 功能性持续（加载指示） | **删除（本文件内）** | `utilities.css` 的 `.loading-spinner` 就地持有同名关键帧（shell-critical.css 同），且 Tailwind v4 自带 `animate-spin` 默认类；保留会成第三处实现 | 单一实现归 utilities |
| 134-146 | `bounce-in` + 类 | 一次性点击反馈 | **保留** | 微交互走 CSS 零 JS 依赖 | — |
| 149-169 | `shake` + 类 | 一次性错误反馈 | **保留** | 同上 | — |
| 172-184 | `gradient-shift` + 类 | 装饰性持续（品牌渐变） | **保留** | background-position 无 motion 竞争 | — |
| 187-196 | `border-glow` | 装饰性持续（选中发光） | **保留**（关键帧保留；工具类本就无） | border-color 无 motion 竞争 | — |
| 199-209 | `sidebar-item-enter` | 进出场（列表项入场） | **删除** | motion 列表 stagger | motion 接管 |
| 212-222 | `card-enter` | 进出场 | **删除** | motion | motion 接管 |
| 225-259 | `modal-enter/exit`、`backdrop-enter` | 弹层进出场 | **删除** | 批次 4 `src/ui/dialog.tsx` / `base-modal.tsx` 已用 Radix `data-[state=open/closed]` + `transition-interactive` 工具类实现等价进出场（CSS 单套，非双驱动） | 弹层只用 CSS 过渡一套 |
| 277-370 | `.animate-*` 工具类汇总 | 随各关键帧 | 删除进出场类；保留 pulse/bounce/shake/gradient-shift 类 | 见各行 | — |
| 312-328 | `will-change` 提示块 | 进出场类的 GPU 提示 | **删除** | 选择器全部是被删类 | — |
| 386-418 | `.animate-delay-*` / `.animate-fill-*` | 进出场类的配套 | **删除** | motion 自带 `delay` / 过渡结束态 | — |
| 420-424 | `.gpu-accelerate` | 工具类（非动画） | **保留** | 与动画机制无关 | — |
| 426-444 | `.hover-animate`、`.nav-hover-effect` | 纯 `:hover` 过渡 | **保留** | §8 判定标准明确保留 | — |
| 446-480 | Vue `.page-*` / `.scale-fade-*` 过渡类 | Vue Transition 专用 | **删除** | `08-22-shell-port` R5 用 motion 改写路由过渡（Vue 命名类在 React 侧无意义） | shell-port 落地时核对 |
| 482-580 | 路由感知 `.page-fade/slide-up/slide-down/cross-fade/slide-lateral-*` | Vue Transition 专用 | **删除** | 同上（depth/group 比较逻辑迁到布局组件，动画值迁 motion variants） | shell-port 落地时核对 |

**保留集**：`pulse-subtle`、`bounce-in`、`shake`、`gradient-shift`、`border-glow` 关键帧 +
`.animate-pulse-subtle` / `.animate-bounce-in` / `.animate-shake` / `.animate-gradient-shift` +
`.gpu-accelerate` / `.hover-animate` / `.nav-hover-effect`。

## 二、双驱动核查（当前时点）

- `motion` 13.1.1 已在依赖中，**src 内尚无任何 import**（`rg "from 'motion"` 无匹配）——当前时点不存在 CSS/motion 双驱动。
- shell-port 与视图子任务引入 motion 时按本表执行：被删段的元素不得再挂保留 CSS 动画类；
  Radix 弹层（dialog/base-modal）只用 `data-state` CSS 过渡，motion 不得再对其 Content/Overlay 的
  opacity/transform 施加动画。

## 三、reduced motion 单点收敛（design.md §9）

- 新增 `src/utils/reducedMotion.ts`：唯一读 `matchMedia('(prefers-reduced-motion: reduce)')` 的模块，
  把结果写入根元素 `data-reduced-motion='true'|'false'` 并跟随系统变化；`main.tsx` 启动时调用。
- `src/styles/**` 的 5 处 `@media (prefers-reduced-motion: reduce)` 全部改为挂
  `[data-reduced-motion='true']` 属性选择器（base.css 通配降级、utilities.css 卡片/按钮降级、
  home.css 局部 token、profiles-page.css spinner、shell-critical.css spinner）。
- **@media 兜底保留判定：保留一处**，位置 shell-critical.css。理由：首帧 JS 未执行时 critical 层的
  `.loading-spinner` 是无 JS 期间唯一可见的动画，属性门控在 JS 前不可用，须 @media 兜底；
  其余四层均在 JS 之后加载或作用于 JS 渲染的元素，属性门控等价。
- `.vue` 组件内散写的 `@media (prefers-reduced-motion)`（约 15 个文件）属阶段 5 视图迁移范围，
  迁移时按本节约定改为消费 `data-reduced-motion` 属性或 motion 的 `reducedMotion="user"`。
- `useAnimationVisibility.ts`（Vue composable，死代码图）的 reduced-motion 职责由
  `reducedMotion.ts` 承接；其视口/页签可见性职责归 `08-22-state-logic-port` 迁移。

## 四、验证口径（AC8 前半）

- `rg -c '@media \(prefers-reduced-motion' src/styles -g '*.css'` == 1（仅 shell-critical 兜底）。
- `src/styles/animations/` 空目录已在批次 2 移除，本批次保留段为单文件规模，不重建目录。
- 行为断言见 `tests/reduced-motion.smoke.test.tsx`。
