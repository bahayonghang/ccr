# TUI 切换路径证据

日期：2026-11-09。只读检查，没有改产品代码。

## 用户看到的界面

截图是 Claude Profile 页。`anyrouter3` 同时是当前项（`●` / Current）和选中项（`▶` / now）。页脚是 `Enter/Space apply`。这一下会走应用，不是翻页。

页签只有六个：Claude / Codex / Grok 的 Profile，以及三者的 Auth。`Tab` / `Shift+Tab` 只换页签，不写运行时。

## 现在按键做什么

| 表面 | 触发 | 确认 | 写入前会先做什么 | 成功后 |
| --- | --- | --- | --- | --- |
| Claude / Codex / Grok Profile | `Enter` 或 `Space` → `Action::ApplySelected` | 无 | `apply_selected` 直接 `MutationRequest::Apply` | 留在 TUI。写入在后台 worker |
| Profile `o` | `Action::ProfileOff` | 无 | `MutationRequest::Off` | 留在 TUI |
| Claude Auth | `Enter` | 无 | 已是当前账号则 toast 返回；否则先 `profile_off_for_platform`，再 `switch_account` | 成功则 `should_quit = true` |
| Codex Auth | `Enter` | 无 | 虚拟账号 / 已是当前账号则返回；只读注册表则返回且不调用 profile off；否则先 profile off 再 `switch_account` | 成功则退出；若有托管 daemon，改为 `ConfirmRestartDaemon`，此时还不退出 |
| Grok Auth | `Enter` | 已有 `Modal::Confirm` | `y` 才 `submit`。`Enter` / `n` / `Esc` 丢掉模态且不提交 | 文档已写「默认取消，按 y 才确认」 |
| Claude / Codex Auth `d` | 删除 | `Overlay::Confirm` | `y` 才删除 | 留在页内 |
| 鼠标左键点 Profile 行 | `SelectAt` | 无 | 只改选中 | 不应用 |

锚点：

- Profile 映射与派发：`crates/ccr-tui/src/tui/app.rs` `map_key`（约 843–856 行）、`dispatch` 的 `ApplySelected` / `ProfileOff`（约 974–982 行）、`apply_selected`（约 1053–1073 行）、`off_selected`（约 1075–1078 行）。
- 鼠标只选择：同文件 `handle_mouse`（约 1455–1461 行）。
- Claude Auth 立即切换：`crates/ccr-tui/src/tui/claude_auth/app.rs` `handle_normal_mode` 的 `KeyCode::Enter`（约 223–225 行）、`switch_selected_account`（约 439–539 行）。`y` 在 `handle_confirm_key` 里固定走删除（约 292–329 行）。
- Codex Auth 立即切换：`crates/ccr-tui/src/tui/codex_auth/app.rs` `KeyCode::Enter`（约 890 行）、`switch_selected_account_with_profile_off`（约 1348–1427 行）。只读注册表测试直接调用该函数，且断言 profile off 未被调用（约 1844–1877 行）。
- Grok 已确认：`crates/ccr-tui/src/tui/grok_auth/app.rs` `handle_key`（约 349–363 行）和 `modal_key`（约 261–265 行）。回归：`grok_auth/tests.rs` `grok_confirm_defaults_to_cancel_and_disabled_executor_never_becomes_busy`。
- 删除弹窗渲染把正文标题写死为「确认删除」：`crates/ccr-tui/src/tui/overlay.rs` `Overlay::Confirm` 分支（约 186–217 行）。不能把切换确认塞进这个变体，否则 `y` 会删账号，正文也会写成删除。
- Profile 应用会重写运行时，当前项也不短路：`crates/ccr-cli/src/application/profile_lifecycle.rs` `apply_profile`（约 591–612 行）最终调用 `instance.apply_profile`。TUI 的 `apply_selected` 不看 `is_current`。
- 页脚文案：`crates/ccr-tui/src/tui/ui.rs` `footer_hints_for_width`（约 2064–2112 行）。Claude Auth 弹窗会换页脚（`claude_auth/ui.rs` 约 638–669 行）。Codex Auth 页脚不看 overlay（`codex_auth/ui.rs` `draw_footer_strip` 约 2086 行）。
- 用户文档仍写「Enter/Space 直接应用」：`docs/reference/commands/tui.md` 第 29 行，英文页对应行。Grok 段已经是确认模型（第 32 行）。

## 2026-08 否决为什么不再沿用

`.trellis/tasks/archive/2026-08/08-17-tui-profile-ux/design.md` 否决了主 Profile App 接入 `Overlay::Confirm`。当时的理由：主 App 没有 overlay 状态机；apply 可切回；误触的根因是「应用并退出」绑在同一个键上。那次修复让应用后留在 TUI，这个行为要保留。

那次修复没有挡住写入。截图里的当前行再按一次 Enter，仍会跑完整 apply。用户这次明确要求切换前加确认框。

## 2026-11-09 范围补充

用户确认把 `o` 加进同一闸门。

- Profile `o`：`app.rs` `off_selected`（约 1075–1078 行）直接 `MutationRequest::Off`。已经解绑的判断在 `profile_off.rs` 的 worker 里，按键线程不能先跑 `needs_login_prep`。
- Claude / Codex `o`：各自 `handle_normal_mode` 直接 `auth_off_for_platform`。`needs_auth_off()`（`auth_off.rs` 约 255–261、439–445、502–506 行）读环境变量上的真实凭据路径，不能拿来决定弹框。
- Grok `o` 已经打开 `Modal::Confirm(GrokLogout)`。保持一层。

## 不在本任务里改的相邻事实

- `Tab` 在 `App::handle_key` 里先于 Grok 模态派发。Grok 确认打开时仍可能被 Tab 换走。本任务不改这条，避免扩大 Grok 行为。
- `settings.skip_confirmation` 只服务 CLI clean/clear。TUI 当前不读它。
- `ccr-ui` 的确认契约在 `.trellis/spec/ccr-ui/frontend/confirm-interaction-contracts.md`。桌面切换确认不在本任务。
- `ccr-vscode` 有 `ccr.confirmBeforeSwitch`。不在本任务。
