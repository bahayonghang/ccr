# 修复本机 mbx 构建接入与 D 盘缓存配置

## Goal

本机普通 Cargo 构建自动通过 mbx；以 CCR CLI 和 Tauri Rust 后端为验收样本，缓存与受管构建输出统一位于 `D:\mbx-cache`，并有实际构建和缓存复用证据。

## Background

2026-10-12 现场检查确认 mbx 1.23.0 已安装且 shim 当前有效，但未进入有效 PATH。全局 `cache_dir` 已正确配置，实际 D 盘缓存为空；根 target 仍链接到旧 C 盘受管目录。完整命令证据见 `research/diagnosis.md`。

## Requirements

- R1：修复本机持久构建入口优先级，保留 Rust 工具链及其他 PATH 条目。覆盖普通 PowerShell、无 profile 子进程、just 和应用继承环境；旧进程需要重启时须明确说明。
- R2：沿用 `D:\mbx-cache` 与既有 100 GiB 总预算，根 workspace 与 Tauri workspace 的受管 target 都归入 D 盘。保留可回滚的旧产物，禁止以清空缓存代替迁移。
- R3：通过项目真实 Rust 构建和独立 target 的缓存复用检查验收；重复构建仅显示 Cargo Fresh 不能证明 mbx 命中。
- R4：修复仅针对本机配置及本项目构建存储；项目可提交内容限必要说明、任务及验证资料，不提交本机完整 PATH、用户配置备份或二进制。

## Acceptance Criteria

- [x] AC1 / R1：使用已更新持久环境启动的新进程，`Get-Command cargo` 指向 mbx shim；`mbx doctor --json` 的 setup、cache、config 均 pass。
- [x] AC2 / R2：两个 workspace 的 target 实际落在 `D:\mbx-cache\targets` 下，`mbx cache dir` 指向 `D:\mbx-cache\actions`。
- [x] AC3 / R3：`just build` 及 Tauri Rust 后端构建成功，mbx session / stats / projects 中有对应 workspace 的活动证据。
- [x] AC4 / R3：用同一 CCR 小型 crate、相同参数、两个新建独立 target 做受控构建，第二次出现 mbx 恢复或命中证据；不得以 Cargo no-op 作为命中结论。
- [x] AC5 / R4：记录修改、验证与回滚方式；区分本机结果和沙箱限制，未验证项目不得标为完成。

## Out of Scope

不改 Rust 产品逻辑，不强制其他开发者或 CI 安装 mbx，不将 Bun/Vite 等非 Rust 构建包装为 mbx 构建，不升级工具链，不调整 NTFS ACL，不启用远程缓存，不清理其他项目缓存。

## Risks

系统 PATH 已包含 Rust 路径，持久优先级修复可能需要 Windows 管理员权限；当前进程不是管理员。C 盘旧 target 跨盘自动迁移可能删除旧输出，必须先保留副本。当前桌面应用持有旧环境，持久设置不能反向更新已运行进程。

## Completion Evidence

2026-10-12 独立技术验收通过。配置证据见 research/path-results.md，迁移与构建证据见 research/implementation-results.md，独立复核见 research/check-results.md。旧 C 盘产物随后按用户要求清理，详见 research/cleanup-results.md；旧目录回滚方式已失效。已运行的 Codex/IDE/终端需重启继承 PATH。部分 Tauri 结果受 mbx 路径映射与链接参数支持范围限制，未宣称全量缓存。按批准范围不自动提交或推送。
