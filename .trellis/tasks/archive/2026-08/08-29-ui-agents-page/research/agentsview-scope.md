# AgentsView 能力筛选与 CCR 证据映射

## Problem restatement

需要解决的问题不是“把 AgentsView 搬进 CCR”，而是“让 CCR 用户以受控的读取、索引和渲染成本，看清八类本地 Agent 会话发生了什么”。

## Reference capability inventory

| AgentsView ability | Decision | CCR first-version form | Basis |
| --- | --- | --- | --- |
| Session browser | 保留 | 八 Agent 统一会话列表 + 详情 | `PRODUCT.md:7-19`, `README.md:296-322` |
| Multi-provider discovery | 保留并收敛 | 专用 provider registry；只做用户点名的八个 family | `internal/parser/types.go:132-176,242-252,449-473,540-549,636-642,764-789` |
| Provider capabilities/diagnostics | 保留 | available/full/partial/locked/error | 不能把加密/缺格式伪装成完整支持 |
| Platform/project/date filters | 保留 | SQLite server-side filters + keyset cursor | CCR 已有 archive columns；需扩共享容器身份 |
| Transcript reading | 保留 | latest page + load older + virtualization | AgentsView provider parsers；CCR Codex 页已有 bounded detail 基线 |
| Session vitals | 保留 | message/tool/time/source variant/fidelity | `ccr_store::Session` 已有部分计数 |
| Full-text search | 简化 | global metadata only；loaded transcript local find | archive 不持久化 message；全盘扫描违反性能目标 |
| Usage/cost dashboard | 简化 | session summary + link `/usage` | CCR 已有 llmusage-backed Usage 页面 |
| Live updates | 简化 | manual incremental refresh + job progress/dedupe | 可验证 O(delta)，不先引入 watcher 生命周期 |
| Keyboard-first | 保留 | 标准键盘流、visible focus、search shortcut | CCR 重度用户定位 |
| Semantic/hybrid search | 排除 | 后续独立任务 | 需 embedding、索引、数据与隐私边界 |
| Analytics/heatmap | 排除 | 复用 Usage | 避免重复数据产品 |
| Recall/Quality/Recent Edits | 排除 | 无 | 需额外提取、Git/评分模型和新表 |
| Pin/Trash/Publish/Export | 排除 | read-only first version | 不是“看清会话”的最小必需机制 |
| Data workspace/remote sync | 排除 | local-only | 超出页面价值和授权边界 |

## CCR assets and gaps

- 导航与路由：`ccr-ui/src/config/mainLayoutShell.ts:18-29`、`ccr-ui/src/shell/routeCatalog.ts:116-121`。
- 现有 Agent 配置语义：`ccr-ui/src/features/platform/agents/AgentsHomeView.tsx:1-6`；不得与会话观测合并。
- Codex 会话 UX：`ccr-ui/src/features/codex/CodexSessionsView.tsx:38-74`、`SessionDetailPanel.tsx:35-45`；可复用 bounded detail 与 virtualization 经验。
- 会话归档：`crates/ccr-db/src/database/schema.rs:270-294` 与 `usage_repo.rs:817-904`；当前 file path unique 是共享 SQLite container blocker。
- 当前 parser：`crates/ccr-store/src/sessions/parser.rs:38-45,508-527`；只对 Claude/Codex/Gemini/Qwen/Droid 有分支，Grok 明确 unsupported，不能宣称已有八 Agent 支持。
- 当前刷新：`ccr-ui/src-tauri/src/commands/usage.rs:302-374`；全文件 parse 是主要性能风险。
- OpenCode usage 导入已有只读 DB 经验：`crates/ccr-db/src/services/usage_import_service.rs:182,335-359,850`；session provider 可复用连接/路径经验，但不能把 usage token row 当 transcript。

## Performance lessons selected from AgentsView

- 热刷新按 changed source 而不是 archive 总量伸缩；未变更 source/container 不重新解析/写入（`docs/internal/performance-gates.md`）。
- 用工作计数不变式作为 CI 主门，不以易抖动 wall-clock 为唯一阈值。
- 共享 SQLite container 必须先做 container gate，再读 per-session watermark；不得每个 member 重复 hash/open 整个 DB。
- bundle provider 必须将 companion stat 纳入 source state，防止只变 companion 而错误跳过。
- 列表窄投影 + cursor；只对选中项加载详情；两级虚拟化使用稳定 archive/message key。
- 刷新任务去重，不让重复点击启动并发全量扫描。

## Decision

第一版以“八 Agent provider 状态 + 会话浏览 + 有界 transcript + 增量刷新”构成最小闭环。八 Agent 的精确来源、变体和降级语义见 `agent-provider-matrix.md`。全局 transcript FTS、语义 Recall、近期文件编辑、Kimi Work 与远程同步不属于该闭环。
