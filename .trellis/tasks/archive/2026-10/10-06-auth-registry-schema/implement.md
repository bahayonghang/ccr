# 实施计划：注册表版本门与未知字段保留

## 前置

- 实施前复核 prd.md 与 design.md 中的 file:line 锚点（基线 dev @ 73b47563）。
- 本任务不依赖其他 P 任务；P1（10-06-auth-identity-key）在本任务独立验收通过后启动。提交和归档等待单独授权。

## 步骤（按序）

1. **往返探针**：在 `codex_registry_store.rs` 测试中先写 AC1 往返测试（顶层未知键、顶层未知表、账号级未知键），确认 toml 1.x flatten 序列化的输出可再次解析。若失败，停止并回到规划（改用手工 `toml::Table` 合并方案），不要绕过。
2. **R1 模型**：`CodexAuthRegistry`、`CodexAuthAccount` 增加 `#[serde(flatten)] pub extra: toml::Table`；更新 `Default` 与全部结构字面量（生产 `codex_auth_service.rs:931`、`:2004`；测试构造点见 prd.md）。补 `save_current --force` 移除旧未知字段的测试。
3. **R2/R3 判定与文本**：在 `codex_registry_store.rs` 增加 `SUPPORTED_REGISTRY_MAJOR`、`REGISTRY_READ_ONLY_PREFIX`、`CodexAuthRegistry::is_read_only`、`registry_read_only_message`、`registry_read_only_version`；测试覆盖 `"1.0"`、`"1.7"`、`"2.0"`、`"abc"`、缺失 version，以及 message/version 往返（含「配置文件错误: 」外层前缀）。
4. **`save` 兜底**：锁内、备份与写入之前拒绝只读注册表；测试只读时文件字节与备份目录不变。
5. **6 个命令入口检查**：按 design.md 第 3 节表格位置调用 `ensure_registry_writable`；每个命令一条测试，断言错误前缀，且注册表、快照、runtime `auth.json` 字节未改变（AC3）。
6. **后台写入跳过**：`update_registry_metadata`、`sync_current_auth_registry` 对只读错误 `warn!` 后按成功返回；测试 AC4。
7. **R5 加载**：`load` 两阶段解析；测试 AC5（`version = "2.0"` 且 `accounts` 为非法类型）与 1.x 解析失败时原错误文本不变。
8. **R6 TUI**：`codex_auth/app.rs` 增加 `localized_service_error` 并接入保存、切换、删除、重命名的错误 toast；EN/ZH 测试 AC6。
9. **spec 写回**：在 `.trellis/spec/ccr-codex/backend/backend-guidelines.md` 记录注册表版本规则（追加字段不升版本；主版本门；只读语义；`extra` 保留）。

## 验证命令

```bash
cargo test -p ccr-codex --all-features -- --skip export_bindings
cargo test -p ccr-tui
just fmt-check
just lint-strict
just test
```

## 风险与回滚点

- 风险文件：`models/codex_auth.rs`（公开结构增加字段，影响所有字面量构造点）、`codex_registry_store.rs`、`codex_auth_service.rs`（6 个命令）、`ccr-tui/src/tui/codex_auth/app.rs`。
- 步骤 1 是方案可行性门；步骤 2–7 可按「模型 → 判定 → 兜底 → 命令 → 后台 → 加载」分别提交，任一步失败可单独回退。
- 回滚：回退本任务提交；回退后版本丢弃 `extra`，与修复前行为一致，不损坏文件。

## 评审门

- 对照 AC1–AC7；确认 1.x 注册表的读写结果与全部现有文本不变。
