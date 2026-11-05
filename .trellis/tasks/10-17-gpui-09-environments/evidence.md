# GPUI 09 切片证据：主机密钥比较与 WSL 文件目标

日期：2026-11-05。验收项 9.1–9.6 **没有**标为通过。macOS 与 Linux 的原生壳能力仍是 `not_run`。这一检查没有打开可见的系统窗口，因此没有像素截图。

## 这一片证明了什么

主机密钥比较只看指纹字符串。`classify_host_key` 做精确相等判断。`decide_host_key` 把空白文本当成没有指纹；其余文本不去掉两端空白再比较，所以 `" SHA256:same "` 和 `"SHA256:same"` 是 `Mismatch`。

`New`、`Mismatch` 和 `Matched` 的 `may_store`、`may_trust`、`may_accept` 都是 false。匹配只表示两段指纹文本相同，不写密钥、不打开套接字、不把主机标成已信任。决策值里没有指纹、密钥、密码、身份文件或 `ssh-keyscan`。这三个布尔字段是私有的，外部不能造出一个 `may_accept = true` 的决策。

Tauri 的 `security.rs` 改为再导出同一个 `classify_host_key` 和 `HostKeyStatus`。`ssh_probe_host_fingerprint` 与 `ssh_confirm_host_fingerprint` 的签名和载荷没有改。桌面决策没有调用这两条命令。Tauri 里原有的匹配后重建 known_hosts、以及确认后写入，仍留在命令层，这一片没有改它们，也没有重跑 Tauri SSH 套件。

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

## 仍是 not_run

- 9.1：没有在 Windows 上对真实发行版做 detect/read/write fixture。没有创建发行版。非 Windows 伪 WSL 入口这一条没有在 macOS/Linux 上跑。
- 9.2：没有对真实 SSH 做首次信任、key changed、认证失败、断线、重连或取消。没有 `ssh-keyscan`，没有 TCP，没有密码，没有读取身份文件。
- 9.3：没有证明环境切换后的缓存、编辑 draft 和 job 不串环境。
- 9.4：托盘仍是不受支持。没有隐藏/恢复按钮。没有退出后的进程树、task、watcher 清理证明。
- 9.5：macOS 与 Linux 原生壳是 `not_run`。Windows 通知只证明提交了请求，没有证明系统弹出。
- 9.6：没有 glass/solid 的原生截图。纸面保持实色。模糊探针、性能预算和系统减少透明度设置都没有跑。

更早一次本机 WSL 分类曾打印 `Listed { names: ["docker-desktop"], exec: PathNotFound }`。这一检查没有重跑被忽略的 `live_wsl_probe_prints_classified_result`，所以上面那行不是这一次的新证据。

## 命令与退出码

| 命令 | 退出码 |
| --- | --- |
| `just desktop-fmt-check` | 0 |
| `just desktop-lint` | 0 |
| `just desktop-test` | 0 |

`just desktop-test`：Python 12 通过；AccessKit 回归 3 通过；库测试 95 通过；`native_shell` 8 通过，其中包含 13 个目的地和实色纸面；`native_ui` 2 通过。环境段落的六项测试在这 95 项里。

另外：`cargo clippy -p ccr-app --all-targets -- -D warnings` 通过；`cargo test -p ccr-app --lib environments -- --test-threads=1` 为 10 通过、1 个忽略。lint 曾等待 package cache 文件锁，随后退出码 0，没有因此重跑。
