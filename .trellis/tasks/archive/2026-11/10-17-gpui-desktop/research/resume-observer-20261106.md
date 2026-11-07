# 2026-11-06 Claude Observer 与订阅设置

## 本批实现

- Usage 增加 Claude 观测入口，复用当前主题和窗口。共享 `ccr-app::usage::observer` 负责现有 Claude 总计/ROI、逐日趋势、项目/模型费用、缓存统计、工具热力/工具排行及会话排行编排。所有用量查询仍走 `ccr-usage::Dashboard`，工具/设置 SQL 留在 `ccr-db` repositories。
- Tauri 保留原 DTO 定义、字段注释、命令签名和 TS 绑定位置，仅转为共享 owner 的薄适配。原嵌入式 `pricing_version` 仍由 Tauri pricing owner 加到 wire；原生不复制价格目录或声称该目录参与 llmusage 计价。原 wire 工具会话中的兼容 tokens=0 保留，原生报告省略未知 token/model 字段。
- 主库正确使用 `~/.ccr-ui/ccr-ui.db`，不把 usage archive `~/.ccr/analytics/usage.db` 当订阅/工具来源。路径在首次 worker 访问时捕获并由 Services owner 保留。fixture 使用 registry 的绝对父目录下独立 `observer.db` 和空的 `observer-migration-home`，无 HOME 回退。
- 打开页面只读现有数据库，不创建或迁移。独立用量来源缺失/不兼容时报告不可用，设置仍可单独工作；缺失主库呈显式初始化提示。只有确认保存才调用既有数据库初始化/迁移 owner；Local 沿用注册主池，fixture 使用显式注入路径。
- 订阅确认默认取消，Escape 与首帧根 Enter 安全取消，过期焦点回调不能抢回焦点。数据写入由应用持有，离页仍完成并可重新读取。已读 subscription 快照在 immediate SQLite transaction 内核对并原子更新三键；冲突拒绝覆盖。现有损坏/缺 schema 数据库不能当成默认设置或新库静默初始化。
- 新增 `ccr-db::claude_tool_calls_repo::session_count` 接回原 Tauri 的 COUNT DISTINCT；数据库失败不再冒充真实会话数量 0。
- 用量导入提取后新增的 Tauri orphan（read_bounded_line/ManagedChild re-export 与未使用的 spawn shim）已删除；共享进程 owner 不变。

## 实际验证

- desktop `cargo test --locked runtime::insights::observer -- --skip export_bindings`：exit 0，2 passed（缺库只读、确认首次设置独立于用量库、stale conflict、损坏 schema 非默认值）。
- root `cargo test -p ccr-app --locked usage::observer -- --skip export_bindings`：exit 0，4 passed，包含第二键写入注入失败时全键回滚、成功更新与旧快照拒绝。
- Tauri `cargo test --locked claude_observer -- --skip export_bindings`：exit 0，21 passed（跳过 exports，默认并行）。首轮服务测试提取后缺测试专用 Local import，编译失败；补回 test-only import 后复验通过。
- desktop `cargo test --locked observer_setup_confirmation_cancels_then_saves_without_a_usage_database -- --skip export_bindings`：exit 0，1 passed。实际 Shell Usage 入口、月费输入、取消确认不建库、确认保存、离页应用持有和重进页面验证。最初回归复现首帧 Escape 焦点未跟踪，已在真实确认根修复后通过。
- 已编译缓存下，仅此 Windows 测试进程使用 `RUST_MIN_STACK=1048576` 的 `cargo test --locked --lib observer_setup_confirmation_cancels_then_saves_without_a_usage_database -- --skip export_bindings`：exit 0，实际 1 passed / 160 filtered，0.95s，没有编译步骤；产品栈未增大。这是低栈 headless 证据，不是 OS 窗口验收。
- owned paths `git diff --check` exit 0；格式修复使用显式 rustfmt。集中全量门禁、最终 bindings 和完整 diff 独立检查由主会话执行。

## 显式导入接通工具事实

- 已关闭上述调用链缺口：scanner/jsonl 移入同一共享 owner，Tauri scanner 是保留默认 root discovery/返回形状的薄入口，原 Tauri jsonl 孤儿模块删除。GPUI 在显式 llmusage 导入成功后继续更新捕获的 `provider_home/.claude/projects`，使用正确 CCR 主库。只有明确导入可能初始化主库，打开观测页仍不写入；没有新增 watcher。
- CLI 与工具扫描属于同一 retained runner。取消在目录遍历和解析行/文件之间检查；等待当前同步操作回到边界后才公布终态。CLI 已成功后保留用量提交数量；工具更新失败/取消不会声称回滚，状态中明确展示工具未完成和已保留变更。每文件扫描错误数与处理计数可见，原始路径/错误不进入 UI 快照。
- 修复原 scanner 追加批次 seq 从 1 重启、可能覆盖旧工具行的问题。mtime 未变仍快速跳过；已变文件从头重读以保持确定性 seq，既有 mtime/offset 游标保留。上限为每文件 64 MiB、每行 1 MiB，超过上限/无效 JSON/读错误会报告来源错误并保留旧游标，不能当作已导入。该受限重读有额外 I/O 成本，没有性能达标声明。
- `ccr-db::record_ingested_file` 在同一 immediate transaction 写入行和游标，任何游标失败全批回滚；不是 shared/native SQL 复制，也没有 schema 变化。
- 后续共享 `cargo test -p ccr-app --locked usage::observer -- --skip export_bindings` exit 0，13 passed，包含移植 parser/scanner 与新增 explicit-root/append/游标失败回滚重试/cancel/损坏追加游标保留回归。
- 后续 desktop `cargo test --locked runtime::insights::import -- --skip export_bindings` exit 0，3 passed。既有合成 CLI 测试新增真实 fixture tool JSONL，确认首次显式导入产生正确主库工具行、再次导入不重复，离页与取消契约继续通过。

## 尚未覆盖的验收
- 无 computer use、WSL/doctor、真实 SSH、用户数据写入、真实外部服务验证；原生 OS/性能/其他平台验收不因此关闭。
