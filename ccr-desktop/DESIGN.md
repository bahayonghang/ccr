---
name: CCR GPUI Desktop
description: Codex 风格的低彩度原生工作台，连续工作面、清晰文字层级与即时操作反馈。
colors:
  primary: "#000000"
  secondary: "#FFFFFF"
  surface: "#FFFFFF"
  neutral: "#F7F7F6"
  elevated: "#F0F1F2"
  text: "#1F2328"
  text-muted: "#646B74"
  border-subtle: "#E6E7E9"
  border-control: "#787F89"
  focus: "#111827"
  success: "#166534"
  warning: "#92400E"
  danger: "#B91C1C"
  primary-dark: "#E7E9EC"
  secondary-dark: "#151617"
  surface-dark: "#151617"
  neutral-dark: "#1C1E20"
  elevated-dark: "#232527"
  text-dark: "#E7E9EC"
  text-muted-dark: "#A1A7B0"
  border-subtle-dark: "#34373B"
  border-control-dark: "#737B86"
  focus-dark: "#E7E9EC"
  success-dark: "#86EFAC"
  warning-dark: "#FCD34D"
  danger-dark: "#FCA5A5"
typography:
  h1:
    fontFamily: "platform-system"
    fontSize: 20px
    fontWeight: 600
    lineHeight: 1.3
  body-md:
    fontFamily: "platform-system"
    fontSize: 14px
    fontWeight: 400
    lineHeight: 1.5
  label-caps:
    fontFamily: "platform-monospace"
    fontSize: 12px
    fontWeight: 400
    lineHeight: 1.5
  sourceScale: "12/14/16/20/24/32"
  weights: "400, 500, 600"
rounded:
  sm: 4px
  md: 8px
  pill: 9999px
  surface: 0px
spacing:
  sm: 4px
  md: 8px
  sourceScale: "4/8/12/16/24/32"
---
<!-- design-md: entry=style:codex; source=https://github.com/bergside/awesome-design-skills@f631a09; license=MIT -->
<!-- customized: CCR native app shell, system fonts and CJK fallback, light/dark semantic contrast, compact data layouts, keyboard/IME states and event-driven performance budgets. -->

## Overview

视觉主张：低彩度、近乎平面的连续工作面，由文字层级和对齐关系建立秩序。内容主张：先定位环境与平台，再显示真实状态，最后在当前上下文完成操作。交互主张：键盘与高频动作即时响应，指针按压给出短反馈，没有装饰性持续动画。

用户是同时管理多个 AI CLI 平台、配置和用量的桌面开发者。设计签名是清晰的主列表与详情区、等宽数据列和极少量黑白主操作，不用市场终端琥珀条或同形指标卡墙。原有 Tauri DESIGN.md 不变，新目录是独立产品设计根。

采用用户指定的 style:codex，不复制 OpenAI 标识或应用品牌。主题默认跟随系统，用户可选 light/dark；两主题均必须验收。生产栈为 GPUI，样式策略是 Rust 语义 token -> GPUI Kit Theme，不使用 CSS/Tailwind 或第二主题系统。

## Colors

主操作只用 primary/secondary 反色，平台身份为小图标与文字，不给整个页面染色。success/warning/danger 仅表示真实状态，始终配文字或图标。上涨/下降不是成功/失败，不默认使用红绿。

surface 是主工作区，neutral 是侧栏/选中背景，elevated 供弹层。使用色阶与空间分区，border-subtle 仅装饰分隔；需要表达控件边界时必须用 border-control。焦点使用 2px focus 环和 2px 间隔。

普通文字、辅助文字、状态文字与其允许背景达到 4.5:1，焦点与关键控件边界达到 3:1。禁用文字保留可读对比度，通过禁用交互、图标与语义状态降低强调，不统一透明化整组内容。来源中的绿色/橙色已调暗以满足浅色正文对比度。

## Typography

| 角色 | 字号 / 字重 | 用法 |
| --- | --- | --- |
| 页面标题 | 20 / 600 | 页面主标题，每屏一个 |
| 分区标题 | 16 / 600 | 列表或编辑分区 |
| 正文/控件 | 14 / 400 或 500 | 文案、按钮、输入 |
| 元信息/数据 | 12 / 400 | 时间、版本、辅助信息 |
| 主要数值 | 24 / 500 | 仅有明确分析价值时使用 |

Windows 用 Segoe UI Variable / Segoe UI，中文由 Microsoft YaHei UI 等系统 fallback；macOS 用系统 UI 字体与 PingFang；Linux 用可用系统 sans 与 Noto CJK fallback。等宽数字使用系统 mono（Consolas / Menlo / DejaVu Sans Mono 等）。GPUI 字体选择不是 CSS 字符串，按 OS 和可用字体配置 text system，再实测 CJK fallback。

使用系统字体是为避免启动字体下载和大字体资源；不打包商业字体。标签不强制英文大写；中文不加宽字距。20px 标题字距 -0.012em，14px 正文字距 0。数值右对齐，label 左对齐；无数量时显示“暂无数据”或不适用状态，不能用假零填充。

## Layout

默认窗口 1280×800 logical px，最小 960×640。无营销 hero、插画背景或页脚 CTA。侧栏 216px，顶部工具行 48px，底部任务状态区 28px；主内容 padding 24px，窄窗 16px。间距来自 4/8/12/16/24/32。

侧栏上方固定环境入口，功能分组为概览、配置/平台、资源、用量/会话、任务/同步、设置；平台选择只呈现真实 capability。默认首页先看当前环境与就绪原因，再看配置/任务状态，必要的用量摘要采用一条读数行。

主列表与详情默认 3:2，列表至少 360px，详情至少 320px；空间不足时详情转为内容区内单页 drill-in，带明确返回操作，不把普通溢出塞进 modal。列表行 36px，表头 32px，表格独立滚动；不要让整个窗口横向滚动。分页与 virtualization 必须保留键盘选择和 stable ID。

## Elevation & Depth

常驻区块不使用阴影或玻璃效果。层次仅由 surface/neutral/elevated、留白及必要细分隔建立。只在真正覆盖上下文的菜单/对话框使用轻阴影，不能给每个数据块加容器。不要嵌套卡片表达层级。

## Shapes

主/次按钮均 pill，输入 8px，列表可选背景 4px，连续工作面 0px。图标使用一个集合，优先 Kit 的 Lucide 资源，实际依赖/资源开销在构建时记录。主操作不使用 emoji 图标。

保留 Codex 条目的克制与柔和控件；数据列表不用 pill 包每一项。只有语义独立的弹层使用 8px 外角。

## Components

- **按钮**：normal 用 primary/secondary 或透明次按钮；hover 提升一个中性色阶；focus 显示 2px 环；pointer pressed 可瞬时 scale(0.96) 并在 100ms 内复原，hitbox 不变；keyboard activation 仅即时状态反馈，无动画。disabled 不触发，loading 保持宽度且有文字状态，error 配相邻说明。
- **输入**：14px 文本、8px 角、border-control；focus/invalid 分别使用 focus/danger 与文字。支持组合输入、选区、复制粘贴、撤销与系统编辑键。密码遮蔽默认开启，明文不得进入截图或日志。
- **导航**：当前选中以 neutral/elevated 色阶和字重表达，没有粗侧边条。Tab 与方向键行为按原生语义区分，不与文本编辑键冲突；支持 F6 在侧栏/主区/状态区切换。
- **表格/列表**：列角色固定，数字等宽右对齐。行 hover 与 selection 用中性色阶，频繁点击行不缩放，避免文字抖动，这是高频数据操作的明确运动例外。过长名称可 tooltip 补全，关键数值/操作名必须完整或按语义缩短，不能靠省略号掩盖。
- **任务反馈**：queued/running/cancel_requested/terminal 状态可见。页内进度文本与可恢复错误优先，toast 不作为唯一结果渠道。取消请求不得立即写“已取消”。
- **确认**：只用于不可逆/敏感操作的业务确认，列明对象和后果；普通导航与展开不用弹窗。Esc 返回安全状态，焦点回到触发者。
- **图表**：有明确问题才显示；趋势用线、比较用条形。高度 160 至 240px，图例与 exact-value table 可达；低对比色必须有表格/文字通道。数据更新不播放入场动画。
- **状态**：empty 说明原因并给已有下一步；loading 保留布局；error 显示可重试原因；unsupported 与未接入明确区分。演示 fixture 必须带“演示数据”标识，生产无伪造数字。

## Do's and Don'ts

### Do

- 用真实配置、真实任务和就绪原因建立首页信息层级。
- 将 DB/扫描/网络移到后台，只更新相关 entity。
- 用稳定 ID 虚拟化长列表，让滚动与后台导入并行。
- 为每个操作实现键盘焦点、busy/disabled/error 与返回路径。
- 在 light/dark、中文/英文、最小窗口及 DPI 变化下复核。

### Don't

- 不使用默认卡片墙、渐变文字、紫蓝霓虹、模糊玻璃或装饰背景。
- 不在 render 中读取文件/SQLite、创建 runtime、启动进程。
- 不用每帧轮询、常驻 spinner 或隐藏页面刷新维持“活跃感”。
- 不把平台色扩散到背景，不把变化值自动解释为成功/失败。
- 不用浏览器 mock 截图声称 GPUI 原生 UI 已验证。

## Responsive Behavior

| 宽度 | 布局 |
| --- | --- |
| ≥ 1200px | 216px 侧栏 + list/detail，主区 padding 24px |
| 1040 至 1199px | 侧栏保留，按可用宽度切到 list/detail drill-in |
| 960 至 1039px | 184px 侧栏，主区 padding 16px，详情为单页 |
| < 960px | 不支持作为桌面最小窗口，不缩字号硬挤 |

分别验收 960×800、1280×640、960×640、1280×800，不只检查一个等比例缩小样本。测试 100/150/200% DPI 与跨显示器移动。键鼠桌面密集控件可为 32/36px 高，这是对 web 44px touch-target 默认的显式例外；触摸模式若未来进入范围需另做 44px 目标，不假称已有移动端支持。

## Iteration Guide / Agent Prompt Guide

1. 先改本文件，再改唯一 Rust token 层。新增角色不能偷偷复用不同含义 token。
2. “实现配置列表：surface #FFFFFF、正文 #1F2328 14px/400、行高36px、选中 #F0F1F2/4px、数值等宽右对齐、主区padding24px、标题20px/600/-0.012em；采用虚拟化并提供键盘选择。”
3. “实现深色任务区：背景 #151617、正文 #E7E9EC 14px/400、辅助 #A1A7B0 12px、控件边界 #737B86、输入角8px；取消中与取消完成分开显示，不播放列表入场动画。”
4. “实现保存按钮：黑底白字14px/500、pill角、逻辑高度36px、水平padding16px、focus #111827 的2px环与2px间隔；pointer按压0.96，键盘无动画，loading保持宽度。”
5. Significant change 后从真实窗口重新看第一屏：环境是否清楚、状态是否真实、主操作是否明确、是否新增无意义容器。

## Known Gaps and Native QA

03 已将语义 token 实现在 `src/theme.rs` 并通过 Kit Theme 同步 Base/Component；`src/shell/` 呈现已接入的 Local 配置、命令收藏/历史与会话内主题/语言设置。列表采用稳定领域 ID 的虚拟化，详情提供返回，F6/Tab 与编辑键分别处理。无持续装饰动画；按钮即时 pressed 色反馈，常用行操作不做缩放。

01/02 已有 Windows 实际原生输入、读取筛选、错误恢复与正常退出证据；03 新外壳的原生核心验证另记任务 evidence。2026-10-18 用户明确不做窗口尺寸、缩放或 DPI 矩阵，该部分 user-deferred，不能记通过。保留设计最小窗口与自适应实现，并不以未运行矩阵阻止已批准后续工程。

待分别验证：03 原生 light/dark、中英文、中文 IME、字体 fallback、焦点恢复、屏幕阅读器 role/name/state，以及 macOS/Linux 实际原生运行。按钮同节点 disabled/busy 标记已实现；现有 Kit snapshot/debug 接口不能完整观测这些 flags，不能以 None 或 headless 标签证明系统读屏已通过。03 的普通输入保留编辑能力；04 的 Inspect/导出临时区使用 readonly，并有实际控件输入抑制测试。确认决策按钮使用 Kit 原有 focus trap；这些 headless 结果不替代系统读屏验证。未检测到系统 reduced-motion API，应用不启用装饰性运动，键盘即时响应。

当前不存在需要“换成真实内容”的业务占位数据；实施 fixture 只用于测试，生产数据应通过 ccr-app/领域 API 注入，不能在 view 中硬编码用户数、余额或认证成功。

04 配置集与配置目录复用连续列表/详情工作面，编辑区在内容区展开，字段超长时独立滚动。破坏性操作显示对象、后果和确认/取消；源码、导出和转换使用明文提示与临时文本区，离开页面清除缓冲。Grok 的部分完成状态和 CAS 冲突持续显示在页内，不使用成功 toast 掩盖未完成阶段。04 的真实原生核心流程由父会话另行采集，本条实施说明不等于原生验收或性能通过。
