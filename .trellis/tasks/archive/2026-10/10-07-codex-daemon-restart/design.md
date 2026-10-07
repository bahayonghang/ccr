# 设计：守护进程检测与重启

## 归属与接口

能力放 ccr-codex（与 CodexProcessService 同文件或相邻），CLI / TUI 只做交互编排。

- `CodexProcessService::find_managed_daemon() -> Option<CodexDaemon>`：读 `CodexPaths::resolve().codex_dir/app-server-daemon/daemon.pid`（serde 解析，仅依赖 pid 字段；processStartTime 与 executableIdentity 保留为诊断信息），与现有 `discover()` 窄匹配目标按 pid 交叉核对；文件缺失、坏 JSON、pid 不在目标集均返回 None（debug 记录原因）。沿用 owner 边界与 argv 规则，不发信号。
- `restart_codex_daemon() -> DaemonRestartOutcome`（async）：`codex` 不在 PATH → `Unavailable`（PATH 探测在 ccr-codex 本地实现或复用其既有查找逻辑；不可反向引用 ccr-cli 的 `install_detect::which_on_path`，crates/ccr-cli/src/services/install_detect.rs:150）；否则 `ManagedProcess::spawn(codex app-server daemon restart)`，stdin 置 null，并发排空受限 stdout/stderr，30 秒截止（沿用 `ccr codex fix --doctor` 先例，Duration 提供可注入 seam），超时 `terminate_tree(grace)` 后返回 `Timeout`；退出码 0 → `Restarted`，非零 → `Failed`（输出经脱敏摘要，不落原始 stderr 全量）。
- 四态枚举：`Restarted / Failed / Timeout / Unavailable`，全部携带可渲染消息；不新增 CcrError 变体（变体冻结）。

## 交互编排

- CLI（crates/ccr-cli/src/commands/codex/auth/switch.rs 与 crates/ccr-cli/src/cli/subcommands/codex.rs:362 的 Switch 变体）：切换成功 → find_managed_daemon；命中且交互（stdin/stdout is_terminal 且非 JSON）：警告 + 确认询问（复用或扩展 commands/common/prompt.rs 的确认模式；该文件在 10-06 在途改动中，实施前对齐）；确认 → restart，结果按四态渲染（成功给 pid 摘要，失败给手动命令）。`--restart-daemon` 跳过询问。非交互：stderr 警告 + 下一步 `codex app-server daemon restart`，JSON DTO 不变。
- TUI（crates/ccr-tui/src/tui/codex_auth/app.rs）：切换成功 → 命中时警告 toast + 新 Overlay 确认变体（仿 Overlay::confirm_delete，app.rs:883）；确认 → 异步任务执行 restart（不阻塞 UI 线程），完成 toast 四态；EN/ZH 同步。
- 顺序固定：先完成文件切换与注册表更新，再检测与重启；重启失败不回滚、不改变退出码。

## 兼容与降级

- daemon.pid 不存在（无守护进程机器）→ None，零副作用。
- `codex` PATH 缺失（CI、精简环境）→ Unavailable 警告，不 exit 127、不 panic。区别于 `ccr codex fix --doctor` 的 127：那是显式请求上游 doctor 的语义；此处是切换后的增强路径。
- 超时与信号只作用于本次 spawn 的子进程树（ManagedProcess 合同），不触碰守护进程自身；重启失败不重试。
- 跨平台：daemon.pid 路径经 CodexPaths 解析（含 CODEX_HOME 覆盖）；ManagedProcess 已抽象 Unix / Windows 树清理。

## 风险与取舍

- 重启会中断守护进程正在服务的 codex 会话（官方 `daemon update` 帮助明确 may interrupt running work），因此默认必须确认；自动化仅显式标志。
- 不采用 kill pid 的取舍：托管守护进程有锁文件（daemon.lock / daemon-updater）与 code-mode-host 子进程，官方命令负责生命周期；kill 路径会绕过托管语义。
- 检测与重启分属两个入口：检测不启动外部进程（快、可测）；重启才 spawn codex。避免把 PATH 依赖带进纯检测路径。
- 与 10-06 展示迁移同文件：实施前重读 diff；新增文本遵守其 R1–R4 样式。
