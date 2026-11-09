# `ccr codex fix` 与本地 `codexfix` 行为差异分析

## 结论

当前命令无效的直接原因不是匹配表达式，也不是 runtime/profile 诊断，而是进程快照
根本没有加载 cmdline：`CodexProcessService` 从 `System::new()` 创建空快照后调用
`refresh_processes(...)`，而 `sysinfo 0.38.4` 的该便捷方法不启用 `cmd` 刷新。
`collect_app_servers()` 随后读取到的 `Process::cmd()` 全为空，所以任何真实 app-server
都会被报告为 `process_state = clean`。

这是 P1 功能缺陷：`ccr codex fix` 的核心清理阶段在当前实现中实际上是 no-op。

## 现场证据

2026-07-23 在当前仓库和本机已安装版本上执行只读检查：

```text
uid=1001(lyh)
ccr 6.5.3
codex-cli 0.145.0

pgrep -a -u 1001 -f 'codex.*app-server'
3479399 node /home/lyh/.npm-global/bin/codex ... app-server --listen unix://
3479422 /home/lyh/.npm-global/.../bin/codex ... app-server --listen unix://

ccr codex fix --dry-run
[INFO] process_state = clean
[OK] 未发现残留的 Codex app-server 进程
```

本轮没有执行本地 `codexfix`，因为它会真实终止承载当前 Codex 会话的 app-server；
脚本使用的同一条 `pgrep` 发现路径已单独验证。

## 源码因果链

1. `crates/ccr-codex/src/services/codex_process_service.rs:67-69`
   使用 `System::new()` 后调用 `refresh_processes(...)`。
2. `sysinfo-0.38.4/src/common/system.rs:309-323`
   该便捷方法只启用 memory、cpu、disk、exe 和 tasks，不启用 cmd 或 user。
3. `sysinfo-0.38.4/src/common/system.rs:2412-2425`
   `ProcessRefreshKind::default()` 的 `cmd` 和 `user` 都是 `UpdateKind::Never`。
4. `crates/ccr-codex/src/services/codex_process_service.rs:164-170`
   分类器只读取 `process.cmd()`；空数组 join 后为空字符串，必然不匹配。
5. 当前测试 `crates/ccr-codex/src/services/codex_process_service.rs:208-237`
   只直接调用纯字符串分类器，没有覆盖 sysinfo 刷新和真实进程发现链路。

## 行为对照

| 维度 | 本地 `codexfix` | 当前 `ccr codex fix` | 影响 |
| --- | --- | --- | --- |
| 进程数据 | `pgrep -f` 直接读完整 cmdline | 默认 refresh 后读 `Process::cmd()` | Rust cmdline 为空，主功能失效 |
| 用户范围 | `pgrep -u $(id -u)` | 不显式过滤 owner | `sudo/root` 下可能触碰其他用户 |
| 匹配 | 有序正则 `codex.*app-server` | 无序 contains，且排除任意含 `ccr` 的路径 | Rust 有额外漏匹配/误匹配边界 |
| TERM 等待 | 每轮重新枚举全部匹配 PID | 只跟踪初始 PID 集合 | 宽限窗口内的新 PID 不会被 KILL |
| KILL 目标 | 轮询末尾仍匹配的全部 PID | 初始且收到 TERM 的 PID | 与脚本恢复效果不等价 |
| 信号结果 | 忽略 kill 返回值，靠最终复检 | `Some(false)` 仍按 TERM 处理，`kill=false` 仍记为 Kill | 输出可把失败误报为成功 |
| PID 复用 | KILL 前重新按 cmdline 匹配 | 只按初始 PID 集合 | 极端情况下可能向复用 PID 发信号 |
| Runtime 诊断 | 与进程阶段解耦，直接 doctor | doctor 前后强制 CCR inspection | inspection 错误可提前结束后续诊断 |
| Doctor 结果 | 非零状态原样退出 | 有效报告优先，部分失败仅 warning | 属于有意增强，不是本次主根因 |
| Dry-run/脱敏 | 无 dry-run，报告原样保存 | 支持 dry-run、显式 repair、脱敏 | 应保留 CCR 增强能力 |

## 次级缺陷

### 1. 终止状态机只处理初始 PID

`codex_process_service.rs:86-149` 把初始 PID 固定进 `targets` / `term_sent`。
脚本则在每轮和 KILL 前重新执行 `pgrep`。如果 Desktop/Remote supervisor 在 TERM 后快速
换 PID，脚本仍会尝试清理新 PID，Rust 只会在 1 秒后把它报告为 `respawned`。

### 2. 信号返回值被错误解释

`Process::kill_with(Signal::Term)` 的类型是 `Option<bool>`：`Some(false)` 表示平台支持
该信号但发送失败。当前 `Some(_)` 分支把它加入 `term_sent`。后续 `process.kill()` 的
布尔返回值也被忽略，只要调用过就记录 `TerminationKind::Kill`。这违反初版设计中
“失败者不视为已清理”的约束。

### 3. 当前用户边界只靠 OS 权限

初版设计明确选择不做 user filter，但这只在非特权运行时近似安全；`sudo/root` 能向
其他用户进程发送信号。脚本的 `-u` 边界更明确。优化后应刷新 owner 信息，并在 owner
未知时 fail closed。

### 4. `!contains("ccr")` 是脆弱的自排除

`ccr codex fix` 本身没有 `app-server` 参数，本来就不会匹配。对完整命令行做
`!contains("ccr")` 会漏掉安装路径或启动参数恰好含 `ccr` 的真实 Codex app-server。
自排除应使用当前 PID/进程身份，而不是路径子串。

### 5. 诊断阶段错误会短路

`crates/ccr-cli/src/commands/codex/fix.rs:45-47` 和 `:99` 对 runtime inspection 使用
`?`。清理已经发生，但 inspection 错误会跳过环境提示、doctor 和统一状态总结。
进程清理、CCR runtime inspection、显式 repair 与上游 doctor 应分别建模状态。

## 测试缺口为什么没有拦住

- 进程服务仅测试纯匹配函数，没有真实或伪造 backend 快照测试。
- CLI 集成测试只验证 dry-run 不写文件和脱敏，并通过清空 `PATH` 提前退出；没有启动
  一个可发现的 app-server fixture。
- 没有快速重生、新 PID、`Some(false)`、KILL 失败、PID 复用或 owner 不同的用例。
- 既有规范把“刷新后 cmdline 可用”当作前提，没有把 `ProcessRefreshKind` 写成可执行契约。

## 推荐修复边界

本任务应同时完成：

1. 显式刷新 cmd 和 owner，恢复真实进程发现。
2. 清理循环按每次快照重新验证 owner、cmdline 和进程身份，并处理宽限窗口内的新 PID。
3. 只把已被事实证明退出的进程记录为成功；保留最终仍存在的进程及信号失败原因。
4. 让 runtime inspection/doctor 作为独立诊断阶段，保留现有 repair、脱敏和退出码兼容。
5. 用可控 process backend + Unix 真实子进程测试覆盖生产链路。

不建议只把 `System::new()` 改成 `System::new_all()`：它虽然能偶然修复 cmdline，但会读取
不需要的 CPU、内存、磁盘、cwd、env 和 task 数据，且仍无法解决用户边界、动态 PID、
信号误报与测试缺口。
