# CCR UI Agent Sessions 独立页面 — 技术设计

## Architecture and boundaries

本任务新增独立 `agent-sessions` 功能域，并在 `ccr-store::sessions` 内建立轻量 provider registry。registry 只借鉴 AgentsView 的 source/format 证据，不复制其 Go provider 框架、watcher、远程同步或数据库。

```text
Canonical local roots
  Grok / Claude / Codex / OpenCode / Pi / OMP / Antigravity / Kimi
        |
        | provider-owned discover + fingerprint + parse/page
        v
ccr-store session providers
        |
        | incremental reconcile; changed sources only
        v
ccr-db usage_session_archive + usage_session_source_state
        |
        | keyset list / archive-id lookup
        v
src-tauri services/agent_sessions.rs
        |
        | named DTOs + generated command registry
        v
src/api/domains/agentSessions.ts
        |
        v
features/agent-sessions (React Query + virtualized list/transcript)
        |
        v
/agent-sessions (workspace nav, immediately above MCP Manager)
```

Layer ownership:

- `crates/ccr-store`: provider definitions, root discovery, source descriptors, fingerprints, summary parsing and bounded message pages；不知道 Tauri/React DTO。
- `crates/ccr-db`: archive/source-state schema、migration 与 repository 查询；不打开原 transcript/container。
- `ccr-ui/src-tauri/src/services/agent_sessions.rs`: 业务编排、limit/cursor 校验、archive ID → provider source 解析、repository ↔ wire DTO 映射。
- `ccr-ui/src-tauri/src/commands/agent_sessions.rs`: 薄命令，只做 State 提取、`spawn_blocking` 和 job 委托。
- `ccr-ui/src/api/domains/agentSessions.ts`: generated client 的域门面，不直接 `invoke()`。
- `ccr-ui/src/features/agent-sessions`: 页面、query keys、纯 view model、组件和局部样式；不 import 其他 feature 域。

## Dedicated provider model

不得把 OpenCode/Pi/OMP/Kimi 等会话来源直接加到全局 `ccr_config::Platform`，因为该枚举同时驱动配置管理、CLI 安装和大量 exhaustiveness match。新域使用专用类型：

```text
AgentSessionAgentId
  = grok | claude | codex | opencode | pi | omp | antigravity | kimi

AgentSessionSourceKind
  = file | bundle | sqlite_member

AgentSessionSourceRef
  agent, variant, root_id, physical_path, member_id?, project_hint?
```

`AgentSessionProvider` 的最小合同：

- `definition()`：稳定 Agent ID、label、canonical roots、环境覆盖变量和 capabilities。
- `discover()`：流式/有界地产生 provider-owned `SourceRef`，不解析 transcript。
- `fingerprint()`：返回适合该 source kind 的快速状态；bundle 组合 companion stat，SQLite 容器只计算一次。
- `parse_summary()`：只为新/变化 source 生成摘要、计数和 source fidelity。
- `read_message_page()`：由 archive source descriptor 读取 latest/before cursor 的有界消息页。
- `validate_stored_source()`：在每次详情读取前重新验证 root、source kind、物理文件和 member ownership。

旧 `SessionParser` 的 Claude/Codex 调用通过 compatibility adapter 逐步迁移，既有非本任务消费者不在一次提交中被强制改用 UI DTO。

## First-version provider matrix

| UI family | Canonical sources | Source model | First-version behavior |
| --- | --- | --- | --- |
| Claude | `~/.claude/projects/**/*.jsonl`，尊重 `CLAUDE_CONFIG_DIR` / `CLAUDE_PROJECTS_DIR` | file | 完整摘要与有界 transcript；保留 malformed/truncated 降级 |
| Codex | `~/.codex/sessions/**/*.jsonl`、`~/.codex/archived_sessions/**/*.jsonl` | file | 完整摘要与 transcript；live/archived variant 可见 |
| Grok | `~/.grok/sessions/<cwd>/<session>/summary.json` + `chat_history.jsonl` 等 companions | bundle | summary 为 anchor，companion 共同 fingerprint；缺 companion 时 partial |
| OpenCode | `~/.local/share/opencode/storage/...` 或 `opencode.db` | bundle / sqlite_member | storage 与 SQLite fallback 都支持；DB 只读，一次容器 gate + per-session watermark |
| Pi | `~/.pi/agent/sessions/**/*.jsonl` | file | Pi-like parser，独立 `pi` 身份 |
| OMP | `~/.omp/agent/sessions/**/*.jsonl` | file | 复用 Pi-like parser，独立 `omp` 身份和标签 OMP/OhMyPi |
| Antigravity | `~/.gemini/antigravity` 与 `~/.gemini/antigravity-cli` 下的 conversation DB/PB、brain/history | file / bundle | IDE+CLI 归为一个 family，variant 可见；DB/明文 fallback 完整或 partial，加密无密钥为 locked |
| Kimi | `~/.kimi/sessions/**/wire.jsonl`、`~/.kimi-code/sessions/**/wire.jsonl` | file | legacy + kimi-code 归为 Kimi；Kimi Work 不纳入 |

路径和格式证据详见 `research/agent-provider-matrix.md`。标准环境变量只读，不新增设置 UI，不写用户全局环境。

## Archive schema and compatibility migration

`usage_session_archive` 保持摘要事实源。guarded additive/rebuild migration 增加：

- `source_variant TEXT NOT NULL DEFAULT ''`
- `source_kind TEXT NOT NULL DEFAULT 'file'`
- `source_member_id TEXT NOT NULL DEFAULT ''`
- `source_size INTEGER`
- `source_mtime_ns INTEGER`
- `source_stat_hash TEXT`
- `user_message_count INTEGER NOT NULL DEFAULT 0`
- `assistant_message_count INTEGER NOT NULL DEFAULT 0`
- `tool_use_count INTEGER NOT NULL DEFAULT 0`
- `source_fidelity TEXT NOT NULL DEFAULT 'full'`

旧 `idx_usage_session_archive_file_path` 必须在同一 migration 中替换为唯一索引：

```text
(platform, file_path, source_member_id)
```

其中 `file_path` 仍是后端私有物理 source/container path；`source_member_id` 为空表示普通 file/bundle，OpenCode SQLite 使用原生 session ID。`archive_id` 对首次插入后保持稳定，不再在 upsert 时覆盖；新行以 agent + canonical source locator 的散列生成 opaque ID，绝不把 raw path 作为前端可见 ID。

新增 `usage_session_source_state` 保存容器/root 级成功状态：

- `platform`, `source_path`, `source_kind` 组成唯一键。
- `source_size`, `source_mtime_ns`, `source_stat_hash`, `content_hash?`。
- `last_success_at`, `last_error_code?`。

它让 unchanged OpenCode DB/bundle 在进入 member 解析前就跳过。旧 file-backed archive 行以 `source_kind=file`、空 member 迁移；不无条件把旧 `gemini` 行改名为 `antigravity`，只有新 provider 重新发现并验证的 source 才使用 canonical `antigravity`。

查询索引：

- `(source_state, updated_at DESC, archive_id DESC)`
- `(platform, source_state, updated_at DESC, archive_id DESC)`
- `(platform, file_path, source_member_id)` unique

## Incremental refresh

刷新复用 `session_index_jobs` 的单一活跃任务语义，不新增并发 job registry：

1. 对八个 provider 读取 canonical/override roots，记录 absent/available。
2. `discover()` 只产生 narrow source descriptors 与 stat hints。
3. 批量查询 archive/source-state fingerprints。
4. source/container 快速状态相同：计为 unchanged，不读全文、不查询 SQLite members、不 upsert。
5. 状态变化：调用 provider fingerprint；hash/stat state 相同时只 touch source-state/last_seen。
6. 内容变化：`parse_summary()`；file/bundle 只处理该 source，SQLite 容器读取 per-session watermark 后只 upsert changed/new members。
7. provider discovery 完整成功后才 reconcile missing；partial/error pass 不获得删除/缺失判定权。
8. transaction 中批量 upsert/touch/reconcile，并返回 provider + total counters。

工作计数：`discovered`, `unchanged`, `fingerprinted`, `parsed`, `upserted`, `partial`, `locked`, `errors`。CI 的主门禁是不变式，不用绝对 wall-clock 作为唯一标准。

## List, provider status, and transcript contracts

`AgentSessionListRequestDto`：

- `agents?: AgentSessionAgentId[]`
- `query?: string`（trimmed，max 200 chars，只匹配 session_id/title/cwd）
- `cwd_prefix?: string`
- `started_at?: string`, `ended_at?: string`
- `source_state?: 'live' | 'missing' | 'deleted_by_user' | 'all'`
- `fidelity?: 'full' | 'partial' | 'locked' | 'all'`
- `cursor?: string`
- `limit?: number`（default 80，clamp 1..200）

Cursor 编码并校验 `(updated_at, archive_id)`；SQL 使用 descending lexicographic predicate。`AgentSessionPageDto` 只含窄行、`next_cursor`、filter-scoped counts、freshness/job snapshot，不含 transcript、physical path 或 member ID。

`AgentProviderStatusDto` 对八个 family 固定返回：availability、variants、detected source/session counts、fidelity、last success/error category。目录不存在是 `unavailable/no_data`，不是全局刷新错误。

详情请求只接受 `archive_id`, `before_cursor?`, `limit?`：

1. repository 按 archive ID 取私有 source descriptor。
2. registry 找到对应 provider，调用 `validate_stored_source()`。
3. provider 以 read-only 模式打开 file/bundle/SQLite member；不允许路径或 member 从前端覆盖。
4. 返回 stable provider cursor/ordinal、role、timestamp、bounded text、tool summary、`next_before`、`has_older`、source fingerprint 与 `clipped/stale/partial/locked`。

单页默认 100、硬上限 200；单消息 UTF-8 字节上限 256 KiB，安全截断并显式标记。日志仅记 opaque archive ID、agent、variant、计数和错误类别，不记 source path、message text、system prompt、raw JSON/PB 或 key。

## Typed IPC domain

新命令域：

- `agent_sessions_list(request) -> AgentSessionPageDto`
- `agent_sessions_get_detail(request) -> AgentSessionDetailDto`
- `agent_sessions_get_provider_status() -> Vec<AgentProviderStatusDto>`
- `agent_sessions_start_refresh() -> StartSessionIndexJobResponse`
- `agent_sessions_get_refresh_status(job_id) -> SessionIndexJobSnapshot`

DTO 在 `services/agent_sessions.rs` 或同域 `dto.rs` 命名，`#[derive(TS)]` 生成到 `src/types/generated/agent_sessions/`。所有 i64/u64 按 typed IPC spec 映射为 TS number；Option 输入字段为 optional。命令登记进 `define_command_registry!`，前端 wrapper 只调用 generated client。

## Frontend composition

```text
AgentSessionsView
  ├─ PageHeader: title, freshness, Refresh, Go to Usage
  ├─ ProviderStatusStrip: eight agents + availability/fidelity
  ├─ AgentSessionFilters: query, agents, date, source state/fidelity
  └─ workspace
      ├─ AgentSessionList (virtualized, cursor load-more)
      └─ AgentSessionDetail
          ├─ metadata/vitals/source variant
          ├─ partial/locked diagnostics
          ├─ loaded-transcript find controls
          └─ AgentTranscript (virtualized, load older)
```

- 导航 label `nav.agentSessions`，icon 使用 `MessagesSquare`；workspace 顺序固定为 Agent Sessions → MCP Manager。
- 桌面布局约 `24rem / minmax(0, 1fr)` 两栏；约 900px 以下改为列表在上、详情在下，不隐藏主要操作。
- 列表与 transcript 复用 `@tanstack/react-virtual`；稳定 key 分别为 `archive_id` 和 provider message key/ordinal。
- React Query keys 包含规范化 filters/cursor/archive ID。切换 filter/session 后不得混入旧页；刷新完成只失效 agent-session keys。
- 滚动区使用 opaque workspace/card surface，不用 glass/backdrop-filter，不新增主题 token。
- provider 状态图标配合文本/aria-label，不只靠颜色；选择与焦点状态分开，reduced motion 禁用布局位移动画。

## Performance contracts

| Boundary | Contract | Verification |
| --- | --- | --- |
| Refresh | O(discovered metadata + changed bytes/rows)，不得 O(all archive bytes) 重解析 | 八 provider 未变更二次刷新 `parsed=0/upserted=0` |
| Shared DB | 每个 unchanged OpenCode container 至多一次 stat/state gate，不按 member hash DB | instrumentation + multi-member fixture |
| Bundle | Grok/Antigravity companion stat 组成一个 source state；变更只影响对应会话 | companion-only mutation fixture |
| List | default 80，hard max 200，keyset cursor | 10k-row fixture 组合筛选/稳定分页 |
| Detail | default 100，hard max 200，per-message 256 KiB cap | long-session/clipped/older cursor tests |
| Frontend DOM | 列表与 transcript 虚拟化 | smoke test + browser DOM count observation |
| Refresh concurrency | 同一时间至多一个 session-index job | concurrent-start unit test |

## Security and privacy

- 原 session source 只读；页面不提供 delete/clone/publish/远程上传。
- provider validation 同时检查 configured root ownership、source kind、regular file/container 和 member ID；不从 archive 字符串拼接任意 SQL/URI/path。
- SQLite 使用只读连接与参数化 member 查询，不修改 WAL/SHM；如果平台/驱动无法保证零写入，先对受控临时副本验证，不对用户源试写。
- Antigravity 只读取进程已存在的 `ANTIGRAVITY_KEY`；不记录、不持久化、不经 IPC 返回，也不新增输入密钥 UI。无密钥时 fail-soft 为 locked/partial。
- 默认不展示 system prompt/secrets 区块；工具参数需结构化截断/摘要。

## Trade-offs and rollback

- 使用专用 provider registry 增加少量抽象，但隔离了全局配置平台枚举，且能真实表达 file/bundle/SQLite 三种来源。
- 八 provider fixture-backed 支持扩大了首版实现量；通过共享 Pi/OMP parser、统一 source contract 和分 provider 验收限制重复代码。
- 手动增量刷新易验证且避免 watcher 生命周期；代价是活跃会话不是秒级实时。
- 前端/路由/IPC 可按 `agent-sessions` 域整体回退，现有 MCP、Usage 和平台 Agent 配置不依赖它。
- DB migration 保留旧列/旧行；代码回退后新增列和 source-state 表无害。若 provider reconcile 有问题，可关闭新刷新入口并继续读取旧摘要，不删除归档或原 session。
