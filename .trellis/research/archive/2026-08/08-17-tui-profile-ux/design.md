# 技术设计 — TUI Profile 界面设计优化

> 行号基准:2026-08-17 规划评审时的 `main`。所有引用都已对照真实代码核验过一遍;实施时若与实际不符,以代码为准并在此更正。

## 边界

代码改动只在 `crates/ccr-tui/src/tui/`:`app.rs`、`action.rs`、`ui.rs`、`footer.rs`(仅在 `ShortcutHint` 结构需要变动时)、`theme.rs`(仅 token 用途调整,不改色值)。不触碰 `ccr-cli` / `ccr-codex` / `ccr-config`,TUI 层只改变「何时调用 apply、调用后是否退出、结果显示在哪」。

`docs/**`、`CHANGELOG.md`、`.trellis/spec/**` 由 WS6 单独收尾。

## WS1:切换安全闭环

**现状契约(已核验)**

- `KeyCode::Enter => Action::ApplyAndQuit`(`app.rs:758`),`Space => Action::ApplySelected`(`app.rs:759`)。
- `Action::ApplyAndQuit` 的全部引用:`action.rs:34` 定义、`app.rs:758` 映射、`app.rs:852` dispatch、`app.rs:1688` 测试。没有鼠标路径使用它。
- `apply_selected()`(`app.rs:927-1002`)成功分支已写 `self.last_applied = Some((platform_label, name, true, None))`(`app.rs:979`)。
- `last_apply_message`(`ui.rs:1993-2014`)输出的是「应用成功」,**不含 profile 名**,需要改成「已切换到 {name}」。
- 常驻反馈当前只经 `render_profile_status_strip`(`ui.rs:775-791`)显示,而该 strip 只在 `ViewportMode::Wide` 分支渲染(`ui.rs:641-652`),Wide 门槛是 `width >= 120 && height >= 22`(`theme.rs:22-30`)。

**设计**

- `Enter` 与 `Space` 都映射到 `Action::ApplySelected`(应用并停留)。
- **删除 `Action::ApplyAndQuit` 变体及其 dispatch 分支**。留着会变成 never-constructed 变体,`just lint-strict`(clippy `-D warnings`)会因 dead_code 失败;`SwitchTab` 那种 `#[allow(dead_code)]` 不该被复制到一个已经没有语义的变体上。
- 常驻反馈迁到 Focus 面板:
  - `profile_summary_fields`(`ui.rs:1951-1991`)增加一个可选的第三个字段(标签沿用「Status」体系之外的独立 label,如「Last apply / 最近切换」),值为「已切换到 {name}」/「切换失败({err})」,tone 分别为 `Success` / `Error`。
  - 该函数需要新增一个入参把 `app.last_applied` 传进来;`summary_height = summary.len() + 2`(`ui.rs:639`)已经是自适应的,三档视口会自动多留一行,不需要改布局常量。
  - `render_profile_context_workspace` 的 Wide 与非 Wide 两条分支都走同一个 `profile_summary_fields`,所以三档视口一次到位。
- Status strip 退化为纯 toast 通道:`profile_status_message`(`ui.rs:793-796`)去掉 `last_apply_message` 分支,只保留 `app.toasts.active()`。这样 Wide 下不会出现 Focus 与 strip 同时显示同一句话。

**已否决方案**

- *主 App 接入 `Overlay::Confirm` 确认弹窗*。主 profile App 没有任何 overlay 状态机(`app.rs` 无 `overlay` 字段/分支;`Overlay::Confirm` 只存在于 `codex_auth` / `claude_auth` / `opencode_auth` 子模块),接入成本高;且 apply 可逆(切回即可,配置写入侧有 backup),「应用后停留 + 结果常驻可见」已把误操作代价降到可接受。评审 P0 的根源是「确认与退出绑在同一个键上」,拆开即解。
- *`Shift+Enter` 承载 apply+quit*(评审原始建议,规划初稿采纳过)。`runtime.rs::setup_terminal`(`runtime.rs:216-243`)只做 raw mode / alternate screen / mouse capture,没有 `PushKeyboardEnhancementFlags`;未启用 kitty 键盘协议的终端里 Shift+Enter 与 Enter 都是裸 `\r`,crossterm 无法区分。只有 Windows 原生控制台通过 `dwControlKeyState` 上报修饰符。本仓库自己就有反证:`Shift+Tab` 靠 `KeyCode::BackTab`(Unix)+ `KeyCode::Tab if SHIFT`(Windows)双路径才成立(`app.rs:731-738`),Enter 没有等价的 Unix 转义序列。降级虽安全(退化成 apply-and-stay),但会让页脚宣传一个在 Unix 上按不动的键,直接违反 WS5 的验收标准。
- *推送键盘增强标志 + 能力探测*。可行但把改动面从渲染/键位扩到终端会话生命周期与 `TerminalGuard` 的 RAII 恢复路径,成本与本任务收益不匹配;记录在 PRD Out of Scope 供未来需要修饰键位时参考。

## WS2:banner 重构 + 文案人话化

**数据来源(规划初稿此处有误,已更正)**

初稿写「绑定状态数据源已存在(`app.rs:123-150`)」——那三个函数是 `profile_source_path` / `current_profile_source_path` / `format_issue`,是**报错用的文件路径格式化**,与绑定状态无关。

真实数据源是 `CodexRuntimeSummary`(定义在 `crates/ccr-codex/src/models/codex_auth.rs:415`),经 `app.current_codex_runtime_summary()` 取得,banner 渲染在 `ui.rs:132-199`,调用点在 `ui.rs:50-80`(条件为 `current_platform() == Codex`,同时覆盖 Codex Profile 与 Codex Auth 页签)。

两个约束:

1. `CodexRuntimeSummary::profile_label()` 把「未绑定」硬编码成中文,TUI 现在靠 `localize_codex_runtime_text`(`ui.rs:1714-1723`)做 `replace("未绑定", "not bound")` 兜底。**新文案不得走这条 replace 链**,而应读 `summary.mode` / `summary.current_profile_name` / `summary.login_state` 这些类型化字段,在 TUI 侧用 `tui_text!` 直接组装。`codex_runtime_mode_label`(`ui.rs:1690-1712`)已经是这个模式的现成范例。
2. `CodexRuntimeSummary` **没有 model 字段**。「profile 名 + model」的 model 要从当前生效 profile 的 `ProfileConfig` 取:在 `app.current_profiles()` 里按 `is_current` 找到条目,再从 `profile_configs` 查 `config.model`。查不到时只显示名字,不显示空的 model 占位。

**改动点**

- `ui.rs:157` / `ui.rs:166`「Active driver: / 当前驱动:」→ 人话;`ui.rs:193` 的 block title「Control plane / 当前控制面」→ 人话。
- 未绑定态显示「未绑定 · 仅运行时认证」/ "Not bound · runtime auth only";绑定态显示 profile 名 + model。
- `ui.rs:182` 的 Auth 值 `Style::default().fg(theme::success())` → 中性 subtext。
- 图例(`ui.rs:1945`)补 `○ available`;`✓` 标记删除(`ui.rs:335` 的 `current_tag`),current 只用 `●`(`ui.rs:333`)。

## WS3:删除 Selection 面板

- 删除 `render_profile_meta_panel`(`ui.rs:567-588`)与 `profile_meta_strings`(`ui.rs:1921-1947`)。
- 同步删除/改写 `profile_list_rail_layout`(`ui.rs:227-234`,那个 `Constraint::Length(5)` 就是给这个面板留的)与 `render_profile_list_rail`(`ui.rs:477-482`);Wide 分支(`ui.rs:468-473`)之后可以直接把整个 list_area 给列表面板。
- 图例文本移入列表面板的 `title_bottom`(block 构造在 `ui.rs:534-541`)。注意 `render_profile_list_panel` 在 `profiles.is_empty()` 时会把同一个 block 交给 `render_empty_state`(`ui.rs:543-546`),需要让空态那条路径不带图例。
- `title_bottom` 渲染在边框行上,不占 inner height,所以 `sync_profile_page_size`(`ui.rs:486-491`)的分页计算不受影响。
- 图例此前只在 Wide 出现,移入列表标题后三档视口都会有——这是 PRD 认可的行为扩张,但要确认 Compact(80 列)下「图例:● 当前 · ▶ 已选择 · ○ 可用」与列表顶部标题不冲突、不溢出。

## WS4:详情空值折叠 + 布尔中性化

**函数定位(规划初稿此处有误,已更正)**

初稿写「`render_profile_details`(`ui.rs:1133-1334`)」把两个东西混成了一个:`render_profile_details` 在 `ui.rs:664-752`(共享渲染器),`ui.rs:1133-1334` 是 `codex_profile_detail_lines`(Codex 专属行构造器)。同层还有 `claude_profile_detail_lines`(`ui.rs:1336`)、`grok_profile_detail_lines`、`generic_profile_detail_lines`(`ui.rs:940`)。

**设计**

- 折叠判据必须显式。spec `backend-guidelines.md`(第 302-303 行)要求 builder 显式赋 `DetailTone`、渲染层不得从 label/value 子串推断语义;在渲染层扫 `value == "-"` 正是被禁的做法。做法:给 `DetailField`(`ui.rs` 详情表示模型)加一个 `unset: bool`,由 builder 侧设置——`opt_text`(`ui.rs:1814-1820`)、`bool_text`(`ui.rs:1822-1828`)、`tags_text`(`ui.rs:1830`)本来就知道自己在返回 `-`,`optional_tone`(`ui.rs:932-938`)也已经判过一次,把这个已知信息带出来即可。
- 折叠在四个 builder 共用的 `detail_line` / section 组装路径上实现,四平台行为一致,不做 Codex 专属分支。
- 折叠态渲染成一行 muted 摘要「N 项未设置」/ "N unset";展开状态存在 `App` 上,由一个新按键切换。
- 展开键只在全局 Keys footer 标注,详情面板内不写键位提示(spec 第 111 行:快捷键提示只存在于 footer 一处)。当前已占用键位:`q` `Esc` `h` `l` `j` `k` `o`/`O` `r` `Space` `Enter` `Tab` `PageUp/Down` `←→↑↓` `Ctrl+C` `Ctrl+L` `Ctrl+T`;`x` 可用。
- `requires_openai` 的 `Some(false) => DetailTone::Warning`(`ui.rs:1302-1305`)改 `Muted`;`Some(true)` 保持。
- **滚动无需额外处理**:`render_profile_details` 每帧都用 `lines.len()` 重算 `max_scroll` 并回写 `app.profile_detail_scroll`(`ui.rs:730-732`),折叠改变行数会自动收敛。初稿列的「折叠后 PgUp/PgDn 滚动范围重算」是多余的。

## WS5:页脚标注

- `footer_hints_for_width`(`ui.rs:2211-2255`)按新契约更新:`Enter/Space apply`,`o deactivate` / `o 解绑`,加入 WS4 的展开键。
- 两条宽度分支都要改:`width < 90` 的紧凑分支(`ui.rs:2214-2229`)和宽分支(`ui.rs:2231-2254`)。spec i18n 契约(第 175-177 行)要求紧凑页脚必须保留 `PgUp/PgDn details` 与 `Ctrl+L language` 及既有主要动作,新增标注不得把它们挤掉。
- 页脚不再出现任何 apply+quit 字样。

## WS6:文档与规范同步

- `docs/reference/commands/tui.md` 第 29 行(键位表)、第 52 行(示例注释);`docs/en/reference/commands/tui.md` 第 29、52 行同位置。改完从 `docs/` 跑 `bun run build && bun run audit`(locale parity 是硬门禁)。
- `CHANGELOG.md` Unreleased 记录 Enter 语义变化。
- Phase 3.3 的 spec 同步见「规范契约变更」。

## i18n

所有新/改文案走 `tui_text!("en", "中文")` / `tui_format!` 宏(现有模式),不新增 i18n 基础设施。注意 spec 契约:后台任务不得在 executor 线程上调用这两个宏,本任务全部改动都在渲染线程,无此风险。

## 规范契约变更(Phase 3.3 必须同步)

本任务会推翻 `.trellis/spec/ccr-tui/backend/backend-guidelines.md` 的既有表述,需在收尾时改掉,否则下一个任务会照着旧契约做:

- 第 308-310 行:「Focus is the sole name/current/enabled summary」→ Focus 还承载最近 apply 结果;「the 3-row Status strip exists only while apply/toast feedback is visible」→ strip 只承载 toast。
- 第 405 行(Grok 场景):对 Enter 语义的描述需要跟上「应用并停留」。
- 新增一条:TUI 键位不得依赖终端对裸修饰键组合(如 Shift+Enter)的上报能力;需要修饰键位时必须同时提供 Unix 转义序列路径,`Shift+Tab` 的 `BackTab` + `Tab+SHIFT` 双路径是参考实现。

## 兼容性

- Enter 行为变化是 UX breaking change:写入 `CHANGELOG.md`(Unreleased)+ 两份 docs。
- 无持久化格式、无跨 crate API 变化;`ccr-types` / `ccr-codex` 不动。

## 测试策略

**会被本任务改红的存量测试(已逐个核验,必须在对应 Step 处理)**

| 位置 | 断言 | 影响它的 WS |
|---|---|---|
| `app.rs:1682` `map_key_o_is_profile_off` | `Enter => Action::ApplyAndQuit` | WS1 |
| `ui.rs:2463` `footer_hint_mentions_reverse_tab_switching` | footer 含 `"o off"`;并引用 `profile_meta_strings` | WS5 + WS3 |
| `ui.rs:2482` `footer_omits_profile_off_on_auth_tabs` | footer 不含 `"o off"` | WS5 |
| `ui.rs:2569` `profile_list_rail_layout_keeps_full_selection_panel_visible` | Selection 面板高度 | WS3(删除) |
| `ui.rs:2722` `profile_meta_strings_show_selection_and_paging` | Selection 三行内容 | WS3(删除) |
| `ui.rs:2782` `profile_status_strip_surfaces_last_apply_feedback` | strip 显示 "Applied successfully" | WS1(改为 Focus 测试) |
| `ui.rs:2824` `profile_meta_panel_render_shows_legend_when_rail_has_extra_height` | 面板渲染出 Legend | WS3(改为列表 title_bottom 测试) |
| `ui.rs:3397` `wide_profile_draw_shows_shortcuts_only_in_global_footer` | `matches("Enter apply").count() == 1` 且含 "Applied successfully" | WS1 + WS5 |

`ui.rs:3397` 那条要特别注意:它用子串计数判断「快捷键只出现在 footer」。改成 `Enter/Space apply` 之后 `"Enter apply"` 这个子串在整屏里一次都不出现,断言会以「count == 0」的形式失败,而不是逻辑失效。更新时要连同判据一起换成新的页脚文案,并保留「只出现一次」的语义。

**新增测试**

- 键位映射单测:`Enter` 与 `Space` 都映射到 `ApplySelected`;`Action` 枚举不再有 `ApplyAndQuit`。
- Focus 反馈渲染测试:80×20、100×30、140×32 三档各一组,断言 apply 成功后出现「已切换到 X」;Wide 下该文案只出现一次(不在 strip 重复)。
- 页脚文案测试:新标注与实际键位一一对应;紧凑分支仍保留 `PgUp/PgDn details` 与 `Ctrl+L language`。
- banner 测试:未绑定/绑定两态文案;`localize_codex_runtime_text` 未被新文案调用。
- 列表图例测试:`title_bottom` 含 ●/▶/○;空态不含图例;`✓` 不出现。
- 折叠渲染测试:折叠摘要行 + 展开态各一组行级断言;四个 builder 各覆盖一次。

**验证命令**(与 spec Verification 一节对齐,`theme::ACTIVE` 是进程全局的,单线程不是可选项)

```
just fmt-check
cargo test -p ccr-tui -- --test-threads=1
just lint-strict
cd docs && bun run build && bun run audit
```

## Rollback

单 crate、无跨 crate 依赖变化;每个 WS 独立 commit,`git revert <sha>` 单点回滚。WS1 的回滚会同时恢复 `Action::ApplyAndQuit`,所以 WS1 与 WS5(页脚标注)必须相邻提交,避免中间态出现「页脚标注与键位不符」。

## 实施偏差记录(2026-08-17 实施完成后补记)

实施中与本文档不一致、以代码为准的点:

- **WS1/WS5**:`DetailTone::Error` 是新增变体(原枚举没有);失败文案统一为「切换失败 ({err})」/ "Switch failed ({err})";紧凑页脚把 `Ctrl+L` 前移到 `o`/`q` 之前——原位换文案会在 80 列中文下把 spec 强制的 `Ctrl+L 语言` 裁掉。
- **WS2**:Compact 视口 banner 只有一行内容,该行直接给 current chip(而非保留改名的 mode 行);加高 banner 会挤压 80×20 布局,否决。Auth 值文案仍复用 `summary.auth_label()` + 既有 replace 链(未新增条目),只改了颜色。
- **WS3**:图例抽成独立 helper `profile_list_legend_line()` 挂到列表面板 `title_bottom`;旧 `profile_meta_strings` 整体删除而非改造。80 列下图例不溢出,未加手动截断(沿用 ratatui 边框裁剪的既有行为)。
- **WS4**:`opt_text`/`optional_tone` 被吸收进 `DetailField::optional`(而非改返回类型),`DetailField.unset` 显式赋值;四个 builder 重构到共享的 `DetailLines` 组装器上统一折叠。Claude 的 `provider_type`/`provider` 保持既有的「未设置时整行省略」行为(改用 Option 判断),不计入折叠。`openai_login` unset 保持 `Info` tone。
- **WS6**:`backend-guidelines.md` 是混合行尾文件(CRLF/LF 混排),编辑时需字节级锚定,不能依赖行号。
- **任务外修复**:`crates/ccr/tests/commands/codex_fix.rs` 有一处预存的 clippy `format_in_format_args` 告警(仅 Windows cfg 分支),`just lint-strict` 不过;已顺手修复,建议独立 commit。
- **commit 粒度**:WS1-WS5 的代码改动交织在 `ui.rs` 同一文件内,按 WS 切 hunk 提交风险大于收益,实施侧建议代码一个 commit、docs/CHANGELOG 一个、spec 一个(见 Phase 3.4 提交计划)。
