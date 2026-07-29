# Claude profile 与官方账号切换互不影响加固

## Goal

父任务:修复 ccr 在 Claude 第三方 profile 与官方订阅账号(OAuth)之间切换时的互相干扰缺陷,对照 Claude Code 官方认证优先级规则逐项整改,达到"任意方向切换,互不影响、诊断真实(在 ccr 可观测边界内)"。

> 2026-07-29 修订:吸收 Codex 交叉审阅结论——修正官方行为过度归纳、增补 C10/C11、重定义 C8、明确诊断能力边界与 OS 范围。

## 背景与机理

**Claude Code 官方认证优先级**(https://code.claude.com/docs/en/authentication,已核对全文):

1. 云厂商 env(`CLAUDE_CODE_USE_BEDROCK/_VERTEX/_FOUNDRY`)
2. `ANTHROPIC_AUTH_TOKEN`(Bearer)
3. `ANTHROPIC_API_KEY`(X-Api-Key,交互模式需一次性批准,批准记录在 `~/.claude.json` 的 `customApiKeyResponses`)
4. `apiKeyHelper`(settings.json 脚本钩子)
5. `CLAUDE_CODE_OAUTH_TOKEN`
6. `/login` 的订阅 OAuth(Windows/Linux:`.credentials.json`;**macOS:Keychain**;受 `CLAUDE_CONFIG_DIR` 影响)

**已确认的事实**:`settings.json` `env` 段被热加载;残留任何高于第 6 级的凭据源都会按优先级**压制订阅 OAuth**(官方文档与 support 文章 12304248 确认)。

**注意不过度归纳**:"Auth conflict" 警告与 `/login`/`/logout` 是否可用属**版本相关行为**,不构成"任意 1–5 残留必然触发警告/禁用"的统一规则(早期 v1.0.11 时代确曾因检测到 env key 禁用 `/login`,issue #1582;后续版本行为多次调整)。规划与文案只承诺"压制"这一优先级事实,不承诺具体警告形态。

另有**官方优先级契约之外**的压制源:`~/.claude.json` 中残留的 `primaryApiKey` 会静默压制活跃 Max 订阅(open issue #80713,2026-07 已亲核属实)——按"issue 报告行为"对待,不标注为官方契约。

ccr 的双维度架构本身正确:Profile → `settings.json` env 段(`ClaudePlatform::apply_profile`,`crates/ccr-cli/src/platforms/claude.rs:315`);Auth → `.credentials.json`(`ClaudeAuthService`)。缺陷在实现层。

## 已确认缺陷清单(源码逐条核实,2026-07-29 二轮复核)

- **C1(P0)** auth 切换清理与诊断用字面 `resolve_profile_auth_mode`(`claude_auth_service.rs:267-272`、`:677`),写入用 `effective_auth_mode`(`platforms/claude.rs:325`)。字面 `subscription` 但含 `base_url+auth_token` 的 profile → apply 写入 token,`auth switch` 跳过清理 → 订阅静默失效且诊断误报正常。
- **C2(P0)** `auth switch`(`claude_auth_service.rs:279`)、`profile off`(`application/profile_off.rs:216`)、`clear`(`commands/lifecycle/clear.rs:134`)只 `clear_anthropic_vars()`,遗留 5 个非 Anthropic 托管键(`crates/ccr-types/src/claude_settings.rs:204-210`)。
- **C9(P0,C1 根因)** `apply_profile` 检测到冲突仅 warn(`platforms/claude.rs:326-336`),不回写纠正后的 auth_mode 到 profiles.toml。
- **C10(P0,二轮新增)** 账号切换后身份元数据错配:`save_current` 把 `oauthAccount` 存入快照(`claude_auth_service.rs:600-605`),但 `switch_account`(`:622-638`)只恢复 `.credentials.json`,快照中的 `oauth_account` 从未被读取;诊断 `build_current_info`(`:408-424`)的 UUID/email 又取自现存 `.claude.json`。A 切 B 后形成"B 凭据 + A 元数据",UI 可能继续显示 A。
- **C11(P1,二轮新增)** 托管键所有权语义缺失:`clear_anthropic_vars` 按前缀删除**所有** `ANTHROPIC_*`(`claude_settings.rs:220-222`),包括 ccr 从未写入的用户自有键(如 `ANTHROPIC_CUSTOM_HEADERS`、用户自设 `ANTHROPIC_API_KEY`)。与"保留非托管 env"目标及 doctor"用户自有来源只告警不删除"原则冲突,需显式所有权模型定夺。
- **C3(P1)** `switch_account` 无备份覆盖 `.credentials.json`(`:630-633`);活跃登录未存快照时永久丢失。
- **C4(P1)** OAuth token 明文落盘 `~/.ccr/platforms/claude/auth/*.json`,`write_atomic`(`:313-332`)无 secret 权限/锁/fsync,违反 `atomic-writer.md:113/:135`。
- **C5(P1)** settings.json 双写路径不等价且**互不互斥**:UI 走 `write_guarded_async` 路径派生锁 + `secret:false` + 同目录明文备份(`ccr-ui/src-tauri/src/platform/local.rs:89-100`);CLI 走固定 `claude_settings` 命名锁 + 直接 `AtomicWriter` + `secret:true` 无备份(`managers/settings.rs:165-186`)。锁域不同 → CLI/UI 并发 RMW 可互相丢更新;只统一 secret/backup 不解决丢更新。
- **C7(P1)** `CLAUDE_CONFIG_DIR`:`ClaudeAuthService` 只对 config dir 部分识别(`:162-167`,state_file 仍回落 home),`SettingsManager::with_default`(`managers/settings.rs:72-80`)与 UI `resolve_config_path`(`local.rs:127-144`)硬编码 `~/.claude/`,组件各写各的目录。本机 Claude Code 2.1.220 探针还确认自定义 config dir 会在目录内创建 `.claude.json`。
- **C8(P2,三轮重定义)** onboarding 写入 `ensure_onboarding_completed`(`platforms/claude.rs:88-119`)与 Tauri MCP user/local 写入(`claude_mcp_config.rs:181-212/:593-613`)都会对 `.claude.json` 做无版本 RMW,与 Claude Code 竞态。**修复机制注意**:ccr 侧文件锁**无法**消除外部竞态(Claude Code 不会获取 ccr 的锁;`guarded_write.rs:12-15` 亦明确 RMW 事务性由调用方负责)。当前结论是删除无必要的 onboarding 写入;合法 MCP 写入改用 CAS(`write_guarded_versioned`)+ 3 次重读重放 + 冲突失败,并声明残余风险。
- **C6(P2,二轮重构定义)** 诊断盲区,按来源性质分三类:
  - **官方契约内、可观测**:settings.json 的 `apiKeyHelper`;env 段与本进程环境的 `ANTHROPIC_AUTH_TOKEN`/`ANTHROPIC_API_KEY`/`CLAUDE_CODE_OAUTH_TOKEN`;云厂商 provider env(`CLAUDE_CODE_USE_BEDROCK/_VERTEX/_FOUNDRY`,优先级最高,此前遗漏)。
  - **状态记录(非独立凭据)**:`customApiKeyResponses`(API key 批准/拒绝记录)——作为解释性信息展示,不作为压制源计数。
  - **issue 报告行为(非官方契约)**:`~/.claude.json` 残留 `primaryApiKey` 压制订阅(open issue #80713)——检测并告警,标注依据等级。

## 诊断能力边界(承诺范围)

Claude Code 设置有五层优先级(managed > CLI 参数 > 项目 local > 项目 shared > user,官方 settings 文档),且 env 可来自任意启动 shell。ccr **不可能**观测:其他 shell 的环境、任意 cwd 的项目级 settings、外部进程命令行、全部 managed 动态来源。因此:

- 诊断输出定义为:**本 ccr 进程可见范围内**的有效/潜在凭据来源,按 `confirmed / potential / unobservable` 三级置信度呈现,并明确列出不可观测项。
- **不承诺**与任意 Claude Code 进程的 `/status` 严格相等;验收改为"对同环境启动的 Claude Code,confirmed 级结论一致"。

## 子任务地图(依赖序,前一个合入后再 start 下一个)

| # | 子任务 | 覆盖 | 优先级 |
|---|---|---|---|
| 1 | `07-29-claude-authmode-consistency` | C1 + C9 + C2 + C11 | P0 |
| 2 | `07-29-claude-config-dir-consistency` | C7(路径解析统一,先落地为后续写路径打底) | P1 |
| 3 | `07-29-claude-credentials-hardening` | C3 + C4 + C5(含统一 RMW/锁域)+ C10 | P1 |
| 4 | `07-29-claude-json-write-strategy` | C8(删除 onboarding 写入,保留并 CAS 加固 MCP 写入) | P2 |
| 5 | `07-29-claude-auth-doctor-spec` | C6 诊断整合 + 优先级速查表规范 | P2 |

各子任务落地时**同步更新自己触碰的持久化契约规范**(atomic-writer / backend-guidelines 相关条款),不全部延后到 #5;#5 只负责诊断整合与跨任务速查表。

## Start Gate(所有子任务适用)

- #1–#5 均按复杂任务处理;`design.md`、`implement.md` 与真实 `implement.jsonl`/`check.jsonl` 已在 2026-07-29 规划收敛时补齐并通过 `task.py validate`。最新规划摘要获用户再次批准后只 start #1,后续按依赖逐个 start。
- 已定夺的设计决策(写入 design.md 时不得推翻,除非记录理由):
  - #1:apply_profile 自愈回写 profiles.toml **失败时在修改 runtime settings 之前阻断 apply**(先纠正持久层,再写运行时)。
  - #3:当前登录未存快照时 **拒绝切换并提示先 `ccr claude auth save`**(不采用自动匿名备份)。
  - #4:**停止 onboarding 写入** `.claude.json`;现有 MCP user/local 写入必须保留并改成 CAS + 有限重试/冲突失败,不得用锁承诺"无丢字段"。

## Acceptance Criteria(跨子任务收口)

- [ ] 第三方 → 官方:任意形态 profile 生效后 `ccr claude auth switch`,settings.json 不残留任何**托管**env 键(托管定义按 #1 定夺的所有权模型),同环境启动 Claude Code 走订阅登录。
- [ ] 官方 → 第三方:`profile use` 后请求走中转站;切回官方后 `.credentials.json` 完好,无需重新 `claude login`。
- [ ] 双账号回归:官方账号 A 存快照 → 切 B → ccr 各端(CLI/TUI/UI)显示的账号身份与 `.credentials.json` 实际凭据一致(C10 消除)。
- [ ] 诊断按 confirmed/potential/unobservable 分级输出;confirmed 级与同环境 Claude Code 行为一致。
- [ ] settings.json 用户自有配置(hooks/permissions/非托管 env,按新所有权定义)在所有切换路径下保留。
- [ ] 不违反 `backend-guidelines.md:118-210` auth_mode 契约与 `atomic-writer.md` 锁序/备份/secret 条款;触碰的契约条款随子任务同步更新。
- [ ] `just lint-strict` + `just test` 通过;UI 涉及处过 `just frontend-check-quick`。

## 范围与约束

- **OS 范围**:auth 快照/切换功能仅覆盖 Windows/Linux 的 `.credentials.json` 路径;**macOS Keychain 明确 out-of-scope**(ccr 不读写 Keychain,macOS 下 auth 子命令应显式报不支持而非静默失效——此行为核实/补齐归入 #3)。
- 不与 `07-29-profiles-*` 任务族(纯前端、不改后端)冲突;UI 依赖点保持 `effective_auth_mode` 语义(`07-29-profiles-claude-page/design.md:10` 的 diff 行取值需澄清为 effective)。
- ccr 定位是"文件搬运工":不调用 `claude /login`、不做 token 刷新,本次不改变定位。
- 保持 secrets 掩码、备份先于破坏性变更、文件锁(仅限 ccr 进程间互斥语义)、原子写四条红线。

## 研究来源

- 官方文档:authentication(优先级/凭据存储/macOS Keychain)、settings(五层优先级、env 热加载)、llm-gateway-connect;support.claude.com 文章 12304248。
- anthropics/claude-code issues:#1084、#1582、#11587、#16238(Auth conflict 与 /login 可用性的版本演化);**#80713(open,`primaryApiKey` 压制订阅,已核实)**。
- 归档任务:06-19-claude-third-party-model-authmode、06-26-claude-third-party-profile-switch-analysis、07-03-arch-claude-settings。
- 二轮交叉审阅:Codex 报告(2026-07-29),8 条意见经逐条源码/文档核验后全部或部分采纳。
