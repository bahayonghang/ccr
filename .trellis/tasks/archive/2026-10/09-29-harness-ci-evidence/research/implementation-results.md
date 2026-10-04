# T07 历史失败复验与当前证据

- 基线：dev / fe9d65972376dbb4c332fde9860c1e42ff2f2005；当前工作区含已批准、未提交的治理与依赖改动。Rust 产品源码和两个 Cargo lockfile 未改。
- 本地证据于 2026-09-30 UTC 采集；原审查日期为 2026-09-29 America/Chicago。
- 前置 T06 集成与最终审查进行中。本文件记录已取得的独立验证结果，不将缺失环境标记通过。

## 历史记录与复验

| 编号 | 原始 Run / SHA | 直接失败与原因边界 | 当前证据 |
| --- | --- | --- | --- |
| H01 | 32684239641 / fac5611f5587dcbe842ef8c133072a26f7ef7be5 | Linux Root Coverage 内 doctor 测试的 report_path 为 None；旧日志不能区分报告保存和捕获分支，底层根因未查明 | Windows 定向测试 1/1；WSL Linux 完整 workspace coverage 通过，当前失败未重现 |
| H02 | 32684151306 / 8fb8f20ac551f2133e4e9c5190fe6f82ac9d8455 | Windows stdin/status.success 与输出洪泛/stdout_truncated 断言失败；启动、超时及回收原因未查明 | Windows 原有进程 smoke：Tauri 10/10、root gateway 7/7；完整 Tauri coverage 通过 |
| H03 | 32684151228 / 8fb8f20ac551f2133e4e9c5190fe6f82ac9d8455 | Vue 测试清理期间 onUserConsoleLog RPC 未完成，触发 EnvironmentTeardownError；没有阈值不足证据 | 当前 React/jsdom 169 个文件、904 项 smoke 和完整 coverage 通过；不能据此证明旧版本根因修复 |

当前复验均通过，因此没有修改 doctor、gateway、React cleanup 或 smoke 配置中的生产行为及诊断代码。原始失败回执保留在父任务 research/evidence。测试夹具规范新增五工具通用的历史证据、诊断字段和环境边界要求。

## 正式命令与原门槛

| 环境和命令 | 结果 | 原始回执 |
| --- | --- | --- |
| Windows cargo test -p ccr-cli non_dry_run_doctor_persists_sanitized_report | 1 passed，退出 0 | [doctor](windows-doctor.json) |
| Windows just tauri-process-smoke | 17 passed，退出 0 | [process smoke](windows-process-smoke.json) |
| WSL Ubuntu-24.04 cargo llvm-cov --workspace --all-features --json --output-path target/evergreen-linux-coverage-workspace.json -- --skip export_bindings | 完整工作区 1847 passed、6 ignored、33 filtered；退出 0 | [coverage](linux-workspace-coverage.json) |
| WSL 原 coverage 阈值脚本，overall 70 / gateway 85 | 总体 74.60%、gateway 96.09%；退出 0 | [thresholds](linux-workspace-thresholds.json) |
| Windows just coverage-tauri | 407 passed、1 ignored、197 export filtered，另 2 个 guard；gateway 95.59% > 85%，退出 0 | [Tauri coverage](windows-tauri-coverage.json) |
| Windows rustup run 1.95 cargo check --workspace --all-targets --all-features | 安装别名对应 rustc 1.95.0；退出 0 | [MSRV](msrv-1_95.json) |
| Windows just frontend-coverage | 行 75.19% > 70%，904 项 smoke；退出 0 | [T08 完整证据](../../09-29-harness-frontend-security/research/implementation-results.md) |
| Windows just vscode-coverage | 行 91.86%、函数 91.50%，原门槛各 70%；退出 0 | [T05 完整证据](../../09-29-harness-vsix-package/research/dependency-patch-results.md) |

Linux 命令展开现有 coverage-rust recipe，仅使用单独输出路径；测试集合、并行方式、binding skip 和 70%/85% 阈值不变。Tauri 的总体 51.67% 为完整报告信息；现有硬门槛只针对 gateway。

Windows 使用 cargo-llvm-cov 0.9.1，WSL 使用项目 ignored target 下安装的 0.9.0，未改全局工具。本机 Bun 1.4.2、Node 26.7.0 与托管固定版本不同。版本、测试计数和无 Rust 源码改动证据见 [current-validation-summary.json](current-validation-summary.json)。

## 未完成的验收边界

- macOS 原生进程测试：UNVERIFIED，本机没有该执行环境。
- 五客户端新会话加载、hooks、trust 和实际权限：UNVERIFIED；文件与静态测试不替代原生证据。本次授权没有 GUI 操作或信任设置变更。
- 最终候选提交的 hosted CI 与真实 Dependabot：UNVERIFIED；工作区尚未提交，本次没有推送或远程重跑授权。
- WSL Linux 结果属于本地 Linux 执行；Windows MSRV 结果属于本地 Windows。均不替代托管 OS 矩阵。
- Tauri 的 7 条 unmaintained 和 2 条 unsound warning 保留；[逐项诊断](tauri-advisory-review.md) 已记录三目标、两种 feature 模式、依赖链、责任层和处置建议。九份缓存公告与固定 Git blobs 逐字节一致；无关未跟踪缓存残留已单独记录，未修改用户缓存。没有新增 ignore。

T07-AC1、AC2 的当前本地证据已取得；AC3 的缺失环境使 T07 保持未完成。AC4 保留旧失败及全部安全告警。父任务不得据本地通过宣称完整托管或原生验收。
