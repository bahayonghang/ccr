# T10 Doctor 进程树测试修复记录

日期：2026-09-28。Owner：`/root/implement_t09`。本切片仅修改 `crates/ccr-cli/src/commands/codex/fix.rs`，其余改动为本目录 `continuation-doctor-*` 证据。已使用 `trellis-before-dev`、`ccr-gate-recovery` 和 `diagnosing-bugs`，读取 CLI fixture、diagnostics 与 managed-process 规范。

## 结论与证据边界

已确认旧测试将 fixture PID 就绪和被测执行 deadline 同时启动。在受控条件下，父进程等待启动许可时被 deadline 终止，随后测试因 `grandchild.pid` 缺失失败。修复将 fixture 初始化与 timeout 清理断言分开，并增加实际生产入口的 deadline 回归。

原始完整 CI 的失败日志记录 `343 passed; 1 failed`，错误位于 `root-continuation-ci.log:870-878`。原始宿主上孙进程迟滞的具体原因未查明。原版单项测试通过；加入诊断后的默认并行包级套件通过。受控红例证明测试的 ready/deadline 竞争机制，不能单独证明原始宿主延迟来源。

原始日志完整副本为 `continuation-doctor-original-ci.log`，原路径与 SHA 见 `continuation-doctor-before.json`。原日志、临时诊断版、受控红例和最终复验分别保存，没有互相覆盖。

## 诊断过程

1. 原版单项测试：exit 0，1/1，测试执行 20.03 秒。
2. 临时 fixture 阶段/异常诊断：默认并行包级套件 exit 0，344 单元测试、12 路由测试、1 doctest 通过，1 doctest 忽略。
3. 5 种隔离子环境探针：正常环境、缺失 HOME、缺失 USERPROFILE、两者同时缺失、隔离 PATH 均成功创建子进程。探针没有改变测试宿主的全局环境。
4. 受控延迟红例：在旧测试时序下关闭 `start-child` 许可；原有 action timeout 和 PID 等待界限保持不变。测试实际 exit 101，记录 `parent-ready:2`，随后报 `grandchild.pid` 缺失。证据：`continuation-doctor-controlled-red.log/json`，源码快照：`continuation-doctor-controlled-red.rs.txt`。

临时阶段计时和异常采集已经从最终源码移除。

## 实现与生产兼容

- 私有 `spawn_doctor` 提取原 command/stdin/stdout/stderr 设置及 `ManagedProcess::spawn`；错误映射保持不变。
- 私有 `capture_doctor_output` 提取原管道接管、reader task、timeout、wait、terminate_tree 与 reader join。该段与修改前源码文本完全相同，证据为 `continuation-doctor-sequence-proof.json`。
- 实际 `capture_doctor` 在 spawn 后立即调用相同等待/清理 helper，直接传递原 timeout。生产入口没有等待 fixture ready。`DOCTOR_TIMEOUT`、`DOCTOR_TERMINATE_GRACE`、取消/kill/wait/管道 join 顺序及错误传播保持不变。
- 树清理测试先确认真实父进程存活，再释放临时目录中的文件许可，随后确认真实孙进程存活，最后运行同一个生产等待/清理 helper。保留 `DoctorError::Timeout` 和两个 PID 消失断言，并增加两个清理前存活断言。
- 新增 `doctor_deadline_applies_before_fixture_is_ready`，直接调用实际生产入口。子进程启动许可持续关闭；200ms 的调用 deadline 必须返回 Timeout，外层 5 秒仅约束测试失效时的等待。该用例验证生产 spawn 后的 deadline。
- 两个平台的测试 fixture 使用文件许可同步。Windows 以 10ms、Unix 以 0.01 秒轮询显式文件条件。既有每阶段 PID 就绪与退出等待界限保持不变；就绪后的 action timeout 改为 200ms。

未修改共享 process gateway，没有新增全套串行、环境全局锁、lint allow 或忽略测试。

## 最终验证

| 验证 | 精确命令 | 结果 |
| --- | --- | --- |
| 窄测复验 | `cargo test -p ccr-cli --all-features --lib commands::codex::fix::tests::doctor_ -- --nocapture` | exit 0，3/3，0.56 秒测试执行时间 |
| 包级默认并行 suite | `cargo test -p ccr-cli --all-features -- --skip export_bindings` | exit 0，345 单元 + 12 路由 + 1 doctest 通过；24 bindings 过滤，1 doctest 忽略 |
| 严格 Clippy | `cargo clippy -p ccr-cli --all-targets --all-features -- -D warnings` | exit 0 |
| 只读格式检查 | `rustfmt --check --edition 2024 --config skip_children=true crates/ccr-cli/src/commands/codex/fix.rs` | exit 0 |
| 范围 diff 检查 | `git diff --check -- crates/ccr-cli/src/commands/codex/fix.rs` | exit 0；保留 LF/CRLF 提示 |

每项 Cargo 与最终检查均在开始与结束采集 `fix.rs` 的原始字节 SHA256，并记录精确 argv、cwd、UTC 时间、exit code、墙钟用时及日志 SHA。逐项记录在 `continuation-doctor-final-*.json` 和 `continuation-doctor-focused-retry.json`，汇总为 `continuation-doctor-implementation.json`。

## 独立保留的启动失败

首次修复版窄测编译通过后，测试程序在输出 `running N tests` 之前以 `0xc0000005 STATUS_ACCESS_VIOLATION` 退出，Cargo exit 5。日志与元数据保留为 `continuation-doctor-focused.log/json`，不能计入断言通过次数。

随后同路径可执行文件 `--list` 成功；源文件保持不变，同一 Cargo 命令重试通过 3 项测试，重试前后可执行文件 SHA 相同。崩溃瞬间未另采二进制 SHA；首次后续探针与重试记录包含 SHA。该启动异常的原因未查明，成功重试没有替代原失败记录。

## 冻结与交接

修改前 SHA：`2615ff8ef43b62d659b9e1604c56ece8161fd6552d93096b6ea07a751d2b667a`。

冻结后 SHA：`9975befad77e14eef324ead6c1ac634819884a7b433f534b22c56304ab9898ce`。窄测重试、包级 suite、Clippy、rustfmt 与 diff 检查的开始/结束 SHA 全部一致。增量见 `continuation-doctor-final.diff`。

Windows Cargo 已释放，session `41695` 已结束；本 owner 无活动测试或构建。源码交由 `/root/implement_t10_gates` 非作者审查，完整 `just ci` 和 WSL 验证由 root 执行。本 owner 的 Windows 真实进程测试不代表 Linux/macOS 或原生 WebView 验收。

未修改受保护 `.tmp` 脚本、任务状态、父报告或矩阵；未提交、推送或归档。
