# Grok 平台底层：Platform::Grok 枚举与 GrokPlatform 切换引擎

## Goal

在 ccr 平台抽象层新增 Grok CLI（xAI Grok Build）平台：扩展 `Platform` 枚举并对全 workspace 穷举 match 位置落显式 capability 决策，实现 `GrokPlatform`（`PlatformConfig` trait）切换引擎——以 CCR profile 驱动 `~/.grok/config.toml` 的 `[model.custom]` + `[models].default` 双路切换（官方模型选择 / 第三方 BYOK），具备原条目恢复、CAS 并发防护、明确的删除语义与脱敏基线。

> rev2：吸收审阅 CORR-001/002/003、ARCH-002、SEC-001。

## Requirements

（上游事实见父任务 research/grok-config-format.md rev2；枚举影响图见 research/platform-enum-impact-map.md）

### R1 Platform 枚举扩展 + 全域 match 决策（ARCH-002）

- `Platform::Grok`：`display_name = "Grok Build"`、`short_name = "grok"`、icon、`is_implemented() = true`、`all()`、`FromStr`（`grok`/`grok-build`/`grok-cli`）。
- 以 `cargo check --workspace` 驱动，对影响图列出的每个穷举 match 落显式分支：工厂=支持；doctor 设置/运行时校验=skip（Qwen 先例）；doctor profile 展示=并入通用臂；MCP preset home=映射 `.grok`、install=明确拒绝；sessions parser/models=明确不支持；profile current 展示名=`"Grok"`。禁止 `_ =>` 兜底；影响图未列位置先回填文档再写代码。
- 本任务**不**改 `auth_profile_supported()`（归 cli-surface）。

### R2 GrokPlatform 切换引擎

- 存储：`~/.ccr/platforms/grok/profiles.toml`（base helpers）；运行时：`$GROK_HOME/config.toml` 缺省 `~/.grok/config.toml`。
- 双路：
  - **第三方（base_url 非空）**：接管 `[model.custom]`（model 必填/base_url/name/api_backend/context_window/supports_backend_search + 凭据二选一），`[models].default = "custom"`。
  - **官方（base_url 空）= 纯模型选择器**：恢复或删除 `[model.custom]`（见 R3）；profile.model 有值 → `[models].default = <model>`；无值 → **移除 `models.default` 键**回落上游默认。认证不经营（auth.json/`XAI_API_KEY` 自然接管）。
- 非托管段（`[cli]/[session]/[memory]/[ui]/[subagents]/[marketplace]/[endpoints]` 及未知键）在**结构与值层面**原样保留；toml round-trip 不保注释/格式（入口状态兜底），文案不得使用"逐字节保留"。
- `~/.grok/auth.json` / `mcp_credentials.json`：**永不读写、不备份、不校验**。

### R3 入口状态与原条目恢复（CORR-001）

- 首次切换前记录入口状态：整份 config.toml 原文 + 结构化字段（原始 `[model.custom]` 是否存在及其内容、原始 `[models].default` 是否存在及其值）。
- 切官方/off/清理运行时：原始 `[model.custom]` 存在 → 恢复原内容；原不存在 → 删除；`models.default` 同理按原值恢复或按 R2 官方语义设置。
- 第三方 → 官方 → 第三方往返后无 CCR 残留、原条目内容不丢。

### R4 并发与删除语义（CORR-003）

- 写序固定：入口状态 → config.toml → profiles.toml `current_config` → registry 指针；config.toml 为运行时真相源，指针滞后由 `get_current_profile` 漂移检测兜底（部分失败自愈路径写入测试）。
- config.toml 的 RMW 用 `ccr_core` `write_guarded_versioned`（读取时记 content token，CAS 写回；Conflict → 重读重建一次，再冲突则报错），防 Grok 自身/其他进程并发写被覆盖。
- **删除当前激活 profile 默认拒绝**（错误提示先 `off` 或 switch）；`--force` 语义（恢复入口态后删除并清指针）由调用方传入意图，引擎提供 `clear_active_profile_runtime` 原语。非激活 profile 删除走常规路径 + registry reconcile。

### R5 校验与脱敏（SEC-001 披露口径）

- validate：base_url http(s) 前缀；第三方必填 `model`；`api_backend` 三值枚举（缺省 responses）；`auth_token`/`env_key` 恰好其一；`env_key` 合法环境变量名且**仅单字符串**；官方 profile 携带任一凭据字段报错（决策 3）。
- `auth_token` 全程 `ccr_core::Secret`；日志只记 profile 名与路线；暴露 `safe_base_url_for_display` 与 `profile_auth_mode` 公共 helper 供 CLI/TUI 复用（CORR-005 上游依赖）。
- 明文存在位置以父 PRD 披露矩阵为准；不承诺"唯一明文消费点"。
- `get_env_var_names`：`XAI_API_KEY`、`GROK_CODE_XAI_API_KEY`。

## Acceptance Criteria

- [ ] `cargo check --workspace` 通过且无 `_ =>` 兜底新增；影响图每行决策有对应代码与（可测处的）测试。
- [ ] 第三方切换：托管条目 + default 正确；含 `[session]/[ui]/[model.other]` 的杂项配置结构与值保留（测试断言）。
- [ ] 官方切换/off：原始 `[model.custom]` 恢复（原存在场景）或删除（原不存在场景）；`models.default` 恢复/移除语义正确；**往返测试**通过。
- [ ] 入口状态首次生成、不被覆盖，且含结构化原条目记录。
- [ ] CAS：模拟外部并发修改 config.toml → 冲突被检测，不覆盖他方内容（测试断言）。
- [ ] 删除当前激活 profile 被拒绝；`clear_active_profile_runtime` 恢复入口态。
- [ ] 凭据互斥、validate 全矩阵（含 env_key array 拒绝、官方带凭据拒绝）中文错误。
- [ ] 漂移：外部改 default 后 `get_current_profile` 不误报。
- [ ] `TestGrokEnv`（临时 `CCR_ROOT`+`GROK_HOME`）隔离；`cargo test -p ccr-cli grok -- --test-threads=1`、`cargo test -p ccr-config -- --test-threads=1`、`just lint-strict` 通过。

## Notes

- 复杂任务：design.md + implement.md 必备（已提供）。
- 本任务是 cli-surface 与 tui-tab 的前置；对外暴露的 helper（auth-mode/safe-url/clear-runtime）是兄弟任务的契约。
