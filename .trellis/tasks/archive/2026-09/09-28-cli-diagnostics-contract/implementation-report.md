# T04 实施报告：CLI 诊断结果与退出码契约

日期：2026-09-28。任务：`09-28-cli-diagnostics-contract`。角色：`trellis-implement`。

## 交付结论

T04 生产实现、测试、命令文档和规范已完成。最终五条测试命令累计 38 项通过；`ccr`、`ccr-cli`、`ccr-config` 三包 all-targets / all-features 严格 Clippy 通过。T04 范围的 `git diff --check` 退出 0。生产文件已冻结，交主代理安排独立检查及 T10 集成门禁。

本报告不修改任务状态、PRD 验收勾选或父任务证据台账。未执行提交、推送或归档。

## 基线与工作区

- 实施基线及交付时 HEAD：`34d8a85e0e48b793733835e0304c8ed33940fcee`。
- 当前改动未提交。共享工作区包含其他任务改动；本报告只描述 T04 的实现责任。
- T01 纯读取契约、T02 平台认证与 enabled 策略是本任务依赖；未撤回这些契约。
- `final-source-evidence.json` 记录 2026-09-28 08:23:10 -05:00 的 HEAD、T04 路径状态、19 个产品/测试/文档/规范文件 SHA256，以及 Cargo.lock SHA256。该记录用于后续检查比对，不能代替独立检查。

## 实现机制

### typed report 与 binary 边界

新增 `services/validate_service/diagnostics.rs`，提供 `DiagnosticSeverity`、`DiagnosticCategory`、`DiagnosticCheck`、`DiagnosticReport` 和 `diagnose()`。报告保留分类、严重程度和稳定退出码，不保存原始 parser 输入或秘密字段值。

`validate_report_command()` 负责终端渲染并返回报告。兼容 `validate_command()` 在报告失败时返回错误。`doctor_report_command()` 返回 `DoctorReport`，兼容 `doctor_command()` 返回 `Result`。这些可嵌入入口不执行 `process::exit`。

`crates/ccr/src/main.rs` 返回 `ExitCode`，在日志初始化之前执行 Validate 和 Doctor。诊断不因此创建 `.ccr/logs`。Validate 使用报告退出码；Doctor 保留 failed 为 1、warning-only 为 0 的现有契约。

### 领域规则与只读读取

诊断从共享平台能力集合遍历 Claude、Codex、Grok。Profile 使用 T01 `ConfigManager::for_platform(...).load()` 纯读取入口；实际认证模式调用对应平台 `validate_profile`。原 `ValidateService` / `FullValidationReport` 兼容 API 保留。

`ConfigValidator::validate_section_with` 提供领域 validator 注入入口。原 `validate_section` API-key 校验入口保留，避免扩大本任务对 legacy 调用者的影响。

Current marker 使用 T01 `base::load_current_profile_marker`，读取失败不再通过 `.ok()` 变成未配置。Claude 保留 file-first，Codex/Grok 保留 registry-first 的 recorded intent 规则。

| 状态 | 分类及退出码 |
| --- | --- |
| 合法 API-key / subscription / session 配置 | 成功，0 |
| 普通 disabled profile，未被声明为 current | warning，0 |
| 已存在空文档或合法配置无 current marker | inactive warning，0 |
| Profile 缺失 | missing warning，0 |
| Profile 文档损坏 | corrupt，14 |
| Profile 或已有 runtime 文件不可读取 | unreadable，51 |
| 领域 invalid 或 disabled current | invalid，90 |
| Current 目标不存在 | invalid，62 |
| 已有 runtime JSON / TOML 损坏 | corrupt，40 / 41 |

多个错误时，按 registry、Claude、Codex、Grok 的确定性顺序使用第一项错误码。Warning-only 不改变成功退出策略。

### Doctor 平台能力和安全详情

Doctor 默认范围加入已配置 Grok，并支持 `--platform grok`。Clap 平台 choices 从 `Platform::all()` 派生；`supports_auth_profile` / `diagnostic_support` 明确 supported、legacy_adapter 和 not_implemented 状态。Gemini/Droid adapter 保留，未新增 legacy writer 调用。

Grok settings 使用 TOML 检查，runtime 复用已有 activation inspection。官方会话检查明确离线诊断无法证明远端登录有效；测试没有访问真实账户。

合成 secret sentinel 反例发现，损坏 JSON 的 parser detail 可能进入旧 Doctor auth-source 错误输出。已在相关 registry、profile、auth source、snapshot、conflict scan 错误路径使用安全错误码详情；JSON/TOML parser 详情不输出源文本；Codex auth failure 仅使用安全 status。最终回归同时断言 stdout 与 stderr 不含 sentinel。

## 需求与验收证据

| 验收 | 实现与已执行证据 | 边界 |
| --- | --- | --- |
| AC1 / R1 | 真实 `ccr` binary 断言分类及退出码；3 平台 × 2 认证模式 × 6 状态，共 36 组；已有 runtime corrupt/unreadable 另覆盖 3 平台 × 2 状态；disabled current 对 3 平台断言 90。 | Windows 原生执行。 |
| AC2 / R2 | 6 组合法 API-key/subscription fixture 先执行真实 `ccr <platform> profile switch main`，再执行 validate；分类区分 missing、corrupt、unreadable；Windows `share_mode(0)` 持有文件句柄时，对 3 平台验证读取失败为 51。 | Windows denied-share 验证的是原生读取拒绝；未宣称覆盖 Windows DACL。Unix chmod fixture 已添加但未执行。 |
| AC3 / R3 | 每次诊断前后比较路径集合、文件 bytes、mtime 和目录，断言不创建 lock、backup、log；Doctor default/explicit Grok 及 Gemini/Droid legacy_adapter 标签有 binary 回归。中英文命令文档和能力说明已同步。 | 未验证 Linux/macOS、真实远端登录、网络探测或原生桌面。 |

验收源文件：`crates/ccr/tests/diagnostics_contract.rs`。主要入口分别位于 `validate_binary_six_state_matrix_is_read_only`、`validate_binary_applied_api_and_subscription_profiles_follow_domain_rules`、`validate_binary_native_denied_read_is_distinct_from_missing`、`doctor_binary_default_scope_includes_grok_and_preserves_simplified_profiles` 和 `doctor_binary_grok_is_supported_and_legacy_adapters_are_explicit`。

## 最终验证

下面结果来自最终顺序验证脚本 `research/verify_diagnostics.py`，已完成且退出 0。机器可读结果为 `verification-results.json`，所有日志位于本任务目录。

| 命令 | 结果 | 日志 |
| --- | --- | --- |
| `cargo test --offline --locked -p ccr --test diagnostics_contract` | 9 passed；exit 0 | `binary-verified.log` |
| `cargo test --offline --locked -p ccr --test commands doctor` | 16 passed；exit 0 | `doctor-binary-verified.log` |
| `cargo test --offline --locked -p ccr --test commands validate` | 2 passed；exit 0 | `validate-binary-verified.log` |
| `cargo test --offline --locked -p ccr-cli validate` | 5 passed；exit 0 | `validate-unit-verified.log` |
| `cargo test --offline --locked -p ccr-config validator` | 6 passed；exit 0 | `validator-unit-verified.log` |
| `cargo clippy --offline --locked -p ccr -p ccr-cli -p ccr-config --all-targets --all-features -- -D warnings -D clippy::unwrap_used` | exit 0 | `clippy-verified.log` |
| `rustfmt --check --config skip_children=true`，精确覆盖下列 12 个 Rust 文件 | exit 0；会话内执行，未单独保存日志 | 会话工具结果 |
| `git diff --check -- <下列 19 个 T04 路径>` | exit 0；有 Git LF/CRLF 提示，无 whitespace error | `scoped-diff-check.log` |

`git diff --check` 检查 tracked diff。三个新增文件也记录于 `final-source-evidence.json`；新增 Rust 文件包含在上述 rustfmt 和测试覆盖内。

实施末期一次全局 `git diff --check` 曾退出 2，报告 T11 生成窗口中的 `DailyTrendDto.ts`、`InsightsDay.ts`、`InsightsTally.ts`、`InsightsTrendSeries.ts` trailing whitespace。该结果是当时工作区快照，不是当前 T11 完成状态；本代理未修改这些生成物。T04 范围检查通过不能代替 T10 全局门禁。

## 反例、修复与依赖取舍

- `baseline-native.log` 保存初始四项失败：Doctor Grok 未被接受、disabled current 返回成功、合法 subscription 被 generic API-key validator 拒绝等。
- 旧 Doctor 回归第一次出现 14 passed / 2 failed，失败断言仍要求仅 Claude/Codex scope。更新预期加入 Grok 后，最终 16 项通过，见 `doctor-binary-regression.log` 与 `doctor-binary-verified.log`。
- `binary-final.log` 保存 secret sentinel 回归的 8 passed / 1 failed 红例；`secret-regression.log` 保存定点修复后的通过结果；最后 `binary-verified.log` 的 9 项全部通过。
- 规划使用 `assert_cmd` 示例。新增依赖尝试受 Cargo Schannel TLS 下载错误阻断，证据保留在 `baseline-tests.log`、`baseline-offline.log`、`baseline-locked.log`。主代理明确授权改用现有 `std::process` 真实 binary harness，继续保留隔离 HOME/CCR_ROOT、真实进程退出码和副作用断言。
- 新增依赖尝试已撤回；本任务未留下 `crates/ccr/Cargo.toml` 或 `Cargo.lock` 改动。最终全部 Cargo 验证使用 `--offline --locked`。
- Cargo.lock SHA256：`ca1f03e85b5e0d888109ac8af7e7943fc1ceecd20a0f6eeb6a60052c5155db79`。

## 变更文件

### 产品与测试：12 个 Rust 文件

1. `crates/ccr/src/main.rs`：binary 退出码和诊断初始化顺序。
2. `crates/ccr-cli/src/commands/lifecycle/validate.rs`：typed report 终端 adapter。
3. `crates/ccr-cli/src/commands/lifecycle/mod.rs`：report command 导出。
4. `crates/ccr-cli/src/commands/mod.rs`：report command 导出。
5. `crates/ccr-cli/src/commands/doctor_cmd.rs`：typed Doctor command 与共享 choices。
6. `crates/ccr-cli/src/services/validate_service.rs`：导出新诊断入口，保留兼容 API。
7. `crates/ccr-cli/src/services/validate_service/diagnostics.rs`：新增分类、报告和只读诊断用例。
8. `crates/ccr-cli/src/services/doctor_service.rs`：Grok、能力、marker 错误传播及安全错误详情。
9. `crates/ccr-config/src/managers/config_validator.rs`：领域 validator 注入入口。
10. `crates/ccr-config/src/models/platform.rs`：能力状态与静态平台名称。
11. `crates/ccr/tests/diagnostics_contract.rs`：新增真实 binary 回归矩阵。
12. `crates/ccr/tests/commands/doctor.rs`：两项默认 scope 断言加入 Grok。

### 命令文档与规范：7 个文件

- `docs/reference/commands/validate.md`。
- `docs/en/reference/commands/validate.md`。
- `docs/reference/commands/doctor.md`。
- `docs/en/reference/commands/doctor.md`。
- `.trellis/spec/ccr-cli/backend/diagnostics-contract.md`：新增诊断契约。
- `.trellis/spec/ccr-cli/backend/index.md`：规范索引。
- `.trellis/spec/ccr/backend/backend-guidelines.md`：binary 边界约定。

上述共享文件可能同时包含其他已授权任务的改动；文件清单不表示完整文件内容均由 T04 生成。另有本任务目录中的研究脚本、日志和证据文件，供独立检查复现与追溯。

## 剩余检查与交接

- 等待独立 checker 审查 T04 的实现、规范一致性和证据。
- Windows 临时目录与合成账户测试已运行。Linux/macOS、Unix chmod permission fixture、Windows DACL、真实账户与在线探测未执行。
- 文档构建、全工作区 `just ci`、生成 IPC 一致性和其他跨模块集成由 T10/主代理执行；本报告不预先标记这些门禁通过。
- 本代理未运行 `export_bindings`、normalizer 或全量 Tauri test，也未修改 T11 生成窗口产物。
- T04 生产写入已冻结。后续缺陷由主代理安排定点返修；任务生命周期和父任务台账由主代理维护。
