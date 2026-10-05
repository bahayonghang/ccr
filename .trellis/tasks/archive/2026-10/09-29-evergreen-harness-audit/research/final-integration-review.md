# 最终集成独立复核

复核日期：2026-09-30 UTC。复核角色：`trellis-check`。基线：`dev` / `fe9d65972376dbb4c332fde9860c1e42ff2f2005`。

结论：T02、T03、T06 的源码复核完成。T02 的 1 个本地检查器缺陷已修复。当前窄验证通过，复核的 10 个源码文件前后 SHA-256 一致。主会话已于 2026-09-30 03:17:38 UTC 启动最终真实 `just ci`，结果待定；主会话跟踪的源快照包含 2509 个文件，排除任务证据和 journal。本报告不认领尚未完成的聚合、托管、原生客户端或 macOS 验收。

## 范围与证据

- 本轮逐项复核 T02 共享说明和检查器、T03 Dependabot 配置及解析器、T06 三平台聚合、叶子检查、工作流、相关性策略和回归测试。
- T01、T04、T05、T08 沿用已独立核验的 [第一轮复核](independent-review.md)。本次未重复完整前端、扩展及 Rust 覆盖率套件。
- 检查命令、退出码、耗时、日志 SHA-256，以及 10 个当前源码的前后 SHA-256 保存在 [final-integration-checks.json](final-integration-checks.json)。原始日志为同目录 `final-review-*.log`。
- T06 实施者的原始失败及修正结果继续保存在 [实施记录](../../09-29-harness-readonly-gates/research/implementation-results.md) 与 `scoped-check-results.json` / `final-scoped-check-results.json`。不覆盖早期失败。

## Findings (fixed)

- 文件：`scripts/quality/check_harness_contracts.py:55`、`:146`；`scripts/quality/test_check_harness_contracts.py:153`。
- 问题：HC005 原先只在包含 `cargo test` 的命令示例中检查 `RUST_TEST_THREADS`。独立示例中的 `export RUST_TEST_THREADS=1` 和 `$env:RUST_TEST_THREADS = '1'` 可以通过检查，随后影响 Rust 测试并行度。
- 修复：提取 `command_examples()`，扫描全部行内和 fenced 命令示例中的环境变量赋值；按不区分大小写匹配 PowerShell 环境变量。`cargo_commands()` 继续从同一结果筛选 Rust 命令，保留既有 `--test-threads` 与 `--skip export_bindings` 检查。
- 回归：新增 Bash、PowerShell 正常及小写、CMD 四种独立赋值形式，并分别注入 `CLAUDE.md` 和 gate skill fixture，共 8 个负例。最终 harness 套件由 20 个测试方法增至 21 个，全部通过。原 T02/T06 的 20 个测试结果保留为修复前记录。

## T03 / T06 核验结果

1. Dependabot 仅把 `/ccr-ui` 和 `/docs` 从 `npm` 改为 `bun`。`/ccr-vscode`、两组 Cargo、Actions、调度与标签未改变。治理检查覆盖精确 Bun pin、三个 JavaScript 目录的生态/锁文件对应关系、缺失与重复映射、冲突锁文件、无效清单。
2. Windows、Linux、macOS 聚合均保留同一组 16 个有序检查。`version-sync` 和 `fmt` 移出验证路径；新增 OMP、harness、Copilot 和 frontend audit。每种聚合遇首个失败后停止并保留退出码。
3. `audit` 检查两个独立 Cargo lockfile。三个工具前置检查仅报告安装命令并失败，不安装全局工具。现有 9 条 Tauri warning 保留。
4. `frontend-build` 的 recipe 级环境变量覆盖 frozen install 与 build 两次 Bun 调用。真实 `just` 临时夹具以外部值 `CCR_SKIP_ICON_GENERATION=0` 启动，确认两次调用收到 `1`。
5. 扩展链路为 `vscode-ci -> ccr-vscode/justfile ci -> build -> npm run package -> check:vsix`。最终 VSIX 在生成后验证。遗漏、提前检查与缩短调用链均有负例。
6. root hosted quality 安装固定 Bun 1.4.0 与 Node 24.20.0，并执行三项工具契约门槛；hosted audit 检查两个 Cargo lock。相关性策略覆盖新增检查器、规则入口、OMP 源码、工具说明和相关清单/锁文件。
7. 四个 required context 未改变：`Root Workspace Required`、`Vue and Docs Required`、`Tauri Linux Required`、`VS Code Required`。Root/React/VS Code 70% 与 Root/Tauri gateway 85% 门槛未改变。Rust 默认并行、`--skip export_bindings` 和独立绑定生成责任未改变。
8. 共享回写已覆盖 `AGENTS.md`、`CLAUDE.md`、双语 harness 文档、gate skill、`dependency-governance.md`、`test-fixtures.md` 及扩展 surface contracts。相关规则明确适用于五套工具；静态交付、生成的本地集成和客户端实际加载分别记录。

## Verification

| 检查 | 结果 | 边界 |
| --- | --- | --- |
| `just workflow-governance-check` | PASS：48/48；47 个 immutable action 引用；serial-only 计数 0 | 本机实际执行 |
| `just harness-check` | PASS：21/21；6 份共享文件；5 套工具 | 静态契约与合成负例 |
| `actionlint` | PASS，退出码 0 | 工作流语法/静态检查 |
| 五个涉及 Python 文件的 `py_compile` | PASS，退出码 0 | Python 编译检查；不能称为独立静态类型检查 |
| 已跟踪 T03/T06 文件 `git diff --check` | PASS，退出码 0 | 空白检查 |
| 两个新增 harness Python 文件 `git diff --no-index --check` | PASS，无诊断；原始退出码 1 | no-index 返回文件差异码；原始日志保留，不改写成 0 |
| 10 个复核源码的前后 SHA-256 | PASS，漂移列表为空 | 本轮窄验证；全仓库最终聚合由主会话负责 |

Lint 采用本轮 `actionlint` 与第一轮已核验的扩展 `npm run lint`。扩展 runtime/test TypeScript 编译已通过。UI 类型、lint、904 个 smoke 测试、75.19% line coverage（13081/17395）、构建和审计沿用 T08 当前 lock SHA 绑定的已核验凭证。本次未执行 Python 独立静态类型检查，不将 `py_compile` 记成 Python TypeCheck。

48 个治理测试包含三个实际聚合 shell body 的 21 个受控场景、6 个 audit 前置场景和真实 `just` 图标环境传播夹具。本机 Windows recipe 使用 PowerShell；Linux/macOS recipe body 使用本机 WSL Bash。以上场景证明顺序和失败传播，原生 macOS 产品测试仍未执行。

## 验收状态复核

| 任务 | 当前证据支持的状态 | 未关闭部分 |
| --- | --- | --- |
| T01 | OMP 契约与选择源码夹具测试通过 | 原生 ExtensionAPI 类型检查、客户端实际加载/trust 未验证；选择源码夹具包含未提交交付内容，不能称为已提交新检出 |
| T02 | 文档对齐及 21 个回归通过；HC005 已修复 | 五客户端 fresh-session 实际加载独立记录 |
| T03 | AC1/AC2 本地配置与相关检查通过 | AC3 真实 Dependabot 运行保留 UNVERIFIED |
| T04 | 34 个测试、CRLF/BOM/字段错误负例及实际 12 个资产检查通过 | fixture 与实际托管 checkout 的边界继续保留 |
| T05 | allowlist、51 个扩展测试、8 个包检查测试、真实 15-entry VSIX 与两节点兼容补丁通过 | 原生激活、托管发布未验证；未修改 release 发布路径 |
| T06 | 源码、控制流、相关性、阈值及窄验证通过 | 最终真实 `just ci` 与全仓库源哈希由主会话归集 |
| T07 | 当前 Windows 进程、Linux root coverage、Windows Tauri coverage、MSRV 凭证已生成 | 历史 H01/H02 原因未查明；macOS、当前 hosted/fresh-client 证据和 9 条 warning 未关闭 |
| T08 | 四个兼容锁节点、本地 audit 0、类型/lint/smoke/coverage/build 通过 | 固定 hosted runtime 和原生客户端证据未验证 |

父任务 R1–R7 已有任务、实施、说明和证据映射，但父 AC4 要求安全公告或正式门槛未关闭时继续保持未完成。父任务应保留 `in_progress`。复核发现的 T07 元数据滞后已由主会话处理：上下文验证后启动任务，当前为 `in_progress`。本复核没有修改主会话拥有的任务元数据。

## Findings (not fixed)

- `release.yml` 仍直接调用 VSCE；prepublish 只检查源文件清单，未验证发布链路中的最终 ZIP。该文件不在 T05/T06 批准白名单内，属于发布边界决策。保留现有流程并在扩展规范中明确边界。
- OMP 原生类型与加载、五工具 fresh-session/trust、真实 Dependabot/当前 hosted 运行、macOS 产品验证没有本轮执行凭证。不能用静态检查、WSL shell fixture 或本地 UI 构建关闭这些项目。
- 九条 Tauri warning 全部保留 UNRESOLVED。详见 [advisory 独立复核](../../09-29-harness-ci-evidence/research/tauri-advisory-review.md)：七条 unmaintained、两条 unsound。glib 补丁跨当前 GTK 依赖版本范围；rand 已核验的图内未满足已知触发条件，但 warning 仍未关闭。升级与风险接受属于后续决策。
- RustSec 缓存存在未跟踪 `crates/stable-vec/RUSTSEC-0000-0000.md`。九个实际消费文件与固定 revision `f23b768236fe2880e4cfa167da662cad8ca79240` 的 tracked blob 逐字节相同；Tauri lock 与本次输出未消费该残留。不得把整个数据库工作区写成 clean 或全量 commit-bound；未删除或修改缓存残留。
- 本机 Bun 1.4.2 / Node 26.7.0 与 hosted pin 1.4.0 / 24.20.0 有差异。保留实际版本，不把本地通过写成固定 hosted runtime 的执行结果。Windows Tauri coverage 使用 cargo-llvm-cov 0.9.1，Linux正式 root coverage 使用 0.9.0，凭证分别记录。

本轮自修仅涉及批准范围内的两个 Python 检查文件。没有修改生产 Rust/TypeScript 行为、全局账户/trust/模型配置，没有提交、归档、远端触发、推送或发布。
