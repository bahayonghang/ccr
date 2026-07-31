# Grok CLI profile 切换支持（底层 + TUI）

## Goal

以 `~/.grok/config.toml` 的 `[model.*]`/`[models].default` 模型为基础，参照 ccr codex 第三方 profile 切换架构，为 Grok CLI (xAI Grok Build) 提供 CCR profile 切换：Platform 枚举/GrokPlatform 底层引擎、`ccr grok` CLI 命令面、TUI Grok Profile tab。不含 ccr-ui 页面。

> rev2（2026-07-28）：按 Codex No-Go 审阅修订——8 项发现全部核验属实并已吸收（CORR-001/002/003/004/005、ARCH-001/002、SEC-001）。

## 背景与来源需求

用户在 `~/.grok/config.toml` 中手工维护第三方中转（BYOK）配置使用 Grok CLI，切换供应商/账号需手改 TOML 且明文 key 无托管。CCR 已为 Codex 提供成熟的第三方 profile 切换架构（双路分发 + 入口状态 + 原子写 + Secret 脱敏 + registry 指针），本任务把同等能力带给 Grok。

## Requirements

### 范围

- ✅ 底层：`Platform::Grok` 枚举 + `GrokPlatform`（`PlatformConfig` trait 实现）切换引擎，含共享枚举全部穷举 match 位置的显式 capability 决策（见 research/platform-enum-impact-map.md）
- ✅ CLI：**`ccr grok profile <current|list|switch|create|set-field|enable|disable|delete|off>`** 子命令树（对齐现行 `ccr claude` / `ccr codex` 架构；`ccr platform switch/profile` 已退休，不复活），并在 `docs/examples/` 提供无真实凭据的 CCR profile 与 Grok 运行时配置示例
- ✅ TUI：Grok Profile tab（列表、Grok 专用详情、切换、tab 顺序持久化迁移）
- ❌ 不做：ccr-ui 页面；Grok 官方 auth.json 读写/备份（会话凭据完全归 `grok login`，自动刷新 + hot reload 使外部快照必然过期）；sessions/usage/skills-MCP 注入域（枚举 match 处显式 skip/拒绝）；Windows DACL 硬化（见"后续候选"）

### 上游事实（rev2，详见 research/grok-config-format.md）

1. 运行时配置：`$GROK_HOME/config.toml`，缺省 `~/.grok/config.toml`。
2. 自定义模型：`[model.<alias>]` 段（`model`/`base_url`/`name`/`api_key`/`env_key`/`api_backend`(`chat_completions|responses|messages`)/`context_window` 等）；`[models].default` 可缺省——缺省回落上游内置默认模型（当前 HEAD 为 grok-4.5，会漂移，不得硬编码）。
3. 凭据解析三层：per-model `api_key`/`env_key` > `~/.grok/auth.json` 会话 token > `XAI_API_KEY`（兼容 `GROK_CODE_XAI_API_KEY`）。
4. xAI 官方建议优先 `env_key` 而非明文 `api_key`；CCR 双模式支持并以 env_key 为推荐口径。
5. config.toml 同时承载 `[cli]/[session]/[memory]/[ui]/[subagents]/[marketplace]/[endpoints]` 等用户段，切换时必须保留（结构与值层面；toml round-trip 不保注释/格式，入口状态兜底，见"明文与保真披露"）。

### 三项核心决策（经审阅裁定）

1. **固定托管别名 `custom`**：保留，但入口状态必须结构化记录原始 `[model.custom]` 与原始 `[models].default`；切回官方/off 时"原存在则恢复原条目、原不存在才删除"（用户现网 `[ui].fork_secondary_model="custom"` 引用该别名，不可悬空）。
2. **inline `api_key` / `env_key` 互斥**：两者同设报校验错；`env_key` MVP 仅支持单字符串（上游 array 形态明确拒绝并提示）。
3. **官方 profile 定位为纯模型选择器**：认证完全归 auth.json / `XAI_API_KEY`；官方 profile 携带 `auth_token`/`env_key` 报校验错；未指定 model 时移除 `models.default` 回落上游默认。

### 明文与保真披露（SEC-001 决策）

采用"接受并准确披露"路线（与 claude/droid 现行 profile 存储一致）：

- 明文 api_key 存在位置矩阵：① `~/.ccr/platforms/grok/profiles.toml`（Secret 类型内存脱敏，落盘走 `expose_plaintext_option`）② 其轮换备份 ③ 入口状态文件 ④ `~/.grok/config.toml` 运行时（Grok 自身要求）。
- 权限事实：`secret=true` 仅 Unix 0o600；**Windows 上为 no-op**（依赖 `%USERPROFILE%` 默认 ACL）。此为 ccr-core 共享原语现状，非 Grok 特有。
- 缓解口径：文档/TUI/CLI 提示均推荐 `env_key` 模式（CCR 侧零明文）；"唯一明文消费点"类表述一律废除。

### 子任务地图

| 子任务 | 交付物 | 依赖 |
|---|---|---|
| `07-28-grok-platform-core` | 枚举扩展 + 全 workspace 穷举 match 决策落地 + GrokPlatform 切换引擎（含恢复语义/CAS 写/删除语义）+ 测试 | 无（首先执行） |
| `07-28-grok-cli-surface` | `ccr grok` 子命令树 + 共享 profile 机制放开与类型化字段解析 + 帮助/文档/`docs/examples/` 示例配置 + 固定面测试 | core 完成后 |
| `07-28-grok-tui-tab` | TUI Grok Profile tab + Grok 专用详情 + tab_order 迁移语义修复 | core 完成后（与 cli-surface 可并行） |

## Acceptance Criteria

- [ ] `ccr grok profile create relay --base-url https://api.example.com/v1 --auth-token sk-xxx --model grok-4.5` + `ccr grok profile switch relay`（或 TUI 切换）后：`[model.custom]` 被 CCR 接管、`[models].default = "custom"`、其余段落结构与值保留、首次切换前生成含原始条目记录的入口状态文件。
- [ ] 切官方 profile 或 `ccr grok profile off`：CCR 托管痕迹清除；入口原有 `[model.custom]` 恢复原内容、原无则删除；`models.default` 按决策 3 处理；**第三方 → 官方 → 第三方往返测试通过**。
- [ ] 删除当前激活 profile 默认被拒绝（提示先 off/switch）；`--force` 路径行为明确且测试覆盖。
- [ ] 运行时写入具备并发防护：外部并发修改 config.toml 时 CAS 检测冲突并安全重试/报错，不覆盖他方写入。
- [ ] 脱敏：日志/TUI 详情/CLI 输出（含 `--json`）不出现明文 key；base_url 展示剥离 userinfo/query；明文存在位置与披露矩阵一致。
- [ ] 携带旧版 `tui.toml`（5 tab 序列）启动：Grok tab 自动补齐且**用户自定义排序保留**（当前 load_or_default 会整体回落默认丢排序，必须修复）。
- [ ] `just lint-strict`、`just test` 通过；`cargo check --workspace` 无遗漏 match 臂；Claude/Codex 行为零回归。
- [ ] `docs/examples/grok-profiles.toml` 与 `docs/examples/grok-cli-config.toml` 不含真实凭据，分别覆盖官方/第三方 CCR profile 和 Grok `[model.custom]`/`[models].default` 运行时形态，并出现在中英文示例索引中。
- [ ] 使用本机已安装的 `grok`（规划时观测为 `0.2.112`）和临时 `GROK_HOME` 对生成的 Grok 配置执行 `grok inspect`；若命令需要登录或联网，至少完成离线配置发现并如实登记其余证据边界。

### 验收环境（2026-07-28 更新）

本机已安装 `grok 0.2.112 (9bbd559437)`，路径为 `C:\Users\lyh\.grok\bin\grok.exe`。除结构化 TOML 自动化断言外，最终验收增加临时 `GROK_HOME` 下的 `grok inspect` 配置发现；涉及真实账号、登录或网络调用的启动验收仍不得读取或改写用户现有 `~/.grok` 配置与凭据。

## 约束

- 遵循 CLAUDE.md：内部注释中文、公共 API 文档英文；脱敏、破坏性变更前备份、文件锁、原子写不可回退。
- 测试直接跑 cargo 时 `-- --test-threads=1`；Grok 测试必须用临时 `CCR_ROOT` + `GROK_HOME` fixture，禁止触碰真实用户配置。

## 后续候选（不在本树，另行决策）

- **Windows DACL 硬化**：AtomicWriter `secret` 在 Windows 落实 owner-only ACL——横切 ccr-core，影响全部平台的凭据文件，应独立立项。
- Grok sessions/usage 域接入；`env_key` array 形态；Codex 式 secret-store overlay。

## Notes

- 父任务持有需求集、任务地图与跨子任务验收；实现落在三个子任务。
- 执行顺序：core 先行，cli-surface 与 tui-tab 并行随后。
