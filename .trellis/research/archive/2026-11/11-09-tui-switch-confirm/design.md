# TUI profile 与 auth 切换确认

## Boundaries

确认只活在 TUI 按键层。写入仍走现有所有者：

- Profile apply / off：`ProfileTasks::start_mutation` → `ccr-cli` `profile_lifecycle`。`y` 之前不调用。
- Claude / Codex 账号切换：各 auth app 里现有的 `switch_selected_account`。
- Claude / Codex 退出登录：现有 `auth_off_for_platform`。`y` 之前不调用，也不调用 `needs_auth_off`。
- Grok：现有 `Modal::Confirm`，本任务不改。切换和退出登录都已经在这一层里。

`ccr-cli`、`ccr-config`、`ccr-ui` 不改。

## Why a new overlay variant

`Overlay::Confirm` 被 Claude 与 Codex 的 `handle_confirm_key` 当成删除。渲染函数又把正文标题写成「确认删除」（`overlay.rs` 约 186–217 行）。复用该变体会把「确认切换」或「确认解绑」做成「确认删除」。

新增一个变体，同时覆盖应用、切换、解绑、退出登录：

```rust
ConfirmGate {
    title: String,
    subject: String,
    message: Vec<String>,
    kind: ConfirmKind,
}
```

`ConfirmKind` 至少区分 `Apply`、`Switch`、`ProfileOff`、`AuthOff`。`y` 读 `kind`，不从标题猜动作。

- 渲染用 `theme::warning()`。正文标题用调用方传入的 title，不写删除。
- 底部提示：`y` 确认；`Enter`、`n` 或 `Esc` 取消。解绑和退出登录再写一句：再按 `o` 也是取消。
- `Overlay::Confirm` 的删除渲染和删除按键原样保留。

Grok 继续用自己的 `Modal`，不要为了统一去改 Grok。

## Profile data flow

`App` 增加一个可选的待确认记录。Apply 保存平台、profile 名、是否当前项。Off 只保存平台。不保存整份配置。同一时刻只有一条。

1. `dispatch(ApplySelected)` 不再直接 `apply_selected()`。`dispatch(ProfileOff)` 不再直接 `off_selected()`。
2. 非 Profile 页、mutation 进行中、没有选中项：保持今天的返回值和 toast。
3. 否则写入待确认记录。名字来自打开瞬间的 `selected_profile()`。
4. `handle_key` 在 `Ctrl+C` 之后、`Tab` 派发之前检查这条记录。确认打开时不进入 `map_key`。
5. `y` 清掉记录。Apply 用冻结的平台和名字调用 `start_profile_mutation(MutationRequest::Apply(...))`，不要再读光标。Off 用冻结平台调用 `MutationRequest::Off`。
6. 取消键清掉记录并 toast。Apply 的取消键含第二次 `Enter` / `Space`。Off 的取消键含第二次 `o`。另一类动作的触发键不替换当前确认。
7. `handle_mouse` 在记录存在时直接返回。
8. `ui::draw` 在 Profile 内容画完后，若记录存在，调用 `render_overlay`。
9. `footer_hints_for_width` 在记录存在时只给出确认页脚。

`apply_selected()` 保持为执行器，供现有直接调用测试和一秒钟 writer benchmark 使用。闸门在 `dispatch` / `handle_key`，不在执行器内部。这样 `enter_and_space_both_map_to_apply_selected` 仍只断言 `map_key`。

空闲页脚继续写 `Enter/Space apply`。行为变化写在对话框、文档和 changelog，避免为了一个词去改紧凑页脚的宽度预算。

## Auth data flow

Claude 与 Codex 的 `handle_normal_mode`：

1. 把「会不会真的切换」的提前返回留在打开确认之前。只读检查、当前账号判断、Codex `ensure_registry_writable` 都在打开前做完。失败则今天的 toast，不打开确认。这些检查今天已经在切换函数里，可以抽出来复用，但不得因此调用 profile off。
2. 会切换时只设置 `Overlay::ConfirmGate { kind: Switch, ... }`。
3. `o` 只设置 `Overlay::ConfirmGate { kind: AuthOff, ... }`。不调用 `auth_off_for_platform`，不调用 `needs_auth_off`。后者会读环境变量上的真实凭据路径。
4. `handle_overlay_key` 为新变体单独分发。不得落入删除用的 `handle_confirm_key`。
5. `y` 按 kind 调用现有 `switch_selected_account` 或 `auth_off_for_platform`。切换函数内部可以再做一次检查。Codex 进程数量放进切换确认正文；`y` 之后现有警告 toast 可以保留。
6. 取消只清 overlay 并 toast。不改 `should_quit`。
7. 测试要能在不碰用户主目录的情况下断言 `y` 才调用退出登录。给退出登录加一个与 `switch_selected_account_with_profile_off` 同类的注入点，默认实现仍是 `auth_off_for_platform`。

Codex 页脚今天不看 overlay。`ConfirmGate` 打开时，页脚改成 `y` / 取消，避免仍显示 `Enter switch`。删除确认的页脚不在本任务重做。

Claude 页脚已按 overlay 分支。给 `ConfirmGate` 加独立分支，不要让它掉进删除分支，也不要掉进空闲的 `Enter switch`。

## Key matrix

| 状态 | 键 | 结果 |
| --- | --- | --- |
| 空闲 Profile | Enter / Space | 打开应用确认，零写入 |
| 空闲 Profile | o | 打开解绑确认，零写入 |
| 空闲 Claude / Codex Auth，且动作会切换 | Enter | 打开切换确认，零写入 |
| 空闲 Claude / Codex Auth | o | 打开退出登录确认，零写入 |
| 确认打开 | y / Y | 执行这一个冻结动作 |
| 确认打开 | n / N / Esc / Enter / Space | 关闭，toast 取消，零写入 |
| 解绑或退出登录确认打开 | o | 关闭，toast 取消，零写入 |
| 应用确认打开 | o | 吞掉，确认还在 |
| 确认打开 | q、Tab、移动、x、r、鼠标 | 吞掉 |
| 确认打开 | Ctrl+C | 退出整个 TUI |
| 确认已关 | Esc / q | 现有退出 |

## Compatibility

- 应用后留在 TUI：保留。
- Auth 成功退出、Codex daemon 重启确认：保留，且都发生在 `y` 之后。
- 删除、保存、重命名、Grok 模态：保留。
- `apply_selected()` 与 `switch_selected_account_with_profile_off()` 的直接调用语义保留，避免拆掉只读注册表回归。
- 不读 `skip_confirmation`。TUI 确认不能被 CLI clean 的开关关掉。

## Trade-offs

- 连按两次 Enter 会取消，不会应用。连按两次 `o` 会取消，不会解绑或退出登录。对话框和文档必须写明。
- 重新应用当前 profile 也要确认。当前项的 Enter 同样会走 `apply_profile` 写文件。静默跳过会拿掉今天可用的重写路径。
- 已经解绑也仍确认。判断「已经解绑」要跑 `needs_login_prep`，而 profile spec 禁止在按键线程跑生命周期。`y` 之后的 worker 会返回未变化。
- 退出登录不先调用 `needs_auth_off`。那个函数读真实凭据路径，测试会碰到用户主目录。没有会话时，`y` 之后沿用今天的 info toast。
- 空闲页脚仍写 apply。用户靠弹出的确认框看到闸门，而不是靠页脚改名。
- 不把三套确认合成一个跨 Grok 的框架。Grok 的默认取消已经符合 R5，合并会改无关按键。

## Rollback

行为闸门都在 TUI 状态上。回滚就是去掉待确认记录和新 overlay 变体，让 `ApplySelected` 与 Auth `Enter` 重新直接调用现有函数。没有迁移，没有新配置。
