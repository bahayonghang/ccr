# Research: MCP Manager — Tauri（ccr-ui）对照 GPUI（ccr-desktop）

- **Query**: 对照 Tauri MCP 管理页与 GPUI 移植版，找出 GPUI 页“朴素/未完成”的结构原因，盘点可复用原生原语与测试契约，给出不虚构数据的重设计方案
- **Scope**: internal（附 gpui-component 0.7.3 registry 源码核对）
- **Date**: 2026-11-07
- 本文只记录研究结论；“推荐重设计”一节是调用方明确要求的产出（§5）。

---

## 0. 结论速览

| 用户截图现象 | 根因（file:line） |
|---|---|
| lens 胶囊在标题**上方**左对齐 | MCP 走“上下文行”挂载：外壳在页面上方渲染 `page_header::row`（`ccr-desktop/src/shell/mod.rs:796-807`），只有 Dashboard 走嵌入式（`shell/mod.rs:789-791` `embeds_lens()` 仅 `Destination::Dashboard`）。`McpView` 与 `Converter` 都没有 `embed_lens` 槽 |
| 全宽空输入框，无标签/占位/图标 | 筛选框 `InputState::new(window, cx)` 未调用 `.placeholder()`（`features/mcp/mod.rs:53`），只有不可见的 `aria_label`（`mod.rs:545-547`），外层 `flex_1()` 撑满（`mod.rs:544`） |
| 三个纯文本按钮 | 全部 `.ghost()` 且无图标（`mod.rs:550-566`），没有主按钮 |
| “Batch presets and synchronization” 居中孤立 | 该按钮直接挂在 `flex_col` root 上（`mod.rs:571-583`），GPUI 默认 `align_items: None` ⇒ stretch（`gpui-pre-0.3.9/src/style.rs:794`），按钮被拉满宽，Kit Button 内部 `justify_center()`（`gpui-component-0.7.3/src/button/button.rs:659`）⇒ 文字居中 |
| 来源状态是一串句子 | `note_line()` 拼接整句（`mod.rs:350-406`），逐条作为 root 子节点纯文本输出（`mod.rs:701-720`） |
| 条目是灰色小字、间距 ~50px | 每个条目是 root 的独立子节点：`div().h(ROW_HEIGHT=36)` 包一个 ghost Button（`mod.rs:756-775`）；root 的 `gap(SECTION_GAP=16)`（`mod.rs:522`）作用于每条之间 ⇒ 行距 52px，Medium 按钮 `h_8`=32px（`button.rs:687-690`），14px 文字上下留白约 38px。没有容器/分隔线/背景 |
| drawio 出现两次 | **不是 bug，也不是重复读取**：本机 `~/.claude.json` 的 `projects` 下有两个不同项目键都定义了 `drawio`（只打印了名称和键长度的探针：键长 40 与 77）。读取器把每个项目键的条目都以 `McpScope::Project(key)` 产出（`runtime/mcp.rs:298-313`），但列表标签只拼 `平台 · 名称`（`mod.rs:745-754`），不显示作用域 ⇒ 视觉上重复。`entry.id()` 含作用域（`runtime/mcp.rs:79-86`），所以选择互不冲突 |
| 无命令/传输/作用域/env 信息、无徽章 | 列表行只用 `platform_name` + `name`；command、scope、has_url、has_args、enabled/disabled 只在点选后的 `detail_lines()` 中以整句出现（`mod.rs:417-475`）。env 数量**根本不在数据模型里**（`runtime/mcp.rs:53-62`） |
| 大片空白 | 没有列表/详情分栏，详情只在选中后追加到页面底部（`mod.rs:778-860`）；页面是单列 `flex_col` 长卷 |

---

## 1. Tauri（ccr-ui）MCP 管理页

### 1.1 文件

| File | 说明 |
|---|---|
| `ccr-ui/src/features/mcp/McpManagerView.tsx` | 页面壳：PageHeader + 预设抽屉 + 作用域胶囊 + 错误条 + MasterDetail |
| `ccr-ui/src/features/mcp/McpPresetsPanel.tsx` | 预设卡片网格 + 安装 modal |
| `ccr-ui/src/features/mcp/McpSyncPanel.tsx` | 同步面板：目标平台 chip、源服务器卡片、Sync All |
| `ccr-ui/src/features/mcp/useMcpManager.ts` | 按名称跨平台分组、作用域过滤、模糊搜索 |
| `ccr-ui/src/features/mcp/useUnifiedMcpList.ts` | `scopeCounts`（:79）、`sourceDiagnostics`（:162） |
| `ccr-ui/src/features/mcp/mcp-constants.ts` | `PLATFORM_META`（claude/codex/gemini 三色与图标，:3-7） |
| `ccr-ui/src/features/mcp/styles/mcp-manager.css` | `.mcp-action` pill 按钮、`.mcp-presets-drawer`、`.mcp-alert` |
| `ccr-ui/src/features/platform/mcp/McpListPanel.tsx` | 左列表：搜索头 + 行 |
| `ccr-ui/src/features/platform/mcp/McpDetailPanel.tsx` | 右详情：头部、Effective、Precedence、Env/Headers、Raw、Instances、Diagnostics |
| `ccr-ui/src/features/platform/mcp/McpCreatePanel.tsx` / `McpImportPanel.tsx` / `McpKvEditor.tsx` | 详情区内的新建/编辑/导入表单 |
| `ccr-ui/src/features/platform/mcp/BaseMcp.tsx` | 平台页（非统一管理）旧版 MCP 面：SurfacePage + 3 统计格 + NamedItemCard 列表 |
| `ccr-ui/src/features/platform/mcp/mcp-format.ts` | `maskSecret`、状态文案/色调映射 |
| `ccr-ui/src/features/platform/mcp/mcp-classes.ts` | 共享 Tailwind class |
| `ccr-ui/src/ui/list-search-header.tsx` / `agent-icons.tsx` / `master-detail-layout.tsx` | 共享原语 |

### 1.2 页面布局（`McpManagerView.tsx:165-281`）

1. **PageHeader**（:169-188）：标题 + 描述 + 右侧动作组：`Refresh`（ghost，`RefreshCw` 图标，loading 时 `animate-spin`）、`Import`（ghost，`Download`）、`Add server`（primary，`Plus`）。`.mcp-action` = pill（`border-radius: 999px`、`min-height: 2.25rem`、`font-weight: 650`，`mcp-manager.css:42-64`）。
2. **预设与同步抽屉**（:191-202）：原生 `<details>`，summary 为“标题 + 灰色副标题”单行（`mcp-manager.css:5-40`，底部 hairline），展开后依次渲染 `McpPresetsPanel` 和 `McpSyncPanel`，`max-height: 32rem` 内部滚动。批量/预设是**次级可折叠区**，不占主区。
3. **作用域胶囊**（:204）：`PillToggleGroup`，选项 effective / local / project / user / hidden，每项标签带计数 `${label} ${count}`（:43-46）。
4. **错误条**（:206-211）：`AlertTriangle` 图标 + danger tint（`mcp-manager.css:71-86`）。
5. **MasterDetailLayout**（:213-266）：列表宽 `23rem`；详情区按 `panelMode` 切换 Create/Edit、Import、Detail。
6. `BulkDeleteDialog`（:268-279）多选批量删除。

### 1.3 列表（`McpListPanel.tsx`）

- 搜索头 `ListSearchHeader`（:103-143）：`Search` 图标 + 带 `placeholder` 的输入（`list-search-header.tsx:24-33`），右侧 3 个 32px 图标按钮：多选切换（`LayoutGrid`/`CheckCircle2`）、添加下拉（手动创建/导入）、刷新（`mcp-classes.ts:36-41` `h-8 w-8 rounded-lg`）。
- 行解剖（:47-77）：`rounded-xl px-3 py-2.5 gap-2.5`，hover `bg-bg-elevated/72`，选中 `border-accent-primary/15 bg-bg-elevated/85`。
  - 左：传输图标 `Terminal`（stdio）/ `Globe`（http），`text-text-muted`。
  - 中三行：① 名称 `text-sm font-medium` + 可选“hidden N”警告小 pill（`rounded-full bg-warning/10 text-[0.58rem] uppercase`）；② 命令或 URL 预览 `text-xs text-text-muted`，截断 40 字（`mcp-format.ts:16-19`）；③ 作用域列表 `text-[0.62rem] uppercase tracking-wide text-accent-primary/82`，如 `Local / User`。
  - 右：`AgentIcons compact`，每个平台一个 chip：`color: var(--agent-color)`，背景 10% / 边框 20% 混色，`border-radius: 0.375rem`（`ccr-ui/src/ui/primitives.css:435-446`）。
- 空态：`EmptyState icon="Server"` + “添加服务器”动作（:146-154）。
- 分组：同名服务器跨平台/作用域合并为一行（`useMcpManager.ts:34-41`），`transportType` 由首个有效实例 `url` 是否存在推断（:15-19）。

### 1.4 详情（`McpDetailPanel.tsx`）

- 头部（:120-156）：44px 圆角方块图标（`border-accent-primary/22 bg-accent-primary/10`）；eyebrow `作用域 · 状态`（`text-[0.64rem] uppercase tracking-[0.14em]`）；名称 `font-serif text-[1.55rem]`；`STDIO · N instances`；右侧 Edit / Delete pill（Delete 为 danger 文字色）。
- Effective 区块（:158-203）：`rounded-2xl border bg-bg-surface p-4 shadow-sm`，右上 `StatePill`（:72-83，色调 `MCP_STATE_TONE_CLASS` ok/warning/danger/muted，`mcp-format.ts:53-58`）；2 列网格：Type、Command/URL（mono xs）、Args（mono）、Timeout、Cwd。
- Precedence（:205-246）：每实例一行，状态点 `h-2 w-2 rounded-full`、作用域名、`source_path`（mono）、hidden_by/approval 说明、StatePill。
- Env & Headers（:248-278）：key（mono bold）+ `maskSecret(value)`（头 4 尾 2，`mcp-format.ts:9-14`）。
- Raw config（:280-287）：`<pre>` 展示 `raw_config` JSON。
- Instances（:289-303）：平台 chip + 作用域 + Enabled/Disabled 文字 + 开关按钮（`ToggleLeft/Right`）。
- Diagnostics（:305-324）。
- 空态（:339-357）：居中图标 + 标题 + 副标题。

### 1.5 预设 / 同步面板

- `McpPresetsPanel.tsx:140-187`：`rounded-3xl border bg-bg-elevated p-6` 容器，标题 + 副标题 + 展开/收起；卡片网格 `grid-cols-1 md:2 lg:3 xl:5`（:154）；卡片（:33-46）顶部 tag pill（`rounded-full bg-accent-secondary/10 text-[0.625rem]`）与“需 API key”警告 pill（`KeyRound` 图标）。安装走 `BaseModal`，平台多选 chip（:190-212），API key 输入带 `placeholder={api_key_env}`。
- `McpSyncPanel.tsx:144-191`：标题行 + Refresh + `Sync all`（success 实色按钮，`Zap` 图标）；“目标平台”标签（`text-xs font-bold uppercase tracking-wider`）+ 平台 chip（:193-215，`min-h-11 rounded-xl`，active 为 success tint）；源服务器计数；空态为**虚线**面板（:176 `border-dashed`）+ `Server` 图标；`ServerCard`（:38-48）名称 + 来源平台 pill + mono 命令行井（`bg-bg-surface font-mono text-xs` + `Terminal` 图标）。

### 1.6 相关 Tauri DESIGN.md 规则（`ccr-ui/DESIGN.md`）

- 平台身份色只用于 2px tick、点、图例色块（:163-165、:363“Don't repaint panels or chrome in platform colors”）。
- Pill 适合独立动作或标签，不适合表格或整页（:211）。
- Mono 仅用于数据/代码/时间/命令（:187 “The Mono Is Earned Rule”）。
- 诚实空态：虚线基线、`—`，不伪造 0（:352）。
- 禁用/阻塞/破坏性状态必须一眼可读（:370）。

### 1.7 注意：Tauri 的数据面比 GPUI 宽，且掩码策略不同

Tauri 列表/详情会显示 URL 明文（`McpListPanel.tsx:70`、`McpDetailPanel.tsx:176`）、args 明文（:183）、raw config JSON（:284-286）、env/header 掩码预览（:254-275）。**GPUI 合同更严格：URL、args、env、header、token 一律不复制出读取器**（`ccr-desktop/src/runtime/mcp.rs:1-3`），这些 Tauri 元素不能照搬（见 §4）。

---

## 2. GPUI（ccr-desktop）MCP 页

### 2.1 文件

| File | 说明 |
|---|---|
| `ccr-desktop/src/features/mcp/mod.rs` | `McpView`：状态、读取、列表、详情、来源选择、预设入口 |
| `ccr-desktop/src/features/mcp/editor.rs` | `McpEditor`：新增/编辑/删除/预设/复制，确认对话 |
| `ccr-desktop/src/features/mcp/editor/form.rs` | 通用表单：Stdio 命令、参数、HTTP URL、env/header 替换对 |
| `ccr-desktop/src/features/mcp/batch.rs` | `McpBatch`：多目标预设安装与整源同步 |
| `ccr-desktop/src/features/mcp/tests.rs` | 渲染级测试（见 §4） |
| `ccr-desktop/src/runtime/mcp.rs` | 只读目录投影 `read_catalog` / `read_catalog_for_project` |
| `ccr-desktop/src/runtime/mcp/editor.rs` | `McpTarget`、`for_entry`、`common_fields` |
| `ccr-desktop/src/features/converter.rs` | 第二个 lens “Configuration converter”（不在 `configs/` 下；`features/configs/converter.rs` 不存在） |
| `ccr-desktop/src/shell/mod.rs`、`shell/page_header.rs` | lens 胶囊挂载 |

### 2.2 数据模型（`runtime/mcp.rs`）

```rust
pub struct McpEntry {            // runtime/mcp.rs:53-62
    pub source: McpSource,       // ClaudeState | ClaudeSettings | CodexConfig | AntigravityConfig | ProjectFile | OpenCode | Grok
    pub name: String,
    pub scope: McpScope,         // User | Project(String 项目键/绝对路径)
    pub command: Option<String>, // 原样命令字符串（非空才有）
    pub has_url: bool,           // url / serverUrl / httpUrl 任一非空（只留布尔）
    pub has_args: bool,          // args 非空数组（只留布尔，无数量）
    pub disabled: Option<bool>,
    pub enabled: Option<bool>,
}
pub struct McpCatalog { pub entries: Vec<McpEntry>, pub notes: Vec<McpSourceNote> } // :95-98
pub enum McpSourceState { Missing, Empty, Entries(usize), Invalid, Unreadable, NotRead } // :37-44
```

- 读取顺序与来源（`read_catalog`，:397-447）：ClaudeState（`~/.claude.json`，含 `projects.*.mcpServers`，:286-316）→ ClaudeSettings → Codex `config.toml`（`mcp_servers` 表，:356-387）→ Antigravity `mcp_config.json`；ProjectFile / OpenCode / Grok 固定写 `NotRead`（:433-435）。传入项目根时 ProjectFile 改为真实读取（`read_catalog_for_project`，:449-474）。
- 排序：source → name → scope（:436-441）。
- **没有的字段**：传输 `type`（http/sse/streamable 不解析，`json_bits` :210-245）、URL 值、args 数量/内容、env 键与数量、headers、timeout、cwd、source 文件绝对路径、进程/连接状态。
- 可编辑性：`McpTarget::for_entry`（`runtime/mcp/editor.rs:27-38`）——ClaudeState user→ClaudeUser、ClaudeState project→ClaudeLocal、ProjectFile→ClaudeProject、Codex、Antigravity；**ClaudeSettings 等返回 `UnsupportedSource`（只读旧来源）**。
- 平台/来源文案：`platform_name`（`mod.rs:327-334`，ClaudeState/Settings→“Claude Code”，ProjectFile/OpenCode/Grok→“MCP”）、`source_title`（:336-348）。

### 2.3 渲染结构（`McpView::render`，`mod.rs:509-863`）

root：`flex_col size_full overflow_y_scroll p(24) gap(16)`（:512-522），依序：

1. h1 “MCP Manager”（:523-532，`TITLE_SIZE` 20 semibold，`Role::Heading`）。
2. 一段长描述纯文本（:533-537）。
3. 工具行（:538-567）：`mcp-filter` Input（`flex_1`）+ `mcp-add`“Add server and select source” + `mcp-presets`“Built-in presets” + `mcp-read`“Read local files / Reading”。全部 ghost，经 `semantics::button`（pill 角）。
4. 若 `batch` 打开：直接 `return root.child(batch)`（:568-570），后续全部不渲染。
5. `mcp-batch-open`“Batch presets and synchronization” ghost 按钮（:571-583），**被 stretch 成全宽，标签居中**（见 §0）。
6. 若 editor 打开：`return root.child(editor)`（:584-586）。
7. `show_presets`：一句说明 + 预设 ghost 按钮 flex-wrap（:587-607）。
8. `show_sources`：说明句 + 胶囊（复用 `page_header::capsule`）内 5 个目标 tab（`mcp-source-user/local/project/codex/antigravity`，按 `platform_scope` 过滤）+ `mcp-project-root` 输入（同样无 placeholder，`mod.rs:62`、:674）+ 说明 + `mcp-open-editor` primary（:608-677）。
9. busy：`mcp-status`“Reading local MCP files.”（:678-690）；错误：`mcp-status` + `service_error`（:691-699）。
10. 来源注记 `mcp-sources`：每条 `mcp-source-{index}`，`aria_label` 与文本相同（:700-720）。
11. 空态 `mcp-empty`（:721-733）/ 筛选无果 `mcp-filter-empty`（:734-739）/ 条目 `mcp-entry-{index}`（:740-776），标签 `平台 · 名称`，disabled==Some(true) 追加 “· disabled in the file”（:745-754）。
12. 选中详情 `mcp-detail`（:778-860）：`aria_label` = 7 行拼接；操作行 `mcp-copy` / `mcp-edit` / `mcp-delete`（:798-850）或只读说明（:852-854）；随后 7 行句子（:856-858）。

**注意 id 冲突**：来源注记的 id 前缀 `mcp-source-{index}`（:714）与“选择新服务器来源” tab 的 `mcp-source-user/local/...`（:629-643）共享前缀，但实际值不同（数字 vs 单词），目前不冲突。

### 2.4 来源状态句的生成（`note_line`，`mod.rs:350-406`）

| 状态 | EN 文案 |
|---|---|
| Missing | `{title}: not found.` |
| Empty / Entries(0) | `{title}: no MCP entries.` |
| Entries(n) | `{title}: 1 entry.` / `{title}: {n} entries.`（中文 `{title}：{n} 条记录。`） |
| Invalid | `{title}: not valid MCP configuration. No entries were taken from this file.` |
| Unreadable | `{title}: could not be read.` |
| NotRead + ProjectFile | `Claude project .mcp.json is not read. This window has no project root.` |
| NotRead + OpenCode | `OpenCode MCP is not read by this page. Use the OpenCode settings source editor.` |
| NotRead + Grok | `Grok MCP credentials are not listed as servers.` |

这些完整句是测试断言的 a11y label（§4），改为 chip 时需保留为 `aria_label` 或 tooltip 文本。

### 2.5 空输入框

- `mcp-filter`：按名称**子串**过滤（`visible()`，`mod.rs:312-324`，`to_lowercase().contains`），只匹配 `entry.name`。`InputState::new(window, cx)` 无 placeholder（:53），无可见 label/图标，`aria_label` “Filter MCP entries by name”（:547）。
- Kit 能力：`InputState::placeholder()` 可用（仓库先例 `ccr-desktop/src/probe.rs:22`）；`Input::prefix(impl IntoElement)` / `suffix` / `cleanable(bool)` 存在（`gpui-component-0.7.3/src/input/input.rs:276-317`）。仓库目前无 `.prefix(` 使用先例。
- 可见 label 先例：会话页“标题”12px muted 标签 + Input 纵排、固定 280px（`features/insights/product.rs:235-256`）。

### 2.6 lens 胶囊位置（`shell/mod.rs`、`page_header.rs`、`DESIGN.md`）

- lens 选项：`Destination::McpManager => [("tool-mcp", Primary, "MCP Manager"), ("tool-converter", Converter, "Configuration converter")]`（`shell/mod.rs:861-869`）。
- `toolbar()`（`shell/mod.rs:793-807`）：平台目的地走 `platform_toolbar`；`embeds_lens()` 为真则不画；否则 `page_header::row("lens-navigation", None, capsule)`，该行 `px(24) pt(16)`（`page_header.rs:38-62`）。外壳把 toolbar 放在 body 之上（`shell/mod.rs:1102-1114`），页面再自带 `p(24)` 和 h1 ⇒ 胶囊在标题上方。
- 嵌入式：`LensBar` 视图由外壳拥有（`page_header.rs:64-156`），Overview/Monitoring 通过 `embed_lens(AnyView)` 接收（`shell/mod.rs:267-271`；`features/overview.rs:81-83`、:322-324；`features/monitoring.rs:84`、:475-491），标题与胶囊同一 `flex_wrap items_center gap(16)` 行。无选项时 LensBar 渲染空 div（`page_header.rs:134-136`），平台作用域下 `lens_items` 返回 None（`shell/mod.rs:810-813`）。
- `ccr-desktop/DESIGN.md:186-206` 写明迁移方法：“A destination moves to the embedded form by giving every page it hosts an `embed_lens` slot in its heading row and adding it to `Shell::embeds_lens`.” ⇒ MCP 目的地需同时给 `McpView` 和 `Converter` 加槽（Converter 自带标题在 `converter.rs:105`，且该标题没有 `Role::Heading`）。
- `McpView` 还被平台 lens 复用（Claude 工作台 `tool-platform-mcp`，`shell/mod.rs:426-431` `open_platform`），此时外壳渲染平台工具栏（`platform_navigation.rs:58`）。

### 2.7 编辑器与批量

- `McpEditor`（`editor.rs:390-517`）：标题 `来源 · 作用域 · 名称`、说明句、`mcp-name`、`mcp-editor-advanced` 切换 JSON 补丁 `mcp-patch`（Textarea，h 180）、`mcp-editor-message`、确认区 `mcp-confirmation`（`Role::AlertDialog` + focus trap，`mcp-confirm-cancel` / `mcp-confirm-save`）、`mcp-editor-close` / `mcp-editor-save`；非高级模式在**底部**渲染 `mcp-common-form`（:513-515）。
- 表单（`editor/form.rs:242-249`）：每个字段是“纯文本标签 + Input”：Stdio 命令 `mcp-command`、参数 `mcp-args`、HTTP URL `mcp-url`（**masked**，form.rs:36）、env/header 替换对（值 `masked(true)`，form.rs:64、:227）。
- `McpBatch`（`batch.rs:419-563`）：标题 20px “MCP batch operations”、模式切换（`mcp-batch-mode-preset` / `-sync`，ghost+selected，不是 `semantics::tab`）、来源/预设/目标按钮组、`mcp-batch-credential`（masked，:130）、`mcp-batch-replace`、`mcp-batch-review`（primary）、`uniform_list` 结果行 `mcp-batch-row-{i}`（36px，:476-498）、确认 `mcp-batch-confirmation`、`mcp-batch-new` / `mcp-batch-close`。批量视图打开时**替换整个列表区**（`mod.rs:568-570`）。

---

## 3. 可复用 GPUI 原语清单

### 3.1 `ccr-desktop/src/theme.rs`

| 原语 | 位置 | 用途 |
|---|---|---|
| 尺寸常量 `CONTENT_PADDING 24`、`SECTION_GAP 16`、`CONTROL_GAP 8`、`ROW_HEIGHT 36`、`META_SIZE 12`、`TITLE_SIZE 20`、`SECTION_TITLE_SIZE 16` | `theme.rs:12-37` | 间距/字号 |
| 圆角 `SELECTION_RADIUS 4`、`INPUT_RADIUS 8`、`PANEL_RADIUS 12`、`PILL_RADIUS 9999`、`CAPSULE_RADIUS 20` | `theme.rs:20-25` | |
| `SPLIT_WIDTH 1200`、`NARROW_WIDTH 1040` | `theme.rs:29-30` | 列表/详情分栏断点 |
| `Palette`（success/warning/danger/info/output/primary/muted/secondary_text…） | `theme.rs:105-169` | 状态色 |
| `platform_color(id, mode)`：claude / codex / grok / antigravity\|gemini / opencode，其余回落 muted | `theme.rs:273-283` | 平台身份墨色 |
| `chart_series` | `theme.rs:287-293` | 降饱和身份色 |
| `tab_colors` / `tab_selected` | `theme.rs:208-243` | 分段胶囊 |
| `nav_colors` | `theme.rs:250-265` | 半透明选中 pill（侧栏专用） |
| `hairline(mode)` | `theme.rs:204-206` | 装饰分隔线 |
| `tabular_figures()` | `theme.rs:297-305` | 数字 tnum |

### 3.2 `ccr-desktop/src/shell/`

- `semantics::button(button, disabled, busy, cx)`：pill 角 + 禁用/忙碌 a11y（`semantics.rs:16-38`）。
- `semantics::tab(button, selected, disabled, cx)`：分段选项（浮起 pill、toggled 语义，`semantics.rs:44-82`）。
- `page_header::capsule(cx)`（`page_header.rs:21-35`）、`page_header::row`（:38-62）、`LensBar`（:64-156）。
- 侧栏图标映射 `IconName`：McpManager→`Network`、ClaudeCode→`Bot`、Codex→`Cpu`、Grok→`Asterisk`、Antigravity→`Globe`、OpenCode→`SquareTerminal`（`platform_navigation.rs:6-20`）。
- 侧栏身份色用法：未选中中性，hover/选中显示身份色（`shell/mod.rs:645-660`）。

### 3.3 页面内已有的“徽章/点/面板”先例（未抽成共享组件）

| 先例 | 位置 | 形态 |
|---|---|---|
| `status_dot(color)` | `features/overview.rs:967-974` | 7×7 圆点 |
| 可用性 pill | `features/insights/product.rs:212-228` | `h 28 px 8 rounded PILL bg secondary META_SIZE`，ok 用 primary 色，否则 muted |
| 平台格 2px 身份色顶线 | `features/overview.rs:1094-1115` | absolute 顶线 `h 2 bg ink` |
| 身份色进度条 | `features/workbench.rs:455-470` | |
| mono 数据列 | `features/workbench.rs:472-480` | `font_family(cx.theme().mono_font_family)` + META_SIZE |
| 实色空态面板 | `features/insights/product.rs:272-283` | `bg background border_1 rounded INPUT_RADIUS muted` |
| Tooltip | `features/overview.rs:1061`、:1090 | `.tooltip(\|w,cx\| Tooltip::new(..).build(w,cx))` |
| `uniform_list` 固定行高 | `features/mcp/batch.rs:476-498` | 36px 行虚拟列表 |
| Kit Icon | `shell/settings.rs:385` | `Icon::new(IconName::Check).size(px(16.))` |

### 3.4 Kit 中存在但仓库未用的组件

- `Tag`（`gpui-component-0.7.3/src/tag.rs:125-197`）：`primary/secondary/danger/success/warning/info/custom(color, fg, border)/color(ColorName)`、`.outline()`。`ccr-desktop/src` 中 0 处使用。
- `Badge`（`src/badge.rs:31`）。
- 若采用需在实现阶段确认其圆角/字号是否符合 DESIGN.md（数据列表“不用 pill 包每一项”，`ccr-desktop/DESIGN.md:161`）。

### 3.5 `ccr-desktop/DESIGN.md` 相关规则

- 平台身份“为小图标与文字，不给整个页面染色”；status 色只表示真实状态且配文字或图标（:103）。
- 分段控件：内容宽度胶囊、选中浮起 pill（:107）。
- 代码/路径/命令文本用系统 mono（:122）；数字用 tnum 不用 mono。
- 主列表与详情默认 3:2，列表 ≥360px、详情 ≥320px；不足时详情转为内容区内 drill-in；列表行 36px、表头 32px，独立滚动（:142）。
- 主/次按钮 pill、输入 8px、列表可选背景 4px、内容面板 12px；Lucide 图标；**数据列表不用 pill 包每一项**（:159-161）。
- 阅读面（列表/输入/表格）一律实色，不叠玻璃（:148-152 Elevation 条件 1-3）。
- 无数量时显示“暂无数据”/不适用，不用假零（:126）。

### 3.6 原生能力限制（`.trellis/research/archive/2026-11/11-07-gpui-airy-glass-polish/research/native-capabilities.md`）

- dashed border 原生支持：`border_dashed()`，宜 1px + 小/零圆角（§3）。
- `overflow_x_scroll` 需元素有 id，位于 `InteractiveElement`（§4）——可用于来源 chip 条窄宽横滚。
- pill：`.rounded(px(PILL_RADIUS))` 或 `semantics::button`（§5）。
- 自定义 SVG 需 `Icon::data(bytes)` 或组合资产源；品牌用 PNG（§1/§6）。**无平台品牌 SVG 图标资产**；平台标识当前只用 Lucide 图标 + 身份色。
- 无 radial gradient、无 backdrop 采样；OS Mica/Blur 不用。

---

## 4. 测试与契约（必须保留或同步更新）

### 4.1 `ccr-desktop/src/features/mcp/tests.rs`

| 测试 | 断言要点（行号） |
|---|---|
| `mcp_page_lists_real_entries_and_hides_secret_fields` | 等待 `mcp-entry-4`（:124-127）；`page-mcp-manager`、`tool-converter` 可见，`nav-converter` 不存在（:130-132）；label 中含 5 个名称、`disabled in the file`（:134-139）；**来源整句** `Claude state file: 2 entries.`、`Claude settings.json: 1 entry.`、`Codex config.toml: 1 entry.`、`Antigravity mcp_config.json: 1 entry.`、`Claude project .mcp.json is not read`、`OpenCode MCP is not read`、`Grok MCP credentials are not listed as servers`（:140-146）；不含 `mcp-secret-sentinel`、`Home insights`、`Apply / refresh`、`no MCP manager connection`（:147-150）；点 `mcp-entry-1` 详情含 `Command: fixture-mcp-command`、`Arguments are set in this entry.`，不含 URL（:151-157）；`mcp-entry-0` 含 `fixture-project`、`A URL is set in this entry.`，不含 `https://`/`example.test`（:158-165）；`mcp-entry-2` 含 `disabled is true in the file.`，**不含 `TOKEN`、`apiKey`**（:166-172）；`mcp-entry-3` 含 `enabled is false in the file.`，不含 `bearer`（:173-179）；`mcp-entry-4` 含 `Command: agy`，不含 `Authorization`（:180-185）；`mcp-filter` 输入后 `mcp-entry-0` label 含 `codex-server`、`mcp-entry-1` 与 `mcp-detail` 消失（:186-202） |
| `mcp_page_with_no_files_stays_empty` | `mcp-empty` 文案 `No MCP entries were found in the files this page reads.`；`Claude state file: not found.` 等（:227-230）；无 `mcp-entry-0`；不创建 fixture 目录（:234） |
| `preset_and_cross_source_copy_use_real_controls_and_keep_stored_credentials_hidden` | 点击序列 `mcp-presets` → `mcp-preset-fetch` → `mcp-source-codex` → `mcp-open-editor`（:262-268）；`mcp-source-codex` checked（:266-267）；`mcp-copy` 按钮不得被 `mcp-manager` 视口裁切（:290-295）；复制后不显示 `copy-secret-sentinel`、不存在 “MCP JSON patch” 节点（:303-313） |
| `mcp_native_actions_write_confirm_and_keep_conflicting_drafts` | `mcp-add` → `mcp-open-editor`（:359-360）；编辑时补丁初值必须为 `{}`（:408-412，“existing secrets must not populate patch controls”）；冲突提示 “The source changed.”（:421）；`mcp-edit`、`mcp-delete`、`mcp-editor-close` 等 id |
| `batch.rs` 内 `rendered_batch_review_focus_escape_confirm_and_navigation_keep_safe_receipt` | 直接用 `McpView::new` 窗口（`batch.rs:594-600`），点 `mcp-batch-open` → `mcp-batch-mode-sync` → `mcp-batch-review`；焦点在 `mcp-batch-back`，Tab/Shift-Tab 循环、Esc 返回 `mcp-batch-review` 聚焦（:617-636）；`suspend` 后 `batch` 为 None、`last_batch` 保留（:658-660）；快照不含凭据（:691） |
| `editor/form/tests.rs`（13.1K，未逐行展开） | 表单字段 id/掩码行为 |

**条目索引依赖排序**：测试中 `mcp-entry-0` = project-server（ClaudeState Project）、`-1` state-server、`-2` settings-server、`-3` codex-server、`-4` agy-server（由 `runtime/mcp.rs:436-441` 排序决定）。若改为分组/分区渲染或改变 id 分配，需同步改测试。

### 4.2 `ccr-desktop/tests/native_shell.rs`

- `nav-mcp-manager` → `tool-converter` → `converter-acknowledge`，离页后输入/输出清空（:300-310）。
- 导航表 `("nav-mcp-manager", "page-mcp-manager", "MCP Manager")`（:582）；焦点/可见性（:638-641）。
- 平台工作台 `tool-platform-mcp`、`workbench-mcp` 存在于 Claude、不存在于 Codex（:893、:906、:956-957）。

### 4.3 运行时测试

- `runtime/mcp.rs:585-760` 系列：`format!("{catalog:?}")` 不含 `mcp-secret-sentinel`（:614、:684、:729）；非法结构时连命令都不采用（:730）。

### 4.4 掩码/安全契约清单（重设计不得破坏）

1. 读取器只复制 name / scope / command / has_url / has_args / disabled / enabled；env、headers、args、URL、token 不出读取器（`runtime/mcp.rs:1-3`、:53-62）。
2. 列表、详情、a11y label 中不得出现 URL 值、args 值、env/header **键名与值**（测试断言不含 `TOKEN`、`Authorization`、`bearer`、`apiKey`：tests.rs:170-171、:178、:184）。⇒ 显示 env 键名会直接违反现有断言；env 数量目前也没有数据。
3. 编辑器不预填已有密钥：补丁初值 `{}`（tests.rs:408-412；`editor.rs:106-109`）；URL、env/header 值输入框 `masked(true)`（`form.rs:36`、:64、:227）；批量凭据输入 masked 且保存后清空（`batch.rs:130`、:267-268）。
4. 跨源复制在后端不透明会话中保留明文，UI 草稿与操作快照不含凭据（`.trellis/spec/ccr-app/backend/shared-desktop-services.md:51-53`；tests.rs:303-313；batch.rs:691）。
5. 写入前确认 + 原始版本冲突检测，草稿保留（spec :79；tests.rs:421-428）。
6. 本页不启动服务器、不推断进程状态（`mod.rs:1`、:535）——**不得新增“Running/Connected”类徽章**。
7. Converter：需确认后才打开、离页清空（`converter.rs:53-62`、:106-108；native_shell.rs:300-310）。

---

## 5. 推荐重设计（GPUI）——逐项标注数据来源

图例：**[有]** = 现有 `McpCatalog`/状态可直接或派生得到；**[新]** = 需要新增读取字段（需评估掩码契约）；**[无]** = 不应做。

### 5.1 页头（与 Overview/Monitoring 对齐）

- h1 “MCP Manager” + 外壳 lens 胶囊（MCP Manager | Configuration converter）同一行，左侧；右侧放全页动作。**[有]** 做法：给 `McpView` 和 `Converter` 各加 `embed_lens` 槽、把 `Destination::McpManager` 加入 `Shell::embeds_lens()`（`shell/mod.rs:789-791`；DESIGN.md:200-206）。Converter 标题补 `Role::Heading` + aria_level 1，保证每页只有一个 h1。
- 描述压成一行 12-14px muted 副标题（截断/换行受控），长文中“密钥不出现 / 不启动服务器”保留在描述或页脚说明。**[有]**
- 页头下一行状态（类似 Overview 状态行）：`本地 · 读取于 HH:mm`。**[新]**（当前不记录读取时间；可省略）

### 5.2 工具栏

- 带可见标签或前置 `Search` 图标的筛选框：`InputState::placeholder("Filter by name / 按名称筛选")` + `Input::prefix(Icon::new(IconName::Search))`，固定宽 ~280-320px，不再 `flex_1` 撑满（`mod.rs:544`）。保留 id `mcp-filter`、aria label。**[有]**（仅匹配 name；若要匹配命令需改 `visible()`，数据 **[有]**）
- 主按钮 `mcp-add`：primary + `Plus` 图标，文案缩短为 “Add server / 添加服务器”。**[有]**
- 次按钮：`mcp-presets`（`Sparkles`/`Package` 图标）、`mcp-batch-open`（`RefreshCw`/`Layers`，文案 “Batch & sync”）、`mcp-read`（`RefreshCw` 图标，busy 时 “Reading”）。全部挪进同一行右侧，**不再单独占行**，从根本上消除 stretch 居中问题。**[有]**
- 可选：来源/作用域过滤胶囊（All · Claude · Codex · Antigravity，或 User · Project），带计数。计数由 `catalog.entries` 派生。**[有]**（Tauri 对应 `McpManagerView.tsx:204`；GPUI 没有 effective/hidden/precedence 概念 → 这两项 **[无]**）

### 5.3 来源健康条（替换句子列表）

- 一条紧凑的 chip 条（实色阅读面或 hairline 分隔的单行），每个 `McpSourceNote` 一个 chip：`状态点 + 来源短名 + 读数`。
  - `Entries(n)` → success 点 + `Claude state · 5`；**[有]**
  - `Empty` / `Entries(0)` → muted 点 + `Claude settings · 0 entries`（“无条目”是真实读数，不是假零）；**[有]**
  - `Missing` → muted 空心点 + `Antigravity · not found`；**[有]**
  - `Invalid` / `Unreadable` → warning/danger 点 + 文字 “invalid / unreadable”；**[有]**
  - `NotRead` → 虚线边框 chip（`border_dashed()`）+ “not read”，原因（`note_line` 原句）放 Tooltip。**[有]**
- 每个 chip 的 `aria_label` 继续使用 `note_line()` 的完整句，保留测试断言（tests.rs:140-146、:228-230）和 id `mcp-source-{index}`。
- 来源短名可带平台身份色小点/图标（`theme::platform_color`；ProjectFile→claude、OpenCode→opencode、Grok→grok）。**[有]**
- 窄宽时 `id + overflow_x_scroll()` 横滚，不换成多行。

### 5.4 服务器列表（列表/详情分栏）

- 布局：宽度 ≥ `SPLIT_WIDTH`(1200) 时 3:2 列表/详情（列表 ≥360、详情 ≥320），以下时详情转为 drill-in（DESIGN.md:142）。列表放在一个实色阅读面（`PANEL_RADIUS` 12、`border_1`），行间 hairline，**行高 36px 且不再受 root gap 影响**（把条目放进单个列表容器，容器内 gap 0/4）。
- 行解剖（左→右）：
  1. 传输图标：`command.is_some()` → `SquareTerminal`（stdio）；`has_url` → `Globe`（HTTP）。**[有·派生]**；区分 http/sse/streamable **[新]**（需解析 `type` 字段，不涉及密钥）。
  2. 名称（14px medium，ellipsis）。**[有]**
  3. 次行 mono 12px muted：`command`（截断 + tooltip 全文）；无命令且 `has_url` 时显示 “URL set / 已设置 URL”（**不显示 URL 值**）。**[有]**
  4. 作用域标签（12px muted 文字，不是 pill）：`User` / `Local · <项目名末段>` / `Project · <末段>`，完整项目键进 tooltip。**解决 drawio 重复的视觉歧义**。**[有]**（`McpScope::Project(key)`）
  5. 平台标识：Lucide 图标 + 平台名，图标/2px tick 用 `platform_color`，不整块染色（DESIGN.md:103）。**[有]**
  6. 右侧小标签：`disabled`（`disabled==Some(true)` 或 `enabled==Some(false)`）→ muted/warning 文字 + 图标；`args` 有/无 → “args” 小字。**[有]**；args 数量 **[新]**；env 数量 **[新]**（且需更新“不得出现 TOKEN”之外的契约评估；**只可显示数量，不可显示键名**，否则违反 tests.rs:170）。
  7. 只读来源（`McpTarget::for_entry` 为 Err，例如 ClaudeSettings）→ `Lock` 图标 + “read-only”。**[有]**
- 可选按来源分组的小节标题（12px muted + hairline，类似 `shell/mod.rs:768-787` 的 `section_title`）。**[有]**；若分组需保证 `mcp-entry-{index}` 的全局序号不变或同步改测试。
- 空态：`mcp-empty` 改为实色/虚线面板 + `Network` 图标 + 文案 + “Add server” 主动作（Tauri `McpListPanel.tsx:146-154`、`McpSyncPanel.tsx:176-180`）。**[有]**；`mcp-filter-empty` 同理。
- 不做：运行/连接状态、approval、precedence/hidden_by、enable 开关直接写入（GPUI 没有该数据或流程）。**[无]**（开关可通过现有 Edit 补丁 `{"disabled":true}` 实现，但那是新交互，需单独评估）

### 5.5 详情面板

- 头部：传输图标方块 + 名称（16-20px semibold）+ eyebrow `平台 · 来源文件 · 作用域`；右侧 `Copy` / `Edit` / `Delete`（Delete 用 danger 文字，pill），只读时显示只读说明。**[有]**（保留 `mcp-copy` / `mcp-edit` / `mcp-delete` 且不被视口裁切，tests.rs:290-295）
- 键值网格（label 12px muted 在上、值在下；命令值 mono）：
  - Transport：stdio / HTTP **[有·派生]**
  - Command：`command` mono **[有]**
  - URL：“Set（hidden）/ 未设置” **[有]**——不显示值
  - Arguments：“Set（hidden）/ 未设置” **[有]**
  - Scope：User / Project key（mono，长路径换行）**[有]**
  - Source file：`source_title` **[有]**；绝对路径 **[新]**
  - enabled / disabled：`flag_line` 语义（文件未设置时写“未设置”）**[有]**
- `mcp-detail` 的 `aria_label` 继续为 7 行拼接（tests 依赖 `Command: …`、`Arguments are set in this entry.`、`A URL is set in this entry.`、`disabled is true in the file.`、`enabled is false in the file.` 等子串，tests.rs:154-183）。可视层改为键值，但文案句子保留在 aria/label 中，或同步改测试。

### 5.6 新增/预设/来源选择

- “Add server” 打开详情区（或 drill-in）中的“选择目标”步骤：5 个目标用现有胶囊 `semantics::tab`（已是），项目根输入框加 placeholder（`/abs/path/to/project`）与可见标签（`mod.rs:62`、:674）。**[有]**
- 预设：从按钮流改为紧凑卡片/行网格：名称 + description（12px muted）+ tags 文字 + “需 API key”（`KeyRound` 图标 + `api_key_env` 名称，这是变量名而非值）。数据来自 `ccr_skills::McpPreset`（`crates/ccr-skills/src/models/mcp_preset.rs:10-26`：id/name/description/tags/homepage/docs/requires_api_key）。**[有]**（当前只用了 `name`）
- 保留 id `mcp-presets`、`mcp-preset-{id}`、`mcp-source-*`、`mcp-open-editor`、`mcp-add`。

### 5.7 批量 / 同步

- 从“孤立全宽按钮 + 替换整页”改为工具栏次按钮打开，主体渲染在详情区（或页内二级区块），带分区标题 “Batch presets & sync”。模式切换 `mcp-batch-mode-preset/-sync` 改用 `semantics::tab` 胶囊（当前是 ghost+selected，`batch.rs:296-321`）。目标/来源按钮组同理。**[有]**
- 结果列表已有 `uniform_list` 36px 行（`batch.rs:476-498`），可加平台图标与状态点（Saved=success、Failed=danger、Pending/NotRun=muted，均配文字）。**[有]**
- 保留焦点契约：`mcp-batch-back` 首焦点、Tab 循环、Esc 回 `mcp-batch-review`（batch.rs:617-636），`suspend` 行为（mod.rs:90-102）。

### 5.8 Converter lens

- 同样嵌入 lens 到标题行；当前标题无 heading 语义、动作按钮与两个 180px Textarea 纵排（`converter.rs:105-180`）。本次只要求 lens 对齐时需改。**[有]**

---

## Caveats / Not Found

- `ccr-desktop/src/features/configs/converter.rs` 不存在；第二个 lens 实现在 `ccr-desktop/src/features/converter.rs`。
- drawio 重复的结论基于对本机 `~/.claude.json` 的只读探针（只输出服务器名与项目键长度，未输出任何值）。用户截图环境若不同，原因仍是同类：同名条目位于不同作用域或不同来源，而标签不含作用域。
- 截图中 “Claude state file: 5 entries” 而列表只看到部分 Claude 条目，按本机数据实际为 exa（user）、chrome-devtools、context7、drawio×2（project）；可能是截图滚动范围所致，未验证。
- `editor/form/tests.rs` 未逐行审阅；改动表单前需读。
- Kit `Tag`/`Badge` 的默认圆角/字号未实测；是否符合 DESIGN.md 需在实现阶段截图验证。
- GPUI 默认 `align_items` 为 `None`（taffy 视作 stretch）的推断来自 `gpui-pre-0.3.9/src/style.rs:794` 与现象吻合，未做单独渲染探针。
- 新增 env 数量、args 数量、传输类型、源文件路径属于运行时读取器改动（`runtime/mcp.rs`），需同时评估 `.trellis/spec/ccr-app/backend/shared-desktop-services.md:51-81` 的掩码契约并补测试。
