# GPUI 09 切片证据：路径规划、连接边界与环境隔离

日期：2026-11-05。验收项 9.1–9.6 **没有**标为通过。任务保持 `in_progress`。`prd.md` 的验收框没有勾选。`task.json` 没有改。没有提交。

这一轮只补现有 fixture 能够证明的边界。没有打开套接字，没有运行 `ssh-keyscan`，没有读取密码或身份文件，没有创建 WSL 发行版。没有改 Tauri 命令签名或载荷。产品源码改动只在 `ccr-desktop` 和 `crates/ccr-app`。

## 9.1 WSL 路径规划

`plan_wsl_file` 对 Detect、Read、Write 使用同一条规则。三种意图都不启动 `wsl.exe`，不创建发行版，不读取、不改写本地文件。

没有发行版、空白发行版、盘符路径、UNC（`\\server\...` 与 `//server/...`），以及路径段里的 `..`，结果都是 `Unavailable`。调试文本里没有本地 Windows 路径。有发行版名并且路径是绝对 POSIX 路径时，结果保留发行版名和该 POSIX 路径。测试前后临时文件仍是 `local-secret`，目录项数量不变。

本机单独跑了被忽略的 `environments::tests::live_wsl_probe_prints_classified_result`（`--ignored --exact --nocapture`）。打印：

`wsl-probe=Listed { names: ["docker-desktop"], exec: PathNotFound }`

该测试函数只打印分类，不断言成功。命令退出码 0 不是可用发行版。`PathNotFound` 表示默认发行版里的命令因路径不存在而失败。没有把它记成通过。没有对真实可用发行版做 read/write。

`probe_wsl` 与 `list_installed_distros` 在 `cfg(not(windows))` 下分别返回 `WslProbe::Unsupported` 和 `WslListError::Unsupported`。这一分支没有在 macOS 或 Linux 上执行。源码不是运行收据。

## 9.2 SSH 边界

`boundary_before_connect` 在套接字、密钥扫描和身份文件之前返回静态错误码。设置页没有重连或断开按钮。测试没有进入 `connect_internal`。

| 决策 | 结果 |
| --- | --- |
| 取消 | `ssh_cancelled` |
| 仅密码 | `ssh_auth_unsupported` |
| 新的或未知密钥 | `ssh_host_key_untrusted`；不保存、不信任、不接受 |
| 密钥变更 | `ssh_host_key_mismatch` |
| 指纹文本相同，但本进程没有确认 | `ssh_host_key_untrusted` |
| 重连且本进程没有信任 | `ssh_host_key_untrusted` |
| 从未连接就断开 | `ssh_not_connected` |

错误码不含主机名、指纹、密码、身份路径或 `ssh-keyscan`。决策之后临时 `known_hosts` 仍是 `original`。指纹相同本身不会把主机记成已确认。`Ok` 只表示这次决策没有拒绝，不表示已经连接。

真实 SSH 服务器上的首次信任、密钥变更、认证、断线、重连和取消仍是 `not_run`。

## 9.3 环境隔离

`EnvironmentSurface` 按 `EnvironmentId` 分开保存 cache、draft 和 job 标签。切到另一个环境后，可见槽不含前一个环境的这三项。切回来后，原来的三项还在，后一个环境的文本不出现。

无头设置页渲染：

- 本地：`No SSH host list is loaded in this desktop shell.`、`Draft: local-draft.`、`Job: local-job.`
- 切到 `EnvironmentId::Ssh("fixture-ssh")` 并刷新后：上述本地句子消失，出现 `This action is local-only. The other environment was not read.`，以及这个环境自己的 draft/job
- 切回本地后：本地三项回来，ssh 的 draft/job 和 local-only 句子不再出现

非本地 `load` 在 `inspect` 之前返回 `local_only_environment`。错误码不含 `fixture-ssh`。读取另一个环境源文件的闭包没有执行，文件仍是 `local-secret`。fixture 没有读用户 HOME、OAuth 或用量库，也没有伪造会话、费用、平台状态或发行版名。

Job 行是按环境保存的显示标签，没有启动命令进程。壳上没有环境切换控件。这次切换由测试写入视图状态，不是用户在侧栏里切换真实 WSL/SSH。真实环境切换仍是 `not_run`。因此 9.3 不标通过。

## 9.4 托盘、隐藏与退出

gpui-kit 没有托盘类型。Windows `Platform::hide` 为空，也没有恢复。托盘和 Windows 隐藏/恢复都是 Unsupported。产品影响：没有托盘，也没有隐藏/恢复控件。Unsupported 不是完成。没有新增托盘 crate，也没有第二套窗口栈。

退出后的 owned process、task 和 watcher 清理仍是 `not_run`。设置页读取取消不是进程树清理。

## 9.5 原生壳

| 能力 | Windows | macOS | Linux |
| --- | --- | --- | --- |
| 托盘 | unsupported | not_run | not_run |
| 隐藏/恢复 | unsupported | not_run | not_run |
| 系统通知 | 只能证明提交了请求；没有确认通知中心弹出 | not_run | not_run |
| 可见系统窗口 | not_run | not_run | not_run |

无头测试窗口不是可见系统窗口的收据。macOS 和 Linux 没有从这台 Windows 机器推导。

## 9.6 视觉

纸面仍走既有实色 `solid_sheet`。这次没有改主题代码，没有截图，没有声称背景模糊或玻璃。原生 light/dark glass 证据仍是 `not_run`。

侧栏仍是原来的 13 个目的地。同一时刻只有一种语言。没有 Home insights，也没有 `Apply / refresh`。

## 此前已经证明、这一轮没有撤回

离开设置页会取消进行中的环境读取。suspend 或更新一代之后，旧的终态结果不替换可见报告。本进程没有信任时，重连在套接字之前被拒绝。从未连接时，断开报告 `ssh_not_connected`，而不是一次干净的远程断开。这些仍然没有打开套接字。

## 命令与退出码

| 命令 | 退出码 |
| --- | --- |
| `just desktop-fmt-check` | 0 |
| `just desktop-lint` | 0 |
| `just desktop-test` | 第一次 101，重试 0 |
| `cargo test -p ccr-app --lib environments -- --test-threads=1` | 0 |
| `cargo clippy -p ccr-app --all-targets -- -D warnings` | 0 |
| `cargo test -p ccr-app --lib environments::tests::live_wsl_probe_prints_classified_result -- --ignored --exact --nocapture --test-threads=1` | 0 |

`ccr-app` environments：17 通过，1 个忽略。忽略的就是上面单独跑过的本机 WSL 探针。

第一次 `just desktop-test` 失败在 `features::insights::tests::rendered_usage_controls_read_fixture_filters_and_keep_exact_costs`，比较的是 SQLite 文件字节。这次没有改 Insights。按允许的一次重试再跑，第二次通过：Python 12，AccessKit 3，库测试 102，`native_shell` 8，`native_ui` 2。

没有跑 `just ci`。差异不是全仓库：产品代码只在 `ccr-desktop` 和 `crates/ccr-app`。没有改 `ccr-ui/src-tauri`。
