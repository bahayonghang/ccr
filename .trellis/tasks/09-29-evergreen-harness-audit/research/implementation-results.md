# 批准后实施与验证记录

- 基线：dev / fe9d65972376dbb4c332fde9860c1e42ff2f2005；所有改动仍未提交。
- 授权：八个子任务已批准；扩展两个传递依赖补丁追加批准见 [approval.json](../approval.json)。
- 原始审查：[audit-report.md](audit-report.md)、[check-results.md](check-results.md)。保留原始失败与日志。
- 当前阶段：新增 UI 公告修复及 LF 换行恢复完成；完整 just ci retry1 在第 8 项 Rust Test 失败，只读诊断已定位测试隔离缺口，六行候选等待补充批准。首次第 14 项审计失败与本次失败均保留；扩展两个新补丁目标等待补充批准。

## 已实施内容

| 任务 | 实际结果 | 证据 |
| --- | --- | --- |
| T01 | OMP 注入和缓存覆盖 PRD/design/implement/info；仅交付必要扩展源 | [T01](../../09-29-harness-omp-context/research/implementation-results.md) |
| T02 | 五工具说明、角色权限和默认并行对齐；新增共享契约检查器 | [T02](../../09-29-harness-contract-alignment/research/implementation-results.md) |
| T03 | UI/docs 使用 Bun Dependabot；缺失、重复和错配均受治理检查 | [T03](../../09-29-harness-dependabot-bun/research/implementation-results.md) |
| T04 | Copilot 解析兼容 LF/CRLF/BOM，保留缺字段及格式负例 | [T04](../../09-29-harness-copilot-crlf/research/implementation-results.md) |
| T05 | VSIX 13 源文件、15 ZIP 条目允许范围；扩展仅更新两个传递锁节点 | [T05](../../09-29-harness-vsix-package/research/dependency-patch-results.md) |
| T06 | 三平台同序 16 项只读源码验证；补齐工具、安全和包内容入口 | [T06](../../09-29-harness-readonly-gates/research/implementation-results.md) |
| T07 | 历史失败复验、覆盖率/MSRV、九条告警诊断；缺失环境仍未完成 | [T07](../../09-29-harness-ci-evidence/research/implementation-results.md) |
| T08 | 前端累计更新七个兼容锁节点；最新独立 audit 为 0，例外上限仍为 0 | [T08](../../09-29-harness-frontend-security/research/implementation-results.md) |

## 当前验证

- 首轮独立审查：T01/T04/T05/T08 无阻断缺陷；独立复跑 56/56 通过。见 [独立审查](independent-review.md)。
- T02 共享契约经独立自修后 21/21；最终治理组合 48/48（包含 T03 与 T06）；扩展既有 51/51 与新增 8/8；前端 169 文件、904 项 smoke 通过。
- 早期窄安全检查：前端 0 公告、0/0 例外；扩展 npm audit 0 vulnerabilities。后续完整 CI 的前端 audit 新报告 7 条命中，早期通过只代表当时快照。Tauri 的 9 条 warning 单列，未添加 ignore。
- Tauri 告警已完成依赖图诊断：[9 条告警、目标与处置](../../09-29-harness-ci-evidence/research/tauri-advisory-review.md)。glib 仅 Linux 普通图可达；rand 当前构建图未启用公告所需 log feature。所有告警仍为 UNRESOLVED，静态图不证明运行时可利用性。
- Linux WSL 完整 Rust coverage：总体 74.60%，gateway 96.09%，原 70%/85% 门槛通过。
- Windows Tauri 完整 coverage：407 passed、1 ignored、197 binding exports filtered，另有 2 个 guard 通过。总体 51.67% 作为报告；gateway 95.59% 通过原 85% 硬门槛。
- Windows doctor 1/1、进程 smoke 17/17；Rust 1.95 全工作区 all-targets/all-features check 通过。
- 前端初轮行覆盖率 75.19%，追加补丁检查为 75.21%；扩展初轮行 91.86%、函数 91.50%，原 70% 门槛通过。各回执输入 SHA 与当前聚合结果分别记录。
- 最终独立复核已完成，HC005 的独立环境变量赋值漏检已自修。见 [最终集成审查](final-integration-review.md)。完整 just ci 已执行并失败，见 [完整回执](final-ci.json)。
- [文件范围核验](implementation-scope.json)：32 个产品、脚本、配置、说明和规范路径全部属于批准白名单。

## 证据与适用边界

- Windows：Rust 1.98.0、cargo-llvm-cov 0.9.1；Linux WSL：Rust 1.98.0、项目局部 cargo-llvm-cov 0.9.0。
- 本机 Bun 1.4.2、Node 26.7.0 与项目/托管的 1.4.0、24.20.0 不同。所有结果保留实际工具版本。
- 当前未提交候选的托管 CI、真实 Dependabot、macOS 进程门槛、五客户端新会话 hook/trust、原生 GUI 和已提交 clean clone 均未验证。
- release.yml 的最终 ZIP 检查未纳入批准白名单；其 prepublish 源清单检查已存在，发布前仍需独立验收。
- 没有提交、归档、推送、PR、release、远程重跑或全局配置变更。

## 持久化回写

共享规则回写 AGENTS.md、CLAUDE.md、双语 docs/agents/harnesses.md 和已跟踪 ccr-gate-recovery skill。依赖治理、测试夹具、扩展打包规范记录可执行检查及五工具适用范围。Trellis 父任务与八个子任务保留实施、失败、复核及未验证状态。

## 完整 CI 新发现

2026-09-30 03:17:38 UTC 启动的真实 just ci 在 745.46 秒后退出 1。前 13 项通过；第 14 项 Frontend Audit 拒绝 7 个命中，后两项 frontend-coverage、vscode-ci 按首错传播规则未执行。它们的早期独立通过凭证保留，不能代替本次完整聚合通过。

- brace-expansion：GHSA-qhr7-859c-m2p7、GHSA-6j4f-fj2g-mc7p、GHSA-q2hr-2g5m-vwhr，各命中两个锁路径。
- fast-uri：GHSA-hrr3-gc8f-f4qj，命中一个锁路径。
- 当前事实：UI lockfile 和全部 2509 个源文件字节未变，但此次 audit 与早期结果不同。4 条官方公告的发布时间为 2026-09-29 23:44:58 至 23:54:25 UTC，均早于初轮通过的本地审计。相同锁文件的审计结果变化原因未查明。
- 构建非阻断警告保留：Vite configLoader 的未来兼容提示（__dirname 与 JSON import attributes）、既有 638.13 kB chunk、MSVC linker_messages。未通过抑制警告改变状态。

当前不认领完整 CI PASS、T08 最终安全闭合或父任务完成。新的兼容补丁范围依据核验证据决定；不添加例外或降低审计级别。

## 新公告兼容补丁方案

官方范围、父依赖 semver 与 3 个候选 tarball SHA-512 已核验。方案见 [机器可读方案](new-advisories-version-proposal.json) 与 [待应用差异](new-advisories-proposed-locks.patch)。

UI 的四个解析节点在 T08 已批准的动态安全修复范围内，正在实施。扩展拟更新 brace-expansion 5.0.9 → 5.0.12 与 fast-uri 3.1.7 → 3.1.8；新增 brace-expansion 超出原两节点追加批准范围，已提交补充确认，尚未据此改动扩展锁文件。docs 无目标节点且审计为 0，不修改 docs 锁文件。

## 最终 UI 候选与聚合复验

最终 LF 锁 SHA-256 为 c4301a8d1909ac7db2f92930e5d91172195d3ad51c84b4e163c16909a2528025。743 个解析节点，追加仅四条解析，反向替换可逐字节还原修改前文件。首次补丁受 core.autocrlf 影响而整体变为 CRLF，原回执保留；纠正后 frozen install、audit、diff 检查均退出 0。

前端完整检查 169 文件 / 904 测试通过；Lines 75.21%（13083/17395），Statements 72.93%（14655/20092），原 70% 门槛不变。这组完整检查使用前一 CRLF 文件；最终 LF 文件仅有换行差异，随后真实 just ci retry1 验证最终源码。详见 [T08 新公告结果](../../09-29-harness-frontend-security/research/new-advisories-implementation-results.md)。

独立方案复核覆盖 22 个 GHSA / 57 个声明范围，拟六节点均满足父依赖范围并通过 tarball SHA-512 核验；扩展部分仅在内存验证，尚未应用。[追加独立审查](new-advisories-independent-review.md)。当前不能认领扩展安全检查已关闭。

## 完整 CI retry1 新失败

2026-09-30 03:49:38 UTC 启动的完整 just ci retry1 在 73.79 秒后退出 1。前 7 项通过；第 8 项 Rust Test 中 managers 为 16 passed / 1 failed。失败测试为 legacy_registry::legacy_registry_fields_load_without_becoming_clean_write_routing_truth，在 crates/ccr/tests/managers/legacy_registry.rs:53 返回 FileLockError（os error 3，锁文件打开时报路径不存在）。已确认 6 个测试经默认写入链读取 CCR_LOCK_DIR 时未持有同一环境 guard。临时锁目录清理竞态是高置信假设，原失败的精确交错未观测。两次定向诊断分别为 1/1 与 17/17 通过；未修改 Rust 文件或测试并行策略。

[重跑回执](final-ci-retry1.json)与[完整日志](final-ci-retry1.log)单独保存。2509 个交付源文件前后 SHA 全部相同。第 9–16 项未执行；本次没有提供最终 LF 文件的完整前端聚合证据。当前不继续盲目重跑以寻求通过。

## 待确认的两个补充范围

1. 扩展兼容补丁：brace-expansion 5.0.9 → 5.0.12，fast-uri 3.1.7 → 3.1.8。未应用。见 [六节点整体方案](new-advisories-proposal.md)；UI 四节点已完成，扩展两节点仍待补充批准。
2. managers 测试隔离：在 general.rs 五个用例与 legacy_registry.rs 一个用例各增加已有 setup_ccr_test_env 命名绑定，共两文件六行。未应用。见 [故障诊断、候选与检查门槛](../../09-29-harness-ci-evidence/research/manager-lock-diagnosis.md)。

两项补充确认均已发送，未收到答复不视为批准。当前 approved source diff 仍限于 32 个原批准路径；任务未提交或归档。UI 换行缺陷已由 [最终 UI 独立复核](new-advisories-final-ui-review.md)关闭。当前无法声明完整 CI 或父任务验收通过。

## 本轮停止点与最终工件核验

[manager 夹具候选独立审查](../../09-29-harness-ci-evidence/research/manager-lock-independent-review.md)确认两文件六行候选覆盖同二进制的 6 个未保护读取者，保留默认并行与现有生产逻辑。候选静态审查通过，尚未取得补充批准，未应用。该报告列明现有 platforms、commands sync_content、managers 专门检查，以及工作区与完整 CI 门槛，后续实施不得省略。

[最终交付范围与工件校验](final-delivery-validation.json)确认：32 个交付路径全部在原批准范围；6 个新增源码文件和已跟踪差异的空白检查通过；9 个任务上下文全部有效；核验时 172 个 JSON 可解析、121 处 Markdown 引用有效。HEAD 未变、暂存区为空；原 Rust 两个候选文件仍无改动。

当前停止在两项新增范围确认。原始完整 CI 与 retry1 都是 FAIL；不以定向通过或候选静态审查替代整体验收。Trellis 父子任务继续保留 in_progress，未提交、归档或推送。
