# GPUI 09 第一切片证据

日期：2026-11-05。验收项 9.1–9.6 **没有**标为通过。macOS 与 Linux 的原生壳能力为 `not_run`。没有打开可见的系统窗口，因此没有像素截图。下面的界面句子来自无头渲染测试和同一套格式化函数。

## 界面实际显示

默认语言是英文，同一时刻只显示一种语言。设置页在原有外观区下面增加「执行环境」。纸面仍是实色，文案不声称背景模糊。侧栏仍是 13 个目的地。

演示数据（fixture，无头测试已断言）：

- `This desktop is running in the local environment on Windows.`
- `WSL was not queried for demo data.`
- `SSH management is not connected in this desktop shell.`
- `The tray is unsupported. This window stack has no tray.`
- `Hide and restore are unsupported on Windows.`
- `A system notification can be posted. Delivery is not confirmed.`

按下「Post a system notification」后追加：`Notification request posted. Delivery is not confirmed.` 无头测试平台记录了 1 条 tag 为 `ccr-gpui-environment` 的通知请求。没有确认 Windows 通知中心真的弹出。

中文是另一套单独文案，例如 `此桌面正在 Windows 上的本地环境中运行。` 切换语言后英文句子不再同时出现。

非 fixture 的本机桌面会走同一次探测。2026-11-05 在这台机器上的分类结果是：

`Listed { names: ["docker-desktop"], exec: PathNotFound }`

对应英文句子：

`WSL listed docker-desktop. A command in the default distribution failed because a path was not found. No usable distribution is connected.`

没有发明发行版，没有创建发行版，也没有把 `ext4.vhdx` 路径写进界面。`~/.ccr-ui/ccr-ui.db` 已存在，只读查询 `ssh_hosts.name` 得到 0 行，因此实机会显示 `No saved SSH host names.` 文件在但打不开或查不了时显示 `Saved SSH host names could not be read.` 不显示用户名、身份文件、指纹或数据库路径，也不发起 SSH 连接，不接受 host key。

Tauri 的 `wsl_list_distros` 仍执行原来的 `wsl -l -v`，成功时仍返回 `Vec<WslDistroInfo>`。共享层只解码和解析这份输出。命令名、参数和返回结构没有新增或改名。

## 壳能力

gpui-kit 0.7.3 / gpui-pre 0.3.9 没有托盘类型。Windows 的 `Platform::hide` 是空函数，也没有恢复调用，所以没有加隐藏按钮。系统通知 API 可以提交请求；本切片不把未确认的系统弹出算作完成。

关闭窗口仍会丢掉设置页持有的读取任务，运行时在事件循环返回后 join。没有做进程树清理证明，不能把 WSL 子进程的后代清理说成已验证。

## 命令与退出码

检查角色于 2026-11-05 重跑。三次桌面命令都是退出码 0。

| 命令 | 退出码 |
| --- | --- |
| `just desktop-fmt-check` | 0 |
| `just desktop-lint` | 0 |
| `just desktop-test` | 0 |

`just desktop-test` 的 Cargo 部分：库测试 93 通过，`native_shell` 8 通过，`native_ui` 2 通过。环境段落的四项测试都在这 93 项里。

另外：`cargo test -p ccr-app --lib environments` 6 通过、1 个忽略；`cargo test -p ccr-db --lib list_host_names_returns_labels_without_secret_columns` 通过；Tauri `platform::wsl::tests` 3 通过。被忽略的 `live_wsl_probe_prints_classified_result` 在检查时又跑了一次，约 3.0 秒，打印：

`Listed { names: ["docker-desktop"], exec: PathNotFound }`
