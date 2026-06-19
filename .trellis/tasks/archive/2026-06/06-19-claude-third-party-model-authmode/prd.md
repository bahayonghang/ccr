# 修复 Claude 第三方模型 Profile 因 auth_mode 语义静默失效

## Background

用户在 ccr-ui「Claude Code Profile」中配置第三方模型（GLM via `chy`，`base_url=https://chybenzun.top`，Opus 默认模型 `glm-5.2[1m]`，effort `max`），保存并切换后启动 `claude`，Claude Code 仍显示官方 Opus 4.8，「根本无法使用 glm」。

经全栈排查，**CLI 持久化与 env 映射链路完全正确**（详见 design.md 验证表），真正失效点在 `auth_mode` 语义与一处历史遗留字段。磁盘真实 `[chy]` profile：

```toml
[chy]
base_url = "https://chybenzun.top"
auth_token = "***"
effort_level = "max"
provider = "chy"
auth_mode = "subscription"            # 根因 #1
custom_model_option = "glm-5.2[1m]"        # 根因 #2（孤儿字段，非 default_opus_model）
custom_model_option_name = "glm-5.2[1m]"
```

对照可用的 `[axiom-guomo]`：`default_opus_model = "glm-5.2[1m]"` + `auth_mode = "api_key"`，两处差异即问题所在。

## Goal

让「在 ccr-ui 配置第三方模型 → 切换 → 在 Claude Code 真正命中第三方模型」这条路径默认可用、不可被静默清空，并消除用户对 `/model` 显示的误解。

## Root Causes（须全部覆盖）

1. **主因：`auth_mode = "subscription"` 在 apply 时清空全部覆盖。**
   `ClaudePlatform::apply_profile`（crates/ccr-cli/src/platforms/claude.rs:278）的 subscription 分支调用 `clear_managed_vars()`，不写任何 `ANTHROPIC_*` / `CLAUDE_CODE_*`。而 `resolve_profile_auth_mode`（crates/ccr-cli/src/services/claude_auth_service.rs:737）**显式 auth_mode 优先于推断**，表单默认值又是 `subscription`（ccr-ui/src/views/ClaudeCodeProfilesView.vue:584）。结果：明显是第三方/API-key 的 profile 被当作官方订阅，全部第三方配置被丢弃。

2. **次因：Opus 模型落在孤儿字段 `custom_model_option`。**
   该 key 当前前端不存在（仅 Codex 编辑器与 i18n 出现），`update_from_config` 不识别，永远不会变成 `ANTHROPIC_DEFAULT_OPUS_MODEL` 或 `ANTHROPIC_CUSTOM_MODEL_OPTION`，被静默忽略。属旧 schema 残留。

3. **认知/软性：`/model` 显示与 `[1m]` 后缀。**
   即便 #1#2 修好，Claude Code 的 `/model` 仍显示「Opus 4.8」而非第三方 ID（官方行为：不替换内置 alias 文案），易被误读为未生效；`[1m]` 需较新 Claude Code 版本。缺少引导与说明。

## Scope（已确认：全量，含 UX 与文档）

- 策略：**自动纠正 + 校验**（不是仅警告，也不是硬阻断）。
- 覆盖 #1 + #2 + #3。

## Requirements

### R1 — auth_mode 自动纠正（主修复）
- R1.1 定义「API-key 形态」判定（保守规则，避免假阳性）：profile 满足任一即视为 API-key 形态：
  - `provider_type == "third_party_model"`；或
  - `base_url` 与 `auth_token` 同时非空。
  - 注：**不**把「模型映射字段非空」纳入判定——`ANTHROPIC_DEFAULT_*_MODEL` 在官方订阅下也能用于钉快照，以此判 api_key 会误伤「订阅+快照钉选」并导致 `section.validate()` 失败。真实第三方必然带 base_url+token，已覆盖 chy 场景。
- R1.2 当 profile 为 API-key 形态但 `auth_mode == subscription` 时，**自动纠正为 `api_key`**，并 `tracing::warn`（不得打印 token）。
- R1.3 纠正在两处生效：
  - **保存时**（权威纠正，落盘后存储值即为 `api_key`，列表/表单随之刷新）；
  - **应用时**（防御式自愈，使既有 `chy` 这类存量 profile 无需重存即可正确 apply）。
- R1.4 纯订阅 profile（无 base_url/token/映射）行为不变，仍走 subscription 清空逻辑。

### R2 — 前端默认与校验
- R2.1 新建 profile 时，若已填入 `base_url`/`provider`/任一模型映射，`auth_mode` 默认或自动切换为 `api_key`（不再死守 subscription 默认）。
- R2.2 表单在「账号与鉴权」分区检测到「API-key 形态 + subscription」矛盾时，给出**内联可见提示**（非弹窗），说明该模式会丢弃第三方配置。
- R2.3 保存后后端若发生纠正，前端列表/编辑态须反映纠正后的 `api_key`（依赖 `profile_to_json` 回传）。

### R3 — custom_model_option 正规化
- R3.1 将 `ANTHROPIC_CUSTOM_MODEL_OPTION`（含 `_NAME`）提升为一等字段：`ProfileConfig` / `ConfigSection` 增加 `custom_model_option`（及 name），`update_from_config` 映射到对应 env，`get_env_var_names` 同步登记。
- R3.2 加载/保存时迁移：platform_data 内的 `custom_model_option` / `custom_model_option_name` 抬升为新 typed 字段（一次性自愈），并从 platform_data 移除残留键。
- R3.3 前端在高级模型映射区暴露该字段（含 helper），不再静默吞掉。
- R3.4 不得把 `custom_model_option` 误等同于 `default_opus_model`（二者语义不同，仅做字段归位，不改语义）。

### R4 — UX 与文档（#3）
- R4.1 高级模型映射/鉴权区 helper 文案补充：第三方模型须用 api_key；`/model` 仍显示 Opus/Sonnet/Haiku 文案属正常，GLM 在底层生效；`[1m]` 需较新 Claude Code。
- R4.2 评估并（按需）补一个「第三方模型」provider 模板：预置 `auth_mode=api_key` + `provider_type=third_party_model`，降低用户踩坑概率（遵循 provider-template-contracts）。
- R4.3 更新相关文档页（第三方模型接入说明）。

## Constraints

- 遵守根 CLAUDE.md 安全约束：保密 masking、改动前备份、文件锁、原子写不得回退；日志不得泄露 token。
- 内部实现注释中文，公共 API doc 英文。
- 改动需可被 `just lint-strict` / `just test`（Rust）与 `just frontend-check-quick`（前端）验证。
- 既有订阅相关测试（crates/ccr-cli/src/platforms/claude.rs 中 `test_subscription_profile_*`）须继续通过；`resolve_profile_auth_mode` 的只读语义不被破坏（纠正逻辑作为独立步骤叠加，而非改写 resolve）。

## Acceptance Criteria

- [x] AC1：构造「base_url+token+default_opus_model+auth_mode=subscription」的 profile，`apply_profile` 后 `~/.claude/settings.json` 含 `ANTHROPIC_BASE_URL` 与 `ANTHROPIC_DEFAULT_OPUS_MODEL`（即未被清空）——`test_apply_mismarked_subscription_third_party_writes_overrides` + `test_apply_defensively_heals_stale_subscription_profile`。
- [x] AC2：保存同样的 profile 后，落盘 `profiles.toml` 中该 profile 的 `auth_mode` 被纠正为 `api_key`——`test_save_corrects_mismarked_subscription_to_api_key`。
- [x] AC3：纯订阅 profile（无 base_url/token/映射）apply 仍清空 `ANTHROPIC_*`，行为不变——既有 `test_subscription_profile_*` 通过。
- [x] AC4：含 `custom_model_option` 的 profile，apply 后 `settings.json` 写出 `ANTHROPIC_CUSTOM_MODEL_OPTION`；platform_data 残留 key 经 typed 化自动迁移——`test_update_from_config_writes_custom_model_option` + `test_custom_model_option_migrates_from_toml_and_writes_env`。
- [x] AC5：前端在「API-key 形态 + subscription」时显示内联提示；模板应用时 auth_mode 默认 api_key——type-check / lint / smoke 通过。
- [x] AC6：`just lint-strict`、`just test`、`just frontend-check-quick` 全绿（`just version-check` 仅因 ccr-ui/README 既有 6.4.2 同步漂移失败，与本任务无关）。
- [x] AC7：`docs/(en/)reference/platforms/claude.md` 新增第三方模型章节；helper 文案明确 `/model` 显示行为与 `[1m]` 版本要求。

## Out of Scope

- 不改变 Claude Code 自身对 `/model` 文案的渲染（属上游行为，仅做说明）。
- 不引入对 GLM/Z.AI 的专有逻辑；保持「任意 Anthropic 兼容第三方」的通用性。
- 不重构 Codex/Gemini/Droid 的 auth_mode 模型（本任务仅限 Claude 平台）。

## References

- 官方：[Model configuration](https://code.claude.com/docs/en/model-config)、[Z.AI Claude Code 接入](https://docs.z.ai/devpack/tool/claude)
- 设计与执行细节见 `design.md` / `implement.md`
