# WebDAV 操作续作（2026-11-06）

范围：完成中断工作区中的 WebDAV 解绑及已启用文件夹批量推拉。复用 `ccr-sync` 传输/事务备份与 GPUI retained runtime；不增加窗口、传输实现或 Tauri 载荷。

- 解绑先捕获 canonical/legacy 两文件版本，明确确认后清除 canonical 凭据，再删除 legacy。保留未知字段、文件夹及受保护备份；任一版本变化拒绝，legacy 清理失败报告部分完成，不回写旧凭据。
- 批量推拉仅选择已启用 canonical folders，复用现有单文件夹操作，逐项记录失败/备份；下载的跨文件夹结果不承诺全批回滚。fixture 禁止远程访问，SSH 请求在 runtime 入口拒绝。
- 修复未完成代码中的 fixture 锁作用域返回类型和两个 rendered 测试 App 上下文读取；仅格式化负责文件。

验证：`cargo test -p ccr-sync --locked sync::clear::tests -- --skip export_bindings` exit=0，2 tests（含无 legacy、保留字段/备份、两文件任一版本冲突）。首次 sandbox root Cargo 锁拒绝访问；获自动批准后对同一命令执行通过。`git diff --check`（sync 范围）exit=0。

`cargo test --locked --lib features::sync::tests::disconnect_and_batch -- --skip export_bindings`（`ccr-desktop/`）exit=0，1 rendered test，1.43s；实际控件覆盖批量确认/取消、只选已启用项、fixture 网络拒绝、解绑取消、过期版本拒绝、成功清除与刷新投影。首次编译暴露本范围错误已修，后续并行模块尚缺的测试文件由对应 agent 修复后通过。未运行 computer use、WSL、真实 WebDAV/SSH 或用户数据操作，不认领这些原生/外部验收。
