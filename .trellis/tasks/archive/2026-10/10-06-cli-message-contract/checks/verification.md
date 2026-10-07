# C1 共享消息展示验证

当前状态：共享主体和 TERM=dumb 补充的独立源码/定向检查 PASS，root+core 联合 strict lint 待最终共同源码门槛。更新日期：2026-10-07。共享主体已提交为 c72ba99f；main 启动调用和 core 进程测试仍在工作区。任务保持 in_progress；新增提交、push、归档或真实账号操作未授权。

下列初始实现记录对应 2026-10-06，历史原始回执保持。2026-10-07 当前源码结果见文末和 `2026-10-07-independent-current-source-review.md`。

## 已修改源码

- `crates/ccr-core/src/core/logging.rs`：增加 `OutputStatus` 与无流选择的 `ColorOutput::format_status`；共享打印方法保留既有 stdout/stderr；普通 info 移除等级标签；仅状态标记和字段名加样式；字段续行增加缩进；敏感字段继续委托原有脱敏算法。
- `crates/ccr-cli/src/commands/common/feedback.rs`：增加 `print_next_steps(&[(&str, &str)])`，空列表无输出，操作名与完整命令分行。
- `crates/ccr-cli/src/commands/common/mod.rs`：注册并导出 helper。
- `crates/ccr-core/tests/output_presentation.rs`：测试子进程捕获输出和隔离环境，不增加产品命令或修改父进程环境。

`OutputStatus` 位于 `ccr_core::core::logging`，未修改根 facade 或既有 re-export。Doctor 使用 `ColorOutput::format_status(status, message, stdout().is_terminal())` 后自行写 stdout。

## 检查结果

所有 Cargo 测试采用默认并行与 `--skip export_bindings`。没有失败检查；首轮日志保留。

| 命令 | 结果 | 原始记录 |
| --- | --- | --- |
| `cargo test -p ccr-core core::logging -- --skip export_bindings` | PASS，11 项 | `01-core-logging.log` |
| `cargo test -p ccr-core --test output_presentation -- --skip export_bindings` | PASS，5 项；1 个 ignored 子进程探针由测试实际调用 | `02-output-presentation.log` |
| `cargo test -p ccr-cli commands::common::feedback -- --skip export_bindings` | PASS，3 项 | `03-feedback.log` |
| `cargo check -p ccr -p ccr-cli -p ccr-sync -p ccr-codex --all-features` | PASS | `04-consumer-check.log` |
| 新增 dumb 字段断言后重跑 output_presentation 命令 | PASS，5 项 | `05-output-presentation-field-retetest.log` |
| `cargo clippy -p ccr-core -p ccr-cli --all-targets --all-features -- -D warnings -D clippy::unwrap_used` | PASS | `06-clippy.log` |
| `just fmt-check` | PASS；含 root/Tauri Rust fmt、11 个 JSON 配置和格式脚本的 5 项测试 | `07-fmt-check.log` |
| `cargo test -p ccr-core core::log_writer -- --skip export_bindings` | PASS，1 项 | `08-log-writer.log` |
| `git diff --check -- <4 C1 source files>` 与 4 个源码文件尾空白检查 | PASS | `09-source-boundary.log` |

`09-source-boundary.log` 同时记录基于 `git show HEAD:crates/ccr-core/src/core/logging.rs` 的精确比较：`fn get_log_dir()` 至 `#[cfg(test)]` 的 logger 生产实现未改变。Git 的 LF/CRLF 提示保留于原始日志，没有为该提示改写其他文件。

## 契约证据

- 五种状态的符号与纯文本词、空内容、长内容、多行内容由纯函数断言覆盖。
- 真实测试子进程的 stdout 包含 success/info/warning/step 与字段，stderr 只包含 error；捕获输出无 ANSI。输出中的邮箱和密钥均为合成值；敏感字段仍输出 `sk-t...cdef`。
- `NO_COLOR=1` 和空 `NO_COLOR` 在指定终端能力的格式测试中保留符号并移除 ANSI。
- `TERM=dumb` 与 `CLICOLOR_FORCE=1` 同时设置时，状态使用中文词，字段和两个流无 ANSI。
- 非 dumb 下 `CLICOLOR_FORCE=1` 与 `NO_COLOR=1` 同时设置时，colored 的强制颜色行为保留；ANSI 只作用于共享状态标记和字段名。
- formatter 不写 stdout/stderr；调用方保留流的选择。建议空数组无标题或空白；命令保持完整。

## 验证边界

- Unit：PASS。测试子进程与捕获的两个流：PASS。
- 实际 TTY、stdout/stderr 混合重定向、窄终端：C1 `NOT_RUN`，由主会话/C3 完成。上述 `terminal` 探针模式传入 `is_terminal=true`，不代表原生终端证据。
- 表格、确认输入和业务逻辑未修改；没有操作用户目录或真实账号。
- 全工作区测试与 `just ci`：C1 `NOT_RUN`，后续父任务最终门槛负责；四 crate 的 consumer check 不等同于该最终门槛。
- 回退仅恢复上述展示和测试文件，无数据迁移。

## 2026-10-07 当前源码复查

HEAD：`528d4bae1b145fd74d7bbc97454caac249a4b896`。共享主体提交：`c72ba99f4970b0b2e11ddc0f5a74a08ec200263c`。未修改产品源码。main 除一次启动调用外与当前 HEAD 相同，保留 daemon restart 的后续提交。

| 命令 | 当前结果 | 原始记录 |
| --- | --- | --- |
| `cargo test -p ccr-core core::logging -- --skip export_bindings` | PASS，11 项 | `2026-10-07-18-core-logging.txt` |
| `cargo test -p ccr-core --test output_presentation -- --skip export_bindings` | PASS，7 项；1 个 ignored 子进程探针由测试实际启动 | `2026-10-07-19-output-presentation.txt` |
| `cargo test -p ccr-cli commands::common::feedback -- --skip export_bindings` | PASS，3 项 | `2026-10-07-20-feedback.txt` |
| `cargo test -p ccr-core core::log_writer -- --skip export_bindings` | PASS，1 项 | `2026-10-07-21-log-writer.txt` |
| `cargo check -p ccr -p ccr-cli -p ccr-sync -p ccr-codex --all-features` | PASS，exit 0，无警告 | `2026-10-07-22-consumer-check.txt` |
| `cargo clippy -p ccr-core --all-targets --all-features -- -D warnings -D clippy::unwrap_used` | PASS，exit 0 | `2026-10-07-23-core-clippy.txt` |
| 当前源码、c72ba99f、既有签名、logger、启动调用边界对比 | PASS，11 项断言 | `2026-10-07-24-current-boundaries.json` |
| `rustfmt --check --edition 2024 <logging.rs / core output_presentation.rs / main.rs / feedback.rs>` | PASS，exit 0 | `2026-10-07-25-scoped-fmt.txt` |
| `git diff --check -- <five C1 source files>` | PASS，exit 0；Git LF/CRLF 提示保留 | `2026-10-07-26-scoped-diff.txt` |

Cargo 测试使用默认并行和 `--skip export_bindings`。时间、命令、退出码保存在 `2026-10-07-current-results.json`；格式和差异检查退出码保存在 `2026-10-07-scoped-results.json`。没有覆盖旧回执。

上述 C1 定向检查结束时，联合 strict lint 与 repo-wide fmt 待 C2/C3 稳定。后续共同源码结果记录在下一节；该阶段记录不作为当前待办。Windows 最终 handler 矩阵、其他 OS、hosted CI 和真实账号仍 `NOT_RUN`。

## 2026-10-07 共同源码最终静态门槛

主会话在 C2/C3 reviewer 产品源码稳定后运行：`just fmt-check`、`just lint-strict`、`just check-workspace` 均 exit 0。`just lint-strict` 执行 `cargo clippy --workspace --all-targets --all-features -- -D warnings -D clippy::unwrap_used`，覆盖 root+core；第 3 个 TERM=dumb 补充验收勾关闭。`just version-check` 的当日 final 回执也 exit 0，无需重复版本检查。

精确命令/退出码见父 `checks/2026-10-07-continuation-static-retest-results.json`；原始回执分别为 `2026-10-07-continuation-fmt-check-retest.txt`、`2026-10-07-continuation-lint-strict-retest.txt`、`2026-10-07-continuation-check-workspace-retest.txt`，版本回执为 `2026-10-07-continuation-version-check-final.txt`。独立 checker 已读取这些原始回执。保留首轮历史 PASS、后续 fmt 首次失败以及本轮复测。

主会话安全 native shared-output probe 的 12 组流/颜色模式 PASS，见父 `checks/2026-10-07-native-stream-verification.json`。该探针不启动 ccr.exe，不覆盖真实 CLI handler、40/80/120 列 handler 展示或真实账号。Windows logger/ConflictChecker Known Folder 隔离缺口使完整 binary、handler native 与 `just ci` 仍为 `NOT_RUN_WINDOWS_LOGGER_ISOLATION`。C1 补充范围已验收，任务仍保持 in_progress；新增提交和归档未授权。

## 2026-10-07 联合验收更新

已批准的 logger 路径分支另由 C3 独立检查，保留输出 formatter、writer/filter/14天清理/redaction/bridge。当前13项logging、7项共享process、3项feedback和1项log_writer均由正式 workspace/all-features/default-parallel/skip-export-bindings Test覆盖。父共享native混合流12与handler18格/162case/复制36均通过，原始身份和流记录见父2026-10-07-isolation-*及C3独立审计。完整just ci尚未结束；旧focused回执不改写，C1仍in_progress，无新增提交或归档。

## 2026-10-07 最终联合门槛

父正式 just ci 16/16、exit0、20:36.985，当前HEAD与82个最终文件hash保持。完整workspace Test2101 passed/0 failed/19 ignored/33 filtered；ignored/filtered不计通过。原生handler18格/162case/复制36与共享混合流12已独立复核。父AC7仍UNVERIFIED，Unknown和完整密码交互运行边界未改变。完整回执见父checks/2026-10-07-isolation-final-gate-verification.json与just-ci-retest.txt。此前pending CI段落是阶段证据；当前fullCI已通过，任务仍in_progress，无新增提交或归档。
