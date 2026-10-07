# C2 实施计划

- [x] 已获继续实施授权，重读父 design、Auth 规范和当前 handlers；C1 共享接口已交付，补充验收仍由 C1/父任务记录。
- [x] 既有实现限定 design 的 handler/测试；trellis-check 于 2026-10-07 独立复核并修复展示范围内问题，保留其他任务改动。
- [x] save 使用普通结果文本、key_value 字段和下一步 helper，去掉重复空行和状态样式。
- [x] 按 design 分支矩阵迁移其余 Auth；当前源码复查未发现 JSON 提前返回、错误返回、service 调用或确认条件变化。
- [x] 原始 Auth 调用 203 项均有处置：迁移 162、共享覆盖 34、保留 7；未修改无展示的 mod。新增 daemon 行归已完成的独立任务，当前检查保留该功能。
- [x] 在 output_presentation binary fixture 覆盖成功、缺字段、重复、未登录、unsupported、空列表、取消与失败；部分导入由既有 renderer 3 项补充。运行中 warning 由受控测试子进程验证，不启动真实 Codex。Unknown 无 native producer，runtime NOT_RUN。
- [x] 断言流、退出码、建议命令文本、DTO、脱敏及存储效果；只比较任务相关副作用。Doctor 保留完整文件字节、文件/目录 mtime 与目录项等式；Windows inventory 使用 fresh fs::metadata。
- [x] `cargo test -p ccr --test commands -- output_presentation --skip export_bindings`：路径隔离后的首次 21/22，保留失败；修复目录缓存元数据夹具后 22/22、0 ignored，exit 0。
- [x] `cargo test -p ccr --test commands -- grok_auth --skip export_bindings`：路径隔离后的 2026-10-07 回归 8/8、exit 0；早期未隔离回执保留。
- [x] `cargo test -p ccr --test commands -- claude_profile --skip export_bindings`：路径隔离后的回归 10/10、exit 0；早期未隔离回执保留。
- [x] `cargo test -p ccr --test commands -- codex_profile --skip export_bindings`：路径隔离后的回归 14/14、exit 0；早期未隔离回执保留。
- [x] Auth 纯单测：Codex filter 6 项、Claude filter 1 项通过；空导入展示修复后 import filter 3 项通过。命令回执区分 lib 实际匹配与 integration 零匹配。
- [x] `just fmt-check`；原定 package strict clippy 规则由根 `just lint-strict` 的 workspace/all-targets/all-features 同等参数覆盖。C3 独立修正后的最终共同源码 fmt/lint/type 各 exit 0，见父 continuation-static-retest-results.json；未另跑重复 package clippy。
- [x] checks/ 已保存历史证据、2026-10-07 首失败及复测、独立复查和当前源码清单。终端、全量 binary/CI 仍未完成。

2026-10-07 路径隔离后的独立局部复查已完成；C2 保持 in_progress。安全门槛 core logging 13/13、ConflictChecker 5/5、双 logger 合成子进程 2/2 通过。当前 `just fmt-check`、scoped strict clippy 和 scoped all-targets/all-features check 均 exit 0。展示 22/22 与既有回归 8/10/14 通过；原生终端及父最终 full CI 由主会话继续。回执名以 `2026-10-07-path-safe-` 开头。

授权前的历史状态：Windows `dirs::home_dir()` 使用 Known Folder，logger 和 ConflictChecker 绕过 HOME/USERPROFILE；日志实际变更范围未审计。主会话曾停止 CLI binary/native 并为 17 项新文件进程测试添加 Windows ignore，ignored 未记为 PASS。展示旧首轮 14/15、第二轮 17/21 均保留。用户随后批准父 research/windows-fixture-isolation-scope.md 的最小修复。独立路径门槛通过后解除 17+5 个临时 ignore，并移除同名 warning 子探针的 `--ignored` 参数；其他 intentional ignored probes 保留。Windows 权限 helper 仍为 no-op。

详见 checks/independent-review.md、checks/verification.md。错误传播、账号服务与存储仍不改；commit、push、发布和归档未授权。
