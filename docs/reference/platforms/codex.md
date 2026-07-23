# Codex 平台配置指南

Codex 平台当前采用“账号面”和“运行时面”分离的设计。

## 当前主路径

```bash
ccr codex auth current
ccr codex auth list
ccr codex profile list
ccr codex profile switch <name>
ccr codex profile current
ccr codex profile off
ccr codex fix
```

## `auth` 与 `profile`

- `ccr codex auth ...`：保存 / 切换 / 导入导出 official auth 账号
- `ccr codex profile ...`：把 profile 写入 `~/.codex/config.toml` 与 `~/.codex/auth.json`
- `ccr codex profile off`：退出 profile mode，恢复到 official auth runtime

## 关键路径

- Runtime config：`~/.codex/config.toml`
- Runtime auth：`~/.codex/auth.json`
- Profiles：`~/.ccr/platforms/codex/profiles.toml`
- Registry pointer：`~/.ccr/config.toml` 中 `[codex].current_profile`

## Runtime 诊断与修复

先切换到需要排查的 profile，再运行诊断：

```bash
ccr codex profile switch future
ccr codex fix
```

`ccr codex fix` 会清理残留 app-server，并比较调用瞬间的 registry pointer、`profiles.toml`、`config.toml`、`auth.json` 与当前进程环境。结果分别报告 `process_state`、`runtime_consistency` 和 `provider_auth_validity`。

进程发现显式读取 cmdline 与 owner，只处理当前用户的 Codex `app-server`。清理期间按
`PID + start_time` 识别进程，TERM 宽限窗口内持续发现替换 PID，并在每次信号前重新验证
owner 与 argv；输出只显示脱敏摘要。无法建立安全快照时会报告
`process_state = unavailable`，不会把未知状态当成 `clean`。

裸命令只诊断，不重写 runtime。发现可安全修复的本地漂移后，显式运行：

```bash
ccr codex fix --repair-runtime
ccr codex fix --dry-run --repair-runtime
```

`--repair-runtime` 只通过既有原子应用路径重放当前保存的 profile，不修改或轮换保存的 secret。组合 `--dry-run` 时既不终止进程，也不写 `config.toml` / `auth.json`。

进程、runtime inspection/repair 和 doctor 是独立阶段。runtime 阶段不可用时命令仍会运行
可用的 doctor，并以退出码 `1` 报告；进程仍存在或进程发现不可用时退出码 `2` 优先。

CCR 自己的 reconciliation 不新增第三方凭据探测；命令仍会运行上游 `codex doctor`，其具体检查行为由当前 Codex 版本决定。即使 `runtime_consistency = match`，`provider_auth_validity` 仍为 `not_checked`；若此时 Provider 返回 `INVALID_API_KEY`，应核验或更新该 profile 保存的 key，而不是继续清理 app-server。

## 历史同步补充

`ccr codex sync-history ...` 仍用于修复 provider namespace 切换后旧历史不可见的问题。跨官方/第三方 profile 迁移时优先使用：

```bash
ccr codex sync-history --bridge official-custom --dry-run
ccr codex sync-history --bridge official-custom --all-history
```

bridge 模式只修复列表可见性；如果历史含 `encrypted_content`，CCR 会提示不能重加密，后续 continue/compact 仍可能受原账号/ provider 加密边界限制。

## 迁移提醒

以下旧路径不再推荐：

- `ccr switch <profile>`
- `ccr platform switch codex`
- `ccr platform current`

改用：

- `ccr current`
- `ccr codex profile switch <profile>`
- `ccr codex profile off`
