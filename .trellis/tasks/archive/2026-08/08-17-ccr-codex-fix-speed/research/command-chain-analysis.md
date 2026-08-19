# `ccr codex fix` 逻辑链与耗时分析

采样日期：2026-08-17。  
采样环境：Windows，已安装 `ccr 7.1.4`，`codex 0.147.0`。  
源码锚点对应当前工作区 `dev` 分支。

## 1. 命令职责

命令入口：`crates/ccr-cli/src/commands/codex/fix.rs` 的 `fix_command`。  
领域契约：`.trellis/spec/ccr-codex/backend/codex-app-server-cleanup.md`。

当前行为分七段，严格串行：

| 阶段 | 代码位置 | 作用 |
|------|----------|------|
| A 进程清理 | `CodexProcessService::cleanup_report` | 发现并（非 dry-run 时）终止当前用户的 Codex `app-server` |
| B 只读 inspection | `CodexPlatform::inspect_runtime` | 比较 registry / `profiles.toml` / `config.toml` / `auth.json` / 进程环境 |
| C 显式修复 | `decide_runtime_repair` + `repair_runtime` | 仅 `--repair-runtime` 且 `repairable=true` 时重放当前 profile |
| D 环境提示 | `render_env_hints` | 打印 `CODEX_HOME` / `CCR_CODEX_DIR` / `OPENAI_BASE_URL` 与 provider env 存在性 |
| E PATH 校验 | `which_on_path("codex")` | 找不到则退出码 127，跳过 doctor |
| F 上游 doctor | `run_codex_doctor` | 先 `codex doctor --json`，失败再回退 `codex doctor` |
| G 二次 inspection + 退出码 | `inspect_runtime` + `diagnostic_exit_code` | 判断 doctor 期间快照是否变化，再按 2 > 1 > 3 退出 |

退出码契约保持不变：`127` PATH 缺失，`2` 进程残留或发现失败，`1` runtime 阶段失败，`3` 本地漂移或 doctor 期间快照变化，`0` 无确定本地漂移。

## 2. 实测墙钟时间

| 场景 | 墙钟时间 | 退出码 | 说明 |
|------|----------|--------|------|
| `ccr --help`（冷启动） | 531 ms | 0 | CLI 解析 + 日志初始化下界 |
| `ccr codex fix --dry-run`（冷启动） | 24945 ms | 0 | 首次拉起 `ccr` + `codex` |
| `ccr codex fix --dry-run`（热启动） | 7708 ms | 0 | `process_state=clean`，`runtime_consistency=match` |
| `codex doctor --json`（热启动） | 7663 ms | 0 | 与上一行几乎相等 |
| `codex doctor --summary`（热启动） | 7304 ms | 0 | 渲染压缩，检查集不变 |
| PATH 中只有 `ccr`、没有 `codex` 的 `--dry-run` | 91 ms | 127 | 仍执行 A/B/D，跳过 F/G |

结论：干净路径上，CCR 本地阶段（进程快照 + 一次 inspection + 环境提示）约 90 ms。热启动约 7.7 s 几乎全部花在 `codex doctor`。冷启动约 25 s，主要是 doctor 冷启动与外部网络/更新检查。

有真实 `app-server` 且非 dry-run 时，代码固定再加约 4 s 睡眠（见第 4 节），与 doctor 叠加。

## 3. 逻辑问题

### 3.1 TERM 宽限窗口不提前结束

`cleanup_with_backend` 在发出 TERM 后固定执行 `POLL_ROUNDS=10` 次 `wait(300ms)` + 全表 `discover()`，随后对截止时仍匹配的目标发 KILL，再 `RESPAWN_SETTLE=1s`。

```337:347:crates/ccr-codex/src/services/codex_process_service.rs
    let mut current = initial;
    for _ in 0..timing.poll_rounds {
        backend.wait(timing.poll_interval);
        current = match backend.discover() {
            Ok(discovery) => discovery.targets,
            Err(issue) => {
                report.discovery_issue = Some(issue);
                return report;
            }
        };
        record_new_processes(&mut report, &mut seen, &initial_identities, &current);
    }
```

目标在第一轮轮询已消失时，仍空转完 3 s。测试用 `CleanupTiming` 可以把间隔调成 0，生产路径没有“目标已空则结束宽限”的分支。

宽限窗口的契约目的是给 SIGTERM 退出时间，并在窗口内记录替换 PID。最终是否 `respawned` 由 KILL 后的 1 s settle 决定。目标已空时继续空转，不增加检测能力。

### 3.2 Windows 先 KILL，仍支付 Unix TERM 宽限

Windows 上 `kill_with(Term)` 返回 `None`（`Unsupported`），代码立刻改发 KILL，然后仍走完整 10 轮 300 ms 轮询。进程通常在 KILL 后立即消失，这 3 s 是空等。

### 3.3 doctor 超时不回收子进程

```569:580:crates/ccr-cli/src/commands/codex/fix.rs
async fn capture_doctor(bin: &Path, args: &[&str]) -> std::result::Result<Vec<u8>, DoctorError> {
    let mut cmd = tokio::process::Command::new(bin);
    cmd.args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    match tokio::time::timeout(DOCTOR_TIMEOUT, cmd.output()).await {
        Ok(Ok(output)) => Ok(output.stdout),
        Ok(Err(e)) => Err(DoctorError::Spawn(e.to_string())),
        Err(_) => Err(DoctorError::Timeout),
    }
}
```

`tokio::process::Command` 默认不设置 `kill_on_drop(true)`。30 s 超时后 CCR 继续往下走，残留 `codex doctor` 仍会做网络探活和 rollout 扫描，直到它自己结束。注释写“不阻塞本命令”，但子进程继续占 CPU/网络，也干扰紧接着的二次 inspection。

### 3.4 `--json` 无效时整段 doctor 再跑一遍

`--json` 能启动且 stdout 不是 JSON 时，`fallback_plain_doctor` 再调用一次 `codex doctor`。旧版或不兼容输出会把 7–30 s 再付一次，上限接近 60 s。超时路径不会回退，这一点是对的。

### 3.5 inspection 重复读盘

一次完整 `--repair-runtime` 路径会读齐 registry、`profiles.toml`、secret store、`config.toml`、`auth.json` 最多四次：

1. 修复前 `inspect_runtime`
2. `repair_runtime` 内部再 `inspect_runtime` 做 CAS
3. `apply_profile` 再次 `load_profiles`（含 secret overlay）后写 runtime
4. 修复后 `inspect_runtime`
5. doctor 后再 `inspect_runtime`

第 2 步与第 1 步间隔极短，通常结果相同。CAS 有必要，但不必无条件重读全部文件；可用快照身份或 mtime/size 做廉价校验。

`CodexPlatform::new()` 与内部 `CodexRuntimeService::new()` 各自调用 `PlatformPaths::new` 和 `CodexConfigManager::with_default()`，构造重复。单次约毫秒级，不是主因。

### 3.6 doctor 期间任意字段变化一律退出码 3

`snapshot_changed` 为真时，即使二次 inspection 已是 `match`，仍走本地漂移退出码 3。契约如此，用户会把“doctor 自己改了缓存/指针”看成 CCR 修复失败。

### 3.7 doctor 检查集已超出本命令原职责

现场 `codex doctor --json`（0.147.0）共 18 项检查。与 `ccr codex fix` 原目标（残留 app-server、本地 profile/runtime）直接相关的只有：

- `app_server.status`
- `auth.credentials`
- `config.load`

其余包括：

- `network.provider_reachability`：对 custom API 做路由探活
- `network.websocket_reachability`：WebSocket 探活
- `updates.status`：更新检查
- `state.rollout_db_parity`：扫描 rollout（现场 `custom=272, openai=27`），本次 `overallStatus=warning` 就来自这里
- `git.environment` / `mcp.config` / `sandbox.helpers` / `terminal.*`

doctor 没有跳过网络或跳过 rollout 的开关。`--summary` 只改变展示，检查集不变。CCR 把这份全量健康报告当作补充证据，因此干净路径也被绑到 7 s 以上。

## 4. 性能问题

按对墙钟时间的贡献排序。

| 项 | 干净路径 | 有 app-server 的真实清理 | 能否在不改对外语义下收回 |
|----|----------|--------------------------|--------------------------|
| 每次调用 `codex doctor` | 7.3–25 s | 同样 | 不能。去掉或改为按需，会改变默认行为 |
| 固定 10×300 ms + 1 s settle | 0（dry-run / 无目标立即返回） | 约 4 s，另加约 12 次全表 refresh | 能。目标已空则结束宽限，保留 1 s settle |
| Windows 先 KILL 仍走满宽限 | 同上 | 约 3 s 空等 | 能 |
| `--json` 失败后二次 doctor | 旧版最多再 30 s | 同上 | 能。超时不回退；无效 JSON 可直接用已有 stdout 文本 |
| 超时后残留 doctor | 抢 CPU/网络 | 同上 | 能。`kill_on_drop(true)` |
| 重复 inspection / 重复构造 platform | 约数十毫秒 | 修复路径略高 | 能，收益小 |
| 全表 `refresh_processes_specifics(cmd+user)` | 含在 91 ms 内 | 每轮一次，最多约 12 次 | 有目标时随早停一起减少 |
| CLI 冷启动 | 约 0.5 s | 同上 | 本任务不改全局启动 |

主因是 F 段 doctor。次因是 A 段在“确实有进程要杀”时的固定睡眠。其余是正确性边角与重复 I/O。

## 5. 日志展示问题

现场一次干净 `--dry-run` 输出 46 行，几乎全是 `[INFO]`。同一事实出现三次：

1. CCR inspection 约 18 行（路径、profile、provider、一致性）
2. 环境提示 5 行（含全部未设置变量）
3. doctor 高亮约 16 行（再次打印 `CODEX_HOME`、`config.toml`、`model provider`、`auth file`）

`HIGHLIGHT_KEYS` 用键名子串匹配 `config|auth|provider|model|...`，把无关字段一并抽出：

- `config.toml parse`
- `configured servers`
- `auth storage mode`
- `search provider`
- `default model provider`
- `rollout DB model providers`
- `model provider` 出现两次（`config.load` 与 `network.websocket_reachability`）

`overallStatus=warning` 没有对应检查项。现场警告来自 `state.rollout_db_parity`，与 profile/runtime 无关，用户无法从当前输出判断含义。

其它展示问题：

- `provider_auth_validity = not_checked` 每次都打印，容易被读成检查失败
- 未使用已有的 `ColorOutput::step` / `title`，阶段边界不明显
- 没有阶段耗时，7 s 停顿看起来像 CCR 卡死
- 修复成功后再完整打印 18 行 inspection
- doctor 后若快照变化，再完整打印一遍

## 6. 保持现有对外语义时可做的优化

这些不改变 flags、退出码含义、匹配规则、脱敏和“裸命令不写 runtime”：

1. 宽限循环在当前匹配目标为空时结束；仍保留 KILL 后 1 s settle，以便发现快速拉起。
2. Windows 在 TERM 不受支持、已改发 KILL 后，按“已发送强制终止”处理，不再空转满 3 s。
3. `capture_doctor` 设置 `kill_on_drop(true)`，超时后回收子进程。
4. `--json` 已产出非 JSON 文本时，直接走文本渲染，不再启动第二次 doctor。
5. `repair_runtime` 的 CAS 用调用方快照 + 廉价一致性校验，避免无条件二次全量读盘。
6. 输出改为分阶段标题，并打印 A/B/F 段耗时；doctor 高亮去掉子串误伤，并展示导致 `overallStatus` 的检查 id。

## 7. 会改变默认行为、需要产品决定的项

1. 默认是否仍无条件运行 `codex doctor`。
2. 若保留 doctor，是改为 `--doctor` 显式开启，还是 `--skip-doctor` 显式关闭，或仅在本地 `clean + match` 时跳过。
3. 日志默认是继续全量，还是默认紧凑、`--verbose` 才展开路径和环境。

## 8. 源码与规范锚点

- `crates/ccr-cli/src/commands/codex/fix.rs`
- `crates/ccr-codex/src/services/codex_process_service.rs`
- `crates/ccr-codex/src/platforms/codex.rs`（`inspect_runtime` / `repair_runtime` / `apply_profile`）
- `crates/ccr/tests/commands/codex_fix.rs`
- `.trellis/spec/ccr-codex/backend/codex-app-server-cleanup.md`
- 前序任务：`07-21-ccr-codex-fix`、`07-22-ccr-codex-fix-provider-auth-diagnosis`、`07-23-codex-fix-parity`
