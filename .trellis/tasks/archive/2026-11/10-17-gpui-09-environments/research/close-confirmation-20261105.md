# 主窗口关闭确认：实现收据

- 基线与门禁时 HEAD：`609cceb305c157428c7b6487fb349910d9635d14`，当前 `dev` 工作区；2026-11-05 23:00 EST 核对并行状态，仅本切片 Shell 文件和主会话负责的 shared-desktop-services 规范变化，无范围外脏文件。
- 产品文件：`ccr-desktop/src/shell/{mod.rs,close.rs}`；回归：`close_tests.rs`。复用现有窗口、Kit 模态栈、locale、solid popover 和退出 owner；无新增依赖、公共服务 API、UI 窗口栈或 Tauri 载荷变化。
- 行为：每次主窗口关闭都确认（闲置也确认）；默认取消，取消/Escape 不清空底层域确认与草稿、不取消活动任务，焦点恢复。重复关闭只一个确认框；本 opening 的失活与焦点检查约束延迟回调。Kit generic Enter 在首帧 root 只安全返回，Cancel Enter 取消；Quit 的明确键盘/鼠标激活才允许一次 remove_window。
- 退出文案只承诺请求取消并等待结束，不把 cancellation request 视为终态，也不保证清理成功。精确 dirty 聚合、可配置偏好仍未交付。

## 实际验证

日志目录：`outputs/desktop/gpui09-close-20261105/`（本地完整日志）。命令在仓库根运行，行为测试保持默认并行与 `--skip export_bindings`。

| 命令/阶段 | 实际结果 |
| --- | --- |
| `python ccr-desktop/scripts/desktop.py cargo test --locked --lib shell::close_tests -- --skip export_bindings`，新增测试、实现前 | 101；`close-before.log`，缺少关闭确认 |
| 同一窄回归，最终版本 | 0；`close-final-narrow.log`，2 passed，包含中英完整键盘流程 |
| `just desktop-fmt-check` | 0；`desktop-fmt-check.log` |
| `just desktop-lint` | 初次 101（测试两处冗余 into），修正后 0；before 与最终日志均保留 |
| `just desktop-test` | 0，无重试；`desktop-test.log`，Python 12、AccessKit 3、库 105、native_shell 8、native_ui 2、runtime_shutdown 2 passed/1 helper ignored |

中间失败记录保留：Kit Enter-on-Cancel 实际关闭窗口；鼠标 Quit 不自动获得焦点导致许可未满足。最终实现补上对应路径，正向退出均由真实输入验证。测试 setup 的语言初始化顺序、异步 UI enabled 等待及滚动编辑器 Tab 导航也已修正；一次 test-clock 等待挂起由实施者停止自身测试进程，记录为 `close-scheduler-stopped.log`（进程 0xffffffff），不是产品通过或超时退出证据。最终 setup 使用真实 10 秒 deadline，推进现有 executor/frame；Kit disabled 快照是 Some(true)/None，另以真实编辑输入成功证明可用。

嵌套场景经实际配置列表选择、编辑输入和明文确认进入；取消关闭后断言原确认、焦点与未保存描述保留，catalog revision 未改。受控任务先握手证明已运行，取消关闭后仍未请求取消/无终态；确认退出后窗口不可访问，测试显式 final owner shutdown，终态 Cancelled 且 active_tasks=0。失败路径的 owner Drop guard 仍请求取消并 join。

## 证据边界

这是 GPUI rendered/headless input 与隔离 runtime fixture；TestAppContext 没有真实 OS event loop，未将显式 shutdown 冒称原生 close/on_app_quit 收据。拦截范围仅是主 Shell 的 should-close callback；Probe、启动失败退出和绕过该 callback 的 application-level Quit 不在本切片的确认保障内。既有生产 on_app_quit 与事件循环外 owner join 没有修改。未运行真实 UI、WSL/doctor、SSH/网络/keyscan 或身份读取；托盘/隐藏恢复、其他 OS、watcher、glass 及尺寸/DPI 不新增结论。9.1–9.6 全部仍开放；不归档、不提交。

独立 [关闭确认检查](check-close-confirmation-20261105.md) 已完成：无产品缺陷，补充了退出入口的精确范围；独立 fmt、all-target lint/type-check、close 窄回归均退出 0，未改产品代码，因此复用本轮不变完整 suite 的通过日志，不重复执行或冒称新完整运行。
