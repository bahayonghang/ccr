# 执行计划：Claude Code 视图迁移

> 父任务：`08-22-react-migration`（阶段 5，与其余六个视图子任务及 `08-22-i18n-port` 并行）。
> 分支：`feature/react-migration/views-claude`，PR 目标 `feature/react-migration`。

## 外壳接口公示（08-22-shell-port 批次 6）

共享接口：`.trellis/tasks/08-22-shell-port/shared-interfaces.md`。阶段 4a 之后不改 `MasterDetailLayout` 与 `src/ui/` 原语。

## 前置确认

- [ ] 父任务统一层门已通过：`08-22-platform-unify` 的 config 契约定稳，本任务范围表已回填（其批次 8）。
- [ ] 前置阅读完成（`design.md` §1 末段的五份文档）。
- [ ] 前置阅读：`.trellis/spec/ccr-ui/frontend/react-rerender-discipline.md`（R8，动手前必读）。
- [ ] `08-22-state-logic-port` 已通知 `claudeObserver` 的 Query key 与事件失效范围（其批次 7）。
- [ ] `08-22-test-contract-rebuild` 已提供本域契约重写稿（`development-resource-contracts.md`）。
- [ ] Auth 面判定结果已知，`ClaudeAuthView` 的归属确定。
- [ ] `git checkout -b feature/react-migration/views-claude feature/react-migration`

## 提交批次

### 批次 1：统一层接入（config + 薄壳）

- [ ] 填 `configs/settings.ts` 的 `claudeSettingsConfig`。
- [ ] 填 `configs/profiles.ts` 的 `claudeProfilesConfig`。
- [ ] `PluginsView` 收敛到 `generic/PlatformPluginsView` 的调用点。
- [ ] 按 Auth 判定填 `configs/auth.ts` 或完整迁移 `ClaudeAuthView`。
- [ ] 四个薄壳视图，各 ≤100 行。
- [ ] `SlashCommandsView`（18 行）框架转换。

验证：四个面的路由可达；差异矩阵中归属 `config.claude` 的项逐项在界面上确认（供 `platform-unify` AC6 的验证矩阵填格）。

### 批次 2：claude-observer（7 文件 2,253 行）

本域主要风险，先做，留出验证时间。

- [ ] 7 个文件迁移，数据读取走 `claudeObserverKeys` 的 Query hook。
- [ ] 事件名不变（R5）。组件级订阅（若需要）在 `useEffect` 内，StrictMode 下不双订阅。
- [ ] `TokenDetailTab` 的主题 token 耦合按 `token-classification.md` 核对。
- [ ] 卸载后订阅解绑验证（AC6）。

验证：观测数据刷新正常；`claude-observer-tabs` smoke 测试通过（AC9）。

### 批次 3：表单类视图

- [ ] `HooksView`(920)：15 处 `v-model` → react-hook-form 非受控注册，动态数组用 `useFieldArray`，校验用 zod。
- [ ] `StatuslineView`(230)：同上。

验证：字段读写正确；输入延迟符合 `08-22-arch-quality-perf` 场景 1 的预期（本批次不测量，由 `regression-release` 统一测）。

### 批次 4：其余视图与域组件

- [ ] `ClaudeCodeView`(745)、`OutputStylesView`(558)、`SkillsMigrationView`(392)。
- [ ] `components/claude/`（3 文件 1,869 行）。
- [ ] `OutputStylesView` 的样式 token 命名空间与 CCR 主题不交叉。
- [ ] 依赖方向自检：本域文件不导入其他 `features/<平台>/`。

### 批次 5：收口与登记

- [ ] 本批次组件内 px 与 `rgba()` 归零，豁免逐条登记（AC4）。
- [ ] `nextTick` 登记表落盘，本批次调用点全部有改写说明（AC7）。
- [ ] `rg --files -g '*.vue'`（PRD AC1 的路径集合）无匹配。
- [ ] `git diff --stat src/api src/types`（应为空，AC5）。

## 验证命令

| 时机        | 命令                                                                                                                                                                                                                                 |
| ----------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| 每批次后    | `bun run type-check`、`bun run lint`（AC8）                                                                                                                                                                                          |
| 批次 2–4 后 | `bun run test:smoke`                                                                                                                                                                                                                 |
| 批次 5 后   | `rg --files -g '*.vue' src/views/Claude* src/views/Hooks* src/views/OutputStyles* src/views/Plugins* src/views/SkillsMigration* src/views/Statusline* src/views/SlashCommands* src/components/claude src/components/claude-observer` |
| 交付前      | `just frontend-check-quick`、`bun run lint:ci`                                                                                                                                                                                       |

## 交付门（父任务视图门的一部分）

- [ ] AC1–AC9 全部满足。
- [ ] AC3 的 10 条核心操作路径逐条验证并记录：Settings 读写、Profiles 切换、Auth 登录、Hooks 增删、插件安装、Skills 迁移、Statusline 配置、Output Styles 切换、斜杠命令增删、观测数据刷新。
- [ ] `nextTick` 登记表与硬编码豁免登记落盘。
- [ ] `src/api`、`src/types` git diff 为空。
- [ ] `development-resource-contracts.md` 覆盖的行为验证通过（R8）。
- [ ] 本域的验证矩阵格已填，交 `08-22-platform-unify` AC6。

## 回滚点

按批次提交，每批次可单独 revert。批次 1（统一层接入）的回滚会使四个面的路由不可达，需与 `platform-unify` 协同。批次 2–4 各自独立。

提交粒度：单文件或单个紧密相关的小组（父任务 `design.md` §12.1 的缓解措施），便于二分定位。

## 协同点

| 编号 | 内容                                   | 对方                          | 时机   |
| ---- | -------------------------------------- | ----------------------------- | ------ |
| E    | 统一层接口消费                         | `08-22-platform-unify`        | 批次 1 |
| D    | 本域契约重写稿先行                     | `08-22-test-contract-rebuild` | 前置   |
| I    | i18n 调用形式按对方约定                | `08-22-i18n-port`             | 全程   |
| —    | `claudeObserver` 的 key 与事件失效范围 | `08-22-state-logic-port`      | 前置   |
| —    | 本域验证矩阵格                         | `08-22-platform-unify`        | 交付时 |
| —    | 逐屏比对的对照依据                     | `08-22-regression-release`    | 交付后 |
