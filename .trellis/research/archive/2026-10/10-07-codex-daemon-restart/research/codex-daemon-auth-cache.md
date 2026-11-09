# 事件记录：codex app-server 守护进程缓存账号导致切换未生效

日期：2026-10-07（事件 2026-10-07 00:00 本地，UTC-5）。平台 Windows 11，codex CLI 0.160.1。

## 症状

ccr 切换 codex 账号显示成功（Switched to account: khanh），但 codex 仍显示与使用旧账号（teacher）。

## 根因

codex 0.160.1 的 TUI 与桌面端会话由共享本地 app-server 守护进程服务。守护进程在启动时读入 auth.json 并缓存，运行期间不重读；ccr 切换只改文件（auth.json 与注册表）。守护进程存活时，切换对 codex 客户端不生效。

## 证据链

1. ccr 侧成功：auth.json mtime 2026-10-07 00:00:04.129 = khanh；teacher 换出备份 auth.runtime_switch.20261007_000004.json.bak；注册表 current_auth=khanh；无 verify 警告。
2. 会话归属旧账号：00:00:12 线程 01a114bb 的 rollout session_meta 与 state_5.threads 均记 creator_account_id=c9d4fcf8（teacher），0 token；它是切换后唯一的新线程。
3. 服务者：logs_2.sqlite 中该线程全部日志 process_uuid=pid:6032x，thread/start 经 unix_socket；00:00:07-08 TUI 进程 pid:6019 连接守护进程，发 account/read 与 thread/start。
4. 守护进程从未重读：PID 60320 启动于 22:16:29（比 ccr 切到 teacher 晚 10 秒），运行至检查时（00:55 后仍在）；auth::manager 日志在其生命周期内无 Reloading auth / logout 记录，00:00 前后无 auth 相关日志。
5. 对照：新起 codex 进程探测返回 planType=plus（khanh）；codex doctor 的 auth 段读文件 = khanh；10-05 21:23 与 10-06 09:43 两次切换后守护进程出现 `Reloading auth / Reloaded auth, changed: true`（都伴随 TUI account/logout），新会话随即换账号。

## 机制细节

- 守护进程状态：~/.codex/app-server-daemon/daemon.pid = {"pid":60320,"processStartTime":"134358165894314170","executableIdentity":{"digest":[...]}}；控制套接字 ~/.codex/app-server-control/app-server-control.sock。
- 管理命令（0.160.1 实测）：codex app-server daemon version（JSON status/pid/版本）；codex app-server daemon restart / stop；codex remote-control stop。
- 重载路径推断：启动时加载 + 登出后的重载；持有有效缓存时不重读（按日志推断，未核对 codex 源码）。

## 影响面

- Cursor 扩展的 codex 进程与桌面端同样按进程生命周期缓存账号。
- Orca runtime-home（AppData/Roaming/orca/codex-runtime-home）与 ~/.codex 约 40 秒同步延迟。
- ccr verify_runtime_matches 与 codex doctor 都只读文件，无法发现守护进程缓存。

## 修复通道

`codex app-server daemon restart` 后守护进程重读 auth.json；重启前新开的 codex 会话仍用旧账号（预期现象）。

## 实施后现场验证发现（2026-10-07 追加）

修复前的端到端验证发现第二个缺陷：经 ccr 重启的守护进程存活不过数秒。

- 复现：`ccr codex auth switch khanh --restart-daemon` 报告重启成功并给出新 pid 56244；随后探测：控制套接字拒绝连接（10061），tasklist 无 56244，logs_2.sqlite 无该 pid 任何行。
- 对照实验：同机直接运行官方 `codex app-server daemon restart`（一次无旧守护进程、一次带旧守护进程）→ 新守护进程 65928、54892 均存活（12 秒后 status running）。旧守护进程竞态排除，唯一差异维度是 ManagedProcess 的作业对象。
- 机制（crates/ccr-core/src/core/process_gateway.rs）：重启命令经 ManagedProcess 放入作业对象；`wait()` 在子进程退出后无条件 `terminate_forceful`（Windows 实现 = TerminateJobObject，:110-118、:360-375），守护进程作为孙进程在作业内，随命令退出当场被杀；作业 kill-on-close（:308）与 Drop（:164、:393）是第二、三道同类保险。
- 结论：成功路径下守护进程是本命令的产物而非残留，ManagedProcess 的「wait 后清理整棵树」合同与该用途冲突。修复方向：新增 opt-in 的 spawn_detached（成功等待不杀树、作业不设 kill-on-close、drop 不杀树；超时显式 terminate_tree 仍回收整棵树）。

## 管道继承探针（2026-10-07 追加）

目的：判定修复后「成功分支无界排空等待」是否会让 ccr 挂住——守护进程存活时是否仍持有重启命令的 stdout/stderr 管道写端。

方法：python 以 PIPE 捕获 `codex app-server daemon restart` 的 stdout/stderr，命令退出后 5 秒内检查双管道是否 EOF。

结果：cmd_rc=0；stdout 5 秒内 EOF（315 字节 JSON）；stderr 5 秒内 EOF（0 字节）。守护进程不继承管道写端（自身 stdio 重定向到 app-server-daemon 日志）。修复后成功分支的排空等待在命令退出后正常返回，不挂起。残余脆弱点：该行为依赖上游 0.160.1 的实现，若未来改变，需给排空加界。

## 现场验证通过（2026-10-07 追加）

修复后经修正脚本（文件重定向 + 每步 timeout + CODEX_HOME 钉住真实 home）现场验证，三条路径全绿：

- [1] `ccr codex auth switch khanh --restart-daemon`（守护进程 38660 在运行）：rc=0，"检测到 app-server 守护进程 (PID 38660)，正在重启..." → "成功: app-server 守护进程已重启 (新 PID 48424)"；命令退出 3 秒后探测：running，pid 文件 48424。新守护进程存活。
- [2] `ccr codex auth switch teacher`（无标志，非交互）：rc=0，输出 "警告: app-server 守护进程 (PID 48424) 仍在使用切换前的账号；运行 codex app-server daemon restart 使切换对 Codex 客户端生效"；守护进程 pid 不变（按设计不重启）。
- [3] `ccr codex auth switch khanh --restart-daemon`：rc=0，重启至新 PID 50780；退出后探测 running。终态：auth.json=khanh，守护进程 50780（在 khanh 写入后启动，服务 khanh）。
- 副产物观察：把 ccr 输出接进管道（`| tail`）时，管道 EOF 会被存活的守护进程继承的句柄拖住（脚本曾因此挂起）；真实终端与本脚本的文件重定向均无此问题，每步 3-4 秒完成。
