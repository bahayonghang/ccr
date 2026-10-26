# GPUI 07：用量与会话观测：执行计划

## Preconditions

02、03 完成。 用户须审阅父任务最新方案。若批准后发现需要改变范围/兼容行为，重新评审变化部分。

## Ordered Work

1. 读取 curated specs、父任务设计和当前代码；记录修改边界及基础 fixture 状态。
2. 迁移首页、Usage/Insights、sessions、Claude observer、monitoring、budget/pricing。
3. ccr-usage 读取 llmusage SQLite，已安装 llmusage CLI 执行 sync，维护 NDJSON 与 job 契约。 大列表分页/虚拟化、后台查询、有界日志、图表与表格双通道，真实 empty/unindexed/stale/error 状态。
4. 先验证 service/adapter 的行为，再验证 native UI；失败/取消/冲突测试使用隔离 fixture。
5. 更新父任务功能台账，写 evidence.md 的实际命令、结果、截图、未运行项。
6. 通过本任务验收再交由 check 角色独立核对，遵守项目角色写入权限。

## Validation

- cargo test -p ccr-usage -- --skip export_bindings
- 共享 usage/session/job fixture；just desktop-test-ui
- just tauri-process-smoke；对应 usage/session Tauri tests；tauri-bindings-check
- research/performance.md S1/S2 探针与结果报告

上列新 desktop 命令以 01 的交付为前提，perf/package 以 10 的交付为前提。若命令或 fixture 尚不存在，先完成相应实现，不写“检查通过”。Rust 行为测试保持默认并行并 skip export_bindings。

## Completion

逐项对应 prd.md 验收，保存失败修复后的最终结果；不提交/发布、不 archive 未完成工作。父任务最终验收负责全量整合。
