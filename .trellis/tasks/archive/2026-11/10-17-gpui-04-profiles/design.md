# GPUI 04：配置与 Profiles：设计

## Change Boundary

- ccr-desktop/src/features/profiles/、features/configs/
- crates/ccr-app/src/config/（仅新增实际共享编排）
- ccr-ui/src-tauri/src/commands/config/、profile_lifecycle.rs、claude_profiles.rs、codex_profiles.rs 的必要 adapter

这些是预期边界，实施者先按职责定位具体函数，不能为方便而扩大重构。父任务 [architecture](../10-17-gpui-desktop/research/architecture.md) 提供源码证据；完整边界见父任务 design.md。

## Mechanism

编辑 draft 保存 base revision，service 接受显式 patch；由 ccr-config repository 事务更新，不把 UI 表格直接写 SQL。apply/off 使用 ccr-cli 现有生命周期，只有真实成功才更新当前状态。原始配置编辑器与结构化编辑均遵循既有本地权限和版本规则。

### 首次创建的失败原子性补充（独立检查复现后）

04检查实际复现：缺失Codex catalog首次创建路径绕过外层CatalogTransaction，platform先发布catalog/credential store，随后journal.verify发现外部runtime变化，回滚凭据却留下已发布profile。此行为违反AC4.3，不把失败记作成功或靠adapter删除SQLite补偿。

最小边界增加`crates/ccr-config/src/managers/config/catalog/{mod.rs,tests.rs}`及必要导出，由现有catalog owner提供缺失目录的事务创建机制，复用Session、BEGIN IMMEDIATE、join、commit/rollback、秘密权限与owner新文件清理策略。`ProfileLifecycleContext::create`的首次创建不再绕开事务。保持旧execute、纯读取、legacy未迁移拒绝、已有空inactive目录的状态、原始revision和平台normalization语义。不得复制活跃SQLite、删除已有/外部文件，或增加第二个持久化owner。

检查者负责body/journal/commit失败与成功、并发首次创建、已有空inactive目录、会话清理的实际回归；实现与结果另记evidence，设计补充本身不代表通过。

## Contract and Compatibility

继承父任务的 typed request/environment/generation、secret masking、CAS、原子写和 job terminal 契约。UI 仅负责呈现和 action；领域 owner 负责数据。每个新增入口补相应失败/取消/空结果，不添加通用动态 dispatch。

## Rollback

保留 Tauri/CLI 入口。按域提取和 native 接入分别形成可审阅变更，契约回归失败先修复；不通过迁移数据库或删旧功能绕过失败。无法验证的 OS 或能力在 evidence.md 记未运行，不标完成。
