# CLI 共享消息展示契约

## 目标

完成父任务 R1、R2、R3、R5、R6 的共享入口，使终端标记、普通信息、字段和建议区块采用已选择的简洁风格。

## 背景

共享入口位于 `crates/ccr-core/src/core/logging.rs:31-49`。标签和整行颜色由同一位置产生。现有测试只调用输出函数（logging.rs:421-429）。详细证据见父任务 research/repository-assessment.md 的 F1、F10–F12。

## 范围与要求

- 保留既有公共函数签名、stdout/stderr 归属和确认/脱敏行为。
- success、warning、error、step 使用统一标记；info 输出普通说明。
- 提供可复用状态格式化，供 Doctor 保留 stdout 输出；支持既有 Skip 语义。
- 字段及建议排版不依赖额外第三方库、终端主题或配置。
- 按父 design 的 TTY、NO_COLOR、TERM=dumb、重定向、CLICOLOR_FORCE 矩阵降级。
- 不修改 tracing layer、日志文件、桥接、账号服务、表格结构或根 facade 导出。

## 验收标准

- [x] 五种状态在终端/纯文本可区分；info 不含 [INFO]；正文不被共享函数整行着色。
- [x] 默认捕获输出与 TERM=dumb 无 ANSI；NO_COLOR 终端保留符号。显式强制颜色例外符合父 design。
- [x] error 仍写 stderr，其他既有方法仍写 stdout；共享 formatter 自身不写流。
- [x] 多行字段保持缩进；无建议不打印空区块；命令与说明分行。
- [x] ColorOutput 既有消费者可编译，确认、mask、table、logger 回归无变化。
- [x] 有文本契约测试与两个流实际输出证据；检查默认并行。

## 依赖与状态

无前置子任务。用户已批准继续实施；共享主体已提交为 `c72ba99f4970b0b2e11ddc0f5a74a08ec200263c`，该提交也包含 `configure_cli_output()`。`main.rs` 的启动调用和 core 进程测试仍在工作区。2026-10-07 当前源码的共享格式、进程输出、消费者类型检查与 core strict clippy PASS；C2/C3 已使用接口。任务保持 `in_progress`，新增提交和归档未授权。证据见 checks/verification.md 与 checks/2026-10-07-independent-current-source-review.md。

## 已批准的 TERM=dumb 补充验收

- [x] 共享启动配置仅在 TERM=dumb 设置一次 colored override；main 在 Clap 解析前调用，普通终端布局、logger、输出流和业务分支保持。
- [x] 隔离子进程覆盖 TERM=dumb + FORCE 下保留装饰 helper 无 ANSI；普通 FORCE/NO_COLOR 优先级保持。
- [x] 独立源码与 focused gate 检查通过；2026-10-07 共同源码稳定后的 `just lint-strict` exit 0，覆盖 root+core，正式 fmt/type 复测均 exit 0。见父 checks/2026-10-07-continuation-static-retest-results.json；历史失败和 PASS 回执保留。
