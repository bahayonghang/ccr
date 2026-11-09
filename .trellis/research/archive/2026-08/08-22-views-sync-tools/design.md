# 技术设计：Sync / MCP / Commands 与工具视图迁移

> 父任务：`08-22-react-migration`。本域 12,055 行（移交统一层后约 10,311 行）。主要风险为 CodeMirror 6 桥接重写与日志面板的 HTML 注入安全。

## 1. 本批次共性转换

与 `08-22-views-claude` 的 `design.md` §1 同表，不重复。前置阅读同，另加 `raw-config-editor-contracts.md`、`sync-security-contracts.md`、`monitoring-log-contracts.md`。

## 2. 范围与统一层切分

`CommandsView.vue`(1,744) 移交 `08-22-platform-unify` 的 Commands 统一层，本任务改为提供 Claude 平台的 config 与薄壳。

`views/mcp/McpManagerView.vue`(523) 为跨平台 MCP 统一管理器，本身即统一层，留在本任务。

`BaseSlashCommands.vue`(507) 留在本任务——它是统一层的参照实现，`08-22-platform-unify` 不动它（其 Scope 明确 `slashCommands.ts` 192 行不动）。三个平台的薄壳视图分属其他子任务。

本任务剩余约 10,311 行。精确切分在 `platform-unify` 批次 8 回填。

## 3. CodeMirror 6 桥接（本域主要风险）

### 3.1 现状与目标

现状：`CodeSourceEditor.vue`(235 行，10 处 `EditorView` / `EditorState` / `Compartment` 引用) + `ConfigSourcePanel.vue`(463 行)，合计 `components/editor/` 698 行。

目标：`@uiw/react-codemirror` 4.25.11。保留 9 个 `@codemirror/*` 包：`commands`、`lang-json`、`lang-markdown`、`language`、`legacy-modes`、`lint`、`search`、`state`、`view`。

CodeMirror 6 本身框架无关。桥接层负责三件事：生命周期（创建 / 销毁 `EditorView`）、扩展装配（`Compartment` 的动态重配置）、受控值同步（外部值变化 → 编辑器内容，编辑器变化 → 外部回调）。

### 3.2 契约先转为可执行断言

PRD Notes 建议：先重写 `raw-config-editor-contracts.md`（7.4 KB）为可执行断言，再改实现。本设计采纳。

断言覆盖 R3 的五项：语法高亮、JSON / Markdown 模式、lint 提示、搜索、快捷键。

### 3.3 升级门（R2.2）

`raw-config-editor-contracts.md` 的断言逐条验证可通过 `@uiw` 的 API 表达。**无法表达的项累计超过 3 条时，改为自建 React hook。**

自建成本参考：现有封装 235 行，自建约 200–250 行。

「无法表达」的判定标准：该断言所需的能力在 `@uiw/react-codemirror` 的 props 与 ref API 中无对应项，且无法通过传入 `extensions` 数组绕过。能通过 `extensions` 表达的不算无法表达——`@uiw` 允许传原生 CodeMirror 扩展，多数能力可由此获得。

因此实际达到 3 条的可能性不高。真正的风险点是受控值同步与 `Compartment` 动态重配置——若 `@uiw` 的受控模式与现有的 `Compartment` 用法冲突，那一条就是硬阻塞。

逐条记录落盘为 `codemirror-expressiveness.md`（AC4.1）。

### 3.4 多实例检查

`08-22-dep-upgrade` 已完成 peer 依赖核对并产出 `codemirror-peer-check.md`（协同点 B）。本任务在实现后再验一次：构建产物中 `@codemirror/state` 只打进一份（AC4）。

CodeMirror 6 的插件系统在多个 `state` 实例下抛运行时错误，因此该检查不能只看 `bun pm ls`，需看产物。

## 4. 日志面板的 HTML 注入（安全敏感位置）

R5：ANSI 渲染经 `ansiRenderer.ts`，输出前经 `dompurify` 消毒。`v-html` 等价的危险渲染点改为 React 的受控 HTML 注入并保留消毒。

React 侧形态：`dangerouslySetInnerHTML={{ __html: DOMPurify.sanitize(html) }}`。

**消毒不得被省略**（PRD Notes：迁移后必须保留 `dompurify` 消毒，不得改为直接注入）。

强制方式：`no-restricted-syntax` 规则，匹配 `dangerouslySetInnerHTML` 的值不是 `DOMPurify.sanitize(...)` 调用结果的写法。该规则提交给 `08-22-arch-quality-perf` 的规则集，或本任务直接加入 `eslint.config.js`。

AC5 由 smoke 测试断言：注入含 ANSI 转义与 HTML 标签的内容，渲染结果无脚本执行。

## 5. 日志流与内存（R7）

`app-log` 事件订阅在页面卸载后正确解绑，长时间运行不累积内存。

设计：

- `app-log` 是高频事件。按 `08-22-state-logic-port` §3 的约定，走 ref 累积 + 定时批量提交，不逐条 `setQueryData`。该形态由对方的事件桥接层提供，本任务消费。
- 日志缓冲需有上限与截断策略（与 `08-22-shell-port` §4 的流式缓冲同类问题）。上限值在实施时定，与 `MonitoringView` 的可见行数配合。
- AC7：连续运行 30 分钟内存占用无持续增长。测量方法复用 `08-22-arch-quality-perf` 场景 3 的脚本。

日志脱敏（R6）经 `logRedact.ts`（框架无关，原样复用）。日志面板与导出文件均无明文凭据（AC6）。

## 6. tray 独立窗口（R10）

`src/views/tray/`（3 文件 1,188 行）在独立窗口中渲染，窗口生命周期行为不变。

路由 `/tray/codex` 是 2 个顶层条目之一，不套 `MainLayout`（`08-22-shell-port` §1）。该任务的批次 0 已确认 tray 是否需要独立 HTML 入口。本任务按其结论落地视图。

tray 窗口有独立的 `ErrorBoundary`（`08-22-shell-port` §9）。

## 7. WSL 平台判定（R9）

WSL 管理仅在 Windows 平台可见，平台判定行为不变。

判定来源为 `runtimeState.ts` / `tauriRuntime.ts`（由 `08-22-shell-port` 接线）。本任务消费其判定结果，不自行判定。

AC9：WSL 管理入口在非 Windows 平台不可见。

## 8. 同步安全（R4）

`sync-security-contracts.md`（4.2 KB）定义的行为不变：WebDAV 凭据掩码、同步前备份、冲突处理。

四项行为由 Rust 侧（`crates/ccr-sync`，不改）实现。前端责任与 `08-22-views-profiles-config` §5 同类——只保证不绕过。

AC8：WebDAV 凭据界面显示为掩码，同步前生成备份。

## 9. 跨平台 MCP（R8）

`unified_mcp` 的读写行为不变。`src-tauri/src/commands/unified_mcp.rs` 不改。

`components/mcp/`（4 文件 2,064 行）是共享组件层，`08-22-platform-unify` 复用不改造（其 Out of Scope）。因此本任务对该目录的迁移需保持接口稳定，与 `components/profiles/` 同类要求（`08-22-views-profiles-config` §3）。

接口定稳的公示要求与其相同，落盘为 `mcp-shared-interfaces.md`，通知 `08-22-platform-unify`。

## 10. 框架无关资产

`src/utils/ansiRenderer.ts`、`sanitize.ts`、`logRedact.ts`、`clipboard.ts`、`download.ts`、`text.ts`、`scheduling.ts` 原样复用，只改调用点。

## 11. 不变量

- IPC 调用点沿用现有 wrapper（R11）。`git diff --stat src/api` 须为空（AC12）。
- `src/types` 不改。
- `src-tauri/src/commands/sync.rs`、`unified_mcp.rs`、`wsl.rs`、`environment.rs` 不改。
- `crates/ccr-sync/` 不改。
- 不更换编辑器，只替换 Vue 绑定。
- `ExecutionEnvironment` trait 与 SSH / WSL 环境抽象的 Rust 侧实现不改。

## 12. 未决项

- `@uiw` 的受控模式与现有 `Compartment` 用法是否冲突（第 3.3 节末段）——这是升级门的实际风险点。
- 日志缓冲的上限值与截断策略（第 5 节）。
- tray 是否需要独立 HTML 入口（依赖 `08-22-shell-port` 批次 0 的结论）。
- 本任务的精确文件清单待 `platform-unify` 批次 8 回填。
