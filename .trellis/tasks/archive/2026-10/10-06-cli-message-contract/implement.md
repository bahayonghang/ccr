# C1 实施计划

- [x] 获实施批准后启动本子任务，重读当前源码及父 design。现有同名文件如已出现，先复核，不覆盖。
- [x] 实现/检查分别交给 trellis-implement/trellis-check，限定 design 中的文件责任；不得撤销其他任务改动。
- [x] 在 logging.rs 现有 ColorOutput 中集中修改状态入口，保持写入流；增加必要 formatter 和状态类型。
- [x] 完成字段环境降级与多行缩进；新增 CLI 建议排版 helper，空列表无输出。
- [x] 测试覆盖状态语义、空/长/多行内容、无色/纯文本和颜色例外；不同 env 用子进程，不降低并行度。
- [x] 新 core integration test 通过 test-binary 子进程探针调用公共函数，捕获 stdout/stderr，验证 formatter 不改变流归属；不增加产品隐藏命令。
- [x] `cargo test -p ccr-core core::logging -- --skip export_bindings`。
- [x] `cargo test -p ccr-core --test output_presentation -- --skip export_bindings`。
- [x] `cargo test -p ccr-cli commands::common::feedback -- --skip export_bindings`。
- [x] `cargo check -p ccr -p ccr-cli -p ccr-sync -p ccr-codex --all-features`。
- [x] `cargo clippy -p ccr-core -p ccr-cli --all-targets --all-features -- -D warnings -D clippy::unwrap_used`。
- [x] `just fmt-check`，受影响范围 git diff --check。
- [x] 保存 checks/verification.md，明确 unit/process/TTY 的不同证据；终端展示由 C3 完成。
- [x] 主会话复核公共 API、logger 未变和父 AC 映射，C1 验收后 C2/C3 才使用接口。

回退限于本子任务批准文件。用户已批准继续实施；共享主体已提交为 `c72ba99f`，main 启动调用与 core 进程测试仍在工作区。任务保留 `in_progress`；新增提交和归档未授权。历史检查与 2026-10-07 当前源码复查分别保存在 checks/。

## 用户批准的环境降级补充

- [x] C3 implementer 实施共享启动配置方法和 root main 一次调用；不更改正常模式默认布局或 logger。2026-10-07 边界复查 11 项断言 PASS；共享方法已在 c72ba99f，main 调用仍未提交。
- [x] 隔离子进程补测 TERM=dumb + FORCE 的装饰 helper；普通 FORCE/NO_COLOR 优先级保持。2026-10-07 当前源码进程测试 7 项 PASS，1 个 ignored 探针由测试实际启动。
- [x] 独立 checker 已完成补充范围、测试、current-source consumer check 与 core strict clippy；2026-10-07 共同源码稳定后，主会话 `just lint-strict` exit 0，覆盖全 workspace/all-targets/all-features（含 root+core）。正式 fmt/type 复测也 exit 0。引用父 checks/2026-10-07-continuation-static-retest-results.json；首次差异及历史 PASS 都保留。
