# T10 专职复核报告

状态：专职复核完成。当前 a31c 冻结上的 Windows 完整 just ci、Linux root quality/audit/coverage/MSRV、Tauri aggregate/coverage 与 native7 限定验收均已通过并核验。4 份最终矩阵、恢复结果和历史证据已独立核对。macOS 矩阵及修复 commit 条款仍未满足；本报告不授予父任务整体通过。

## 范围与独立性

复核者为 /root/check_t01（trellis-check）。范围为父任务 09-28-cli-tauri-architecture、T01–T11、39 项子 AC 和 9 组 P1。初始源码复核涵盖责任路径与代表性断言；后续源码变更使用独立增量审查与对应版本的执行记录。

T01 包含本代理此前修复及自检，不能称为完整作者独立审查。T02–T09/T11 的初始源码检查为非作者复核。DC-02 权限修复由本代理编写，root 承担该六文件增量的非作者审查。DC-03 路径测试修复由 remaining_gates 编写，本代理完成非作者复核。复用的 remaining-independent-check.md/json 保留其作者曾编写早期 Doctor deadline/readiness 的边界。

初始审查未执行 Cargo。DC-02 授权窗口内，本代理执行配置/core 定向测试、Clippy、check、rustfmt 与限定 diff-check。完整 CI、exporters、bindings、构建和原生浏览器均由 root 执行；本代理核验其回执。未提交、推送、归档或修改任务生命周期。

## Findings (fixed)

- DC-01：typed-ipc-bindings.md:148 将已删除 Vue install dialog 路径更正为实际 React 路径 ccr-ui/src/features/usage/components/LlmusageInstallDialog.tsx。限定 diff-check 通过，修复已纳入源码冻结。
- DC-02：profile repository 的语义不变 mutation 原先提前返回，跳过 Unix secret 权限修复。现通过共享 leaf 锁、同一 file handle 和版本校验修复 mode；不替换内容、不创建备份。0400/0600 保持，过宽或特殊位按既有 secret policy 处理。原 base.rs 红例未修改。
- DC-02 作者边界：本代理编写六文件增量，root 已非作者审查。新增 helper 不添加 content rollback entry；已有 entries 保留原 metadata 恢复政策。不能宣称所有回滚都不会恢复较宽权限。注入故障不等于实测 OS chmod 拒绝。
- DC-03：providers.rs 单测试的 Windows backslash 成功断言按平台拆分。全平台 native/canonical 成功断言保留；Windows backslash 继续成功；Unix 显式验证路径不存在及 restore 拒绝。单测试以外原始字节等于 HEAD，生产代码未变。
- root 的八进制字面量修复：Permissions::from_mode(0) 改为 Permissions::from_mode(0o0)。原字节、唯一替换、当前字节和换行均独立核验，数值仍为零。
- 原生报告语义修正：styles_after_save 读取 style CSP 事件。当前派生报告改为 post_save_style_csp_violations_empty；旧报告用独立 correction artifact 更正过度推断，raw6/raw7 与旧派生文件 SHA 保留。

## Findings (not fixed)

- 必需 macOS 进程矩阵未运行。Linux WebKitGTK、Windows 测试或 cfg(unix) 源码分支不构成 macOS 执行证据。
- 历史 exporter 0xc0000005 的原因未查明。后续成功或 Doctor PATH/耗时问题不能解释历史访问冲突。
- 没有获准提交，fix_commit 保持 null。源码、diff 和执行指纹提供追溯，但不满足字面的修复 commit 条款。
- 原生验收未测保存后的 style DOM 移除或 body scroll-lock 还原。该限制已记录，未扩大当前验收范围或补写未执行结论。

## Verification

| 当前 a31c 检查 | 结果与范围 |
| --- | --- |
| Linux workspace quality | exit 0，224.101 秒；version/governance/fmt/strict Clippy/tests/workspace check 全部完成。41 个 harness 结果合计 1869 passed、16 ignored、33 filtered。 |
| Linux audit | exit 0，3.849 秒；cargo-audit 0.22.2 安装日志、当前可执行文件 SHA 和版本匹配。 |
| Linux MSRV | exit 0，4.96 秒；实际 rustc 1.95.0，locked workspace all-targets/all-features check。版本依据为记录的 rustc 输出。 |
| Linux root coverage | exit 0，213.25 秒；独立重算总体 74554/99931 = 74.60547777966798% ≥70%；core gateway 393/409 = 96.08801955990221% ≥85%。 |
| Linux Tauri aggregate | exit 0，216.576 秒；6 governance、fmt、desktop-bin check/strict Clippy、397 behavior、2 guards、230 exporters、1 inventory；1 helper ignored。 |
| Linux Tauri coverage | exit 0，166.002 秒；gateway 654/703 = 93.0298719772404% ≥85%。总体 17964/34456 = 52.1360575806826% 仅为 baseline，该 recipe 无总体门槛。 |
| Linux native7 | build exit 0，38.995 秒；六项限定验收通过，进程组清理记录为空。 |
| Windows 完整 just ci | exit 0，1078.351 秒；14 阶段通过，407 Tauri behavior；前端 smoke 与 coverage 各 169 文件/904 测试，docs/full frontend 与 VSCode CI 通过。 |
| Windows 前端行覆盖 | 独立重算 13081/17395 = 75.19977004886461% ≥70%；CI 显示 75.19%。72.92% 为 statement 覆盖，不能替代行覆盖。 |

8 个当前执行回执的日志 SHA、269 项源码 before/after 与 a31c 冻结一致。Windows 完成后重新读取 267 文件和 2 删除标记，并比较 230 个 generated 的完整路径集合及原始字节。4 个 schema 原字节、保留的 Linux schema 副本和移除状态均通过最终检查。零匹配 target、ignored 与 filtered 未计为通过。

## 源码版本与历史归属

| 冻结 | 路径数 | 归属 |
| --- | --- | --- |
| d8ee10 / remaining-final-source-freeze.json | 268：266 文件、2 删除标记 | 初始最终版；旧 Windows ci、旧 Linux Tauri/coverage 与 native6 属于该版。 |
| db558b / remaining-permissions-source-freeze.json | 268 | 相对 d8ee10 仅六个授权权限修复文件变化。五份最终定向回执绑定此版。 |
| 49801f / remaining-portability-source-freeze.json | 268 | 仅增加等值八进制测试字面量修复；随后 root quality 在 ccr-store 测试失败。 |
| a31c / remaining-platform-tests-source-freeze.json | 269：267 文件、2 删除标记 | 原 268 项未变，既有 providers.rs 新增进入冻结范围；并非新建生产文件。 |

当前完整冻结 SHA 为 a31c8730384c91076a051f226d3201e1b6cd2d9efa7ec63f881b8899601db273。旧三份冻结 SHA 均核验未变。原 providers.rs 不在旧 268 项内；新的绑定红例提供新增执行证据，不回填旧运行的源码身份。

原 root coverage exit 101/194.99 秒因 DC-02 失败，未到阈值阶段。其结束快照与 Windows bindings 事务重叠，记录 7 个 generated 源码路径差异和 201 项 generated 差异，不能称为零漂移。独立红例保留，后续 Windows 恢复已单独核验。

权限修复后先出现 Python 入口缺失（exit 127/6.372 秒），再出现 Unix 八进制 Clippy 错误（exit 101/29.196 秒）。49801f quality 随后在 ccr-store 测试失败（exit 101/186.517 秒）；单测试原源码 exact 红例 exit 101，修复后 Linux/Windows exact 与 Linux lib 通过。所有失败回执均保留。

旧 Windows ci exit 0/930.453 秒、旧 Linux Tauri/coverage 和 native6 保留 d8ee10 归属。native attempts1–5 的失败、旧执行 binary SHA 和后续 binary 替换观察记录均保留；没有用当前字节替代历史执行身份。

## 原生证据边界

native7 六项检查覆盖 native boot、明文确认、scroll-lock nonce、CodeMirror 有效样式、生产 CSP 拒绝无 nonce inline script，以及 WebDriver 编辑保存与磁盘 JSON 精确比较。独立读取时磁盘 model=haiku，env 和 unknown 对象保留；保存后仅确认 style CSP 事件为空。

截图已独立目视，显示保存前 sonnet 源文件编辑器、语法着色和对齐 gutter。保存结论来自输入动作与磁盘比较，截图不能单独证明保存。binary SHA 为 7b72d615f1909baece92bc60c9c327daf9eacbbd692e73dc0ba5797e462c8918，保存文件 SHA 为 b57aed982cf108e3ee2f9e2f153fdaa68a2e00c47a62411d1ae00d403235db10，均已独立读取核验。

harness 归一化换行后的逻辑文本仅扩展 build receipt 白名单。原始字节同时由 227 LF 转为 227 CRLF；两份执行保留各自原 SHA，未重写 harness 或重跑 native。830 个非 generated 前端输入当前匹配旧 UI 构建；230 个 generated 输入匹配 native 构建稳定快照；1060 个输入前后一致，284 dist 与 native 构建前后一致。CSP config SHA 另行核验，不声称该未变配置属于 269 个变更范围路径。

范围限定 Linux WebKitGTK/WSLg、debug automation binary、生产 custom-protocol/CSP、合成 Local 配置与 WebDriver 输入。环境目录重定向不构成 OS 文件系统沙箱。不覆盖 Windows/macOS WebView、物理输入、发布安装包、真实 SSH 或 provider OAuth。

## 逐任务源码复核

下表记录初始源码责任路径与代表性断言；后续 DC-02/DC-03、Doctor fixture 和共享 nonce 增量另有来源和执行记录。条款原文、源码 SHA 及完整引用保存在同名 JSON。

| 范围 | 当前责任路径与代表性断言 | 结论边界 |
| --- | --- | --- |
| T01 AC1–AC3 | repository 同源读取、跨进程锁、CAS、unknown TOML、current/default；原检查报告和真实双进程 fixture | 先前作者范围，保留自检标记；最终执行另验 |
| T02 AC1–AC5 | `profile_lifecycle.rs:201/223/258/385` 统一预检、enable、journal、update；`profile_contract.rs:82/108/200/321` 逐写故障、三 adapter、幂等重放、rename 指针和未知字段 | apply 覆盖 Claude/Codex/Grok；已批准 update/rename 迁移为 Claude/Codex；非 OS 级多文件事务 |
| T03 AC1–AC3 | `commands/config/adapter.rs` 显式 Claude、严格对象/absent/null、active 保护、CAS、application 委托；真实 handler fixture 与配置页底层 IPC mock 分层 | 保留两层证据，不称为 native UI 全链路对照 |
| T04 AC1–AC3 | `validate_service/diagnostics.rs` 类别和 severity；`ccr/src/main.rs` 在 logger 前分派只读诊断与退出码；`diagnostics_contract.rs` 二进制结果、字节/mtime、拒绝读取 | 实际 binary fixture 支撑退出码；当前域规则与 apply 共用 |
| T05 AC1–AC4 | pending store 的 `secret:true/BackupPolicy::None`、私有序列化、错误脱敏；原 writer/journal 审查和混合备份/权限 fixture | Windows DACL、Linux mode 与注入权限错误分开；macOS 未运行 |
| T06 AC1–AC4 | `usage_jobs.rs:236/265/291` admission、cancel_requested、唯一 complete；`cli.rs:108/187` deadline 和共享 cleanup 预算；barrier 断言 spawn=0，五种终态不回退 | fake process、真实 Windows/Linux 子进程和 stdout/stderr 上限分别记录 |
| T07 AC1–AC3 | shell store 保留 job；同步 submitting；job-ID、序号和终态合并；history pending/saved/failed；路由重挂载和迟到 listener 断言 | history 为每 job 至多一次写入尝试；无后端幂等键，不宣称跨 IPC exactly-once |
| T08 AC1–AC3 | dirty leaf patch、notifications union、unknown enum/managed 锁、平台共享 raw editor、CAS/确认、Grok 无备份；真实 Codex helper 磁盘往返 | 前端底层 IPC mock 与 Rust 持久化各自成立；当前 a31c 的 Linux native/CSP 第 7 次执行已核验通过；第 6 次保留 d8ee10 历史归属；Windows/macOS WebView 未覆盖 |
| T09 AC1–AC3 | Auth probe/load/stale/off 状态；Settings snapshot/draft、环境身份/世代、旧响应拒绝；Configs 翻译订阅 | 保留既有非作者复核及本轮源码检查；不外推真实 SSH/WSL 远程主机 |
| T10 AC1–AC4 | 三平台 aggregate 调用 `tauri-ci`；strict Clippy、全行为测试、bindings/inventory；事务恢复原生成字节；实际 aggregate 图注入失败；命令计数和 usage owner 索引 | 模拟 Cargo fixture 只证明失败传播；正式结果必须来自执行记录 |
| T11 AC1–AC4 | registry 23-ID 生命周期分类；真实 owner admission；OAuth bind/save 后发布、截止时间、commit 不可伪取消、唯一 cleanup/terminal | Local synthetic OAuth/socket/HTTP；不代表真实账户登录；macOS 保持开放 |


## 映射与保留检查

最终冻结矩阵独立核验通过：39 个 AC ID/原文保持初审与 11 个 PRD 的逐字对应；48 组/164 个 selector 的当前源码 SHA 和准确行位置匹配；9 组 P1 的 20 个唯一原反例保留；86 个运行记录和 393 个仓库 artifact SHA 匹配。41 个历史运行对象的 canonical 指纹及 7 份源码冻结文件 SHA 未变。8 个当前执行记录均为 exit 0，并绑定 a31c 的 269 路径 before/after。初审的 42 组/148 selectors/166 artifact 对仅保留为历史计数。

最终保留检查重新验证 4 份原 schema 字节与 Linux schema 副本、12 次 task validate 的退出结果、24 份当前 context JSONL 哈希。父任务及 11 子任务均保持 in_progress，两项 Insights 分别保持 planning/in_progress。validator 报告 dependency-governance.md 为 33513 字节，超过 32768 字节注入上限；此前已读取完整文件，未改写冻结规范。

矩阵的 ledger_status_at_capture 与 dedicated_review.status_at_capture 保留捕获时状态。当前结果由 continuation_final_evidence 与本报告给出。两条 Linux /tmp 保存文件引用属于运行时命名空间，另有 native 独立读取证据，不计入 393 个仓库 artifact。

## 复核记录

主要结构化证据：remaining-dedicated-check.json；权限增量 remaining-permissions-fix.md/json 和 remaining-permissions-independent-review.json；平台测试 remaining-store-path-independent-review.json 与 remaining-store-path-red-independent-review.json；当前门禁 remaining-after-path-gate-review.json、remaining-after-path-coverage-msrv-review.json、remaining-after-path-tauri-ci-review.json、remaining-after-path-native-coverage-review.json、remaining-windows-final-independent-review.json；最终矩阵 remaining-final-matrix-independent-review.json。

本报告于 2026-09-29 UTC 完成终审。未发现新的源码问题。macOS、fix_commit、历史访问冲突原因与原生测量边界保持开放；没有提交、归档或更改任务状态。
