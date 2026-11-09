# T10 本轮证据映射独立审查

结论：本轮证据一致性审查通过，未保留未解决的映射问题。完整工程验收仍开放。

完成时间：2026-09-29 UTC。审查人：/root/implement_t10_specs。

## 范围与方法

审查范围包含三个证据脚本本轮语义、生成后的 AC/P1 矩阵、父任务当前报告与最终文件状态，以及后续 codex_auth.rs:22 的单行测试导入修正。本人此前编写的规范和旧矩阵不计入本次独立审查。

三个未跟踪证据脚本在本轮修改前没有保存副本或 SHA。审查依据为已保存的观察样本、当前脚本、生成前四份产物及最终输入输出。没有逐行 before/after 源码对比的完整依据。

本轮未运行 Cargo、Bun、浏览器或作者生成器/校验器。检查包含 Python AST 解析、独立只读断言、源码/原始日志/元数据 SHA、选择器与行号、数学计数、文件集合和任务状态比对。仅写本报告及同名 JSON。

## 最终核验

| 项目 | 结果 |
| --- | --- |
| AC / 测试证据组 / P1 | 39 / 42 / 9 |
| 本轮运行记录 / 当前唯一证据产物 | 41 / 227，全部指纹一致 |
| 历史原始日志 / 原 P1 baseline | 26 / 20，指纹保持 |
| 生成类型 | 230 个路径和字节均与成功 guard 快照一致 |
| 最终源码清单 | 263 个路径 = 261 个现存文件 + 2 个删除标记 |
| 保护与生命周期 | 两脚本原字节、12 组架构任务状态、2 个 Insights 状态和 HEAD 均保持 |
| 修复提交 | fix_commit = null |

42 个证据组由旧 33 组和本轮 9 组构成。最后增加的 Tauri gateway 与 Windows pending-failure 组映射实际平台运行，没有因此新增产品测试。独立核验结果与作者的 39/42/9/41/227 结果一致。七个作者公布的脚本/产物 SHA 均与最终文件相同。完整指纹和可复现的只读校验源码保存在 JSON。

逐条检查了 AC 的原始需求文本、组或 artifact 引用、Markdown 行、P1 组内容、原始红例、null fix_commit 及当前源码选择器。父 integration-report、continuation-integration-report、implementation-progress、execution-ledger 与 final-delivery-verification 的当前结论一致。进度日志中的旧 900 测试、33 组及审计失败保留为历史检查点；当前入口使用 901 前端测试和 42 组。

## 已解决的审查问题

| 编号 | 问题与最终处理 |
| --- | --- |
| EVIDENCE-01 | 本轮正式 frontend 失败原先缺少直接运行链接。最终矩阵加入 formal_frontend_gate 和 T10.AC4 原始 metadata/log，旧失败也保留。 |
| EVIDENCE-02 | Windows MSRV 使用单数 environment_override。生成器已兼容，最终值保持 CCR_SKIP_ICON_GENERATION=1；命令前后快照时间顺序和 230 个文件字节均可核验。 |
| EVIDENCE-03 | 新 freeze 继承的 changes_since_previous 只列 3 项，实际为 4 项。另附 provenance correction 明确更正；被运行引用的 fa79 快照原字节保持。 |
| EVIDENCE-04 | 父报告已限定为项目 features/MSRV 声明，并将失败进程描述收紧为未观察到测试计数 banner；没有推定崩溃原因。 |
| EVIDENCE-05 | Linux 日志聚合计数和完整名称提取数不同。最终分别记录 OAuth 20/18、process 8/5，保留名称缺损说明，不猜补缺失选择器。两次零匹配运行排除行为验收。 |

## 源码与执行来源

当前源码 SHA 用于定位选择器。旧指纹保留于 prior_source_captures；各运行的 source_attribution、source_binding 和快照作用范围单列。刷新源码指纹不会使历史日志获得新源码执行证明。原历史日志与 baseline 没有被静默改写。

原 a531 冻结清单与当前 fa79 清单均保留。实际差异仅为 codex_auth.rs。独立反向替换 cfg 一行可精确恢复 before 原字节；Mutex 的两处使用位于同一个 Windows 专用测试，生产路径不变。

子清单 files、parent SHA 及 changes_from_parent 正确。provenance correction 准确列出相对继承 previous 的 4 项增量，以及相对显式 parent 的 1 项增量。旧运行保持原来源，当前运行使用 fa79。导入修正前 Linux 8 项通过并含 unused-import 告警；该记录没有被改称测试失败。

修正后的 Linux Tauri strict Clippy、process、OAuth，Windows MSRV 与精确 pending 测试及 fmt 均有独立记录。Linux OAuth 汇总为 20 通过，18 个完整名称可提取；process 汇总为 8 通过，5 个完整名称可提取。Windows 精确用例为 1/1。零匹配的两次命令虽然退出 0，均没有进入行为证据组。

## 失败与验收边界

- 最新根 just ci 保持 exit 1。CLI export_bindings 返回 0xc0000005 / STATUS_ACCESS_VIOLATION，未观察到失败进程的 running 测试计数 banner，原因未查明。此前 Tauri 行为 407 通过、1 忽略，guard 2 通过；该次 aggregate 未到达 frontend。
- 后续独立完整 Tauri 与 bindings 检查通过。407 行为、2 guard、24/9/197 导出及 inventory 分开记录，inventory 与行为集重叠，零用例 target 不累计。失败后 exe SHA 观测不能证明失败时 exe 的身份。
- 本轮独立正式 frontend-check 仍为 exit 1，两个受保护脚本共 5 条 no-console。局部 ESLint、完整前端测试、构建和旧隔离工作区结果均未替代该正式失败。
- UI production build 真实命令为 exit 0；后续 GBK 控制台打印失败的 capture wrapper 为 exit 1。两者分别记录。
- 必需 macOS 原生进程清理矩阵、native WebView/CSP 和完整 pinned Linux lane 尚未执行。合成 IPC 的 7 个 Web receipt 与真实磁盘 helper 证据分层记录。
- root 再次派发 trellis-check/continuation_check 仍返回 agent thread limit reached，记录明确 agent_created=false。本报告属于已有代理的限定范围非作者交叉审查。

台账的 local_verification_complete_acceptance_blocked 描述本轮记录状态。所有架构 task.json 仍为 in_progress，未提交、推送或归档。本报告不授予完整 CI、原生端到端、跨平台或发布验收。
