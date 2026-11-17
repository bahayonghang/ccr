# SSH 执行环境本地实施记录归档

## 2026-11-17 关闭核验：本轮条件已核销

本节是用户明确授权实际关闭 [#205](https://github.com/bahayonghang/ccr/issues/205) 后的当前依据，覆盖下方旧轮次“保持 OPEN”“验收未完成”和最终 CI 尚待执行的时态；旧正文及 Session177/178 保留当时日期语境。本轮按 A→B→C 串行核销，不再执行已完成的九票或历史整改。最终 GitHub `state`、`state_reason`、`closed_at` 与评论 URL 只以 `outputs/ssh-checks/205/closure/run-20261117-080548/closure-receipt.json` 的独立实读为准；本节写入时，公开评论与关闭动作尚待协调者执行，未提前声明 CLOSED。

### 关闭条款与复用依据

- [A 关闭矩阵](../../../../outputs/ssh-checks/205/closure/run-20261117-080548/closure-matrix.md) 已逐条核销父规格 §2 至 §6 和九票全部 26 条 AC：`blocking_defect=0`、`required_evidence_gap=0`，无需产品改动。实现、owning 回归、原协议日志、截图与实跑 HEAD 均逐项对应，不以子票 CLOSED 代替验收。
- [B 单一独立只读审查](../../../../outputs/ssh-checks/205/closure/run-20261117-080548/B/review.md) 为 clean，Standards 0 / Spec 0、`closure_ready=true`。session UUID authority、retained owner/global Busy、当前环境展示、SQL/协议 owner、预算与审计例外保持；B 没有修改文件、构建、测试或 SSH/UI 操作。
- 产品/测试源码仍为 `3ebfbadaa509db4a73d6195fa6e778eef3b2b137`，协调者在该 HEAD 独立运行的 11 项门禁全部 exit 0。随后三文档提交 `77aa5719966fc876d96193d5ac4a2494dcc1effa` 的完整 `just ci` 已实际通过：exit 0、1131.402 秒、17/17 阶段，最终修复轮 0/1，见 [原最终集成收据](../../../../outputs/ssh-checks/205/completion/run-20261116-230934/authorized-20261116-234754/final-integration-receipt.json)。旧 `173c` CI 仅保留历史 lineage，不冒充该次完整 CI。
- 本轮 A/C 仅核销和关闭文档，没有产品、测试、依赖、tracked 执行脚本或门禁改动。[协调者输入/日志/引用核验](../../../../outputs/ssh-checks/205/closure/run-20261117-080548/coordinator-integrity-A-accepted.json) 确认源码、原五脏与 CI 输入相同，11 门禁日志和完整 CI 日志 hash 对应，故按本轮授权复用上述结果；新文档 HEAD 未新跑 CI，最终冻结输入由协调者复核。[独立源码核验](../../../../outputs/ssh-checks/205/closure/run-20261117-080548/coordinator-A-evidence-verification-normalized.json) 对 30 个文件保留 raw SHA 与 Git 规范化 blob 比较；两项 CRLF/LF 差异不是源码漂移，首轮验证器误判也保留。

### 交付证据、恢复与保留限制

- S1 是生产 GPUI 的原生输入加 fixture：11 类场景，85 图实际查看、83 可用 WGC / 2 黑图拒绝，原协调者亲看 8 图；本轮协调者另看 3 图。它不代表真实远端 UI 一体联调或全局物理 pointer/hit-test。#209 两图、#210 八图确有公开 GitHub 附件，A 本轮重新 GET 10/10 为 200/PNG 且 hash 相同，仍标原拍摄 HEAD。
- S2 真实 SSH、Windows keyring、认证、信任、HOME、SFTP/备份/原子替换等独立 60/60；S3 新随机设施的真实 llmusage 2.1.0、1 event/120 tokens、WAL/schema/平台日期、取消与私有快照等独立 66/66。A/B 实读原日志 126 个 passed checkpoint；真实协议/CLI、故障 shim、原 Tauri owner 与 GPUI fixture 分层。
- 三目标历史身份整改已按用户精确授权完成，19 OID 映射的 tree/message/timestamps 保留；旧 objects/reflog 保留，旧公开评论的“三条署名未修”仅是旧轮次事实。本轮不再改写历史。
- 应用不能证明远端树退出时保留 `CleanupFailed`，失权对象不借新 app session 清理；独立管理恢复不等于应用杀树成功。已知 S1/S2/S3 运行设施有精确恢复收据，未确认对象为空，本轮没有新设施。首次空 bootstrap 的本地 ValueError 发生于管理成功响应后的记账，仍保留缺独立二次精确查询的证据限制，不将其改写为查询通过。
- 生产 HTTPS WebDAV 的额外真机验收维持 `excluded_by_user`，不计通过或本轮阻断；原 `ccr-sync` v2/显式 v1、冲突/force、部分发布、备份恢复 fixture 仍已核。活跃 SQLite 同步、SSH WebDAV 解绑/legacy clear、纯空目标新建、会话克隆/回收站/恢复、多主机并发、WSL 切换及 Tauri russh 沿用既有接受边界。
- 继续未测：其他 native OS、Pageant/部分私钥格式的额外实体互通、真实远端 UI、物理 pointer、长 profile/重定向根与完整辅助功能。深 264 字符 ACL 失败来自 ignored 验收路径，默认本 profile cache 约 160 字符且失败不发布；UIA `IsPassword=false` 与无 Value/Text 读 pattern、像素掩码/保存清空分别记录。四处合成工具 canary 曾事后 mask，不声称始终落盘前脱敏。上述逐项理由在关闭矩阵保留，未豁免真实功能缺陷。

### 本轮文档与公开收口

C 仅更新本主题并追加唯一 Session179 与对应 index，保留旧 journal 前缀。当前 recorder 使用 `--no-commit --topic` 后仍因旧 Session28 重复在写前 exit 1，workspace Markdown 零写；确认写前字节 CAS 后已用 append-only 手工 fallback 完成179与索引，失败和方式保存在本轮 `C/recorder-receipt.json`、`C/recorder-manual-fallback-receipt.json`，没有修改旧编号或保护脚本，也未冒称 recorder 成功。公开评论草稿仅在本机 ignored 输出，由协调者填最终 HEAD、敏感信息审查与确定性标记后发布；随后以 completed 关闭、回读 GitHub 并生成关票收据。本轮本地提交不包含原五脏、outputs 或截图，未 push/PR/merge，dev/main 与保护 refs 保持；本地证据不伪装成公开链接。

## 2026-11-17 授权补验：当前交付与终态边界

本节是后来获得明确授权后执行的补验；[本轮入口](../../../../outputs/ssh-checks/205/completion/current-authorized-run.txt) 指向 `outputs/ssh-checks/205/completion/run-20261116-230934/authorized-20261116-234754`，下称“当前证据根”。[逐阶段记录](../../../../outputs/ssh-checks/205/completion/run-20261116-230934/authorized-20261116-234754/progress.md) 保留首次失败、退修及后续通过的顺序。后面的 2026-11-16 原轮次及旧 Session 177 仅描述当时事实，不代表当前状态。

当前分支为 `feature/ssh-environment`，最终产品/测试 HEAD 为 `3ebfbadaa509db4a73d6195fa6e778eef3b2b137`。S1 已按原生输入＋fixture 的明确边界验收；S2 真实资源 60/60、S3 新 nonce 独立 66/66、S4 Standards 0 / Spec 0，且该源码 HEAD 的协调者独立 11 项门禁全部 exit 0。**本轮最终完整 `just ci` 由协调者在所有 tracked 提交完成后冻结新 HEAD 运行，实际终态以最终收据为准。** 本文与新 journal 是 S5 文档交付，不提前宣称最终集成、归档终态或父任务完成。

最终集成和本地归档终态以当前证据根的 `final-integration-receipt.json`、`archive-receipt.json` 为准，由协调者在本次文档提交后生成。本地归档不等于关闭父票。协调者在 **2026-11-17T09:38:40.878Z** 通过 GitHub connector 只读 REST 查询：[状态收据](../../../../outputs/ssh-checks/205/completion/run-20261116-230934/authorized-20261116-234754/github-state-before-final-ci.json) 中 #205 OPEN、#206–#214 CLOSED；最终 CI 后仍需再次只读核验。

### 授权变化与精确历史整改

- 后续明确授权涵盖私有远端 nonce 设施、精确历史元数据整改及仅操作本轮 own PID/窗口的 Win32/UIA 工具；原 prompt 本身不授权任意主机、个人配置或其他窗口。[权限收据](../../../../outputs/ssh-checks/205/completion/run-20261116-230934/authorized-20261116-234754/permissions.json) 为准。
- 用户本轮明确“不用管 webdev”，按 WebDAV **`excluded_by_user`** 记录：不是 passed，不再作为本轮阻断，也未为其新增服务或依赖。
- 已授权并执行仅 `303c898e`、`12e741a2`、`5d908cb6` 三项 author/committer 名称和邮箱整改，共 19 OID 映射。原 HEAD `7c2f3677a46d07c2a0ef09fe01e5c7dbaa4faad8` 映射为 `b3f0dbab17c87fc44ad45581cc9bf73b695cbe63`；原源码 `173c9f0b6d75cff5179a3942c90d3b4f586ef0fa` 映射为 `654862f97251a0cb5dde89302c0b6d94ad09cb6e`。树、消息、时间戳、非目标身份、其他 refs、index 与五个原脏文件均保持；目标身份合规检查为 true。旧 objects/reflog 保留，不复制旧个人值。[执行与逐 OID 核验](../../../../outputs/ssh-checks/205/completion/run-20261116-230934/authorized-20261116-234754/history-execution-receipt.json) 保存完整映射。
- 上述三项元数据阻断已解决；这不自动证明所有公开发布条件满足。本阶段不再改历史，不 push、PR、merge、删分支或修改 dev/main、全局 Git/主机设置。

### 新增代码、回归与门禁来源

相对整改后基点 `b3f0dbab17c87fc44ad45581cc9bf73b695cbe63`，产品/测试共 5 提交、12 文件；author/committer 与规定 footer 合规布尔均已由协调者独立核验：

| 提交 | 交付与证据边界 |
| --- | --- |
| `aa34e28e6a2efebfd724ee8d1fad4ca9199032ae` | exec 发起后失权统一保留 `CleanupFailed`；执行前拒绝仍为零命令，不借用或撤销替换 session。IO 17、remote_activity 20 窄测通过，并补 owning 契约。 |
| `5d0e2e2356dc392463611daf005726aa84efd454` | 用量 fixture 明确单 Connection seed、close 后才取源文件 baseline；保留原全文件 byte 断言。 |
| `55a164ed9db1ccf28edce72710aff67fa4c6114c` | 修复离线 Recovery 早返回隐藏既有安装/导入状态：只读 retained owner，保留 Busy、取消、导航后状态和自动终态通知，不创建 owner/IO 或加载旧 report/plan；叶层按当前环境过滤。 |
| `8e297eba5dba10cd29e770bcf73b8e8a2c7e0944` | 导入展示、取消入口和终态 refresh 限定当前环境；raw shared owner/global Busy 保持，文案明确当前执行环境，安装取消卡保持紧凑。 |
| `3ebfbadaa509db4a73d6195fa6e778eef3b2b137` | 仅 cfg(test) 只读观测与 owning 回归等待真实 view snapshot/loading、确认帧就绪；不改变 shipping UI、原预算或默认并行。 |

协调者在最终产品/测试 HEAD `3ebfbadaa509db4a73d6195fa6e778eef3b2b137` 新跑的 [11 项独立门禁](../../../../outputs/ssh-checks/205/completion/run-20261116-230934/authorized-20261116-234754/coordinator-final-source-gates.json) 全部 exit 0：根 `just version-check`、`just fmt-check`、`just lint-strict`、`just test`、`just audit`、`just ui-check`；`ccr-desktop` 下 `just desktop-fmt-check`、`just desktop-lint`、`just desktop-test`、`just desktop-test-ui`、`just desktop-audit`。完整命令、cwd、完整 HEAD、退出码、耗时和原始日志路径在收据中；这是独立子门禁结果，不能冒充本轮最终完整 CI。

原 `aa34` 首轮 10/11：desktop-test 源 DB byte 断言失败，before 缺少六个 seed marker、after 出现；pool 生命周期 600 样本未复现的否定结果也保留，不能称为重复复现。`5d0e` [后续 11/11](../../../../outputs/ssh-checks/205/completion/run-20261116-230934/authorized-20261116-234754/coordinator-S3-final-gates.json) 才全部通过。`8e` 首轮 10/11 的 desktop-test-ui 失败及临时诊断保留；根因是 backend ready 早于 view receiver/loading/确认帧，而 Kit `disabled=None` 不能证明可操作。`3eb` 在原预算及默认并行下转绿，不将两轮首失败抹写为首次全绿。

旧完整 `just ci` 只发生于原 `173c9f0b`，exit 0 / 1245.8 秒 / 17 阶段；[历史 lineage](../../../../outputs/ssh-checks/205/completion/run-20261116-230934/authorized-20261116-234754/history-ci-lineage.json) 仅证明它与元数据映射后 `654862f9` 的源树对应，不覆盖新增源码或五个用户未提交 Trellis 文件。旧 B 的 RustSec TLS EOF 与同 HEAD 窄重试通过仍是历史事实。本轮最终 CI 修复预算在 S5 交付时为 0/1；默认并行、独立 binding 导出、容量/阈值/超时及既有审计忽略均未放宽。

### S1：原生输入、截图与 fixture 分层

[11 类场景矩阵](../../../../outputs/ssh-checks/205/completion/run-20261116-230934/authorized-20261116-234754/native-S1/run-20261117-014949/scenario-matrix.md) 覆盖 Local/SSH 与失败停留、草稿/作业确认、保存 SSH 启动一次、离线独立页、迟到请求、同 host authority、安装 Busy/取消/导航/终态、每 host 隔离、Light/Dark、GPUI ImportOwner 和合成凭据修复。生产 GPUI 页面、请求及状态机由 ignored consumer 注入既有 Workspace fixture/transport；**不是实际远端 UI 联调**。真实 SSH/CLI 层由 S2/S3 单独证明，原始 Tauri registry 结果也不替代本组 GPUI ImportOwner。

最终 consumer 对应 `3ebfbadaa509db4a73d6195fa6e778eef3b2b137`，185 个源文件/hash 与复制源已核，只有 `lib.rs`、`runtime/environments.rs`、`runtime/mod.rs` 三个既有 bootstrap/admission shim；main 原样。旧 5d/55a/8e 阶段图与失败逐项标源，不把它们全部写为 3eb 新图。[协调者 S1 验收](../../../../outputs/ssh-checks/205/completion/run-20261116-230934/authorized-20261116-234754/coordinator-S1-acceptance.json) 与 [source/cleanup 独立核验](../../../../outputs/ssh-checks/205/completion/run-20261116-230934/authorized-20261116-234754/coordinator-S1-source-cleanup-verification.json) 为最终依据。

输入使用实际 UIA supported patterns、own HWND 标准 Win32 消息；317/319 的严格 own 前台键盘 SendInput 实际成功。UIA Invoke 成功布尔不等于业务生效，按可见像素和 backend 结果核销；没有验证全局物理 pointer/hit-test，也未放宽输入守卫。[精确输入 API](../../../../outputs/ssh-checks/205/completion/run-20261116-230934/authorized-20261116-234754/native-S1/run-20261117-014949/input-method-summary.json) 保留各项边界。WGC `CreateForWindow` 每次前后限定 own PID/HWND，未捕全桌面或操纵其他窗口。

[85 图查看与 hash 清单](../../../../outputs/ssh-checks/205/completion/run-20261116-230934/authorized-20261116-234754/native-S1/run-20261117-014949/viewed-images.json) 中 85 张均实际 view：83 张可用 WGC，2 张早期黑图明确拒绝。协调者亲自 view 8 张，审查者另 view 278/292/306/319。关键最终图 278 Dark 取消中、282 capture-only 自动 Failed、292 Light 终态、306 cleanup 后自动 B Connected；没有用额外 UIA/Refresh 冒充终态自动通知。A/B 用量种子原本同 41 requests/841 tokens，不凭相同数值证明隔离；数据隔离使用 `HOST_A`/`HOST_B`、草稿、请求路径与 authority。

合成凭据修复显示缺失→输入掩码→保存后提示消失→重开空白；UIA 无 Value/Text 读取 pattern，但 **IsPassword 为 false**，不声称完整 accessibility 验收通过。运行证据四处合成 canary 是捕获后才 mask，[redaction 收据](../../../../outputs/ssh-checks/205/completion/run-20261116-230934/authorized-20261116-234754/native-S1/run-20261117-014949/redaction-receipt.json) 保留前后 hash；fixture 源码合成字面值有意留作复现。真实凭据未使用，不将本轮写成“所有日志始终落盘前脱敏”。[精确清理](../../../../outputs/ssh-checks/205/completion/run-20261116-230934/authorized-20261116-234754/native-S1/run-20261117-014949/cleanup-receipt.json) 后 TEMP nonce 根 140 文件已删、10 个登记进程均独立确认不存在，全部工具/exe/lock 与图片 hash 已核。

### S2/S3：真实资源、失败终态与管理恢复

| 阶段 | 已独立验收范围 | 实际边界 |
| --- | --- | --- |
| S2 | [60/60 收据](../../../../outputs/ssh-checks/205/completion/run-20261116-230934/authorized-20261116-234754/coordinator-S2-acceptance.json)：direct 28、authentication 25、rotation 6、credential cleanup 1；真实 Windows 系统 keyring、三认证、信任/密钥轮换、HOME、SFTP CAS/完整备份/原子替换及恢复、顺序双端点。 | 两个逻辑端点位于同一物理 Linux 机，不代表两物理机或其他 OS。应用取消/断线后真实父子孙仍存活，独立管理通道按 nonce/PID/starttime 精确恢复后才证明退出。 |
| S3 | [新 nonce 独立 66/66 收据](../../../../outputs/ssh-checks/205/completion/run-20261116-230934/authorized-20261116-234754/coordinator-S3-acceptance.json)：真实 llmusage 2.1.0 投影 1 event/120 tokens；WAL、平台与 7/30/90 日期、schema/link/integrity、NDJSON、取消、私有快照与清理。实施者亦为 66/66。 | 真实 CLI/Python/SFTP 数据与时序/依赖/网络/cleanup fault shim 分层。原 Tauri owner 和 S1 GPUI owner 分别记证据；静态合成 seed 留在 ignored 证据目录，不是用户数据或运行时 cache。 |

取消标志、channel 关闭或本地进程数都不能证明远端树退出。真实树仍活时应用返回 fail-closed `CleanupFailed`；失权快照不借新 app session 下载、发布或清理。只有后来授权的独立管理通道，依据精确 nonce、PID/starttime 和对象清单，证明遗留已回收；**管理恢复成功不等于应用已杀进程树**。S2 本轮根、凭据/keys/DB/trust/locks、server/manager 与依赖均精确回收；S3 私有 Rust/Cargo/依赖也全部回收，新独立 run 删除 24,285 对象，server/installer exit 0，独立查询 server/manager 身份均消失，本地 key/cache/trust/locks 均不存在。

S3 协调者首次独立 `run-20261117-011806` 在 live 前因响应文件 PermissionError 失败，原日志保留；cleanup/service exit 0、精确 root/PID 消失已有独立证据。ignored 桥修复只在同 UUID/原 deadline 重试文件操作，不重发 mutation；随后新 nonce `run-20261117-013821` 才 66/66 通过。首次空 bootstrap `004025` 的本地 receipt 异常有成功 cleanup 调用路径依据，但没有单独精确路径二次查询，这个证据局限继续保留。

Windows 深 264 字符 output cache 触发既有 ACL error 123，短 142 字符路径可用；本轮采用短 TEMP 隔离，未扩改通用 writer。Pageant、未实测私钥变体、其他 native OS、真实远端 UI 及全局物理 pointer 未验证，不能推及这些范围。WebDAV 仍是用户排除项，既不计 passed 也不计本轮阻断。

### S4、S5 与后续唯一入口

[S4 单一只读报告](../../../../outputs/ssh-checks/205/completion/run-20261116-230934/authorized-20261116-234754/S4/review.md) 对 `b3f0dbab…3ebfbada` 的 Standards 0、Spec 0，[协调者核销](../../../../outputs/ssh-checks/205/completion/run-20261116-230934/authorized-20261116-234754/coordinator-S4-acceptance.json) 确认无产品退修；审查者未派生、未改文件、未执行新门禁。现有 [execution-environment 契约](../../../spec/ccr-app/backend/execution-environment.md) 已准确包含失权清理、离线 retained owner 和按当前环境展示的边界，S5 不重复新增规则。

S5 仅更新本主题并追加新 journal/index，保留旧 Session 177。按当前 help 执行 recorder `--no-commit --topic` 时，既有 journal-1 的两条 Session 28 使其在写前 exit 1，journal/index 零写；保留失败收据及旧记录，核写前 hash 后以明确的 append-only 手工 fallback 追加唯一 Session 178 和对应索引，不冒称脚本成功。只 stage 本组必要文档，五个原脏文件字节和 dev/main 保持，输出/截图继续 ignored、不 force-add。文档读回、真实相对 Markdown 链接、scoped diff、秘密/路径审计、fallback 与提交核验落在当前证据根 `S5/`；全局原 AGENTS EOF 空行不在本组修复范围。

协调者下一步在所有 tracked 提交后冻结 HEAD，运行最终完整 `just ci`、只读复核 GitHub/保护 refs/五原脏/署名，然后生成最终集成与归档收据。当前没有未确认的本轮运行设施遗留；上述未测/证据限制仍需如实报告。#205 保持 OPEN，不把本次文档提交写成父任务验收完成。

## 2026-11-16 原轮次历史（仅为当时事实）

以下保留原轮次记录；其资源缺失、工具阻断、未改历史和元数据阻断描述仅适用于 2026-11-16，当时尚未获得后来授权。当前状态由前节及当前证据根收据覆盖，不静默改写原失败。

日期：2026-11-16（执行时本地日期）。范围：[实施规格 #205](https://github.com/bahayonghang/ccr/issues/205) 的本轮本地验收分类与归档；GitHub Issues 继续作为任务源，mini-trellis 不驱动任务。

本轮没有新增产品、测试、依赖、执行脚本或门禁改动，只整理归档和 journal。S1–S4 的交付与协调者核销已完成；**父任务验收未完成，#205 保持 OPEN，#206–#214 保持 CLOSED**。最终提交、工作区、GitHub 状态及归档终态由协调者在本地提交后核验，以 `outputs/ssh-checks/205/finalization/final-integration-receipt.json` 和 `archive-receipt.json` 为准；本文不提前声明这些收据已生成。

### 代码交付与验证基线

源码与既有测试/CI HEAD：`173c9f0b6d75cff5179a3942c90d3b4f586ef0fa`。以下是本轮开始前已经完成的 A/B/C，不是本轮新增提交：

| 既有提交 | 交付 |
| --- | --- |
| `de174ad8` | 上传祖先链接检查纳入原取消与 10 秒预算；生产入口内存 SSH/SFTP 回归。 |
| `d776bad8` | 安装取消清理期间保留 Busy/终态；状态观察不授予新执行权限，清理中的旧 owner 不被重连替换。 |
| `173c9f0b` | 仅新增在途 helper 失权重连组合测试，验证不借用或撤销新会话、不下载/发布缓存；没有新增产品修复。 |

协调者和 S4 已核对源码 manifest、原始日志哈希及退出标记。本轮明确**复用**同源码 HEAD 的以下结果，没有重新运行整套门禁或 CI；文档提交 HEAD 另见最终收据：

- 根 6 项：`just version-check`、`just fmt-check`、`just lint-strict`、`just test`、`just audit`、`just ui-check`，全部 exit 0。
- desktop 5 项：`just desktop-fmt-check`、`just desktop-lint`、`just desktop-test`、`just desktop-test-ui`、`just desktop-audit`，全部 exit 0。
- 完整 `just ci`：exit 0，1245.8 秒，17 阶段通过。来源为 [旧 CI 结果](../../../../outputs/ssh-checks/205/next-session/final-ci/results.json)；复用依据及原始日志路径见 [本轮复用收据](../../../../outputs/ssh-checks/205/finalization/reuse-receipt.json)。

旧 B 首次 audit 因 RustSec 拉取 TLS EOF 失败、同 HEAD 窄重试通过的事实和原日志均保留，不改写为首次全绿。本轮最终 CI 修复轮为 0/1；与更早轮次的修复预算分开记账。默认并行、独立 binding 导出、预算/容量/阈值及既有审计忽略未改变。

### 本轮逐组结论与证据层级

| 阶段 | 本轮实际结果 | 证据 |
| --- | --- | --- |
| S1 原生窗口 | 9 项 `blocked_tool`；其中同主机重连、远端安装、每主机连接另有 `blocked_fixture`。初始化后首次 `list_apps` 报 native pipe `os error 2`。应用启动、窗口输入、新截图和截图查看均为 0。 | [S1 验收](../../../../outputs/ssh-checks/205/finalization/S1/run-20261116-222816/acceptance.md) |
| S2 SSH/凭据/文件/进程 | 7 项及子项全部 `blocked_resource`：三类认证/keyring、信任、HOME、SFTP、两主机、断线重连、真实进程树。没有授权资产，未扫描或连接。 | [S2 验收](../../../../outputs/ssh-checks/205/finalization/S2/run-20261116-223151/acceptance.md) |
| S3 用量/快照/WebDAV | 4 组、31 子项全部 `blocked_resource`：远端 CLI/Python/schema/流生命周期、WAL/平台/日期、WebDAV、私有快照。未运行本机 CLI 冒充远端、未新建 WAL/cache。 | [S3 验收](../../../../outputs/ssh-checks/205/finalization/S3/run-20261116-223525/acceptance.md) |
| S4 单一只读审查 | clean review，零产品发现、零 tracked 修改、零新门禁；覆盖与复用依据成立。 | [S4 审查](../../../../outputs/ssh-checks/205/finalization/S4/run-20261116-224008/review.md) |

已有 fixture/mock、本机 SQLite 和 fake DAV 仅为本地回归证据；A 的内存加密 SSH/SFTP 使用 `tokio::io::duplex`，不是 TCP、外部主机、真实文件系统或系统凭据库验收。旧截图不绑定当前源码，本轮没有复用为新截图。

S1 只做静态隔离路径审计，没有完成动态准备或启动。普通 fixture 不自动发现本机 Insights，但两个 catalog 参数不足以证明所有页面已经完成动态隔离；普通启动也没有可连接 SSH/远端安装注入入口。不把这些验收设施缺口写成产品失败或静态审计通过即原生通过。

### 自主决策、失败事实与恢复

- 原生管道失败后不重复同类探测、不修改系统设置、不自建 UIA/管道替代控制器，也不扩产品 fixture 只为截成功图。
- 缺资源逐项阻断，不读取保存主机、agent、keyring、个人配置或真实私钥推定授权。复用充分覆盖的旧协议/fixture 证据，不重复 mock，不为制造提交寻找缺陷。
- [执行环境契约](../../../spec/ccr-app/backend/execution-environment.md) 保持准确：SQL/backup/校验由 `ccr-usage` 持有，协议/加密/冲突由 `ccr-sync` 持有；session authority、状态保留、失败终态和既有范围限制不扩展。
- 本轮外部连接、读写、WebDAV 请求、凭据变更、远端进程、快照路径、缓存及外部对象均为 0/空清单。前像和恢复验证为不适用，**不是恢复通过**；没有本轮未确认恢复对象，历史外部清理未验证项继续保留。
- 初次恢复 wrapper 的 GBK/UTF-8 解码失败及后续独立重跑、S4 日志匹配器大小写误判及窄更正均保留原记录，属于证据工具问题，不是产品/门禁失败或 CI 修复轮。
- T3 重启取消原 S5；协调者先确认无主题/journal/提交/Git 锁/相关在途命令，再恢复未写入的 S5，未重做 S1–S4。见 [恢复检查](../../../../outputs/ssh-checks/205/finalization/restart-20261116-2243.json)。

### 未通过与后续入口

原生交互/浅深截图、真实 SSH/OS agent/keyring、SFTP 权限与原子替换、两主机顺序切换、远端 CLI/Python/schema/WAL/日期对照、外部 WebDAV、真实 nonce 进程树退出、精确私有快照清理及跨 OS 验收均未完成。获得明确授权的独立资产后，沿用 [真实资源清单](../../../../outputs/ssh-checks/205/next-session/real-host-checklist.md)；先记录前像和精确对象，逐对象验证恢复。

真实远端树需要本轮 nonce 绑定的父子进程身份与退出证据；取消标志、本机进程数或 channel 关闭不能证明退出。旧会话私有对象仅可由授权管理通道依据精确路径、原 session UUID/所有权处理，不借新 session 自动补删、不通配清理；无法确认继续保留 CleanupFailed/遗留。

历史 `303c898e`、`12e741a2`、`5d908cb6` 的个人 author/committer 元数据发布阻断仍在，只记录合规布尔，不复制真实值、不改写历史。因此不能宣称全部公开发布条件满足。

本轮归档定位是“本轮本地实施记录”，不是父任务完成。未推送、未建 PR/合并、未移动 dev/main、未改历史或全局 Git/主机设置。`outputs/` 和截图保持原忽略策略，原始证据只在本机保存；新 checkout 不包含这些忽略输出。协调者的最终收据负责落定“本轮本地实施已归档，父任务验收未完成”或有失败时的实际状态。
