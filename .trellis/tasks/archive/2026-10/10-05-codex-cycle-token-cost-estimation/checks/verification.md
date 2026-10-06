# 验证记录

日期：2026-10-06。任务状态：功能实施、独立合同检查、合成原生检查和完整 CI 全部完成；AC1–AC12 全部通过。用户随后要求拆分提交并归档。

已实现 Token 分类、逐请求 API 等值费用、有限 quota 观测、5h/7d 经验容量、双语后台 TUI 展示及质量降级。独立检查没有未修复的已确认产品缺陷。最终 `just-ci-06.log` 覆盖领域 272 PASS/2 个既有 benchmark ignored、类型 47 PASS、TUI 247 PASS 和 216 组实际 composition；Windows 合成原生两个尺寸均退出 0。

第五轮完整 `just ci` 退出 1，停在 Frontend Audit：当时既有 postcss-selector-parser 7.1.4 和 source-map-js 1.2.1 被当前公告拒绝，锁文件与 HEAD 相同。用户批准后，仅更新两份 JS 锁文件的 3 条目；冻结安装、安装版本/integrity 和独立范围检查 PASS，例外数量保持 1/1。第六轮完整 `just ci` 退出 0，16 项步骤全部通过，用时 12:30.169。实施与独立记录分别见 `dependency-recovery.md` 和 `dependency-recovery-review.md`；首次失败日志保持原值。

## 最终源码的检查

| 命令 / 检查 | 结果 | 日志 / 边界 |
| --- | --- | --- |
| `just ci` | PASS，exit0，12:30.169 | `just-ci-06.log`；16 项步骤全部通过 |
| 版本、格式、OMP、harness、Copilot、strict Clippy、workspace check | PASS | 第六轮完整门槛；harness为静态资产检查，不推断客户端native loading |
| `cargo test --workspace --all-features -- --skip export_bindings` | PASS | 第六轮；ccr-codex272/2 ignored、ccr-types47、ccr-tui247、ccr-cli345；均为0 FAIL |
| 最后parser focused | PASS，26/1 ignored | `final-parser-retest.log`；后续全工作区已再次覆盖 |
| 实际主界面composition | PASS，216组 | 6尺寸×EN/ZH×18状态；TestBackend，覆盖Standard/assumed/Fast/partial/Unpriced/tiny/stale/各N/A、catalog/mixed和独立USD |
| Release Build、安全审计、CI治理 | PASS | 第六轮；Rust审计保留9项allowed warnings，未新增例外或降低政策 |
| `just tauri-ci`及独立binding generation/check | PASS | 第六轮；407 PASS、1既有子进程helper ignored、197 bindings从普通测试过滤；llmusage无crate guard2 PASS，绑定专门检查PASS |
| `just frontend-check` | PASS | 第六轮；169 test files、904 tests，i18n24项、类型/lint/边界、前端与docs构建及审计PASS |
| `just frontend-audit` | PASS，exit0 | 第六轮；批准的3条目恢复，1 reported advisory、1/1 active exceptions；清单、allowlist和阈值未改 |
| `just frontend-coverage` | PASS，exit0 | 第六轮；statements72.92%、branches59.46%、functions71.89%、lines75.20%，既有阈值保持原值 |
| `just vscode-ci` | PASS，exit0 | 第六轮；lint/build、8项package检查+51项单元测试、VSIX检查；npm ci保留1 moderate warning，不表示依赖零公告 |
| Windows observation ACL / 原子替换失败 | PASS | 领域全包执行原生Get-Acl和deny-delete sharing fixture；旧完整文件及私有权限保留 |
| Windows原生终端 | PASS | `native-verification.md`、`native-pty-output.log`；140×40、100×22、10.2M/100/$5.15、Ctrl+L/r/q、退出0 |
| 独立合同检查、spec写回 | PASS | `independent-contract-review.md`、两包backend spec；完整门槛通过后AC1–AC12完成 |
| task context / scoped diff | PASS | `task.py validate`：implement/check各9项；`git -c core.safecrlf=false diff --check`无错误；批准补丁反向零context check PASS。正式产品命令未因证据文档修改重复扩大 |

两项领域ignored为既有session inventory和rolling cache benchmarks，未计为功能PASS；既有ignored doctests和Tauri子进程helper也保持原值。费用、Token、经验容量的合成算例已经由真实领域计价路径验证：100个短请求为10.2M/$5.15 Standard或$10.30 Fast；300个计价请求的3段dp10pp得到容量中值10M/$5、经验范围8M–12M/$4–$6、50%剩余5M/$2.50。该证据没有使用真实账户。

## 已运行检查

| 检查 | 已有证据 | 边界 |
| --- | --- | --- |
| 启动前 `just version-check` / `just fmt-check` | PASS | 原始基线，不代表最终源码 |
| `cargo check -p ccr-codex -p ccr-tui --all-features` | PASS | 领域/TUI 初版源码 |
| 领域恢复阶段 ccr-codex 全特性测试 | 225 PASS，1 ignored | 合成数据；后续合同修正仍需重跑 |
| 领域恢复阶段 ccr-types 全特性测试 | 47 PASS | 后续价格回归仍需核对 |
| TUI 恢复阶段 ccr-tui 全特性测试 | 245 PASS | 合成 TestBackend；后续布局修正仍需重跑 |
| `native-smoke.ps1` PowerShell 语法解析 | PASS | 帮助脚本语法，不代表原生运行 |

## 首次失败及修正

初版 `cargo test -p ccr-tui --all-features -- --skip export_bindings`：241 PASS，4 FAIL。

- `attribution_notes_are_neutral_and_fallback_scope_stays_with_numbers`：fixture 缺少新的范围说明。
- `compact_counts_promote_rounded_units_and_columns_align_in_both_languages`：费用列宽与旧断言不一致。
- `long_cjk_identity_model_and_error_are_clipped_without_hiding_statistics`：fixture 账号改名后快照账号名未同步，进入加载显示。
- `codex_auth_composed_layout_matrix_preserves_scope_quota_and_errors`：新增价格覆盖行后，旧总量出现次数断言失效。

TUI 恢复代理修正上述 fixture 和断言，完整包测试达到 245 PASS。
并发 Cargo 曾导致 Windows `LNK1104`；后续串行运行检查命令恢复成功。
独立审查又发现容量行截断、历史写失败警告丢失和领域边界缺口；上述恢复阶段 PASS 只保留为阶段证据。

- `just-ci-01.log`：版本 PASS，格式 FAIL；新 `lib.rs` / `services/mod.rs` 导出顺序已格式化。
- `just-ci-02.log`：格式及 harness PASS，严格 Clippy FAIL；TUI 条件分支和新增测试 `unwrap` 已修正，未增加 allow。
- `just-ci-03.log`：严格 Clippy、workspace check PASS；测试途中发现 request chunks/completed 合并丢失负证据，主动停止后保留全部日志。
- `just-ci-04.log`：领域 272 PASS、2 ignored；TUI 245 PASS、2 FAIL。旧 quota 标题断言未同步等值费用标题，catalog 断言误匹配价格版本号中的日期。只修正两处断言，金额、容量、来源和阈值要求保持不变；最终 TUI 247 PASS、216 composition PASS、严格 Clippy PASS。第六轮完整门槛再次覆盖最终源码。
- `just-ci-05.log`：前述产品失败均已通过，完整命令停在既有前端依赖审计。Frontend Coverage和VSCode CI随后单独补跑PASS；该两项单独结果不替代正式完整命令。该轮执行时未应用依赖恢复patch或新增审计例外。
- `just-ci-06.log`：批准的3条目恢复完成后，完整16项步骤PASS、退出0；该轮为最终正式验收证据。
- 原生首次启动因TERM=dumb退出74，未进入TUI；只调整测试进程TERM后，两个实际尺寸成功运行并退出0。产品终端能力检查保持原值。

公开协议补核确认顶层 `compacted` 和 `token_usage_record`。parser 已补齐两种格式及 schema 4 重建；turn/thread 累计量不重复收费。非法 usage、重复请求 metadata 冲突与补充字段均有合成回归。最终执行状态以修正后的完整门槛为准。

## 最终门槛

- 完整独立合同检查：PASS；详见 `independent-contract-review.md`，没有未修复的已确认产品缺陷。
- 当前源码领域、价格和 TUI 子系统测试：PASS，最终源码由第六轮工作区测试覆盖。
- `just lint-strict` / `just test`：PASS，均在第六轮正式路径执行，保持默认并行及独立binding导出。
- 完整 `just ci`：第六轮 PASS、退出0、16项步骤全部通过，用时12:30.169；AC12完成。
- 合成隔离原生终端检查：PASS；实际140×40与100×22、10.2M/100/$5.15、Ctrl+L/r/q和退出0，详见 `native-verification.md`。首次TERM=dumb退出74单独保留。
- 真实 OpenAI 账户端到端、个人用量校准、Linux/macOS 和 hosted CI：NOT_RUN。

原生检查已分别记录实际窗口尺寸、显示内容、输入响应和退出状态。第六轮 Release Build 后的 CLI SHA-256 与原生记录一致，详见 `native-verification.md`。
TestBackend PASS 不替代原生检查。用户随后要求拆分提交并归档；未推送、未创建 PR、未发布。

参考sub2api保持既有AGENTS.md未提交状态，未编辑参考项目产品。CCR的ccr-usage SQL投影、llmusage历史、原始JSONL与账号激活历史未迁移或重价。
