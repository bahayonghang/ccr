# auth_mode 判定一致性与 env 清理修复

## Goal

修复 C1/C9/C2/C11:消除"官方登录被残留 env token 静默架空、诊断却显示正常"这一核心互扰缺陷,并定夺托管 env 键的所有权模型。父任务 `07-29-claude-auth-profile-isolation` 的第 1 号子任务,必须最先落地。

## 问题(源码已核实,含二轮复核)

1. **C1 判定不一致**:写入侧 `apply_profile` 用 `effective_auth_mode`(`crates/ccr-cli/src/platforms/claude.rs:325` → `claude_auth_service.rs:808`),清理侧 `clear_profile_api_key_overrides_if_needed` 用字面 `resolve_profile_auth_mode`(`claude_auth_service.rs:267-272`),诊断侧 `get_runtime_summary` 同(`:677`)。
   复现:profile 字面 `auth_mode="subscription"` 但含 `base_url+auth_token` → apply 写入 `ANTHROPIC_AUTH_TOKEN` → `auth switch` 跳过清理 → 订阅静默失效;`api_key_profile_override_active` 误算 `false`。
2. **C9 根因**:`apply_profile` 检测到冲突仅 `tracing::warn`(`platforms/claude.rs:326-336`),不回写 profiles.toml,错误字面值永久留存。
3. **C2 清理不彻底**:三处只 `clear_anthropic_vars()` 而非 `clear_managed_vars()`,遗留 5 个非 Anthropic 托管键(`crates/ccr-types/src/claude_settings.rs:204-210`):`claude_auth_service.rs:279`、`application/profile_off.rs:216`、`commands/lifecycle/clear.rs:134`。
4. **C11 所有权语义缺失(二轮新增)**:`clear_anthropic_vars` 按前缀删**所有** `ANTHROPIC_*`(`claude_settings.rs:220-222`),包括 ccr 从未写入的用户自有键(`ANTHROPIC_CUSTOM_HEADERS`、用户自设 `ANTHROPIC_API_KEY` 等)。直接把 C2 的三处改成 `clear_managed_vars()` 会**扩大**这一误删面,与"保留非托管 env"及 doctor"用户自有来源只告警不删除"原则冲突。

## Requirements

- R1:`clear_profile_api_key_overrides_if_needed` 与 `get_runtime_summary` 的 auth_mode 判定改用 `effective_auth_mode`,与写入侧对齐。
- R2:`apply_profile` 检测到纠正时,将纠正后的 auth_mode 回写 profiles.toml(持久化自愈)。**已定夺**:回写失败时在修改 runtime settings **之前**阻断 apply(先持久层后运行时),错误信息指引用户重存 profile;保留现有 warn 日志。
- R3(替代原"三处直接改 clear_managed_vars"):建立**显式所有权模型**——
  - 在 ccr-types 定义显式 `CCR_MANAGED_KEYS`(以 `ConfigSection::to_managed_env_pairs` 实际可写常量为准,不得硬编码过时数量),新增 `clear_ccr_managed_vars()` 只删显式清单内的键,新增 `has_managed_overrides()` 与之对偶;
  - `auth switch` / `profile off` / `clear` 三路径改用新语义;
  - `apply_managed_env` 与 subscription profile apply 同样使用显式清单,避免 profile 切换删除用户自有 `ANTHROPIC_API_KEY`/`ANTHROPIC_CUSTOM_HEADERS`;
  - 用户自有 `ANTHROPIC_*` 键(不在清单内)**不删除**,交由 doctor 检测告警(与 `07-29-claude-auth-doctor-spec` 的 R2 衔接);
  - design.md 必须记录该选择相对"前缀全清"的 tradeoff:清单外残留键(旧版本 ccr 或第三方工具写入)可能继续压制订阅——这正是 doctor 告警要覆盖的场景,不靠误删兜底。
- R4:同步更新 `commands/lifecycle/clear.rs:49-65` 的收集枚举(当前按 `ANTHROPIC_` 前缀过滤)、空判断、确认文案与计数,使展示与实际清理范围一致。
- R5:不得直接扩大 `has_anthropic_overrides()` 的语义(现有调用方依赖);新增 `has_managed_overrides()` 并逐调用点评估替换。
- R6:遵守 `.trellis/spec/ccr-cli/backend/backend-guidelines.md:118-210`:`resolve_` 与 `effective_` 两层不得合并(`:124-125`);只改调用点选择与清理语义,不改两函数本身。
- R7:本任务触碰的契约条款(backend-guidelines auth_mode 契约、ccr-types env_keys 所有权)在本任务内同步更新规范,不延后。

## Acceptance Criteria

- [ ] 回归:字面 subscription + api_key 形态 profile,apply 后 `auth switch` 清空全部**显式托管**键(含 5 个非 Anthropic 键)。
- [ ] 回归:同形态下 runtime summary 报 `current_profile_auth_mode=api_key`,且存在显式托管 override 时 mode 为 `profile_only`,不再误报订阅正常。
- [ ] 回归:`apply_profile` 后 profiles.toml 字面 auth_mode 已纠正,重复 apply 不再 warn;模拟回写失败时 apply 报错且 settings.json 未被修改。
- [ ] 回归:env 中用户自有键(如 `ANTHROPIC_CUSTOM_HEADERS`、清单外 `ANTHROPIC_API_KEY`)在三条清理路径后**保留**。
- [ ] 回归:同一用户自有键在 API-key profile apply 与 subscription profile apply 后同样保留。
- [ ] `clear` 命令的列表/计数/文案与实际清理集合一致。
- [ ] 既有测试(`platforms/claude.rs:831` oauthAccount 保留断言、06-19 双点纠正测试)通过;`just lint-strict` + `just test` 通过。
- [ ] backend-guidelines / ccr-types 规范条款同步更新完成。

## Notes

- 改动集中在 `crates/ccr-types`(claude_settings.rs)与 `crates/ccr-cli`;与 `07-29-profiles-*` 前端任务族无文件交集。
- Planning status:`design.md` 已记录所有权 tradeoff、回写失败语义与调用点审计,`implement.md`/JSONL 已就绪;等待最新父任务规划批准后作为第 1 个子任务 start。
