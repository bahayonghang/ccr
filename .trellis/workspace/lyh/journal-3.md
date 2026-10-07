# Journal - lyh (Part 3)

> Continuation from `journal-2.md` (archived at ~2000 lines)
> Started: 2026-10-06

---



## Session 120: Codex Auth 参考加固审计与类别 A 修复
<!-- trellis-session: v=2 fp=0008daac1c0fb9e3 -->

**Date**: 2026-10-06
**Task**: Codex Auth 参考加固审计与类别 A 修复
**Branch**: `dev`

### Summary

完成 10-06-reference-hardening：输出 codex-auth 对照审计报告（F1-F24，6 条类别 B 后续任务提案）；实施类别 A 修复：备份按内容去重并复用既有路径、同秒备份追加序号、快照/注册表/回滚改用 AtomicWriter::secret(true)、删除无调用方备份代码、配额错误预览按字符边界截断；写回 ccr-codex 备份合同。cargo test -p ccr-codex 291 passed，lint-strict 与 just test 通过；3 个 cfg(unix) 测试在 Windows 未运行。

### Git Commits

| Hash | Message |
|------|---------|
| `cb56de7d` | fix(Codex认证): 🐛 备份按内容去重并避免同秒覆盖 |
| `eb003ce2` | fix(Codex认证): 🔒 账号快照与注册表改用私有原子写 |
| `3c044536` | fix(Codex配额): 🐛 错误响应预览按字符边界截断 |
| `cc8d4006` | docs(规范): 📝 写回 Codex Auth 备份去重与私有原子写合同 |

### Status

[OK] **Completed**


## Session 121: Codex Auth 后续加固提交与归档
<!-- trellis-session: v=2 fp=be2a8c8670393b3a -->

**Date**: 2026-10-06
**Task**: Codex Auth 后续加固提交与归档
**Branch**: `dev`

### Summary

按用户授权拆分提交 Codex Auth P1–P6 已验收改动并归档父任务与六个子任务。P5 只归档评估，快照文件名迁移未实施。未推送、未建 PR、未发布。

### Main Changes

- ccr-core 增加版本校验后的 owner-only 权限加固
- Codex 认证落地复合身份、注册表兼容、破坏性备份、导入校验与脱敏
- 写回 core/Codex/CLI/TUI 规范合同
- 归档 10-06-codex-auth-followups 及 P1–P6

### Git Commits

| Hash | Message |
|------|---------|
| `dac00bacc8a137ffd0bc670a5a62f33f2dc2d4eb` | fix(核心写保护): 🔒 按进程用户 SID 做版本校验后的 owner-only 权限加固 |
| `1420d8db317cd6bea36766886b4ab3da1d48acc8` | feat(Codex认证): ✨ 复合身份、注册表兼容、破坏性备份与导入校验 |
| `1f284bd2281f097586085802707e63644d62eeba` | docs(规范): 📝 写回 Codex Auth 身份、注册表、备份、权限与导入合同 |

### Testing

- [OK] 沿用源码冻结后的 just ci 16 阶段 PASS（workspace 2038/0/16）；本次拆分提交未单独重跑

### Status

[OK] **Completed**

### Next Steps

- 推送、PR、发布与安装仍未授权
- P5 快照文件名迁移仍待用户决定
- k12 真实账号恢复仍为 UNVERIFIED


## Session 122: Codex 切换后感知并重启 app-server 守护进程
<!-- trellis-session: v=2 fp=737b3d7b3c784ba8 -->

**Date**: 2026-10-07
**Task**: Codex 切换后感知并重启 app-server 守护进程
**Branch**: `dev`

### Summary

实施 10-07：ccr 切换账号后检测托管 app-server 守护进程并按交互模式重启（CLI --restart-daemon、交互确认、非交互警告；TUI 确认 overlay 与四态 toast）。现场验证发现 ManagedProcess 成功等待与 Drop 会连带回收守护进程（Windows Job Object KILL_ON_JOB_CLOSE 链条），以分离模式 spawn_detached 修复并完成红绿回归。现场端到端三条路径通过，check 两轮复查零修复；just ci 仅剩 2 个 10-06 在途测试失败。未推送、未 amend。

### Git Commits

| Hash | Message |
|------|---------|
| `9947bee0` | feat(core): ✨ ManagedProcess 分离模式：成功等待与 Drop 不回收后代进程 |
| `c72ba99f` | feat(cli): ✨ 输出呈现基础：结构化状态输出与下一步提示 |
| `ff3bcdb7` | feat(codex): ✨ 账号切换后感知并重启 app-server 守护进程 |

### Status

[OK] **Completed**
