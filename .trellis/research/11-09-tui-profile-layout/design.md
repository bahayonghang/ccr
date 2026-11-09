# Design：Profile 工作台满高布局

> 方向 A 已由用户批准（父任务 `direction-approved.md`）。

## 1. 边界

- 只改 `crates/ccr-tui/src/tui/ui/workbench.rs`（详情与表格渲染）和 `crates/ccr-tui/src/tui/ui.rs` 中 Profile 页的布局函数（`draw` 的运行态高度、`render_profile_workspace`、`footer_hints_for_width`）。
- 渲染只读 `App` 缓存快照；不新增渲染期 I/O，不新增 SQL。
- `workbench.rs` 已有 1874 行。新的分节布局放到新文件 `crates/ccr-tui/src/tui/ui/detail_layout.rs`，存储字段投影放到 `crates/ccr-tui/src/tui/ui/stored_fields.rs`，不继续堆进 `workbench.rs` 或 4449 行的 `ui.rs`。

## 2. 详情面板的新几何

当前流程：`detail_lines` 生成一个扁平行列表，`render_details` 把面板高度裁成 `lines.len() + 2`，一个 `Paragraph` 整体滚动。

新流程分两层：

1. `DetailSections`：由 `detail_lines` 拆出的结构化分节（identity、connection、usage、activity、elastic），每节自带"空态单行"和"有数据"两种行集。
2. `DetailLayout::new(inner: Rect, sections, platform)`：
   - 固定区 = identity + connection + usage + activity。connection 的高度按 **平台字段集**（不是当前 profile 实际有值的字段数）预留，保证同平台内 R2 的稳定性；无值字段在非展开模式下仍省略，但空出的行留作分节内空白，而不是让后面的分节上移。
   - 弹性区 = `inner.height - 固定区`，交给第 3 节的内容。
   - 固定区 + 弹性区最小高度超过 inner 时，退化为现有的整体滚动（保留 `profile_detail_scroll`、滚动条和行数指示）。

`app.detail_area` 继续存放 inner Rect，鼠标滚轮命中不变。

## 3. 弹性区域（方向 A：存储字段检查器）

- 数据源：缓存的 `ProfileConfig`（已在 `app.current_tab().profile_configs`），把核心字段与 `platform_data` 投影为 `key = value` 行，顺序为核心字段在前、`platform_data` 按插入顺序在后。不读文件，不渲染原始 TOML。
- 标题：`─ 存储字段 · <profiles.toml 路径>`，路径来自 `PlatformPaths` 在快照读取时已解析的值，渲染期不解析。
- 密钥：`auth_token`、`api_key` 一律显示 `‹已设置›` / 省略；Grok 不渲染任何 token（含掩码），与 spec 一致。`env_key` 只显示变量名。复用现有掩码与安全 URL 帮助函数（`configured_token_text`，`ui.rs:1529`）。
- 弹性区有自己的滚动偏移；PgUp/PgDn 在固定区放得下时滚动弹性区，放不下时滚动整体（与现有键位兼容）。

## 4. 其它打磨的落点

| 需求 | 落点 |
| --- | --- |
| R3 空分节单行 | `usage_ledger`（`workbench.rs:739`）与 `activity_lines`（`workbench.rs:940`）返回 `SectionBody::Empty { status }` / `SectionBody::Rows(..)`；分节标尺行右侧拼状态 |
| R4 身份行 | `detail_lines` 开头（`workbench.rs:1050-1093`） |
| R5 双列连接 | `connection_rows`（`workbench.rs:572`）增加 `columns: u8` 参数，配对规则：值宽 ≤ `(inner - gutter)/2 - label` 的相邻字段成对；Endpoint / Key / 长值独占 |
| R6 运行态单行 | `draw` 中 Profile 分支高度常量（`ui.rs:53-61`）与 `render_runtime`（`workbench.rs:315`）的单行分支阈值 |
| R7 上次使用 `—` | `last_used`（`workbench.rs:307-310`）仅改 `Some(Ok(_))` 无记录分支；`activity_lines` 仍显示文字版"无记录" |
| R8 页脚 | `footer_hints_for_width`（`ui.rs:2069-2124`） |

## 5. 兼容与回滚

- 不改数据模型、不改持久化，回滚即 revert 渲染文件。
- `profile_details_expanded`（`x`）继续控制是否显示未设置字段；展开时固定区可能超过 inner，自动进入整体滚动。
