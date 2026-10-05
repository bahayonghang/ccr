# T10 主会话集成审查

日期：2026-09-28。审查基线：34d8a85e0e48b793733835e0304c8ed33940fcee。本文记录主会话审查和集成修正；最终门禁结果以父任务 integration-report.md 和原始日志为准。主会话编写的补丁另经 gates owner 非作者复核。

## 审查范围与结论边界

核对全部 11 个子任务的独立检查记录、T10 门禁和绑定事务源码、39 项 AC / 9 组 P1 的证据结构，以及根 workspace 和独立 Tauri workspace 的实际门禁。T10 专用 trellis-check 派发失败，原因是 agent thread limit reached。两名作者对非本人切片进行交叉审查，主会话承担集成复核。该替代证据不等于专用角色派发成功。

## 责任边界

- Profile repository 负责资源身份、锁内最新读取、partial edit、CAS 和未知字段保留。CLI/TUI/Tauri 通过 application owner 执行 apply，不在呈现层重新组合 off/apply。
- T02 的 R5/AC5 迁移范围以批准设计为准：Claude/Codex update/rename 使用共享 lifecycle。Grok rename 保留既有具名部分状态；本轮不宣称三个平台的所有 rename 均已统一。依据：T02 design.md 的 R5/AC5 行与 check-report.md 的 AC5 行。
- Tauri background owner 持有实际 execution permit，控制入口能到达 owner；取消请求与最终完成分离。Usage SQL 仍归 ccr-usage。
- Settings 保留原快照并提交 dirty leaves；环境切换以会话身份和版本约束旧草稿。Commands 状态由 shell owner 持有，页面重入恢复快照；历史写入为每 job 至多一次尝试，失败明确可见。
- 生成目录由事务入口管理。直接生成仅成功保留输出；check 在成功和失败后恢复调用者的原目录与字节。原始失败记录不得删除。

## 集成发现与处理

### I01：clean 的可选配置读取依赖隐式初始化

T01 移除查询副作用后，clean backups 的可选 skip_confirmation 读取在缺文件时中断。仅把 ConfigMissing 映射为 false；损坏和不可读错误继续在删除前返回。真实 CLI 回归保留交互确认、dry-run 和备份存在性；损坏配置要求 exit 14、正确标题与错误信息、坏配置原字节不变。强化后 13 项通过。证据：research/root-clean-integration-fix-strengthened.json 与 .log。

### I02：list 分派测试沿用旧初始化预期

成功用例现在显式写入合成 profiles；新增缺文件用例精确要求 ConfigMissing。前后快照保留全部目录和文件字节，以 None/Some(bytes) 区分空目录与空文件。最新源码重新编译后 12 项通过。证据：research/root-final-dispatch-typed-inventory.json 与 .log。

### I03：共享 CLI fixture 未加入桌面环境互斥

完整 Tauri 首轮 401 passed / 5 failed。Claude rename 的前后快照出现两个不同临时 CCR_ROOT，证明并行测试读取了其他 fixture 的环境路径。9 个新增共享 profile 合同入口只持 CLI 测试锁，既有桌面测试持另一把 desktop 锁。现按 T03 既有顺序先取 desktop、再进入 CLI fixture，guard 持有到环境恢复完毕。没有把整个 runner 改为串行，没有跳过行为测试。非作者确认无反序和重入。

第二轮为 404 passed / 2 failed。Codex conflict 仍断言旧中文片段，现精确匹配共享 owner 的 ValidationError；四个 Codex fixture 改用既有 TestProcessEnv RAII，panic 也会恢复变量，并隔离 CCR_LOCK_DIR。Grok 五个通过显式平台路径写文件的测试仍依赖底层 CCR_LOCK_DIR，补持 desktop guard；第六个已有 TestProcessEnv，没有重复加锁。Grok 首次 os error 3 的具体缺失路径未记录，不把推测交错当作独立复现。

最终窄回归 profile 41 项、Grok 16 项均通过，二者存在重叠，不累计。生产错误文案和持久化逻辑未因这些 fixture 失败改变。证据：research/root-tauri-profile-regression.json 与 .log、research/root-tauri-grok-regression.json 与 .log。全量结果另由父报告记录。

### I04：Windows doctor 进程 fixture 偶发未生成孙进程 PID

根 workspace 一次运行在原有 doctor_timeout_terminates_parent_and_grandchild 失败。该源码未改；已有另一次 aggregate 成功记录，失败后原 exe 定向复跑也通过。重跑前保存 exe SHA；失败前未采集 SHA。原因未查明，不声明偶发问题已修复，不删除原失败。证据：父任务 research/root-final-after-fixtures-test.log 与本任务 research/root-doctor-process-retry.json/.log。

### I05：Grok TUI 确认提示测试错误匹配终端网格

根 CI 在 ccr-tui 的 245 passed / 1 failed 后停止。原测试要求英文 Unsaved 首字母大写，而普通弹窗正文为小写 unsaved；修正大小写后，中文帧又因宽字符 continuation 空格及 CCR/保存账号换行而失败。两次失败均保留。该旧断言在基线 commit 中已存在，未改生产文案或布局。

最终断言按真实弹窗几何提取正文，排除边框和背景页面，只规范化大小写及终端空白，再分别要求保留 CCR 账号和未保存凭据的警告。2 种语言 × 4 个尺寸 × 6 个状态的 48 帧、Enter/n/Esc、q 与 personal 断言保留。实际定向测试 1 passed；完整 TUI 结果由最终根 CI 单独记录。gates owner 已非作者复核。证据：research/root-tui-frame-repro.log、root-tui-frame-layout-repro.log、root-tui-frame-normalized-fix.log 和 root-tui-frame-assertion.json。

## 验证与保留

12 组父子任务上下文均通过 task.py validate。两份原 .tmp 脚本 SHA 与实施基线一致，原 Insights 任务分别保持 planning/in_progress。完整前端测试 900 项及 i18n 24 项通过，coverage 75.19% 高于既有 70% 阈值。主工作区正式 frontend-check 的 5 个 no-console 错误保留；隔离授权源代码副本的同一命令通过，129 个现存相关输入文件的最终字节匹配，2 个 CSS 删除路径仍不存在。

macOS、完整 Linux pinned-toolchain CI、原生 WebView、真实远程环境和真实 provider OAuth 未验收。Linux 补充结果属于 Rust 1.95 MSRV 定向测试。额外的 Tauri all-targets Clippy 基线问题未被正式 bin Clippy 通过覆盖。未提交、推送或归档。

## 最终集成补充

独立完整 just tauri-ci 已 exit 0（406 行为测试、2 guard，含 bindings/inventory）。完整 TUI 库 246 项通过。新增 A14 组仅承诺 1 项真实文件读取/投影和 2 项内存合并的具体断言。

随后根 just ci 再次出现 doctor 的 PID 夹具失败；临时启动 trace 下完整 CLI 344 项通过但未重现。trace 已撤除，fix.rs 与 HEAD 原字节一致，原因未查明。再一次根 aggregate 在 commands.exe 启动出现 0xc0000005；原始日志保留。单独完整 workspace 复核与所有最终验收状态以父报告为准，不能将该 aggregate 标为通过。

### I06：Usage UTC 夹具默认使用主机本地时区

单独完整 workspace 复核已通过 ccr-cli 344 项及 TUI 246 项，随后两个既有 ccr-usage 测试失败。夹具存 UTC 午夜且期望 UTC 日期与范围，查询却使用默认 Local，在 America/Chicago 对应前一天。仅给两个测试 QueryFilter 显式设置 ReportTimezone::Utc；全部原期望值、排序、token 数和日期范围断言不变，生产查询及 Local 默认未修改。完整 ccr-usage 行为 suite 为 65 passed / 0 failed（9 exports 由专门生成入口负责）。原失败见父 research/root-delivery-test.log，修后见 research/root-usage-utc-fixtures.json/.log。最终根 aggregate 另用 root-delivery-after-utc-ci 记录。

## 最终门禁结论

最新 root-delivery-after-utc-ci：Version Sync/Check、Format/Check、Strict Clippy、Workspace Check、完整 workspace Test、Release Build 全部通过；Security Audit 因基线已锁定的 rustls 0.23.40（RUSTSEC-2026-0285）失败。Cargo.lock 与 HEAD 逐字节相同，未修改依赖。该 audit 失败不影响已执行的独立完整 Tauri gate 通过证据，也不被通过结果覆盖。最终 source/生成物/受保护文件与任务状态见父 research/final-delivery-verification.json。
