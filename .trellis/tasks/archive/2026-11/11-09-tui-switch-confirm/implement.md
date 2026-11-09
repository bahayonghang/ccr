# 实施清单

按这个顺序做。不要先改 `ccr-cli`。

## 1. Overlay

- 在 `crates/ccr-tui/src/tui/overlay.rs` 增加 `ConfirmGate` 和 `ConfirmKind`。Kind 覆盖 Apply、Switch、ProfileOff、AuthOff。
- 渲染走警告色。正文不得包含删除标题。
- 给删除变体补一条渲染测试：正文仍是确认删除。给切换变体补一条：含目标名，不含删除。

## 2. Profile 闸门

- `App` 保存可选的冻结目标。
- `dispatch` 的 `ApplySelected` 和 `ProfileOff` 只负责打开或拒绝，不调用 `start_profile_mutation`。
- `handle_key` 在 Tab 之前吃掉确认键。`handle_mouse` 同样短路。
- `y` 按 kind 调用现有 Apply 或 Off。Apply 用冻结名字。Off 用冻结平台。
- `ui::draw` 末尾画 overlay。`footer_hints_for_width` 在确认打开时换页脚。
- 不要改 `apply_selected()` 的函数体，除非只是让 `y` 复用它且不会重新读取光标。若复用会读光标，就新增一个按名字执行的私有函数，两个入口共用。

## 3. Claude Auth 与 Codex Auth

- `Enter` 的可切换分支改为设置 `ConfirmGate { kind: Switch }`。
- `o` 改为设置 `ConfirmGate { kind: AuthOff }`。此分支不调用 `auth_off_for_platform` 或 `needs_auth_off`。
- 提前返回的 toast 分支保持原顺序，并且不设置 overlay。
- overlay 分发增加新变体。删除用的 `handle_confirm_key` 继续只处理 `Overlay::Confirm`。
- `y` 按 kind 调用现有切换函数或退出登录函数。
- 退出登录默认仍是 `auth_off_for_platform`。测试注入一个替代函数，避免 `y` 的测试读用户主目录。
- Claude 与 Codex 的确认页脚单独分支。
- Codex 进程数量放进打开确认时的 message。

## 4. 测试

用现有夹具和 `ProfileTasks::with_backends`。不要碰用户主目录。

Profile：

- Enter / Space 后 `mutation_active()` 为假，待确认记录含打开时的名字。
- 把光标移开的测试不要发生在确认打开之后；确认打开时 `j` 不能改选择。`y` 调度的名字等于打开时的名字。
- 当前项的文案路径：冻结记录标记 `is_current`。
- n、Esc、第二次 Enter、Space：writer 计数为 0。
- `o` 后 Off writer 计数为 0。`y` 后恰好一次，平台等于打开时的平台。第二次 `o` 不增加计数。
- 应用确认打开时按 `o`：确认仍是 Apply，Off 计数为 0。
- Tab、鼠标：页签和选择不变。
- 空列表与 mutation 进行中：无待确认记录。

Auth：

- Claude / Codex 用假 profile off 计数。Enter 后计数仍为 0，且未调用 switch。
- `y` 后才增加计数。取消后运行时文件与 profile 文件字节不变。
- 当前账号、虚拟账号、只读注册表不产生 Switch kind。只读测试可以继续直接调用 `switch_selected_account_with_profile_off`。
- `o` 产生 AuthOff kind，且注入的退出登录函数计数为 0。`y` 后计数为 1。取消后计数仍为 0。
- 删除确认的 `y` 仍走删除服务，不走 switch。

Grok：

- 不改测试意图。跑现有默认取消测试即可。

渲染：

- 用 `TestBackend` 看切换确认的警告色和目标名。删除确认快照不被换成警告色。
- 中英各断言一条标题。测完恢复 `active_language()`。

## 5. 文档与 spec

- `docs/reference/commands/tui.md` 与英文页：Profile Enter/Space 和 `o` 先确认，`y` 才执行并留在 TUI；取消键含第二次 Enter 或第二次 `o`。Claude / Codex Auth 的 Enter 和 `o` 同样先确认。Grok 段不改语义。
- `CHANGELOG.md` Unreleased 增加一条。
- `profile-workbench.md` 补四条：Apply 键和 Off 键先确认；mutation 只在 `y` 之后；Apply 目标是打开时的平台和名字；确认打开时重复按键不得再启动一个 writer。Auth 账号切换和退出登录不要写进这份 profile spec 的执行所有者。

## 6. 验证

```text
cargo test -p ccr-tui -- --skip export_bindings
```

默认并行。不要为了这个任务跑 `just ci`。改完 Rust 后再跑 `cargo fmt -p ccr-tui` 与该包 clippy，保持干净。

文档只改了两页 Markdown。不需要 VitePress 全量构建，除非改动超出这两页。

## 风险与回滚

- 风险文件：`overlay.rs`、`app.rs`、`ui.rs`、`claude_auth/app.rs`、`claude_auth/ui.rs`、`codex_auth/app.rs`、`codex_auth/ui.rs`。
- 最容易犯的错：新变体落到 `Overlay::Confirm` 的删除分支。测试要锁住「切换确认的 y 不删除」。
- 第二个容易犯的错：`y` 重新读取光标。测试要锁住冻结名字。
- 回滚：恢复上述文件里的确认状态，让 Enter 重新直接调用现有函数。无数据迁移。

## 开工前

- 用户批准本任务的规划摘要之后，才能 `task.py start`。
- 开工前再读 `.trellis/spec/ccr-tui/backend/index.md` 指向的两份 spec，以及 `research/switch-paths.md`。
