# Research: CLI 与 Tauri 任务树独立规划复核

- Query: 独立复核父任务和 11 个子任务的验收可追溯性、依赖图、20 组问题及延后项覆盖、规范兼容性和源码引用。重点核对多文件结果、rename、OAuth 无备份、事件顺序、共享编辑器、Auth 范围和控制并发。
- Scope: internal；只读审阅规划与现有源码。唯一写入为本文件。
- Date: 2026-09-28
- Active task: `.trellis/tasks/09-28-cli-tauri-architecture`

## Findings

### 结论

任务划分和依赖方向成立。11 个子任务共 39 项 AC，均存在对应的设计行与实施步骤；父任务 11 项 AC 分别承接对应子任务。20 组审查问题和 D01–D05 均有责任归属或明确延后边界。

本次发现 6 项规划修订事项：1 项 P1、3 项 P2、2 项 P3。主要问题为 T08 回滚说明违反无备份契约，以及 T05/T07/T11 的部分关键要求缺少精确测试断言。以下事项应在规划定稿前修订。本报告不表示实现已通过验收，也不授权启动子任务。

读取过程中主会话继续完善规划。已看到 T05 `design.md:13` 增加 `backup:None`；该新增机制已计入本报告，未将旧版缺失机制重复列为问题。主会话已处理的 validate 引用范围修正也未列为新问题。

### PV-01 · P1 · T08 回滚步骤要求保存前镜像，与 Grok 无备份契约冲突

- 发现位置：`.trellis/tasks/09-28-settings-lossless-capabilities/design.md:13` 明确保留 Grok 无备份，但同文件 `:30` 要求“发布前保存用户配置前镜像”；`implement.md` 的回滚段重复该要求。
- 规范：`.trellis/spec/ccr-ui/frontend/grok-settings-contracts.md:124-126` 要求 Grok raw save 使用 `BackupPolicy::None` 并显示无备份提示；`:141-142` 也要求 invalid/stale 保存不产生备份。
- 影响：通用回滚步骤可以被执行为对无备份域自动复制敏感原文。该冲突会改变用户数据的存储范围。
- 修订：明确只回退代码；禁止对无备份域自动生成前镜像或把原文写入发布/研究产物。其他配置仅使用所属领域已经批准的备份策略。同步 design、implement 和用于生成计划的研究数据。
- 验收：T08 的 Grok fixture 覆盖 typed/raw 保存及失败路径，断言不出现新增备份；保留现有 no-backup notice。

### PV-02 · P2 · T07 AC2 的三个时序条款未落实为可区分旧缺陷的测试

- 发现位置：`.trellis/tasks/09-28-command-workbench-lifecycle/prd.md:35` 要求终态先于 start 响应、旧 job 不覆盖新 job、提交中禁止重复启动；`design.md:12` 已给出统一 reducer 和独立 pending 的机制，但测试列仅写重复事件、序号间隙、snapshot/event 倒序和过期 job。实施测试步骤相同。
- 源码：`ccr-ui/src/features/commands/useCommandsPage.ts:160-168` 在 start Promise resolve 后直接覆盖 snapshot；`:139-142` 的执行约束只根据已存在 snapshot 计算。这些路径需要控制 start 响应和点击顺序，普通 snapshot/event 测试不能单独证明修复。
- 修订：补 3 项明确测试：延迟 start response，先发送该 job 的终态，再 resolve queued snapshot；新 job 生效后发送旧 job event/response；start Promise 未 resolve 时连续点击，两次交互只产生 1 次 start。每项同时断言展示状态、history 次数和控制能力。
- 边界：现有设计方向足够；该项针对验证完整性，无需新增任务状态框架。

### PV-03 · P2 · T11 R1 的 get/status 和逐命令族控制矩阵未进入明确验收

- 发现位置：`.trellis/tasks/09-28-desktop-control-oauth-lifecycle/prd.md:28` 覆盖 get/status/cancel，但 `:35` 的 AC1 仅直接要求 cancel，并以“命令、install、usage 或 OAuth”描述命令族。`design.md:11` 的测试仅明确观察 cancel token。
- 源码：`ccr-ui/src-tauri/src/commands/runtime_policy.rs:41-61` 把 permit 绑定到 handler future；后台 start 返回与后台 owner 完成是不同事件。`:69-81` 决定共享 permit 排队方式。registry 中命令任务控制、usage 控制、install 控制和 OAuth 控制的 risk/模块不同，单一 cancel 用例无法证明所有受影响入口。
- 修订：列出实际命令 ID、资源、start/control 类别、当前并发策略和目标策略。逐个受影响命令族验证现有 get/status/cancel 在 barrier 释放前能到达 owner；没有该操作的命令族标记不适用。
- 修订：明确后台 start 返回后资源 admission 仍由 owner 持有；同资源二次 start 被拒绝或等待；控制调用不需要该执行 permit。保留 ACL/确认断言，不以全部 Parallel 通过测试。
- 边界：get/status 中已为 Parallel 的命令应作为保留行为回归，不能据此宣称这些命令当前已经阻塞。

### PV-04 · P2 · T05 已补 OAuth backup:None，但缺少对应文件系统断言

- 发现位置：`.trellis/tasks/09-28-safe-persistence-backups/design.md:13` 已明确 `backup:None`，禁止备份 code_verifier/state；`prd.md:37` 和设计测试列仍只直接验证权限失败、不发布新文件及旧字节不变。`prd.md:38` 的可见输出 sentinel 不验证备份目录。
- 当前行为：`ccr-ui/src-tauri/src/commands/codex_auth.rs:508-531` 写入或删除 pending；该调用未创建备份。迁入 guarded writer 时应保留该边界。T05 同时改造备份唯一性，容易把备份启用域与无备份域混用。
- 修订：AC3 和对应测试明确 `secret:true`、`BackupPolicy::None`；创建、替换、取消、过期清理以及权限失败后，目录中不得出现 verifier/state 的备份或历史副本。使用合成 sentinel 检查目标目录及配置的备份目录。
- 已充分覆盖：旧/新备份混合发现、恢复、轮换在 `prd.md:36` 和 `design.md:11-12` 已有机制与测试，不需要再创建兼容迁移任务。

### PV-05 · P3 · T04 的现存 validator 路径不准确

- 发现位置：`.trellis/tasks/09-28-cli-diagnostics-contract/prd.md:15` 把 `crates/ccr-config/src/validators/` 列为覆盖路径。该目录不存在。
- 实际路径：`crates/ccr-config/src/managers/config_validator.rs`。
- 修订：按当前 owner 修正范围和研究生成数据。若未来确需新目录，必须标明为拟新增结构并说明必要性；当前任务没有该拆分设计。

### PV-06 · P3 · A10 对 status 阻塞的概括超出现有引用所证明的范围

- 发现位置：父审查 `research/architecture-audit.md:81` 写“无关长操作可阻止 cancel/status 到达真实 owner”。引用说明了 ProcessExecution singleton 和 permit 获取，未逐一证明 status 的策略。
- 反例：`ccr-ui/src-tauri/src/commands/handler_registry.rs:178-202` 将 get 类操作识别为 ReadOnly；`:229-231` 为 ReadOnly 使用 Parallel；`:840` 注册 `get_ccr_command_job_status`。该 status 不使用 ProcessExecution singleton。
- 修订：将已证缺陷限定到具体 cancel 或受 module-exclusive 阻塞的控制命令，并写出实际 ID。get/status 的及时可达可以保留为 T11 的防回归要求。未逐 ID 跑过的原生阻塞行为继续标为未验证。
- 边界：该修订不否定控制并发重构的必要性，只修正事实强度。

### 逐任务验收覆盖

| 子任务 | AC 数量 | 机制和验证核对 | 需要修订 |
| --- | ---: | --- | --- |
| T01 | 3 | 读取前 resource lock；纯查询与显式 repair 分离；双进程和非法 patch fixture | 无新增项 |
| T02 | 5 | prepare/execute/outcome；跨文件版本保护补偿；成功副作用；三端结果；rename 三阶段故障 | 无新增项 |
| T03 | 3 | UI→typed domain→真实 handler/service；CRUD 严格 patch；旧 payload/生成物/ACL | 无新增项 |
| T04 | 3 | binary exit 断言；platform/auth-mode 状态表；只读能力矩阵 | PV-05 范围路径 |
| T05 | 4 | 唯一备份；旧/新恢复与轮换；发布前权限；secret sentinel | PV-04 无备份断言 |
| T06 | 4 | admission/token 原子记录；单 owner 终态；deadline/cleanup/reap/reader join；输出上限 | 无新增项 |
| T07 | 3 | shell owner；route remount；单调 reducer；迟到 listener dispose | PV-02 精确时序测试 |
| T08 | 3 | dirty patch；managed/unknown enum；raw callback；共享 editor composite | PV-01 无备份回滚冲突 |
| T09 | 3 | Grok BaseAuth error/pending；Settings baseline/draft/environment；同实例语言切换 | 无新增项 |
| T10 | 4 | root+Tauri aggregate；行为证据账；规范漂移；保留正式基线失败 | 无新增阻断项 |
| T11 | 4 | resource/control 策略；OAuth listener/pending 发布；取消/deadline；协议与权限保留 | PV-03；PV-06 事实范围 |

重点要求的复核结果：

- T02 `design.md:11-15` 已覆盖 runtime/profiles/registry 写点故障、外部更新导致拒绝补偿、applied_with_warning/recovery_required 和 rename 的 save-new/delete-old/apply。`prd.md:36-40` 覆盖三端后果比较及 current/default 不悬空。不能把跨文件方案描述为操作系统级事务，当前设计未作该承诺。
- T08 `design.md:13` 要求先把领域中性 composite 移入明确共享层，再供 Base 使用，且禁止全域 import 豁免。该安排符合 `.trellis/spec/ccr-ui/frontend/layering-contracts.md:42-55` 和 `:74-86`。无需扩大 feature 跨域依赖。
- T09 `prd.md:28` 和 `design.md:11` 明确仅治理 Grok 使用的 BaseAuth，不强制迁移 Claude/Codex 特殊认证页。Settings `design.md:12` 明确 baseline、dirty、环境身份、冻结旧会话写入和保留草稿；未把敏感编辑原文移入持久 storage。
- T06 保留外部 llmusage CLI 和 `ccr-usage` SQL owner；未引入上游 llmusage Rust 依赖。

### 20 组问题与延后项覆盖

| 问题 | 责任任务 / 主要验收 |
| --- | --- |
| A01 配置页旧 handler 断路 | T03 AC1/AC3；T02 AC4 |
| A02 RMW 锁不一致 | T01 AC1/AC3；T03 AC2 |
| A03 off/apply 和跨文件部分结果 | T02 AC1/AC3 |
| A04 成功副作用及 rename | T02 AC2/AC5 |
| A05 诊断退出与 auth-mode | T04 AC1/AC2 |
| A06 隐式平台及查询写入 | T01 AC2；T04 AC3 |
| A07 备份碰撞 | T05 AC1/AC2 |
| A08 usage 早取消和终态覆盖 | T06 AC1/AC2 |
| A09 stream deadline/cleanup | T06 AC3/AC4 |
| A10 控制 permit | T11 AC1；事实范围见 PV-06 |
| A11 OAuth secret writer | T05 AC3/AC4 |
| A12 OAuth listener/网络/pending | T11 AC2/AC3；T05 AC3 |
| A13 Commands 路由及事件顺序 | T07 AC1–AC3 |
| A14 Settings 无损映射 | T08 AC1 |
| A15 Settings 能力/锁/原文入口 | T08 AC2/AC3 |
| A16 BaseAuth 失败状态 | T09 AC1 |
| A17 Settings 草稿 | T09 AC2 |
| A18 语言订阅 | T09 AC3 |
| A19 完整 gate 和行为测试 | T10 AC1/AC2；各 owner 落地行为回归 |
| A20 规范事实漂移 | T10 AC3 |

| 延后项 | 当前处理边界 |
| --- | --- |
| D01 legacy Gemini/Droid | T04 design R3 盘点能力与兼容风险，禁止无证增加调用或删除公共 API |
| D02 未测性能 | 父 PRD 明确延后；容量记录可以附 T06，不将未经测量的优化计入当前验收 |
| D03 环境可达性 | 父 design 的环境行及 implement 最终验收要求 Local/WSL/SSH 支持矩阵；T10 集成收敛 |
| D04 OpenCode notify | T08 AC1 明确未编辑时请求不含 autoupdate；完整后端支持链仍需实施 fixture 证据 |
| D05 全面视觉/可访问性 | 全站重做在父非目标内；触及表面的 Web/native/视觉验证保留在实施验收 |

### 依赖图和元数据

逐个解析父任务及 11 个子任务的 `task.json`。父 children 列表有 11 个唯一节点，child.parent 均指向当前父任务，所有 meta.depends_on 均指向本任务树中的节点，无环。层次与文档一致：

1. T01、T05、T06、T08。
2. T02 依赖 T01/T05；T11 依赖 T05/T06。
3. T03、T04 依赖 T01/T02；T07 依赖 T11。
4. T09 依赖 T03/T08。
5. T10 依赖其余全部 10 个子任务。

T10 design 的 fixture 文案写 T01–T09；最终集成应同时包含 T11。父 AC11 和 T10 的显式依赖已包含 T11，因此未单独提升为阻断事项，定稿时可同步文字。

所有新任务状态为 planning；父 `meta.implementation_authorized` 为 false。没有调整既有 Insights 任务的状态。

### 源码引用与文件清单

- 检查 37 份规划/审查 Markdown 中 64 个去重后的完整 `path:line[-line]` 源码引用：文件均存在，行号范围均有效。简写的后续 `:line` 未计入该 64 项计数。
- 对 profile/off/rename、配置 handler、usage admission/cancel/deadline、OAuth listener/storage、命令页晚响应、Settings mapper、BaseAuth 和语言订阅读取实际代码核对语义。范围存在不等于断言已得到验证；PV-06 单独记录语义过度概括。
- 子任务 relatedFiles 指向的现存路径未发现缺失。PV-05 来自 PRD 内单独列出的目录，未进入上述完整源码引用计数。

| 文件或范围 | 用途 |
| --- | --- |
| `.trellis/tasks/09-28-cli-tauri-architecture/{task.json,prd.md,design.md,implement.md}` | 父范围、依赖、矩阵和最终验收 |
| 父任务 `research/architecture-audit.md`、`research/task-designs.json` | 问题集合、原始记录合并关系和任务生成依据 |
| 父 children 指向的 11 个目录的 task/PRD/design/implement | 39 个子 AC、机制、测试和元数据 |
| `.trellis/workflow.md`、相关 CLI/config/core/TUI/Codex specs | 角色、流程、共享持久化与兼容约束 |
| `ccr-ui/AGENTS.md`、`ccr-ui/code_map.md` | 前后端模块导航与验收边界 |
| `.trellis/spec/ccr-ui/frontend/{grok-settings-contracts,layering-contracts}.md` | 无备份、Local-only、共享层和跨 feature 依赖规则 |
| `.trellis/spec/ccr/backend/desktop-command-policy.md` | ProcessGateway、取消清理、流输出和 OAuth 端点边界 |
| `ccr-ui/src-tauri/src/commands/{handler_registry,runtime_policy,codex_auth,usage}.rs` | 控制策略和后台状态事实 |
| `ccr-ui/src/features/commands/useCommandsPage.ts`、平台 BaseAuth/BaseSettings、Settings mappers | 前端失败反例对应路径 |

外部参考：本次为仓库内规划复核，没有查询外部文档或推定最新依赖行为。所用技术版本以仓库当前 manifests 和现存契约为边界。

## Caveats / Not Found

- 没有运行 Rust/Tauri 编译或测试、原生桌面、视觉验收、真实 OAuth、真实用户配置、跨进程故障注入或性能测量。父审查中已有通过/失败结果属于该报告的既有证据，本次没有独立重跑。
- 机械引用检查证明文件和行号可解析，不替代全部语义或运行时验证。源码可证、故障推导、已复现和未验证状态仍需分开。
- 没有修改产品代码、规范、父子任务规划、task.json、任务状态或 Git。未读取实施/检查角色的 JSONL。
- 这是一轮规划完整性检查。修订后的事项需要重新读取对应文档；产品验收只能在实现和要求的检查完成后标记通过。

## 第二轮：PV-01–PV-06 修订闭合复核

- Date: 2026-09-28
- Query: 核对主会话修订后的 task-designs.json、生成规划和 control-command-matrix.md，逐项判断首轮六项问题是否闭合。
- Scope: 仅复核六项修改及新增控制矩阵。保留首轮发现作为历史记录；本节给出当前状态。

### 当前结论

PV-01–PV-06 **6/6 在规划层闭合，剩余规划问题 0 项**。新增控制矩阵包含 23 个唯一命令 ID；逐项与当前 generated manifest 的 risk、concurrency、timeout_enforcement 对照，69 个字段全部一致。六项修订均已进入对应生成规划和研究生成数据。

本轮复核范围内，规划可以提交用户审批。该结论不表示产品实现完成、运行时测试通过或可以启动任务。首轮保留的 Rust/Tauri、原生、视觉、真实 OAuth 和故障注入未验证边界继续有效。

### 逐项闭合记录

| 首轮项 | 状态 | 修订证据和闭合理由 |
| --- | --- | --- |
| PV-01 · T08 无备份回滚冲突 | 已闭合 | `09-28-settings-lossless-capabilities/design.md:30` 与 `implement.md:44` 改为同批回退代码，禁止为无备份域自动复制前镜像或敏感原文，其他领域不新增备份动作。`prd.md:36`、`design.md:13`、`implement.md:18` 同时要求 Grok typed/raw 保存及 invalid/stale 失败路径不产生备份，比较目标与备份目录，并验证 no-backup notice。 |
| PV-02 · T07 时序测试缺口 | 已闭合 | `09-28-command-workbench-lifecycle/design.md:12` 和 `implement.md:15` 明确三个受控顺序：terminal 先到、queued start 响应后到；新 job 生效后旧 job event/start response 到达；start Promise 未 resolve 时双击。每项要求断言 UI 状态、history 次数、启动及取消控件，不再只依赖一般 snapshot/event 测试。 |
| PV-03 · T11 控制矩阵及 admission | 已闭合 | `09-28-desktop-control-oauth-lifecycle/prd.md:35` 将 C01–C06 纳入 AC1；`design.md:11` 和 `implement.md:12-13` 写明逐族控制可达、get/status 防回归、owner admission 和 ACL/确认。`design.md:16-18` 把父 research/control-command-matrix.md 纳入强制设计，逐 ID 和 N/A 边界可追溯。详见下一节。 |
| PV-04 · T05 无备份文件断言 | 已闭合 | `09-28-safe-persistence-backups/prd.md:37`、`design.md:13` 和 `implement.md:18-19` 均明确 secret:true 与 BackupPolicy::None；create/replace/cancel/expiry/permission failure 各自有合成 verifier/state sentinel。测试比较目标与配置备份目录，并扫描非当前目标文件；首次权限失败不发布目标，替换失败保留旧字节。 |
| PV-05 · T04 validator 路径 | 已闭合 | `09-28-cli-diagnostics-contract/prd.md:15` 现指向已存在的 `crates/ccr-config/src/managers/config_validator.rs`；`research/task-designs.json:213` 同步该路径。未新增无依据的 validators 目录。 |
| PV-06 · A10 事实强度 | 已闭合 | `research/architecture-audit.md:81` 现列出四个具体 cancel ID 及全局/module permit，明确既有 Parallel get/status 不计入当前缺陷，并保留“实际原生阻塞未运行”。该描述与 manifest 及 registry 的静态策略一致。 |

本表中短路径均相对 `.trellis/tasks/`；`research/` 指当前父任务的 research 目录。

### T11 新增矩阵的专项核对

1. **当前事实与目标策略分离。** `research/control-command-matrix.md:7-11` 明确当前字段来自 generated manifest，目标为待实施设计。`:17-39` 的 23 个命令 ID 全部存在且无重复，三项当前策略字段全部匹配 `ccr-ui/src/api/generated/command-manifest.json`。没有把目标 owner 行为写成当前保证。
2. **四个命令族逐一验收。** 矩阵 `:43-46` 分别要求 command、install、usage、OAuth 在相关长 handler 的 barrier 释放前到达控制 owner。查询逐 ID 保留原有可达性。usage 单源/全源同步入口均被覆盖；OAuth token exchange/body barrier 与 callback/release 分别纳入。
3. **N/A 保持实际接口范围。** 矩阵 `:10`、`:44` 和 `:46` 明确 install 无独立 get/status，以 recent 使用现有恢复接口；OAuth 无独立 login get/status。`codex_oauth_login_completed` 被正确标为秘密变更，port probe 不被当成登录状态。未要求为验收新增接口。
4. **后台 admission 覆盖真实生命周期。** 矩阵 `:47` 在 start handler 已返回后再测试第二次 start；同资源不得产生并发作业。收到 cancel 或初步终态后，cleanup barrier 未释放仍占用 admission，实际完成和清理后才允许新作业。T11 `design.md:11` 同时保留 foreground 的完成语义。
5. **权限与确认保留。** 矩阵 `:24` 保留 install opaque capability；`:35-39` 保留账号写入互斥及外部进程归属边界；`:48` 要求逐 ID 比较 authorization、confirmation、audit，并验证辅助窗口、确认、错误 ID 和过期 ID。读取当前 manifest 可见 command/install 查询仍有 system_capability、OAuth port probe 仍有 secret_access，矩阵没有把 Parallel 等同于免授权。
6. **完成与取消语义保留。** T11 `design.md:11` 保留 completion-aware future，矩阵 `:48-49` 区分收到取消与实际清理完成，保留 cleanup failure。测试以 owner acknowledgement、barrier 和虚拟时钟/有界看门狗判断，不允许先释放 barrier、任意 sleep 或仅比较 Parallel 快照。

新增矩阵的这些要求足以关闭首轮 PV-03 的规划缺口。C01–C06 尚未实施或运行。

### 生成数据和集成范围

- `research/task-designs.json:301-303` 保存 T05 secret/no-backup AC、机制与 sentinel 测试；`:423` 保存 T07 三个顺序；`:493` 保存 T08 代码回滚边界；`:672-674` 保存 T11 矩阵 AC、机制与测试。对应 PRD/design/implement 已逐项读取核对。
- 首轮记录的 T10 fixture 文字遗漏已收敛：`09-28-architecture-contract-gates/design.md:12` 和 `implement.md:16` 现为 T01–T09 **及 T11**，研究数据 `:606` 同步。
- T11 design 对控制矩阵的显式引用已核对。本研究角色未读取 implement/check JSONL，因此不对两份角色隔离 context manifests 的内容作独立验证声明。

### 第二轮未验证边界

- 只进行文件读取、JSON 字段比较和本报告追加。没有运行 renderer、产品编译/测试、原生并发、真实账户或用户配置操作。
- 没有修改产品代码、规范、其他规划、task 元数据或任务状态；未启动任务，未执行 Git 操作。
- 本轮没有新的代码差异可供运行时验收。实现后的 C01–C06、secret ACL、无备份目录、时序和全量门禁仍需按计划执行，并保留真实结果。
