# C3 独立源码与安全检查

日期：2026-10-07。角色：trellis-check。HEAD：`528d4bae1b145fd74d7bbc97454caac249a4b896`。

状态：C3 展示源码、规范复核与最终静态门槛 PASS；Windows 完整命令进程、handler 原生终端矩阵和 `just ci` 因路径隔离缺口未验收。任务保持 `in_progress`。无提交、归档、推送或发布。

## Findings (fixed)

- File：`crates/ccr-cli/src/commands/platform/init.rs:118`。
  Issue：新增平台 profile 帮助建议对已退役的 Gemini/Droid 路由不可执行。
  Fix：建议改为现有 `ccr platform --help`；初始化分支及参数不变。
- File：`crates/ccr-cli/src/sync/commands.rs:476`、`:880`、`:1150`、`:1230`、`:1305`。
  Issue：上传准备阶段重复显示“正在上传”；空列表建议缺少 folder add 的必填参数；批量上传/下载的执行前信息使用普通说明状态。
  Fix：准备阶段改为“正在准备上传内容”，保留实际上传步骤；两处建议改为 `ccr sync folder add --help`；两个批量执行前消息使用 `step`。WebDAV、备份、过滤、执行、返回和原输出流不变。
- File：`crates/ccr-cli/src/commands/codex/fix.rs:146`、`:289`。
  Issue：Apply 操作前信息使用普通说明状态；已取得清理报告后仍显示“正在清理”。
  Fix：操作前信息使用 `step`，报告标题改为“清理结果”。修复及清理调用不变。该文件属于 C3；未修改 C2 的 `codex/auth/repair.rs`。
- File：`crates/ccr-cli/src/commands/temp_token.rs:46`、`crates/ccr-cli/src/commands/platform/profile_open.rs:163`。
  Issue：应用临时配置、启动编辑器之前的信息使用普通说明状态。
  Fix：对应两处使用 `step`；应用及启动条件不变。
- File：`crates/ccr-cli/src/commands/provider_cmd.rs:71`、`:144`、`:238`。
  Issue：测试 Provider、批量测试和验证 API Key 的执行前信息使用普通说明状态。
  Fix：三处使用 `step`。网络请求、凭证处理和表格布局不变。
- File：`crates/ccr-cli/src/commands/update.rs:74`。
  Issue：非默认分支的预览后建议执行 `ccr update`，该命令会丢失用户已选择的分支。
  Fix：建议改为 `ccr update --help`，显示“查看更新选项”；未引入 shell 命令模板或修改更新参数。
- File：`crates/ccr-sync/src/sync/content_selector.rs` 的正常无内容分支。
  Issue：正常无可选内容使用 warning。
  Fix：该分支改为 `info`。空返回值、checkbox 布局、输入循环和必须至少选择一项的 warning 保留。
- File：`crates/ccr-cli/src/commands/doctor_cmd.rs:128`、`:164`。
  Issue：已有 renderer smoke test 未断言四种状态、流归属、计数和失败时不得显示全通过结论。
  Fix：新增合成报告的 test-executable 子进程探针，断言 Success/Warning/Error/Skipped 均在 stdout、stderr 为空、计数完整、TERM=dumb 无 ANSI，存在 Fail 时无全通过结论。子进程只调用 renderer，不调用 DoctorService、ConflictChecker、logger 或 ccr.exe。
- File：`doctor_cmd.rs` 和 `platform/init.rs`。
  Issue：正式 `just fmt-check` 首次发现新增 map closure 和建议数组布局不符合 rustfmt。
  Fix：限定 reviewer 的 9 个文件运行 scoped rustfmt，exit 0。主会话正式 fmt/lint/type 复测均 exit 0；首次失败全文保留于父 `2026-10-07-continuation-fmt-check-final.txt`。

## Findings (not fixed)

- Windows fixture 隔离：`crates/ccr-cli/src/managers/conflict_checker.rs:120` 通过 `dirs::home_dir()` 读取 `.claude/settings.json`、`.codex/settings.json`、`.gemini/antigravity-cli/settings.json`。Windows 的该函数使用 Known Folder API，HOME/USERPROFILE 覆盖不能隔离这些读取。首轮 Doctor receipt 含 fixture 未生成的信息，状态为 `FAILED_FIXTURE_ISOLATION`。未再次读取真实配置。路径来源修复涉及业务边界，超出批准的展示迁移范围，留给主会话取得范围授权。
- Windows logger 隔离：`crates/ccr-core/src/core/logging.rs:250`、`:392` 在真实 home 的 `.ccr/logs` 初始化 writer，包含创建目录、14 天旧日志清理及既有日志权限处理入口。`CCR_LOG_LEVEL=off` 不阻止初始化。Windows owner-only 设置为 no-op；不声称权限发生变化。过去运行对真实目录的实际修改范围未经审计。未修改 logger、Known Folder 或真实日志目录。
- 必需运行验收：完整 root binary suite、受影响多 crate 全量测试、Windows handler 40/80/120 列和明暗背景矩阵、最终 `just ci` 均 `NOT_RUN`/`UNVERIFIED`。5 项新 binary fixtures 的 Windows ignore 均有原因；ignore 和 compile 不算 PASS。Unix、hosted、真实账号与在线 Provider/WebDAV 运行均未验证。共享 native 探针不能关闭 handler 的验收标准。
- 预存框架名称：`crates/ccr-cli/src/services/ui_service.rs:330`、`:566` 保留 HEAD 中的 `Vue 3` 文本，而 `ccr-ui/package.json:83` 声明 React 19。框架说明内容修订不属于批准的状态、字段和建议迁移；未修改该预存文本。该项不影响新输出流及业务边界结论。

## 源码与契约复核

`2026-10-07-independent-source-flow.json` 固定 45 个 C3 产品源码的 SHA-256。去除 import、ColorOutput、直接 print 及 next-step 输出后的 lexical projection 有 42 个文件与 HEAD 相等。该投影只支持源码审阅，不构成运行等价证明。3 个剩余差异已逐段人工复核：

- Doctor 增加现有 DoctorStatus 到 OutputStatus 的展示映射；报告、summary、JSON 路径、诊断调用及 stdout 保留。
- profile/delete 移除只含输出的 else，并将建议放到共同位置；删除、保存、返回及条件保持。
- sessions 根据已有 `stats.errors` 选择 warning/success 输出；两个新增分支只包含展示调用，索引和统计结果保持。

8 个边界文件与 HEAD 精确相同：DoctorService、ConflictChecker、Clap definitions、dispatch、common/table、ccr-codex 平台逻辑、core logging、root lib。C1 main 仅新增启动输出配置调用，由 C1 独立回执验证。未增加产品命令、依赖、选项、DTO 字段或 JSON 建议区块。

逐段检查表格结构、确认默认值、checkbox 和输入条件。保留输入提示与表格自己的布局，执行阶段使用 step；取消、计数和正常空状态使用 info。Sync 的回车覆盖进度迁移为逐行 step，既有 flush 与批量错误输出流保留。共享 formatter 自身不决定写入流，Doctor 继续写 stdout。

父调查清单保留 931 个基线调用身份和 452 个基线语义入口：migrated 494、shared-covered 302、preserved 135；新增当前覆盖 1091 项单独记录，未改写历史行号。旧标签 79 项分为兼容 API 4、测试 3、历史/任务记录 72，未分类 0。主会话独立复核所有当前引用及 72 个源码 hash；清单由 C3 implementer 刷新，reviewer 未覆盖清单。

已复核 `.trellis/spec/ccr-cli/backend/cli-output-presentation.md` 与 index，规范准确记录 info、五状态、字段、建议、流归属、dumb/force 优先级及 Windows Known Folder 隔离限制。clean/doctor 中文文档与英文镜像表达一致；历史文档保留。规范由主会话维护。

## Verification

| 检查 | 结果 | 回执与边界 |
| --- | --- | --- |
| Lint：主会话 `just lint-strict` | PASS，exit 0 | 父 `checks/2026-10-07-continuation-lint-strict-retest.txt`；workspace/all-targets/all-features，含 root+core；reviewer 已读取原始输出。 |
| TypeCheck：主会话 `just check-workspace` | PASS，exit 0 | 父 `checks/2026-10-07-continuation-check-workspace-retest.txt`；cargo check --workspace，未执行命令进程。 |
| Format：主会话 `just fmt-check` | PASS，exit 0 | 父 `checks/2026-10-07-continuation-fmt-check-retest.txt`；首失败与 scoped 修复回执保留。 |
| 主会话 `just version-check` | PASS，exit 0 | 父 `checks/2026-10-07-continuation-version-check-final.txt`；无版本修改。 |
| `cargo test -p ccr-cli --lib commands::doctor_cmd::tests -- --skip export_bindings` | PASS，2 项，1 个 ignored 探针由真实测试调用 | `2026-10-07-independent-doctor-renderer-first.txt`，exit 0，默认并行。运行后仅修改 rustfmt 布局；测试逻辑保持。未初始化产品服务。 |
| C1 core logging / output_presentation / common feedback / log_writer | PASS，11/7/3/1 项 | C1 当日独立原始回执 18–21；纯函数及测试进程边界。 |
| C3 scoped `git diff --check` | PASS，exit 0 | `2026-10-07-independent-scoped-diff-first.txt`；45 个产品源文件及 4 个文档，不改写原始 receipts 的 CRLF。 |
| 45 个 source hash、42 个 projection equality、3 个人工差异、8 个边界断言 | PASS_SOURCE_REVIEW | `2026-10-07-independent-source-flow.json`；当前 hash 均匹配；非运行证明。 |
| docs `bun run build` / `bun run audit` | PASS，均 exit 0 | C3 implementer 回执 `2026-10-07-docs-build.txt`、`2026-10-07-docs-audit.txt`；文档此后未变。 |
| 主会话 native shared-output | PASS，12 组 | 父 `checks/2026-10-07-native-stream-verification.json`；仅 safe shared probe 的两个流、颜色模式和重定向，不启动 ccr.exe。 |
| 历史 C3 non-Doctor binary 4 项 | PASS_OUTPUT / UNVERIFIED_LOG_ISOLATION | 只支持修正前输出断言；不作为最终源码的完整 fixture 或 runtime 等价证据。 |
| 完整 binary/native handler/just ci | NOT_RUN_WINDOWS_LOGGER_ISOLATION | 路径来源隔离修复及实际验收仍开放。 |

最终共同源码命令/退出码总表见父 `checks/2026-10-07-continuation-static-retest-results.json`。产品源码在 scoped rustfmt 与完整审阅后固定；此后的更新限于任务证据。首轮失败、历史 PASS 与最终复测分别保留。

回退只恢复批准的展示、测试、文档及规范变更。无需账号 restore、真实配置写入或数据迁移。任务仍为 `in_progress`；当前安全门槛不能代替尚未运行的最终验收。
