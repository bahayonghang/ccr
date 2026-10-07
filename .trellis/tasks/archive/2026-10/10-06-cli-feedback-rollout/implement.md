# C3 实施计划

## 2026-10-07 已批准的路径隔离补充

- [x] trellis-implement 按父 research/windows-fixture-isolation-scope.md 修复 logger/ConflictChecker 的路径解析及已确认的进程夹具，保留其他业务与日志策略。
- [x] 安全路径与作用域回归通过，trellis-check 独立核对后解除当前 Windows task tests 的隔离 ignore。
- [x] 更新路径规范与当前源码身份，恢复完整相关 binary/crate 套件、原生 handler 矩阵和 just ci；保留首次失败与旧隔离状态。

用户已明确批准该最小范围扩展；不含提交、推送或归档。

- [x] 核实 C1 共享主体、原实施授权及 2026-10-07 继续实施指令；最终集成等待 C1/C2/C3 联合验收。
- [x] trellis-implement 按 scope-inventory.md 负责非 Auth 消息与批准测试/文档，保留其他任务改动；主会话已调度独立 trellis-check。
- [x] 逐组迁移 Platform/Profile、Data、Lifecycle/Project、Sync/Temp/服务启动、Update/Provider/Sessions；源码保持服务调用和命令条件。
- [x] DoctorStatus 映射 OutputStatus，通过 format_status 返回文本并继续 stdout；Skip 与 Fail 使用各自状态。JSON/只读/退出码的 Windows binary 验收仍开放。
- [x] 完成直接 println 提示和重复 emoji 复核；全仓旧标签命中分类为兼容 API、测试或历史证据，禁止无差别替换。
- [x] 同步 clean/doctor 中文及英文文档，只改受影响示例与状态说明。
- [x] `cargo test -p ccr --test commands -- output_presentation --skip export_bindings`：安全复测 22/22、0 ignored；首次 21/22 和修复回执保留。
- [x] 新增独立 `output_presentation_rollout.rs`：非 Doctor 4 项显示断言通过；首失败回执保留。Windows Known Folder 日志隔离未验证，不关闭机器接口整体验收；5 项新测试在 Windows 明确 ignored 并保留编译。
- [x] 完整 root commands 由 `just ci` Test 的 workspace/all-features/default-parallel/skip-bindings 覆盖，139 项通过；旧 help 断言首失败保留。
- [x] 相关完整 crate 套件由 `just ci` Test 的 workspace/all-features/default-parallel/skip-bindings 覆盖；总计 2101 passed、0 failed，ignored/filtered 不计通过。
- [x] `just version-check`、`just fmt-check`、`just lint-strict`：2026-10-07 最终共同源码均 exit 0；另有 `just check-workspace` exit 0。原始回执由主会话运行，独立 checker 已读取，见 verification.md 最终安全门槛。
- [x] docs/ 下依次 `bun run build`、`bun run audit`，2026-10-07 均 exit 0。
- [x] 按父 implement 的终端矩阵使用隔离合成 fixture，在实际 Windows 终端展示并捕获回执；不能用只编译、HTML 预览或截图 mock 替代实际运行。
- [x] 若原生终端/特定平台无法执行，保留 NOT_RUN/UNVERIFIED，不关闭相应 AC。
- [x] 重新扫描 ColorOutput、raw 标签、提示、重复图标；两个父 JSON 清单保留 931/452 个基线身份，并追加当前源码覆盖记录与 SHA-256。所有基线项有处置，新引用独立记录。
- [x] 主会话更新最终展示 spec/index；独立 checker 已完成规范、展示限定 source-flow 与最小 diff 复核。完整机器接口运行验收仍被 Windows Known Folder 隔离缺口阻塞，相关勾保持开放。
- [x] 父任务最终 `just ci`；保留第一失败和复测，禁止为通过门槛扩大依赖修复范围。
- [x] 在 checks/verification.md 记录各父 AC、unit/process/native/OS/hosted 状态、原始失败及 scoped git diff --check。

用户已批准实施。源码与 docs 已更新；binary 已执行部分保持显示结果与副作用隔离边界分开。Windows native/隔离 binary、Unix、hosted 和最终 `just ci` 未完成；commit、发布和归档未授权。

2026-10-07 当前复测：路径独立安全 13+5+2、展示 binary 22、既有回归 8/10/14、原生 handler 18×9=162、屏幕复制 36、混合流 12 均通过。完整 workspace Test 已通过；完整 just ci 16/16通过。旧暂停状态仅属此前证据，见 checks/2026-10-07-pre-path-verification.md。父 AC7 保持 UNVERIFIED。

最终门槛已通过：just ci 16/16、exit0、20:36.985；sourcemanifest82与HEAD无变化。父AC7的Unknown/密码交互运行边界仍UNVERIFIED，交付与归档未授权。
