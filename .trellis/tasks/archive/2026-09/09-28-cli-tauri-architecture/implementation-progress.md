# 实施进度

2026-09-28：用户在上一轮完整规划和独立复核交付后批准“请按照规划开始实施”。父任务和 11 个子任务的既定范围已获批准，依赖和验收条件不变。先前报告的“本轮仅规划”描述审查阶段，不再表示当前实施授权状态。

## 实施顺序

1. T01/T05/T08 先实施；资源空闲后进入独立的 T06。
2. T01/T05 对应契约通过后实施 T02；T05/T06 对应契约通过后实施 T11。
3. T02 后实施 T03/T04；T11 后实施 T07；T03/T08 后实施 T09。
4. 最后 T10 聚合门禁、规范和父任务验收。

同一文件由负责代理串行修改。实施代理不得自行提交、归档或扩大范围；主会话记录证据并安排独立检查。

## 基线

- HEAD：34d8a85e0e48b793733835e0304c8ed33940fcee。
- 两个原有临时脚本保留，哈希见 research/implementation-baseline.json；正式 lint 的既有 5 条 no-console 失败单独记录。
- 原有 Insights 任务保持独立，不更改生命周期。
- 审查阶段 Rust/原生未执行的检查进入实施验收；环境阻碍不能计为通过。

## 进度

- 父任务：已激活；T01、T05、T08 已激活并交由独立实施代理。
- T06 等待执行资源，其余任务等待已写明的前置契约验收。

- T05 首轮窄包结果：ccr-core guarded_write 21 项、ccr-codex oauth_pending 3 项通过；仍在实施其余验收，尚未独立检查。

- T01 兼容决策：既有 `config_manager_default_ignores_legacy_current_platform_routing` 测试明确 generic legacy 配置域为 Claude。保留并文档化该 legacy adapter，新增调用用 `for_platform`；不新增不确定平台默认值。
- T01 中期回归：ccr-config 94 项通过（一个只供父测试启动的 helper 常规忽略；两个父测试实际执行 12 个子进程）；CLI profile 60 项通过，含已修复的两个缺文件 off 回归；Codex profile 18 项通过。最终独立检查尚待进行。
- T08 中期回归：真实 mapper 8 项、完整 Settings/CodeMirror 行为 11 项通过；backend no-backup fixture 和完整 gate 继续执行。
- 验证异常：ccr-core 与 ccr-codex 测试进程各出现过 0xc0000005；同一用例/二进制随后复跑通过。发生阶段与原因未查明，不能由缺少输出推定启动阶段。`cargo` 当前解析到 `C:/Users/lyh/AppData/Local/mbx/bin/cargo.exe`，mbx help 说明其为编译缓存包装器；原生 cargo 位于 Rust 1.98.0 toolchain。未设置 RUSTFLAGS/RUSTC_WRAPPER 等相关变量，Application Error 日志未找到匹配事件；这些事实不构成缓存包装器致因证据。
- 另有可复现基线失败：未修改的 `core::lock::tests::grok_auth_lock_preserves_native_holder_metadata` 在 Windows 持锁后另开句柄读取时收到 OS 33；T05 单例重现，交最终门禁阶段核对，不记为已通过。

- 2026-09-28 续接：T01 已进入独立检查；T05 checker 发现 Windows persist_noclobber 清除备份只读属性的回归，已按同一源权限恢复并新增行为回归；当前 guarded_write 25 项通过，最终审查未结束。
- T08：32 文件 / 97 项相关 smoke 通过，Grok commands 16 项及强化的已有配置 no-backup fixture 通过；type-check/build 和其余边界门禁通过，正式 lint 仍保留原有 5 项 no-console 失败。等待实施报告及独立审查。
- 已核对 generate_icons.py 的输出集合并恢复本次 build prehook 产生的 57 个无关图标改动；清单和生成时哈希见 research/icon-build-restoration.json。后续构建使用仓库已有 CCR_SKIP_ICON_GENERATION=1，仅作用于当前命令。

- T06 已激活并实施；Usage snapshot/token/active 收敛到单一 registry。授权该 owner 定向生成 UsageImportJobStatus.ts / UsageImportJobStage.ts 并更新消费方，新增 cancel_requested、timed_out、cleanup_failed 语义；T11 继续负责 control 入口直达 owner 的调度接入，不改为全局 Parallel。
- T05 独立行为检查当前 25+9+10+5=49 项通过；CLI 首轮出现 STATUS_DLL_NOT_FOUND，增加 --nocapture 后相同测试通过，缺失 DLL 与原因未确认；不以成功复跑删除原始失败记录。
- 根 version-check 通过；两份用户临时脚本 SHA256 与基线一致，两个既有 Insights 任务状态未变化。

- T05 独立检查已交付：本地契约通过，可供 T02/T11 使用；AC2/AC3 的 Unix 权限分支及完整门禁继续开放。详见子任务 check-report.md。
- T01 独立审查新增 Windows ordinary/verbatim 路径锁、大小写路径备份、空 inactive 首次 add、Grok 无 marker 编辑/删除等反例。root 明确通用服务和仓储统一以磁盘声明 current 为权威，兼容 parser 的首项默认值不能用于激活投影或被无关 patch 持久化；由 T01 完成，避免把修补分散到 T03/T04。
- T08 独立审查确认 Codex 后端为已知字段投影及逐叶合并，旧 smoke fixture 的顶层 shallow replace 语义不符。checker 将 Codex patch 收敛为 dirty leaf 并增加加载后外部 sibling 更改保留回归；Claude/OpenCode 按实际后端语义处理。

- T01 独立检查已交付：7个反例先失败后通过，另增Windows双进程身份回归；config101、CLI platforms63/profile62（重叠）、Codex18、legacy1通过。最后原CLI命令通过；异常复跑前后SHA256相同，原因仍未查明。T02已按T01/T05本地契约激活，实施三端共享profile用例。
- T08清空能力决策：Codex使用已存在null删除契约；Claude/OpenCode没有通用顶层删除接口，不能把可存空串等同于unset。checker为不支持的顶层清空提供明确校验、保留草稿，不扩大后端协议；已支持的嵌套替换/空数组/空对象保持。
- T06完整Usage前端检查出现既有日期跨度失败。root实测实际helper：UTC/AsiaShanghai的100天和31天，在AmericaChicago为99天和30天；原因定位至local midnight毫秒差除以86400000向下取整。证据见T10 research/usage-calendar-day-preflight.json，T10做日历天数最小修正和回归，不能改弱测试预期。

- T08独立检查交付：119项相关smoke、Grok16项、i18n24项通过；修复Codex dirty leaf/null清空、其他平台不支持清空的诚实反馈及RHF字段初始化。T09继续环境身份/dirty refetch，Web/native验收仍开放。
- T06实现已交付并进入独立审查；checker发现core Unix进程父退出后可能遗留忽略TERM的后代。该直接违反T06清理契约的问题纳入T06最小修复，由其独占core process_gateway.rs，与T02 journal文件分离。Linux已有MSRV工具链可尝试离线补验，环境探测过程和限制已记入T10 research。

- T11已激活：T06 reviewer明确交付68+2后端、18前端和typecheck通过的稳定控制接口；T06仍负责core进程清理及Linux补验。T11不得改变其进程owner，实现23个control矩阵ID与C01-C06，保留旧执行资源共享互斥。
- T02负责ProfileOutcome具名类型、GrokProfileActionResponse及profile frontend呈现，新增3个profilesSurface翻译key；T11负责自己的registry/manifest/OAuth生成，批量生成串行协调。

- T06独立检查已交付：本域68项、Windows进程17项、Linux进程9项及前端18项通过；共享usage过滤器新增T11控制反例失败，取消未在执行permit占用时到达owner。交T11完成红绿修复，未把共享门禁标为通过。
- T05新增Linux guarded_write补验24/24，涵盖Unix私有权限及跨进程备份；使用已有MSRV 1.95工具链，Cargo.lock哈希保持一致。OAuth pending Unix分支和macOS仍未由该补验覆盖。
- T05 Linux补验已补齐：pending store 5/5，连同guarded_write共29项。创建0600和替换0400行为通过；macOS、Linux严格Clippy、完整CI仍未通过本次补验覆盖。
- T02实施者已交CLI6、TUI11、desktop5和绑定3项通过结果；独立checker开始只读审查稳定application/journal，与实施者剩余adapter/frontend边界测试并行，源码修改保持单一owner。
- T02独立审查发现尚未通过的事务边界：AtomicWriter替换后目录同步失败可能漏记journal；删除补偿需保留原权限；非激活update的附属失败文案不能误称已激活。已交现有owner修复，允许必要的core内部最小接口改动，保持T05安全/CAS契约；T03/T04暂不激活。
- T04相关前置现已独立通过：平台validator67、Codex profile18、同binary的三平台invalid/deleted/disabled预检1项；实现者确认对应契约冻结。T04据此激活，仅改诊断/report/binary，不修改application/journal。T03仍等待事务恢复的完整相关验收。
- T02核心修复后Linux补验通过：开启test-support的journal9项（含发布后失败与Unix权限恢复），guarded_write24项。默认特性8项为同一journal集的子集，不重复累计。真实MSRV1.95，锁和core源码前后指纹一致；最终独立AC检查仍待交接。
- T04新增测试依赖遭schannel下载失败，已精确撤回该owner新增manifest和lock条目，恢复原lock SHA；采用现有std::process真实binary隔离矩阵，保留退出码/分类/纯读取要求，没有降低验收。
- T11实施者已交runtime11、OAuth17、command24、usage70+2 guards、registry21通过；先前T06共享usage过滤器的控制反例已在该复验通过。尚需最终严格检查和独立复核，未提前宣告父任务通过。
- T02最终三端合同结果：CLI9、TUI8、desktop10（T02用例9与既有ACL1）通过；just lint-strict在等价合并collapsible_if后通过。源码已稳定，待独立checker确认事务与rename验收后解锁T03；全局生成仍由T11串行负责。
- T04真实binary扩展矩阵8项通过，覆盖三平台、两种认证模式、六种状态及实际switch后validate；Windows不可读fixture、runtime损坏和disabled-current分类已有证据，最终审查尚待交接。
- T02最终独立检查新增并发缺口：journal按规范化路径匹配，但guarded writer叶锁身份仍区分Windows normal/verbatim路径；顺序alias测试不足以证明CAS安全。checker负责统一既有身份规则并增加真实锁竞争反例，T03保持未激活。
- T04新增第9项secret反例发现doctor Claude auth-source错误详情可泄露合成源值；owner正在本范围脱敏并保留红例。T11生成校验发现11个DTO尾部残留，原因未查明，已暂停其他导出并保留唯一串行修复窗口。
- T11命令工作台前置独立通过：command_exec24与runtime_policy11通过，真实owner、status/cancel、跨族permit与cleanup barrier已有证据；T07已激活，等待执行资源。T11其余OAuth/生成验收继续，不能将该scoped批准等同完整通过。
- T02别名叶锁竞争两项从0/2红到2/2绿；统一core文件身份helper供repository/journal/叶锁使用。最终稳定五文件Linux复验：journal9、guarded_write24、atomic_writer9、repository17通过，repository唯一ignored为跨进程父测试使用的worker。四组前后指纹一致，原Linux证据保留，新文件带final-alias后缀。
- T04独立检查补查profiles缺失时已有runtime的读取/语法错误；Claude current冲突策略依T01既定valid file→valid registry规则：有效回退加warning，全显式候选悬空保持62，无marker保持inactive；不写回修复。相关实际binary与文档正在同步。
- T11独立检查已复现旧login cleanup可清除replacement pending，owner负责确定交错回归与单次cleanup claim修复；restore等待admission期间的取消竞争继续验证，不修改已通过的command前置。
- T02最终独立验收scoped通过，新增真实CLI历史失败呈现测试1/1通过；三端合同CLI9/TUI8/desktop10与workspace strict均通过。T03已正式开始实施，generic配置页继续Claude显式平台范围，shared enable在一个journal中完成enabled+activation，不扩展其他平台的generic入口。
- T04最终独立验收scoped通过：实际binary12、既有doctor16/validate2、服务doctor14/validate5、configvalidator6与三包strict检查通过；缺profiles时漏检runtime、错误current来源标签与Claude fallback不一致均已修复。
- T11最终独立行为通过：OAuth21（含旧cleanup与restore取消两项红绿）、command24/runtime11/usage70+2/registry21/install60等通过。正式bindings guard在CLI export进程启动返回0xc0000005，原因为未明；其先清空的227文件已按字节快照完整恢复。AC4正式门禁继续开放，T10负责。
- T07已开始实施shell级命令owner、路由恢复、乱序/重复事件与异步listen清理；与T03文件范围隔离。T09等待T03相关契约，T10保留最后聚合顺序。

## 2026-09-28: continued T03 and T07 verification

T03 implementer reports 17 backend tests passed (one child helper ignored), including actual current marker protection. Inventory generation reported Windows OS1224; exact target diagnosis and protected snapshot remain with T03 owner. No final generation acceptance is asserted.

T07 implementer reports 78 commands/shell tests passed in 16 files. Read-only check_t07 started; remaining test expansion and source freeze stay with implementer. Typecheck/build wait for the exclusive T03 generation window. Root prepared T09 environment/session source notes without starting production edits.

## 2026-09-28: T03 generation recovered, T09 baseline reproduced

Root resumed the blocked T03 generation window and added the failed output path to the inventory test diagnostic. Inventory regeneration passed 1 test; the earlier OS1224 did not recur, and its cause remains unknown. Seven config type exports and seven existing config checks passed. Normalization returned zero. The captured 257 files remain present; three expected config DTO files were added and seven expected registry/config artifacts changed. See T03 root-generation-evidence.json. The full bindings guard is still pending T10.

The T03 checker found old draft/new CAS baseline mixing and an unreachable enable action. Implementation and independent frontend verification continue in distinct owned files. Root recorded three T09 red regression cases for auth probe/load errors and dirty-form refetch, including the exact input overwrite. These tests are prerequisite fixtures, not T09 acceptance.

## 2026-09-28: T03 dependency acceptance; T09 activated

Independent check_t03 released the prerequisite: backend 17 passed plus one child helper, shared enable 1 passed, configs/API 26 files and 90 passed. TypeScript, Tauri strict Clippy, formatting and diff checks passed. Formal lint remains failed only by the original protected five no-console findings. Full bindings and native UI remain unaccepted. T09 is activated against these scoped T03 and previously checked T08 contracts.

## 2026-09-28 — T03 final check and T07 late-response correction

T03 independent final report exists; scoped status is passed. T07 checker reproduced two Shell-unmount races (late start and cancel reactivated recovery timers), then fixed recovery scheduling ownership with resume/suspend and retained job identity. The two targeted regressions changed from 0/2 to 2/2. Final T07 regression remains in progress. T09 implementation continues; its temporary type errors are owned by T09. No task archives or commits.

## 2026-09-28 — T07 accepted; T09 backend environment boundary

T07 final independent 19 files /102 tests passed with stable pre/post source fingerprints. T09 independent review found a Claude multi-active-selection TOCTOU and fixed-local Codex/OpenCode capability mismatch. Root assigned the minimum backend owner and clarified R2 scope; no platform expansion. T10 still waits for T09 acceptance.

## 2026-09-28 — T09 independently accepted, T10 activated

T09 independent final: 67 files/279 frontend tests,22 Claude behavior tests,21 registry tests; type-check,formal Tauri bin strict Clippy,fmt,i18n,style,boundaries and cycles passed.100 source/protected-file fingerprints unchanged. Formal lint retains5 protected baseline errors; extended all-targetsClippy4 baseline test findings remain separate. Root accepted related contracts and activated T10 for aggregate/failure-safety/spec/evidence convergence. No commits or archives.

## 2026-09-28 — T10 check dispatch platform limit

Dedicated trellis-check spawn and existing checker follow-up both failed with agent thread limit reached. T10 implementation continues. Root will run full-scope independent review and formal integration gates; active source/spec authors will cross-review non-authored scopes. The failed mandatory role dispatch is retained as a workflow limitation, not represented as passed.

- 最终集成补充：clean 只将缺失的可选配置映射为默认需要确认；损坏配置在删除前以 exit 14 返回，13 项真实 CLI 清理测试通过。list 分派测试改为显式 fixture，并增加缺配置零写入；快照已区分目录与空文件。
- 首次完整 Tauri suite 401 passed / 5 failed；失败快照显示两个隔离 CCR_ROOT 混用。9 个共享 profile fixture 测试缺少桌面环境锁，已按既有 desktop → CLI 顺序补齐，保留默认并行与全部断言，正在完整复验。
- root workspace 出现一次 doctor 孙进程 PID fixture 未生成；未修改该源码，原 exe 定向复跑 1 passed，原因未查明。该失败与后续正式结果分别保留。

## 最终集成补充：TUI 帧断言

根 CI 的 Grok Auth 双语确认提示失败已定位为旧测试的大小写及终端宽字符/换行匹配问题。仅修正测试正文提取与规范化，保留全部 48 帧及两项安全提示断言；定向通过，非作者复核通过。当前正在运行独立完整 Tauri gate，之后执行根完整 CI。原始失败日志全部保留。

## 实施交付检查点

11 个子任务的代码、规范与验收台账已落盘。最终 root workspace 测试和 Release 构建通过；独立完整 Tauri gate、900 项前端测试、24 项 i18n、coverage、VS Code、docs 和隔离 frontend-check 通过。根 aggregate 停在基线 rustls 安全审计；主工作区原脚本 5 条 lint 错误保留。39 AC/33 测试组/9 P1 映射已校验；248 个冻结文件与230个生成类型一致。专用 checker 数量限制及原生/跨平台未验收明确列出。未提交、推送或归档，所有相关任务仍为 in_progress。最终状态以 integration-report.md 为准。

## 2026-09-28 — 续作修复与当前验收

两锁安全修复、4项Tauri测试lint、Codex模型单字段磁盘往返及Settings双语文案已完成。非作者Rust和安全审查通过；完整前端168文件/901测试、i18n24/4523、Lines75.19%通过。Web合成IPC模型保存/原文编辑和纯Web错误状态通过，native/CSP未验证。第一次续作根CI在doctor_timeout缺孙进程PID失败，正在限定诊断；正式frontend-check仍仅原脚本5条no-console，保护文件未授权改动。当前报告：continuation-integration-report.md。

## 2026-09-28 续作检查点：平台补验与证据收敛

安全修复、Settings 无损/双语、Doctor 夹具与单行跨平台导入修正均完成。独立完整 Tauri、901 前端测试、24 项 i18n、75.19% Lines、生产构建、Windows/Linux MSRV、Linux Tauri strict/process/OAuth 定向补验通过。原 root aggregate 仍为 exporter 启动访问冲突 exit 1；后续成功独立记录，原因未查明。两次零匹配过滤结果排除行为验收，正确重跑 Linux aggregate20/Windows1 通过，Linux 有18个完整 selector 可提取。263 路径及230生成类型一致，保护文件和任务生命周期不变。主工作区5条lint、必需macOS、native/CSP与完整Linux lane仍开放，专用checker派发重试仍受限。完整当前结论以 integration-report.md、execution-ledger.json 及最终 evidence matrix 为准；未提交、推送或归档。

本轮最终映射校验：39 AC、42 测试组、9 P1、41 本轮运行、227 个 SHA 核验产物。映射通过不代表完整验收；来源链、失败与未验证状态保留。非作者终审见 T10 research/continuation-evidence-check.md。

## 2026-09-29 UTC — 剩余门禁与原生验证

正式frontend-check及Linux Tauri/coverage通过，5条脚本lint与Doctor PATH夹具缺陷已修复。根CI随后在Tauri版本探测5秒断言失败；正在保留阈值修正宿主依赖。Linux原生验证定位确认弹窗运行时style缺少nonce，共享初始化修复与7项定向测试、type-check、lint、UI build已通过，原生重建复验待执行。macOS和专用checker派发仍受环境限制；全部失败与输入快照保留。

## 2026-09-29 UTC — 权限缺陷修复和 Linux 完整门禁复验

Windows 完整 just ci 在 d8ee 冻结上通过（930.453 秒）。Linux Tauri 和 Gateway 覆盖率通过；原生 attempt6 的六项检查及清理通过。随后根 coverage 原始测试发现语义 no-op 未收紧秘密文件的 Unix 权限；已在既有叶锁和版本检查内补元数据修正，原失败断言保持。两包定向测试和 strict Clippy 通过，db558b 冻结保存六文件修复。

完整 Linux quality 的首次重试因 task-local 环境缺少 python 返回 127，补充既有 python3 的局部入口后，第二次在 T04 测试的非八进制权限字面量返回 101。两次失败均保留。root 已作 0 到 0o0 的等价替换，最新冻结 remaining-portability-source-freeze.json（49801fe0）仅比 db558b 多这一处测试修改。完整门禁正在新冻结上重跑；尚不将旧成功归入新源码。专用 checker 已恢复，5 条脚本 lint 已授权修复，二者均不再是当前阻塞。

## 2026-09-29 UTC — Linux quality 与安全审计通过

ccr-store 原失败精确复现后，仅修正一个测试的平台条件；Windows/Linux exact 均通过，Linux 包 81 项通过、2 项既有 ignored，非作者审查通过。最终冻结 a31c8730 新增登记该文件，共 269 路径。完整 Linux version/governance/fmt/strict Clippy/parallel workspace tests/check-workspace 返回 0（224.101 秒），cargo-audit 0.22.2 返回 0，源码和生成类型无漂移。root coverage 已启动；任务仍 in_progress。

## 2026-09-29 UTC — 最终本地门禁与专职复核完成

最终 a31c 冻结包含 269 路径。Windows 完整 just ci 退出 0（1078.351 秒），前端 169 文件/904 测试、24 项 i18n、75.19% 行覆盖率、docs 与 VS Code 51 测试通过。Linux workspace quality、Rust 1.95.0 MSRV、cargo-audit 0.22.2、root coverage 74.61%、core gateway 96.09%、Tauri gateway 93.03%、完整 Tauri 门禁及 native attempt7 六项限定检查通过。Tauri 总体 52.14% 没有整体门槛。

专职全范围审查与冻结矩阵核对完成：39 AC、48 测试组、164 选择器、9 P1、86 运行、393 仓库产物。41 旧运行与全部历史失败、7 份源码 manifest 保留；源码和 230 生成类型无漂移。4 原 schema 字节及 12 任务/24 JSONL/两 Insights 状态保留，4 条注入警告仍在。5 条脚本 lint 与线程限制已解决；历史访问冲突原因未查明。

原生范围为 Linux WSLg WebKitGTK 的合成本地 Claude 原文编辑和生产 CSP；未测 style DOM 移除或保存后滚动锁还原。必需 macOS、其他原生页面与平台、真实远程/OAuth、交互式 TUI、物理输入及发布包仍未验证；部分旧基线反例边界保留。任务继续 in_progress，未提交、推送或归档，fix_commit 为 null。最终状态以 integration-report.md 和 research/remaining-final-delivery-verification.json 为准。
