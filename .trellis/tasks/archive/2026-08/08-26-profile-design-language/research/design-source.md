# 设计稿来源与规格提取

## 来源

- Claude Design 项目：`0a3d3dfa-8ad5-4bdf-861d-305f1e2c6389`（名称：CCR-UI 首页重新设计）
- 文件：`CCR UI Profile 成品稿.dc.html`
- 取用方式：`DesignSync` 工具，`method: get_file`。WebFetch 与普通浏览器访问返回 403，需登录态。
- 同项目内另有 `CCR UI 首页重设计.dc.html`，对应已归档任务 `08-25-react-home-style-redesign`。

设计稿是 `.dc.html` 原型：内联样式 + `x-dc` 模板语法（`sc-for` / `sc-if` / `{{ }}`）+ 底部 `DCLogic` 类持有 state 与 `renderVals()`。原型硬编码暗色 hex，不含明色主题。

## 页面结构（自上而下）

1. **面包屑栏**（高 60px，下边框）：`{平台名} / Profiles`；右侧环境徽标（绿点 + 「本机」）+ 配置文件名徽标（mono）。
2. **页头**：平台字形方块（38px 圆角 9px，平台色边框/底/前景，内容为单字母 glyph）+ 标题（21px/600/-0.02em）+ 配置路径（mono 12px）；右侧「导入」次按钮 + 「新建 Profile」主按钮（accent 底，深色字）。
3. **统计条**：`grid-template-columns: repeat(4, 1fr)`，间距 10px。
   - 卡 1「总数」：mono 30px/600 大数字 + 副行 `N 家供应商`（按 base URL host 去重计数）。
   - 卡 2「运行中」：accent 边框与底色高亮；值为当前 profile 名（mono 16px，超出省略）+ 说明行。未应用时显示「未应用」。
   - 卡 3「标签分布」：tag chip 列表，每个 chip 为 `#tag` + 计数。
   - 卡 4「认证方式」：auth chip 列表，每个 chip 为 auth 值 + 计数。
4. **筛选栏**（高 36px 一行）：
   - 搜索框（占据剩余宽度）：放大镜图标 + 输入框 + `⌘K` 提示。占位文案「搜索名称 / 描述 / Base URL / 标签」。匹配字段为 `id + desc + url + tags`，小写包含匹配。
   - 标签 pill 组：`全部 N` + 每个 `#tag`，单选。
   - 视图切换段控件：`表格` / `卡片`。
5. **列表区**（滚动容器）：卡片视图、表格视图、空态三选一。
6. **表单弹窗**（条件渲染）。
7. **Toast**（条件渲染，底部居中）。

## 卡片视图

`grid-template-columns: repeat(3, 1fr)`，间距 12px，`grid-auto-rows: min-content`。

单卡结构：

- 顶行：状态点（7px 圆点，运行中为绿 `#7cab82`，否则 `#5d4d43`）+ 名称（mono 13px/600，省略号）+ 描述（12px muted，省略号）+ 右上徽章（运行中为 accent 实底「运行中」，否则中性底显示平台名）。
- 字段网格：`grid-template-columns: 1fr 1fr`，间距 `9px 12px`。四项固定为 `BASE URL / 模型 / 认证 / {平台 extraLabel}`；label 为 mono 10px + `letter-spacing 0.1em`，value 为 mono 12px 省略号。
- 底栏（上边框分隔，padding-top 11px）：tags chip 列表占据左侧余量 + 「编辑」次按钮 + 「应用」/「停用」按钮（运行中态为 accent 软底）。

运行中卡整体使用 accent 边框 `#6b4028` 与暖底 `#241a14`。

## 表格视图

`grid-template-columns: 216px minmax(200px,1fr) 176px 104px 136px 132px`，列间距 14px，容器 `min-width: 1024px`（窄屏横向滚动）。

表头（mono 10px/600，`letter-spacing 0.14em`，muted）：`名称 / BASE URL / {col3} / {col4} / 标签 / 操作`（操作列右对齐）。

行结构：状态点 + 名称/描述两行块 → BASE URL（mono 省略号）→ col3 值（mono）→ col4 值（chip）→ tags chips → 操作（编辑 + 应用/停用，右对齐）。行本身与卡片共用同一套 `border` / `bg` / `dot` / `nameColor` 计算结果。

## 空态

虚线边框框（420px 宽，居中，`margin: 40px auto`）：`?` 字形方块 + 「没有匹配的 Profile」+ 提示行（`"{查询词}" + #tag 没有结果`）+ 「清除筛选」与「新建 Profile」两个按钮。

## 表单弹窗

外层遮罩 `rgb(8 6 5 / 68%)` + `backdrop-filter: blur(2px)`。面板宽 720px，`max-height: 88vh`，圆角 12px。

- **头部**：平台 glyph 方块（34px）+ 标题（新建为 `新建 {平台名} Profile`，编辑为 `编辑 · {name}`）+ 副行配置路径（mono 11px）+ 右上关闭按钮。
- **正文**（可滚动，间距 16px）：
  1. 两列：`名称 *`（mono 输入，提示「写入 profiles.toml 的键名，唯一且不含空格」）+ `描述`（提示「列表里显示在名称下方」）。
  2. 整行：`BASE URL *`（mono 输入）。
  3. 两列：`模型` chip 单选组（来自 registry `models`）+ `{extraLabel}` chip 单选组（来自 registry `extraOpts`）。
  4. 认证分组框（内嵌深色底 `#191412`）：右上为 `api_key / oauth / no_auth` 段控件；当选中 `api_key` 时条件渲染 API Key 输入（占位「sk-··· 粘贴后本地加密存储」，提示「仅写入本机 keychain，导出配置时自动脱敏」）。
  5. `标签` chip 多选组（`work / free / backup / test`）。
- **底部**：左侧提示（新建为 `将追加到 {file}`，编辑为 `将覆盖 {file} 中的 [{name}]`）+ 「取消」+ 「保存」+ 「保存并应用」（accent 主按钮）。

校验：名称与 Base URL 为空时不提交，弹 toast「名称与 Base URL 必填」。保存后 toast `{name} 已保存` 或 `{name} 已保存并应用`。

## 平台元数据表（原型 `META`）

原型用一张表驱动全部平台差异，这是「统一设计语言 + 平台差异」的核心机制。

| key         | name        | title                | glyph | file             | col3 | col4   | extraLabel | extraOpts                    | models                                               |
| ----------- | ----------- | -------------------- | ----- | ---------------- | ---- | ------ | ---------- | ---------------------------- | ---------------------------------------------------- |
| claude      | Claude Code | Claude Code Profiles | C     | profiles.toml    | 模型 | 认证   | 最近使用   | 刚刚 / 今天 / 本周           | claude-sonnet-4.6, claude-opus-4.1, claude-haiku-4.5 |
| codex       | Codex       | Codex Profiles       | X     | CCR Unified      | 模型 | AUTH   | WIRE API   | responses / chat             | gpt-5.6-sol, gpt-5.6-high, gpt-5.6-mini              |
| grok        | Grok        | Grok Profiles        | G     | grok.toml        | 模型 | EFFORT | REASONING  | low / medium / high          | grok-4.6, grok-4-fast                                |
| antigravity | Antigravity | Antigravity Profiles | A     | antigravity.toml | 模型 | 区域   | 区域       | us-east / eu-west / cn-north | ag-pro-2, ag-lite                                    |
| opencode    | OpenCode    | OpenCode Profiles    | O     | opencode.toml    | 模型 | 模式   | 模式       | 默认 / 本地                  | oc-large, oc-small                                   |

配置路径（`path`，显示在页头与弹窗副行）：

- claude：`~/.ccr/claude/profiles.toml · 与 CCR Core 保持同步`
- codex：`~/.codex/profiles.toml · CCR Unified 模式`
- grok：`~/.ccr/grok/profiles.toml · 管理运行时与激活状态`
- antigravity：`~/.ccr/antigravity/profiles.toml`
- opencode：`~/.ccr/opencode/profiles.toml`

平台色四件套（`dot` / `bg` / `border` / `fg`）：

| key         | dot       | bg        | border    | fg        |
| ----------- | --------- | --------- | --------- | --------- |
| claude      | `#d97757` | `#33231b` | `#6b4028` | `#e8835b` |
| codex       | `#7cab82` | `#1f2a22` | `#3c5442` | `#93bf98` |
| grok        | `#a79bc4` | `#2b2637` | `#4b4463` | `#b3a8cc` |
| antigravity | `#98afc9` | `#212c38` | `#3d4d5e` | `#a8c0d8` |
| opencode    | `#735f52` | `#2a221d` | `#4a3d35` | `#c4b3a3` |

## 原型色板（暗色，供 token 映射参考）

- 背景层：`#17120f` 应用底 / `#1c1613` 表面与卡片 / `#221b18` 输入与控件 / `#241d19` hover 与次按钮 / `#191412` 内嵌凹陷 / `#1a1512` 空态 / `#2c2420` chip
- 边框：`#322a25` 分隔线 / `#2b2320` 卡片边框 / `#3a302a` 控件边框 / `#4a3d35` hover 边框
- 文本：`#f3eadf` 主 / `#dacbbc` 次 / `#c4b3a3` / `#b9a695` / `#8d7768` muted / `#977f6d` / `#7a675a` / `#6d5b4f` / `#5d4d43` / `#4d3f36`
- Accent：`#e8835b` 主 / `#f0926c` hover / `#20140c` accent 上文字 / `#33231b` accent 软底 / `#6b4028` accent 边框 / `#f0c3aa` accent 软底文字
- 状态绿：`#7cab82`

## 排版

- 等宽字体为 JetBrains Mono（原型内联 `@font-face`，权重 400/500/600）。用于：profile 名、Base URL、模型、字段 label、统计数字、徽标、chip、表头。
- 字段 label 统一为 mono 10px / 600 / `letter-spacing 0.14em` / muted；卡片内字段 label 为 `letter-spacing 0.1em`。
- 页面标题 21px / 600 / `-0.02em`；弹窗标题 17px / 600 / `-0.01em`。

## 与本仓库的偏差（必须在实现中处理）

1. 原型只有暗色，硬编码 hex。仓库要求明暗双主题且都要高对比，实现必须走 `--color-*` token。
2. 原型 `claude.extraLabel = 最近使用`，但 `ClaudeProfile` DTO 无 `last_used` 字段。该槽位需改用后端已有字段。
3. 原型的模型/标签为固定候选列表。实际候选需来自 registry 或后端，且必须允许自由输入。
4. 原型左栏平台切换器是演示装置。已确认路由保持 `/claude-code/profiles`、`/codex/profiles`、`/grok/profiles` 三条不变。
5. 原型「导入」按钮无行为定义，仓库现有的是「导出」（`exportAll`）。按钮集合以仓库实际能力为准。
6. 设计稿没有 Inspector 右栏。仓库规格保留 Inspector，卡片网格在 Inspector 展开时从三列降为两列（父任务决策 5）。
