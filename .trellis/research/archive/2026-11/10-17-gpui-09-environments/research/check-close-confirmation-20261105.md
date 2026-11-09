# 主窗口关闭确认：独立检查

日期：2026-11-05（America/New_York）。基线 `dev@609cceb305c157428c7b6487fb349910d9635d14`；检查初始只有本轮七个变更文件，无范围外脏文件。本收据为第八个文件。未提交、归档或建立 worktree。

## 发现与修正

- 无需修复产品代码。补充规范、实施收据和 evidence 的退出入口边界：只保障主 Shell 的 `on_window_should_close` callback；Probe、启动失败退出和绕过 callback 的 application-level Quit 不在本切片确认保障内。保留 idle 也确认、尚无统一 dirty 聚合和配置偏好的限制。
- `Shell::new` 安装 callback；每个窗口的 pending 防止重复弹框，取消/打开均不请求运行时 shutdown。确认移除窗口，沿既有 LastWindowClosed、quit observer 和最终 owner join；未新增退出 owner。
- 对照锁定的 GPUI/Kit 源码：should-close 返回 false veto；DialogAction 在按钮 click 之后冒泡 dispatch Confirm，因此 Quit 按钮的 focus 设置先于许可检查。Cancel 的专用 Enter context 和 on_ok 的 Quit subtree focus 检查共同拦截 Cancel/root Enter。
- 每次 opening 有独立 alive 标记；关闭使其失活，延迟回调还检查捕获焦点是否保持，旧 opening 不会在取消后重开或用户 Tab 后抢回焦点。使用既有 AlertDialog 栈及 solid popover，无新增依赖、窗口栈、主题层、Tauri payload 或路由变化。

## 独立验证

从仓库根运行，保持默认并行与 `--skip export_bindings`；使用既有构建缓存权限，未改缓存配置。

| 命令/证据 | 结果 |
| --- | --- |
| `just desktop-fmt-check` | 退出 0；`outputs/desktop/gpui09-close-20261105/check-fmt.log` |
| `python ccr-desktop/scripts/desktop.py cargo test --locked --lib shell::close_tests -- --skip export_bindings` | 退出 0，2 passed；`check-narrow.log` |
| `just desktop-lint` | 退出 0；all-target 严格 Clippy/type-check，`check-lint.log` |
| `task.py validate .trellis/research/archive/2026-11/10-17-gpui-09-environments` | 退出 0；两个 manifest 各 8 项有效 |
| `git diff --check` | 退出 0 |
| 完整 `just desktop-test` | 复核实现阶段同切片 `desktop-test.log` 及退出 0 收据；本次未重跑，因为检查未改产品。Python 12、AccessKit 3、库 105、native_shell 8、native_ui 2、runtime_shutdown 2 全通过；helper 默认 ignored，但由进程树测试显式调用 |
| `just ci` | 本轮 not_run；仅 desktop 产品与规范/任务文档变化 |

窄回归通过真正 installed callback 和 rendered 输入，未给模态内部直接 seed focus：中英取消/退出标签、首帧 Enter、默认 Cancel Enter、Tab/Shift+Tab、Escape、重复 close、取消即重开与过期 next-frame、键盘和鼠标明确 Quit 都覆盖。嵌套场景真实输入未保存草稿并打开原业务确认；取消关闭后原确认、焦点、草稿及 catalog revision 保留，握手确认已启动的任务仍未请求取消且无终态。enabled 快照检查和后续实际编辑共同证明控件可用；确认移除窗口后才显式最终 shutdown。

## 结论与剩余边界

本切片检查通过，无未修复的范围内问题。它仍是 rendered/headless 证据，不能替代原生 OS close/on_app_quit、真实 watcher、托盘/隐藏恢复、其他 OS 或 glass 验收。没有 UI 操作、WSL/doctor、SSH/keyscan/socket 或真实身份文件读取。9.1–9.6 全部开放；尺寸/DPI 继续延期，任务不归档。
