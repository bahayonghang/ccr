# TUI profile 与 auth 切换确认

## Goal

在 CCR TUI 里，下面四类写入之前先问一句：应用 profile、切换 Claude / Codex 官方账号、Profile 解绑、Claude / Codex 退出登录。默认不写入。只有按 `y` 才执行现有动作。用来挡住 Enter、Space 和 `o` 的误触。

用户价值：当前行再按一次 Enter 不会改运行时文件，也不会先关掉 profile 模式。误按 `o` 不会解绑，也不会删官方会话。取消后界面和账号保持原样。

## Background

证据在 `research/switch-paths.md`。

- Profile 三个平台的 `Enter` / `Space` 走 `Action::ApplySelected`，`apply_selected` 立刻调度 `MutationRequest::Apply`。不看 `is_current`。成功后留在 TUI。锚点：`crates/ccr-tui/src/tui/app.rs` 约 843–856、974–982、1053–1073 行。
- Claude Auth 与 Codex Auth 的 `Enter` 立刻切换。切换前会 `profile_off`。Claude 成功后退出 TUI。Codex 成功后退出；若有托管 daemon，改为重启确认。锚点：`claude_auth/app.rs` 约 223–225、439–539 行；`codex_auth/app.rs` 约 890、1348–1427 行。
- Profile `o` 直接 `MutationRequest::Off`。Claude / Codex Auth 的 `o` 直接 `auth_off_for_platform`。Grok Auth 的 `o` 已经是「默认取消，`y` 才执行」。
- `needs_auth_off()` 读的是环境变量指向的真实凭据路径，不是夹具里的 service。按键线程不能为了决定要不要弹框去调它。
- Grok Auth 的切换、删除、退出登录已经是「默认取消，`y` 才执行」。不要再套一层确认。
- `Overlay::Confirm` 的 `y` 固定删除账号，渲染正文也写死「确认删除」。切换确认必须用新变体。
- 2026-08 任务 `08-17-tui-profile-ux` 否决过主界面确认框，因为当时的误触是「应用并退出」。应用后留在 TUI 的行为保留。这次补上写入前的闸门。用户已明确要求这个闸门。

## Requirements

### R1 Profile 应用确认

- Claude、Codex、Grok 三个 Profile 页都覆盖。
- `Enter` 与 `Space` 打开确认，不调度 mutation。
- 打开时冻结平台和 profile 名。确认时用这份冻结目标调用现有 apply 生命周期，不用当时的光标再读一次。
- 目标已经是当前 profile 时仍要确认。文案说明这是重新应用，会再次写运行时文件。
- 没有可选 profile，或已有 mutation 在跑：保持今天的 toast，不打开确认。
- `y` / `Y` 之后的成功、失败、忙碌提示、留在 TUI、单 writer、退出等待，都沿用现有 mutation 路径。

### R2 Claude Auth 与 Codex Auth 切换确认

- 只有会真正调用切换的 `Enter` 才打开确认。
- 这些情况保持今天的 toast，不打开确认：没有可切换账号、Codex 虚拟当前登录、已经是当前账号、Claude 已登录且不是 API key 覆盖、Codex 注册表只读。
- 确认前不得调用 `profile_off`，不得调用 `switch_account`。
- `y` / `Y` 之后走现有切换函数。Claude 成功仍退出 TUI。Codex 成功仍退出；托管 daemon 的重启确认仍排在切换成功之后。
- 只读注册表失败时，profile 文件和运行时文件保持不动。现有夹具测试继续成立。

### R3 解绑与退出登录确认

- Claude、Codex、Grok 三个 Profile 页的 `o` 打开解绑确认，不调度 `MutationRequest::Off`。
- 打开时冻结平台。`y` 用这个平台调度一次现有 Off。mutation 进行中则保持今天的忙碌 toast，不打开确认。
- 不在按键线程调用 `needs_login_prep` 或 `profile_off_for_platform`。已经解绑时仍确认；`y` 之后由现有 worker 返回未变化。
- Claude Auth 与 Codex Auth 的 `o` 打开退出登录确认，不调用 `auth_off_for_platform`，也不调用 `needs_auth_off`。
- `y` 之后走现有 `auth_off_for_platform`。没有可删会话时，保持今天的 info toast。有变化时保持今天的 success toast 和刷新。
- Grok Auth 的 `o` 不加第二层确认。

### R4 确认框按键

- 只有 `y` / `Y` 执行当前这一个待确认动作。
- `n` / `N` / `Esc` / `Enter` / `Space` 关闭确认并 toast「已取消」，不写入。
- 打开应用确认的 `Enter` / `Space`，以及打开解绑或退出登录确认的 `o`，再按一次也是取消，不能执行。
- 应用确认打开时，`o` 不改成解绑确认。解绑确认打开时，`o` 取消；其他动作键不另开一个确认。
- 确认打开时，`q`、`Tab`、方向键、`j` / `k`、`x`、`r`、鼠标点击和滚轮都不改变选择、不换页、不写入。
- `Esc` 先关确认，不退出 TUI。确认关掉之后，再按 `Esc` 仍退出。
- `Ctrl+C` 仍立刻退出整个 TUI。此时还没有开始写入。
- 确认打开时，页脚改成 `y` 确认和取消键。空闲页脚可以继续写 `Enter/Space apply` 和 `o deactivate`。

### R5 文案、外观、文档

- 新文案走 `tui_text!` / `tui_format!`，中英都有。
- 应用、切换、解绑、退出登录都用警告色。删除确认保持错误色和现有「确认删除」正文。
- 对话框写出目标。Profile 应用写平台和「留在 TUI」。Auth 切换写「会先关闭配置模式」。Profile 解绑写「关闭配置模式，并清理 CCR 留下的登录现场」。Auth 退出登录写「移除当前官方会话」。Codex 若检测到正在运行的进程，把数量写进切换确认正文，不要在确认下面先弹一条会被挡住的 toast。
- 更新 `docs/reference/commands/tui.md` 与 `docs/en/reference/commands/tui.md`：Profile 的 Enter/Space 和 `o` 先确认；Claude / Codex Auth 的 Enter 和 `o` 先确认；Grok 段保持原意。
- `CHANGELOG.md` 的 Unreleased 记一条 TUI 行为变化。
- 把 Profile 的 Apply 与 Off 闸门补进 `.trellis/spec/ccr-tui/backend/profile-workbench.md`。Auth 确认属于各 auth app，不把账号切换或退出登录的生命周期搬进 profile spec。

### R6 Grok Auth 保持原样

- 不增加第二层确认。
- 不改 Grok 的 `y` 提交、`Enter` / `n` / `Esc` 取消、保存覆盖确认、退出登录确认。
- 现有 `grok_confirm_defaults_to_cancel_and_disabled_executor_never_becomes_busy` 继续通过。

## Acceptance Criteria

- [ ] Profile 页按 `Enter` 或 `Space` 后，mutation 未开始，确认框可见，正文含目标名，且不含「删除」。
- [ ] 当前 profile 被重新选中时，确认框说明这是重新应用。
- [ ] Profile 确认上按 `y` 后，才以打开时冻结的平台和名字调度一次 apply。按 `n`、`Esc`、第二次 `Enter`、`Space` 都不调度。
- [ ] 应用确认打开时按 `Tab`、`j` / `k`、`o` 或点击列表，选择和页签不变，也不出现 Off mutation。
- [ ] Profile 页按 `o` 后，Off mutation 未开始。`y` 才以打开时的平台调度一次 Off。`n`、`Esc`、第二次 `o` 都不调度。
- [ ] Claude Auth 与 Codex Auth 按 `o` 后不调用 `auth_off_for_platform`。`y` 才调用。取消后会话文件不变。测试夹具不读用户主目录，也不调用 `needs_auth_off()`。
- [ ] 无 profile 或 mutation 进行中时，行为与今天的 toast 一致，不出现确认框。
- [ ] Claude Auth 与 Codex Auth：会切换的 `Enter` 只打开确认；`y` 之前不调用 profile off 和 switch。取消后账号与 profile 文件不变。
- [ ] 已是当前账号、虚拟登录、Claude 已登录无需切换、Codex 只读注册表：不打开切换确认，文件不变。
- [ ] `y` 之后 Claude 成功仍退出；Codex 成功仍退出，或在有托管 daemon 时进入现有重启确认。
- [ ] 删除确认的 `y` 仍然删除，正文仍然是删除，不会触发切换。
- [ ] Grok Auth 仍是单层确认。`Enter` 不提交。`o` 不加第二层确认。
- [ ] 中英文案都走 `tui_text!` / `tui_format!`。
- [ ] 两份 TUI 文档和 Unreleased changelog 与上述按键一致。
- [ ] `cargo test -p ccr-tui -- --skip export_bindings` 通过。夹具不读不写用户主目录。

## Out of Scope

- 删除、重命名、保存。
- 页签切换、列表选择、`x` 展开未设置字段、`r` 刷新。
- 鼠标点击直接应用。今天点击只选择，保持这样。
- Grok 确认打开时仍可能被 `Tab` 换页。不在本任务修改。
- `settings.skip_confirmation`、新的「跳过 TUI 确认」设置、CLI `profile switch` / `auth switch`、`ccr-ui`、`ccr-desktop`、VS Code `confirmBeforeSwitch`。
- 改 apply / switch / profile off 的文件格式、备份、掩码或原子写。

## Constraints

- 产品改动停在 `crates/ccr-tui`、两份 TUI 文档、`CHANGELOG.md`、`profile-workbench.md`。
- 不在 `ccr-tui` 新增配置解析或数据库写。继续调用现有 `ccr-cli` 生命周期。
- 生产路径不加 `unwrap` / `expect`。
- 内部注释中文，对外文档英文或现有双语文档各自的语言。
