# 修复 ApexCharts tooltip marker 大圆闪现

## Goal

Usage 页及其他 ApexCharts 页面在首次挂载、路由/标签往返和样式运行时节点异常时，
都不得把 tooltip 的内部 marker SVG 放大成占据卡片的大圆；正常 tooltip、图表布局和
现有按需加载行为必须保持完整。

用户价值：消除遮挡数据且会自行消失的高干扰视觉故障，并让第三方图表的布局契约不再
依赖单个易失的运行时样式节点。

## Background And Confirmed Facts

- 用户截图中的大圆已通过“首次挂载前阻止 `#apexcharts-css`”的受控故障注入在真实
  Usage 页稳定复现，最佳证据为 `no-apex-css-repro.png`；常规流程尚未自然复现。
- 大圆是 ApexCharts 5.16.0 默认 tooltip marker：SVG 使用
  `viewBox="0 0 12 12"`，圆为 `r=5`，SVG 本身没有 `width` / `height`。
- 正常情况下，上游主样式把 marker host 固定为 `12x12`、隐藏未激活 tooltip group，
  并把 tooltip 设为绝对定位浮层。
- 主样式缺失时，项目 `ccr-ui/src/styles/base.css:67` 的全局 SVG reset 与无尺寸 marker
  组合，使 SVG 扩展到卡片内容宽度。836px 宿主实测产生 `836x836` SVG 和
  约 696.67px 圆；比例与用户截图一致。
- donut 第一次处理 tooltip 交互后会给 group 和 marker 写入内联 `display`，大圆因此
  消失。这解释了用户观察到的完整时间序列。
- 真实 WebView2 正常态中的 `#apexcharts-css` 为 24,436 字符、134 条规则且未禁用；
  常规页面往返、筛选和主题切换没有自然触发丢失。用户现场为何缺失或失效仍未知。
- 上游初次 `render()` 只按 ID 信任该节点，update/donut updateSeries 不重新检查完整性，
  destroy 也不删除它；这是已确认的恢复缺口，不是现场失效 actor 的证据。
- 根因证据、候选方案差分和未确认边界见 `research/root-cause.md`。

## Requirements

### R1 完整样式契约

- ApexCharts 主样式必须通过应用构建系统拥有独立、可验证的交付路径。
- 该路径必须随图表按需加载，不能无条件进入无图表页面的首屏关键样式。
- 上游现有运行时样式注入必须保留为第二条交付路径，任一路径可用时图表均应正常。

### R2 故障行为

- 在首次挂载前阻止 `#apexcharts-css` 运行时节点创建时，tooltip group 仍必须初始隐藏，
  marker host 仍必须保持 12px 尺寸，tooltip 仍必须是绝对定位浮层。
- 不得只把大圆缩小而留下 100% 宽、静态占位或错误可见的 tooltip DOM。
- 不能依靠首次 hover、自动刷新、路由往返或 reload 才恢复。

### R3 作用域与兼容性

- 保持 `apexcharts` / `vue3-apexcharts` 当前版本范围、模块化入口和图表类型注册方式。
- 不修改全局 SVG reset，不复制一组不完整的 ApexCharts 内部 selector，不 patch 上游包。
- Usage 趋势图、donut、Tokens/Cost 图表、Claude Observer 和 Platform Usage 的现有
  ApexCharts 行为不得回归。
- 不改变图表数据、主题、tooltip 文案、动画、KeepAlive 或 Tauri 后端行为。

### R4 回归证据

- 保留一个自动化 smoke contract，验证应用入口显式拥有 ApexCharts 主样式，并锁定
  marker 尺寸、group 初始隐藏和 tooltip 定位所需的关键上游规则。
- 使用 `research/apexcharts-tooltip-marker-probe.mjs` 在真实 WebView2 验证正常路径和
  “首次挂载前阻止运行时 CSS”路径。
- 构建产物必须实际包含完整 ApexCharts CSS，而不是只有源码字符串断言。

### R5 结论边界

- 变更说明必须区分“已确认的 DOM/CSS/交互机制”和“尚未定位的现场触发源”。
- 不得把未自然复现的样式删除、HMR、CSP 或主题时序假设写成已确认根因。

## Acceptance Criteria

| ID | Observable acceptance |
| --- | --- |
| AC1 | 正常真实 WebView2 首次挂载 Usage 后，探针输出 `healthy=true`、0 个 giant marker、marker host `12px`、group 初始 `display:none`、tooltip `position:absolute`。 |
| AC2 | 首次挂载前阻止所有 `#apexcharts-css` append 后，AC1 的全部条件仍成立，且探针确认 runtime style 不存在、构建管理的关键规则仍可用。 |
| AC3 | donut hover 后 tooltip 是紧凑浮层，不进入文档流，不改变 distribution card 高度；marker 不遮挡自定义 legend。 |
| AC4 | focused smoke、type-check、lint、production build、`just frontend-check-quick` 和最终 `just frontend-check` 全部通过；production CSS asset 含关键 tooltip marker/group 规则，且仍是图表异步入口的预加载依赖而非首屏直链样式。 |
| AC5 | 最终 diff 不包含全局 reset、依赖版本、Usage 数据/主题/KeepAlive、Tauri 后端或其他无关改动。 |
| AC6 | 研究与交付说明明确记录现场样式失效触发源仍未定位，不把故障注入当作自然触发证据。 |

## Out Of Scope

- 在无法自然复现的前提下猜测或声称定位用户现场的 style 删除/失效触发源。
- 修复 Vue host 存在但 `.apexcharts-canvas` 变为 0 的独立 KeepAlive/实例生命周期问题。
- 更换、降级或锁死 ApexCharts，修改 tooltip 视觉设计，或重做全局媒体 reset。
- 处理与本几何无因果关系的主题检测、legend 或其他第三方图表问题。
