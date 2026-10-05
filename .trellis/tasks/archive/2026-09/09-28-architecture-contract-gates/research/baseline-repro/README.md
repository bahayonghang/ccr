# T10 原版本反例补验

基线为 34d8a85e0e48b793733835e0304c8ed33940fcee。CLI、A09 和 A15 在 managed worktree 新增测试模块或测试文件。A14 从原 commit 提取 mapper/helper，在本 research 目录执行。主工作区产品源码没有被这些反例改写。Rust 用临时目录和合成状态，A09 使用自编译静默子进程。

- CLI/library：0 passed / 4 failed；A01/A02/A03/A11 均失败在目标行为断言。编译成功。
- A01：执行原 Tauri handler 所调用的 retired switch_command 目标。未执行完整旧版 Tauri UI 流程。
- A02：用实际 Platform helper 和 ConfigService 按确定性读-写-旧快照写交错，证明已成功服务写丢失。修复后的多进程证据另见 T01/T03。
- A03：实际 switch_profile_for_platform 完成 runtime/current=new 后，合成历史 DB 故障逃逸为 generic Err。没有把该结果等同于全部 TUI 路径红例。
- A11：对合成文件的真实 icacls 无效用户命令非零退出。原 AtomicWriter -> ensure_private_permissions 调用顺序仍返回成功。是原函数组合 fixture；未运行完整 OAuth handler，也未声明真实凭据泄露。
- A09：0 passed / 1 failed。仅缩短 descriptor 的 1 小时配置为 30ms；原 stream/spawn/cleanup 保持。真实子进程超过 300ms，显式取消后完成清理。cleanup-error 原分支仍由源码和修复后故障测试支持。
- A14：0 passed / 1 failed。原 mapper/helper 在只修改 model 时，把已有 notifications 事件数组变为 false。唯一导入改写指向同一基线 helper；原 SHA、命令和日志保留。该反例覆盖 mapper 边界。
- A15：0 passed / 1 failed / 29 skipped；原 GrokSettingsView、mapper、domain 调用真实执行，IPC 用合成响应。managed_keys_locked=true 时 model.disabled=false。输入 fixture 与修复后正式测试相同。

本目录共记录 7 项预期失败断言；不包含原审查阶段已有的 A08/A13 反例。所有预期红例保留原日志与命令 JSON。测试工具 wrapper 在输出日志时的 GBK 编码错误不影响 a15-baseline.log 已记录的 Vitest 断言失败。

不把旧代码反例、修复后测试、原生用户环境验收或提交状态混为一种证据。修复 commit 未创建。
