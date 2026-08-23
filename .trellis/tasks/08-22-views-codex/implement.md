# 执行计划：Codex 视图迁移

> 父任务：`08-22-react-migration`（阶段 5，七个视图子任务并行）。本批次规模最大，拆 3 个主要提交批次（PRD Notes）。
> 分支：`feature/react-migration/views-codex`，PR 目标 `feature/react-migration`。

## 外壳接口公示（08-22-shell-port 批次 6）

共享接口：`.trellis/tasks/08-22-shell-port/shared-interfaces.md`。阶段 4a 之后不改 `MasterDetailLayout` 与 `src/ui/` 原语。
Profiles 共享层（`08-22-views-profiles-config` 批次 1）：`ccr-ui/src/components/profiles/*.tsx`，接口见 `.trellis/tasks/08-22-views-profiles-config/profiles-shared-interfaces.md`。`CodexProfilesView` 改接到这些 React 模块，不要再 import `.vue`。

## 前置确认

- [ ] 父任务统一层门已通过，本任务范围表已回填。
- [ ] 前置阅读完成（`08-22-views-claude/design.md` §1 末段的五份文档）。
- [ ] 前置阅读：`.trellis/spec/ccr-ui/frontend/react-rerender-discipline.md`（R8，动手前必读）。
- [ ] Codex auth off 契约文档已定位，断言项清单已抽出。
- [ ] Auth 面判定结果已知，`CodexAuthView` 与 `codex-auth-shared.css` 的归属确定。
- [ ] `08-22-test-contract-rebuild` 已提供本域契约重写稿。
- [ ] `git checkout -b feature/react-migration/views-codex feature/react-migration`

## 提交批次

### 批次 1：统一层接入（config + 薄壳）

- [x] 填 `codexSettingsConfig`（原 33 处 `v-model` 的字段全部落在 config 或 base）。
- [x] 填 `codexProfilesConfig`。
- [x] `CodexMcpView` → `generic/PlatformMcpView` 调用点。
- [x] `codex/CodexAgentsView` → `generic/AgentsView` 调用点。
- [x] 按 Auth 判定填 `configs/auth.ts` 或将 `CodexAuthView` 留在批次 2。
- [x] 薄壳视图各 ≤100 行。
- [x] `CodexSlashCommandsView`(229) 迁移，核对超出另两平台薄壳的 202 行：可下沉的下沉到 `BaseSlashCommands`，属 Codex 差异的进 config。

验证：五个面路由可达；归属 `config.codex` 的差异项逐项界面确认。

### 批次 2：`views/codex/` 目录

- [x] `AddCodexAccountModal`(1,179) 按 `design.md` §3 拆为按向导步骤划分的多个组件，共享单个 form context。
- [x] 凭据字段掩码显示，日志经 `logRedact.ts`，无明文。
- [x] OAuth 回调等待机制确认并迁移，超时与取消分支逐个保留。
- [x] `views/codex/` 其余文件（扣除已移交的 `CodexAgentsView`）。
- [x] 按 Auth 判定，`CodexAuthView`(958) 若留在本域则在本批次迁移。
- [x] `codex-auth-shared.css` 按 `design.md` §5 逐个选择器落位，落位清单落盘（AC5）。

验证：账号添加向导逐步走通；Codex auth off 行为验证（AC6）；`bun run test:smoke`。

### 批次 3：7 个根级视图

- [x] `CodexSessionsView`(883)：长列表按需接 `@tanstack/react-virtual`，形态复用 `08-22-views-usage` 的接线。
- [x] `CodexView`(880)。
- [x] 超过行数上限的文件拆分，不改对外接口。

### 批次 4：`components/codex/`（5 文件 3,201 行）

- [x] 5 个文件迁移，均值 640 行/文件，超限项拆分。
- [x] 依赖方向自检：不导入其他 `features/<平台>/`。

### 批次 5：收口与登记

- [x] 本批次组件内 px 与 `rgba()` 归零，豁免逐条登记（AC4）。
- [x] `nextTick` 登记表落盘（AC8）。
- [x] `rg --files -g '*.vue' src/views/codex src/views/Codex* src/components/codex` 无匹配（AC1）。
- [x] `git diff --stat src/api src/types`（应为空，AC7）。
- [x] `src/utils/codexProfileEditor.ts`、`codexProfiles.ts`、`codexHelpers.ts` 的 git diff 为空。非空则登记为独立缺陷。

## 验证命令

| 时机        | 命令                                                                          |
| ----------- | ----------------------------------------------------------------------------- |
| 每批次后    | `bun run type-check`、`bun run lint`（AC9）                                   |
| 批次 2–4 后 | `bun run test:smoke`（AC10）                                                  |
| 批次 5 后   | `rg --files -g '*.vue' src/views/codex src/views/Codex* src/components/codex` |
| 交付前      | `just frontend-check-quick`、`bun run lint:ci`                                |

## 交付门（父任务视图门的一部分）

- [ ] AC1–AC10 全部满足。
- [ ] AC3 的 8 条核心操作路径逐条验证并记录：Profiles 增删改切换、MCP 服务器管理、Agents 管理、斜杠命令增删、Auth 登录与 auth off、Sessions 浏览、Settings 读写、账号添加向导。
- [ ] `codex-auth-shared.css` 落位清单落盘，14.8 KB 全部归类（AC5）。
- [ ] Codex auth off 契约断言逐条验证通过（AC6）。
- [ ] 凭据脱敏验证：界面掩码、日志无明文。
- [ ] `nextTick` 登记表与硬编码豁免登记落盘。
- [ ] 本域验证矩阵格已填，交 `08-22-platform-unify` AC6。

## 回滚点

五个批次各自独立提交。批次 2 内 `AddCodexAccountModal` 的拆分按向导步骤分多次提交，可精确回退某一步。

提交粒度：单文件或单个紧密相关的小组，便于二分定位。

## 协同点

| 编号 | 内容                                   | 对方                          | 时机   |
| ---- | -------------------------------------- | ----------------------------- | ------ |
| E    | 统一层接口消费                         | `08-22-platform-unify`        | 批次 1 |
| D    | 本域契约重写稿先行                     | `08-22-test-contract-rebuild` | 前置   |
| I    | i18n 调用形式                          | `08-22-i18n-port`             | 全程   |
| —    | 虚拟滚动接线形态复用                   | `08-22-views-usage`           | 批次 3 |
| —    | `codex-auth-shared.css` 的分层归属标准 | `08-22-design-system`         | 批次 2 |
| —    | 本域验证矩阵格                         | `08-22-platform-unify`        | 交付时 |
