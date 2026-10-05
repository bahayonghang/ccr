# 复核历史 CI 失败并补齐诊断证据

## Goal

对历史 Root/Tauri/Frontend 失败形成当前原参数复核证据，区分首故障、确定原因、假设与已消失路径，避免猜测修复。

## Confirmed Facts

历史 run 32684239641 报告缺失；32684151306 Windows 5s process smoke失败；32684151228 teardown错误。当前根1739/Tauri517通过；历史Vue测试已不存在。

父任务需求映射：根 R2/R4/R5，工具上下文与规则项同时映射根 R3；证据见父任务 research/audit.md 与 research/harnesses.md。

## Requirements

- R1：对历史 Root/Tauri/Frontend 失败形成当前原参数复核证据，区分首故障、确定原因、假设与已消失路径，避免猜测修复。
- R2：不自动调高timeout、串行化、降低阈值、吞IO/teardown错误，不编辑产品代码、不安装工具、不触发hosted重跑。
- R3：审批后按本项文件边界执行，将真实验证和适用工具交父任务整合；当前保持 planning。

## Acceptance Criteria

- [x] AC1（R1）：逐一记录历史SHA、原命令/环境、首故障；当前对应命令的退出码和本机环境被保存，未复现不能记已修复。
- [x] AC2（R1）：Root报告IO错误与Windows timeout的假设有可证伪的复核方案，缺hosted/底层错误证据明确UNVERIFIED。
- [x] AC3（R1）：仅产生本任务research/verification.md，不按猜测修改 fix.rs/gateway.rs；确需产品修复时给出具体方案及检查并单独申请批准。
- [x] AC4（R2）：diff 不超出 implement.md 白名单，未执行所列排除动作。
- [x] AC5（R3）：交付记录含适用工具、实际执行模型、检查结果与 UNVERIFIED；未经用户批准不启动。

## Dependencies

可独立批准；前端coverage复核等UI子任务通过后；本项不阻塞前四项局部完成。

## Out of Scope

不自动调高timeout、串行化、降低阈值、吞IO/teardown错误，不编辑产品代码、不安装工具、不触发hosted重跑。
