# TERM=dumb 验收差异与修正提案

状态：用户已于 2026-10-06 选择“采用统一降级（推荐）”。修正进入实施，首次差异与提案保留。

实际 Windows ConPTY，TERM=dumb、CLICOLOR=1、NO_COLOR/CLICOLOR_FORCE 均不存在。当前 C1 test binary 调用既有 test_output_methods：状态行均为纯文本，但 separator 仍输出 dim ANSI。回执：父 checks/native-decoration-dumb-retest.json。首轮测试名称不匹配，零项执行，仅为无效探测，保留 native-decoration-dumb.json。

父 PRD AC5 要求 TERM=dumb 输出无 ANSI；父 design 同时保留 title、separator、banner、表格和确认的默认样式。C1 独立检查确认这些入口继续直接使用 colored，TERM=dumb 门槛只覆盖状态和字段。该差异不能按完整 AC5 通过记录。

建议在 CLI 启动时设置一次进程内颜色限制，仅 TERM=dumb 调用 colored::control::set_override(false)。增加 ColorOutput::configure_cli_output()，由 crates/ccr/src/main.rs 入口调用。其余终端、变量优先级、默认布局、输出流、JSON、业务调用和退出码保持现状。限制覆盖同一进程内使用 colored 的标题、分隔线、表格和确认文字；不修改 tracing 的独立 ANSI 配置，不处理外部工具输出，不改 TUI 布局。

该提案增加一个 CLI 启动调用和一个小型共享方法。没有逐条消息切换全局状态，没有新依赖、参数、配置或环境变量。用户已确认采用统一降级；普通终端默认布局与业务边界保持。

待确认的可审阅补丁：

```diff
--- a/crates/ccr-core/src/core/logging.rs
+++ b/crates/ccr-core/src/core/logging.rs
@@ ColorOutput
 impl ColorOutput {
+    /// Configure colored once at CLI startup for terminals without ANSI support.
+    pub fn configure_cli_output() {
+        if std::env::var("TERM").is_ok_and(|term| term == "dumb") {
+            colored::control::set_override(false);
+        }
+    }
--- a/crates/ccr/src/main.rs
+++ b/crates/ccr/src/main.rs
@@ main
 async fn main() -> ExitCode {
+    ccr_core::core::logging::ColorOutput::configure_cli_output();
     let matches = build_cli_command().get_matches();
```

如获批准，补充隔离子进程测试：TERM=dumb + FORCE 时 title/banner/separator/config_status/env_status 没有 ANSI；普通 FORCE 和 NO_COLOR 优先级保持；实际 CLI list/current/确认、JSON 继续回归。重新验证共享层、root clippy 和当前源码全量门槛，并在最终 spec 记录“一次启动配置”的限制。
