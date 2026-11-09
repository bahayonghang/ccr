---
name: CCR GPUI Desktop
description: 轻盈玻璃原生工作台，静态薄荷紫光场、并列的半透明导轨与纸面、实色阅读面与清晰的绿色操作。
colors:
  primary: "#156F4E"
  secondary: "#FFFFFF"
  surface: "#FFFFFF"
  neutral: "#E7F1EE"
  elevated: "#F2F7F5"
  text: "#1E2932"
  text-muted: "#5C6B76"
  border-subtle: "#CEDBD5"
  border-control: "#787F89"
  focus: "#156F4E"
  success: "#166534"
  warning: "#92400E"
  danger: "#B91C1C"
  primary-dark: "#85D5B1"
  secondary-dark: "#172225"
  surface-dark: "#1D292C"
  neutral-dark: "#10191B"
  elevated-dark: "#172225"
  text-dark: "#F0F8F5"
  text-muted-dark: "#9FAFA9"
  border-subtle-dark: "#35474A"
  border-control-dark: "#737B86"
  focus-dark: "#85D5B1"
  success-dark: "#86EFAC"
  warning-dark: "#FCD34D"
  danger-dark: "#FCA5A5"
  # 材质：仅覆盖应用自绘的静态低频场景；8 位十六进制末两位为 alpha。
  scene-start: "#E7F6F1"
  scene-end: "#EDE7F8"
  scene-glow-mint: "#BFEADB8C"
  scene-glow-lavender: "#D9CDF58C"
  chrome: "#FFFFFF94"
  sheet: "#FFFFFFC2"
  highlight: "#FFFFFFCC"
  tab-well: "#E7F1EEB3"
  scene-start-dark: "#102B26"
  scene-end-dark: "#25213D"
  scene-glow-mint-dark: "#1F4A4080"
  scene-glow-lavender-dark: "#3A2F6680"
  chrome-dark: "#1D292C8C"
  sheet-dark: "#172225DB"
  highlight-dark: "#FFFFFF14"
  tab-well-dark: "#10191B99"
  info: "#245CB3"
  info-dark: "#9BC4F5"
  # 平台身份：只用于 8px 圆点、2px 细线与图表系列；omp 等未列来源回落 text-muted。
  platform-claude: "#A15840"
  platform-codex: "#547458"
  platform-grok: "#726985"
  platform-antigravity: "#617081"
  platform-opencode: "#735F52"
  platform-kimi: "#A3467F"
  platform-pi: "#7B56B0"
  platform-claude-dark: "#E3987B"
  platform-codex-dark: "#89BD91"
  platform-grok-dark: "#B9ABD7"
  platform-antigravity-dark: "#A9C1DC"
  platform-opencode-dark: "#C9B6A8"
  platform-kimi-dark: "#D775AE"
  platform-pi-dark: "#AC88E7"
  # 状态标签与热力图由 ink/primary 按 alpha 预合成到 surface，结果不透明：
  # 标签 fill light 10% / dark 18%，装饰 edge light 24% / dark 32%；热力图 1-4 档 25/45/70/100%。
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
  display-kpi:
    fontFamily: "platform-system"
    fontSize: 32px
    fontWeight: 600
    lineHeight: 1.2
    fontFeature: "tnum"
  numeric-md:
    fontFamily: "platform-system"
    fontSize: 24px
    fontWeight: 500
    lineHeight: 1.2
    fontFeature: "tnum"
  sourceScale: "12/14/16/20/24/32"
  weights: "400, 500, 600"
rounded:
  sm: 4px
  md: 8px
  panel: 12px
  pill: 9999px
  capsule: 20px
  surface: 20px
spacing:
  sm: 4px
  md: 8px
  sourceScale: "4/8/12/16/24/32"
---
<!-- design-md: entry=style:airy-glass; adaptation=.trellis/tasks/10-17-gpui-desktop/research/airy-glass-plan.md -->
<!-- customized: CCR native app shell, system fonts and CJK fallback, light/dark semantic contrast, compact data layouts, keyboard/IME states and event-driven performance budgets. -->

## Overview

视觉主张：静态 160deg 薄荷绿至淡紫光场之上，并列漂浮半透明侧栏导轨与带亮边的纸面；表格、输入和弹层等阅读面保持实色，由文字层级和对齐关系建立秩序。内容主张：先定位环境与平台，再显示真实状态，最后在当前上下文完成操作。交互主张：键盘与高频动作即时响应，指针按压给出短反馈，没有装饰性持续动画。

用户是同时管理多个 AI CLI 平台、配置和用量的桌面开发者。设计签名是清晰的主列表与详情区、等宽数据列和克制的绿色主操作，不用市场终端琥珀条或同形指标卡墙。原有 Tauri DESIGN.md 不变，新目录是独立产品设计根。

采用已批准的 style:airy-glass 增量合同，与 Tauri 共享工作台语言；保留原生系统字体和密度。主题默认跟随系统，用户可选 light/dark；两主题均必须验收。生产栈为 GPUI，样式策略是 Rust 语义 token -> GPUI Kit Theme，不使用 CSS/Tailwind 或第二主题系统。

## Colors

主操作、链接和焦点使用绿色（light #156f4e / dark #85d5b1），有独立对比文字色；明确输出使用紫色（light #6d4bd6 / dark #b9a4f4）。选中背景保持克制的实色阶，平台身份为小图标与文字，不给整个页面染色。success/warning/danger 仅表示真实状态，始终配文字或图标。上涨/下降不是成功/失败，不默认使用红绿。

reading 是列表/输入的实色工作区，chrome 是侧栏导轨的半透明材质，sheet 是主纸面的半透明材质，elevated 是纸面的实色参照色，overlay 是实色弹层；neutral 仍是实色中性色阶，供页内 hover/读数井使用。使用色阶与空间分区，border-subtle 仅装饰分隔；需要表达控件边界时必须用 border-control。焦点使用 2px focus 环和 2px 间隔。

装饰 hairline 使用 border-subtle 的 light 24% / dark 35% alpha；需要辨识的输入与 tab 边界仍使用不透明 border-control。分段控件是内容宽度胶囊：tab-well 半透明凹槽容器加 1px highlight 亮边，选中项为浮起的实色 pill（light surface / dark overlay、轻阴影、primary semibold 文字），未选中项透明无边框；避免 ghost 的 selected 与纸面同色。选中文字对选中面至少 4.5:1；选中面与凹槽的色差很小，选中态由浮起阴影、主色文字、字重与 toggled 语义共同表达，不只靠颜色。

状态语调（`theme::tone_colors`）共 neutral / success / warning / danger / info 五种，ink 分别取辅助文字与对应状态色。fill 是 ink 在 light 10% / dark 18%、edge 是 ink 在 light 24% / dark 32% 预合成到 reading 的不透明色，不构成额外半透明层，所以标签放在阅读面或纸面上文字都只落在 fill 上：ink 对 fill ≥4.5:1。标签不是控件，由文字识别状态，edge 与 hairline 同属装饰描边，不适用控件边界 3:1；描边加重会让标签读作可点击的描边按钮，削弱“直角标签不可点、pill 可点”的形状区分。平台身份色覆盖 claude / codex / grok / antigravity（gemini）/ opencode / kimi / pi，omp 及其他未列来源回落 muted；身份色只用于 8px 圆点、2px 细线与图表系列，不给面板染色。活动热力图（`theme::heat_level`）共 5 档：0 档是实色 neutral 格，1–4 档是 primary 以 25/45/70/100% 预合成到 reading 的不透明色，对 reading 的对比度逐档递增，第 4 档 ≥3:1。

普通文字、辅助文字、状态文字与其允许背景达到 4.5:1，焦点与关键控件边界达到 3:1。半透明面按合成后颜色验收：在场景两端、中点、两个光斑峰值及叠加纸面阴影的最坏组合上分别合成后计算，正文 ≥12、辅助 ≥7、muted/主色/状态 ≥4.5、控件边界 ≥3。禁用文字保留可读对比度，通过禁用交互、图标与语义状态降低强调，不统一透明化整组内容。来源中的绿色/橙色已调暗以满足浅色正文对比度。

## Typography

| 角色 | 字号 / 字重 | 用法 |
| --- | --- | --- |
| 页面标题 | 20 / 600 | 页面主标题，每屏一个 |
| 分区标题 | 16 / 600 | 列表或编辑分区 |
| 正文/控件 | 14 / 400 或 500 | 文案、按钮、输入 |
| 元信息/数据 | 12 / 400 | 时间、版本、辅助信息 |
| 主要数值 | 24 / 500 | 仅有明确分析价值时使用 |
| 主读数 | 32 / 600 | 运行概览 KPI 带中的成本，每屏最多一个 |

Windows 用 Segoe UI Variable / Segoe UI，中文由 Microsoft YaHei UI 等系统 fallback；macOS 用系统 UI 字体与 PingFang；Linux 用可用系统 sans 与 Noto CJK fallback。读数与精确值表的数字使用系统 UI 字体并开启 OpenType `tnum`（`theme::tabular_figures()`），不用带斜杠零的 mono；Windows 上 Segoe UI Variable 的“1”默认是比例宽，`tnum` 后与其他数字同宽，已用真实 DirectWrite 整形确认（`.trellis/tasks/11-07-gpui-airy-glass-material/research/numeric-font-probe.md`），原生像素与 macOS/Linux 未验收。代码、路径与命令文本仍用系统 mono（Consolas / Menlo / DejaVu Sans Mono 等）。GPUI 字体选择不是 CSS 字符串，按 OS 和可用字体配置 text system，再实测 CJK fallback。

使用系统字体是为避免启动字体下载和大字体资源；不打包商业字体。标签不强制英文大写；中文不加宽字距。20px 标题字距 -0.012em，14px 正文字距 0。数值右对齐，label 左对齐；无数量时显示“暂无数据”或不适用状态，不能用假零填充。

## Layout

默认窗口 1280×800 logical px，最小 960×640。无营销 hero、插画背景或页脚 CTA。侧栏 216px，页内分段胶囊行无边框、无固定高度，底部任务状态区 28px；主内容 padding 24px，窄窗 16px。间距来自 4/8/12/16/24/32。

侧栏上方是产品名和工作台状态，功能分组为运行概览、配置中心、平台、工具、设置。默认首页是运行概览：先看本机是否已连上用量库和会话归档，再看该窗口内已经读到的请求、token、成本和会话，最后是可执行的下一步。平台井只在读取成功后出现；某个来源没有记录就显示无记录，库不存在或查询失败时不铺零值。会话未索引写“未索引”，不把归档里的未知写成 0。工作台初次进入使用浅色，让薄荷绿到淡紫的场景成为第一眼；设置里仍可改回跟随系统或深色。这些读数井是实色，不是嵌套玻璃，也不是营销卡片墙。

运行概览自上而下：

1. **页头一行**：唯一 h1“运行概览” + 外壳 lens 胶囊（Overview / Monitoring）在左，整页作用的 7D/30D/90D 范围胶囊与刷新在右；其下一行状态（来源 · 查询窗口 · 截止日期与时区，演示数据另标），并写新鲜度 `Last successful sync MM-DD HH:mm / 上次成功同步 MM-DD HH:mm`（报告时区）。llmusage 只记录成功的同步；失败的同步仍可能导入部分数据，所以只陈述成功时间，不据此推断哪些日期“未导入”。最后一次成功同步的本地日期早于报告截止日，或没有成功记录（写“未记录成功同步”）时算过期，文字旁加 warning 圆点，并说明“同步成功前，较新的用量可能缺失”。
2. **KPI 带**：一张实色面板内 4 栏，以 hairline 分隔，不做同形卡片墙。成本为主读数（32/600），请求、Token、会话为 24/500；12px muted 标签在上，12px 说明在下，只用已有数据（窗口天数、全历史）；派生的“日均 = 总量 ÷ 窗口天数”必须标“日均 / avg”。成本不可用显示“–”与原因，会话未索引/不可用显示文字状态（16px）与原因。窄宽（< 1040）改为 2×2。
3. **平台带**：只有存在逐日记录的平台单独成格（2px 身份色顶线、中性 sparkline、峰值身份色、请求/Token 读数）；无记录或无逐日明细的来源合并为一格“其他来源”，逐行写“名称 · 原因”，有总数时保留真实总数，不铺 0。窄宽横向滚动，不换行。
4. **主列 + 右栏**：右栏宽 = 内容宽 × 24%，取 8 的倍数后限制在 280–360px，主列占其余宽度。主列依次是逐日请求图（含可展开的精确值表，图表面板不伸展）与活动面板；右栏依次是下一步操作、近 7 日对比、数据源（配置状态、报告截止、上次同步、过期说明、图表来源活跃日覆盖）。等高规则：两列 stretch，每列只有最后一个面板吸收多余高度（活动 / 数据源）：活动面板的内容块垂直居中，数据源内容顶对齐、脚注贴底，吸收的高度只表现为面板内部间距，不形成死区。全部来自同一次报告，不新增查询。
5. **首屏之下的洞察行**：请求分布（Hourly / Weekday 胶囊）与排行（Models / Projects / Agents 胶囊，前 8）两个面板 1:1 等高。

窄宽（< 1040）整页单列：内容 padding 16px，KPI 2×2，右栏排到主列之下，洞察行堆叠，平台带与热力图各自在面板内横滚，整页不横向滚动。

运行概览的模块规则：

- **活动面板**：标题“Activity”，口径“Last 53 weeks · since YYYY-MM-DD”（起点在一年前，保留年份，避免被读成本月）。53 周 × 7 天网格，周一在上，只标周一/周三/周五；晚于报告截止日的格子不画。格子 = clamp(floor((主列宽 − 2 边框 − 32 内边距 − 28 星期标签列 − 52×2 格间距) / 53), 8, 14)，算出小于 8 时取 8 并在面板内横滚，星期标签列固定不动，默认露出最新一周，竖向滚轮仍只滚整页。色阶用 `theme::heat_level`，按非零日的四分位分 1–4 档。只有有记录的格子带 id 和 tooltip（`YYYY-MM-DD · N requests`）。摘要写活跃天、当前连续、最长连续、最忙日，与网格同一 53 周口径；热力图宽 + 200 不超过面板内宽时放在网格右侧，否则放在下方。图例 Less ▢▢▢▢▢ More 紧贴网格下方并与网格右缘对齐（堆叠时摘要排在图例之后）。活动面板为与右栏等高吸收的余量均分在内容块上下（内容块垂直居中），不让图例沉底悬空、也不在面板底部留一整块空白。网格容器的无障碍标签就是摘要句。53 周内没有记录时写“No requests in the last 53 weeks”，不铺 0 格；投影读取失败时用虚线空态面板说明原因。
- **近 7 日对比**：行是 Requests / Sessions / Active days，列是 Last 7、Prior 7、Change（近 7 日 / 前 7 日 / 变化），表头同一句式。变化用中性墨色的上下箭头或“=”加百分比，不用红绿；前期为 0 而本期不为 0 时写“new”，不出现除零。会话归档不可用写“不可用”，未索引写“未索引”，都不写 0。
- **请求分布**：Hourly（24 柱）/ Weekday（7 柱，周一在前）胶囊，高度 160px，中性柱加 primary 峰值柱，说明写“Peak 14:00 · N requests · All history”。全为 0 时画虚线基线加文字。
- **排行**：Models / Projects / Agents 胶囊，取请求数大于 0 的前 8 行。名称单行省略并带 tooltip；Projects 显示路径末段，Agents 用平台显示名。行高 28px，比例条中性底、primary 填充，以当前榜首为满格；数字 tnum 右对齐。无行时写“No records”。
- **口径**：KPI 与逐日图跟随页面范围胶囊；活动面板标 53 周；分布与排行标 All history；对比固定是近 7 日与前 7 日。不同口径的读数必须写明，不能放在同一口径标题下。全平台聚合数据不用平台身份色。

主列表与详情默认 3:2，列表至少 360px，详情至少 320px；空间不足时详情转为内容区内单页 drill-in，带明确返回操作，不把普通溢出塞进 modal。列表行单行 36px / 两行 56–60px，表头 32px，表格独立滚动；不要让整个窗口横向滚动。分页与 virtualization 必须保留键盘选择和 stable ID。

MCP 管理页自上而下：

1. **页头一行**：左侧是唯一 h1“MCP 管理”和外壳 lens 胶囊（MCP Manager / Configuration converter）；右侧是“重新读取”（ghost，refresh-cw）和“添加服务器”（primary，plus），空间不足时整组换行。下一行是 12px muted 描述，只写一句。Claude 工作台复用的 MCP 视图不带胶囊，平台 tab 仍由外壳上下文行承载。
2. **工具栏**：左侧是 `badge::search_field`，placeholder 写“Search name or command / 搜索名称或命令”，按名称或命令做不区分大小写的子串匹配；其后是平台胶囊“All n | Claude Code n | Codex n | Antigravity n”。右侧是“预设”（star）和“批量与同步”（gallery-vertical-end）两个 ghost 按钮。平台计数在搜索之前按 catalog 计算，只列有条目的平台，项目文件来源计入 Claude Code。平台胶囊与搜索取交集，两者都过滤掉全部条目时，用虚线空态说明，并给出“清除搜索与平台”。平台作用域下不显示平台胶囊。
3. **来源健康条**：每个来源文件一个不可交互的 22px、4px 角 chip，内容是“指示符 + 短名 · 读数”。完整原句同时作为 aria label 和 tooltip；窄宽时在条内横向滚动。
   - 有条目：success 实心圆点加条目数。
   - 空：muted 实心圆点加“none”。
   - 未找到：空心圆点。
   - 未读取：空心圆点加虚线边；Grok 写“not listed”。
   - Invalid：triangle-alert，warning 语调；Unreadable：circle-alert，danger 语调。
4. **主体**：窗口宽度 ≥ `SPLIT_WIDTH`（1200）时，列表与 pane 按 3:2 并排，间距 16，列表 ≥ `LIST_MIN_WIDTH`（360），pane ≥ `DETAIL_MIN_WIDTH`（320）。外层列只负责尺寸，不带内边距和描边，实色阅读面板放在列内并填满它。原因是 flex 基准为 0 时，taffy 会把条目自身的 padding + border 抬进基准，比例就不再是 3:2。
   - catalog 没有条目时，宽窗也只放列表列的空态面板，不并排第二块空面板。
   - pane 的内容按优先级决定：批量 > 编辑器 > 来源选择 > 预设 > 选中详情 > 空态。宽窗未选中时，pane 是虚线空态，汇总 catalog 读到的条数。不自动选中第一项；筛选让选中项不可见时，详情随之消失。
   - 窄窗 drill-in：pane 有内容时只渲染 pane，否则只渲染列表。进入 pane 后，焦点在下一帧移到 pane 的第一个控件。详情顶部有“返回列表”，返回后焦点回到原来的行。预设与来源选择用标题旁的“取消”，编辑器与批量用各自的关闭按钮。离页或重新读取后，延迟的焦点回调不再执行。
5. **列表行**：56px 两行（`MCP_ROW_HEIGHT`），左侧 16px 传输图标：square-terminal 表示 stdio，globe 表示只有 URL。
   - 第一行：14/500 名称，单行省略并带 tooltip；右侧是平台身份圆点加平台名。
   - 第二行：12px muted。先是命令（mono，省略并带 tooltip），只有 URL 时写“URL set (hidden)”；再接作用域“User”或“Project · 末段目录名”，完整项目键放在 tooltip。参数不显示。
   - 标签只在异常时出现：Disabled 表示 `disabled = true` 或 `enabled = false`，warning 语调，带 ban 图标；Read-only 表示这里不能写入的旧来源，neutral 语调，带 eye 图标。
   - 行的 aria label 是“平台 · 名称 [· disabled in the file] · 作用域 · 传输 [· read-only here]”。
6. **详情**：
   - 头部：40px 方块（primary 10% 底）里放传输图标；眉题是“平台 · 来源文件 · 作用域”（12 muted）；名称 20/600。
   - 操作行：“复制到…”、“编辑”、“删除”，删除用 danger 文字，可换行。只读来源不给操作，只显示 eye 图标和一句说明。
   - 键值网格：标签在上，值在下；两列，窄时单列。字段依次是 Transport、Source、Command（mono，占整行，因为常是长路径；放在半宽单元格里会折成多行，把后续字段挤到折叠线以下）、URL（Set (hidden) / Not set）、Arguments（Set (hidden) / Not set）、Scope（项目键用 mono 并占整行）、State。State 写文件里 disabled / enabled 的原文，只有原文用 mono；未设置时用正文字体写“文件中未设置”。
7. **pane 内的其他内容**：
   - 预设行 56px：名称加一行描述和标签，单行省略，tooltip 补全；需要密钥时，用 warning 标签写出环境变量名。
   - 来源选择：“目标来源”和“项目目录”都有可见字段标签；项目目录输入框的 placeholder 是“Absolute path (optional) / 绝对路径（可选）”。
   - 批量：两种模式是 `semantics::tab` 胶囊。预设、来源、目标和“替换已有条目”都是 `badge::filter_chip`，未选中时也有 hairline 边，不能看起来像正文；选项行可换行。顶部说明是 12px muted 辅助句；替换开关的后果写在下方一句 muted 辅助文字里。“返回 MCP 列表”放在批量标题行右侧，较矮的分栏 pane 里也不会落到折叠线以下；确认区挂载时不显示它，返回由确认区的“继续编辑”承担。
   - 决策控件放在可变高度内容之前。编辑器的确认与保存，放在补丁文本区和表单之前；批量的“检查批量更改”、确认区与结果状态，放在设置选项和逐行列表之前。这样在 1280×800 的分栏 pane 里，主按钮和确认区的初始焦点都在视口内。
8. **诚实与掩码**：
   - 本页不启动服务器，也不推断进程状态，不出现 Running / Connected 一类徽章。计数只来自 catalog，未读取与不列出的来源照实写。
   - URL 值、args 值、env/header 的键名和值，不出现在列表、详情、aria label、tooltip 或快照里。
   - 编辑器补丁初值是 `{}`。URL、env/header 值输入框和批量凭据输入框保持 masked，批量凭据保存后清空。跨源复制得到的草稿不带凭据。

Agent 会话页自上而下：

1. **页头一行**：左侧是 40px 方块（primary 10% 底）里的 book-open 图标，旁边是唯一 h1“Agent 会话 / Agent sessions”和一句 12px muted 描述。右侧依次是索引状态标签、“CCR 废纸篓”（ghost，inbox）和“刷新索引”（primary，refresh-cw）。两颗按钮都是默认尺寸、图标加文字，与 MCP 管理页页头一致；空间不足时整组换行。
   - 索引状态标签是 `badge::tag`，带 `role=Status`。Running 用 info 语调，写“Indexing…”；Finished 用 success 语调，写“Indexed · parsed X / upserted Y”；Failed 和 Interrupted 用 danger 语调，并写出原因。Idle 不显示标签。
2. **提供方 chip**：每个会话家族一个 `badge::filter_chip`，可多选，可换行。没有选中任何 chip 时表示全部提供方。chip 内容是“指示符 + 名称 · 读数”，完整句子同时作为 aria label：
   - Available：平台身份圆点，读数是来源数（tnum），比如“Codex · 3 sources”。
   - NoData：muted 实心圆点，写“no files”。可选。
   - NotInstalled：空心圆点加虚线边，写“not installed”。disabled。
   - Error：circle-alert，danger 语调，写“unreadable”；tooltip 是提供方报告的 `error_category`，不含路径或正文。disabled。
   - 平台作用域（Claude / Codex 工作台的会话 tab）锁定为该家族，不渲染 chip，改为一行 muted 文字“Showing Claude sessions / 只显示 Claude 会话”。
3. **工具栏**：左侧是 `badge::search_field`，placeholder 写“Search session titles / 搜索会话标题”；其后是两组互斥的 `semantics::tab` 胶囊：记录完整度“All | Full record | Partial | Locked”（全部 | 完整记录 | 部分 | 锁定），来源状态“All | Live | Missing | Deleted”（全部 | 现存 | 缺失 | 已删除）。右侧是 tnum 的本页计数，写“100 on this page · older available”；列表请求在途时改写“Reading the archive…”。
   - 搜索按 Enter 立即查询；输入后防抖 300ms 再查询，每次输入让旧的防抖 token 过期，过期 token 到期后不执行。
   - 筛选变化（chip、胶囊、搜索）回到第一页，并关闭已打开的会话。
4. **操作反馈**：导出、克隆、移入或恢复的运行、失败与结果，在工具栏下方以一组 12px 文字陈述，带 `role=Status`。失败用 danger 墨色。空闲且没有结果时不渲染。
5. **主体**：窗口宽度 ≥ `SPLIT_WIDTH`（1200）时，列表与 pane 按 3:2 并排，间距 16，列表 ≥ `LIST_MIN_WIDTH`（360），pane ≥ `DETAIL_MIN_WIDTH`（320）。外层列只负责尺寸，不带内边距和描边，实色阅读面板放在列内并填满它，原因同 MCP 管理页。1280×800 下列表约 576px、pane 约 384px。
   - pane 的内容按优先级决定：废纸篓 > 打开的会话 > 空态。宽窗未打开会话时，pane 是虚线空态，只汇总本页列出的行数。不自动打开第一行。
   - 没有可列出的会话、也没有打开的内容时，宽窗只放列表列，不并排一块空 pane。
   - 窄窗：打开会话时只保留页头和 pane（drill-in），不渲染提供方 chip 和工具栏，pane 顶部是“返回列表”；返回后 chip 和工具栏恢复。焦点在下一帧移到 pane 的第一个控件，返回后焦点回到列表，选中行不变。离页、刷新或又打开别的会话后，延迟的焦点回调不再执行。
   - 窄窗打开废纸篓时，废纸篓按内容高度排在列表上方，列表保留 320px，整页纵向滚动。搜索框在工具栏里，废纸篓打开时宽窗与窄窗都可见、可点。
6. **列表**：`uniform_list` 放在 12px 实色面板内，行高 60px（`TWO_LINE_ROW_HEIGHT`），行底 hairline。列表容器是 `role=ListBox`，获得焦点后上下键移动选中，Enter 打开；点击直接打开。行是 `role=ListBoxOption`，带 `aria_selected`、位置与总数；行 id 仍是 `insights-row:{archive_id}`。
   - 第一行：平台身份圆点；14/500 标题，单行省略并带 tooltip（无标题时用 session id）；右侧 12px tnum 更新时间 `MM-DD HH:mm`。
   - 第二行：12px muted。“平台 · 变体 · N messages · N tools”（无工具调用时省略工具数）不收缩、不省略；其后的“· 项目路径”是唯一可伸缩的部分（mono，单行省略，带 tooltip），摘要放不下时路径整段让出。
   - 标签只在异常时出现：Partial / Locked 用 warning 语调，未知完整度用 neutral 语调；来源 Missing 用 warning，Deleted 用 neutral。完整记录与现存来源不加标签。
   - 底部翻页条：有 `next_cursor` 时显示 `insights-next`“Older sessions / 更早的会话”；不在第一页时显示“Newest sessions / 最新的会话”。两者都不适用时不渲染翻页条。翻页保留已打开的会话。
   - 没有行时用虚线空态陈述原因：读取失败写原因；未索引给出“刷新索引”；筛选无匹配写“这不代表提供方没有会话”，并在有筛选时给出“清除搜索与筛选”。不铺 0。
7. **会话 pane**：
   - 头部：16/600 标题，单行省略并带 tooltip；下面是 mono session id 和“平台 · 变体” neutral 标签，再下一行只写“N messages · N tools”，非完整记录时追加完整度，不重复平台与变体。Codex 活跃会话的“导出 Markdown / 克隆会话 / 移入 CCR 废纸篓”放在头部右侧，可换行；其他提供方不给操作。
   - 完整度为 partial 或 locked 时，在头部下方显示 warning 语调的提示条（triangle-alert），说明只读到一部分或记录不可读。
   - 决策控件放在可变高度内容之前：克隆与移入的确认区、导出预览（保存与复制）、本页消息数与“Earlier messages / 更早的消息”，都放在消息列表之前。
   - 消息卡：头部依次是角色图标（user、bot；工具等其他角色用 square-terminal）、角色名、mono 工具名，右侧是时间，没有时间时写 `#序号`；归档截断时用 warning 墨色写“Clipped in archive / 归档中已截断”。
8. **CCR 废纸篓**：由页头“CCR 废纸篓”打开或重新读取，宽窗占据 pane。面板标题“CCR trash (Codex)”，右侧是 `session-trash-close`“关闭”；关闭后 pane 回到打开的会话或空态。
   - 恢复确认放在翻页与行之前；确认区挂载后，初始焦点进入“取消”，Tab 留在决定子树，Esc 返回并把焦点还给触发者（例如搜索框）。确认区挂载时，下面的行与翻页都 disabled。
   - 每页 6 行，行之间用 hairline。空废纸篓用虚线空态。
9. **诚实与掩码**：
   - 只陈述索引给出的数据：不编造会话数，不显示 Running 一类推断出的进程状态；完整度只用索引里的 full / partial / locked。
   - 会话正文只在 pane 内渲染，不进入 aria label、tooltip 或日志。离页时清除正文、导出预览、在途列表与详情请求和搜索防抖。
   - 列表与详情是两条请求，`RequestKey.environment` 分别为 `insights-explicit` 与 `insights-detail`，各自带 gate 与 generation：快速连续打开两行时只显示后一行；列表刷新或筛选变化会丢弃在途的详情请求。Workbench 渲染器与用量页仍走单 snapshot 路径。

## Elevation & Depth

材质是**已知静态场景上的半透明分层材质**。纸面背后的像素全部由本应用绘制，且只有低频内容：160deg 两色线性渐变场，以及两个静态柔光斑（GPUI 无 radial gradient，用透明圆形加同色大模糊阴影绘制）。对这种平滑内容再做高斯模糊，结果与原图的差异低于 8-bit 量化误差，所以 alpha<1 的面直接合成到场景上，像素上等价于 backdrop blur；这不需要背景采样 API，也不是“假玻璃开关”。成立条件在代码评审时强制执行：

1. 半透明材质背后只能是场景（渐变与光斑），不能有文字、图标、图片、列表等高频内容。
2. 不叠玻璃：侧栏 chrome 与主纸面 sheet 是并列兄弟，不互相嵌套；页内分段凹槽只是同一纸面上的浅色 tint，不是第二层材质。
3. 承载正文的阅读面（表格、列表、输入、弹层、代码/导出区）一律实色。

三档材质：scene（渐变 + 左上薄荷光斑 + 右下淡紫光斑，完全静态，无动画）；chrome（侧栏导轨，light 白 58% / dark #1d292c 55%，透出场景色）；sheet（主纸面，light 白 76% / dark #172225 86%）。两者都有 1px highlight 亮边（light 白 80% / dark 白 8%）与 inset 0 1px 顶部高光；整体框架外层一组漫射阴影 0/20/48 + 近影 0/2/6（light 5%+3%，dark 30%+18%）。GPUI 外阴影会延伸到半透明面下方，所以阴影透出的变暗也计入合成对比度的最坏背景。

侧栏左侧 20px 外角、纸面右侧 20px 外角，并列组成一个 20px 圆角框架；侧栏右边的亮边就是两者的分隔线。GPUI 的 overflow mask 为矩形，不能用父纸面圆角假定子背景会被圆角裁剪：直接接触纸面右上角的配置阅读面使用内侧 19px 背景角；28px 状态区不绘制背景，保持父纸面及其 20px 底角可见。其余密集控件角度、布局和滚动所有者不变。转换结果用紫色可见标签标明输出，代码正文保留阅读文字色，错误/成功状态仍使用独立语义色。

窗口 Blurred/Mica/Acrylic 是操作系统透出桌面的能力，本合同不使用。减少透明度/高对比偏好 API 未暴露，不声称系统设置联动已验证；elevated 保留为纸面实色参照色。

外场景边距 = clamp(min(可用宽余量, 可用高余量)/8, 0, 20) logical px，余量相对最小窗口 960×640 计算：默认 1280×800 为 20px，最小窗口归零，保留密集操作空间。原 1040/1200 导航/详情断点不变。

## Shapes

主/次按钮均 pill，输入 8px，列表可选背景 4px，内容面板 12px，框架 20px（仅外层，侧栏左角 + 纸面右角）。分段胶囊容器 20px 角（单行即 pill，换行时与内侧 pill 同心），内部选项均为 pill。形状区分可点与不可点：非交互的状态标签是 4px 角矩形，交互的筛选 chip 是 pill；虚线空态面板 8px。图标使用一个集合，优先 Kit 的 Lucide 资源；品牌独立复用仓库生成的蓝色 CCR mark，不重绘或染色。实际依赖/资源开销在构建时记录。主操作不使用 emoji 图标。

保留克制与柔和控件；数据列表不用 pill 包每一项。只有语义独立的弹层使用 8px 外角。

## Components

### Platform workbench and navigation

The sidebar keeps all 13 destinations. Each 32px row has one 16px Lucide icon,
an 8px gap and a left-aligned label; all icons share one alignment column.
Group headings use the 12px muted role and a hairline; a group sits 12px
below the previous row and its first row 4px below the heading. The brand row
has 12px of breathing room above the first destination. The rail has 12px
padding and 4px row gaps. Settings stays pinned above a 12px version line,
separated from the routes by a hairline. Selection is a translucent reading
pill (70%) with an inset top highlight, primary text and icon and semibold
weight, without an opaque border. Hover uses a lighter 35% reading pill and
keyboard focus keeps an independent 2px ring, so the three states differ.
Platform icons are neutral by default and show their identity color only on
hover or selection; non-platform icons turn primary when selected.

Context tabs (lenses) render as a content-width segmented capsule, without a
full-width bar, bottom border or fixed 48px height. There is exactly one h1
per page, mounted in one of two ways:

- **Embedded (Dashboard: Overview and Monitoring; MCP Manager: MCP Manager
  and Configuration converter).** The shell owns the lens
  state and a small `page_header::LensBar` view; it hands that view to the
  page once, and the page renders it inside its own heading row, so the title
  and the capsule share one line. The bar reads the shell's current
  destination/lens at render time and calls the shell's `go` on click, so
  keyboard order, `toggled` state, focus ring and test ids (`tool-overview`,
  `tool-monitoring`, `tool-mcp`, `tool-converter`) are unchanged. The shell
  renders no row above the page.
  Switching lens moves the capsule to another page, and Kit keeps button focus
  per element path, so the bar owns a stable focus anchor: after a keyboard
  activation it returns focus to the activated option on the next frame, only
  while the shell still shows that destination and lens. The Claude workbench
  reuses the MCP view under `tool-platform-mcp`; there the view renders no
  capsule and the shell keeps the context row with the platform tabs.
- **Context row (all other lens destinations).** Those pages host several lens
  entities or up to ten platform tabs, so the shell still renders the capsule
  above the page, carrying only the platform name as a muted context label for
  platform destinations; the page's own heading follows directly below. A
  destination moves to the embedded form by giving every page it hosts an
  `embed_lens` slot in its heading row and adding it to `Shell::embeds_lens`.

The brand row embeds the generated app PNG with its original colors and alpha;
the Windows executable embeds the matching existing ICO as resource ID 1.
The status bar keeps the actual environment/source label, a matching state dot,
four numbered route shortcuts and a local HH:mm clock refreshed every 30 seconds.
The source dot means configured, never connected or successfully read.

Platform destinations open a workbench, with Profiles as a separate context
tab. The first reading line names the captured local profile, credential
observation and source; credential detection must never claim a remote login.
Claude offers direct context tabs for authentication, MCP, Agents, Plugins,
Commands, instruction files, settings and usage/subscription. Other platforms
expose only their actual capabilities. Context tabs retain the platform name
while the existing feature entity performs the operation. Summaries use the
same captured services and read-only usage projections as full feature pages.
Usage periods, local timezone and cost basis are explicit. Missing databases,
unreadable sources and unknown counts show a reason, never sample zeroes.

Codex has a separate local rolling-usage reading for five hours, seven days
and all history, with an explicit UTC observation time. It is not account
quota or the llmusage 30-day calendar report. An explicit read action explains
that the domain scanner refreshes its derived cache. Missing sessions remain
unknown; partial scan diagnostics and usage-record counts remain visible.

Common Claude collection forms keep their save/review controls above the
variable-height fields. Confirm/cancel stay visible inside the editor rather
than falling beneath a tall form's clipped viewport.

The Claude Code home starts with profile and local credential status, then
shows today's, this month's and all-time recorded costs. A calendar-spaced
line shows daily USD cost for the last 30 days, with gaps for missing dates.
The report action opens Claude's daily rows for keyboard access to exact
amounts. Subscription settings sit beside the report action. The platform
tabs give access to the other tools. Other platform homes retain the ruled
status section, compact usage summary, daily bars and management links.

- **按钮**：normal 用 primary/secondary 或透明次按钮；hover 提升一个中性色阶；focus 显示 2px 环；pointer pressed 可瞬时 scale(0.96) 并在 100ms 内复原，hitbox 不变；keyboard activation 仅即时状态反馈，无动画。disabled 不触发，loading 保持宽度且有文字状态，error 配相邻说明。
- **输入**：14px 文本、8px 角、border-control；focus/invalid 分别使用 focus/danger 与文字。支持组合输入、选区、复制粘贴、撤销与系统编辑键。密码遮蔽默认开启，明文不得进入截图或日志。
- **导航**：当前选中以半透明 reading pill、内顶高光、主色与字重表达，没有粗侧边条或不透明边框。Tab 与方向键行为按原生语义区分，不与文本编辑键冲突；支持 F6 在侧栏/主区/状态区切换。
- **表格/列表**：列角色固定，数字使用 tabular figures 右对齐。行 hover 与 selection 用中性色阶，频繁点击行不缩放，避免文字抖动，这是高频数据操作的明确运动例外。过长名称可 tooltip 补全，关键数值/操作名必须完整或按语义缩短，不能靠省略号掩盖。
- **任务反馈**：queued/running/cancel_requested/terminal 状态可见。页内进度文本与可恢复错误优先，toast 不作为唯一结果渠道。取消请求不得立即写“已取消”。
- **确认**：只用于不可逆/敏感操作的业务确认，列明对象和后果；普通导航与展开不用弹窗。确认区域挂载后将初始焦点转入真实决策按钮，即使此前焦点在侧栏；取消或离页后的延迟回调不得重新抢焦点。Tab/Shift+Tab 留在决定子树，Esc 返回安全状态，焦点回到触发者。测试须覆盖从外部焦点真实打开的路径，不能手动把焦点塞进确认框后声称初始转移通过。
- **图表**：有明确问题才显示；趋势用线、比较用条形。高度 160 至 240px（Overview 逐日图 216px），图例与 exact-value table 可达；低对比色必须有表格/文字通道。数据更新不播放入场动画。Overview 逐日图自动收紧到首个有数据日的前一天，标题旁标“since MM-DD / 自 MM-DD 起”，查询窗口仍写在页头状态行；整个窗口都没有记录时保留完整窗口的虚线基线。窗口末尾有无记录的日子时，标题旁再追加“· latest record MM-DD / · 最新记录 MM-DD”，只陈述最后一个有记录的日期，不推断末尾空白是没有用量还是尚未导入。堆叠段仅最上段 2px 圆角，段间 1px 阅读面间隙；系列色是平台身份色相、饱和度 ×0.72（`theme::chart_series`），图例使用同一函数，对阅读面图形对比度 ≥3:1。
- **精确值行焦点**：Overview 精确值表默认收起；展开后最新在上，每段连续全零日（含单日）折叠为一行“MM-DD – MM-DD · no records (N days)”/“MM-DD – MM-DD · N 日无记录”，折叠行与日期行同样可聚焦。焦点句柄与显示行按同一顺序一一对应，第 i 个显示行对应表格第 i+1 个子项（表头为第 0 项）。真实行焦点始终显示预留的 2px ring 边框，跟随逻辑 focus，不由最后一次输入模态隐藏。Tab/Shift+Tab 同时 reveal 表格与外层页面中的当前行；UIA focused 或 headless bounds 通过不能替代原生可见边框验收。
- **状态**：empty 说明原因并给已有下一步；loading 保留布局；error 显示可重试原因；unsupported 与未接入明确区分。演示 fixture 必须带“演示数据”标识，生产无伪造数字。
- **状态标签（`badge::tag`）**：不可交互的 4px 角矩形，22px 高，12/500，1px edge，单行不换行；颜色只来自 `theme::tone_colors` 的五种语调，不用 Kit 的 Tag/Badge。文字本身传达状态，颜色不单独表意。只在异常或需要强调的状态上出现，不给每个列表项都套标签。
- **筛选 chip（`badge::filter_chip`）**：可交互的 28px pill，12/400，toggled 语义表达选中，并沿用 `semantics::button` 的 disabled 与 tab stop。未选：透明底加 hairline 边、辅助文字；选中：tab-selected 浮起面、轻阴影、primary semibold（600）；禁用：透明底、muted 文字、hairline 边，选中样式不得覆盖禁用色。不可选的来源由页面再加虚线边。点击后立即切换，无动画。
- **身份圆点（`badge::dot` / `hollow_dot` / `identity`）**：8px 实心圆，圆点旁必须有名称或状态文字。`hollow_dot` 是 1px 描边空心圆，表示 missing / not-read。`identity` 是身份圆点加平台名，身份色范围见 Colors。
- **搜索框（`badge::search_field`）**：前缀 16px muted 的 search 图标，必须有 placeholder，固定宽 300px（合同范围 280–320px），8px 角；窄容器内不超出父宽。id 与 aria 由调用方给定。
- **工具栏**：一行，筛选在左、操作在右，空间不足时换行，不出现全宽按钮。
- **两行列表行**：定高 56–60px（MCP 56、Agent sessions 60），放在 12px 实色阅读面板内，行间 hairline 分隔，hover/selection 用中性色阶与 4px 角，不缩放。第一行是名称，第二行是 meta，数字 tnum，超长省略并有 tooltip；异常才显示状态标签。
- **来源健康条**：每个来源一个 chip：状态圆点、短名、读数。未读取用虚线边，Invalid/Unreadable 用 warning/danger 语调；完整原句作为 aria label 与 tooltip。窄宽在条内横向滚动，不换行撑高。
- **虚线空态面板（`badge::empty_panel`）**：1px dashed border-subtle，8px 角，居中排列 24px muted 图标、14/500 原因、12px muted 说明和可选下一步。下一步由调用方传入真实可聚焦控件，面板不伪造动作。
- **分区标题（`badge::section_heading`）**：16/600 标题，同基线 12px muted 口径文字（如“All history”“Last 53 weeks”），让读数范围一眼可知。

## Do's and Don'ts

### Do

- 用真实配置、真实任务和就绪原因建立首页信息层级。
- 将 DB/扫描/网络移到后台，只更新相关 entity。
- 用稳定 ID 虚拟化长列表，让滚动与后台导入并行。
- 为每个操作实现键盘焦点、busy/disabled/error 与返回路径。
- 在 light/dark、中文/英文、最小窗口及 DPI 变化下复核。

### Don't

- 不使用默认卡片墙、渐变文字、紫蓝霓虹或动态装饰背景；场景只用已批准的静态渐变与两个静态光斑。
- 不叠玻璃，不把半透明材质盖在文字、图标、图片、列表等高频内容上；阅读面不做半透明。
- 不开启窗口级 Mica/Acrylic/Blurred，不加 Glass 开关。
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
2. “实现配置列表：reading #FFFFFF、正文 #1E2932 14px/400、行高36px、选中 #E3ECE8/4px、数值等宽右对齐、主区padding24px、标题20px/600/-0.012em；采用虚拟化并提供键盘选择。”
3. “实现深色任务区：背景 #1D292C、正文 #F0F8F5 14px/400、辅助 #9FAFA9 12px、控件边界 #737B86、输入角8px；取消中与取消完成分开显示，不播放列表入场动画。”
4. “实现保存按钮：绿色底配独立对比文字14px/500、pill角、逻辑高度36px、水平padding16px、focus #156F4E 的2px环与2px间隔；pointer按压0.96，键盘无动画，loading保持宽度。”
5. Significant change 后从真实窗口重新看第一屏：环境是否清楚、状态是否真实、主操作是否明确、是否新增无意义容器。

## Known Gaps and Native QA

03 已将语义 token 实现在 `src/theme.rs` 并通过 Kit Theme 同步 Base/Component；`src/shell/` 呈现已接入的 Local 配置、命令收藏/历史与会话内主题/语言设置。列表采用稳定领域 ID 的虚拟化，详情提供返回，F6/Tab 与编辑键分别处理。无持续装饰动画；按钮即时 pressed 色反馈，常用行操作不做缩放。

01/02 已有 Windows 实际原生输入、读取筛选、错误恢复与正常退出证据；03 新外壳的原生核心验证另记任务 evidence。2026-10-18 用户明确不做窗口尺寸、缩放或 DPI 矩阵，该部分 user-deferred，不能记通过。保留设计最小窗口与自适应实现，并不以未运行矩阵阻止已批准后续工程。

待分别验证：03 原生 light/dark、中英文、中文 IME、字体 fallback、焦点恢复、屏幕阅读器 role/name/state，以及 macOS/Linux 实际原生运行。按钮同节点 disabled/busy 标记已实现；现有 Kit snapshot/debug 接口不能完整观测这些 flags，不能以 None 或 headless 标签证明系统读屏已通过。03 的普通输入保留编辑能力；04 的 Inspect/导出临时区使用 readonly，并有实际控件输入抑制测试。确认决策按钮使用 Kit 原有 focus trap；这些 headless 结果不替代系统读屏验证。未检测到系统 reduced-motion API，应用不启用装饰性运动，键盘即时响应。

当前不存在需要“换成真实内容”的业务占位数据；实施 fixture 只用于测试，生产数据应通过 ccr-app/领域 API 注入，不能在 view 中硬编码用户数、余额或认证成功。

04 配置集与配置目录复用连续列表/详情工作面，编辑区在内容区展开，字段超长时独立滚动。破坏性操作显示对象、后果和确认/取消；源码、导出和转换使用明文提示与临时文本区，离开页面清除缓冲。Grok 的部分完成状态和 CAS 冲突持续显示在页内，不使用成功 toast 掩盖未完成阶段。草稿关闭后清除仅适用于该草稿的冲突恢复文案；已提交警告或部分完成结果保留，不随普通只读状态刷新丢失。04 的真实原生核心流程由父会话另行采集，本条实施说明不等于原生验收或性能通过。

## 2026-10-27 视觉增量能力与证据边界

唯一 theme.rs 映射 light/dark 的 field、sheet/base/reading/overlay、主次辅助文字、action、output 与状态角色。材质能力研究见 `.trellis/tasks/archive/2026-11/10-17-gpui-03-shell/research/airy-glass-capabilities.md`；2026-11-07 起由 `.trellis/tasks/11-07-gpui-airy-glass-material/design.md` 的“已知静态场景”论证取代实色降级，`theme::Material` 是材质唯一来源，合成对比度单测覆盖 light/dark 全部承载文字的半透明面。单测与 headless 绘制断言不等于原生像素验收。历史原生证据仍对应原构建，新增配色、焦点、弹层和核心业务需新构建实测；S1/S2、UIA/IME、macOS/Linux 与用户延期矩阵继续分别记录。
