# T04 独立检查报告

- 日期：2026-09-28。角色：`trellis-check`，任务：`09-28-cli-diagnostics-contract`。
- 结论：**Windows 本任务范围通过**。独立复现并修复 3 项问题。最终没有未解决的 T04 范围问题。全工作区、Tauri、文档构建及跨平台验收仍由 T10 集成。
- HEAD：`34d8a85e0e48b793733835e0304c8ed33940fcee`。修改未提交；未启动、完成、归档任务，也未修改父台账。
- 已按 `trellis-check` 读取任务、适用规范、实现报告、源码和真实 binary fixture；保留 T02/T11 并发改动。

## Findings (fixed)

### 1. 缺少 profiles 时漏检已有 runtime 错误

- File：`crates/ccr-cli/src/services/validate_service/diagnostics.rs:151`、`:309`。
- Issue：`inspect_profiles` 在 profile 文件缺失时提前返回。隔离目录中已有损坏的 Claude settings 仍得到全 warning 和 exit 0。原有测试仅在 profiles 存在时覆盖 runtime 损坏。有效 current 分支还先调用平台 current 解析器，可能让 adapter 的错误映射优先于 runtime 文件分类。
- Fix：平台实例先解析；profile 读取失败时仍检查 runtime 文件。无有效 current、目标不存在或目标禁用时继续进行 runtime 读取/语法检查。有效 current 的 runtime 读取和语法分类先于 operational-current 查询。没有新增文件写入或恢复动作。
- Evidence：`check-regressions-red.log` / `check-regressions-red-final.log` 保留缺 profile + malformed runtime 返回 0 的失败断言；`check-binary-final.log` 的新用例 `validate_binary_runtime_errors_are_independent_of_profile_presence` 覆盖 3 平台 × profiles 缺失/有效 current × runtime corrupt/unreadable，共 12 个状态，全部通过。路径、文件字节、目录和 mtime 前后相同。

### 2. Doctor current 冲突描述没有跟随实际来源

- File：`crates/ccr-cli/src/services/doctor_service.rs:826`。
- Issue：Codex/Grok 的有效 current 已按 registry 优先选择，但冲突 summary 仍写 `from profiles.toml`。有效 registry fixture 的红例显示实际选中 `alternate`，却将来源写成文件。
- Fix：只修正诊断 summary 的来源和冲突另一端名称。Claude 显示文件来源，Codex/Grok 显示 registry 来源。没有改变 current 选择策略。
- Evidence：`check-source-summary-red.log` 保存有效 registry 的旧失败；`doctor_binary_current_conflict_names_the_authoritative_source` 对 3 平台检查来源文字和文件不变。最终通过。早期 `check-regressions-red*.log` 的该项测试曾使用错误的嵌套 registry fixture；有效红例以 `check-source-summary-red.log` 为准。

### 3. Validate 的 Claude current 候选未遵守 T01 回退契约

- File：`crates/ccr-cli/src/services/validate_service/diagnostics.rs:179`。
- Issue：直接使用 `file_current.or(registry_current)`，使悬空文件标记遮挡有效 registry 标记；领域 current 与 Doctor 使用有效文件候选 → 有效 registry 候选。独立真实 binary 红例为 exit 62，合法回退期望 exit 0。
- Decision：主代理核对 T01 `profile-repository.md:43` 及 Claude 已有回归后，明确要求保持既定有效候选优先级。没有新增更严格的公开诊断政策。
- Fix：调用共享 `base::resolve_file_current_profile`，按同源候选解析。有效候选伴随悬空或冲突旧标记时报告 `warning/invalid`，不修复标记。所有显式候选都不存在时保留 error 62；没有标记时保持 inactive；被选中的现有 profile 禁用时保留 error 90，不能跳过该项选择另一个启用项。Codex/Grok 未扩展规则。
- Evidence：`check-claude-marker-red.log` 保存旧失败；`validate_binary_claude_current_uses_valid_marker_precedence_without_repair` 覆盖有效 registry 回退、有效文件覆盖悬空 registry、两个有效标记冲突、两个标记均悬空、文件优先候选禁用，共 5 种状态，全部通过且 fixture 不变。
- Sync：更新 `.trellis/spec/ccr-cli/backend/diagnostics-contract.md` 和中英文 `validate.md`，同时说明 runtime 检查独立于 profile 存在。

## Findings (not fixed)

本次审查没有剩余的 T04 范围代码问题。以下为验收边界，没有预先标记通过：

- Unix chmod permission fixture、Linux/macOS binary 行为未在本次独立检查中执行。
- Windows 不可读取 fixture 使用 `OpenOptionsExt::share_mode(0)`；该证据不等同于 DACL 拒绝读取验收。
- 真实账户、真实远端认证、在线 Provider 网络请求及 native UI 不在本次本地诊断验收中。Doctor 单元测试使用受控 probe，不能证明真实在线探测。
- 全工作区 `just ci`、Tauri aggregate、文档构建及生成一致性由 T10 检查。范围内 Clippy/格式通过不替代这些门禁。

## Verification

所有 Cargo 命令使用 `--offline --locked`。行为测试使用 `--skip export_bindings`；没有导出或覆盖 TypeScript 生成物。原有 Doctor/Validate 测试进程移除继承的 `GROK_HOME`、`CODEX_HOME`、`CCR_DATA_DIR`、`CCR_CONFIG_PATH`；测试内部 HOME、USERPROFILE 和各平台路径指向临时目录。

| 检查 | 结果 | 原始证据 |
| --- | --- | --- |
| 原实现新增 binary suite 独立基线 | 9 passed | `check-binary.log` |
| 最终 `cargo test -p ccr --test diagnostics_contract` | **12 passed** | `check-binary-final.log` |
| `cargo test -p ccr --test commands doctor` | **16 passed** | `check-doctor-binary.log` |
| `cargo test -p ccr --test commands validate` | **2 passed** | `check-validate-binary.log` |
| `cargo test -p ccr-cli doctor` | **14 passed** | `check-doctor-service.log` |
| 最终 `cargo test -p ccr-cli validate` | **5 passed** | `check-validate-service-final.log` |
| `cargo test -p ccr-config validator` | **6 passed** | `check-config-validator.log` |
| 最终三包 `cargo check --all-targets --all-features` | **pass / exit 0** | `check-rust-types-final.log` |
| 最终三包 `cargo clippy --all-targets --all-features -- -D warnings -D clippy::unwrap_used` | **pass / exit 0** | `check-strict-clippy-final.log` |
| 精确 12 个 Rust 文件 `rustfmt --check --edition 2024 --config skip_children=true` | **pass / exit 0** | `check-rustfmt.log` |
| 精确 19 个 T04 文件 `git diff --check` | **pass / exit 0** | `check-scoped-diff.log` |

三包为 `ccr`、`ccr-cli`、`ccr-config`。完整命令和耗时保存在 `check-results.json`、`check-results-final.json`；本报告按命令列出用例数，不将潜在重叠过滤结果合计。

首次两个修复完成后运行 Doctor/Validate/validator 回归及三包检查。随后仅修改 Claude 诊断候选逻辑，重新运行最终 12 项 binary、validate service、三包类型检查和严格 Clippy。格式检查覆盖最终文件。

## AC 追溯

| AC | 独立结论 | 证据 |
| --- | --- | --- |
| AC1 / R1 | Windows 范围通过 | 3 平台 × 2 auth 模式 × valid/warning/invalid/corrupt/unreadable/missing = 36 组；invalid 90、profile corrupt 14、read error 51；warning-only 0；disabled current 90。新增缺 profile/current runtime 错误 12 组；错误返回非零。 |
| AC2 / R2 | Windows 范围通过 | 6 次真实 `ccr <platform> profile switch main` 成功后再 validate；领域 validator 同源；3 平台原生 share-denial 与 missing 区分；Claude current 候选 5 状态符合 T01。 |
| AC3 / R3 | Windows 范围通过 | fixture 路径、bytes、mtime、目录前后相同；Doctor default/explicit Grok；Gemini/Droid legacy_adapter 标签；source-conflict 3 平台；无 process::exit 的可嵌入诊断入口；binary 在日志初始化前返回 ExitCode。 |

合成 sentinel 测试覆盖 profile 文档、registry 和 runtime settings 的错误路径；stdout、stderr 和 Doctor JSON 不含 sentinel。源码追踪确认 `DiagnosticReport` 只保存固定安全描述，未保存原始 parser error/source；Doctor 相关 parser/auth-source错误使用安全详情。没有读取真实用户账户。

## 变更和源快照

本次 checker 只改变 6 个产品/回归/规范/文档文件：

1. `crates/ccr-cli/src/services/validate_service/diagnostics.rs`。
2. `crates/ccr-cli/src/services/doctor_service.rs`。
3. `crates/ccr/tests/diagnostics_contract.rs`。
4. `.trellis/spec/ccr-cli/backend/diagnostics-contract.md`。
5. `docs/reference/commands/validate.md`。
6. `docs/en/reference/commands/validate.md`。

另新增本任务中的 checker runner、报告和验证日志。未修改 T02 core/repository/application、T11 runtime/OAuth/生成物，未修改依赖。

`check-source-evidence.json` 记录最终 19 个 T04 文件 SHA256、HEAD、范围格式结果。Cargo.lock SHA256 保持 `ca1f03e85b5e0d888109ac8af7e7943fc1ceecd20a0f6eeb6a60052c5155db79`。本报告的最终源快照和验证结果替代 implementation-report 中对应的检查前状态。没有伪造修复 commit。
