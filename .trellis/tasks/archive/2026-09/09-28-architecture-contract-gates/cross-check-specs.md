# T10 规范与证据独立交叉审查

日期：2026-09-28。审查者：`/root/implement_t10_gates`。被审实现 owner：`/root/implement_t10_specs`。审查者负责门禁代码，未编写本报告审查的规范、code map、验收矩阵或 P1 映射。本文记录独立源码核对、证据核对和 root 新增集成修复的补充审查。

专用 T10 checker 派发因 `agent thread limit reached` 失败。主会话授权分离所有权的交叉审查。本报告不表示专用 checker 派发成功，也不授予父任务最终验收。

## 结论

规范与证据产物通过本范围复核。8 个规范与导航文件、39 个唯一 AC、33 个测试证据组、9 个 P1 映射均有对应内容。独立检查核对 128 份唯一产物的路径与 SHA、128 个测试选择器的源码行、25 条相对规范链接，未发现不一致。T06 与 T10 的 `implement.jsonl` / `check.jsonl` 均通过实际 `task.py validate`，未出现 32768 字节注入上限警告。

执行脚本：`research/cross_check_specs.py`。冻结快照、完整计数、路径、SHA、选择器和限制：`research/cross-check-specs.json`。上述计数对应最终 33 组矩阵的本轮复核快照；root 集成测试修正另见本文补充审查、`cross-test-env-guards.json` 和 `cross-tui-a14-review.json`。本轮已核对矩阵中的源码与原始日志指纹。脚本只读取被审源码与证据，写入自身审查结果和 context 验证日志；没有执行 Cargo、exports、原生 UI 或修改任务状态。

## 已修正问题

| 发现 | 作者修正及独立复核 |
| --- | --- |
| 本地 aggregate 描述未准确涵盖三平台 desktop gate | 规范明确根三平台 `ci` 复用现有 `tauri-ci`，覆盖 strict Clippy、全量行为测试、bindings 和 inventory。普通行为/coverage 仅跳过 `export_bindings`，保留默认并行。 |
| 绑定 check 的只读语义可能被外推到根 CI | 规范明确根 `ci` 的 version-sync / fmt 仍可能写文件；只有 bindings check 在所有结果下恢复调用前原字节。 |
| 生命周期规范仍建议直接 Cargo exports | 改为事务入口 `just tauri-bindings` / `just tauri-bindings-check`。未改变 normalizer 算法。 |
| dependency-governance 和 provider-adapter 超过 context 注入预算 | 作者压缩前者，并把 usage job 生命周期抽为独立规范。当前文件均不超过 32768 字节；T06/T10 context 真正可加载。原 analytics、pricing 与 Home Insights 内容保留证据单独记录。 |
| UI 版本标签消费者位置不准确 | Good 示例改为实际导入 appMeta 的 `MainLayoutChrome.tsx` / `Titlebar.tsx`；没有继续把 MainLayout.tsx 描述为直接消费者。 |
| T10 AC2/3/4 无独立 artifact 证据入口 | 矩阵新增相应 artifact_evidence，分别为 6 / 4 / 7 项。Python suite 的 `Ran 27 tests` / `OK` 进入结果提取。 |
| A14 映射仍保留旧 raw-log 缺失状态 | 已采用 root 新保存的原 mapper 失败执行、原始日志和基线 SHA，明确 instrumentation 只改 helper runtime import。未将 mapper 反例外推为完整原生链验收。 |

## 源码和证据核对

- 当前 manifest 为 base 340、Windows 348、typed 278、exact-wire 278；registry 记录 38 个基础模块。规范引用权威 registry / manifest，旧 336 / 315 / 323 描述不再作为当前计数。历史失败例仍可保留历史路径。
- `crates/ccr-usage` 的 SQL 和只读 SQLite projection 所有权在 root code map、UI code map、backend index 可找到。Desktop adapter 只负责已安装 CLI 同步、NDJSON、DTO 和错误映射。
- 39 个 AC 的测试组引用全部可解析。各组源文件 SHA、实际选择器所在行、原始日志 SHA 均匹配。报告路径存在。不同测试组的 mock 层与平台边界保留，未把静态计数当作运行时行为证据。
- 9 个 P1 都有原问题、原版反例、修后行为测试及来源映射。所有 `fix_commit` 为 null；没有虚构提交。`final_mapping_status` 明确保留 root 最终验收 pending。
- A14 独立对照 `git show 34d8a85e0e48b793733835e0304c8ed33940fcee`：probe mapper 只将 `@/configs/settings-helpers` 改为同基线 helper 的相对导入；helper 与原 blob 一致。原始执行 0 passed / 1 failed：仅修改 model 时 notifications 数组变成 false。
- A09 metadata 中两个 gateway 原 SHA 分别对应同一基线 blob 的 LF 和 CRLF 字节。已核对该差异；不将换行差异表述为实现差异。silent-child 的实际执行、短 deadline 和显式取消/reap 证据由 root 保存。
- protected `.tmp-desktop-probe.mjs`、`.tmp-insights-visual.mjs` 的 SHA 未变化。既有 Insights 生命周期由 spec-validation 分别记录，未用新架构任务验收覆盖旧任务状态。

## root 集成修复补充审查

### clean 缺配置处理

修改位置：`crates/ccr-cli/src/commands/lifecycle/clean.rs:132`。match 只把 `ConfigMissing` 转成 `configured_skip_confirmation = false`；其他错误在 BackupService 构造与删除前原样返回。`!dry_run && !skip_confirmation` 的确认分支保留。Clap 的 `CleanBackupsArgs` 真实支持 `--force`。

初版新增损坏配置测试只断言非零，可能被提前错误误满足。已向 root 要求明确错误分类及执行路径。root 补充 `CONFIG_FORMAT_INVALID`（exit 14）、stdout 的执行标题、stderr 的配置格式错误、原备份存在及坏配置原字节断言。源码错误链已核对：`ConfigService::load_config` → repository snapshot → `parse_config_from_str` → `ConfigFormatInvalid` → binary 错误出口。

root 最窄重跑记录 `research/root-clean-integration-fix-strengthened.json` / `.log`：13 passed、0 failed，包含该加强测试。审查者读取原始日志，未重复启动 Cargo。首次集成失败仍保存在父任务日志。

### dispatch 只读测试

修改位置：`crates/ccr-cli/tests/dispatch_routing.rs`。原成功用例依赖读取时自动初始化，与 T01 的纯读合同冲突。新成功用例显式写入 synthetic profile，并比较执行前后目录清单和文件字节；新增缺配置用例要求精确 `ConfigMissing`，同时验证不创建配置。实际 `list` 分派入口和环境隔离已核对。

root 首次最窄重跑 `research/root-dispatch-integration-fix.json` / `.log`：12 passed、0 failed。补充审查发现 inventory 以空 Vec 同时表示空文件与目录，无法证明两者类型不变。root 已将快照改为 `Option<Vec<u8>>`：文件为 `Some(bytes)`，目录为 `None`。审查者已独立读取该实现，类型混淆问题关闭。后续 `research/root-final-dispatch-typed-inventory.json` / `.log` 实际重编译运行 12 passed、0 failed，包含成功 list 与缺配置无初始化两个用例；新增类型断言验证项关闭。旧日志单独保留。

### Tauri 跨 crate fixture 环境锁

root 的首次完整 `just tauri-ci` 在 desktop 行为 suite 得到 401 passed / 5 failed / 1 ignored。原始日志 `../09-28-cli-tauri-architecture/research/root-final-after-fixtures-tauri.log` 显示 Claude rename 的 before 快照来自 `.tmpC9mV1h`，after 又引用 `.tmpZmRPDP` 的 CCR_ROOT。新增共享 fixture 只持有 CLI 自己的环境锁，未与既有 desktop 测试使用的环境锁互斥。

root 为 `profile_lifecycle::application_contract` 的 7 个入口，以及 Claude / Codex 的各 1 个 rename 合同入口增加外层 `crate::test_support::lock_env()`。审查者逐入口核对 9 处覆盖，并检查两套锁的实现、fixture 恢复、回调及现有 T03 双锁路径。

- 九个入口先取得 desktop `ENV_LOCK`，随后由 `TestHome::new_with_home_env` 取得 CLI `ENV_LOCK`。命名的 `_desktop` guard 保留至共享合同函数返回，未提前 drop。
- CLI `TestHome::Drop` 先恢复进程变量，随后字段析构释放 CLI guard。desktop guard 在共享 fixture 已完成恢复和销毁后才释放。
- 三个被合同调用的实际 adapter / helper 没有调用 `lock_env()` 或 `TestProcessEnv::new()`；CLI fixture 不依赖 desktop 测试支持。审查范围内未发现反序获取或同一非重入 Mutex 的重复获取。
- T03 `fixture()` 保持 desktop → CLI 顺序；返回 tuple 的第一项是 TestHome，析构时先恢复 CLI fixture，再释放 desktop guard。其 `enable_lifecycle` 测试也先取得 desktop guard。
- 新规范 `profile-application-lifecycle.md` 的 Cross-crate behavioral fixtures 明确该顺序、恢复边界及默认并行要求。测试没有改成全局单线程，也没有删除行为断言。

本轮源码补充审查通过。复跑 `root-final-after-isolation-tauri.json` / `.log` 仍为 exit 1，404 passed / 2 failed / 1 ignored，不能宣称完整门禁已经通过。剩余失败已反馈 root：

1. Codex conflict 测试要求旧错误文案“已存在”，当前共享 owner 返回 `Invalid or existing profile target`，错误文案合同失配已由源码确认。root 已将断言改为与现行 `CcrError::ValidationError(...).to_string()` 精确相等，保留后续 profiles、secret store 和秘密值断言，未修改生产错误文案。
2. Codex `update_tests` 四项原来在闭包后手工恢复环境，断言 panic 会跳过恢复。root 已统一使用既有 `TestProcessEnv`，分别设置临时 `CCR_ROOT`、`CCR_CODEX_DIR`、`CCR_LOCK_DIR`，移除手工恢复 helper 和额外 `lock_env`。审查者确认四项均没有重复获取 desktop Mutex，恢复发生在 guard 释放前，覆盖正常返回和 Rust unwinding。
3. Grok 的 `rename_updates_real_inactive_and_active_profile_state` 原来未持 desktop 环境锁。虽然 platform 使用显式路径，底层平台持久化仍经 `LockManager::with_default_path()` 读取 `CCR_LOCK_DIR`。root 已为调用 `test_platform` 的五个写入测试补充 desktop guard；第六个配置/原始保存测试原有 `TestProcessEnv`，未重复取锁。审查者确认六个 caller 全部覆盖。与其他 fixture 环境或目录清理竞争仍是 `os error 3` 的原因假设，具体失败交错未单独证实。

第二轮测试修正通过源码审查，root 后续最窄回归通过。审查者读取并核对 `research/root-tauri-profile-regression.json` / `.log`：41 passed、0 failed，包含九项共享合同入口及四项 Codex update；`research/root-tauri-grok-regression.json` / `.log`：16 passed、0 failed，包含新增锁覆盖的 Grok 写入测试及既有配置/原始保存测试。两组有重叠，不累加为唯一通过数；被 filter 排除而运行 0 个测试的 integration target 不计入通过。默认并行未修改。修正后的正式 `just tauri-ci` 已通过：406 项行为测试与 2 项 guard 测试通过，1 项 helper 忽略。最终 aggregate 仍待 root 的完整 `just ci` 结果，401/5 和 404/2 的旧失败日志保留。

审查者未运行 Cargo / exports，未修改这些测试或业务代码。锁覆盖与源文件快照记录于 `research/cross-test-env-guards.json`；root 已执行修复后的完整并行 Tauri suite；审查者核对其原始执行日志，未重复运行。doctor 超时用例的 PID 创建失败与未改源码原 exe 定向重跑 1 passed 单独记录，原因未查明，不合并为本项环境锁原因。

### T02 的 Grok rename 范围

按批准规划核对，T02 的共享 rename 迁移范围为 Claude / Codex。`09-28-profile-application-usecases/prd.md:32` 的 R5 和 `:40` 的 AC5 未枚举平台；`design.md:15` 对 R5 / AC5 明确规定“将 Claude/Codex handler 中的 load/patch/save/delete/apply 编排迁入同一 application lifecycle”；`check-report.md:70` 的 AC5 证据也明确限定 Claude / Codex application 与实际 desktop helper 的同一 harness。

当前 `ccr-ui/src-tauri/src/commands/grok.rs:718` 的 `update_profile_with_platform` 继续调用保存新名与 `finish_profile_rename`，保留 `RenameApplyFailed` / `RenameCleanupFailed` 具名部分结果。该路径不在上述明确迁移机制的范围内。本轮保留既有 Grok rename 机制与已批准设计一致，不需要为了泛化总结扩大迁移。交付报告应分别描述三平台 apply 的共享用例与 Claude / Codex rename 的共享用例，不能写成三个平台所有 rename 均已统一。

### Grok TUI 帧测试断言

root 的完整 CI 在 TUI suite 出现 245 passed / 1 failed。单项 `research/root-tui-frame-repro.log` 真实重现 0 passed / 1 failed。基线 commit 的 `grok_auth/tests.rs` 含相同 `Unsaved` 大写断言；`grok_auth/ui.rs:473` 的正常布局正文使用小写 `unsaved`，`:486` 的紧凑布局以大写 `Unsaved` 开头。先只修大小写后，`root-tui-frame-layout-repro.log` 又记录 SimplifiedChinese 140×40 的失败：Ratatui 宽字符 continuation cell 产生空格，`CCR` 与 `保存账号` 在弹窗正文跨行，整个 buffer 的原始连续子串无法匹配。两次失败日志保留。

最终修改仅在 `crates/ccr-tui/src/tui/grok_auth/tests.rs` 的确认状态断言。测试按 `draw_modal` 的实际宽度 `min(width, 78)`、顶部 y=3 和高度 `min(content_height, 16)` 提取弹窗内部，排除四边框；去除该区域的空白并转小写，然后分别要求保留账号警示与未保存凭据警示。将检查限定于弹窗正文可防止背景页面的“未保存”误满足条件。

两种语言、四种尺寸、六个状态共 48 个渲染组合保持；`Enter/n/Esc`、`q`、`personal` 断言保持。双语警示仍必须存在，只取消终端 cell padding、换行 padding 和大小写对语义查找的影响。`ui.rs` 无 git diff，生产布局、业务操作与确认逻辑未改。审查者未发现警示内容被删除或确认条件被放宽。

`root-tui-frame-normalized-fix.json` / `.log` 为 exit 0、1 passed、0 failed，执行完整该测试循环。`root-tui-frame-assertion.json` 最终 SHA 与当前源码相符；最初错误的 failure-log 相对路径已修正为可解析路径。机器复核与三次执行记录见 `research/cross-tui-a14-review.json`。后续 root 完整 `cargo test -p ccr-tui --all-features --lib` 的 `research/root-tui-full-final.json` / `.log` 为 exit 0、246 passed、0 failed、0 ignored。审查者核对原始日志，未自行执行该 suite。该结果不授予原生终端视觉验收或完整根 CI 通过。

### A14 Codex Rust 补充证据的范围

最终矩阵已增加第 33 组 `codex_settings_backend`，与 renderer mapper/domain/底层 transport mock 证据分别记录。审查者只核对以下真实 Rust 函数及断言，未修改矩阵或运行测试。源码均位于 `ccr-ui/src-tauri/src/commands/codex.rs`：

| 测试 / 位置 | 实际执行路径与断言 | 不授予的结论 |
| --- | --- | --- |
| `read_codex_config_accepts_tui_notifications_event_array`，2137 | 临时文件写入 TOML，真实 `read_codex_config` 读取；精确断言 Events 数组与 parsed `tui.other.notification_condition`；真实 `codex_settings_to_json` 的 notifications 投影仍为相同数组。 | 未断言 notification_condition 的 JSON 投影；未执行文件保存。 |
| `apply_codex_settings_update_preserves_tui_notifications_event_array`，2177 | 从字符串解析内存 config，只 patch `tui.animations=false`；真实 merge 后序列化，检查 animations、notifications 两事件、notification_condition、notification_method、status_line 保留。 | 没有磁盘写回或仅改 model 的后端用例；不能外推全部 unknown fields。 |
| `apply_codex_settings_update_clears_nested_fields_with_nulls`，2233 | 内存 config 的 sandbox/history/tools 叶字段显式 null，features 空对象；真实 merge 后四个对应 Option 均为 None。 | 没有序列化、磁盘读写、invoke 或原生 WebView。 |

三项为一项真实文件读取与数组投影、一项内存 merge 加序列化、一项内存清空 merge。相邻 boolean notifications 测试不属于该组三个选择器，不计入该组三例。前端仅改 model 的 mapper 反例与这些 Rust 证据组合属于分层验证，不能表述为同一次原生端到端保存。

父任务 `research/root-acceptance-after-tui-tauri.json` / `.log` 记录正式 `just tauri-ci` exit 0：406 项行为测试和 2 项 guard 测试通过，1 项 helper 忽略。197 项 Tauri exports 随后由专用 generator 执行，不计为额外行为测试。审查者在原始完整日志中逐条确认上述三个选择器成功，并核对最终 33 组矩阵的日志 SHA 一致。运行环境覆盖 `CCR_SKIP_ICON_GENERATION=1` 如 metadata 所记。分层证据不授予磁盘保存或原生端到端验收；根完整 `just ci` 仍由 root 负责最终验收。

### doctor 超时 fixture 的未解决基线

根 CI 的 `../09-28-cli-tauri-architecture/research/root-acceptance-after-tui-ci.log` 再次记录 `doctor_timeout_terminates_parent_and_grandchild` 失败：CLI suite 为 343 passed / 1 failed，`grandchild.pid` 未写入。root 只在 Windows fixture 增加临时 Start-Process 前后时间与异常记录；`research/root-doctor-cli-instrumented.json` / `.log` 的完整默认并行 CLI suite 为 344 passed / 0 failed。该复跑没有重现 PID 缺失，原因未查明。

审查者只读核对 `fix.rs:611-650`、`:1165-1186`、Windows fixture 与 `process_gateway.rs:120-136`、`:329-386`。生产 timeout 在 ManagedProcess 启动后开始；测试的 PID 等待与 20 秒 timeout 同时运行，`tokio::join!` 没有建立先取得 PID 再超时的屏障。fixture 在 Start-Process 返回后才写入子进程 PID。Windows 清理路径调用 TerminateJobObject、等待直接子进程并确认 Job Object 的 active_processes 为零。现有失败日志不足以判定该清理路径失败。

root 已撤除全部临时诊断修改。审查者用 `git show HEAD:crates/ccr-cli/src/commands/codex/fix.rs` 逐字节比较，当前文件相同，SHA 为 `2615ff8ef43b62d659b9e1604c56ece8161fd6552d93096b6ea07a751d2b667a`。没有保留生产代码或 fixture 修复。精确记录见 `research/root-doctor-diagnostic-outcome.json` 与本审查 `research/cross-doctor-fixture-review.json`。

只读建议仍未实施：若后续证据支持，可评估显式 ProcessStartInfo 的 UseShellExecute=false / CreateNoWindow=true，保留 Timeout 结果、两个 PID 必须存在以及父孙进程终止断言。另一个源码层限制是 fixture 约 30 秒自然退出，而超时为 20 秒、随后死亡轮询最多再等 20 秒；自然退出可能误满足死亡断言。该可能性未复现。长期存活 fixture 或单独的有界清理断言可作为后续验证设计，不能把建议写成当前已修问题。

### Usage UTC 日期 fixture

root 的完整 `just test` 在 ccr-usage library suite 记录 63 passed / 2 failed，原始证据为父任务 `research/root-delivery-test.log`。`trends_daily_keeps_output_compatible_and_exposes_reasoning` 得到日期 2026-05-20，期望 2026-05-21；`source_breakdown_respects_date_range_and_ignores_source_filter` 的来源列表也因日期范围不同而失配。

源码 `crates/ccr-usage/src/db.rs:24-38` 的 QueryFilter 默认时区为 Local。`trends_daily` 按所选时区分组，`source_breakdown` 沿用所选时区生成日期边界。两个 fixture 的记录与既有期望按 UTC 日期编写；此前调用未声明时区，因此依赖执行机器的本地时区。root 只在 `:1619` 与 `:1713` 的两处 QueryFilter 中显式选择 `ReportTimezone::Utc`。

审查者将 HEAD 源码仅应用这两处精确文本变换，与当前完整文件逐行比较相同。生产 Local 默认、SQL、fixture 数据、全部原期望断言均未修改：日趋势日期与 41 / 11 / 137 token 数，来源顺序、705 总 token、Codex 2 次事件、份额总和及排除 OpenCode 的断言保留。该修改明确 UTC 聚合用例的输入时区，没有放宽输出要求。

`research/root-usage-utc-fixtures.json` / `.log` 记录 root 执行 `cargo test -p ccr-usage --all-features --lib -- --skip export_bindings` exit 0：65 passed、0 failed、0 ignored，9 项 exports 被筛除。审查者逐条核对两项原失败 selector，以及现有 UTC 边界、23/25 小时 DST 边界、历史 IANA 偏移、跨 DST 本地日期与未知时区错误测试的成功日志；metadata 的源码 SHA 与当前文件一致。机器证据为 `research/cross-usage-utc-fixtures-review.json`。审查者未执行 Cargo，也未修改被审源码。最终根 `just ci` 仍由 root 验收。

## 验证范围与剩余验收

- 本审查实际执行证据核对脚本与 T06/T10 context 验证；39 / 33 / 9、128 份产物、128 个 selector、25 条链接全部通过。机器结果保留最终 33 组 matrix 和 ledger SHA。128 份唯一产物包含规范、导航和受保护文件，不与其他报告的不同统计范围混用。
- 对 gates owner 的反向独立审查由 spec owner 写入 `cross-check-gates.md`；独立 governance 27 项和前端 25 项通过。该报告明确 native lock / full bindings 复用原执行证据。
- root 负责最终完整 `just ci` 验收；本文不把历史失败轮次日志作为最终成功证据。root CI、正式 lint、coverage、docs、跨平台与原生 UI 结论由父任务汇总；本文不提前宣布全绿。
- Linux 证据的 MSRV 环境、macOS 未执行、真实账户/OAuth/远程 SSH/WSL 未验、native WebView 未验均保留。不同层的测试没有被合并描述为原生端到端测试。
- 没有修改、提交、推送或归档用户的其他产物。没有更改 hosted required lane、coverage 阈值或受保护脚本来获得通过结果。
