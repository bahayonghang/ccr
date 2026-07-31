# 认证盲区诊断与 Trellis 规范补齐

## Goal

修复 C6(二轮重构定义)并补齐规范缺口:让 ccr 的诊断面覆盖**本进程可观测范围内**所有会压制订阅 OAuth 的凭据源,按置信度分级输出;沉淀 Claude Code 认证优先级速查表规范。父任务序列第 5 号(最后做诊断整合;各子任务自己触碰的持久化契约由其自行同步更新,不汇总到本任务)。

## 问题(二轮重构后的清单)

Claude Code 认证优先级中高于订阅 OAuth 的来源,ccr 目前只管理 `ANTHROPIC_*` env 键,其余零检测。按来源性质分三类(混为一谈会导致需求不闭合——一轮 PRD 的教训):

**A. 官方契约内、本进程可观测(检测 + 计入压制源)**
- 云厂商 provider env:`CLAUDE_CODE_USE_BEDROCK` / `_VERTEX` / `_FOUNDRY`(**优先级最高,一轮遗漏**)
- `settings.json` 顶层 `apiKeyHelper`(随 `ClaudeSettings` flatten `other` 保留,全仓零处理)
- settings.json `env` 段与本进程环境中的:`ANTHROPIC_AUTH_TOKEN`、`ANTHROPIC_API_KEY`、`CLAUDE_CODE_OAUTH_TOKEN`

**B. 状态记录,非独立凭据(解释性展示,不计入压制源)**
- 共享路径解析命中的 state_file `.claude.json` 的 `customApiKeyResponses`:API key 的批准/拒绝记录(官方文档:批准后 `ANTHROPIC_API_KEY` 才接管)。用于解释"为什么 env key 生效/不生效",本身不是凭据。

**C. issue 报告行为,非官方契约(检测 + 告警,标注依据等级)**
- state_file 残留 `primaryApiKey` 静默压制活跃订阅:open issue anthropics/claude-code#80713(2026-07-29 亲核属实,Windows 11 复现)。不得标注为"官方已确认"。

**规范缺口**:`.trellis/spec` 对 `.credentials.json`、官方 OAuth 流程零覆盖(对照 Grok 侧红线 `grok-profile-runtime.md:59-68`);无认证优先级速查表可供后续任务引用。

## 诊断能力边界(与父任务一致,写进输出契约)

ccr 不可能观测:其他 shell 的 env、任意 cwd 的项目级 settings(`.claude/settings.json` / `settings.local.json`)、外部进程 CLI 参数、managed settings 动态来源。诊断输出必须:

- 限定为"本 ccr 进程可见的有效/潜在来源";
- 每项标注置信度 `confirmed`(读到了文件/env 实值)/ `potential`(存在但无法确认 Claude Code 会取用,如批准状态未知的 API key)/ `unobservable`(明确列出检测不到的层);
- **不承诺**与任意 Claude Code 进程 `/status` 严格相等。

## Requirements

- R1:doctor 与 `get_runtime_summary` 增加"订阅压制源检测",覆盖 **A 类全部三组**(含云厂商 env)与 **C 类** `primaryApiKey`;输出按官方优先级排序的"当前推定生效凭据源"+ 置信度;B 类 `customApiKeyResponses` 作为解释信息附带展示。
- R2:检测**只读**:不自动删除 `apiKeyHelper`、云厂商 env、`primaryApiKey` 等用户自有配置,只告警并附官方文档链接与手动处理建议;与 `07-29-claude-authmode-consistency` 的所有权模型衔接——清单外用户自有 `ANTHROPIC_*` 键同样进入告警而非删除。
- R3:`auth switch` / `profile off` 完成时,若检测到 A/C 类压制源残留,在结果输出中显式警告(CLI/TUI/UI 三端一致文案,区分"ccr 托管、已清理/可清理"与"用户自有、需手动处理")。
- R4:新增 `.trellis/spec` 条款:
  - Claude Code 认证优先级速查表(六级 + gateway 例外 + `primaryApiKey` issue 行为附注,标注依据等级与核实日期),作为后续 Claude 集成任务的判定依据;
  - `.credentials.json` 边界条款(谁可读写、secret、备份/快照要求)——**如第 3 号任务已立,此处只补链不重复**;
  - 诊断输出的置信度分级契约。
- R5:UI(ClaudeAuthView / 运行时概览)透出同一结论;文案避免"保证与 /status 一致"类表述。
- R6:规范更新走 `trellis-update-spec` 流程,挂对应 index。

## Acceptance Criteria

- [ ] 场景测试逐项覆盖 A 类:`apiKeyHelper`、`CLAUDE_CODE_USE_BEDROCK`、env `ANTHROPIC_AUTH_TOKEN`/`ANTHROPIC_API_KEY`/`CLAUDE_CODE_OAUTH_TOKEN`——doctor 报出来源与 confirmed/potential 等级;只有 confirmed 才表述为当前压制,且不修改用户配置。
- [ ] C 类:构造 `.claude.json` 含 `primaryApiKey`,doctor 告警并标注"issue 报告行为(#80713)"依据等级。
- [ ] B 类:`customApiKeyResponses` 仅作解释展示,不计入压制源计数。
- [ ] 输出含 unobservable 层的明确列举(项目级 settings、其他 shell env、managed)。
- [ ] `auth switch` 在压制源存在时三端输出警告,文案区分托管/自有。
- [ ] 速查表与置信度契约条款合入并出现在相应 index.md。
- [ ] `just lint-strict` + `just test` + `just frontend-check-quick` 通过。

## Notes

- 依赖:第 1 号(`effective_auth_mode` 修正 + 所有权模型)先合入;建议第 3/4 号后做,以引用其落定的边界条款。
- 检测 shell env 只覆盖 ccr 进程可见环境,文档需注明 Claude Code 的启动 shell 可能不同(这正是 potential/unobservable 分级存在的原因)。
- Planning status:`design.md` 已定义共享检测数据流、跨端 action outcome 与置信度模型,`implement.md`/JSONL 已就绪;作为最后一个诊断整合子任务 start。
