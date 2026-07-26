# Journal - lyh (Part 2)

> Continuation from `journal-1.md` (archived at ~2000 lines)
> Started: 2026-07-23

---



## Session 48: 修复 Codex fix 进程清理行为差异

**Date**: 2026-07-23
**Task**: 修复 Codex fix 进程清理行为差异
**Branch**: `dev`

### Summary

修复 sysinfo 未刷新 cmdline 导致 app-server 发现失效的问题，补齐同用户精确匹配、动态 PID 清理、信号结果与进程身份校验；隔离 process/runtime/doctor 阶段，新增回归测试并同步双语文档与 Trellis 规范。安装 cargo-audit 与 cargo-binstall，最终 CCR_SKIP_ICON_GENERATION=1 just ci 全量通过。

### Main Changes

- Detailed change bullets were not supplied; see the summary above.

### Git Commits

| Hash | Message |
|------|---------|
| `14a3c677` | (see git log) |
| `ed780638` | (see git log) |

### Testing

- Validation was not recorded for this session.

### Status

[OK] **Completed**

### Next Steps

- None - task complete


## Session 49: 实现 ccr project init 项目初始化命令

**Date**: 2026-07-23
**Task**: 实现 ccr project init 项目初始化命令
**Branch**: `dev`

### Summary

新增 ccr project init，幂等编排 Git、原生 Trellis 初始化与 Agent 目录忽略规则；补齐跨平台测试、双语文档和 CLI 规范，并通过两轮 just ci。

### Main Changes

- Detailed change bullets were not supplied; see the summary above.

### Git Commits

| Hash | Message |
|------|---------|
| `cdebd82c484a190babdf52ec3551cc1399875187` | (see git log) |

### Testing

- Validation was not recorded for this session.

### Status

[OK] **Completed**

### Next Steps

- None - task complete


## Session 50: Install opaque handle hardening

**Date**: 2026-07-26
**Task**: Install opaque handle hardening
**Branch**: `dev`

### Summary

将安装执行收紧为后端一次性 plan_id，补齐 canonical plan、稳定错误码、生成 TypeScript 绑定与前端重试刷新，并通过 Rust、Tauri、前端及格式检查。

### Git Commits

| Hash | Message |
|------|---------|
| `b444b459` | (see git log) |

### Status

[OK] **Completed**


## Session 51: 完成 SSH 信任与传输加固

**Date**: 2026-07-26
**Task**: 完成 SSH 信任与传输加固
**Branch**: `dev`

### Summary

实现严格 SSH 参数与 app-owned known_hosts、120 秒单次主机密钥 challenge、真实 nonce 握手和两阶段 SFTP 原子写入；补齐 hostile corpus、失败清理、状态撤回与前端 challenge-only 测试。

### Git Commits

| Hash | Message |
|------|---------|
| `19cef4b2` | (see git log) |

### Status

[OK] **Completed**
