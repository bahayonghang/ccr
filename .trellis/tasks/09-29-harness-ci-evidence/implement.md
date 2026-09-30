# T07 实施计划

## 启动条件
- [x] 用户在父任务最终总结后明确批准本项。
- [x] 已核对基线、Rust 源码/lock 未改、实际工具链和诊断输入哈希。
- [ ] T06 本地实施与独立窄检查已通过；完整 just ci 两次失败，交付源哈希均未变化，整体验收尚未关闭。
- [x] 上下文验证通过，主会话已启动；task.json 为 in_progress。

## 步骤
- [x] 已复核 H01/H02/H03 原始 run/SHA 及 O01 九条警告，未扩大产品修改范围。
- [x] 当前测试未重现失败；未改产品诊断逻辑。完成证据与规范回写，保留原始失败。
- [ ] AC1/AC2/AC4 的本地证据已取得；AC3 的 macOS、新会话和托管边界未完成。
- [x] 强模型完成九条警告的依赖图与来源复核，父任务独立集成审查归集其他结果。
- [x] 测试夹具和依赖治理规范已回写，注明五工具及 OS 边界；未提交归档。

## 必须通过的检查
- [x] `cargo test -p ccr-cli non_dry_run_doctor_persists_sanitized_report`
- [x] `just tauri-process-smoke`
- [x] `just frontend-coverage`
- [x] `just vscode-coverage`
- [x] WSL Linux 执行 `just coverage-rust` 的完整展开命令，保留 workspace/all-features/binding skip 与 70%/85% 阈值；见实施记录。
- [x] `just coverage-tauri`
- [x] Windows 已安装别名 `rustup run 1.95 cargo check --workspace --all-targets --all-features` 通过；rustc 报告 1.95.0，未安装新工具链。
- [ ] `记录托管 workflow 对应最终 SHA 的结果；本任务不含 push、PR、rerun 或 release 授权`

## 证据
记录命令、退出码、日期、OS、版本、输入 SHA 和输出摘要。没有运行的正式环境检查保留 UNVERIFIED。不要把 shell 退出 0、jsdom、原生测试或托管结果互相替代。

完整结果见 [实施与边界](research/implementation-results.md)；九条警告见 [诊断](research/tauri-advisory-review.md)。原报告与当前复验分开保存。

## managers 新失败的补充范围（待批准）

[只读诊断与六行候选](research/manager-lock-diagnosis.md)记录 6 个未保护的默认锁目录读取者、1 个已有环境 guard 的修改者与 10 个无关用例。原失败具体交错未观测；定向 1/1 和默认并行 17/17 通过，原完整 CI FAIL 保留。

- [x] 保存第二次完整失败和两次有假设的只读诊断；未修改源码。
- [ ] 用户批准 general.rs 与 legacy_registry.rs 共 6 行现有 RAII 夹具绑定；这两个文件仍在当前设计白名单之外。
- [ ] 批准后更新精确白名单、应用候选并完成定向、工作区和完整 just ci 门槛。
- [ ] 在已有 test-fixtures 规范写回传递环境读取及适用五工具，完成独立复核。

六行候选已通过 [独立静态审查](research/manager-lock-independent-review.md)。批准后必须保留该报告列出的现有 fixture 专门检查和完整入口；候选仍未应用。
