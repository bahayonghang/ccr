# Sync / MCP / Commands 与工具视图迁移

> 父任务：`08-22-react-migration`

## Goal

将同步、跨平台 MCP、命令管理、原始配置编辑器、监控日志与环境管理视图从 Vue 迁移到 React，约 12,055 行，并完成 CodeMirror 6 桥接重写。

## Scope

> **范围变更（跨平台统一决策后）**：`CommandsView.vue`（1,744）移交子任务 `08-22-platform-unify` 的 Commands 统一层，本任务改为提供 Claude 平台的 config 与薄壳视图。`views/mcp/McpManagerView.vue`（523）为跨平台 MCP 统一管理器，本身即统一层，留在本任务。本任务剩余约 10,311 行。精确切分由 `08-22-platform-unify` 的差异普查（R1）确定后回填本表。

| 文件 / 目录 | 行数 |
|---|---|
| `src/views/CommandsView.vue` | 1,744 |
| `src/components/mcp/`（4 文件） | 2,064 |
| `src/components/sync/`（4 文件） | 1,554 |
| `src/views/tray/`（3 文件） | 1,188 |
| `src/views/SyncView.vue` | 1,032 |
| `src/views/MonitoringView.vue` | 699 |
| `src/components/editor/`（2 文件） | 698 |
| `src/views/mcp/`（1 文件） | 523 |
| `src/views/SshManagementView.vue` | 508 |
| `src/components/BaseSlashCommands.vue` | 507 |
| `src/views/WslManagementView.vue` | 415 |
| `src/components/McpPresetsPanel.vue` | 416 |
| `src/components/McpSyncPanel.vue` | 297 |
| `src/components/CommandFormModal.vue` | 247 |
| `src/components/CommandList.vue` | 163 |
| 合计 | 12,055 |

关联的框架无关资产（原样复用，只改调用点）：`src/utils/ansiRenderer.ts`、`sanitize.ts`、`logRedact.ts`、`clipboard.ts`、`download.ts`、`text.ts`、`scheduling.ts`。

关联的契约：`raw-config-editor-contracts.md`、`sync-security-contracts.md`、`monitoring-log-contracts.md`。

关联的样式：`src/styles/checkin-shared.css` 之外的共享样式由 `08-22-design-system` 落位。

## Requirements

- R1 上表 26 个文件迁移为 React 组件，对应 `.vue` 文件删除。
- R2 CodeMirror 6 桥接改用 `@uiw/react-codemirror` 4.25.11（选型见父任务 `design.md` §10）。保留 9 个 `@codemirror/*` 包：`commands`、`lang-json`、`lang-markdown`、`language`、`legacy-modes`、`lint`、`search`、`state`、`view`。
- R2.1 前置依赖：`08-22-dep-upgrade` 需先完成 `@uiw/react-codemirror` 的 peer 依赖范围核对。CodeMirror 6 对 `@codemirror/state` 的多实例敏感，若产生重复实例需通过 `overrides` 收敛到单一版本。
- R2.2 **升级门**：`raw-config-editor-contracts.md` 的断言逐条验证可通过 `@uiw` 的 API 表达。无法表达的项累计超过 3 条时，改为自建 React hook（现有 `CodeSourceEditor.vue` 仅 235 行，自建成本约 200–250 行）。该判定点写入本任务 `implement.md` 作为显式检查门。
- R3 `raw-config-editor-contracts.md` 定义的编辑器行为在迁移后成立：语法高亮、JSON / Markdown 模式、lint 提示、搜索、快捷键。
- R4 `sync-security-contracts.md` 定义的同步安全行为不变：WebDAV 凭据掩码、同步前备份、冲突处理。
- R5 `monitoring-log-contracts.md` 定义的日志行为不变。ANSI 渲染经 `ansiRenderer.ts`，输出前经 `dompurify` 消毒，`v-html` 等价的危险渲染点改为 React 的受控 HTML 注入并保留消毒。
- R6 日志脱敏（`logRedact.ts`）在迁移后生效，日志面板不显示明文凭据。
- R7 日志流的 Tauri Event 订阅（`app-log`）在页面卸载后正确解绑，长时间运行不累积内存。
- R8 跨平台 MCP 管理（`unified_mcp`）的读写行为不变。
- R9 WSL 管理仅在 Windows 平台可见，平台判定行为不变。
- R10 tray 视图（3 文件，1,188 行）在独立窗口中渲染，窗口生命周期行为不变。
- R11 IPC 调用点沿用 `src/api` 现有 wrapper，不新增或修改 wrapper。

## Acceptance Criteria

- [ ] AC1 上表 26 个文件全部迁移，对应目录下 `rg --files -g '*.vue'` 无匹配。
- [ ] AC2 全部视图的路由可达，页面渲染无报错，tray 独立窗口可打开。
- [ ] AC3 核心操作路径手动验证通过并记录：命令增删改、斜杠命令管理、跨平台 MCP 读写、MCP 预设应用、MCP 同步、WebDAV 同步上传与下载、原始配置编辑与保存、日志实时查看与过滤、SSH 环境管理、WSL 环境管理（Windows）、tray 操作。
- [ ] AC4 CodeMirror 编辑器验证：JSON 与 Markdown 语法高亮、lint 错误提示、搜索替换、撤销重做、快捷键，逐项记录。`@codemirror/state` 无重复实例，由构建产物核对确认。
- [ ] AC4.1 `raw-config-editor-contracts.md` 断言的可表达性逐条记录。无法通过 `@uiw` API 表达的项数落盘；超过 3 条时给出换自建 hook 的判定与执行结果。
- [ ] AC5 日志面板注入含 ANSI 转义与 HTML 标签的内容后，渲染结果无脚本执行，由 smoke 测试断言。
- [ ] AC6 日志面板与导出文件中无明文凭据，由 smoke 测试断言。
- [ ] AC7 `app-log` 事件订阅在页面卸载后解绑，连续运行 30 分钟内存占用无持续增长。
- [ ] AC8 WebDAV 凭据在界面显示为掩码，同步前生成备份。
- [ ] AC9 WSL 管理入口在非 Windows 平台不可见。
- [ ] AC10 三份契约（`raw-config-editor-contracts.md`、`sync-security-contracts.md`、`monitoring-log-contracts.md`）的验证项全部通过。
- [ ] AC11 本批次组件内 px 字面量与 `rgba()` 数量为 0（登记豁免除外，CodeMirror 主题内联样式可豁免并登记）。
- [ ] AC12 `src/api` 的 git diff 为空。
- [ ] AC13 `bun run type-check` 与 `bun run lint` 退出码 0。

## 前置与后续

- 前置：`08-22-shell-port`。
- 可与其余六个视图子任务并行。
- i18n 调用点在本批次内同步转换，运行时切换与收尾校验属 `08-22-i18n-port`。

## Out of Scope

- 新增功能与信息架构调整。
- `src/api`、`src/types` 的修改。
- `src-tauri/src/commands/sync.rs`、`unified_mcp.rs`、`wsl.rs`、`environment.rs` 的改动。
- `crates/ccr-sync/` 的改动，含加密与凭据处理。
- 更换编辑器。本任务只替换 Vue 绑定，保留 CodeMirror 6。
- `ExecutionEnvironment` trait 与 SSH / WSL 环境抽象的 Rust 侧实现。

## Notes

- CodeMirror 6 桥接重写是本批次的主要风险。CodeMirror 6 本身框架无关，桥接层负责生命周期、扩展装配与受控值同步。建议先重写 `raw-config-editor-contracts.md` 为可执行断言，再改实现。
- 日志面板的 `v-html` 等价渲染点是安全敏感位置。迁移后必须保留 `dompurify` 消毒，不得改为直接注入。
- `MonitoringView.vue` 原按业务归类可入 Usage 域，此处按其依赖（`ansiRenderer.ts`、`logRedact.ts`、`app-log` 事件）归入本批次。
