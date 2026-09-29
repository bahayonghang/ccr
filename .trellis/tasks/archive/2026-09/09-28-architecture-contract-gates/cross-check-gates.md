# T10 门禁代码独立交叉审查

日期：2026-09-28。审查者：`/root/implement_t10_specs`。被审实现 owner：`/root/implement_t10_gates`。审查者负责规范和证据，未编写本报告审查的门禁、生成事务、日期或文件锁测试修改。

专用 T10 checker 派发因 `agent thread limit reached` 失败。主会话明确授权两个 owner 互审；本报告记录非作者代码审查与独立检查范围，不代表专用 checker 派发成功。

## 结论

在所列 16 个源码、测试和编排文件范围内，未发现阻止本切片交付的未解决问题。当前文件 SHA 与 owner 冻结记录、独立运行日志和差异指纹见 `research/cross-check-gates-evidence.json`。本结论覆盖该切片，不授予全项目最终通过。

## 实际审查

| 范围 | 审查事实与证据 |
| --- | --- |
| 三平台本地 aggregate | 三份 recipe 均复用既有 `tauri-ci`；该 gate 包含严格 desktop Clippy、全部行为 suite、bindings 和 inventory。hosted 名称、矩阵与 coverage 阈值保留。 |
| 失败传播 | 测试从真实 justfile 提取实际 aggregate/desktop recipe，仅替换外部工具叶节点。Windows 控制失败使真实 `just ci` 非零，阻止后续 bindings/frontend；成功控制会到达后续步骤。 |
| 测试与生成边界 | 普通行为与 coverage suite 仍并行，仅跳过 `export_bindings`。export 阶段在专门生成入口串行运行。 |
| 目录事务 | 快照保存根存在状态、嵌套和空目录、文本与二进制字节。直接生成仅成功保留输出；check 所有结果均恢复原目录。异常/非零和恢复异常均不能返回成功。 |
| 比较基线 | check 对当前工作区做规范化比较，随后恢复规范化前调用者字节；不要求已提交到 HEAD。 |
| Windows 锁测试 | 修改仅位于测试。持锁元数据通过已持有句柄读取并 rewind，仍检查竞争失败前后内容及释放后的文件路径内容；生产锁实现与原断言保留。 |
| 本地日历日期 | helper 将本地年月日映射到 UTC 日序号。测试在 UTC、America/Chicago、Asia/Shanghai 子进程运行真实 helper；覆盖 allTime、March、春季和秋季 DST。 |

## 本审查者独立运行

| 命令 | 结果 | 原始证据 |
| --- | --- | --- |
| `python -m unittest scripts.ci.test_check_workflow_governance scripts.ci.test_architecture_contract_gates` | exit 0；27 passed | `research/cross-gates-governance.json` / `.log` |
| `bun.exe run test:smoke -- tests/quality/bindings-transaction.smoke.test.ts tests/usage/usage-date-window.smoke.test.ts` | exit 0；2 files、25 passed | `research/cross-gates-bindings-calendar.json` / `.log` |

绑定测试覆盖 export/normalizer 非零、抛异常、spawn 错误、漂移、恢复异常、根原本不存在、空目录和精确字节恢复。日期测试为真实子进程时区边界；没有放宽日期期望或加入时间容差。

## 复用证据与限制

- Windows native lock 用例采用源码审查并复用 owner 的 1 passed 原始日志 `gates-lock-green-retry.log`，本审查者没有另起 Cargo 重跑。首次修后进程启动错误 `0xc0000005` 原样保留；原因未查明。
- 实际完整 bindings guard 复用 owner 的 `gates-full-bindings-final.json` / `.log`：exit 0，230 → 230，changed/missing/added 均为空。没有将该次执行称作本审查者独立重跑。
- 首次 guard 的 6 文件漂移与后续不复现均保留。原始差异内容未在恢复前记录，原因未查明。成功的后续 guard 不删除旧失败记录。
- Linux/macOS aggregate 仅检查定义，实际故障传播执行为 Windows。没有新运行 macOS、真实原生 WebView、全部 Tauri suite 或完整根 CI；这些结果由主会话最终整合。
- 独立窄检查不替代正式 lint 和父任务全门禁。两个受保护 `.tmp` 文件及原 Insights 状态的保留另由 `spec-validation.json` 验证。
