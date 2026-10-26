# GPUI 06：MCP 与扩展资源：执行计划

## Preconditions

02、03、05 完成。 用户须审阅父任务最新方案。若批准后发现需要改变范围/兼容行为，重新评审变化部分。

## Ordered Work

1. 读取 curated specs、父任务设计和当前代码；记录修改边界及基础 fixture 状态。
2. 迁移 MCP、Skills、Agents、Plugins、Hooks、Slash commands、system prompts、output styles、statusline 现有业务。
3. 列表、搜索、详情、编辑与已有安装/卸载流程；复用 ccr-skills/ccr-store/ccr-types。 按能力区分平台和环境，统一资源编辑状态而不强迫不同配置格式共享写法。
4. 先验证 service/adapter 的行为，再验证 native UI；失败/取消/冲突测试使用隔离 fixture。
5. 更新父任务功能台账，写 evidence.md 的实际命令、结果、截图、未运行项。
6. 通过本任务验收再交由 check 角色独立核对，遵守项目角色写入权限。

## Validation

- ccr-skills/ccr-store 目标测试与资源 fixture
- just desktop-test；just desktop-test-ui；just desktop-lint
- 对应 Tauri MCP/资源测试和 typed binding/inventory gate

上列新 desktop 命令以 01 的交付为前提，perf/package 以 10 的交付为前提。若命令或 fixture 尚不存在，先完成相应实现，不写“检查通过”。Rust 行为测试保持默认并行并 skip export_bindings。

## Completion

逐项对应 prd.md 验收，保存失败修复后的最终结果；不提交/发布、不 archive 未完成工作。父任务最终验收负责全量整合。
