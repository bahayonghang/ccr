# 修复 ccr codex fix 与 codexfix 行为差异

## Goal

让 `ccr codex fix` 在清理当前用户残留 Codex `app-server` 的核心效果上不弱于
本地 `/home/lyh/.local/bin/codexfix`，同时保留 CCR 已有的跨平台、安全匹配、
profile/runtime 诊断、脱敏和显式修复能力。

## Background

- 用户现场确认：本地 `codexfix` 能恢复 URL/Key 切换效果，而当前
  `ccr codex fix` 不能达到相同结果。
- 2026-07-23 现场复现：当前 UID `1001` 下存在 PID `3479399` 和 `3479422`
  两个真实 Codex app-server；脚本使用的 `pgrep` 条件可命中，但
  `ccr 6.5.3 codex fix --dry-run` 输出 `process_state = clean`。
- 主根因已确认：当前 sysinfo 快照没有加载 cmdline，导致真实 app-server 永远无法命中。
- 脚本会在 TERM 等待期间动态重查进程；Rust 实现只跟踪初始 PID，并且会把部分信号失败
  误报为成功。完整证据见 `research/root-cause-analysis.md`。
- 前两次归档任务已实现基础进程清理和本地 runtime/profile 诊断。本任务只修复行为差异
  和回归防护，不重复扩展无关能力。

## Requirements

- `ccr codex fix` 必须从真实进程快照可靠发现当前用户的目标 app-server。
- 清理必须覆盖初始 PID 和宽限窗口内出现的新匹配 PID，最终结果以复检事实为准。
- 每次发送信号前必须重新确认进程身份、当前用户归属和 app-server 匹配状态。
- 不得误杀普通 `codex` / `codex exec` / `codex resume`、CCR 自身或其他用户进程。
- 信号失败、进程重生和无法安全发现进程必须输出可区分的状态，不得渲染为已清理。
- 进程清理、CCR runtime inspection/repair 和 `codex doctor` 必须分别报告阶段结果；
  后续诊断失败不得掩盖已执行的进程阶段。
- 保留 `--dry-run` 无信号、无 runtime 写入的契约；只有 `--repair-runtime` 才能重放
  可安全修复的 profile，且继续使用既有原子提交路径。
- 输出不得泄露完整命令行中潜在的 secret 或敏感配置覆盖。
- 为真实发现链路、PID 更替、快速重生、信号失败、用户边界和诊断失败补充回归测试。
- 同步双语文档和 `.trellis/spec/ccr-codex/backend/codex-app-server-cleanup.md`。

## Acceptance Criteria

- [x] 受控同用户 app-server fixture 能被 `ccr codex fix --dry-run` 发现，不再错误报告
      `process_state = clean`。
- [x] 在相同伪进程场景下，本地脚本与 `ccr codex fix` 都能清理初始 PID，并对宽限窗口
      内出现的新匹配 PID 给出等价或更可靠的处理结果。
- [x] 普通 Codex 命令、CCR 自身、其他用户进程和 PID 复用后的无关进程不会收到信号。
- [x] `Some(false)` / KILL 失败不会被记录为 `Term` / `Kill` 成功；最终仍存在的目标
      稳定报告为 `respawned` 或 `unavailable`，并返回进程阶段退出码 `2`。
- [x] CCR runtime inspection 异常和 `codex doctor` 异常都有独立、稳定的输出与退出语义，
      不会跳过或误报进程阶段结果。
- [x] `ccr codex fix --dry-run` 不发送信号、不写入 `config.toml` / `auth.json`。
- [x] `ccr codex fix --repair-runtime` 仍只在快照可安全修复时走既有原子提交路径。
- [x] 进程输出和 doctor 报告中不出现测试 sentinel secret。
- [x] 相关单元/集成测试、`just fmt-check`、`just lint-strict`、`just test` 通过；最终交付前
      `just ci` 通过。

## Out of Scope

- 不验证第三方 Provider 远端是否接受 API Key。
- 不改变 Codex Desktop 或 VS Code Remote-SSH 的自动拉起机制。
- 不重构无关的 Codex profile、session、quota 或 UI 功能。
- 不处理仓库内其他 `sysinfo::refresh_processes` 调用；另有真实缺陷时单独立项。
