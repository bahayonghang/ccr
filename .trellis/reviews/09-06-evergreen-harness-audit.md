---
skill: trellis-plan-review
version: 0.5.0
task_dir: D:/Documents/Code/Github/ccr/.trellis/tasks/09-06-evergreen-harness-audit
task_name: 09-06-evergreen-harness-audit
task_status: planning
review_scope: task-tree
task_count: 6
task_members:
  - 09-06-evergreen-harness-audit
  - 09-06-evergreen-ui-smoke
  - 09-06-evergreen-ci-verdict
  - 09-06-evergreen-omp-context
  - 09-06-evergreen-harness-rules
  - 09-06-evergreen-ci-history
task_statuses:
  09-06-evergreen-harness-audit: planning
  09-06-evergreen-ui-smoke: planning
  09-06-evergreen-ci-verdict: planning
  09-06-evergreen-omp-context: planning
  09-06-evergreen-harness-rules: planning
  09-06-evergreen-ci-history: planning
verdict: 可执行
blocking: 0
should_fix: 0
notes: 0
generated_at: 2026-09-06T11:56:00+08:00
---

# Trellis 规划审阅报告

## 审阅范围

- 根任务：09-06-evergreen-harness-audit。
- 模式：task-tree；数量：6。顺序为根优先，仅代表成员归属，不代表执行依赖。
- 有序成员及状态：09-06-evergreen-harness-audit、09-06-evergreen-ui-smoke、09-06-evergreen-ci-verdict、09-06-evergreen-omp-context、09-06-evergreen-harness-rules、09-06-evergreen-ci-history，均为 planning。
- 阅读范围：六成员 task.json、prd.md、design.md、implement.md、implement.jsonl/check.jsonl，父 research/audit.md 与 research/harnesses.md；定向追踪 UI 夹具和 DTO、CI workflow/surface/just recipes、OMP 默认扩展入口和 session key、根与目录级规则及文档要求。
- 审阅代理仅执行只读检查并写本报告；修订由主线程负责。以下结论针对最终修订后的规划，不代表产品修复通过，也不构成用户实施批准。

## 结论

可执行 — 阻断 0 / 应修 0 / 提示 0

可以提交用户批准推荐的四个 P1 批次。P2 仅为独立证据复核，不隐含产品修复。未批准实施前六个任务继续保持 planning。

## 问题清单

无。定向复核发现的 OMP fixture session key、CI audit 输入归属、目录规则责任文件、文档中英镜像和部分批准后的回写边界，已由主线程在本报告写入前修订并复核；不将已解决事项计入剩余发现。

## 未能核实

- 修复后测试结果：尚未实施；UI 原始 2 项失败仍是修复基线。现有测试结果取自父任务本轮运行账本，审阅代理未重复执行 Rust/Tauri/前端/扩展整套测试。父 research/audit.md 已记录命令、失败摘要和临时日志边界。
- 五套 harness 的真实加载、权限、实际模型解析和成本：本轮没有启动这些真实客户端会话，官方文档及仓库文件不能证明运行已对齐。research/harnesses.md 的来源与能力矩阵为本轮研究输入，本审阅没有再次逐页打开官网。
- OMP 新增默认入口测试：设计已能到达真实入口，但测试文件尚不存在；Bun mock 通过后仍不能替代实际 OMP 会话、extension 生效和主/子任务绑定的运行证据。
- 当前 HEAD 的 hosted CI、Linux/macOS、Windows WebView、账户/provider 和发布：未运行对应环境。Root/Tauri 历史失败的底层 IO 原因、资源竞争假设仍未证实，当前本地测试通过不能证明历史问题已修复。
- 完整 just ci、coverage 及必要工具可用性：本轮拆分运行了安全组成检查，未获得 full gate green。后续缺必要条件时仍须标记该门未完成，不能用局部检查替代。

## 可靠部分

### Pass 0 与状态

- 独立运行 `python -X utf8 C:/Users/lyh/.agents/skills/trellis-plan-review/scripts/plan_precheck.py .trellis/tasks/09-06-evergreen-harness-audit --include-descendants`，初审及最终修订复核均 exit 0，6 成员、0 blocking。层级与 backlink 有效，计划及 manifest 齐全；报告目标为 ignored。
- precheck 将子任务“根 R2/R4/R5”等跨任务映射中的 R4/R5 列为 advisory undefined；原文明确限定为根需求，对照根 prd.md:9–15 可解析，不是缺失的子任务需求。
- 所有成员均 planning，Pass 7 实现漂移不适用。审阅时 `git diff --stat` 为空，工作区新增的是本任务规划产物；没有把后续实施文件当作已改。

### AC 子句追溯与机制

| 成员 | 逐条追溯结果 | 机制与验证依据 |
|---|---|---|
| 父任务 | AC1–AC6 对应 R1–R6；AC7 对应 R7，区分当前规划交付与后续批准集验收 | design.md:5、:11–15、:19、:29–31、:39；implement.md:11–19、:23–29。审查账本、白名单、子项检查、工具分工及部分批准后的待回写状态均有机制 |
| UI smoke | AC1–AC3 的双语路由、正确 DTO、完整测试及不弱化断言对应 R1；AC4/AC5 对应 R2/R3 | design.md:5 的域专属 typed fixture、hoisted 限制、finished 终态和单文件边界；implement.md:20–24 的 focused + agent-sessions、type-check、lint、完整 test |
| CI verdict | AC1 的失败/成功/日志、AC2 的精确映射、AC3 的既有治理与覆盖率对应 R1；AC4/AC5 对应 R2/R3 | design.md:5；implement.md:22–26。已有 surface 类补正负用例，失败探针 exit 1 是预期通过；不扩大到任意文件触发所有 surface |
| OMP context | AC1–AC4 的三件套、缺文件、角色 manifest、既有边界、人工补读与真实运行证据分别有机制；AC5/AC6 对应 R2/R3 | design.md:5、:9；implement.md:5–9、:24。测试通过 default extension 注册和 session_start，而非文本包含断言或导出私有 helper |
| Harness rules | AC1–AC4 的事实唯一来源、五工具边界、权限、命令副作用、双语镜像与结果回写对应 R1；AC5/AC6 对应 R2/R3 | design.md:5、:9–13；implement.md:5–27、:42–47。既有规则和说明有完整责任白名单，先事实校正再建立 Claude 共享 import |
| CI history | AC1–AC3 的 SHA/命令/首故障、有限复核、可证伪假设、仅研究文件对应 R1；AC4/AC5 对应 R2/R3 | design.md:5 的最多三次停止条件和 coverage 条件；implement.md:5、:20–25。无复现与缺工具仍可交付调查结论，但不能标 gate 通过或问题已修复 |

### 关键源码核对

- UI 失败链成立：`ccr-ui/tests/shell/route-view-mount.smoke.test.tsx:70` 按 session 名称匹配，:76 返回数组；:81–86 递归替换 API 函数。`ccr-ui/src/types/generated/usage/StartSessionIndexJobResponse.ts:4` 要求 job_id/snapshot，`ccr-ui/src/features/agent-sessions/AgentSessionsView.tsx:216` 读取 snapshot.status，:212–213 以 finished/failed 停止轮询。计划将修复约束在 fixture，而非无证据改变生产容错。
- CI 退出码修复范围准确：`.github/workflows/vscode-ci.yml:49` 的 coverage 管道未指定 shell，计划只加显式 bash。`scripts/ci/ci_surface_policy.py:25` 起的既有集合缺三项输入；`justfile:1553` 消费 tauri-ci.toml，`.github/workflows/ci.yml:148` 在根运行 cargo audit。定向搜索 `cargo audit|cargo-audit|audit.toml` 于 workflow 与根/UI justfile 未发现 Tauri audit 消费者，故最终映射为 tauri-ci→tauri、config→root+tauri、audit→root。
- `just vscode-coverage` 经 `justfile:1656–1657` 到扩展现有 coverage recipe，实际 Node coverage 命令不包含 npm ci；无需为了该项检查安装额外 runner。
- OMP 默认入口可由规划 fixture 到达：`.omp/extensions/trellis/index.ts:35–40` 的 session key 是 `omp_review-fixture`；:186–197 读取对应 sessions JSON，:222–239 消费 current_task/task.json；:455–497 注入主会话/子代理。最终 design.md:9 已使用精确文件名，提供 ctx/ui mock，省略 get_context.py；:251–252 因缺文件直接返回，不启动 Python。
- OMP 角色隔离与边界没有被重定义：扩展 :294–304 分派 implement/check/research manifest，:325 使用 resolveProjectFile，:167–172 保留真实路径信任判断。新增三件套属于同一任务相邻读取；不重构缓存、compaction 或信任根。
- 根/目录事实冲突确有依据：`AGENTS.md:4` 的 Vue/上游 crate 与 `ccr-ui/package.json:83` 的 React、根 `CLAUDE.md:53` 的 CLI+SQLite 相冲突；`ccr-ui/CLAUDE.md:109` 与 `ccr-ui/AGENTS.md:35` 的当前设计真源相冲突。规则白名单同时包含这些入口和导航，不只修根文件。
- 文档与生成说明边界可执行：`docs/AGENTS.md:8`、:21 要求中英镜像；计划提供两份 harness 页且沿用现有内部 agents 文档链接方式。四份 platform-map 的本地定制有声明，不修改上游模板 hash 或用户全局配置。

### 顺序、数量与证据边界

- 6 成员 = 1 根 + 5 子任务；后续建议 4 个 P1 + 1 个独立 P2。UI route 原文件确有 4 项测试，保留 2 项双语路由失败断言。规则计划中的 11 个 Trellis 文件 = 4 Kimi skills + 3 Grok 入口/角色 + 4 platform-map。
- UI、CI、OMP 的独占文件无重叠；规则子项待已批准结果收口，历史复核只写自身研究报告。依赖由文字声明，未把 children 顺序当依赖证明。
- 父 design.md:31 与 implement.md:19 明确：单独批准局部修复不自动授权完整规则改造；未批准回写时父交付仍待最小回写批准。推荐四个 P1 批次已包含回写责任。
- 强模型负责根因、边界和最终审查，低成本执行限固定文件/契约；research/harnesses.md:37、:47 不将 harness 名或 selector 当作实际低价模型证据。真实五工具会话验收保持单列 UNVERIFIED，未被 mock、源码或文档改写降格替代。

## 盲区

代理审阅代理写出的规划不等同于完全独立的第二意见，审阅者与作者仍可能共享盲区。本报告无剩余发现只表示本轮未找到更多问题，不表示计划完备或已经批准。
