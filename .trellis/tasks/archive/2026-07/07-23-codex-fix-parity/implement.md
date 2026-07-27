# Implement - `ccr codex fix` 进程清理行为等价性

## 1. 建立失败基线

- [x] 在 `ccr-codex` 增加 Unix 真实子进程 discovery 测试，先证明当前实现看不到带
      `codex ... app-server` argv 的同用户 fixture。
- [x] 增加内部 fake backend 测试骨架，覆盖初始 PID、动态新 PID、signal bool 和 start_time。
- [x] 保留本任务 research 中的现场证据，不在自动测试中触碰用户真实 app-server。

验证：

```bash
rtk cargo test -p ccr-codex codex_process_service -- --test-threads=1
```

## 2. 修复进程快照与分类

- [x] 在 `codex_process_service.rs` 引入显式 `ProcessRefreshKind`，只刷新 cmd + owner，关闭 tasks。
- [x] 建立当前进程 owner scope；owner 不可用时 fail closed。
- [x] 使用 `pid + start_time` 内部身份，并把 matcher 改为 argv-aware 的 Codex 启动项 +
      `app-server` 子命令顺序匹配。
- [x] 删除 `!contains("ccr")` 路径排除，改用当前 PID/身份排除。
- [x] 输出安全命令摘要，不回显完整潜在敏感参数。

验证：运行分类矩阵、真实 discovery fixture，并确认 `--dry-run` 能发现测试进程。

## 3. 修复清理状态机

- [x] 把 sysinfo 调用收敛到私有 backend，保留现有 `cleanup()` / `CodexAppServerCleanup`，
      新增 `cleanup_report()` / `CodexAppServerCleanupReport` 供 CLI 获取详细状态。
- [x] TERM 后每轮重新发现当前用户匹配身份；到期 KILL 当前仍匹配的全部身份。
- [x] 信号前重新验证 PID、start_time、owner 和 argv matcher。
- [x] 正确区分 `Some(true)`、`Some(false)`、`None` 和 `kill()` bool。
- [x] 扩展 cleanup 结果，记录动态发现、信号失败、最终 remaining/discovery issue。
- [x] 用 fake backend 覆盖 TERM success、KILL escalation、新 PID、respawn、PID reuse 和失败路径。

验证：

```bash
rtk cargo test -p ccr-codex codex_process_service -- --test-threads=1
rtk cargo test -p ccr-codex -- --test-threads=1
```

## 4. 隔离 CLI 诊断阶段

- [x] 更新 `fix.rs` 渲染新的 process result，保持现有状态名并增加 `unavailable`。
- [x] 将 runtime inspection/repair 错误转成阶段结果，保证环境提示、doctor 和最终总结按设计执行。
- [x] 保持 `--dry-run`、`--repair-runtime`、脱敏、报告保存和 doctor 有效 JSON 处理契约。
- [x] 固化退出优先级 `127 > process 2 > runtime error 1 > local drift 3 > 0`。
- [x] 扩展 command fixture，覆盖 process 可见、runtime unavailable、signal failure 和无写入。

验证：

```bash
rtk cargo test -p ccr-cli --lib fix -- --test-threads=1
rtk cargo test -p ccr --test commands codex_fix -- --test-threads=1
```

## 5. 同步契约与文档

- [x] 更新 `.trellis/spec/ccr-codex/backend/codex-app-server-cleanup.md`，明确 refresh kind、
      owner scope、动态 PID、signal bool、PID identity 和阶段退出矩阵。
- [x] 更新中英文 `docs/reference/commands/codex.md` 与 `docs/reference/platforms/codex.md`。
- [x] 检查代码注释中“等价脚本”“非特权 OS 保护”“cmdline nonsensitive”等不再成立的陈述。

## 6. 质量门与现场验收

```bash
rtk just fmt-check
rtk cargo test -p ccr-codex -- --test-threads=1
rtk cargo test -p ccr-cli -- --test-threads=1
rtk cargo test -p ccr --test commands codex_fix -- --test-threads=1
rtk just lint-strict
rtk just test
rtk just ci
```

现场验收先用受控 fixture，再由用户确认可中断当前 Codex 会话后运行真实命令：

```bash
rtk ccr codex fix --dry-run
rtk ccr codex fix
```

成功信号：dry-run 能列出现有 app-server；实际清理后 `process_state = cleaned`，或在客户端
持续拉起时明确返回 `process_state = respawned` / exit `2`，不再错误报告 `clean`。
