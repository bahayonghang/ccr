# Design - `ccr codex fix` 进程清理行为等价性

## 1. 目标与边界

目标是让 `ccr codex fix` 在“发现并清理当前用户 Codex app-server”这一核心行为上不弱于
本地脚本，同时保留 CCR 的跨平台、dry-run、runtime reconciliation、显式 repair 和脱敏。

本设计不验证 Provider 远端 key，不修改 profile 存储格式，也不扩大到其他 sysinfo 调用。

## 2. 分层与数据流

```text
SysinfoProcessBackend
  -> owner-scoped process snapshot
  -> argv-aware app-server classifier
  -> cleanup state machine (TERM -> poll -> KILL -> settle)
  -> CodexAppServerCleanup
  -> CLI process rendering
  -> optional CCR runtime inspection/repair
  -> supplemental codex doctor
  -> final stage summary + exit code
```

- `ccr-codex` 拥有进程发现、身份校验、信号与状态机。
- `ccr-cli` 只编排、渲染和合并阶段结果，不复制进程逻辑。
- 公共 `CodexProcessService::new()` / `cleanup(dry_run)` 与现有结果类型保持兼容；CLI 使用
  新增的详细报告入口。

## 3. 进程快照

新增内部 `SysinfoProcessBackend`，集中封装 refresh 和 signal：

- 使用 `System::refresh_processes_specifics`。
- refresh kind 只启用 `cmd(UpdateKind::Always)`、`user(UpdateKind::Always)`，关闭 tasks；
  不读取 memory/cpu/disk/cwd/root/environ。
- 每次全量发现都重新刷新 cmd 与 owner，避免命令行变化和 PID 复用使用旧数据。
- 内部进程身份包含 `pid + start_time`；信号发送前必须再次确认身份、owner 和 classifier。
- 以当前 CCR 进程的 owner 为边界。Unix 优先 effective owner；Windows 使用可用的 owner ID。
  当前 owner 无法解析时不发送信号，并返回结构化 discovery issue。

不直接使用 `System::new_all()`，因为它读取过多数据且掩盖调用方真正依赖的字段。

## 4. 分类与展示

分类基于 argv，而不是 join 后的无序子串：

- 命令序列中存在 Codex 启动项（`codex` / `codex.exe`，允许 node wrapper 路径）。
- 其后存在精确 `app-server` 子命令。
- 普通 `codex`、`exec`、`resume`、`login` 均不命中。
- 自排除使用当前 PID/身份；删除任意路径含 `ccr` 即排除的规则。

匹配使用原始 argv，展示使用安全摘要（PID、可执行标识、`app-server`），不直接打印可能
携带配置覆盖或 secret 的完整命令行。

## 5. 清理状态机

### 5.1 Dry-run

只执行 owner-scoped discovery 并渲染候选；不发信号、不进入轮询、不写 runtime。

### 5.2 实际清理

1. 发现初始候选并对每个身份发送 TERM。
2. 每 300ms 全量重查当前用户匹配进程，最多约 3 秒。
3. 将轮询期间出现的新身份记录为 `discovered_during_cleanup`。
4. 到期后，对“当前仍匹配且身份仍相同”的全部候选发送 KILL，不限于初始 PID。
5. 等待 1 秒后再次全量重查；最终仍匹配者进入 `respawned_or_remaining`。
6. 终止类型只在信号成功且后续确认身份消失后记录。`Some(false)` / `kill()==false`
   进入 signal failure，不得渲染为成功。

该算法保留脚本对快速换 PID 的处理能力，同时用 owner 和进程身份校验降低误杀风险。

## 6. 结果模型

为避免给公开 struct 增加字段造成 Rust 源码级破坏，保留现有
`CodexAppServerCleanup` 和 `cleanup(dry_run)`，新增 `cleanup_report(dry_run)` 返回
`CodexAppServerCleanupReport`。旧 `cleanup()` 委托新状态机并投影为旧结果；CLI 改用
详细报告。报告包含：

- `cleanup.found`：旧结果中的初始候选。
- `discovered_during_cleanup`：TERM 后新出现的匹配身份。
- `cleanup.terminated`：已由后续快照确认消失的终止结果。
- `signal_failures`：发送失败及阶段信息。
- `cleanup.respawned`：settle 后仍匹配的进程；兼容现有字段名，语义包含重生或未能终止。
- `discovery_issue`：owner/cmdline 快照不可用等 fail-closed 原因。
- `cleanup.dry_run`：保持现有语义。

CLI 的 `process_state` 取值保持 `clean` / `dry_run_found` / `cleaned` / `respawned`，新增
`unavailable` 表示无法安全建立 owner-scoped 快照。退出码 `2` 仍表示最终进程不干净。

## 7. CLI 阶段隔离

`fix_command` 按以下顺序编排，每一阶段保存结构化结果，不用 `?` 直接跳过后续诊断：

1. process cleanup（始终首先执行并渲染）。
2. `codex` PATH 检查。
3. CCR runtime inspection；失败时输出 `runtime_consistency = unavailable`。
4. 仅在 inspection 成功、存在漂移、`repairable=true` 且显式 `--repair-runtime` 时 repair。
5. 环境提示。
6. `codex doctor`；保持有效报告优先、脱敏和超时降级。
7. 输出最终阶段总结并按固定优先级退出。

退出优先级保持兼容并补充 unavailable：

1. `127`：PATH 中没有 `codex`。
2. `2`：最终仍有 app-server，或无法安全完成 process discovery/termination。
3. `1`：CCR runtime inspection/repair 本身失败。
4. `3`：本地漂移或 doctor 期间快照变化。
5. `0`：上述问题均不存在；Provider 有效性仍可能为 `not_checked`。

Doctor 检查项失败但提供有效报告时仍是补充证据，不改变退出码；这保留当前项目已决策行为。

## 8. 可测试性

生产 backend 保持私有；状态机通过内部 backend 接口运行，测试使用预编程快照和信号结果，
无需真实等待。必须覆盖：

- 默认 refresh kind 确实加载 cmd 和 owner。
- 同用户 node wrapper / native binary 命中，其他用户与普通 Codex 命令不命中。
- TERM 后退出、TERM 超时后 KILL、宽限窗口内新 PID、KILL 后重生。
- `Some(false)` / `kill=false` 不计成功。
- PID 相同但 start_time 改变时重新分类，不按旧身份发送信号。
- owner 无法确定时 fail closed。
- Unix 上启动一个同用户伪 app-server 子进程，证明真实 discovery 链路可见；该测试只清理自己
  创建的 fixture，并用 Drop guard 回收。
- CLI fixture 覆盖 runtime inspection 失败仍输出 process 和 doctor 阶段状态。

## 9. 兼容与回滚

- CLI 命令和 flags 不变。
- 裸命令继续不写 runtime；`--repair-runtime` 授权边界不变。
- 不新增生产依赖，继续使用 `sysinfo 0.38.x`。
- 若状态机回归，可先回滚动态 PID/结果模型，保留显式 cmd refresh 的最小修复；但不可回到
  默认 `refresh_processes`，否则主功能再次失效。
