# 执行计划

## 当前阶段

Implementation and independent check complete。用户于 2026-10-12 批准方案后启动；本机修复、迁移与真实构建/缓存复用均已验证，AC1–AC5 通过。按已批准的不自动提交约束，以 --no-commit 归档和记录。

## 执行顺序

- [x] 创建任务并确认入口、全局配置、真实 PATH、两个 target 和沙箱外 doctor。
- [x] 用临时子进程 PATH 前置实验验证入口根因。
- [x] 完成 PRD、设计、研究及实现/检查 context manifests。
- [x] 用户审阅最终方案后启动任务；已派发 implement 子代理，完成实施后交 check 子代理独立核验。
- [x] 将本机环境备份存于非仓库本地位置；记录摘要而非提交完整 PATH。
- [x] 修复持久 PATH 顺序，确认 OS 权限；验证新组合环境及 `pwsh -NoProfile` 解析。见 `research/path-results.md`。
- [x] 重新核对占用本次 CCR 源/目标目录的活动构建、目录解析和 D 盘剩余空间，复制保留旧 C 盘产物并核验。35,407 个文件 SHA-256 一致。
- [x] 安全切换根 target，再用 mbx 接管根与 Tauri 的 D 盘同卷目录，核对元数据。
- [x] `mbx setup --status`、`mbx doctor --json`、`mbx cache dir`、两处 `Get-Item target`。迁移后 doctor 0 failures / 0 warnings。
- [x] 在带正确 PATH 的进程执行 `just build` 与 `just ui-build-backend`，记录耗时、退出码、mbx 活动及实际产物位置。
- [x] 选择 CCR 的小型 crate，两个全新专用 target、同参数构建，检查第二次 mbx restore/hit；ccr-types 第二次 35 hits / 0 misses，恢复 74 个文件。
- [x] 记录 `mbx cache stats` / `mbx cache projects` 与会话证据，必要时 `mbx explain --last` 分析绕过原因。
- [x] 检查子代理逐项验收；更新必要文档/规范及任务状态，最终说明是否需要重启桌面应用。

## 验证范围

只改本机配置时无需全仓 `just ci` 或新业务单元测试；实际 Cargo 构建和共享缓存复用是必要验证。若实施中必须改项目脚本，则先更新方案，并执行该脚本相关静态和行为检查。保持 Rust 测试默认并行及 `--skip export_bindings` 约定。

## 权限与失败处理

D:\mbx-cache 和用户/系统配置在当前工作区沙箱之外，必要操作使用工具的正常权限申请。系统 PATH 修改可能还需 Windows 管理员能力；不得把沙箱外执行等同于管理员权限。权限不足时保留已完成的诊断、备份和具体步骤，不宣称持久接入完成。

递归复制/移动前核实绝对路径，所有文件操作使用同一 PowerShell 或安全文件 API；只解除链接时不得跟随链接删除源目录。原迁移阶段不执行旧缓存清理；验收后用户另行明确授权，现已清理旧 C target，详见 research/cleanup-results.md。不自动提交或推送。
