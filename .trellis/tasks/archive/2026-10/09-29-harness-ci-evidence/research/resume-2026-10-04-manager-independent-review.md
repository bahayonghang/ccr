# T07 六处夹具绑定主会话复核

日期：2026-10-04。复核人：主会话。本复核核对已保存实施记录、当前 diff 统计和规范新增句，没有重跑 Cargo 或 `just ci`。

## 范围

`git diff --numstat` 当前为 `crates/ccr/tests/managers/general.rs` +5/−0，`crates/ccr/tests/managers/legacy_registry.rs` +1/−0。这与 `resume-2026-10-04-manager-results.md` 的 5 个 `let _env = crate::setup_ccr_test_env();` 和 1 个 `let _env = setup_ccr_test_env();` 一致。候选 SHA-256 `a7082c421018a9b54167b3069ec3000cc21210bdebe3ca135cf6d7f52e775738` 已在该记录中重算，并在 `git apply --check` 退出 0 后应用。生产锁逻辑不在本次 diff 中。

`.gitignore` 的精确忽略仍在工作区，诊断 JSON 未读取。

## 规范

`.trellis/spec/ccr/backend/test-fixtures.md` 相对 HEAD 为 +1/−0。新增句要求调用 `ConfigManager::save`、`PlatformConfigManager::save` 或 `SettingsManager::save_atomic` / `restore` 的测试，在整个操作期间持有具名 `CcrIntegrationTestEnv`。该句写明 `guarded_write` 会读取 `CCR_LOCK_DIR`，并适用于 Claude Code、Codex、Grok Build、Kimi Code、OMP。该句仍未提交。

## 门槛

六个 Windows 局部门槛的退出码 0 与计数以 `resume-2026-10-04-manager-results.md` 为准：exact 1 passed，managers 17 passed，platforms 35 passed，commands sync_content 3 passed，clippy PASS。父任务迁移后 `just ci` 退出 0 的回执是 `../09-29-evergreen-harness-audit/research/resume-2026-10-04-vsce4-full-ci.json`。首轮绑定漂移 FAIL 与更早的 managers FAIL 保留。旧失败交错原因未查明。

## 未关闭

macOS 进程测试、fresh-client hook/trust、hosted CI 仍为 UNVERIFIED。T07-AC3 要求缺少环境时任务保持未完成。本复核不归档 T07。
