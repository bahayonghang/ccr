# Implement — ccr Claude profile fable 层与显示名端到端支持

## 执行顺序（按依赖排序）

### 步骤 1：后端类型字段（R1.1 / R2.1）

- [ ] `crates/ccr-config/src/managers/config/types.rs` `ConfigSection` 增 8 个字段中尚缺的 5 个：`default_fable_model` + `default_{opus,sonnet,haiku,fable}_model_name`（opus/sonnet/haiku 模型字段已有）。
- [ ] `crates/ccr-config/src/models/platform.rs` `ProfileConfig` 同步增同样字段，并更新 `new()` / builder（如有）。
- [ ] 验证：`cargo check -p ccr-config`

### 步骤 2：env 常量 + 写出 + 清理 + 登记（R1.2~R1.4 / R2.2）

- [ ] `crates/ccr-cli/src/managers/settings.rs`：增 5 个常量；`update_from_config` 增 5 段 `if let Some`；清理集合（`clear_anthropic_vars`/`clear_managed_vars`）纳入 5 个新常量。
- [ ] `crates/ccr-cli/src/platforms/claude.rs` `get_env_var_names`(:335) 追加 5 个 env 名。
- [ ] 同步 `profile_to_section` / `section_to_profile`（claude.rs / managers）转换函数带上新字段。
- [ ] 验证：`cargo check -p ccr-cli`

### 步骤 3：加载迁移（R3）

- [ ] 对照前置任务 `custom_model_option` 迁移实现，处理 `other`/platform_data 残留键抬升与移除。
- [ ] 验证：迁移单测

### 步骤 4：单测（AC1~AC4）

- [ ] settings.rs：仿 `test_update_from_config_writes_custom_model_option` 增 fable + name 写出测试。
- [ ] settings.rs：增「先含 fable 后不含 → 被清除」防串档测试。
- [ ] claude.rs：仿 `test_custom_model_option_migrates_from_toml_and_writes_env` 增 fable/name 迁移测试。
- [ ] 验证：`just test`（或 `cargo test -p ccr-cli -p ccr-config -- --test-threads=1`）

### 步骤 5：前端录入（R4，AC5）

- [ ] `ccr-ui/src/types/claude.ts` + `claudeProfileEditor.ts` 增可选字段。
- [ ] `ccr-ui/src/components/claude/ClaudeProfileEditorSections.vue` 高级模型映射区追加 fable 模型框 + 四显示名框。
- [ ] `ccr-ui/src/i18n/locales/zh-CN.ts` + `en-US.ts` 增 label/helper（同步 keys.txt）。
- [ ] 验证：`just frontend-check-quick`

### 步骤 6：研究验证项（A2）

- [ ] 用 `context7` / 官方文档确认 `ANTHROPIC_DEFAULT_FABLE_MODEL` 与 `*_MODEL_NAME` 拼写语义；与截图一致则记录确认，冲突则以实际生效为准并在 prd 备注。

## 验证命令（全量门禁）

```
just version-check
just fmt-check
just lint-strict
just test            # Rust，内部 --test-threads=1
just frontend-check-quick
```

## Review Gate

- 触发 `rust-security-reviewer`（碰 settings/credential 写路径）与 `frontend-quality-reviewer`（ccr-ui 多文件）。
- 重点核对「四处登记点」是否齐全（漏一处即丢配置/串档）。

## Rollback Point

- 步骤 1~2 是一组（类型+映射），回滚需整组还原。
- 回滚后注意：已写入 settings.json 的 fable env 不再被 ccr 清理，回滚前手动从 `~/.claude/settings.json` 移除 `ANTHROPIC_DEFAULT_FABLE_MODEL*` / `*_MODEL_NAME`。
