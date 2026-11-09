# Design: 首页 Insights 界面设计定稿

本文件记录设计子任务的输入、约束与待决项。视觉结论在执行后写入 `ccr-ui/DESIGN.md`，不在本文件重复。

## 1. 输入

- 参考：Wake Insights 截图（父任务对话附件）与 `ref/repo/Wake/DESIGN.md`（只读，GPUI 实现，只取信息结构，不取视觉风格）。
- 现有视觉源：`ccr-ui/DESIGN.md`（行情终端）、`ccr-ui/src/styles/tokens.css`、`ccr-ui/.impeccable/design.json`。
- 数据契约：backend 子任务 `design.md` §5 的 `HomeInsightsResponse` 字段。设计只使用该 DTO 提供的字段。

## 2. Wake 信息结构 → ccr 映射

| Wake 区块 | ccr 区块 | 度量 | 备注 |
| --- | --- | --- | --- |
| 顶部 6 项总数 | 总量行 | 会话、token、请求、agent、项目、活跃日 | Wake 的 Prompts 换成请求数（D2） |
| Last 7 days | 最近 7 天对比 | 会话、请求、活跃日 + 相对前 7 天变化 | |
| Activity 热力图 | 53 周热力图 | 每日请求数 | 说明行：当前 streak、最长 streak、最忙日 |
| Over time | 按周分 agent 趋势 | 每周请求数，按来源堆叠 | 53 列，前 N 来源 + 其他 |
| By hour（左右切换） | 分布面板 | 小时 / 星期 / 月 | 三视图切换 |
| Agents/Projects/Models 榜单 | 榜单 | 请求 / token / 会话 | 会话只对 Agents 可用 |

## 3. 约束（来自 DESIGN.md，设计不得违反）

- 首屏签名构图不变（`ccr-ui/DESIGN.md:199-205`）。
- 图表有界（`ccr-ui/DESIGN.md:269`、`:300`）。
- 禁止指标卡片行 + 无界主图（`ccr-ui/DESIGN.md:306`）。
- Amber 只表示命令、激活、焦点，单屏占比低于 10%（Amber Scarcity Rule）。
- 平台色只用于 2px 标记、图例色块、图表分段、身份点（Platform Tick Rule）。
- 等宽字体只用于数据（数值、时间、坐标轴）。

## 4. 待决项（执行时决定，结论写入 DESIGN.md）

- Q1 热力图梯度色相：中性墨色梯度，或一个非 amber、非状态色的数据色相。由 dataviz 校验器在亮/暗主题下比较后选定。
- Q2 周趋势合并阈值 N：取值使 "其他" 在典型数据下占比低于 15%，并保证 N 个身份色两两可区分（dataviz 校验器的色差检查）。
- Q3 总量行与 7 天对比的形式：候选为行情带式单行、终端表格行。选定形式须与首屏平台行情带在视觉上可区分。
- Q4 分布面板视图切换控件：分段控件或左右箭头。须支持键盘操作。
- Q5 1280px 与更窄宽度下的栅格：热力图 53 列在窄宽度下的处理（横向滚动容器或缩小格子，最小格子边长不低于 8px）。

## 5. 首屏图表越界排查方法

1. `cd ccr-ui && bun run dev:web -- --host 127.0.0.1 --strictPort`，打开 `http://127.0.0.1:5173/`，窗口 1920×1080。
2. 在内置浏览器中读取 `.dashboard-usage__chart` 的计算样式 `height` 与所在网格行的高度；检查 `dashboard-usage-movement.css` 是否加载（`.trellis/spec/ccr-ui/frontend/usage-chart-stability-contracts.md` 记录过 CSS 双路径交付问题）。
3. Web 预览下 Tauri `invoke()` 不可用，图表可能进入空态。若空态无法复现，用 fixture 数据或 Storybook（`ccr-ui/.storybook/`）渲染 `DashboardUsageMovement` 再测。
4. 仍无法复现时，记录"仅在 Tauri 运行时出现，原因未查明"，由 frontend 子任务在 `just ui-dev` 下复现。

浏览器工具可用不等于已授权操作界面（`ccr-ui/AGENTS.md`）：本步骤只读取样式与截图，不点击会写配置的控件。

## 6. 定稿结论（2026-09-24，用户已批准画布）

画布：https://claude.ai/code/artifact/d0871e3f-04ba-4a2d-a82f-2f2788af83d1 。颜色数据见 `research/palette-validation.md`。现有 6 个平台色不改（用户未要求另开任务）。

### 6.1 待决项结论

- Q1 热力图梯度：暖中性单色相（OKLCH H≈82°，C 0.025），5 档。token `--color-chart-heat-0..4`，值见 palette-validation §6。heat-0 = 无活动的格子底色。
- Q2 合并阈值：N = 4。取 53 周窗口内总量前 4 的来源独立着色（并列按 `SourceKind::ALL` 顺序），其余并为一段"其他"。前 4 在整张图内固定。堆叠顺序固定（自下而上）：claude、zcode、grok、deepseek_harness、antigravity、kimi_code、opencode、codex、pi，"其他"置顶；图例同序。
- Q3 总量行：单行读数。上下各一条 1px `--color-border-subtle` 发丝线，6 组"标签 + 数值"同一基线，组间 1px 竖发丝线；标签 sans 0.8125rem muted，数值 mono 1.125rem semibold primary tabular-nums。无面板底色、无单元格、无 2px 标记、无迷你柱。7 天对比：面板内固定行高（2.25rem）对照表，列为 度量 / 7 天 / 前 7 天 / 变化；变化用符号 + 文字（`+21.7%`、`−17.6%`、`+2 天`、`持平`、前 7 天为 0 时写 `新增`），颜色保持 secondary ink，不用状态色。
- Q4 视图切换：分段控件（`role="radiogroup"`，方向键切换，可见焦点环）。分布面板：小时 / 星期 / 月。榜单：Agent / 项目 / 模型 + 度量 请求 / Token / 会话（会话只对 Agent 可用；会话未索引时禁用并带 `title` 说明）。
- Q5 热力图栅格：53 列 × 7 行，格宽随容器在 0.5rem–0.875rem 之间缩放，格间距 `--space-0-5`；格宽低于 0.5rem 时容器改横向滚动，默认滚动到最新一周。1024px 以下所有双栏改单列。

### 6.2 区块与高度规则

| 区块 | 布局 | 高度规则 | 颜色 |
| --- | --- | --- | --- |
| 区块标题行 | 标题"使用洞察" + 说明 + 右侧 mono 日期范围 | 自然高度 | text tokens |
| 总量行 | 全宽 | 单行 | text tokens + border-subtle |
| 活动热力图 | 与 7 天对比同行，列比 1.85fr : 1fr（同首屏 `.dashboard-lower`） | 7 行格子 + 月份行 + 说明行 | `--color-chart-heat-*` |
| 7 天对比 | 同上右栏 | 表头 1.5rem + 3 行 × 2.25rem | text tokens |
| 每周请求 | 全宽面板 | 图表 `clamp(10rem, 26vh, 16rem)`（同首屏） | 来源色 + `--color-chart-other`（叠加 45° 斜线纹理：`repeating-linear-gradient(45deg, other 0 0.1875rem, surface 0.1875rem 0.3125rem)`） |
| 时段分布 | 与榜单同行，两列等宽 | 图表固定 10rem | 柱 `--color-chart-heat-2`，峰值柱 `--color-chart-heat-4` |
| 榜单 | 同上右栏 | 行高固定 2rem，最多 8 行 | Agent 行：2px 平台色标记 + 0.375rem 高平台色条（底轨 `--color-bg-overlay`）；项目/模型行：中性条 `--color-chart-heat-2` |

通用：面板沿用首屏 `.dashboard-usage` 面板规则（surface 底、1px subtle 边、`--home-card-radius`、`--home-card-pad`）；区块间距 `--home-section-gap`；图表网格线 25/50/75%；柱体无圆角；周趋势段间 `--space-0-5` 表面色间隙；每个柱/格有 `title` 与悬停提示；序列数 ≤4 时图例外加直接标注；大数缩写沿用首屏 `compactLabel`（跟随运行时默认 locale，即 WebView 系统语言，不跟随应用界面语言：中文系统显示 万/亿，英文系统显示 K/M/B），完整值放 `title`；周趋势与热力图各有「图表 / 表格」分段切换，表格视图给出每周（热力图为每日）完整数值，作为低于 3:1 颜色的替代通道（2026-09-24 用户决定）；不引入图表依赖。

### 6.3 空态

| 状态 | 判定 | 呈现 |
| --- | --- | --- |
| 空库 | 无请求且无会话 | 标题行 + 总量行全部 `—` + 一个虚线面板（`1px dashed --color-border-default`）："还没有可分析的记录" + 说明 + 主按钮"导入用量"（amber，命令语义） |
| 仅会话 | 无请求，`sessions_indexed = true` 且有会话 | 请求类数值 `—`；热力图全部 heat-0 + 说明"导入用量后显示" + "导入用量"按钮；7 天对比只有会话行有值；榜单默认度量"会话"；周趋势与分布不渲染 |
| 仅用量 | `sessions_indexed = false` | 所有会话数显示 `—`（带 `title="会话尚未索引"`），总量行下方一行 info 点 + 说明；榜单"会话"度量禁用；其余区块照常 |
| Web 预览 | 非 Tauri 运行时 | 标题行 + 实线面板：info 点 + "Web 预览不读取本机数据" + 说明；不发起调用 |

### 6.4 新增 token 名（layer 1，`:root` 与 `[data-theme='dark']`）

- 4 个来源 × 5 个角色，与现有 6 平台同结构：`--color-platform-{kimi-code,pi,zcode,deepseek-harness}`、`-rgb`、`-surface`、`-border`、`-text`（20 个）。主色亮暗同值、neutral/clay 同值：`#b65790`、`#9773d0`、`#1d83a4`、`#1e8dfe`（deepseek_harness 由原 `#635dab` 调整：原值对暗色 clay 卡片面 `#2a221e` 仅 2.74:1，见 palette-validation §8；2026-09-24 用户确认采用 `#1e8dfe`）。主色对 4 种卡片面（亮/暗 × neutral/clay）均 ≥3:1；`-text` 对 `-surface` 与两种 flavor 的卡片面均 ≥4.5:1。
- `--color-chart-other`（亮 `#968b76` / 暗 `#a1937c`）。
- `--color-chart-heat-0` … `--color-chart-heat-4`（5 个）。暗色 heat-0 为 `#332f27`（原 `#29251d` 对暗色 clay 卡片面仅 1.02:1），一个取值同时用于 neutral 与 clay，clay 块不覆盖。
- 合计 26 个新名。来源键 `kimi_code` / `deepseek_harness` 在 token 名中写作 kebab（`kimi-code` / `deepseek-harness`），frontend 用来源键 → token 名的映射表。
