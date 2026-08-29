# CCR UI Agent Sessions 独立页面

## Goal

将 `ref/repo/agentsview` 中对 CCR 用户价值最高的本地 Agent 会话观测能力收敛为 `ccr-ui` 的独立 **Agent Sessions** 页面。首版面向用户点名的 Grok、Claude、Codex、OpenCode、Pi、OMP、Antigravity、Kimi 八个 Agent，提供高效、只读、可诊断的会话浏览体验，同时避免移植 AgentsView 的整套产品、远程服务和高成本检索系统。

## Background and Confirmed Facts

- 用户已确认页面定位为“统一会话观测页”，不是统一 Agent 配置管理页；最新范围要求首版尽量支持图片列出的八个 Agent。
- 图片中的 `OMP` 在参考实现中对应 OhMyPi，且与 Pi 共用 Pi-like transcript 格式，但必须保留独立身份（`ref/repo/agentsview/internal/parser/types.go:449-473`）。
- 参考实现对八个目标均有 provider 证据：Claude/Codex（`ref/repo/agentsview/internal/parser/types.go:136-176`）、OpenCode（`ref/repo/agentsview/internal/parser/types.go:242-252`）、Kimi（`ref/repo/agentsview/internal/parser/types.go:540-549`）、Grok（`ref/repo/agentsview/internal/parser/types.go:636-642`）、Antigravity IDE/CLI（`ref/repo/agentsview/internal/parser/types.go:764-789`）。详细格式与降级边界见 `research/agent-provider-matrix.md`。
- `ccr-ui` 当前是 React 19 + React Router + TanStack Query + Tauri 2；新功能必须沿用这套边界。
- 主导航的 workspace 分组当前只有 MCP Manager（`ccr-ui/src/config/mainLayoutShell.ts:18-29`），但已有 `/agents` 路由（`ccr-ui/src/shell/routeCatalog.ts:116-121`）。现有 `/agents` 是 Claude Agent 配置薄壳，与会话观测不同（`ccr-ui/src/features/platform/agents/AgentsHomeView.tsx:1-6`），必须保留。
- CCR 当前 session 归档只主动索引 Claude/Codex/`Gemini` 三个枚举值，并对每次发现的全部文件调用 `parse_files`（`ccr-ui/src-tauri/src/commands/usage.rs:302-374`）。`ccr-store` 的 Grok 分支当前明确返回 unsupported，且没有 OpenCode/Pi/OMP/Kimi session provider（`crates/ccr-store/src/sessions/parser.rs:38-45`）。
- `usage_session_archive` 可继续作为摘要事实源，但当前 `file_path` 全局唯一（`crates/ccr-db/src/database/schema.rs:270-294`），无法表示 OpenCode SQLite 等“单容器、多会话成员”来源，必须做兼容迁移。

## In Scope

- 新建 `/agent-sessions` 顶层路由和 `Agent Sessions` / `Agent 会话` 导航入口，在 workspace 分组中位于 MCP Manager 之上。
- 首版提供八个一等 Agent family：`grok`、`claude`、`codex`、`opencode`、`pi`、`omp`、`antigravity`、`kimi`；UI 标签使用图片中的名称，OMP 可在辅助说明中标注 OhMyPi。
- Antigravity family 同时识别 IDE 与 CLI 的本地 canonical roots，并在列表中统一筛选为 Antigravity、保留 source variant；Kimi 首版覆盖 `.kimi` 与 `.kimi-code` CLI transcript，不把 Kimi Work 桌面运行时暗中并入。
- 每个 provider 分别暴露 availability（未安装/无数据/可用/错误）与 fidelity（full/partial/locked）。目录缺失、格式不可读或加密内容无密钥时，页面显示原因与已得到的安全元数据，不伪装为完整支持。
- 提供按 Agent、项目/cwd、日期范围、源状态和元数据关键词的服务端筛选，以稳定 keyset cursor 分页。
- 提供会话摘要、消息/工具计数、时间范围、source variant/status 和有界 transcript 详情；默认加载最新一页，用户可显式加载更早消息。
- 通过专用 session provider 注册表统一单文件、multi-file bundle、SQLite member 三类来源；不把八个会话来源直接扩进全局配置平台枚举。
- 只读访问原 session source；前端只传 archive ID，后端由 provider 校验已归档 source descriptor，不接受任意文件路径或数据库 member。
- 将刷新改为 provider-owned 快速指纹的增量索引：未变更文件/容器不重新读全文、不解析、不重写；共享 SQLite 容器只做一次容器级判断。
- 复用现有 Usage 页面；Agent Sessions 仅展示可靠的会话摘要，需要跨会话成本/模型分析时链接到 `/usage`。
- 完整的加载、空、错误、缺失源、锁定/部分解析与刷新状态，以及明暗主题、中英文、窄宽响应式、键盘和 reduced-motion 支持。

## Requirements

- R1. 新页面必须是独立会话观测面，不改变现有 Agent 配置语义、路由或 CRUD 数据流。
- R2. 八个目标 Agent 必须拥有独立 provider 定义、fixture 和能力状态；“支持”必须由真实发现、摘要和 transcript 解析测试证明，不能只添加筛选项或图标。
- R3. provider 注册表必须把 Agent family 与 source variant/format 分离，并兼容单文件、bundle、SQLite member；不得为了此页扩大 `ccr_config::Platform` 的配置语义。
- R4. 列表查询必须在 SQLite 中筛选、稳定排序和分页，不把全部归档发到前端再筛选。
- R5. transcript 读取必须限定单页消息数，对大会话只增量加载、去重合并，且前端仅渲染可见窗口。
- R6. 刷新必须有并发去重、provider 级诊断和可观测工作计数；未变更二次刷新不得重新解析或重写摘要。
- R7. archive schema 必须安全表达共享容器成员，保留旧行可读性，并以 provider-owned source descriptor 作为重新定位与校验依据。
- R8. 新 IPC 域必须使用 Rust 命名 DTO + ts-rs 生成 TypeScript + registry-generated client，不新增 `Value` 类型擦除或直接 `invoke()` 逃生口。
- R9. 会话文本、system prompt、解密密钥和原始事件不得写入日志；默认只展示用户/助手消息，工具事件经结构化摘要后再展示。Antigravity 不新增凭据 UI，已有进程环境中无密钥时必须安全降级。
- R10. 必须使用现有语义 token、原语组件、API facade、查询缓存和 React 重渲染约束，不引入 Svelte/Go sidecar 或新的表格/虚拟化依赖。
- R11. 实施必须以 provider contract、回归测试和真实渲染走查验证导航顺序、数据边界、响应式、可访问性和性能合同。

## Acceptance Criteria

- [ ] AC1 (R1). workspace 导航中 `Agent Sessions` 位于 `MCP Manager` 之上，点击进入 `/agent-sessions`；直接刷新、返回导航和页面标题正常，现有 `/agents` 及平台 Agents 页仍保持原行为。
- [ ] AC2 (R2-R3). Grok、Claude、Codex、OpenCode、Pi、OMP、Antigravity、Kimi 八个选项始终可识别；每个 provider 至少有一个 canonical fixture 能完成发现、摘要归档和有界 transcript 读取。OMP 保持独立 `omp` 身份；Antigravity IDE/CLI 在 UI 中归为同一 family 但 source variant 可见；Kimi Work 不计入该验收。
- [ ] AC3 (R2,R9). provider availability 可区分未安装、无数据、可用和错误，session fidelity 可区分 full、partial、locked。Antigravity SQLite fixture 可完整读取；加密 fixture 在没有 `ANTIGRAVITY_KEY` 且没有明文 fallback 时返回 `locked`，有 brain/history fallback 时返回 `partial`，均给出安全说明且不泄露密文、密钥或原始事件。
- [ ] AC4 (R4). 列表可按八个 Agent、cwd/项目、日期、source state 与元数据关键词组合筛选；每页默认不超过 80 条、服务端硬上限 200 条，使用 `(updated_at, archive_id)` 稳定 cursor，无重复/跳项。
- [ ] AC5 (R5). 选中会话默认最多加载 100 条最新消息，服务端单页硬上限 200；“加载更早消息”按 provider cursor/ordinal 合并且不重复，列表与 transcript 均使用虚拟化渲染，单消息超过 256 KiB 时安全截断并标记。
- [ ] AC6 (R6). 增量刷新返回每个 provider 和总计的 `discovered/unchanged/fingerprinted/parsed/upserted/partial/locked/errors`；八类受控 fixture 的首次刷新建立归档，第二次未变更刷新 `parsed=0` 且 `upserted=0`。单文件/bundle 变更只处理对应 source；OpenCode SQLite 未变更时不逐 member 解析，单 member 水印变化时不重写其他成员。
- [ ] AC7 (R7). 迁移后两个 OpenCode session 可共享同一物理 `opencode.db` 并同时归档，旧 file-backed archive 行仍可读取。archive ID 对更新保持稳定，唯一性使用 `(agent, physical source, member)` 语义；前端响应不返回原始 source path/member locator。
- [ ] AC8 (R8-R9). 前端只发 archive ID 和有界查询参数；provider 重新验证 source root、kind、regular file/container 和 member ownership。命名 DTO、生成绑定和命令 registry 漂移门禁通过，错误与日志不泄露 transcript/path/secret 内容。
- [ ] AC9 (R10-R11). 页面具备稳定的 loading/empty/error/refreshing/missing/partial/locked 状态，支持明暗主题和中英文；滚动内容不使用 glass，无新主题 token。1440×900 与约 900px 窄宽布局可用，核心操作可键盘完成，焦点明确，reduced motion 下无必需动画。
- [ ] AC10 (R11). ccr-db、ccr-store/provider、Tauri 命令/服务、生成绑定、前端 smoke/type/lint/i18n 与浏览器走查全部通过，最终 `just ui-check` 和 `just ci` 通过。

## Out of Scope

- 除八个点名 Agent 外新增其他一等 provider；旧 archive 中其他平台数据只保持兼容，不扩展本任务 UI/fixture 承诺。
- Kimi Work 桌面运行时、Claude Cowork、Antigravity 远程来源；它们需要独立来源与隐私评审。
- 语义/混合检索、embedding 提供商与全局 transcript FTS；第一版只支持归档元数据检索和已加载 transcript 内的本地查找。
- AgentsView 的 Recall/Quality、Recent Edits、Pin/Trash、Data workspace、Git 结果、GitHub Gist 发布与 HTML 分享。
- SSE/文件 watcher 实时追尾；第一版使用手动增量刷新与已有 session-index job 状态。
- 远程同步、WSL/SSH 扫描、S3、PostgreSQL、DuckDB、独立 Go server/sidecar 和 Svelte 前端。
- 新建自定义 source path 设置页；首版只读取 canonical roots 与已存在的标准环境覆盖变量，不写用户环境或配置。
- 在本规划获得新的实施批准前修改 `ccr-ui`、`crates/` 或生成绑定。

## Risks and Deferred Items

- 八个上游格式会独立演化；每个 provider 必须用最小脱敏 fixture 固定格式版本和降级行为，未知版本 fail-soft 而不是跨 provider 猜测。
- Antigravity CLI 可能只有 AES 加密 `.pb`；无现有密钥时只能提供元数据/brain/history fallback，完整 transcript 明确为 locked。首版不收集、保存或提示用户粘贴密钥。
- OpenCode 同时存在 file storage 与 SQLite backend；共享容器必须使用容器级指纹和 per-session watermark，避免把一次刷新放大为 N 次 DB 读取或 hash。
- `gemini` 旧 archive 行与真正 Antigravity source 不可无条件等同；只对经过 provider 重新发现并验证的 source 归类为 Antigravity，旧行保留兼容且不静默改写语义。
