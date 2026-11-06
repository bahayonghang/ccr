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

## 2026-11-05 续作：重连不能只凭旧进程信任

上文是提交 `c9c9578a` 对应的历史切片，保留原始命令与结果。这里记录 `5a0c6bdb` 之后的独立续作。历史末段没有跑 `just ci` 不能作为本轮豁免：本轮涉及 `ccr-app` 与 GPUI 消费路径，独立 check 必须运行根 `just ci` 并记录真实退出码。

修复前，新回归 `remembered_trust_does_not_replace_the_current_host_key` 实际失败：`Again`、`process_trusted=true`、`stored=None`、`observed=None` 返回 `Ok(())`，预期为 `ssh_host_key_untrusted`。共享决策现在拒绝该组合。其余缺失/空白/单边 key 组合继续拒绝，已有当前 key 匹配且进程信任的路径仍由旧测试覆盖。

GPUI 夹具在 SSH 环境状态下经 `EnvironmentView::plan_connection` 验证该拒绝。这个状态依旧由测试写入，不是真实环境切换器。生产 `Open` 唯一消费路径是 Local 设置刷新时的空 facts 检查；它没有调用 `connect_internal`、套接字或身份读取，其 `Ok` 不构成 SSH 授权。未改变 Tauri 命令签名或载荷。

| 本轮 implement 命令（仓库根） | 退出码 / 结果 |
| --- | --- |
| `cargo test -p ccr-app --locked --lib remembered_trust_does_not_replace_the_current_host_key -- --skip export_bindings`（修复前） | 1；新回归以实际 `Ok(())` 复现缺口 |
| `cargo test -p ccr-app --locked --lib environments -- --skip export_bindings`（修复后） | 0；18 通过，1 ignored |
| `just desktop-fmt-check` | 0 |
| `just desktop-lint` | 0 |
| `just desktop-test` | 0；默认并行，未重试 |
| `git diff --check` | 0 |
| `just ci`（独立 check） | 0；完整 16 阶段通过，14:13.540，未重试 |

最初共享 cargo 调用被沙箱拒绝访问既有 `D:\mbx-cache`，申请既有构建缓存访问后才获得上述真实回归结果；没有改缓存配置。desktop 使用既有独立 target。lint/test 有增量缓存 hard-link 失败后复制的警告，退出码仍为 0；未降低检查或测试要求。

本轮没有运行 `wsl`、`ssh-keyscan`、网络连接或真实密码/身份文件读取。9.1 实机读写仍 `not_run`；9.2 实机 SSH 仍 `not_run`；9.3 真实切换仍 `not_run`；9.4 退出进程树/task/watcher 清理仍 `not_run`；9.5 macOS/Linux 原生壳仍 `not_run`；9.6 原生 light/dark glass/solid 截图仍 `not_run`。托盘与 Windows 隐藏/恢复维持 `unsupported`，纸面维持实色。未改侧栏 13 入口、文案或主题。全部 9.1–9.6 验收框未勾选，任务保持 `in_progress`，没有归档或提交。

### 本轮独立检查终态

独立 [check 报告](research/check-20261105.md) 核对了实际调用链、完整切片、规范和逐项验收边界。根 `just desktop-fmt-check`、`just desktop-lint`、`just desktop-test` 均退出 0；`ccr-app` 全套默认并行测试退出 0（91 通过、4 ignored），all-target/all-feature 严格 Clippy 退出 0。真实 WSL probe 保持 ignored；没有把夹具测试当作原生收据。

跨模块根 `just ci` 实际退出 **0**，16 阶段全绿，耗时 **14:13.540**。本次进程 PATH 前置 `C:\Program Files\Git\bin`，确认实际 bash 为 `C:\Program Files\Git\bin\bash.exe`。日志在 `outputs/desktop/gpui09-check-20261105/`，其中 `just-ci.log` 保留完整结果；没有弱化覆盖率、测试或审计要求，没有测试重试。

检查没有发现需再修的产品问题；只将规范中的本轮 WSL 禁令改写为长期 fixture/授权边界。`task.py validate` 和 `git diff --check` 退出 0，检查完成时没有范围外脏文件。该切片可本地提交，9.1–9.6 仍开放，任务不能归档。

## 2026-11-05 续作：Windows 合成退出清理

新增 `ccr-desktop/tests/runtime_shutdown.rs`，通过现有运行时和命令 owner 验证视图 detach 后退出清理、真实合成父/后代进程句柄退出、任务归零与拒绝新工作；另验证 200ms quit observer 超时后最终 owner join 仍等待阻塞工作。只新增测试，没有修改产品生命周期实现。

详细实际命令、初始失败、Windows 后代句柄信号间隙和并行工作边界见 [退出生命周期夹具收据](research/exit-lifecycle-20261105.md)。根 `desktop-fmt-check`、`desktop-lint` 最终均 0；完整 `desktop-test` 首次遇既有 SQLite 字节比较失败，按授权重跑一次后 0。新 suite 为 2 passed / 1 closed helper ignored；helper 由进程树测试显式启动。所有行为 suite 保持默认并行和 `--skip export_bindings`。

这份证据是 Windows OS 合成 process/task 与共享运行时集成，不是用户关闭原生窗口收据，也不覆盖 watcher、真实 sidecar 或其他 OS。9.4 整项仍未通过；其他 9.1–9.6 的 not_run/unsupported 状态保持，未归档、未提交。

独立 [退出检查收据](research/check-exit-lifecycle-20261105.md) 修正了自然退出假通过：从命令入队前至整个 shutdown 和句柄确认必须小于 5 秒，排除 10 秒自然结束；共享测试规范同步这一因果断言。首次独立完整 desktop-test 退出 **101**，原因是新 Codex 默认存储语义触发旧 GPUI 断言（实际 NotDetected、期望 StoreUnsupported），失败日志保留。

用户随后明确要求使用当前工作区的新默认 `File` 标准，不建立新 worktree。独立 check 将 `platforms.rs` 的 `auto` 夹具显式配置为 auto，并新增缺 config/缺 store 设置且无 auth 文件的 `NotDetected`、未登录、无账户及不写入/不泄漏断言。生产映射无需修改；该消费测试依赖另一任务的共享默认值改动。

最终当前工作区 `desktop-fmt-check`、`desktop-lint` 和完整 `desktop-test` 均退出 **0**，无本次重试；Codex 窄测 2/2、新退出测试 2/2、库 103、native_shell 8、native_ui 2、Python 12、AccessKit 3 均通过。最终完整门禁覆盖了修正后的整个本切片，不是旧基线隔离通过；`just ci` 本切片未运行。此前隔离 worktree 仅跑过 fmt，未 build/lint/test，已由主会话移除，日志/输入清单保留。9.1–9.6 与任务继续开放。

## 2026-11-05 续作：主窗口关闭先确认

`Shell` 现在拦截主窗口关闭并显示当前语言的实色确认框。保守策略为每次关闭都确认；取消保留现有草稿、域确认及活动任务，明确退出后才移除窗口并进入已有退出链。Kit 的通用 Enter 默认会确认，因此实现同时限制 Cancel 键盘路径和首帧 root 的确认权限，避免仅有 Cancel 外观焦点却意外退出。

[关闭确认收据](research/close-confirmation-20261105.md)：新测试先以缺少确认框退出 **101**；最终窄测 **0**（2/2，包含中英键盘流程与嵌套草稿/任务）。根 `desktop-fmt-check`、`desktop-lint`、完整 `desktop-test` 最终均 **0**，完整测试没有重试；库 105、native_shell 8、native_ui 2、退出生命周期 2（闭合 helper 1 ignored）、Python 12、AccessKit 3 均通过。产品改动仅在 desktop，无跨模块产品修改，本切片未运行 `just ci`。

这些是 rendered/headless 回归，最终运行时 shutdown 在测试中显式调用；不等于真实用户关闭 OS 原生窗口，不证明托盘、watcher、macOS/Linux、远程环境或 glass。9.1–9.6 仍未整项通过；尺寸/DPI 继续延后，未归档、未提交。

独立 [关闭确认检查](research/check-close-confirmation-20261105.md) 的 fmt、all-target lint/type-check、close 窄测均退出 0；产品未再变动，完整 suite 沿用上列同切片收据。检查明确边界为主 Shell 的 should-close callback，Probe、启动失败退出及绕过该 callback 的 application-level Quit 未由本切片覆盖；不声称全部退出入口均已确认。

## 2026-11-06 核心功能连续实施（进行中）

基线为干净 `dev@7473927b`；复用当前工作区和既有任务，未新建 worktree 或任务。工程批次已独立检查并本地提交 `13b29f95`，之后继续环境、生命周期和父台账的实际业务缺口。

- SSH：已接保存主机 CRUD、明确的 Local/SSH 选择、指纹 challenge/确认、nonce 连接、取消及断开；固定平台源文件经 SFTP 读取和写前版本检查/远程备份保存。文档绑定 session，草稿按 host/file 归属，Local-only 页面阻止跨环境操作。完整独立检查和最新门禁尚待完成，真实 SSH 与原生切换未运行。
- 生命周期：主题/语言/退出确认使用独立 GPUI 偏好文件；菜单、设置按钮与快捷键共用窗口 guard。独立 check 修复相对 fixture 路径拒绝后，close_tests 3/3、preferences 3/3、native_shell 8/8 退出 0。系统直接 App::quit 缺公开 veto hook，不宣称覆盖。
- Settings 不自动运行 WSL；fixture 有 SSH 数据库时仍拒绝真实网络，rendered 测试通过显式内存 transport 运行同一 action/runtime 链。未连接真实 SSH、未运行 keyscan/doctor/发行版探测、未读用户密码或身份文件。
- 01 的 macOS 编译和原生跨平台收据仍缺；托盘/Windows 隐藏恢复仍受锁定 GPUI 能力限制；尺寸/DPI 仍 user-deferred。所有任务与未满足 AC 保持开放。

此段是中间检查点，不代表最终三门禁、完整 CI、原生系统操作或整项迁移通过；最终结果在后续段落明确记录。

### 核心操作批次检查点

现已接通 SSH 主机/信任/连接与四平台配置文件、独立主题/语言/退出偏好、MCP 服务 CRUD、WebDAV 设置与文件夹 CRUD/测试/推拉、服务商/账户 CRUD 与签到/余额/记录。共享 SSH 安全和 MCP 策略由 Tauri/GPUI 共用；Tauri 签名及载荷保持。父功能台账更新 41 个明确命令映射，全部保留 partial/pending-native 状态，不把整个模块视作完成。

独立 check 完整审查并修复信任探测后旧 session 权限、取消上传清理错误、旧确认目标、fixture 路径隔离等边界。集中检查中还修正 Settings 区块 flex 压缩造成的实际点击遮挡；MCP 测试等待内层输入引擎通知，关闭测试等待 Kit 使用真实墙钟的滑入动画完成，再执行一次真实取消点击。后者原本在并行 suite 间歇失败，已捕获 `cancel dialog dismissal` 阶段；没有扩大 10 秒等待、重复点击、串行测试或弱化断言。普通 sandbox 的 loopback/Windows 加密限制与产品缺陷分开记录，最终隔离 fixture 测试使用已授权本机权限。

- `just desktop-fmt-check`、`just desktop-lint` 退出 0。
- 完整 `just desktop-test` 退出 0：库 120、native_shell 8、native_ui 2、runtime_shutdown 2 通过（闭合 helper 1 ignored）；默认并行与 `--skip export_bindings` 保持。最终日志 `outputs/desktop/core-completion-20261106/desktop-test-complete.log`。
- 共享 `ccr-app --lib`：106 passed / 2 ignored；`ccr-checkin`：88 passed；均为 locked/offline、默认并行。未改动部分沿用本批已取得结果。
- Tauri 仅有 bin target；`cargo check --locked --offline --bin ccr-desktop` 退出 0，随后移除提取导致的无调用薄包装。没有启动 Tauri 或执行环境探测。
- CCR release 侧车在 `ccr-desktop/target/sidecar` 实际构建退出 0；完整便携 ZIP 与新候选原生验收尚未运行。

完整 `just ci` 留待后续资源/窗口状态批次最终整合执行一次。当前核心操作源码与 fixture/headless 已交付，真实 SSH、原生环境切换、外部 WebDAV/签到、macOS/Linux 和系统直接退出仍未获原生收据。WSL 与尺寸/DPI 保持 user-deferred，托盘/Windows 隐藏恢复受当前依赖限制。三个任务及全部未满足 AC 继续 in_progress；接着实施动态资源、source-agent、MCP 预设与窗口状态，不以本检查点结束整项请求。
