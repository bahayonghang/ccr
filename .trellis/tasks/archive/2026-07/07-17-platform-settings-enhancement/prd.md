# 系统配置管理完善:settings.json / config.toml 分层与 raw 编辑

> 父任务:`.trellis/tasks/07-17-ccr-ui-config-mgmt-enhancement`。跨子任务契约(C1 锁内 CAS、C2 raw 仅 Local、C3 明文信任边界、C4 共享编辑器、C5 工程规范、C6 测试底线)以父任务 prd.md 为准,本文不重复展开。
>
> 本任务是三任务中的**先行任务**:负责交付共享编辑器组件与 ccr-core versioned 写入 API,另外两个子任务是其显式后继。

## Goal

把 ccr-ui 现有的"表单式部分字段"系统配置管理,补全为"分层可见 + 源文件可直编 + 校验有兜底"的完整配置管理:覆盖 `~/.claude/settings.json`、`~/.codex/config.toml`,并为后继任务沉淀 raw 编辑的通用前后端模式。

## 现状(2026-07-17 摸底,含审阅修正)

- **Claude**:`claude_get_settings` / `claude_update_settings`(`commands/claude_settings.rs`)经 active `ExecutionEnvironment` 读写 settings.json(可能指向 Local/WSL/SSH,见 `commands/claude.rs` `active_environment`),merge-patch 后经 `ccr_types::ClaudeSettings` 验证;前端 `ClaudeCodeSettingsView.vue` 表单式。无 raw 编辑,无分层展示。
- **Codex**:`codex_get_settings` / `codex_update_settings` 读写本机 `~/.codex/config.toml`,剔除 `mcp_servers` 和 `profiles`;前端 `CodexSettingsView.vue` 表单式。无 raw 编辑。
- **⚠️ 写路径既有缺陷(审阅确认)**:`LocalEnvironment::write_config`(`ccr-ui/src-tauri/src/platform/local.rs:88`)是裸 `tokio::fs::write`,**没有**备份/文件锁/原子替换。"复用既有写路径"不成立,新旧路径都要接 ccr-core guarded write。
- ccr-core 已有 `write_guarded`(锁 + 备份轮换 + 原子写),但仅保证单次写互斥,RMW 事务性由调用方负责且其路径锁不可重入(`crates/ccr-core/src/core/guarded_write.rs` 头注释)——这是父任务 C1 要求新增 versioned API 的原因。

## Research(外部规范,2026-07 检索)

- **Claude Code settings 分层**(https://code.claude.com/docs/en/settings):优先级 managed → CLI flags → `.claude/settings.local.json` → `.claude/settings.json`(项目)→ `~/.claude/settings.json`(user)。数组字段(permissions.allow、hooks 等)跨层合并去重而非覆盖。官方有 JSON Schema。
- **Codex config 分层**(https://developers.openai.com/codex/config-reference、/codex/config-advanced):`~/.codex/config.toml` 为主;profile overlay 文件 `~/.codex/<profile>.config.toml`;项目级 `.codex/config.toml` 仅 trusted project 且忽略安全键(`model_provider`、`openai_base_url`、`notify`、`otel` 等);`-c key=value` 优先级最高;`wire_api` 仅支持 `responses`。

## Requirements

### R1 ccr-core:versioned 写入 API(共享前置物之一)

- 按父任务 C1 在 ccr-core 实现锁内 CAS 写入:同一把路径锁内完成 读当前内容 → 比对内容哈希令牌 → 备份 → 原子写;令牌不匹配返回专用 `VersionedWriteOutcome::Conflict` 结果(不扩展已冻结的 `CcrError`;前端可识别区分"冲突"与"校验失败")。
- 目标文件不存在时的令牌语义(如空令牌 = 期望不存在)需在 design.md 冻结,覆盖"首次创建"场景。
- 附带单测:令牌匹配写入成功、不匹配拒写、并发竞争下无覆盖丢失、备份轮换不回归。

### R2 后端:raw 源文件读写命令

- 每平台两条命令(命名 design.md 冻结,如 `claude_get_settings_raw_text` / `claude_save_settings_raw_text`、`codex_get_config_raw_text` / `codex_save_config_raw_text`):
  - get:返回原始文本 + 内容哈希令牌 + 绝对路径;**仅 Local 环境可用**,active env 非 Local 时返回明确的 unsupported 错误(父任务 C2)。
  - save:入参原始文本 + 令牌。写前校验:语法(serde_json / toml,错误带行列号)→ 语义(Claude 反序列化 `ccr_types::ClaudeSettings`;Codex 反序列化现有 config 结构)→ 经 R1 API 锁内 CAS 落盘。校验失败与令牌冲突分别返回可区分错误。
- Codex raw 是**完整 config.toml**(含 mcp_servers/profiles);内容可能含敏感值,全流程遵循父任务 C3(直读磁盘、requestConfirm、不入日志、前端不持久化)。保存成功后失效 dashboard overview 缓存(参照 `codex_update_settings`)。

### R3 后端:配置层级探测(只读)

- 新增 list 命令,返回各层配置文件的存在性/绝对路径/大小/mtime(展示用途,mtime 仅展示不作令牌):
  - Claude:`~/.claude/settings.json`(user 层,本期唯一可编辑层)+ managed/project/local 层的说明性占位(标注"本工具本期不管理")。
  - Codex:`~/.codex/config.toml` + 枚举 `~/.codex/*.config.toml` profile overlay(只读列出)。

### R4 后端:修复既有裸写路径

- `LocalEnvironment::write_config` 从裸 `tokio::fs::write` 切换到 ccr-core guarded write(SameDir 备份策略;是否需要 versioned 由 design.md 决定——表单模式是 merge-patch 语义,至少要求锁 + 备份 + 原子写)。
- WSL/SSH 环境的 `write_config` 等价保障明确记录为已知限制(本期不做),在代码注释与 design.md 中留痕。

### R5 前端:Settings 页面增强(交付共享编辑器,父任务 C4)

- 交付共享编辑器组件(JSON/TOML/Markdown 三模式、行列号错误定位、未保存标记、明暗主题);技术选型(CodeMirror 6 vs textarea 方案)在 design.md 冻结并记录理由。
- `ClaudeCodeSettingsView` / `CodexSettingsView` 增加"表单 / 源文件"双模式切换:
  - 源文件 tab 仅 Local 环境启用;非 Local 禁用并展示原因(C2)。
  - 进入源文件模式先 requestConfirm 明文警示(C3);显示文件绝对路径。
  - 保存:后端校验错误按行列号内联展示;令牌冲突提示"文件已被外部修改",提供"重新加载"动作,不提供静默覆盖。
  - raw 保存成功后表单态强制重新拉取;表单保存成功后 raw 态(若打开)提示已过期。
- 新增"配置层级"说明面板(R3 数据):按优先级顺序展示各层与存在性,不可管理层标注只读。
- 表单模式字段缺口本期不逐一补齐(raw 模式即兜底);仅修已知明显缺失项,清单 design.md 冻结。

## Acceptance Criteria

- [ ] Local 环境下:Claude settings 页源文件模式可编辑 `~/.claude/settings.json` 全文并保存;非法 JSON / 不符合 ClaudeSettings 结构时拒写且错误含行列号。
- [ ] Local 环境下:Codex settings 页源文件模式可编辑完整 `~/.codex/config.toml`(含 mcp_servers/profiles)并保存;非法 TOML 拒写。
- [x] 切换 active 环境为非 Local 后,两个页面的源文件入口均禁用且展示原因文案。
- [x] 并发保护:get 后在外部修改文件,再 save 必须收到冲突错误(自动化测试覆盖,非仅手工)。
- [x] 保存生成备份、原子写、锁内 CAS —— 由 R1 单测 + 命令层单测覆盖:令牌冲突、非法 payload 拒写、备份生成、文件不存在分支。
- [x] `LocalEnvironment::write_config` 不再裸写(单测或代码审查确认经由 guarded write)。
- [x] 明文不入日志:单测断言错误消息不含文件内容片段(C6)。
- [x] 共享编辑器组件落地并被本任务两处页面消费;组件带最小渲染/交互测试。
- [x] 配置层级面板正确展示 user 层与 Codex profile overlay 文件存在性。
- [x] `bun run type-check`、`bun run lint`、`bun run test:i18n`、`bun run test:smoke -- tests/api-facade-boundary.smoke.test.ts`、`cargo check`(src-tauri)、相关 Rust 单测(`just test` 范围)全部通过。

## Out of Scope

- 项目级 `.claude/settings.json` / `settings.local.json`、`.codex/config.toml` 的编辑(ccr-ui 无"当前项目"上下文)。
- managed/enterprise 策略文件;WSL/SSH 环境的 raw 编辑与远程 guarded write。
- JSON Schema 驱动的自动补全;跨层合并结果(effective config)预览。

## Notes

- 复杂任务:`task.py start` 前必须补 `design.md`(versioned API 签名与冲突结果类型、raw 命令签名、双模式状态机、编辑器选型、表单缺失字段清单)与 `implement.md`。
- 完成后触发 rust-security-reviewer 复核(命中:配置写路径 + 可能含 credential 的字段)。
