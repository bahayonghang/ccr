# 系统提示词管理:CLAUDE.md / AGENTS.md / GEMINI.md 子页面

> 父任务:`.trellis/tasks/07-17-ccr-ui-config-mgmt-enhancement`。跨子任务契约(C1 锁内 CAS、C2 raw 仅 Local、C3 明文信任边界、C4 共享编辑器、C5 工程规范、C6 测试底线)以父任务 prd.md 为准。
>
> **显式后继**:前端编辑面依赖 platform-settings-enhancement 交付的共享编辑器组件;写入路径依赖其 ccr-core versioned 写入 API。后端命令可并行开发,前置物就绪前不满足 `task.py start` 条件。

## Goal

在平台子页面下新增"系统提示词"(System Prompts / Memory)管理页,让用户不离开 ccr-ui 就能查看、编辑各 AI CLI 的**全局(home 目录)** memory/instructions 文件。项目级文件明确不在本期范围(task.json 描述已同步)。

## 现状(2026-07-17 摸底)

- 前端无任何 CLAUDE.md / AGENTS.md 管理页面;后端唯一触点是 `sync.rs` 把 `~/.claude/CLAUDE.md` 与 `~/.codex/AGENTS.md` 注册为同步资产(id: `claude-memory` / `codex-agents`),无读写编辑命令。
- `claude_list_prompts` / `claude_update_prompts` 是 PromptsManager 的 preset 管理,与 memory 文件无关,不得混淆或复用命名。
- 子页导航:`ClaudeCodeView.vue` 等各平台 View 持有 tab 条;路由 `ccr-ui/src/router/index.ts`,子页 meta `{ depth: 2, group: '<platform>' }`。
- **⚠️ 路径解析警示(审阅补充)**:`ExecutionEnvironment` 的 `resolve_config_path` 将 opencode 映射到 `~/.opencode`,与 `commands/opencode.rs` 实际使用的 `~/.config/opencode/` **不一致**;Antigravity 在本仓库的配置根是 `~/.gemini/antigravity-cli/`(非 `~/.gemini/`)。本任务路径解析必须复用各平台命令模块的现成函数(如 `opencode_config_dir()`),**不走** ExecutionEnvironment 的路径映射。

## Research(外部规范,2026-07 检索)

- **Claude Code memory 层级**(https://code.claude.com/docs/en/memory):managed → project `./CLAUDE.md` → `.claude/rules/` → user `~/.claude/CLAUDE.md` → `~/.claude/rules/` → `CLAUDE.local.md` → auto memory;支持 `@path` import(深度 5);最佳实践单文件 < 200 行。
- **AGENTS.md 开放标准**(https://agents.md/):纯 Markdown,Linux Foundation Agentic AI Foundation 管理;Codex、Gemini CLI、OpenCode 等 20+ 工具读取。
- **Codex**(https://developers.openai.com/codex/guides/agents-md):全局 `~/.codex/AGENTS.md`;项目内逐层 `AGENTS.override.md` → `AGENTS.md` 拼接,上限 `project_doc_max_bytes` 32 KiB。
- **Gemini CLI 上游**:默认 `~/.gemini/GEMINI.md`;但本仓库对接的是 **Antigravity CLI**(配置根 `~/.gemini/antigravity-cli/`),它是否消费 GEMINI.md、路径在哪一层,**外部资料不足以裁决,必须在 design.md 阶段以本机实际安装核实**。
- **OpenCode**:遵循 AGENTS.md 标准,全局文件按官方文档在 `~/.config/opencode/AGENTS.md`(与本仓库 `opencode_config_dir()` 一致);design.md 阶段以本机安装复核一次。

## 平台支持分级(审阅修订:验收必须可判定)

| 平台 | 目标文件 | 本期支持级别 |
| --- | --- | --- |
| Claude Code | `~/.claude/CLAUDE.md` | **P0 可编辑**(验收硬性);`~/.claude/rules/*.md` 只读列出 |
| Codex | `~/.codex/AGENTS.md` | **P0 可编辑**(验收硬性) |
| OpenCode | `opencode_config_dir()/AGENTS.md` | **P1 可编辑**,design.md 复核路径后冻结;复核不通过则降级为"本期不支持" |
| Antigravity | 候选 `~/.gemini/GEMINI.md` 或 `~/.gemini/antigravity-cli/GEMINI.md` | **待核实**:design.md 必须给出三选一结论 —— 可编辑 / 只读展示 / 本期不支持(不显示入口)。核实方法:查 Antigravity CLI 文档与本机实际行为 |

规则:任何平台若结论为"本期不支持",**不显示入口**(不做假 tab),并在 design.md 记录原因;验收只针对冻结后为"可编辑/只读"的平台。

## Requirements

### R1 后端:memory 文件读写命令

- 新增 Tauri 命令组(建议 `commands/system_prompts.rs`),按上表冻结后的平台清单提供:
  - list:每平台返回文件描述(逻辑名、绝对路径、存在性、大小、mtime——mtime 仅展示)。
  - get:返回原文 + 内容哈希令牌(C1)。
  - save:原文 + 令牌,经 ccr-core versioned 写入 API 锁内 CAS 落盘;memory 文件同样可能被外部编辑器并发修改,冲突保护不豁免。
  - create:不存在时从空模板创建(create 的令牌语义 = 期望不存在,复用 versioned API 的首建分支)。
- Markdown 无语法校验需求,但保留大小提醒阈值(如 > 64 KiB 警示,Codex 上限 32 KiB 的提示信息);不做硬拒绝。
- 仅 Local 环境可用(C2):active env 非 Local 时命令返回 unsupported,前端入口禁用并展示原因。
- memory 文件本身非 secret 类,但内容可能含用户敏感偏好——遵循 C3 的日志隔离(内容不入日志),无需明文警示确认(与 settings/profiles 区分,design.md 可复核该裁量)。

### R2 前端:平台"系统提示词"子页面

- 冻结清单内的平台子页导航新增 tab("System Prompts" / “系统提示词”),路由 `/<platform>/system-prompts`,meta `{ depth: 2, group: '<platform>' }`。
- 页面内容:
  - 文件卡片列表(逻辑名、路径、存在状态、大小、最后修改时间)。
  - 编辑:共享编辑器组件 Markdown 模式(C4);未保存标记;令牌冲突提示"文件已被外部修改"+ 重新加载;保存 toast。
  - 未创建文件提供"创建"入口,创建后立即可编辑。
- Claude Code 页展示 memory 层级简表(managed → project → user → local),明确"此处编辑的是 user 级"。

### R3 规范

- API 包装放 `src/api/domains/systemPrompts.ts`;i18n 双语;确认交互契约(离开未保存需确认)。

## Acceptance Criteria

- [x] design.md 已冻结四平台支持级别(含 Antigravity 核实结论与依据、OpenCode 路径复核),验收范围以冻结清单为准。
- [ ] Claude Code 与 Codex(P0):子页入口存在,能读取并保存 `~/.claude/CLAUDE.md` 与 `~/.codex/AGENTS.md`,保存后磁盘一致且生成备份。
- [ ] 冻结为"可编辑"的其余平台:同等读写验收;冻结为"只读/不支持"的平台:行为与冻结结论一致(不支持者无入口)。
- [ ] 文件不存在显示"未创建"并可一键创建;创建后立即可编辑。
- [x] 自动化测试(C6):get/save 令牌冲突拒写、create 首建分支、文件不存在分支、备份生成、错误消息不含文件内容片段;新路由可解析 smoke。
- [x] 非 Local 环境下入口禁用并展示原因。
- [x] `bun run type-check`、`bun run lint`、`bun run test:i18n`、`bun run test:smoke -- tests/api-facade-boundary.smoke.test.ts`、`cargo check`(src-tauri)+ 新增 Rust 单测通过。
- [ ] 明暗主题下页面可读性符合既有设计语言。

## Out of Scope

- 项目级(仓库内)CLAUDE.md / AGENTS.md 的发现与编辑(task.json 描述已同步排除)。
- `@path` import 解析/预览、`.claude/rules/` 的编辑、managed policy 文件。
- 版本历史/回滚 UI(备份由写入机制保证)。
- WSL/SSH 环境的 memory 文件编辑。

## Notes

- 复杂任务:`task.py start` 前必须补 `design.md`(平台清单冻结 + 路径核实证据、命令签名、DTO、路由与组件结构)与 `implement.md`(首项 checklist:共享编辑器与 versioned API 可消费)。
- Antigravity 核实建议:本机检查 `~/.gemini/` 与 `~/.gemini/antigravity-cli/` 下实际文件 + Antigravity CLI 官方文档;必要时用 `trellis-research` 留档到本任务 `research/`。
