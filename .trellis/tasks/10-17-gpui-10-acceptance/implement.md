# GPUI 10：性能、三平台与交付验收：执行计划

## Preconditions

01 至 09 全部完成，台账无无人负责项。 用户须审阅父任务最新方案。若批准后发现需要改变范围/兼容行为，重新评审变化部分。

## Ordered Work

1. 读取 curated specs、父任务设计和当前代码；记录修改边界及基础 fixture 状态。
2. 完成逐命令/route 台账和真实端到端场景，不以视觉相似替代功能。
3. 对完整应用执行性能合同、三平台原生视觉/输入/可访问性验收。 独立本地打包、desktop-bench/perf/package、版本/依赖/CI 路由治理和文档。
4. 先验证 service/adapter 的行为，再验证 native UI；失败/取消/冲突测试使用隔离 fixture。
5. 更新父任务功能台账，写 evidence.md 的实际命令、结果、截图、未运行项。
6. 通过本任务验收再交由 check 角色独立核对，遵守项目角色写入权限。

## Validation

- just desktop-ci；just desktop-audit；just desktop-test-ui
- just desktop-bench；just desktop-perf；just desktop-package
- just version-check；just ci-governance-check；just ci
- 三平台原生验收清单、真实功能路径、安装后 smoke 与截图复核

上列新 desktop 命令以 01 的交付为前提，perf/package 以 10 的交付为前提。若命令或 fixture 尚不存在，先完成相应实现，不写“检查通过”。Rust 行为测试保持默认并行并 skip export_bindings。

## Completion

逐项对应 prd.md 验收，保存失败修复后的最终结果；不提交/发布、不 archive 未完成工作。父任务最终验收负责全量整合。
