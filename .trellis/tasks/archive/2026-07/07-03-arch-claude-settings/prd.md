# 合并 ClaudeSettings

## Goal

同一 `~/.claude/settings.json` 有两个 Rust shape：ccr-types 富类型（只读，供 UI）、ccr-cli 贫瘠 `{env, flatten other}`（持有全部变更逻辑）。合为 ccr-types 一个深类型（parse/merge/managed-var 变更/未知字段保留），CLI 与 UI 成为该类型的两个 adapter。审查候选 5（Worth exploring）。

## Requirements

### 现状（探索报告定位）

- `crates/ccr-types/src/claude_settings.rs:116`：完整类型（env、outputStyle、permissions、mcpServers、slashCommands、agents、plugins、hooks），为 UI 后端而建，只读。
- `crates/ccr-cli/src/managers/settings.rs:65`：极简 `{ env, #[serde(flatten)] other }`，拥有全部变更逻辑（`clear_managed_vars`、`update_from_config`）。
- 风险：变更逻辑运行在看不见字段结构的 shape 上，UI 已知字段可能被 CLI 写路径静默丢弃或搅乱；同文件两 shape 跨 CLI/UI seam 漂移。

### 要做的

1. 以 ccr-types 的富类型为基础，吸收 SettingsManager 的变更逻辑（clear_managed_vars、update_from_config），成为唯一 `ClaudeSettings`：负责 parse、merge、managed-var 变更、未知字段无损保留（flatten 兜底仍要保留——富类型未覆盖的字段不能丢）。
2. `ccr-cli SettingsManager` 与 Tauri settings 命令改为该类型的两个 adapter：只做 IO 与调用，不再各持 shape。
3. 补齐"读→改→写→再读"往返保留测试：包括富类型字段、未知字段、注释外字段顺序可容忍的规范化行为。

### 约束

- settings.json 磁盘格式不变；未知字段绝不丢失（这是本任务的核心回归风险）。
- ccr-types 是 leaf crate，不得因此引入对上层 crate 的依赖；变更逻辑必须以纯数据操作形式进入。
- 写盘路径继续走既有原子写（若 07-03-arch-guarded-write 已完成则走 guarded write）。
- 遵守 `public-api-boundary.md`：若 prelude/根 re-export 面变化，走快照有意更新流程。

## Acceptance Criteria

- [ ] 全仓 `ClaudeSettings` 仅一处定义（`rg 'struct ClaudeSettings'` 唯一命中 ccr-types）。
- [ ] `clear_managed_vars`、`update_from_config` 逻辑归属该类型并有单元测试。
- [ ] 往返保留测试：含 mcpServers/hooks/plugins 等富字段与人为注入的未知字段，读改写读后无损。
- [ ] CLI 侧（settings 相关命令集成测试）与 UI 侧（settings 相关 Tauri 命令测试）行为不回归。
- [ ] `just lint-strict`、`just test`、`cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml settings` 通过。
- [ ] 相关 spec 更新（trellis-update-spec）。

## Notes

- 复杂任务：`task.py start` 前需补 design.md（变更 API 形状、未知字段保留策略、adapter 切分）与 implement.md。
- 依赖：无硬依赖；与 guarded-write 软衔接（写盘入口）。
