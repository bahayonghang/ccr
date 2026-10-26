# GPUI 01：独立工程与开发命令

## Goal

建立独立 workspace、锁定最新兼容依赖、原生可行性探针与 desktop-* 命令。

## Parent and Dependencies

父任务：[gpui-desktop](../10-17-gpui-desktop/prd.md)。需求映射：R1、R2、R3、R7、R8。

前置：无；开始前需要父任务最新方案审阅通过。 父子关系不自动保证依赖完成。

## Requirements

- 创建 ccr-desktop 独立 workspace、ccr-gpui bin、独立 lock/target；最小 native window 含真实输入控件。
- 根与子 justfile 增加 desktop-help/doctor/dev/build/build-debug/check/fmt/fmt-check/lint/test/test-ui/ci/audit/clean；不改变已有入口。
- 核查最新兼容版本与 MSRV，完成 Windows/macOS/Linux 编译探针；先在可用 Windows 主机验证窗口、文字、IME、焦点与关闭。
- 采集 Tauri 基线及 GPUI 探针数据；版本脚本双端加入新 manifest，文档标明能力尚在建设。

## Acceptance Criteria

- [ ] 1.1：独立 cargo metadata 的 workspace_root 指向 ccr-desktop；默认根 CLI 图没有 gpui/tauri，新工程无 Tauri。
- [ ] 1.2：just --list 无重名；desktop-dev 能运行可见窗口；desktop-build 构建 release；检查命令失败原样返回非零。
- [ ] 1.3：desktop-ci 不调用 fmt 或 version-sync；test 默认并行；清理路径经 metadata 验证位于独立 target。
- [ ] 1.4：dependencies.md 记录每个直接依赖最新候选/选中/原因/特性/MSRV；冻结锁文件构建通过。
- [ ] 1.5：三 OS 编译证据与 Windows native probe 证据齐全；GPU/IME 不可用必须作为 blocker。macOS/Linux 完整 native 交互由 10 验收。

## Out of Scope

不扩大到本任务未列的业务域；不删除旧表面、不改真实用户数据、不降低已有门禁。父任务声明的完整迁移范围由后续任务继续承接。

## Status

planning。技术待验证项由本任务检查闭环，不视作已完成。开始前读父任务 design.md、implement.md、当前子任务全部规划和 curated context。
