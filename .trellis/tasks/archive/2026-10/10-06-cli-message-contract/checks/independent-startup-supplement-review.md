# C1 TERM=dumb 启动配置独立检查

当前状态：2026-10-07 当前源码/进程检查、consumer check 和 core strict clippy PASS；共同源码稳定后的 `just lint-strict` exit 0，覆盖 root+core，正式 fmt/type 复测也 exit 0。第 3 个补充验收勾已完成。最新记录见 `2026-10-07-independent-current-source-review.md`。未修改 C1 产品源码。

以下保留 2026-10-06 的历史检查。该轮并行迁移中间态 root+core strict clippy 首次失败仍保留，不能代表 2026-10-07 当前源码结果。

## Findings (fixed)

无。补充实现未发现需要局部修正的 C1 问题。

## Findings (not fixed)

联合 strict clippy 的首轮检查在并行 C2/C3 源码中发现 28 项诊断，超出本次 C1 文件责任。已向主会话转发完整日志，由对应实施者处理。原始文件和行号对应该首轮中间态；后续迁移可改变行号。

| 首轮文件（crates/ccr-cli/src/ 下） | 未修复诊断 |
| --- | --- |
| commands/platform/current.rs | 13 unused_imports；94、99、102、111、117 useless_format |
| commands/platform/switch.rs | 10 unused_imports；52 useless_format |
| commands/profile/current.rs | 22 unused_imports；289 useless_format |
| commands/profile/list.rs | 13 unused_imports；37、50、53 useless_format |
| commands/profile/switch.rs | 18 unused_imports；38、346 useless_format |
| commands/claude/profile.rs | 20 unused_imports |
| commands/codex/profile.rs | 20 unused_imports |
| commands/codex/auth/save.rs | 57 collapsible_if |
| commands/codex/sync_history.rs | 67 useless_format |
| commands/data/stats.rs | 299、364 useless_format |
| commands/platform/info.rs | 83 useless_format 与 format_in_format_args；86 useless_format |
| sync/commands.rs | 1475、1559 useless_format |

已经启动的 root bin type-check 后续完成，另报告 sync/commands.rs:237、649 的两个 unused_imports。该命令共 9 项 unused-import 警告，包含上表的 7 项 import 诊断。新增两项也已转发主会话。没有修改 C2/C3 文件，没有等待其他实施者完成后重复联合检查。

父 AC5 与保留 helper 样式的初始冲突已通过用户批准的启动策略处理：C1 增加 colored 一次启动限制，保留普通终端默认布局。补充进程测试通过；真实新 handler 的终端验收仍由主会话负责。logger 的独立 ANSI 配置和外部工具输出保持批准的范围外边界。

## 核对结果

- `ColorOutput::configure_cli_output()` 仅在 TERM 精确为 dumb 时调用 colored::control::set_override(false)。其他条件不调用 override、不写流、不修改环境或持久化配置。
- `crates/ccr/src/main.rs` 相对 HEAD 仅增加一个启动调用，位于 Clap 解析前。既有 diagnostics 提前返回、logger 初始化、TUI launcher、dispatch 和退出码代码保持相同。
- logging.rs 中只有一个 set_override 调用，位于该启动方法；共享逐条打印和字段格式不切换全局状态。未新增 unset_override。
- 日志生产实现与 HEAD 在换行规范化后完全相同。源文件 SHA256 记录于 `15-independent-startup-boundaries.log`。
- 隔离探针在子进程调用启动配置；普通测试进程不调用启动配置或修改全局环境。ask_confirmation 使用子进程的 EOF/default=false，不访问真实账号或用户配置。
- 新测试覆盖 TERM=dumb + FORCE 下 title、banner、separator、config_status、env_status 和确认提示的无 ANSI 行为。非 dumb 下 FORCE 优先于 NO_COLOR、默认 NO_COLOR 禁用 ANSI 的既有行为继续通过。
- 原有状态/字段语义、NO_COLOR 符号保留、formatter 不写流、stdout/stderr 归属和脱敏断言继续通过。没有新依赖、参数、环境变量或 TUI 布局更改。

## Verification

| 检查 | 结果 | 原始记录 |
| --- | --- | --- |
| 独立 `cargo test -p ccr-core --test output_presentation -- --skip export_bindings` | PASS，7 项，1 个 ignored 子进程探针由测试启动 | `13-independent-startup-output-tests.log` |
| 独立 `cargo clippy -p ccr-core -p ccr --all-targets --all-features -- -D warnings -D clippy::unwrap_used` | FAIL，exit 101，28 项并行 C2/C3 诊断；未复测 | `14-independent-startup-clippy.log` |
| 启动调用与 logger 边界比较 | PASS | `15-independent-startup-boundaries.log` |
| 独立 `cargo clippy -p ccr-core --all-targets --all-features -- -D warnings -D clippy::unwrap_used` | PASS，含 C1 core 类型检查 | `16-independent-startup-core-clippy.log` |
| 已启动的 `cargo check -p ccr --bin ccr --all-features` | PASS，exit 0；有 9 项并行 C3 unused-import 警告 | `17-independent-startup-root-check.log` |

所有测试采用默认并行和 --skip export_bindings。首轮失败未覆盖，未将 core-only 通过替代 root+core 联合 strict clippy。

## 验证边界与交付

C1 补充接口、root 启动调用及隔离进程验证已完成。联合 lint 尚待最终共同源码复测；主会话负责该项、实际新 handler 40/80/120 列及明暗背景矩阵和 just ci。没有将当前 root+core lint 记录为 PASS，没有验证其他 OS、hosted CI 或真实账号操作。
