# Implement：GrokPlatform 切换引擎（rev2）

> 执行前置：`python ./.trellis/scripts/task.py start 07-28-grok-platform-core`；读 `.trellis/spec/ccr-cli/backend/index.md`、`.trellis/spec/ccr-config/backend/index.md` pre-dev checklist；对照 `research/platform-enum-impact-map.md`。

## 步骤清单

### 1. Platform 枚举扩展（crates/ccr-config/src/models/platform.rs）

- [ ] `Platform::Grok` 变体：display/short/icon/all/implemented/FromStr（`grok`/`grok-build`/`grok-cli`）；**不改** `auth_profile_supported()`。
- [ ] 本文件测试更新（all 5→6、implemented 4→5、from_str、auth_profile_supported 的 all 断言补 Grok）。
- 验证：`cargo test -p ccr-config -- --test-threads=1`

### 2. 全域穷举 match 决策落地（ARCH-002）

- [ ] `cargo check --workspace` 列出全部编译错误位置，逐一对照影响图落显式分支：
  - `ccr-cli/services/doctor_service.rs:1050,1427` → skip（Qwen 先例文案）；`:1619` → 并入通用 profile 展示臂
  - `ccr-skills/managers/mcp_preset_manager.rs:110` → `.grok` home 映射；`:156,167` → 明确"暂不支持"错误
  - `ccr-store/sessions/parser.rs:41,510,521`、`models.rs:121` → 明确不支持（空/None/保守占位，按现有签名最小化）
  - `ccr-cli/commands/profile/current.rs:453` → `"Grok"`
  - 影响图未列的新位置 → 先回填影响图（含决策理由）再写代码；全程禁止 `_ =>` 兜底
- 验证：`cargo check --workspace`；受影响 crate 各自 `cargo test -p <crate> -- --test-threads=1`

### 3. GrokPlatform 骨架（crates/ccr-cli/src/platforms/grok.rs + mod.rs 注册）

- [ ] struct/new（PlatformPaths + `GROK_HOME` 解析）/from_parts；profiles 走 base helpers；trait 骨架 + `create_platform` 分支。
- [ ] `GrokProfileAuthMode` + 推断与互斥校验（env_key 仅单字符串、合法 env var 名、官方带凭据拒绝）。
- 验证：`cargo check -p ccr-cli`

### 4. 切换引擎核心

- [ ] `ProfileEntryConfigState`（exists/content/original_custom_model/original_default_model）：capture（仅首次）/restore/清理；AtomicWriter secret + 私有权限。
- [ ] `build_switch_spec`：路线判定 + 字段收集 + validate 全矩阵。
- [ ] `apply_switch_spec`：RMW 只触碰 `model.custom` 与 `models.default`；官方路线恢复/删除原条目 + default 写/恢复/移除；**CAS 循环**（`content_version_token` + `write_guarded_versioned`，冲突重读一次再冲突报错）。
- [ ] `apply_profile` 写序：validate → 入口状态 → config.toml(CAS) → `base::update_current_config` → `base::update_registry_current_profile_with_paths`；指针步失败时不回滚 config、返回可重试错误。
- [ ] `clear_active_profile_runtime`（off 原语）：恢复入口态 + 清指针/current_config + 删入口状态文件。
- [ ] `delete_profile`：激活中拒绝（中文提示先 off/switch）；非激活走 base reconcile。
- [ ] `get_current_profile` 漂移检测（含官方"default 键缺省"期望态）。
- [ ] pub helpers：`profile_auth_mode()`、`safe_base_url_for_display()`（兄弟任务契约）；`get_env_var_names` → `XAI_API_KEY`/`GROK_CODE_XAI_API_KEY`。
- 验证：`cargo clippy -p ccr-cli`

### 5. 测试（TestGrokEnv：tempdir + CCR_ROOT/GROK_HOME）

- [ ] 第三方切换写入 + 杂项段（`[session]/[ui]/[model.other]`/未知键）结构与值保留。
- [ ] 官方切换/off：原条目恢复（原存在）与删除（原不存在）双场景；default 恢复/移除；**第三方→官方→第三方往返**。
- [ ] 入口状态：首次生成、不覆盖、结构化字段正确。
- [ ] CAS：写前外部篡改 config.toml → Conflict 检测与安全重试/报错。
- [ ] 指针步失败自愈：config 已写、registry 未写 → get_current_profile 不误报，重试 apply 收敛。
- [ ] 删除激活 profile 拒绝；clear 后可删。
- [ ] validate 矩阵（api_backend 非法/缺 model/双凭据/env_key array/env_key 非法名/官方带凭据）。
- [ ] 漂移检测；FromStr/display。
- 验证：`cargo test -p ccr-cli grok -- --test-threads=1`

### 6. 收尾门

- [ ] `just fmt` → 查 diff → `just fmt-check` → `just lint-strict` → `just test`
- [ ] 触发 `rust-security-reviewer`（新增凭据字段 + 原子写路径）。
- [ ] 提交拆分：① `feat(config): ✨ add grok platform enum with workspace-wide capability arms`（步骤 1-2）② `feat(cli): ✨ add grok platform switch engine`（步骤 3-5）。

## 回滚点

- 两个 commit 独立可 revert；步骤 2 的决策臂不改变既有平台行为（skip/拒绝为纯新增分支）。

## 明确不做

- `auth_profile_supported()` / CLI 命令面（cli-surface）；TUI（tui-tab）
- auth.json 任何读写；sessions/usage/skills 域功能实现（只落"不支持"臂）
- Windows DACL（父 PRD 后续候选）；env_key array；secret-store overlay
