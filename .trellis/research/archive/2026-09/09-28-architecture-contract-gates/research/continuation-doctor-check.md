# Doctor 增量独立审查

在本轮 fix.rs 增量中未发现新增代码缺陷。生产启动、超时、进程树回收、管道 reader join 和错误分支顺序保持。新增测试将 fixture 就绪条件与被测 deadline 分开，并单独覆盖生产入口在 fixture 未就绪时的超时。

## 范围与独立性

Reviewer 为 /root/implement_t10_gates，作者为 /root/implement_t09。本轮只审查 crates/ccr-cli/src/commands/codex/fix.rs。源码冻结 SHA 为 9975befad77e14eef324ead6c1ac634819884a7b433f534b22c56304ab9898ce；before 为 2615ff8ef43b62d659b9e1604c56ece8161fd6552d93096b6ea07a751d2b667a。

依据 root 的派发，复用现有非作者代理完成独立检查。未派发新的 trellis-check，未运行 Cargo、UI、真实 provider、bindings 或完整 CI；只写本报告和 JSON。读取的规范为 ccr-codex/backend/codex-app-server-cleanup.md 的 Doctor 调用、deadline、ManagedProcess 回收和输出边界，以及 ccr-cli/backend/diagnostics-contract.md。

## 生产流程

| 位置 | 结论 |
| --- | --- |
| fix.rs:611–618 | capture_doctor 保持原签名，先同步 spawn_doctor，再立即 await capture_doctor_output。生产入口没有等待 start-child、PID 文件或 fixture ready。 |
| fix.rs:620–627 | Command 参数、stdin null、stdout/stderr piped、ManagedProcess::spawn 与 Spawn 错误映射保持。 |
| fix.rs:634–661 | take pipes → 并发 reader → timeout(child.wait()) 顺序保持。正常结束先 await stderr 再 stdout；wait error 先 join stdout/stderr 再返回 Spawn；timeout 先 terminate_tree，保留 cleanup warning，再 join readers 并返回 Timeout。 |
| fix.rs:664–686 | drain_bounded_pipe 的上限、读取和返回代码未改。生产常量、其余报告解析/脱敏/输出路径未改。 |

reviewer 独立比较 before 与 final 快照，在规范化换行后确认上述生产前缀、Command 设置、整个 wait/timeout/terminate/readers 分支以及后续生产代码完全一致。没有仅采纳作者 sequence-proof 的布尔值。共享 process_gateway.rs SHA 仍为 bd8890b9cae7707f9e523f31123a1bb045e1553f0947669e3b55a859f97634f6。

本轮保持已有 cleanup 错误传播方式和 reader join 行为；报告没有将原有行为描述为新增的全局 deadline 或更强回收保证。

## 测试合同

- fix.rs:1176–1195 的原 parent/grandchild 测试保留 Timeout 断言和两个进程最终消失的断言。新增父进程存活、grandchild 尚未创建、释放 start-child、grandchild 存活的前提，然后调用同一生产输出/回收 helper，deadline 为 200ms。
- fix.rs:1199–1217 的新测试保持 start-child 关闭，通过完整生产 capture_doctor 入口执行 200ms deadline，并用 5s 外层上界防止测试无界等待。测试要求 Timeout 且没有 grandchild PID；如果父 PID 已写出，则确认父进程退出。父 PID 尚未写出的分支不提供按 PID 独立核对退出的证据。
- fix.rs:1285–1338 的 Windows/Unix fixture 仅新增显式启动屏障。屏障属于测试脚本，生产函数不读取屏障。Windows Start-Process 仍使用 Hidden。
- fix.rs:1356–1410 的 PID 等待与进程退出检查 helper 未改。没有新增 retry、serial 或忽略测试设置，也没有删除旧的进程消失断言。

原测试把 fixture 准备与动作 deadline 并发执行。受控红例在 fixture 写出 parent-ready 后持续关闭 child-start 屏障；原生产代码与原超时测试主体均未改。红例实际因 grandchild.pid 缺失失败，证明该竞争机制可以触发失败。原宿主为什么迟滞仍未查明，该红例不能证明原宿主故障的完整根因。

## 已有运行证据

下表运行由作者执行；后续 Linux 补充运行由 root 执行。reviewer 核对命令元数据、原始日志 SHA、真实 selector 和 before/after 源码 SHA，没有重新执行。

| 运行 | 结果与边界 |
| --- | --- |
| 原冻结源码的窄测试 | exit 0；1 passed。说明原失败未在该窄场景稳定重现。 |
| 受控准备阻塞红例 | exit 101；0 passed / 1 failed；parent-ready 后缺 grandchild.pid。 |
| 最终源码首次 focused | 元数据 exit 5；可执行程序在 running N tests 前报 0xc0000005 / STATUS_ACCESS_VIOLATION。保留原日志。 |
| --list 与同命令一次 retry | --list exit 0；retry exit 0，3 passed。--list 与 retry 前后 binary SHA 一致。首次失败元数据没有独立 binary SHA，不能据此补造逐字节验证。 |
| 最终 ccr-cli package suite | cargo test -p ccr-cli --all-features -- --skip export_bindings；exit 0。345 单元测试、12 集成测试、1 doctest 通过，另 1 doctest ignored。默认并行；3 个 focused 用例包含在 package suite 中，不重复计为新增测试。 |
| 最终 ccr-cli Clippy | cargo clippy -p ccr-cli --all-targets --all-features -- -D warnings；exit 0。 |

最终 focused retry、package suite、Clippy 的命令前后 source SHA 均匹配冻结源码。Clippy 与 package 日志保留 mbx 输出；报告不宣称零警告。首次可执行程序启动异常的具体原因仍未查明，后续成功不能删除该失败事实。

原 root-continuation-ci.log 的 grandchild.pid 失败保持原 SHA 与原始内容。本报告不将 package suite 通过改写为原 full CI 通过。root 负责新的聚合门禁及最终整合状态。

root 于 2026-09-28T23:43:37.449096Z 记录 continuation-final-source-freeze.json。reviewer 已核对该冻结的 fix.rs SHA 与本报告、实际文件一致。该冻结位于本轮窄测/package/Clippy 之后、root 最终 CI/MSRV 之前；本报告仍使用各窄运行自己的 before/after SHA 作为来源。

## 未验证边界

本轮已有 Windows 执行证据与 Linux Rust 1.98 的 3 个 Doctor 定向用例。Linux 定向结果不替代完整 Linux CI。规划要求的 macOS 原生进程清理矩阵仍未运行，native WebView/CSP 与完整最终 CI 由父任务单独记录。

## Linux Doctor 补充审查

root 在 2026-09-28T23:49:16.318308Z 启动 WSL Ubuntu-24.04 / Rust 1.98.0 定向测试，命令为 cargo test --locked -p ccr-cli --all-features --lib commands::codex::fix::tests::doctor_ -- --nocapture。原始日志记录 exit 0、3 passed / 0 failed。3 个实际通过的 selector 为 doctor_reports_redact_sensitive_values_before_render_or_save、doctor_deadline_applies_before_fixture_is_ready、doctor_timeout_terminates_parent_and_grandchild。

reviewer 独立提取日志中的 unique selector，核对与命令元数据的 passed_selectors 完全一致，并验证日志 SHA、命令前后 fix.rs SHA 与当前源码一致。后两个用例补充了修改后 Unix fixture 的实际执行证据。3 个用例是已有 Doctor 用例在另一平台的运行，不累计为新增唯一用例。

日志路径为 research/continuation-linux-doctor.log，SHA 为 96b6f9da560dcd027b7e1c3d636c0c43375275bfc829da3857bec74e7ca93398。root 冻结文件 SHA 为 a531395a7d9857c1739491af54b8a781c840199a43c5e8e0bd9ad7959ed60f54；测试前后 fix.rs SHA 均为 9975befad77e14eef324ead6c1ac634819884a7b433f534b22c56304ab9898ce。该检查没有把 Windows 初次启动异常或原 full CI 失败记录改为通过。
