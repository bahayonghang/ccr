# GPUI 02：共享应用服务与运行时：执行计划

## Preconditions

01 工程接口已可消费：独立 workspace、锁文件、desktop 命令和 Windows 原生探针已通过相应检查。2026-10-18 用户要求继续后续任务，按父任务新增的实施依赖/开放验收分开跟踪规则开始 02；01 的外部平台和性能未验证项保留，不能记完成。父子方案已获实施批准，只有实际改变产品范围/兼容/安全边界的变化才另行评审。

## Ordered Work

1. 读取 curated specs、父任务设计和当前代码；记录修改边界及基础 fixture 状态。
2. 建立最小框架无关 ccr-app 编排 crate，首条真实链只做配置列表读取；已有领域 API 不重新封装一整层同名方法。
3. 定义 typed request/result、environment/generation key、长期 Tokio runtime、有界队列及 app-owned jobs 生命周期。 为后续迁移提供按域提取方法；Tauri 当前业务行为与 wire/export 保持。
4. 先验证 service/adapter 的行为，再验证 native UI；失败/取消/冲突测试使用隔离 fixture。
5. 更新父任务功能台账，写 evidence.md 的实际命令、结果、截图、未运行项。
6. 通过本任务验收再交由 check 角色独立核对，遵守项目角色写入权限。

## Validation

- cargo test -p ccr-app -- --skip export_bindings
- just desktop-check；just desktop-test；just desktop-lint
- just tauri-bindings-check；对应 config Tauri 测试
- just version-check；just fmt-check；just lint-strict；just test

上列新 desktop 命令以 01 的交付为前提，perf/package 以 10 的交付为前提。若命令或 fixture 尚不存在，先完成相应实现，不写“检查通过”。Rust 行为测试保持默认并行并 skip export_bindings。

## Completion

逐项对应 prd.md 验收，保存失败修复后的最终结果；不提交/发布、不 archive 未完成工作。父任务最终验收负责全量整合。
