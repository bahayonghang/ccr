# Implement — 执行计划

> 顺序执行；每步带验证。Rust 直跑测试时统一 `-- --test-threads=1`。
> 仅在 `task.py start`（状态 → in_progress）后开始编码。

## Stage 0 — 前置基线

- [ ] 读 spec：ccr-cli `backend-guidelines.md`、ccr-config `backend-guidelines.md`、ccr-ui `provider-template-contracts.md`、`api-facade-boundary.md`。
- [ ] 基线绿：`just version-check` → `just fmt-check` → `just lint-strict` → `just test`。
- 验证：基线全过，记录初始状态以便回滚对比。

## Stage 1 — auth_mode 自愈核心（R1，主修复）

- [ ] 1.1 在 `ClaudeAuthService`（crates/ccr-cli/src/services/claude_auth_service.rs）新增 `is_api_key_shaped(&ProfileConfig) -> bool` 与 `effective_auth_mode(&ProfileConfig) -> ClaudeProfileAuthMode`（设计 §3.1）。`resolve_profile_auth_mode` **不改**。
- [ ] 1.2 `ClaudePlatform::profile_auth_mode`（claude.rs:158）委托 `effective_auth_mode`；复核 `profile_auth_source` / `apply_profile` / `profile_to_json` 调用方语义。
- [ ] 1.3 `ClaudePlatform::normalize_profile`（claude.rs:175）：若 `effective != 字面存储` 则写回 `platform_data["auth_mode"]="api_key"`（权威纠正）。
- [ ] 1.4 单测：
  - AC1：subscription+base_url+token+default_opus_model → `apply_profile` 后 settings 含 `ANTHROPIC_BASE_URL` 与 `ANTHROPIC_DEFAULT_OPUS_MODEL`。
  - AC2：`save_profile` 后 profiles.toml 该 profile `auth_mode == "api_key"`。
  - AC3：`test_subscription_profile_*` 仍过（纯订阅不受影响）。
- 验证：`cargo test -p ccr-cli -- --test-threads=1`，新增用例全过。
- **Review Gate G1**：确认未破坏 `resolve_profile_auth_mode` 只读语义、日志无 token。
- **回滚点**：此 Stage 自成闭环（主因止血），可独立提交。

## Stage 2 — custom_model_option 正规化（R3）

- [ ] 2.1 `ProfileConfig`（ccr-config/.../platform.rs）、`ConfigSection`（.../types.rs）新增 `custom_model_option` / `custom_model_option_name`（Option，skip_serializing_if）。
- [ ] 2.2 `base::profile_to_section` / `section_to_profile`（base.rs）补 clone。
- [ ] 2.3 `ClaudeSettings::update_from_config`（settings.rs）映射 → `ANTHROPIC_CUSTOM_MODEL_OPTION` / `_NAME`；确认 `clear_anthropic_vars` 已覆盖这两个前缀键。
- [ ] 2.4 `get_env_var_names`（claude.rs:312）追加两项。
- [ ] 2.5 迁移自愈：`normalize_profile`（或加载归一处）将 platform_data 内同名键抬升为 typed 并删除残留键；**不**写入 `default_opus_model`。
- [ ] 2.6 单测 AC4：含 platform_data `custom_model_option` 的 profile，save→load 后 typed 字段就位、platform_data 残留清除；apply 后 settings 写出 `ANTHROPIC_CUSTOM_MODEL_OPTION`。
- 验证：`cargo test -p ccr-config -p ccr-cli -- --test-threads=1`。
- **Review Gate G2**：迁移幂等、无语义混淆（custom ≠ default_opus）。

## Stage 3 — Tauri 与前端字段贯通（R2 + R3.3）

- [ ] 3.1 Tauri `patch_profile_with_config` / `profile_to_json`（ccr-ui/src-tauri/src/commands/claude.rs）补 `custom_model_option(_name)` 读写。
- [ ] 3.2 `ClaudeProfileEditorForm`（ccr-ui/src/types/claudeProfileEditor.ts）补字段；表单高级映射区（ClaudeProfileEditorSections.vue）加输入框 + helper。
- [ ] 3.3 ClaudeCodeProfilesView.vue：保存 payload 携带新字段；新建/编辑时第三方信号下 `auth_mode` 默认 `api_key`（不再死守 subscription:584/795/835）。
- [ ] 3.4 内联校验（R2.2）：检测「API-key 形态 + subscription」→ 鉴权区显示 `editor-banner--warn` 提示。
- 验证：`cd ccr-ui && bun run type-check && bun run lint`；按需 `bun run test:smoke -- tests/provider-templates.smoke.test.ts`。
- **Review Gate G3**：dispatch `tauri-ipc-reviewer`（命令签名/注册）与 `frontend-quality-reviewer`（多文件前端改动）。

## Stage 4 — UX 文案、模板与文档（R4）

- [ ] 4.1 i18n（zh-CN/en-US）补 helper：第三方须 api_key；`/model` 显示 Opus/Sonnet/Haiku 属正常、GLM 底层生效；`[1m]` 版本要求。
- [ ] 4.2 评估「第三方模型」provider 模板：预置 `auth_mode=api_key` + `provider_type=third_party_model`（遵循 provider-template-contracts，仅非密字段）。
- [ ] 4.3 文档页更新（docs 第三方模型接入说明）；若动 docs：`cd docs && npm run build`。
- 验证：i18n 键齐全（zh/en 对齐 `*.keys.txt`）。

## Stage 5 — 全量验收

- [ ] `just version-check` → `just fmt-check` → `just lint-strict` → `just test`（`-- --test-threads=1`）。
- [ ] `just frontend-check-quick`（含必要 smoke）。
- [ ] AC1–AC7 全部勾选确认。
- [ ] （可选）真机回归：重切 `chy` → 启动 `claude` → 确认 `~/.claude/settings.json` 写出 chybenzun.top + 模型映射。
- **Review Gate G4（finish 前）**：trellis-check / spec 更新（auth_mode 纠正契约写入 ccr-cli spec）。

## 风险与回滚

- 每个 Stage 自成可提交单元；Stage 1 即主因止血，可优先合入。
- 回滚 = revert 对应 Stage；迁移为「抬升+删冗余键」，typed 字段保留原值，无数据销毁。
- 安全红线贯穿：原子写/锁/备份不回退，日志脱敏，masking 不破坏。

## Sub-agent 派发提示

- 每次派发 prompt 首行：`Active task: .trellis/tasks/06-19-claude-third-party-model-authmode`。
- 上下文顺序：implement.jsonl → prd.md → design.md → implement.md。
