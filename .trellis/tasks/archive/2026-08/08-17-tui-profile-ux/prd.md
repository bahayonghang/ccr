# TUI Profile 界面设计优化(操作安全闭环+信息层级+界面减负)

## Goal

修复 Impeccable 设计评审(2026-08-17,健康度 23/40)在 Codex Profile 主界面发现的 5 个优先问题,按「操作安全闭环 > 当前状态可见性 > 界面减负」排序交付。评审快照:`.impeccable/critique/2026-08-17T09-37-05Z__crates-ccr-tui-src-tui.md`(副本在本任务 `research/critique-snapshot.md`)。

用户已确认的决策:优先操作安全闭环;文案人话化(中英双语);范围为全部 5 项(P0×1、P1×2、P2×2);允许调整键位语义。

## Requirements

### WS1(P0)切换安全闭环

- Enter 应用 profile 后 **TUI 不再退出**,用户必须能看到切换结果反馈。
- **不再提供任何单键「应用并退出」**。想退出的用户按 Enter 后再按 `q` / `Esc`。
  - 理由见「评审修订记录」D1:`Shift+Enter` 在未启用 kitty 键盘协议的 Unix 终端上与裸 Enter 无法区分,只有 Windows 原生控制台能报告修饰符,做成快捷键会在半数平台上是死键。
  - `Space` 保留为 Enter 的同义键位(照顾既有肌肉记忆)。
- 切换成功与失败都必须在界面上可见,且**三档视口(Compact / Standard / Wide)都可见**。
  - 常驻反馈落在 **Focus 面板**(三档视口都渲染),文案为「已切换到 X」/ "Switched to X";失败沿用现有 error toast + 失败态反馈行。
  - Wide 专属的 Status strip 退化为纯 toast 通道,不再重复 apply 结果(避免 Focus 与 strip 同屏重复)。

### WS2(P1)当前生效 profile 可见性 + 文案人话化

**适用范围:Codex 页签**(顶部 runtime banner 只在 `platform == Codex` 时渲染,同时覆盖 Codex Profile 与 Codex Auth 两个页签)。其他平台页签没有这条 banner,本 WS 不为它们新增。

- banner 改为常驻 current chip:绑定中 = profile 名 + model;未绑定 = 大白话「未绑定 · 仅运行时认证」/ "Not bound · runtime auth only"。
- 列表图例补全所有标记(● current、▶ selected、○ available);删除与 ● 冗余的 ✓ 标记。
- 「Control plane」「Active driver: Runtime/Auth only」等内部术语改为人话(中英双语)。
- banner 中 Auth 状态不再占用 success 绿(中性状态用中性色)。

### WS3(P1)删除 Selection 冗余面板

**适用范围:Wide 视口**(Selection 面板只在 Wide 分支经 `render_profile_list_rail` 渲染;Standard / Compact 本来就没有它)。

- 整块删除 Selection 面板(`Selected`/`Profiles`/`Legend` 三行,与 Focus 面板和列表标题完全重复)。
- 图例并入列表面板 `title_bottom`;释放的高度让给列表/详情。
- 图例移入列表标题后会在三档视口都出现(现状只在 Wide 可见),这是预期内的行为扩张;空态 / 错误态不得带出图例。

### WS4(P2)详情面板空值折叠 + 布尔中性化

- Routing/Auth 等 section 中未设置的字段默认折叠为一行 muted 摘要(如「5 项未设置」),提供按键展开查看。
- **折叠判据必须是显式的 unset 标记,不得在渲染层比较字符串 `"-"`**(spec `backend-guidelines.md` 要求 builder 显式赋语义,渲染层不得从 label/value 子串推断含义)。
- 折叠能力对四个详情构造器(codex / claude / grok / generic)一致生效,不做 Codex 专属分支。
- 展开键的发现路径:只在全局 Keys footer 标注,**不得**在详情面板内写「按 x 展开」之类的面板内键位提示(spec 明令快捷键提示只存在于 footer 一处)。
- `requires_openai` 等布尔字段的 false 分支改用 neutral muted;warning 黄只留给真正异常(如 token missing)。

### WS5(P2)页脚标注与实际语义一致

- 页脚键位标注与 WS1 后的真实行为一一对应(`Enter/Space apply`;不再出现任何 apply+quit 标注)。
- 「o off」改为明确语义(「o deactivate」/「o 解绑」)。
- WS4 的展开键需要在 footer 有一席之地;宽度不足时按现有自适应逻辑降级,降级掉的标注不得留下错误描述。

### WS6 文档与规范同步

- 更新 `docs/reference/commands/tui.md` 与 `docs/en/reference/commands/tui.md` 的键位表与示例注释(各 2 处写着「Enter 应用并退出」)。
- `CHANGELOG.md` Unreleased 记录 Enter 键位行为变化(UX breaking change)。
- 同步 `.trellis/spec/ccr-tui/backend/backend-guidelines.md` 中被本任务推翻的契约(详见 Notes)。

## Constraints

- 代码改动限于 `crates/ccr-tui`;不改 `ccr-cli` / `ccr-codex` / `ccr-config` 的 apply(写配置)逻辑与 runtime summary 数据结构。
  - WS2 的 banner 文案必须由 `CodexRuntimeSummary` 的**类型化字段**(`mode` / `current_profile_name` / `login_state` 等)在 TUI 侧用 `tui_text!` 组装;**不得**继续扩充 `ui.rs::localize_codex_runtime_text` 的中文字符串 replace 链。
- 文档(`docs/**`)、`CHANGELOG.md`、`.trellis/spec/**` 在本任务范围内,是 WS6 的交付物。
- 所有新文案走 `tui_text!` / `tui_format!` 中英双语,不得硬编码单语言。
- 不得回归既有能力:CJK 宽度感知截断、三档 viewport 布局、双主题(Mocha/Latte)token 契约、平台身份色、per-tab selection 与分页助手红线。
- 键位设计不得依赖终端对裸修饰键组合的上报能力(Enter/Space/字母键之外的修饰组合需要先确认编码可行性)。
- Rust 代码保持 `cargo fmt` / clippy 干净,生产路径不新增 `unwrap`/`expect`。
- 配置写入侧的 backup/masking/atomic-write 行为不得受影响(本任务预期不触碰,但不得回归)。

## Out of Scope(评审提及但本任务不做)

- 列表搜索/过滤/按使用频率排序(评审"值得思考的问题"1,另议)。
- Latte 亮色主题对比度系统性偏低(另立任务)。
- `$936` provider 级成本数字的展示取舍(留待讨论)。
- 主 App 接入完整 overlay 确认弹窗体系(设计文档记录了否决理由)。
- 为 Claude / Grok / OpenCode 页签新增 runtime banner(WS2 只覆盖已有的 Codex banner)。
- 在 `runtime.rs` 引入 kitty 键盘协议增强(D1 已否决,记录在案供未来需要修饰键位时参考)。

## Acceptance Criteria

- [ ] 按 Enter 应用 profile 后 TUI 保持运行;Focus 面板出现「已切换到 X」类反馈,并在 80×20、100×30、140×32 三种尺寸下都有 `TestBackend` 渲染断言锁定。
- [ ] 不存在任何单键 apply+quit 路径;`Action::ApplyAndQuit` 已从 `action.rs` 移除,`just lint-strict` 无 dead_code 告警。
- [ ] Wide 视口下 apply 结果只出现一次(在 Focus,不在 Status strip)。
- [ ] 页脚每个标注与实际键位行为一一对应,且有测试锁定;页脚不再出现任何 apply+quit 字样。
- [ ] Codex 页签未绑定状态下 banner 显示人话文案;绑定状态下显示 profile 名 + model;「Control plane」「Active driver」字样不再出现;`localize_codex_runtime_text` 的 replace 链没有新增条目。
- [ ] 图例覆盖列表全部标记(●/▶/○);✓ 标记不再出现;空态/错误态不带图例。
- [ ] Wide 视口 Selection 面板不再渲染;图例出现在列表面板底部标题;三档视口布局无溢出。
- [ ] Routing/Auth 未设置字段默认折叠为一行摘要且可展开;折叠判据是显式 unset 标记而非字符串比较;四个详情构造器行为一致;`requires_openai=false` 不再渲染为黄色。
- [ ] `docs/reference/commands/tui.md` 与英文镜像键位表/示例已更新;`cd docs && bun run build && bun run audit` 通过。
- [ ] `CHANGELOG.md` Unreleased 记录键位行为变化;`.trellis/spec/ccr-tui/backend/backend-guidelines.md` 的 Focus / Status strip 契约已同步。
- [ ] `cargo test -p ccr-tui -- --test-threads=1` 全绿(含新增/更新的键位与渲染测试);`just fmt-check`、`just lint-strict` 通过。
- [ ] 手动验证:宽/窄两种终端宽度下实际运行 TUI,截图核对布局、文案、折叠行为。

## 评审修订记录(2026-08-17,`task.py start` 前)

规划评审对照真实代码逐条核验后,以下三点被修订:

- **D1 放弃 Shift+Enter**。`runtime.rs::setup_terminal` 未推送 `PushKeyboardEnhancementFlags`,Unix 终端下 Shift+Enter 与 Enter 同为裸 `\r`,crossterm 拿不到修饰符;仅 Windows 原生控制台可用。同文件 `app.rs:731-738` 的 `Shift+Tab` 之所以能工作,是因为它同时处理了 `BackTab`(Unix)与 `Tab+SHIFT`(Windows)两条编码路径,而 Enter 没有等价的 Unix 编码。改为:Enter 应用并停留,退出走 `q`/`Esc`。
- **D2 反馈落到 Focus 而非 Status strip**。Status strip 只在 `ViewportMode::Wide`(≥120 列且 ≥22 行)渲染,100×30 这类常见尺寸下 P0 的常驻反馈根本不存在。改为由三档视口都渲染的 Focus 面板承载。这与 archive 里 `07-06-tui-profile-page-polish` 的原始 AC(「Focus 块:Name/Status/最近 apply 结果保留」)一致,当时实现把它挪去了 strip。
- **D3 文档纳入范围**。`docs/reference/commands/tui.md:29,52` 与英文镜像明写「Enter 应用并退出」,不改就是错的;docs 有 locale parity 审计门禁。

## Notes

- 复杂任务:实施前需 `design.md` + `implement.md`,评审后 `task.py start`。
- 每个 WS 一个独立 commit,便于单点回滚。
- WS3 推翻了 `07-06-tui-profile-page-polish` R2「保留 Selection 面板的 Selected/Profiles/Legend」的决定。核验结论是这次删除成立:Focus 面板的 `Name` 行已显示选中 profile 名,列表标题已显示计数与页码,该面板确为纯冗余。
- Phase 3.3 需要同步的 spec 条目:`backend-guidelines.md` 第 308-310 行(Focus 是唯一 name/current/enabled 摘要 + Status strip 仅 Wide 可见)、第 405 行(Grok 场景里对 Enter 语义的描述),以及新增一条「TUI 键位不得依赖终端对裸修饰键组合的上报」的约束。
