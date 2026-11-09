# P2 正式 workspace 验证

状态：AC1–AC7 PASS（Windows本地）。独立检查通过，未提交或归档。跨crate最终 `just ci` 由父任务在后续源码冻结后执行，本记录不表示全量CI通过。

- `just version-check`：退出0。
- `just lint-strict`：退出0，敏感写入策略与workspace all-targets/all-features严格Clippy通过。日志：workspace-lint.log。
- `just test`：退出0，默认并行度、`--skip export_bindings`。共 1955 passed、0 failed、16 ignored；41 个结果组。日志：workspace-test.log。
- 最终局部修复后的fmt/scoped测试见independent-check.md。

NOT_RUN：just ci、Unix原生权限、真实账户与原生终端验证。P1等后续改动会使本记录成为P2阶段证据；父任务必须对最终源码复验。
