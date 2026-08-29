# CCR UI Agent Sessions 独立页面 — 执行计划

## Ordered checklist

1. **冻结 provider contract 与脱敏 fixture**
   - 为 Grok、Claude、Codex、OpenCode storage/SQLite、Pi、OMP、Antigravity IDE/CLI、Kimi legacy/kimi-code 建立最小脱敏 fixture。
   - fixture 覆盖正常、空文件、末尾半行、超长消息、未知事件、源删除；Antigravity 额外覆盖 DB、明文 fallback、加密无 key；OpenCode 额外覆盖共享 DB 两个 members。
   - 先写 provider contract 红测：discover → fingerprint → summary → bounded page → validate stored source。

2. **建立专用 session provider registry**
   - 在 `ccr-store::sessions` 新增 `AgentSessionAgentId`、source kind/ref/fingerprint/capability/status 与 provider trait。
   - 定义八个 family 和 canonical roots/标准环境覆盖；Antigravity IDE/CLI 与 Kimi legacy/kimi-code 用 variant 表达。
   - 为旧 Claude/Codex session parser 建 compatibility adapter；不扩大全局 `ccr_config::Platform`。

3. **扩展 archive 与 source-state schema/repository**
   - 同步更新 fresh schema 和 guarded migration，增加 variant/kind/member/fingerprint/计数/fidelity 列。
   - 替换全局 `file_path` unique 为 `(platform, file_path, source_member_id)`，确保旧 file rows 迁移为空 member，且 upsert 不覆盖既有 archive ID。
   - 新增 `usage_session_source_state` 容器级状态；实现批量 fingerprint lookup、touch-only、batch upsert、按 archive ID 私有 descriptor 查询、组合筛选 + keyset cursor。
   - 所有写入在 transaction 中；partial/error discovery 不获得 missing reconcile 权限。

4. **抽出增量归档编排**
   - 将 `commands/usage.rs` 中旧 `sync_platform_session_archive` 迁到 State-free service，让现有 home/session-index job 与新页面刷新共用。
   - 实现 provider roots → narrow discovery → batch state lookup → provider fingerprint → changed-only parse → batch upsert/touch → authoritative reconcile。
   - 暴露 provider 与 total `discovered/unchanged/fingerprinted/parsed/upserted/partial/locked/errors`，保持旧 job 消费者兼容与单任务去重。

5. **迁移 Claude/Codex 并实现 Grok provider**
   - Claude/Codex 复用现有解析语义并补 bounded page、archived Codex root、malformed/truncated fixture。
   - Grok 以 `summary.json` 为 anchor，组合 `chat_history.jsonl`、`signals.json`、`updates.jsonl`、`prompt_context.json` 的 stat/fingerprint；缺 transcript companion 时返回 partial。
   - 验证只改变一个 companion 时只重建该 Grok session。

6. **实现共享 Pi-like 与 Kimi providers**
   - 用一个参数化 Pi-like parser 服务 Pi 与 OMP，保证 `pi:` / `omp:` identity、parent/session 字段和 tool events 不串线。
   - Kimi 识别 `.kimi/sessions` 与 `.kimi-code/sessions` 的 `wire.jsonl`，过滤辅助 agent/非用户 conversation 结构，保持 family 为 `kimi`、variant 可见。
   - 每个 provider 都实现 summary 与 latest/before bounded page，不把整个 archive 读入前端。

7. **实现 OpenCode storage + SQLite provider**
   - 复用 CCR 已有 OpenCode 默认目录和 usage DB 读取经验，支持 storage/session + message/part 文件布局及 `opencode.db` fallback。
   - file storage 使用 session composite stat；SQLite 使用只读容器 gate、per-session watermark/member ID 和参数化查询。
   - 验证同一 DB 两个 members 可同时归档、unchanged 容器不逐 member parse、单 member 变化不重写其他 rows。

8. **实现 Antigravity IDE/CLI provider 与安全降级**
   - 支持 `~/.gemini/antigravity` conversation DB + brain/annotation bundle，以及 `antigravity-cli` DB/PB + brain/history fallback。
   - 只读 SQLite；仅在进程已有 `ANTIGRAVITY_KEY` 时尝试解密，不新增 key 设置/持久化/IPC。
   - 无 key 或未知格式时返回 locked/partial 与错误类别；日志、DTO、fixture 断言均不得泄露密文、key、raw path 或 message 原文。

9. **新增 typed Agent Sessions IPC 域**
   - 建立 State-free `services/agent_sessions.rs` 和薄 `commands/agent_sessions.rs`，实现 list/detail/provider-status/start-refresh/get-refresh-status。
   - 实现 archive ID → provider-owned source descriptor → root/kind/member 验证；后端校验 limit/query/cursor/date。
   - 在 handler registry 登记 ReadOnly/runtime policy，生成 `src/types/generated/agent_sessions/` 与 generated client，建立 `src/api/domains/agentSessions.ts` facade。

10. **接入路由、导航、i18n 与页面数据层**
    - 新增 lazy feature route `/agent-sessions`、title map 与 workspace nav 顺序 Agent Sessions → MCP Manager；保留 `/agents` 和各平台 Agent 路由。
    - 添加八 Agent 标签、provider status/fidelity、筛选、状态、错误与加载的中英文文案；OMP 主标签保持 `OMP`。
    - 建立 query keys、filter normalization、cursor pages 合并、选中 session 状态与 refresh job 去重；切换 filter/session 不混入旧页。

11. **实现高密度 UI 与虚拟化**
    - 实现 header/provider strip/filter/list/detail/transcript；桌面双栏、约 900px 以下纵向。
    - session/message rows 使用 `memo`、稳定 key/回调与现有 `@tanstack/react-virtual`；已加载 transcript find 明确标注局部范围。
    - 完成 loading/empty/error/refreshing/missing/partial/locked/clipped 状态、可见 focus、aria 文本、明暗主题和 reduced motion。

12. **回归、性能、真实渲染与收口**
    - 运行 provider fixture 矩阵、10k archive keyset、shared DB、bundle-only mutation、二次 unchanged refresh、并发 job 与 path/member tamper 测试。
    - 浏览器验证 1440×900、约 900px、light/dark、中文/英文、键盘、八 Agent 筛选、partial/locked 状态与虚拟 DOM 数量。
    - 先跑窄门禁，再跑 `just ui-check` 与 `just ci`；未获得 native Tauri 视觉证据时明确记录为 `UNVERIFIED`，不以 web preview 替代桌面证明。

## Validation commands

外部命令按仓库 RTK 约定加 `rtk`：

```powershell
rtk cargo test -p ccr-store sessions::providers
rtk cargo test -p ccr-db usage_session_archive
rtk cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml agent_sessions
rtk cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml command_registry
rtk just tauri-bindings
rtk just tauri-bindings-check

Set-Location ccr-ui
rtk bun run test -- agent-sessions
rtk bun run test -- api-boundaries
rtk bun run typecheck
rtk bun run lint
rtk bun run i18n:check
Set-Location ..

rtk just fmt-check
rtk just lint-strict
rtk just frontend-check-quick
rtk just ui-check
rtk just ci
```

Web preview 的 Playwright/截图流程按 `.codex/skills/ccr-ui-visual-workflow/SKILL.md` 执行，不默认启动 Tauri 桌面壳。

## Risky files and rollback points

- `crates/ccr-db/src/database/schema.rs`、`migrations.rs`、`repositories/usage_repo.rs`：先以 fresh + upgrade + two-members-one-container 测试锁定迁移；失败时回退新 migration/repository，不删用户 DB。
- `crates/ccr-store/src/sessions/**`：provider registry 与八 adapters 是最大风险面；每个 provider 保持独立模块/fixture，单个 provider 可禁用而不影响其他七个。
- `ccr-ui/src-tauri/src/commands/usage.rs` 与 session job：先抽兼容服务再替换调用，避免 Usage 首页刷新回归。
- generated handler/client/bindings：生成器为唯一来源，不手改生成文件；漂移时回退 command 注册和生成结果为一组。
- `ccr-ui/src/config/mainLayoutShell.ts`、`routeCatalog.ts` 与 feature 目录：新路由独立，回退不触碰 `/agents`、MCP 或 Usage。
- Antigravity/OpenCode 用户源：全程只读；任何驱动可能创建/修改 WAL/SHM 的证据都阻止直接读取，先修正连接策略，不复制或改写用户源作为默认行为。

## Follow-up before `task.py start`

- 本次范围调整属于重大规划变更，旧批准（若有）失效。
- 重新运行 task convergence、JSONL manifest、path 与 `git diff --check` 验证。
- 向用户展示新的八 Agent 支持矩阵、降级边界和性能合同；只有用户在该最终摘要后的下一条消息明确批准实施，才运行 `task.py start 08-29-ui-agents-page`。
