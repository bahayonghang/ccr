# Research: ccr-ui React 前端架构审查

- Query: 审查 React 前端调用 CLI/Tauri 功能的状态、契约、配置与认证闭环、测试与规范问题。
- Scope: internal；仅源码与隔离测试，保持 planning。
- Date: 2026-09-28

## Findings

### 结论与证据等级

前端的主要问题集中在跨层功能接线和状态所有权。API 生成、统一设置页和全局事件桥已经存在，但命令页另建路由局部任务状态，Settings 把复杂配置压成标量表单，部分功能标志没有对应运行时机制。现有结构检查通过，仍不能证明用户操作闭环完整。

本审查记录 6 个主要问题和 2 个次要问题。F01、F02、F03、F04、F06 有隔离组件或纯函数复现。F05、F07、F08 由当前源码确认。原生 Tauri、真实配置写入、浏览器与视觉均未验证。

证据用语：confirmed 表示当前源码与明确调用链支持；confirmed / isolated additionally 表示本次隔离测试复现；inferred 表示机制推导，尚未测试触发环境；untested 表示本轮未覆盖。隔离复现不等于原生验收。

### F01 — P1 — 命令任务状态由路由页面持有，页面返回后失去恢复和取消入口

- 状态：confirmed / isolated。
- 触发：启动一个 queued/running 的 CCR 命令，离开 /commands，再返回；或任务在离开期间完成。
- 证据：ccr-ui/src/features/commands/useCommandsPage.ts:49 把 currentSnapshot 放在 useState；:99-134 只订阅未来的进度和终态事件；:125-128 在页面事件回调中追加历史；:173-179 的取消依赖 currentSnapshot。
- 已存在但未接线的机制：ccr-ui/src/api/generated/commandExec.ts:28-29 提供 getCcrCommandJobStatus；ccr-ui/src/api/tauri.ts:613 提供兼容包装。全局 ccr-ui/src/shell/eventBridge.ts:187-189 追加独立 stream store；命令页没有读取该 store。全仓 src 搜索 getCcrCommandJobStatus 仅命中 API 定义；useCommandsStreamStore 仅有 store、导出及 eventBridge 消费。
- 根因：Rust job registry、外壳 stream store、页面 currentSnapshot 三份状态没有统一投影协议。页面生命周期同时拥有后台任务观察、取消入口和历史写入。
- 影响：页面返回后 isRunning=false、currentSnapshot=null，用户看不到活跃任务并不能从页面取消。离开期间的终态没有页面订阅者，页面负责的 history 写入也不会发生。后端任务仍可继续执行。
- 最小重构 owner：features/commands 的 job controller 与 shell event bridge；真实 job registry、终态和取消结果继续由 Tauri 后台拥有。
- 可观察验收：启动→离开→返回仍显示同一 job_id；返回立即从 status API 对账；漏掉终态事件也能得到终态；历史恰好写入一次；停止按钮指向原任务；不会因组件挂载而重复启动。
- 后端依赖：保留 get/status/cancel 的稳定 DTO 和保留期；如要支持窗口重新加载后恢复，需要明确活跃任务枚举或可恢复 job_id 持久化契约。仅解决路由切换时不必扩展为跨应用重启恢复。

### F02 — P1 — start 响应和终态事件没有单调状态合并规则

- 状态：confirmed / isolated；原生事件与响应实际时序频率未测。
- 触发：后台完成很快，commands:job-finished 先到达，startCcrCommandJob 的 queued snapshot 后返回。
- 证据：ccr-ui/src/features/commands/useCommandsPage.ts:113-120 先接收 success；:160-165 随后无条件用 response.snapshot 覆盖。ccr-ui/src-tauri/src/commands/command_exec.rs:1848-1850 先 spawn 后返回初始 snapshot，所以协议未规定 invoke 响应先于事件。
- 隔离结果：先注入 success，随后 resolve queued start 响应，页面重新变成 queued。本次证据测试直接调用真实 useCommandsPage，IPC 全部 mock。
- 根因：事件、start 响应、cancel 响应都直接替换页面快照，没有按 job_id、revision/sequence、terminal 状态归并。提交中的瞬态也没有独立状态。
- 影响：已完成任务可永久显示运行中；用户的取消和后续执行入口根据错误状态启用或禁用。提交 pending 阶段仍可能再次点击，属于同一状态模型缺口；本轮未单独测试双击。
- 最小重构 owner：与 F01 共用 command job controller；不要仅在视图中加时间延迟。
- 可观察验收：start/terminal/cancel 任意允许顺序下，终态不回退；旧 job 事件不能覆盖新 job；重复 seq 被忽略；提交 pending 禁止重复提交；超时或漏事件由 status 查询收敛。

### F03 — P2 — 命令页面订阅异步建立时可在卸载后泄漏

- 状态：confirmed / isolated。
- 触发：路由卸载发生在 Tauri listen Promise resolve 之前；React StrictMode 也可形成类似时序。
- 证据：ccr-ui/src/features/commands/useCommandsPage.ts:130-132 在 Promise resolve 后直接 push unlisten，:133-135 cleanup 只清理当时已加入的数组。没有 disposed 标记，也没有 listen rejection 的 catch。
- 隔离结果：在 3 个 listener Promise 均挂起时卸载，再 resolve 3 个 Promise，3 个 unlisten 调用数全部为 0。
- 对照：ccr-ui/src/shell/eventBridge.ts:150-160 已正确处理迟到 unlisten，且 tests/shell/event-bridge-leak.smoke.test.tsx 覆盖了该桥接层的 3 种时序。该测试未挂载 useCommandsPage。
- 根因：两处任务事件订阅分别实现生命周期，正确机制没有成为共享能力。
- 影响：旧页面回调继续处理任务事件；重复挂载可能导致重复历史写入和冗余状态更新。真实持续会话中的累积数量未测。
- 最小重构 owner：F01 的单一订阅 owner；若保留局部订阅，抽取小型 lifetime-safe listener 工具并覆盖其消费者。
- 可观察验收：延迟 resolve、StrictMode、快速路由切换均满足 listen/unlisten 计数相等；卸载页面不会追加历史；订阅失败进入可见恢复态。

### F04 — P1 — Codex 设置保存覆盖未编辑的合法通知数组

- 状态：confirmed / isolated，未写真实用户配置。
- 触发：现有 config.toml 的 tui.notifications 为事件名数组；用户只修改 model 并保存。
- 证据：ccr-ui/src/types/codex.ts:194 明确允许 boolean | string[]。ccr-ui/src/configs/settings-codex-map.ts:40 把所有数组映射成 false；:96-103 的 buildUi 总是序列化 notifications；ccr-ui/src/configs/settings-codex.ts:66-67 忽略 dirtyKeys 并提交完整表单 payload。
- 后台证据：ccr-ui/src-tauri/src/commands/codex.rs:908-927 解析布尔值或字符串数组；:1162-1163 在 payload 含 notifications 时更新该字段；:2137、:2177 的测试分别确认数组可读、无关更新应保留数组。因此不能按废弃字段处理。
- 隔离结果：输入 {model:'old',tui:{notifications:['agent-turn-complete']}}，只把表单 model 改为 new，buildCodexSettingsPayload 的 tui.notifications 仍为 false。
- 根因：通用 SettingsValues 标量模型没有表达完整领域值，flatten→build 丢失联合类型；保存层未以脏字段构造补丁。
- 影响：保存无关字段会改变通知行为。缺失布尔字段也会被普遍实体化为 false；这些额外默认值改变的逐项业务影响未验证。
- 最小重构 owner：configs/settings-codex-map.ts 与 settings-codex.ts；共用表单负责脏状态，平台 mapper 负责无损领域补丁。后端合并仍为权威。
- 可观察验收：仅修改 model 时，通知数组、未知嵌套字段、未编辑的缺省字段均保持原语义；明确修改通知时才改变该字段；事件数组具备可见表示；未保存和保存成功后的表单基线明确。
- 相关待验证：ccr-ui/src/types/opencode.ts:211 允许 autoupdate 为 boolean | 'notify'，settings-opencode.ts:62 将 'notify' 压为 false，:81 写回布尔值。前端映射损失已可见，但本轮没有追完后端写入链，也未单独跑测试，不据此再提高严重性。

### F05 — P1 — Settings capability 声明没有完整运行时实现

- 状态：confirmed / source。
- 影响平台：Claude、Codex、Grok 的 Settings 原始编辑入口；Grok 的 managed lock、未知枚举和策略层提示。OpenCode 的双文件保存另有平台函数，本条不声称双文件完全未实现。
- 证据：三个 SettingsView 均为 BaseSettings 薄壳；ccr-ui/src/configs/settings-types.ts:35-57 声明 rawSource、managedLocks 等标志，但 load 仅返回 Record<string, scalar>，没有 locks、配置层、版本 token 或 raw callbacks。ccr-ui/src/configs/settings-grok.ts:50 声明全部标志，:101-109 只提取表单值，丢弃 managed_keys_locked 等响应元数据。
- 视图证据：ccr-ui/src/features/platform/settings/BaseSettings.tsx:25-41 只加载 probe/values 和 reset；:97-116 仅表单保存、tab、field controls。ccr-ui/src/features/platform/settings/SettingsFieldControl.tsx:23-68 没有锁定 disabled 或原因字段；:37 只渲染静态 options，不补充当前未知枚举。
- 原始 API 状态：ccr-ui/src/api/domains/claude.ts:227、:238，codex.ts:430、:441，grok.ts:133、:161 仍导出 raw read 和 layers；对应 typed generated clients 仍存在。ccr-ui/src/features/editor/ConfigSourcePanel.tsx:15-47 提供现成 read/save/layers 界面，但当前 src 里没有 Settings 消费者。没有发现明确退休这些入口的依据；现行 raw-config-editor-contracts 与 grok-settings-contracts 仍要求它们。
- 规范证据：.trellis/spec/ccr-ui/frontend/grok-settings-contracts.md:45-55 要求 raw chrome、managed-lock disablement、整数校验；:116-129 要求 managed_keys_locked、未知枚举当前值、Grok 无备份提示和 policy layer 提示；:137 要求 managed_locked 恢复链接。
- 根因：按页面外观统一的 capability flags 没有配套可执行接口；平台响应先被压平，后续 Base 无法恢复领域元数据。
- 影响：用户无法从 Settings 进入已声明支持的原始编辑；Grok 托管字段仍可编辑，提交后只收到字符串化 managed_locked；未知枚举没有当前值选项，界面无法准确表达磁盘状态。后台权限与锁校验仍存在，本条不声称可绕过后台。
- 最小重构 owner：configs/settings-types + 各平台 Settings adapters + BaseSettings。保留无平台名分支的 Base；增加可选 raw source capability、lock metadata、current-value options 和 structured save outcome。不要通过 cacheKey 判断平台。
- 可观察验收：Claude/Codex/Grok 的源码入口均能读取、编辑、携带原 token 保存、显示 invalid/conflict、成功后退出源码模式并重载表单；非 Local 时不调用原始文件 IPC；Grok 展示无备份提示、真实 policy layer 提示、锁原因与 Profiles/off 恢复；未知枚举显示且未编辑时保持；所有原始内容不进入全局 store/localStorage/log。
- 依赖：ConfigSourcePanel 位于 features/editor，复用时需要遵守 features 只能跨域依赖 features/platform 的现有规则。可将领域中性的 editor composite 放到明确共享层或登记精确适配边界，不能为了接线扩大所有 feature 的跨域豁免。

### F06 — P2 — Grok Auth 读取失败显示已退出，环境失败持续显示加载

- 状态：confirmed / isolated。
- 当前活跃消费者：ccr-ui/src/features/grok/GrokAuthView.tsx:5，渲染 BaseAuth。Claude/Codex 仍用各自账号页面，因此本条不扩大为三个 Auth 页均已复现。
- 证据：ccr-ui/src/features/platform/auth/BaseAuth.tsx:15-23 分别查询 probe 和 session；:49-63 只处理 unsupported 和 session pending；:65-67 将缺少 session 数据映射为 signedOut。没有 probe/session error 分支。:30-46 的 authOff 也没有 catch 或 mutation pending 管理。
- 隔离结果一：session load reject 后 Query 状态为 error，但 UI 显示 audit.signedOut，notify.error 未调用。
- 隔离结果二：probe reject 后 dependent query 保持 pending，界面持续为 loading，load 未调用且没有 retry 控件。
- 根因：view model 把 missing、loading、error、signed-out 四类状态合并成两类；probe 和业务 query 错误由不同 source 持有，但统一界面没有完整状态图。
- 影响：用户将磁盘/环境故障误认为已退出登录；环境探针失败没有恢复入口。退出失败的用户反馈和重复提交仍需单独回归。
- 最小重构 owner：BaseAuth 与 auth config 的错误/结果契约；不要求把 Claude/Codex 的 OAuth/provider 全部塞入通用 Base。
- 可观察验收：初始 probe error、初始 load error、带旧数据的 refresh error、signed-out、signed-in、unsupported 各有明确状态；retry 重试正确依赖；authOff pending 防重复、错误保留原状态并显示反馈。

### F07 — P2 — Settings 查询刷新无条件替换用户草稿

- 状态：confirmed / source；自动触发频率和真实环境切换场景未测。
- 触发：用户修改表单但未保存，相关 Query 后台刷新返回不同的数据对象，例如环境切换引发全量失效。
- 证据：ccr-ui/src/features/platform/settings/BaseSettings.tsx:40-41 在 valuesQuery.data 变化时无条件 reset；:26、:31 queryKey 只有 config.cacheKey；ccr-ui/src/shell/eventBridge.ts:183-184 对环境事件 invalidate([])。没有 dirty 保留、切换确认或 environment identity。
- 根因：服务器快照和本地编辑草稿没有独立版本边界；缓存失效直接重置编辑表单。
- 影响：未保存输入可能丢失；不同环境复用 key 的短暂旧数据和在途竞争需在实施时补充场景测试。后端 Local-only 限制是否拦截写入属于后端 owner，本条不认定跨环境数据已被写错。
- 最小重构 owner：Settings snapshot/draft session。新服务端数据应更新 baseline 或展示冲突，不应覆盖 dirty fields；环境切换需要显式会话重建规则。
- 可观察验收：dirty 表单遇后台 refetch 保留编辑；环境切换有明确的保留/放弃策略；保存成功才更新 baseline；过期读响应不能覆盖新环境或新编辑会话。

### F08 — P2 — 配置页语言订阅与缓存标签不符合现行规范

- 状态：confirmed / source；本轮没有浏览器交互复现。
- 触发：配置页已挂载后更换应用语言，且配置数组引用不变。
- 证据：ccr-ui/src/features/configs/ConfigsView.tsx:4 导入裸 t；:32-38 的 tab 标签 useMemo 依赖为空。hooks/useConfigsPage.ts:7 裸 t，:37 的 summary memo 仅依赖 configs。features/configs/locale.ts:4 明确要求组件使用 useAppT 订阅。
- 根因：翻译函数被当成稳定纯工具使用，语言变化没有进入组件订阅和 memo 依赖。
- 影响：标题/摘要/tab 可保留旧语言；memoized ConfigCard 等子组件也使用裸 t，不能依赖偶然父重渲染恢复。
- 最小重构 owner：configs locale hook 与对应视图；在 Settings/配置 UX 子任务中限定修复当前触及表面，避免全仓文案清理。
- 可观察验收：保持查询数据不变，切换 zh-CN/en-US，tab、summary、已挂载卡片同步变化；测试不通过重新挂载页面满足断言。

### 已有合理设计，应保留

1. 生成命令客户端和 manifest 驱动 invokeRuntime 已形成真实边界。src/api/generated/commandExec.ts 与 invokeRuntime.ts 应继续作为调用层；不要把所有调用退回自由字符串 invoke 或让页面重做确认策略。
2. 外壳 eventBridge 已正确处理延迟 listen 的卸载，现有 3 个泄漏测试通过。命令页应复用正确机制。
3. shell/queryClient.ts:3-19 设定有限 gcTime 120 秒、staleTime、重试与 focus 策略。不能把有限缓存本身列为错误；应修复 query key、状态 owner 和 mutation 失效范围。
4. Grok dirty patch 已按字段构造 set/unset，settings-grok.ts:111-122 使用现有专用 mapper。Codex 的修复可以保持平台独立 mapper，不需要扩大通用 Base 的领域判断。
5. raw editor 的版本 token、Local-only gate、语法错误和冲突结果已存在。问题是 Settings 接线和元数据传递缺失，优先恢复功能，避免另建第二套 editor。
6. 部分 Auth 特性没有统一是现行 platform-surface-contracts 的明确设计；Claude 官方账号诊断与 Codex OAuth/providers/quotas 不应仅因代码不同而强行合并。

### 规范、设计与测试质量

- 薄壳行数、禁止平台名字分支、API facade 冻结和类型签名检查都有价值，但不覆盖 capability 是否真正可操作。platform-base-settings.smoke.test.tsx:38-49 只验证两个标题出现，没有调用 raw source、locked fields 或保存联合类型。
- cache-route.smoke.test.ts:41-54 直接向 stream store 写入并检查数据保留，没有挂载 CommandsView；因此无法发现实际页面不消费 store。名称中的「卸载不清空」没有对应路由生命周期操作。
- event-bridge-leak.smoke.test.tsx 只测外壳桥接，未覆盖 useCommandsPage 的独立订阅。
- grok-settings-api.smoke.test.ts:26-85 检查 wrapper 的 token、status 和 patch 转发，未证明 Settings 页面提供源码入口或锁定控件。
- typed-command-boundary 检查 Rust 签名不返回 Value；该检查不能证明前端 mapper 不会压缩联合类型。因此应增加 read→edit-one-field→save→reread 的契约测试。
- DESIGN.md 要求 useAppT、12px surface 上限和抑制紫色装饰。ConfigCard.tsx:18-34 的 violet/gradient 映射以及 BaseAuth.tsx:83 的 rounded-2xl 与当前方向不一致。这里只记录源码偏差，不声称实际对比度、布局或可访问性已完成视觉验收。功能优先；不建议单独以全面美化扩张任务。
- code_map.md 中「just check 的 lint 可能 auto-fix」与当前 ccr-ui/AGENTS.md 及 package.json 的 lint/no-fix 分离冲突；属于文档事实漂移。影响操作选择，应由相关文档 owner 在后续更新中核对。
- 当前范围内没有发现需要取消整个 React/TanStack Query/typed IPC 架构的证据。需要的是补全领域契约和用户路径测试。

### 本次验证

运行目录：D:/Documents/Code/Github/ccr/ccr-ui。

研究配置仅写本任务 research 目录，cacheDir 也定向该目录；沿用原 smoke 的 React plugin、jsdom、setupFiles、restoreMocks 与 clearMocks。执行前已检查选中的测试，只使用源码读取或 mock IPC；没有启动真实 Tauri、读写用户配置、生成 bindings 或运行 auto-fix。

现有测试命令（加入额外证据文件前运行）：

    node node_modules/vitest/vitest.mjs run --config ../.trellis/tasks/09-28-cli-tauri-architecture/research/frontend-audit.vitest.mjs --configLoader native

结果：9 files passed，29 tests passed，exit 0，8.38 秒。

范围：api-facade-boundary、typed-command-boundary、command-runtime-policy、event-bridge-leak、cache-route、platform-base-settings、platform-surface-unify、grok-settings-api、claude-auth-view。

额外证据命令：

    node node_modules/vitest/vitest.mjs run --config ../.trellis/tasks/09-28-cli-tauri-architecture/research/frontend-audit.vitest.mjs --configLoader native frontend-audit.evidence

结果：1 file passed，6 tests passed，exit 0，0.987 秒。6 个断言刻画当前缺陷：通过代表缺陷被复现，不能计为产品修复通过。证据源文件是同目录 frontend-audit.evidence.test.tsx。

研究配置初次运行漏加 clearMocks，造成 runtime policy 测试被前序调用污染；配置纠正为原项目语义后 29 项通过。额外证据配置初次遇到 Windows glob 分隔符与外部目录 bare import 解析问题，补充路径标准化和 dedupe 后 6 项通过。上述均为研究 harness 修正，没有据此给产品新增问题。

主线程另行提供的基线：type-check、cycles（722 文件）、boundary fixtures 通过；正式 lint:ci 因两个原有 .tmp 脚本的 5 条 no-console 失败。研究 agent 未独立重跑这些命令，也没有把排除临时文件后的诊断等同正式门禁通过。

### 建议前端子任务与后端依赖

#### 子任务 A：命令执行任务状态与事件生命周期收敛

- 包含 F01/F02/F03；owner 为 features/commands、shell event bridge 与必要 typed job API adapter。
- 先与后端确认 job_id、序列、终态、cancel、保留期契约，再实现单一 job controller。页面保留 selection/filter/draft，后台执行状态不由路由 mount 决定。
- 第一批验收：路由往返、终态先于 start 响应、漏事件/status 恢复、延迟 unlisten、旧任务事件、重复 seq、提交 pending、历史恰好一次。
- 后端依赖：稳定 status API 已存在；不自动扩张到任务重启恢复。历史 owner 的迁移应由前后端共同确定，避免双方同时写入。
- 回归范围：正常成功、非零退出、取消、cleanup_failed、截断输出、错误恢复。保留后端命令白名单和确认机制。

#### 子任务 B：Settings 与 Auth 的领域契约和用户操作闭环

- 包含 F04/F05/F06/F07；F08 仅限触及的配置表面，或作为明确附属修复清单。
- 优先级顺序：先修复未编辑字段保存损失；补全 settings snapshot metadata/patch/outcome；恢复 raw source 和 managed locks；再统一错误态、恢复入口和 draft 刷新策略。
- 先用 Codex notifications 数组建立无损保存反例；对 OpenCode autoupdate union 完整追踪后再决定相同修复。不能把所有未知值统一归零或归 false。
- 平台矩阵：Claude raw/layers；Codex raw/layers + notifications union；Grok local gate + locks + unknown enum + policy + no-backup；OpenCode dual-file 与现有 union（待确认）。
- 后端依赖：读取 DTO 中返回锁/层/token；save 返回 structured outcome。现有 CAS、权限、备份、Local-only、managed_locked 保持后台权威。
- 可独立验证：使用临时或内存 fixture 形成 read→edit→save→reread 场景；发生 conflict/invalid/unsupported 时真实文件不变、表单输入保留；UI 显示正确恢复动作。
- 验收门：现有 focused smoke + 新行为测试 + type-check/lint；跨 Rust DTO 修改再补 bindings drift、对应 Rust 测试与项目规定的综合门禁。原生/视觉另列，不用源码断言替代。

### Files Found

| 路径 | 用途 |
| --- | --- |
| ccr-ui/src/features/commands/useCommandsPage.ts | 命令选择、启动、事件、取消、history、账本的页面控制器 |
| ccr-ui/src/shell/eventBridge.ts | 应用级事件到 Query/store 的桥接 |
| ccr-ui/src/features/commands/stores.ts | 独立 stream 缓冲与视图偏好 |
| ccr-ui/src/api/generated/commandExec.ts | 后台 job 的 typed start/status/cancel |
| ccr-ui/src/features/platform/settings/BaseSettings.tsx | 所有平台 Settings 共用表单 |
| ccr-ui/src/configs/settings-types.ts | Settings capability、字段与值契约 |
| ccr-ui/src/configs/settings-codex-map.ts | Codex 领域值与标量表单转换 |
| ccr-ui/src/configs/settings-grok.ts | Grok dirty patch 与 probe 接线 |
| ccr-ui/src/features/editor/ConfigSourcePanel.tsx | 已有原始文件编辑和配置层展示 |
| ccr-ui/src/features/platform/auth/BaseAuth.tsx | Grok 活跃使用的共享认证状态视图 |
| ccr-ui/src/features/configs/ConfigsView.tsx | 配置页面与缓存文案标签 |
| ccr-ui/tests/platforms/platform-base-settings.smoke.test.tsx | 当前共享设置页的基础渲染断言 |
| ccr-ui/tests/shell/cache-route.smoke.test.ts | store 保留与滚动缓存断言 |

### Related Specs / External References

- 已阅读：.trellis/workflow.md；ccr-ui/frontend 的 index、layering-contracts、api-facade-boundary、platform-surface-contracts、environment-scoped-dashboard-contracts、react-rerender-discipline、raw-config-editor-contracts、profiles-page-contracts、grok-settings-contracts；ccr-ui/AGENTS.md、DESIGN.md、根与前端 code_map。
- 已使用技能：research（主源码证据）和 su-architecture-first（owner、source of truth、change class、验收边界）；trellis-start 只读取入口，遵守研究角色隔离，没有自行 start 或创建子任务。
- 版本来源为当前 ccr-ui/package.json：React 19.2.8、TanStack React Query 5.102.8、Tauri API 2.11.1、Vitest 4.1.11。这里记录仓库声明，不声称是上游最新版。
- 无需外部网页才能确认上述内部调用链，因此未开展外部文档检索。所有结论指向当前 checkout；没有读取 implement.jsonl/check.jsonl。

## Caveats / Not Found

- 不修改产品代码、规范、任务状态或其他任务目录。
- 不操作真实 UI、配置、浏览器或截图；不将源码结论描述为原生复现。
- 根 CONTEXT.md 不存在。
- 首页 Insights 前端任务仍在实施；重叠问题只记录，不改动。
- 两个原有临时脚本 ccr-ui/.tmp-desktop-probe.mjs、ccr-ui/.tmp-insights-visual.mjs 保留。
- usage 事件目前存在广域 invalidation（eventBridge.ts:170），可能带来额外请求，但本轮没有测量请求数量、后台成本或用户延迟，不列性能已复现问题。该区域与首页 Insights 在做工作重叠，应由原 owner 协调。
- 初始内存关键词快查没有当前 CCR 项目的相关命中；没有把其他仓库的旧事实用于结论。
- 本报告属于重点调用链审查，不声称逐一审阅了 722 个前端文件或所有平台功能。安全、原生、视觉、真实数据保存和全量门禁仍应在对应实施任务里完成。
