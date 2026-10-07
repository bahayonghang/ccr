# Codex 切换后感知并重启 app-server 守护进程

## 目标与用户价值

ccr 切换 codex 账号后，如果本机 codex app-server 守护进程正在运行，用户能立即知道守护进程仍持有旧账号，并可在确认后一键重启，使切换对 codex 客户端立即生效。消除「ccr 提示切换成功但 codex 仍显示旧账号」的误判。

## 背景与证据

2026-10-07 00:00：ccr 切到 khanh 成功（auth.json 00:00:04 写入、备份与注册表齐全），但 codex 0.160.1 的共享 app-server 守护进程（PID 60320，22:16:29 启动）在启动时缓存 teacher，运行期间不重读 auth.json；00:00:12 的 codex TUI 会话经 unix socket 由该守护进程服务，界面与会话归属仍是 teacher。完整证据链见 [research/codex-daemon-auth-cache.md](research/codex-daemon-auth-cache.md)。

现状代码（行号为规划时快照，实施前复核）：

- `CodexProcessService`（crates/ccr-codex/src/services/codex_process_service.rs:248）已有窄 argv 匹配的 app-server 枚举与清理（`find_app_servers` / `cleanup_report`），被 `ccr codex fix` 使用；它命中全部 app-server（含托管守护进程、扩展、桌面端），但不区分托管守护进程，也没有重启语义。
- CLI（crates/ccr-cli/src/commands/codex/auth/switch.rs:44）与 TUI（crates/ccr-tui/src/tui/codex_auth/app.rs:1316）在切换前只给通用进程警告；仓库没有 app-server 守护进程感知代码。
- 官方通道已存在并实测可用：`codex app-server daemon version`（JSON：status/pid/版本）与 `codex app-server daemon restart`；守护进程状态文件 `~/.codex/app-server-daemon/daemon.pid` 含 pid、processStartTime、executableIdentity.digest。

规划基线：dev @ 9fafc111。工作树含 10-06 CLI 展示迁移的在途改动，与 ccr-cli / ccr-tui 同文件，实施前重读最新 diff。

## 要求与范围

| ID  | 要求                                                                                                                                                                                      |
| --- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| R1  | 切换成功后检测托管守护进程是否在运行：daemon.pid 解析 + 现有窄匹配枚举按 pid 交叉核对，owner 边界与 argv 规则沿用 codex-app-server-cleanup 合同；检测不发信号、不读凭据、不启动外部进程。 |
| R2  | 交互式 CLI 在检测到守护进程时询问是否重启；确认后经 `codex app-server daemon restart` 执行并报告结果；拒绝或失败不改变切换结果与退出码。                                                  |
| R3  | CLI 新增一个正向标志（暂定 `--restart-daemon`）跳过询问直接重启；非交互 / `--json` 环境默认不询问、不自动重启，只输出警告与手动命令。                                                     |
| R4  | TUI 在切换成功后检测到守护进程时给出警告并提供重启确认（复用既有 Overlay 确认模式）；重启在后台执行，结果以 toast 呈现；EN/ZH 同步。                                                      |
| R5  | 降级：codex 不在 PATH、子命令不支持、非零退出或超时，统一转为警告 + 手动命令提示，不报错、不阻塞、不 panic。                                                                              |
| R6  | 保留现有行为：切换前进程警告、备份、原子写、脱敏、verify_runtime_matches、`ccr codex fix` 全部合同（codex-app-server-cleanup.md）不变。                                                   |
| R7  | 兼容：JSON DTO、退出码、既有参数签名不变；新增用户可见文本遵守 10-06-cli-output-presentation 的样式约定。                                                                                 |

## 非目标

- 不修改 codex 本体；不通过信号 kill 守护进程（重启只走官方命令，避免锁文件、daemon-updater 与 code-mode-host 子进程孤儿）。
- 不做后台常驻监控、自动轮换、跨客户端强制一致。
- ccr-ui 桌面面板与 VS Code 扩展不在本任务接线；服务层能力就绪后由后续任务复用。
- 不自动重启未经确认的守护进程（R2/R4 的确认是硬要求；自动路径仅由显式标志触发）。

## 验收标准

- [ ] AC1：守护进程运行时交互切换出现重启询问；确认后守护进程重启（`codex app-server daemon version` 返回新 pid、status running），新 codex 会话 creator_account_id 为新账号。
- [ ] AC2：无守护进程时输出与现状一致（无新增行）。
- [ ] AC3：非交互 / `--json` 不出现询问，JSON 可解析，警告走 stderr，退出码与切换结果一致。
- [ ] AC4：codex 不在 PATH / 子命令失败 / 超时 → 警告 + 手动命令，无 panic、无孤儿进程。
- [ ] AC5：`--restart-daemon` 直接重启；失败仍保留切换成功语义。
- [ ] AC6：TUI EN/ZH 文本与尺寸矩阵通过，确认与 toast 有运行时证据。
- [ ] AC7：`just lint-strict`、`just test` 通过；新增单测覆盖 daemon.pid 解析、枚举交叉核对、假 CLI 四态（成功/非零/超时/缺失）与 CLI 非交互分支。
- [ ] AC8：codex-app-server-cleanup.md 同步更新守护进程检测与重启合同；`ccr codex fix` 既有测试不变。

## 依赖与状态

无父任务。与 10-06-cli-output-presentation / 10-06-auth-cli-feedback 共享文件，实施需在其最新 diff 之上进行。实施需用户审阅本 PRD 与 design.md / implement.md 后 `task.py start`；提交、推送、归档等待单独授权。
