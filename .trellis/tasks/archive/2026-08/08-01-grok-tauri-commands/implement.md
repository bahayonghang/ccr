# 执行计划:Grok Tauri 命令桥接层

修订记录:2026-08-01 依据 Codex 审阅重排(核心 inspection API 先行 + spec 更新步骤 + bindings/CAS 验证)。

## 前置

- [x] 读父 design D1-D9 与本任务 design.md;读 spec `grok-profile-runtime.md`、`tauri-handler-registry.md`、`atomic-writer.md`
- [x] 参照物:`commands/gemini.rs`(单文件)、`commands/claude_profiles.rs`(CRUD;**注意其 rename 顺序对 grok 不可行**)、`commands/settings_raw.rs`、`commands/profile_lifecycle.rs`(仅参考,不接入)、`ccr-cli/src/commands/grok/profile.rs`(CLI 脱敏基准)

## 步骤(按序,每步后 `cargo check`)

1. [x] **核心层**:`grok.rs` 加 `GrokActivationState` + `inspect_activation_state()`(复用私有判定件,零写入)+ 四态/只读单测;Grok 保存保留 inactive 空指针并持操作锁;facade 导出;`cargo test -p ccr-cli grok -- --test-threads=1` 回归
2. [x] **spec 更新**(3.3 前置执行):`grok-profile-runtime.md` Signatures/Contracts 增补 inspection API(read-only、无副作用、UI 判定唯一入口)
3. [x] `commands/grok.rs` 骨架:Local-only 门控 helper + DTO 五件套 + `profile_to_dto` + 脱敏单测先行
4. [x] 读命令:`grok_list_profiles`、`grok_get_profile`、`grok_get_dashboard_overview`(activation 来自 inspection)
5. [x] 写命令:`grok_add_profile`、`grok_update_profile`(patch + credential_action + rename 状态机)、`grok_apply_profile`、`grok_profile_off`、`grok_delete_profile`(status 信封 + force 编排)+ 单测(rename 四结局/delete 五场景)
6. [x] settings typed:`grok_get_settings`(含 custom_models/managed_keys_locked)、`grok_update_settings`(set/unset 白名单 + CAS 重试循环)+ round-trip/并发 CAS 单测
7. [x] config raw:`commands/grok.rs` 复用 raw helper(BackupPolicy::None)+ layers(user/project/managed/requirements)+ 无备份产物单测
8. [x] `system.rs` + `platform/local.rs` 补 grok
9. [x] `handler_registry.rs` 注册全部命令;`just tauri-command-inventory` 再生;`just tauri-bindings` 再生 ts-rs 绑定;检查生成物 diff

## 验证

- [x] `cd ccr-ui/src-tauri && cargo check && cargo test grok -- --test-threads=1`
- [x] `cargo test -p ccr-cli grok -- --test-threads=1` 与 `cargo test -p ccr --test commands grok_profile -- --test-threads=1`(核心回归)
- [x] `just tauri-command-inventory-check` + `just tauri-bindings-check`
- [x] `just fmt-check` / `just lint-strict` / `just test`
- [x] 隔离冒烟(临时 `CCR_ROOT`/`GROK_HOME`):CLI create(保持 inactive)→字段 patch→apply→off→delete;Tauri rename/settings patch 由聚焦状态机与真实文件 CAS 测试覆盖
- [x] `git diff --stat crates/` 仅含 grok.rs(inspection + inactive 指针修复)+ mod.rs 导出 + spec + 测试

## 评审门

- 步骤 1-2 后:inspection API 语义与 spec 增补文本过一次人审(核心层改动,一票否决面)
- 完成后:对照父 prd「凭据边界」「状态机与数据完整性」逐条自查;交 `trellis-check` 全量检查;契约冻结公告(前端子任务解锁)

## 回滚点

步骤 1-2(核心层)/ 3-9(Tauri 层)分 commit;逆序 revert。
