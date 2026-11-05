# GPUI 09 切片证据：主机密钥比较、WSL 文件目标与设置页读取取消

日期：2026-11-05。验收项 9.1–9.6 **没有**标为通过。设置页进行中的环境读取在离开页面后取消，这一项已经证明。没有进程内信任时，重连会在套接字之前被拒绝；从未连接的断开不会报成一次成功的远程断开。这两项由纯决策和命令委托证明，没有打开套接字。process-tree cleanup、live WSL read/write 和 live SSH 仍是 `not_run`。macOS 与 Linux 的原生壳能力仍是 `not_run`。这一检查没有打开可见的系统窗口，因此没有像素截图。

## 重连与从未连接的断开

`ProcessHostTrust` 只记录本进程里确认过的主机和端口。空白主机名、两端空白、控制字符和端口 0 不会记入。比较前把主机名收成 ASCII 小写，所以 `Example.COM` 与 `example.com` 是同一对。集合为空时，`reconnect_gate` 返回 `ssh_host_key_untrusted`，调用方不得再解析目标。确认过 `example.com:22` 之后，只有这一对主机和端口可以通过；别的主机、别的端口，以及 `$(id)` 这种文本，都得到同一个静态错误码。没有已经载入内存的端点时同样拒绝。错误码里没有主机名、密钥、密码、身份文件或 `ssh-keyscan`。

`ssh_reconnect` 的参数和返回类型没有改。它先取本进程信任集合。集合为空时直接返回 `ssh_host_key_untrusted`，不查注册表，也不进入 `connect_internal`，因此不会读主机行、不会调用 `ssh-keyscan`、不会打开 TCP，也不会读取身份文件。集合里已经有别的主机时，只从本进程注册表里取目标的主机和端口；注册表里没有这个环境，或这一对没有确认过，就返回同一个错误码，仍然不读 SQLite 主机行，不扫描密钥，不打开套接字，不读取身份文件。只有内存中的端点与已确认的主机和端口一致，才会进入原来的 `connect_internal`。那里会再读已保存的主机行，并在 `validate` 和 `test_connectivity` 之前再比较一次；行里的主机若已不再匹配，连接不会开始。`ssh_connect` 仍走原来的连接路径。已保存的指纹匹配不会调用 `note_process_trust`，所以不能单独把主机标成已确认。

`ssh_confirm_host_fingerprint` 只在 known_hosts 和 SQLite 都写成功之后，才把该主机记入本进程信任集合。这一写入路径没有在测试里执行，因为它需要 `AppState`、挑战和磁盘。

`plan_disconnect(false)` 是 `NothingConnected`，对应 `ssh_not_connected`。`plan_disconnect(true)` 仍是原来的干净断开，`last_error` 为空。`ssh_disconnect` 的参数和 `SshConnectionState` 载荷没有改。从未连接时，返回的 `last_error` 是 `ssh_not_connected`，不是一次干净的远程断开。命令仍会把活动环境切回本地；如果之前的活动项是 `ssh:`，运行时状态记下的也是这个错误，而不是空错误。已经连上的会话仍按原来的成功断开返回。

`ssh_reconnect` 和 `ssh_disconnect` 这两个命令函数本身没有用 `AppState` 跑过。测试覆盖的是它们委托的纯决策，以及命令文件里同名的薄包装。确认之后允许重连的那条路径仍会进入现有的 `connect_internal`，这一次没有执行它，也没有打开套接字。

设置页没有新增重连或断开按钮，也没有新文案。无头测试确认不存在 `env-reconnect` 和 `env-disconnect`。侧栏仍是 13 个目的地。纸面仍是实色。没有 Home insights，也没有 `Apply / refresh`。

## 这一片证明了什么

主机密钥比较只看指纹字符串。`classify_host_key` 做精确相等判断。`decide_host_key` 把空白文本当成没有指纹；其余文本不去掉两端空白再比较，所以 `" SHA256:same "` 和 `"SHA256:same"` 是 `Mismatch`。

`New`、`Mismatch` 和 `Matched` 的 `may_store`、`may_trust`、`may_accept` 都是 false。匹配只表示两段指纹文本相同，不写密钥、不打开套接字、不把主机标成已信任。决策值里没有指纹、密钥、密码、身份文件或 `ssh-keyscan`。这三个布尔字段是私有的，外部不能造出一个 `may_accept = true` 的决策。

Tauri 的 `security.rs` 改为再导出同一个 `classify_host_key` 和 `HostKeyStatus`。`ssh_probe_host_fingerprint` 与 `ssh_confirm_host_fingerprint` 的签名和载荷没有改。桌面决策没有调用这两条命令。匹配后重建 known_hosts，以及确认后的写入，仍留在命令层。这一轮只在写入成功之后追加本进程信任记录，没有改写入本身。

没有发行版、空白发行版，或路径是 Windows 盘符 / UNC 时，`resolve_wsl_file` 返回 `Unavailable`。它不返回本地 Windows 路径，不读取、不改写那个文件，不启动 `wsl`，也不创建发行版。有发行版名且路径是绝对 POSIX 路径时，结果保留发行版名和该 POSIX 路径。

界面同一时刻只显示一种语言。设置页在原有外观区下面仍是「执行环境」。纸面仍走既有实色 `solid_sheet`，文案不声称背景模糊。侧栏仍是 13 个目的地。没有 Home insights，也没有 `Apply / refresh` 这种双语斜杠句。

无头测试断言的英文句子：

- `This desktop is running in the local environment on Windows.`
- `WSL was not queried for demo data.`
- `No SSH host list is loaded in this desktop shell.`
- `Saved SSH names stay names. A new or unknown host key is not stored. A changed host key is not accepted.`
- `The tray is unsupported. This window stack has no tray.`
- `Hide and restore are unsupported on Windows.`
- `A system notification can be posted. Delivery is not confirmed.`

按下「Post a system notification」后追加：`Notification request posted. Delivery is not confirmed.` 无头测试记录了 1 条 tag 为 `ccr-gpui-environment` 的通知请求。没有确认 Windows 通知中心真的弹出。

中文是另一套句子，例如 `此桌面正在 Windows 上的本地环境中运行。` 以及 `已保存的 SSH 名称仍只是名称。新的或未知主机密钥不会保存。变更的主机密钥不会被接受。` 切换语言后，英文信任句不再同时出现。

## 设置页读取取消

离开设置页会调用 `suspend`。这一步已经证明。

慢速 fixture 走和实机读取相同的 `task` 与 `ResponseGate`。延迟只在 fixture 里，并且可以被取消。`inspect` 仍用 fixture 选项：不启动 `wsl.exe`，不读 SSH 数据库，不写主机密钥，也不打开网络。零延迟的演示设置页仍立刻给出本地报告，原有设置页测试保持通过。

`leaving_settings_suspends_the_in_flight_environment_read`：点进设置后，读取还在进行，界面是「Checking WSL.」，报告还没写入。再点回仪表盘，`task` 被丢掉，加载结束。仪表盘上没有环境段落。等被取消的 worker 结束后，报告仍是空的，环境句子没有出现在另一条路由上。

`stale_environment_terminal_does_not_replace_the_visible_report`：当前代的成功结果可以写成 `late-host`。`suspend` 之后，或又开始了更新的一代之后，旧 key 的成功或失败结果都不替换可见报告。第二次 `suspend` 之后，当时那一代的 key 同样无效。两个被挂起的 worker 结束后，界面仍是 `Saved SSH host names: late-host.`，不是 fixture 的「No SSH host list is loaded」。

侧栏仍是 13 个目的地。同一时刻只有一种语言。没有 Home insights，没有 `Apply / refresh`。纸面仍是实色。没有新增托盘、隐藏/恢复、发行版、网络连接或主机密钥写入。

## 仍是 not_run

- 9.1：live WSL read/write 仍是 `not_run`。没有在 Windows 上对真实发行版做 detect/read/write fixture。没有创建发行版。非 Windows 伪 WSL 入口这一条没有在 macOS/Linux 上跑。
- 9.2：live SSH 仍是 `not_run`。没有对真实 SSH 做首次信任、key changed、认证失败、断线、重连或取消。没有 `ssh-keyscan`，没有 TCP，没有密码，没有读取身份文件。本进程没有信任记录时拒绝重连，以及从未连接时不把断开报成成功，这两项已经由纯决策证明。
- 9.3：设置页进行中的环境读取离开后不会写进另一条路由，这一取消已经证明。缓存、编辑 draft 和 job 不串环境仍没有证明。
- 9.4：托盘仍是不受支持。没有隐藏/恢复按钮。process-tree cleanup 仍是 `not_run`。退出后的进程树和 watcher 清理没有证明。设置页读取任务的取消是另一件事，不能当作进程树清理。
- 9.5：macOS 与 Linux 原生壳是 `not_run`。Windows 通知只证明提交了请求，没有证明系统弹出。
- 9.6：没有 glass/solid 的原生截图。纸面保持实色。模糊探针、性能预算和系统减少透明度设置都没有跑。

更早一次本机 WSL 分类曾打印 `Listed { names: ["docker-desktop"], exec: PathNotFound }`。这一检查没有重跑被忽略的 `live_wsl_probe_prints_classified_result`，所以上面那行不是这一次的新证据。

## 命令与退出码

这一轮（2026-11-05，重连门闩与从未连接的断开）。检查又收紧了门闩：别的主机已经确认时，未确认目标不再读取 SQLite 主机行。下表是这次收紧之后重新跑过的结果。验收项 9.1–9.6 没有标为通过。

| 命令 | 退出码 |
| --- | --- |
| `just desktop-fmt-check` | 0 |
| `just desktop-lint` | 0 |
| `just desktop-test` | 0 |
| `cargo test -p ccr-app --lib environments -- --test-threads=1` | 0 |
| `cargo clippy -p ccr-app --all-targets -- -D warnings` | 0 |
| `cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml --bin ccr-desktop commands::ssh:: -- --test-threads=1` | 0 |

`just desktop-test`：Python 12 通过；AccessKit 回归 3 通过；库测试 98 通过；`native_shell` 8 通过，其中包含 13 个目的地和实色纸面；`native_ui` 2 通过。环境段落的测试在这 98 项里，包含离开设置页取消进行中的读取、suspend 或更新一代之后的终态结果不替换可见报告，以及没有进程内信任时拒绝重连。

`ccr-app` environments：13 通过，1 个忽略。忽略的仍是会查询本机 `wsl.exe` 的探针，这一轮没有跑它。Tauri `commands::ssh` 15 通过，其中包括原有的活动目标回退，以及重连拒绝和从未连接的断开报告。没有重跑整个 `ssh` 过滤，也没有执行会打开套接字的连接测试。
