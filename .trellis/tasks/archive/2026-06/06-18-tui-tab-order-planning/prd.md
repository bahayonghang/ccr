# 优化 TUI Tab 排序规划

## Goal

分析并规划 `ccr` 主 TUI 的 tab 顺序优化，使默认顺序符合当前使用趋势：`Codex Profile`、`Claude Code Profile`、`Codex Auth`、`Claude Auth`、`OpenCode Auth`，并把顺序做成 `~/.ccr/` 下的正式 TOML 配置。

## Confirmed Facts

- 当前 tab 顺序在 [crates/ccr-tui/src/tui/app.rs](D:/Documents/Code/Github/ccr/crates/ccr-tui/src/tui/app.rs) 中硬编码构建。
- `ui.rs` 只是按 `app.tabs` 的当前顺序渲染，不负责排序。
- 现有 tab 相关入口已经支持预选中：`with_claude_auth_tab`、`with_codex_tab`、`with_opencode_auth_tab`。
- 仓库已有稳定的 `~/.ccr` 用户目录约定，很多配置和注册表文件都落在这个根目录下。
- `ccr-sync` 已在 `~/.ccr/` 根目录下使用独立文件名 `sync.toml`、`sync_folders.toml`，说明“根目录单功能 TOML”是现成模式。
- `ccr-config` 已统一处理 `CCR_ROOT`/`~/.ccr` 路径解析，因此新的 TUI 偏好配置也应复用这套根目录解析，而不是在 TUI 层自己拼 home path。

## Requirements

- 默认 tab 顺序必须按用户给定优先级排列。
- 排序调整不能破坏现有的键盘切换、鼠标命中测试和预选中入口。
- 必须支持 `~/.ccr/` 下的正式 TOML 配置。
- 配置缺失、缺项、重复项或非法值时必须回退到内置默认顺序，不能阻塞 TUI 启动。
- TOML 读取应归属配置层，不应在 TUI 层直接散读文件。
- 配置文件应为可扩展的 TUI 偏好文件，而不是只为 tab 顺序做一次性临时文件。

## Acceptance Criteria

- [ ] 明确 `~/.ccr/tui.toml` 的配置契约、默认值和回退规则。
- [ ] 说明配置化方案的实现成本、维护成本和回滚成本。
- [ ] 指出最小影响文件范围。
- [ ] 给出后续实现时需要更新的测试点。

## Open Questions

- 配置项是否只接受完整顺序列表，还是允许部分覆盖。
- 是否要为未来其他 TUI 偏好预留同文件扩展字段。

## Out Of Scope

- 本次不改 TUI 视觉样式。
- 本次不扩展新的 tab 类型。
- 本次不实现除 `tab_order` 之外的其他 TUI 偏好项。
