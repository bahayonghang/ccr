# GPUI 08：命令、同步与签到：执行计划

## Preconditions

02、03、04 完成。 用户须审阅父任务最新方案。若批准后发现需要改变范围/兼容行为，重新评审变化部分。

## Ordered Work

1. 读取 curated specs、父任务设计和当前代码；记录修改边界及基础 fixture 状态。
2. 迁移命令工作台、工具安装、同步、签到及余额刷新。
3. 取消、超时、重试、并发/单例、离开页面、关闭窗口、订阅恢复等 lifecycle。 保留命令 allowlist、network/local/secret mutation 分级、WebDAV 密码生命周期和签到 per-origin 限流。
4. 先验证 service/adapter 的行为，再验证 native UI；失败/取消/冲突测试使用隔离 fixture。
5. 更新父任务功能台账，写 evidence.md 的实际命令、结果、截图、未运行项。
6. 通过本任务验收再交由 check 角色独立核对，遵守项目角色写入权限。

## Validation

- 目标 ccr-core/ccr-sync/ccr-checkin 测试
- just desktop-test；just desktop-test-ui；just secret-write-check
- just tauri-process-smoke；Tauri command/sync/checkin 回归
- controlled fake process/HTTP server 测取消、超时、限流与重试

上列新 desktop 命令以 01 的交付为前提，perf/package 以 10 的交付为前提。若命令或 fixture 尚不存在，先完成相应实现，不写“检查通过”。Rust 行为测试保持默认并行并 skip export_bindings。

## Completion

逐项对应 prd.md 验收，保存失败修复后的最终结果；不提交/发布、不 archive 未完成工作。父任务最终验收负责全量整合。
