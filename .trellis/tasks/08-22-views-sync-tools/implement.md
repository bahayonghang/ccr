# 执行计划：Sync / MCP / Commands 与工具视图迁移

> 父任务：`08-22-react-migration`。**批次 3 的前半（`components/mcp/`）属阶段 4a（共享层前置），其余批次属阶段 5（七个视图子任务并行）。**
> 分支：`feature/react-migration/views-sync-tools`，PR 目标 `feature/react-migration`。批次 3 前半单独开 PR 并先行合入，`08-22-platform-unify` 批次 5 依赖它。

## 前置确认

### 批次 3 前半的前置（阶段 3 外壳门通过后即可开工）

- [ ] 父任务外壳门已通过。
- [ ] `git checkout -b feature/react-migration/views-sync-tools feature/react-migration`

### 其余批次的前置

- [ ] 父任务统一层门已通过，本任务范围表已回填。
- [ ] `08-22-dep-upgrade` 的 `codemirror-peer-check.md` 已落盘，`@codemirror/state` 无重复实例（协同点 B）。
- [ ] `08-22-shell-port` 批次 0 的 tray 独立 HTML 入口结论已知。
- [ ] 前置阅读完成，另加 `raw-config-editor-contracts.md`（7.4 KB）、`sync-security-contracts.md`（4.2 KB）、`monitoring-log-contracts.md`（1.1 KB）。
- [ ] `08-22-test-contract-rebuild` 已提供三份契约的重写稿。

## 批次 3 前半：`components/mcp/` 迁移与接口公示（阶段 4a）

必须在 `08-22-platform-unify` 批次 5 收敛 `PlatformMcpView` 之前完成（协同点 F2）。React base 组件无法复用未迁移的 Vue 组件。

- [ ] `components/mcp/`（4 文件 2,064 行）迁移，接口保持稳定，不改造（Out of Scope：复用不改造）。
- [ ] 按 `design.md` §9 公示接口，`mcp-shared-interfaces.md` 落盘，通知 `08-22-platform-unify`。
- [ ] 改造需求登记为独立缺陷，不在本批次做。

验证：`bun run type-check` 退出码 0。该批次单独开 PR，先行合入迁移分支（父任务阶段 4a 门的准出项）。

## 批次 0：编辑器契约转为可执行断言

先做（PRD Notes 建议）。

- [ ] `raw-config-editor-contracts.md` 的断言逐条转为可执行形式，覆盖 R3 五项：语法高亮、JSON / Markdown 模式、lint 提示、搜索、快捷键。
- [ ] 与 `08-22-test-contract-rebuild` 的重写稿对齐，不产生两份断言。
- [ ] 断言先失败（红），批次 1 完成后转绿。

## 批次 1：CodeMirror 桥接 + 升级门

- [ ] `components/editor/`（2 文件 698 行）迁到 `@uiw/react-codemirror` 4.25.11。
- [ ] 9 个 `@codemirror/*` 包保留。
- [ ] 三件事逐个落地：生命周期、`Compartment` 动态重配置、受控值同步。

### 升级门检查（R2.2，显式门）

- [ ] 逐条判定断言的可表达性。判定标准见 `design.md` §3.3：能通过 `extensions` 数组表达的不算无法表达。
- [ ] `codemirror-expressiveness.md` 落盘，记录无法表达的项数（AC4.1）。
- [ ] **无法表达项 > 3 条**：改为自建 React hook（约 200–250 行），并记录该判定与执行结果。
- [ ] **无法表达项 ≤ 3 条**：继续用 `@uiw`，3 条以内的项逐条记录规避方式。

### 多实例复验

- [ ] 构建产物核对 `@codemirror/state` 只打进一份（AC4）。不能只看 `bun pm ls`。

验证：批次 0 的断言全绿；AC4 的编辑器逐项验证（JSON 与 Markdown 高亮、lint 提示、搜索替换、撤销重做、快捷键）。

- [ ] 通知 `08-22-views-profiles-config`：桥接组件可用，其批次 2 的嵌入编辑器部分可开工。

## 批次 2：日志面板与监控

安全敏感位置，单独一批。

- [ ] `MonitoringView`(699) 迁移。
- [ ] ANSI 渲染经 `ansiRenderer.ts`，输出前经 `dompurify` 消毒，形态为 `dangerouslySetInnerHTML={{ __html: DOMPurify.sanitize(html) }}`。
- [ ] 加 `no-restricted-syntax` 规则：`dangerouslySetInnerHTML` 的值必须是 `DOMPurify.sanitize(...)` 的结果（`design.md` §4）。规则提交给 `08-22-arch-quality-perf` 或直接加入 `eslint.config.js`。
- [ ] `app-log` 走 `08-22-state-logic-port` 的事件桥接层（ref 累积 + 定时批量提交），不逐条更新。
- [ ] 日志缓冲上限与截断策略确定并实现。
- [ ] 日志脱敏经 `logRedact.ts`，面板与导出文件均无明文（AC6）。
- [ ] `monitoring-log-contracts.md` 的断言验证。

验证：AC5（注入含 ANSI 与 HTML 标签的内容，无脚本执行，smoke 测试断言）；AC7（订阅解绑；连续运行 30 分钟内存无持续增长，复用 `08-22-arch-quality-perf` 场景 3 脚本）。

## 批次 3 后半：MCP 管理器

共享层部分已在「批次 3 前半」（阶段 4a）交付。

- [ ] `views/mcp/McpManagerView`(523)、`McpPresetsPanel`(416)、`McpSyncPanel`(297)。
- [ ] `unified_mcp` 读写行为不变（R8）。

## 批次 4：Commands 与斜杠命令

- [ ] `CommandsView`(1,744) 移交统一层：填 Claude 平台的 `commandsConfig` + 薄壳（≤100 行）。
- [ ] `BaseSlashCommands`(507) 迁移。它是统一层的参照实现，`08-22-platform-unify` 不动它。
- [ ] `CommandFormModal`(247)、`CommandList`(163)。
- [ ] `commands/:client?` 路由的缓存行为：数据走 Query，流式累积缓冲入 Zustand，切回续读（`08-22-shell-port` 批次 3 已建，本批次在真实视图上验证）。

## 批次 5：同步与环境管理

- [ ] `SyncView`(1,032)、`components/sync/`（4 文件 1,554 行）。
- [ ] WebDAV 凭据掩码显示，同步前生成备份（AC8）。前端只保证不绕过 Rust 侧实现。
- [ ] `sync-security-contracts.md` 的断言验证（R4）。
- [ ] `SshManagementView`(508)、`WslManagementView`(415)。
- [ ] WSL 入口的平台判定消费 `runtimeState` / `tauriRuntime`，不自行判定（AC9）。

## 批次 6：tray 独立窗口

- [ ] `views/tray/`（3 文件 1,188 行）迁移。
- [ ] 按 `08-22-shell-port` 批次 0 的结论落地入口形态。
- [ ] tray 窗口的 `ErrorBoundary` 就位。
- [ ] 窗口生命周期行为不变（R10）。

验证：AC2 的 tray 独立窗口可打开。

## 批次 7：收口

- [ ] 本批次组件内 px 与 `rgba()` 归零，CodeMirror 主题内联样式可豁免并登记（AC11）。
- [ ] 三份契约的验证项全部通过（AC10）。
- [ ] `nextTick` 登记表落盘。
- [ ] 对应目录 `rg --files -g '*.vue'` 无匹配（AC1）。
- [ ] `git diff --stat src/api src/types`（应为空，AC12）。

## 验证命令

| 时机        | 命令                                                  |
| ----------- | ----------------------------------------------------- |
| 每批次后    | `bun run type-check`、`bun run lint`（AC13）          |
| 批次 0–5 后 | `bun run test:smoke`                                  |
| 批次 1 后   | `bun run build` + 产物核对 `@codemirror/state` 单实例 |
| 批次 2 后   | 连续运行 30 分钟的内存测量（场景 3 脚本）             |
| 批次 6 后   | `bun run tauri dev` 打开 tray 窗口                    |
| 交付前      | `just frontend-check-quick`、`bun run lint:ci`        |

## 交付门（父任务视图门的一部分）

- [ ] AC1–AC13（含 AC4.1）全部满足。
- [ ] AC3 的 11 条核心操作路径逐条验证并记录：命令增删改、斜杠命令管理、跨平台 MCP 读写、MCP 预设应用、MCP 同步、WebDAV 同步上传与下载、原始配置编辑与保存、日志实时查看与过滤、SSH 环境管理、WSL 环境管理（Windows）、tray 操作。
- [ ] 升级门已执行：`codemirror-expressiveness.md` 落盘，无法表达项数明确，超 3 条时已切自建 hook 并记录（AC4.1）。
- [ ] `@codemirror/state` 单实例经产物核对确认（AC4）。
- [ ] `dangerouslySetInnerHTML` 的消毒强制规则已生效。
- [ ] 日志注入安全断言通过（AC5），脱敏断言通过（AC6），30 分钟内存无增长（AC7）。
- [ ] `mcp-shared-interfaces.md` 落盘并已通知，且 `components/mcp/` 4 文件已迁为 React（批次 3 前半，属阶段 4a 门，早于 `08-22-platform-unify` 批次 5）。
- [ ] 三份契约验证通过（AC10）。

## 回滚点

批次 3 前半属阶段 4a，单独开 PR 并先行合入。**定稳后不回滚**——`08-22-platform-unify` 的 `PlatformMcpView` 依赖它。

其余批次各自独立提交。批次 1 若走到自建 hook 分支，`@uiw` 版本与自建版本各一次提交，可精确回退。批次 2 的消毒规则单独提交。

## 协同点

| 编号 | 内容                                | 对方                                         | 时机                                |
| ---- | ----------------------------------- | -------------------------------------------- | ----------------------------------- |
| F2   | `components/mcp/` 迁移 + 接口公示   | `08-22-platform-unify`                       | 批次 3 前半（阶段 4a），须早于对方批次 5 |
| B    | `codemirror-peer-check.md` 结论     | `08-22-dep-upgrade`                          | 前置                                |
| D    | 三份契约的重写稿                    | `08-22-test-contract-rebuild`                | 前置与批次 0                        |
| E    | Commands 统一层接口消费             | `08-22-platform-unify`                       | 批次 4                              |
| I    | i18n 调用形式                       | `08-22-i18n-port`                            | 全程                                |
| P    | 桥接组件交付后通知对方开工          | `08-22-views-profiles-config`                | 批次 1 后                           |
| —    | tray 入口形态结论；`app-log` 桥接层 | `08-22-shell-port`、`08-22-state-logic-port` | 前置与批次 2                        |
| —    | `dangerouslySetInnerHTML` 消毒规则  | `08-22-arch-quality-perf`                    | 批次 2                              |
