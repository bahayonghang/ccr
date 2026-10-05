# Doctor Windows 夹具修复

两项 Doctor 测试的空 PATH 故障已修复。3 项定向测试、345 项 CLI 库测试、严格 Clippy、格式与差异检查均通过。源码改动只在 fix.rs 的测试模块；生产区字节保持一致。

## 故障证据

- remaining-windows-ci.json：2026-09-29T01:42:05Z 启动的正式 just ci，174.077 秒，exit 1。workspace ccr-cli 有 343 项通过、2 项失败；失败均为 Spawn("program not found")。该次运行未到达 bindings exporter，未观察到访问冲突。
- Windows hanging_doctor_command 使用裸 powershell.exe。并行 install_detect 测试通过 TestHostEnv 清空或替换 PATH；原 Doctor 测试没有参与同一环境锁。
- 对原 CI 的 ccr_cli-74d992b21dfc8d04.exe 在独立子进程设置 PATH 为空，稳定得到相同两项 spawn 失败：1 项通过、2 项失败、exit 101。原 exe SHA 与正式 CI 记录一致，详见 remaining-doctor-empty-path-red.json/log。
- 正式 CI 没有记录故障瞬间的 PATH 值和环境锁 owner，因此具体并发调度尚未直接观测。已确认的缺陷是测试夹具依赖可变 PATH。历史 bindings exporter 的 STATUS_ACCESS_VIOLATION 原因仍未查明。

## 最小改动

- 父夹具使用 SystemRoot 下的绝对 PowerShell 路径。
- 后代通过 PSHOME 下的 PowerShell 执行内置 Start-Sleep，消除 cmd/ping 的 PATH 依赖。
- 两项已有 Windows 测试使用现有 TestHostEnv 局部控制空 PATH，并在 Drop 还原。保持库测试默认并行，没有新增 serial annotation、test-threads 参数、skip 或重试。
- 原有 ready barrier、200ms action deadline、5s 外部 watchdog、父/后代存活检查及最终退出断言均保留。
- 生产 capture_doctor、spawn_doctor、capture_doctor_output 和清理策略均未变。

源码 old SHA：9975befad77e14eef324ead6c1ac634819884a7b433f534b22c56304ab9898ce。

源码 new SHA：9d2a321acb63ab1563b51cddc56cd3c4c6b02cd141b9fdd18ee8b02ef762194c。

原字节和精确差异：remaining-doctor-before.json、remaining-doctor-change.diff。

## 验证

| 命令 | 结果 | 证据 |
| --- | --- | --- |
| cargo test --locked -p ccr-cli --all-features --lib commands::codex::fix::tests::doctor_ -- --nocapture | exit 0；3 passed | remaining-doctor-focused.json/log |
| cargo test --locked -p ccr-cli --all-features --lib -- --skip export_bindings | exit 0；345 passed；24 export filtered；默认并行 | remaining-doctor-lib.json/log |
| cargo clippy --locked -p ccr-cli --all-targets --all-features -- -D warnings | exit 0 | remaining-doctor-clippy.json/log |
| rustfmt --edition 2024 --check crates/ccr-cli/src/commands/codex/fix.rs | exit 0 | remaining-doctor-static-checks.json |
| git diff --check -- crates/ccr-cli/src/commands/codex/fix.rs | exit 0；Git 保留 LF/CRLF 提示 | remaining-doctor-static-checks.json |

实际工具链为 Rust/Cargo 1.98.0。三个 Cargo 运行的源码前后 SHA 均与上述 new SHA 相同。各日志 SHA 在 companion JSON 中可核验。

## 交接

Windows Cargo 已释放。此修复后没有由本子代理运行完整 CI、bindings 或 Linux gate；这些检查由主会话协调。已保留所有历史失败。未修改任务状态，未提交、推送或归档；fix_commit=null。
