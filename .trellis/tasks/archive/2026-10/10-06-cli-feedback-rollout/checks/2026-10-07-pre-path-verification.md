# C3 实施证据

日期：2026-10-07。状态：源码迁移、独立源码/规范复核和最终静态门槛 PASS；机器接口整体验收、Windows handler 终端矩阵及最终 `just ci` 未完成。独立结果见 [independent-review.md](independent-review.md)。

## 范围与源码

- 非 Auth handler 的状态参数采用普通文本，字段使用 key_value，统计、取消及正常空状态使用 info，操作建议使用 print_next_steps。保留服务调用、参数、JSON 分支和既有输出流。
- 续做修正：lifecycle/init、platform/init 与 optimize 的纯路径字段/统计使用中性表达；Temp 输入提示使用普通数字；Sync 12 个进度段不再用回车覆盖行，2 个批量步骤使用共享 step，非表格状态字段采用普通文字。表格结构和交互条件保留。
- Doctor 命令将 Ok/Warn/Fail/Skip 映射为 Success/Warning/Error/Skipped，并继续 println 到 stdout。DoctorStatus::label 的 4 个旧字面量保留为公共兼容 API；DTO、服务与诊断条件不变。
- clean/doctor 中文文档和英文镜像记录终端符号、纯文本状态、NO_COLOR、字段/建议格式。

## 处置清单

`complete_inventories.py` 保留父 output-callsite-inventory.json 的基线 head、counts、files 和 931 个调用点身份，并在原调用点附加 owner/disposition/reason。semantic-review-inventory.json 保留 452 个基线入口及原 source。当前源码覆盖作为独立 snapshot 追加，另存 SHA-256，不将基线行号伪装为当前行号。

最终源码复核统计：931 个基线调用（C1 11、C2 203、C3 717），修改 494、共享覆盖 302、保留 135；当前覆盖 1091 个 ColorOutput/建议/直接状态与提示引用。C2 处置直接引用其已有 callsite/semantic JSON。最终当前覆盖数以 `2026-10-07-inventory-summary.json` 为准。

旧标签扫描范围 crates、docs、.trellis/spec、.trellis/tasks、.github，79 个命中：公共兼容 API 4、测试 3、历史/任务证据 72、未分类 0。见 `2026-10-07-legacy-label-audit.json`。保留历史回执和调查源码，不修改历史输出。

## 当前执行回执

| 检查 | 结果 | 回执及边界 |
| --- | --- | --- |
| 新 C3 binary 首轮，5 项 | FAIL：3 通过、2 失败 | `2026-10-07-rollout-binary-first.txt`；平台路径字符串分隔符预期错误，Doctor 缺 profiles 文件；同轮发现环境隔离缺口。 |
| 新 C3 非 Doctor binary 复测，4 项 | PASS_OUTPUT / UNVERIFIED_LOG_ISOLATION | `2026-10-07-rollout-binary-retest.txt`；平台列表 DTO/中性统计、Claude 空 profile JSON/文本、40/80/120 COLUMNS 捕获长路径、clean 取消警告与保留文件。4 项显示断言 exit 0，默认并行；属于独立源码修正前回执，最终源码未复跑 binary。Windows logger 副作用未隔离，不能据此确认整体 synthetic 验收。COLUMNS 捕获不代表实际终端宽度验收。 |
| docs `bun run build` | PASS | `2026-10-07-docs-build.txt`，VitePress build complete，exit 0。 |
| docs `bun run audit` | PASS | `2026-10-07-docs-audit.txt`，docs audit passed，exit 0。 |
| `cargo check -p ccr --tests --all-features` | PASS | `2026-10-07-rollout-compile.txt`，独立源码修正前 exit 0，无警告；包含新测试编译，未启动 CLI binary，不代表运行验收。最终源码的安全 compile/lint 由独立 checker 回执记录。 |
| scoped rustfmt 与 git diff --check | PASS | 修改源文件/测试经 rustfmt；CLI handlers、ui_service、sync、commands.rs 及 4 份 docs 的 diff --check exit 0。正式 repo fmt/lint 由主会话与 checker 记录。 |
| 完整 root binary suite、多 crate 全量、native、hosted、最终 just ci | NOT_RUN / UNVERIFIED | 路径隔离缺口发现后停止 binary/native；不以 ignore、compile 或 focused 输出代替正式验收。 |

## Windows 隔离缺口

`ConflictChecker::collect_env_vars` 在 commands 展示范围之外直接使用 dirs::home_dir，读取 home 下 .claude/settings.json、.codex/settings.json、.gemini/antigravity-cli/settings.json。dirs 6.0.0 的 Windows home_dir 调用 Known Folder API，忽略测试 HOME/USERPROFILE。Doctor 首轮的 global.conflicts detail 含夹具未创建的 model 信息；该回执标 `FAILED_FIXTURE_ISOLATION`。

主会话与 C2 复核又发现既有 logging get_log_dir/build_file_writer 使用同一 Windows home，日志创建及 14 天保留清理路径指向真实 home；CCR_LOG_LEVEL=off 不隔离这些路径。已执行运行对真实目录的实际变更范围未审计。set_owner_only_dir/file 在 Windows 为 no-op，不据此声称 Windows 权限变化。已执行的非 Doctor 回执只支持输出断言通过，日志副作用隔离仍为 `UNVERIFIED`。未声称所有运行副作用均在合成目录。

仅对已生成的 C3 首轮与 C2 联合复测回执执行 apiKey/api_key conflict 明文布尔检查，结果均 false；未打印值、未读取真实 runtime 文件。见 `2026-10-07-receipt-redaction-audit.json`。

已停止 CLI binary/native 复测，不修改真实账户、日志目录、Known Folder 配置或范围外业务路径。新增 5 项测试在 Windows 保留编译，并使用带原因的 ignore；忽略状态不算 PASS。非 Windows binary 行为 `NOT_RUN`。是否扩大到路径来源修复由主会话另行取得授权。

## 父验收对应

| 父 AC | C3 当前证据 |
| --- | --- |
| AC1、AC4、AC7 | Auth 属 C2；C3 的字段/建议源码复核有记录，不代替 Auth 进程或真实账号验收。 |
| AC2、AC3 | 字段、取消、计数、普通说明与状态语义源码完成；Doctor 四状态映射完成；C1 formatter 单测与 C3 renderer 独立单测分开记录。binary/native 隔离验收仍开放。 |
| AC5、AC9 | 字段/命令无新增截断，Sync 无新增回车覆盖进度。Windows 实际终端、明暗背景、NO_COLOR/dumb、混合流矩阵仍 UNVERIFIED。 |
| AC6 | DTO/分支/输出流源码复核及 4 项非 Doctor 输出断言存在；Windows logger/Doctor 隔离缺口使完整 binary 兼容验收未关闭。历史 release baseline 身份未证明，不作为当前源码比较证据。 |
| AC8 | 931 个调用及 452 个语义入口均有处置，当前源码覆盖追加，4 个 Doctor label 兼容 API 保留有理由。独立 checker 继续复核。 |
| AC10 | docs 与 focused 输出有当前源码回执；完整 binary、多 crate gate、native 与 just ci 仍未完成。 |

## 回退与未完成项

回退范围为批准的 handler 展示文本、共享 formatter 调用、测试和文档；不执行 auth restore，不回滚账号或外部状态。无数据迁移、依赖更改、提交、推送、发布或归档。

独立源码/规范审查与共同源码安全 fmt/type/lint 门槛已完成。Windows 路径隔离缺口、真实 handler 终端矩阵、完整 binary 和最终 just ci 保持未完成；任务保持 in_progress。

## 2026-10-07 独立复核与最终安全门槛

Reviewer 局部修正 9 个文件：无效/不完整的帮助建议、update 非默认 branch 后续提示、执行前 step、已完成报告标题和正常空状态。Doctor 新增安全 test-executable renderer 探针，覆盖四状态/两个流/summary/dumb，无 DoctorService、ConflictChecker、logger 或 ccr.exe 初始化。见 independent-review.md 的逐文件问题与修正。

最终 scoped rustfmt 后，45 个 C3 产品源文件 SHA-256 均匹配 `2026-10-07-independent-source-flow.json`。42 个去展示 lexical projection 与 HEAD 相等，3 个差异人工复核为状态映射或只含展示的分支；8 个 service/DTO/dispatch/table/logger 边界文件未变。该证据是源码复核，不代替运行等价证明。

| 安全当前检查 | 结果 | 回执 |
| --- | --- | --- |
| 主会话 `just version-check` | PASS，exit 0 | 父 checks/2026-10-07-continuation-version-check-final.txt。 |
| 主会话 `just fmt-check` | PASS，exit 0 | 父 checks/2026-10-07-continuation-fmt-check-retest.txt；此前 map closure/数组布局首失败全文保留。 |
| 主会话 `just lint-strict` | PASS，exit 0 | 父 checks/2026-10-07-continuation-lint-strict-retest.txt；workspace/all-targets/all-features，含 root+core。 |
| 主会话 `just check-workspace` | PASS，exit 0 | 父 checks/2026-10-07-continuation-check-workspace-retest.txt；只编译，不启动命令进程。 |
| `cargo test -p ccr-cli --lib commands::doctor_cmd::tests -- --skip export_bindings` | PASS，2 项 | 2026-10-07-independent-doctor-renderer-first.txt；1 个 ignored 探针由实际测试调用，默认并行；此后只修 rustfmt 布局。 |
| C3 scoped git diff --check | PASS，exit 0 | 2026-10-07-independent-scoped-diff-first.txt；45 source + 4 docs。 |
| 独立源码/契约检查 | PASS_SOURCE_REVIEW | independent-review.md 与 2026-10-07-independent-source-flow.json；主会话复核全部当前清单和 72 个源码 hash。 |
| 主会话 safe native shared-output 12 组 | PASS | 父 checks/2026-10-07-native-stream-verification.json；不启动 ccr.exe，仅验证共享 helper 两个流/颜色/重定向。 |

静态命令/退出码总表为父 checks/2026-10-07-continuation-static-retest-results.json。初始 PASS、后续首次 fmt 失败、最终复测分别保留。产品源码在完整 review 与 scoped rustfmt 后固定；后续仅更新任务证据。

共同安全门槛及 shared native probe 不关闭 Windows 完整 handler binary/native、实际 40/80/120 列明暗矩阵、多 crate 全量或 `just ci`。这些项仍 `NOT_RUN_WINDOWS_LOGGER_ISOLATION`/`UNVERIFIED`。Unix、hosted 和真实账号未运行。C3 仍 in_progress；不降低 PRD 验收条件。
