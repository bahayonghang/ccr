# GPUI 01：独立工程与开发命令：执行计划

## Preconditions

无；开始前需要父任务最新方案审阅通过。 用户须审阅父任务最新方案。若批准后发现需要改变范围/兼容行为，重新评审变化部分。

## Ordered Work

1. 读取 curated specs、父任务设计和当前代码；记录修改边界及基础 fixture 状态。
2. 创建 ccr-desktop 独立 workspace、ccr-gpui bin、独立 lock/target；最小 native window 含真实输入控件。
3. 根与子 justfile 增加 desktop-help/doctor/dev/build/build-debug/check/fmt/fmt-check/lint/test/test-ui/ci/audit/clean；不改变已有入口。 核查最新兼容版本与 MSRV，完成 Windows/macOS/Linux 编译探针；先在可用 Windows 主机验证窗口、文字、IME、焦点与关闭。 采集 Tauri 基线及 GPUI 探针数据；版本脚本双端加入新 manifest，文档标明能力尚在建设。
4. 先验证 service/adapter 的行为，再验证 native UI；失败/取消/冲突测试使用隔离 fixture。
5. 更新父任务功能台账，写 evidence.md 的实际命令、结果、截图、未运行项。
6. 通过本任务验收再交由 check 角色独立核对，遵守项目角色写入权限。

## Validation

- just --list；just --dry-run desktop-build；just --dry-run desktop-dev
- just desktop-doctor；just desktop-ci；just desktop-test-ui；just desktop-audit
- just version-check；相关 version-sync PowerShell/Bash 测试；根命令路由回归
- 三 OS cargo check/build 的 CI receipt；Windows native probe 截图和 IME/focus 检查

上列新 desktop 命令以 01 的交付为前提，perf/package 以 10 的交付为前提。若命令或 fixture 尚不存在，先完成相应实现，不写“检查通过”。Rust 行为测试保持默认并行并 skip export_bindings。

## Completion

逐项对应 prd.md 验收，保存失败修复后的最终结果；不提交/发布、不 archive 未完成工作。父任务最终验收负责全量整合。
