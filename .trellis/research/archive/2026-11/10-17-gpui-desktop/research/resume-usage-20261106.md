# 2026-11-06 用量导入续作

## 行为

- 完成上轮未接合的 `ccr-app::usage::import` 提取。Tauri 保留原命令、DTO 和错误载荷；CLI 协议、NDJSON、默认一小时 deadline、五秒进程树/reader 清理由同一共享 owner 持有。原 lifecycle tests 随 owner 移动，未另建 runner。
- GPUI Usage 增加显式“导入本地用量”、状态和取消。Services 持有 owner，离页只丢弃观察；重新进入可以看到当前状态并取消。排队、运行和取消请求均占用 admission，terminal 由 runner/已丢弃的排队闭包确认，不把取消请求立即显示成完成。
- 配置路径可在首次数据库创建之前捕获，但打开页面只读，不自动导入。自定义 provider home、远程环境与普通 fixture 启动拒绝执行，不能静默扫描默认本机来源。测试合成进程仅在 test cfg 下显式开启，并检查根目录包含关系。
- 仅保留安全计数与类型化失败，不将 CLI 错误、当前文件路径或原始事件放进 UI/owner 快照。进度通知与页面消费采用共享 100ms 预算；来源错误数独立显示，optional absent 不作为失败。
- 修复完整 CI 暴露的 `usage_jobs` 测试引用失效。消费端测试通过公开 `run_sync_stream` 验证预先取消返回；零 spawn 的私有边界仍由共享 lifecycle 回归覆盖，未为测试公开 spawn hook。

## 本次验证

- `cargo test -p ccr-app --locked usage::import -- --skip export_bindings`：exit 0，27 passed。
- 独立 desktop `cargo test --locked runtime::insights::import -- --skip export_bindings`：exit 0，3 passed；覆盖远程/fixture/custom-root 拒绝、离页成功保留、运行取消与 busy、排队取消。
- 独立 desktop `cargo test --locked import_controls_reject_fixture_then_reattach_and_cancel_retained_work -- --skip export_bindings`：exit 0，1 passed，实际 headless 控件。
- 首次 Tauri 窄测试 filter 写成 `usage_jobs::tests::...`，编译通过但 0 tests；不能当行为通过。随后以实际函数名 `cancel_before_runner_start_returns_cancelled_and_keeps_admission` 重跑，结果见下行。
- Tauri `cargo test --locked cancel_before_runner_start_returns_cancelled_and_keeps_admission -- --skip export_bindings`：exit 0，1 passed。
- `git diff --check`：本批 owned paths exit 0。格式修复使用显式 rustfmt，与 validation 分开；默认测试并行与 `--skip export_bindings` 保持。
- root/Tauri 既有 target build-lock 遇到 sandbox Access denied 后，仅检查命令使用已授权 elevated execution；未修改 HOME/USERPROFILE、工具配置或 target 权限。

## 验收边界

本批没有 computer use、WSL/doctor、真实 SSH、真实 llmusage 扫描或外部服务。合成进程与 headless 通过不是原生实机验收。集中 desktop gates、Tauri bindings 和最终完整 CI 由主会话串行完成，父任务/01/09 状态保持开放。
