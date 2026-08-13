# UI / TUI 清理入口现状

调研日期：2026-08-13。

## Grok Profiles（已有 Off）

`ccr-ui/src/views/grok/GrokProfilesView.vue`

- 位置：`ProfilesHeader` 下方、StatStrip 上方。
- 条件：`activation !== 'inactive'` 显示横幅；`activation === 'active' | 'drifted'` 显示 Off 按钮。
- 确认：`openConfirmDialog`，`type: 'warning'`。
- 命令面板：`__off` → `handleOff`。
- Tauri：`grok_profile_off`。

## Claude / Codex Profiles

骨架（契约）：Header → StatStrip → QuickRail → Toolbar → 列表 → Inspector。

- 无 Off 按钮、无激活横幅。
- Header 溢出菜单只有 Reload / Export / Edit TOML。`ProfilesHeader.vue:85-126`
- Tauri 只有 `claude_apply_profile` / `codex_apply_profile`，无 off 命令。

## Auth 页

- Claude Auth 副标题已写明切换会清 CCR 托管设置。`ClaudeAuthView.vue:15-16`
- Claude Auth 有诊断面板与 `remaining_suppressors`，无独立 Off。
- Codex Auth 头部是返回 / 刷新 / 保存当前登录，无 Off。`CodexAuthView.vue:32-80`
- 平台 Home（`ClaudeCodeView` / `CodexView` / `GrokView`）无 Off，只有跳到 Profiles 的链接。

## TUI

- Profile 页 footer：`Tab`、`Enter` apply、`r` reload、`q`。无 Off。`tui/ui.rs:2211-2238`
- `App::apply_selected` 只调 `apply_profile`。`tui/app.rs:921`
- Claude Auth `switch_account` 已清托管设置并 toast 清理数量。`claude_auth/app.rs:454-477`
- Codex Auth `switch_account` 不调 `profile off`。`codex_auth/app.rs:1307`

## 放置备选

1. Grok 同构横幅（Header 与 StatStrip 之间）+ 命令面板 + Auth 诊断区按钮。
2. 仅 Profiles 横幅，Auth 只给跳转文案。
3. Header 与「添加」并列的常驻次主按钮。
