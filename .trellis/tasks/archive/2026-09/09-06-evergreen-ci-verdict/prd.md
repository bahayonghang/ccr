# 修复 CI 失败退出码与 Tauri 变更触发范围

## Goal

修复 F2/F3：覆盖率失败必须使 job 非零；实际 Cargo 配置输入变更必须触发相应 CI surface，避免 required 假绿。

## Confirmed Facts

.github/workflows/vscode-ci.yml:49 未指定 shell；scripts/ci/ci_surface_policy.py:25 对三个现有 .cargo 配置单独变更四surface均False。

父任务需求映射：根 R2/R4/R5，工具上下文与规则项同时映射根 R3；证据见父任务 research/audit.md 与 research/harnesses.md。

## Requirements

- R1：修复 F2/F3：覆盖率失败必须使 job 非零；实际 Cargo 配置输入变更必须触发相应 CI surface，避免 required 假绿。
- R2：不新增 CI 引擎，不改依赖版本，不远程触发 workflow，不改变 coverage 阈值。
- R3：审批后按本项文件边界执行，将真实验证和适用工具交父任务整合；当前保持 planning。

## Acceptance Criteria

- [x] AC1（R1）：coverage step 使用 pipefail shell；相同管道的失败探针非零、成功探针零，输出日志仍被保留。
- [x] AC2（R1）：.cargo/tauri-ci.toml 只触发 tauri；.cargo/config.toml 触发 root+tauri；.cargo/audit.toml 只触发 root，frontend/vscode 无关结果保持 false。
- [x] AC3（R1）：现有 workflow 治理单测和检查通过、VSCode 覆盖率原阈值通过；不降低覆盖率、不扩大为任意文件触发全部surface。
- [x] AC4（R2）：diff 不超出 implement.md 白名单，未执行所列排除动作。
- [x] AC5（R3）：交付记录含适用工具、实际执行模型、检查结果与 UNVERIFIED；未经用户批准不启动。

## Dependencies

独立于 UI/OMP；修改完成后向规则子任务交付检查退出码和精确路径映射。

## Out of Scope

不新增 CI 引擎，不改依赖版本，不远程触发 workflow，不改变 coverage 阈值。
