# GPUI 04：配置与 Profiles：执行计划

## Preconditions

用户已批准父子方案并要求按依赖继续，无需重复实施批准。02 已验收完成；03 可消费工程和当前指定核心原生流程已独立检查通过。依父 implement.md 的工程依赖/开放验收分开跟踪规则，03 的 UIA、IME、user-deferred 尺寸/DPI 与设计细节仍开放，不判完整完成，也不阻断已批准04代码。若实际发现需改变范围/兼容行为/安全边界，仅评审变化部分。

验证继续聚焦真实编辑、校验、CAS冲突、保存重读、应用/停用、错误恢复和取消/退出；不执行缩放/DPI矩阵，不修改真实用户配置或账户，不改全局HOME。

## Ordered Work

1. 读取 curated specs、父任务设计和当前代码；记录修改边界及基础 fixture 状态。
2. 迁移配置目录读取、创建/编辑/删除、profile 切换、应用/停用、备份恢复和已有 converter 能力。
3. 保留 profiles.db 单一 owner、resource version/CAS、冲突反馈、原子写与字段级 patch 语义。 配置列表/编辑器/保存结果构成真实闭环，支持取消编辑和返回。
4. 先验证 service/adapter 的行为，再验证 native UI；失败/取消/冲突测试使用隔离 fixture。
5. 更新父任务功能台账，写 evidence.md 的实际命令、结果、截图、未运行项。
6. 通过本任务验收再交由 check 角色独立核对，遵守项目角色写入权限。

## Validation

- 相关 ccr-config/ccr-cli profile fixture 单测
- just desktop-test；just desktop-test-ui；just secret-write-check
- 对应 Tauri config/profile 测试；必要时 tauri-bindings-check
- just version-check；just fmt-check；just lint-strict；just test

上列新 desktop 命令以 01 的交付为前提，perf/package 以 10 的交付为前提。若命令或 fixture 尚不存在，先完成相应实现，不写“检查通过”。Rust 行为测试保持默认并行并 skip export_bindings。

## Completion

逐项对应 prd.md 验收，保存失败修复后的最终结果；不提交/发布、不 archive 未完成工作。父任务最终验收负责全量整合。
