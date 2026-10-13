# 独立检查结果

2026-10-12，trellis-check 独立复核；本轮只写本文件，不运行新构建、不终止进程、不执行缓存清理。

## 验收状态

**最终技术验收通过，AC1–AC5 均有证据支持。** 初审时待定的 Tauri release 已成功结束，最终结果、原始会话及产物已独立复核。主会话可同步 PRD/执行计划并关闭任务；现有桌面进程仍需重启以继承新 PATH。

## 独立验证

- 在实际用户权限上下文读取备份和真实 HKLM/HKCU。Machine PATH 严格等于 `shim + ';' + 原字符串`；Machine 类型、User PATH 内容及类型均未变化。全局 mbx 配置与备份 SHA-256 相同。修复结果记录 `environmentBroadcast=true`。
- 用读取到的持久 Machine + User PATH 构建进程环境，再启动 `pwsh -NoProfile -NonInteractive`，Cargo 解析为 `C:\Users\lyh\AppData\Local\mbx\bin\cargo.exe`，版本 1.98.0。
- `mbx setup --status` 确认安装 current；`mbx doctor --json` 九项 pass、零 failures、零 warnings；预算仍为 100 GiB。`mbx cache dir` 返回 `D:/mbx-cache\actions`。
- 根 target 和 Tauri target 均为 SymbolicLink，分别指向 `D:\mbx-cache\targets\v1\6b736f3e8608d890383d61470c216d50cd84b38db0e955508f465ce6ab4ea1b5` 和 `D:\mbx-cache\targets\v1\44061aa3da5bb563d33700d15e5ae7660d5d3da33a41e506f28329282a48c2b6`。旧 C 盘根目录仍存在，未递归重新计算已有 35,407 文件的完整校验；原始 `verified-copy.json` 记录逐文件 SHA-256 全部相同。
- 最终缓存快照 `cache-stats-final.json`：721 objects、81 action results、2 live checkouts、2 target directories，action store 607,960,738 字节，combined logical 50,776,949,215 字节；`cache-projects-final.txt` 同时列出 CCR 根与 Tauri workspace。项目归属统计不能相加当作物理磁盘占用。
- 读取原始 `just-build-result.json`、日志及 session `1791866980979-41064-uehl`：exit 0，28.886 秒，workspace/命令吻合，stored_bytes=412418727。
- 读取两个 ccr-types 结果和原始 sessions `1791867082602-54236-mxub` / `1791867127682-41064-m0rd`：两次 exit 0，两目录构建前不存在，命令只差 target 目录；第二次 35 hits、0 misses、35 compiler invocations avoided、74 restored files、51,284,578 restored bytes。copied_output_files=74，hardlinked_output_files=0。证明真实共享缓存恢复，不能写成 Cargo Fresh 或硬链接恢复。
- Tauri session `1791867018324-5804-dzim` 归属 `ccr-ui/src-tauri`，命令 `build --release --timings`；最终已有 `session_finished`。结果 JSON 为 exit 0、645.648779 秒，Cargo 日志为 10m 43s；session 记录 23 unconsulted、0 hits、0 misses、stored_bytes=144056329。`target/release/ccr-desktop.exe` 存在，59,179,520 字节，位于此前验证的 D 盘 target 链接之下。此项目构建证明入口与存储接入，实际复用仍由独立 A/B 会话证明。

## Findings (fixed)

- `.trellis/spec/ccr/backend/local-build-cache.md` 的 hard-link 诊断行重复。已通知主会话在其负责文件中移除，检查确认只剩一行。本检查代理未越过文件所有权修改。

## Findings (not fixed)

- 无新增配置或迁移缺陷，没有未解决的技术验收阻塞。任务 PRD/执行计划由主会话在本报告后同步完成状态。
- setup 输出含既有 rust-analyzer `check/overrideCommand: unexpected field` 提示，同时明确报告检查仍通过 mbx。当前任务验证普通 Cargo/just，未调整 IDE 配置或宣称独立验证 IDE 原生加载。
- Tauri 日志中的跨 workspace 绝对路径导致部分结果不存储，以及首次 A 探针的 build-script declarations 警告，已在 implementation-results.md 如实记录。最终 session 另确认 `ccr_desktop` outcome=bypass、reason=`unmodeled-link-argument`，约 601 秒；最终实施记录已补充并复核。上述限制不影响构建成功，也不代表 mbx 未接入，但不能声称 Tauri 所有编译步骤均可缓存；未扩大任务去修改业务源码或 mbx 行为。
- 现有 Codex/IDE 仍可能继承旧 PATH。持久环境子进程已验证，应用重启仍需用户完成，不把环境广播当成旧进程已更新。

## Verification

- 文档/清单：`git diff --check` pass；check.jsonl 与 implement.jsonl 所有 JSONL 条目可解析且引用文件存在；task.json 可解析且保持 in_progress；PRD 未提前勾选 AC3。
- 变更范围：仅任务资料与 backend spec/index；未见产品源码、Cargo 配置、justfile 或私人环境备份进入变更。
- Lint：不适用；本次无产品代码或项目脚本修改，按批准计划不重复全仓静态检查。
- TypeCheck：不适用；配置修复使用运行时诊断验证，未改类型代码。
- Tests：配置与缓存运行时检查 pass；CLI 构建、A/B 构建、Tauri release 全部 pass。未运行新业务单元测试或全仓 CI，也未在检查阶段重复构建。

原始证据目录：`D:\mbx-cache\migration-10-12-ccr\verification`、`D:\mbx-cache\actions\sessions\v1`、`D:\mbx-cache\repair-backups\ccr-20261012-path`。私人完整 PATH 不写入仓库。
