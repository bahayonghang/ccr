# Codex Auth 周限展示优化

## Goal

把 Codex Auth TUI 里当前的周限展示放到 `7d` 语义上，避免继续把现行 weekly quota 写成泛化的 `Weekly limit`，同时保留 `5h` 相关数据路径，方便未来 5h 限制恢复时直接复用。

## Confirmed Facts

- OpenAI 官方 Codex 文档仍同时提到 5-hour window 和 weekly window；银行式 reset 也会同时重置两者。
- `crates/ccr-codex/src/services/openai_quota_core.rs` 仍会填充 `hourly_window_present` 和 `weekly_window_present`。
- `CodexQuota` 仍保留 `hourly_percentage` / `weekly_percentage` 两套字段。
- `crates/ccr-tui/src/tui/codex_auth/ui.rs` 里，账号 snapshot 已经用 `5h:` / `7d:`，但 `Usage & Quota` 面板仍写 `5h limit` 与 `Weekly limit`。

## In Scope

- 只改 `crates/ccr-tui/src/tui/codex_auth/ui.rs` 里 Codex Auth 的 quota 文案和渲染逻辑。
- 让 weekly quota 在当前 UI 里使用 `7d` 语义，与账号列表 / snapshot 的现有写法对齐。
- 保留 5h 相关字段和渲染路径，不把它从数据模型或服务层删掉。

## Out of Scope

- 不改 `CodexQuota` 数据结构。
- 不改 `ccr-codex` 的 quota 解析 / 拉取逻辑。
- 不改 OpenCode 或 `ccr-ui` 的同类 surface。

## Acceptance Criteria

- Codex Auth TUI 的 `Usage & Quota` 面板不再用泛化的 `Weekly limit` 描述当前周限，而是以 `7d` 语义呈现。
- 账号 snapshot / 列表仍能正常显示现有的 `5h` 与 `7d` 值，且周限仍对应 weekly 数据。
- 相关测试能覆盖更新后的文案，并保持现有 quota 计算逻辑不变。

## Risks / Deferred

- 上游 Codex 文档现在仍写着 5-hour + weekly 两个窗口，所以这次只做本地展示优化，不把它当成协议变化。
- 如果以后 5h 窗口重新变成当前有效窗口，现有保留的数据路径应能直接复用。

## Status

planning
