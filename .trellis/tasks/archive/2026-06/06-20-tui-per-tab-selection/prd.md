# 修复 TUI 切换 tab 光标不定位已启用项：选中状态改为 per-tab

## Goal

切换到任意 profile tab 时（尤其首次进入 Claude Code tab），光标高亮应定位到该 tab 当前**已启用**（`is_current`）的 profile；并且每个 profile tab 独立记住各自的浏览/选中位置，tab 之间互不干扰（per-tab）。

## Background

- 现状：`selected_index`、`current_page`、`selected_profile_name` 是 `App` 级**全局字段**，被所有 tab 共享（`crates/ccr-tui/src/tui/app.rs:142/144/148`）。
- `sync_selection_to_profile_name()`（app.rs:304）仅对 `Platform::Codex` 用 `current_profile_global_index()`（`is_current`）优先定位；Claude 及其它平台走 `selected_profile_name`(按名) → 残留 `selected_index` 的 fallback，**不**看 `is_current`。
- 切 tab 流程（app.rs:585 起）先 `remember_selected_profile()` 把旧 tab 选中写入全局字段，再 `sync_*`。后果：切到 Claude Code tab 时光标继承上一个 tab（Codex）的名字/索引，落在错误项（如 `husan`）而非已启用的 `anyrouter3`。

## Requirements

1. 选中状态 per-tab：每个 profile tab 独立保存其 `selected_index`、`current_page`、`selected_profile_name`。
2. 首次进入某 tab（无历史选中）时，光标定位到该 tab 的 `is_current` profile；若该 tab 无已启用项，则定位第 0 项。
3. 再次进入此前访问过的 tab 时，恢复上次离开时的选中位置（含页码）。
4. 不回归现有交互：上下移动（j/k）、翻页（h/l）、reload（r）、apply（空格切换并刷新）、`page_size` 自适应、name 同步、鼠标命中。
5. `page_size` 保持全局（由终端可见行数驱动，所有 tab 同高）。
6. Auth tab（Claude/Codex/OpenCode Auth）为独立 sub-app，不受影响。
7. 遵守 ccr-tui Startup Contract：`active_tab = 0`，由配置 tab order 决定首个可见 tab。

## Acceptance Criteria

- [ ] 打开 TUI，切到 Claude Code tab，光标高亮在已启用 profile（`is_current`）上。
- [ ] 在 Claude Code tab 移动到非启用项后，切到 Codex tab 再切回，光标回到上次离开的项与页码（per-tab 记忆）。
- [ ] Codex tab 同样满足：首次进入定位 `is_current`，切走切回保留位置。
- [ ] 两个 profile tab 的选中互不串扰。
- [ ] 单测覆盖：首次进入定位 `is_current`、切走切回恢复快照、跨 tab 互不干扰。
- [ ] `just fmt-check`、`cargo test -p ccr-tui -- --test-threads=1`、`just lint-strict` 全部通过。

## Out of Scope

- 不改动 Auth sub-app 的选择逻辑。
- 不调整 tab 顺序、tab 渲染或分页算法本身。
- 不持久化 per-tab 选中到磁盘（仅进程内会话级记忆）。
