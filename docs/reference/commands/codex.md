# codex - Codex 运行时与多账号管理

`ccr codex` 是 Codex 平台的专项命令组，当前重点能力包括：

- `ccr codex auth ...`：official auth 多账号管理
- `ccr codex profile ...`：runtime/profile 路由管理
- `ccr codex fix`：残留进程清理与本地 profile/runtime 一致性诊断
- `ccr codex sync-history ...`：修复 provider namespace 切换后的历史可见性

## 常用命令

```bash
ccr codex auth current
ccr codex auth list
ccr codex auth off
ccr codex profile list
ccr codex profile switch <name>
ccr codex profile current
ccr codex profile off
```

## `auth` 与 `profile` 的区别

| 命令组 | 作用 |
|---|---|
| `ccr codex auth ...` | 保存、切换、导出、导入 official auth 账号，或登出当前官方运行时登录 |
| `ccr codex profile ...` | 把某个 CCR profile 应用到 Codex runtime，或清理 profile 路由与运行期凭据 |

`ccr codex auth off` 登出当前官方运行时登录，与 `profile off` 独立。该命令不修改 profile 指针和 `config.toml` 路由。即使当前处于第三方 profile，仍会清除运行期 `auth.json`（file store）或调用 `codex logout`（keyring / auto）。`--json` 可报告仍存在的 `profile_pointer`，提示需要再次 `profile switch` 写回 key。该提示不算失败。

## `profile` 当前支持面

- `list`
- `current`
- `switch <name>`
- `off`
- `create`
- `set-field`
- `enable`
- `disable`
- `delete`
- `open`

## `fix`

```bash
# 先切换目标 profile，再诊断；裸命令不写 runtime，也不调用上游 doctor
ccr codex profile switch future
ccr codex fix

# 显式修复可安全处理的本地漂移
ccr codex fix --repair-runtime

# 仅预览进程清理与 runtime 重放，不发送信号、不写文件
ccr codex fix --dry-run --repair-runtime

# 需要上游健康检查时再运行 codex doctor
ccr codex fix --doctor
```

进程阶段只匹配当前用户的 native Codex / Node Codex wrapper `app-server`，不会匹配
`codex exec`、`codex resume`、`codex login` 或仅把 `codex app-server` 当作参数的其他工具。
实际清理会先发送 TERM，每 300ms 重查匹配目标，最多约 3 秒；目标已空则立即结束宽限。
截止时仍匹配的进程发送 KILL。只要本轮发过信号，仍会再等待约 1 秒做最终快照。
宽限结束后、settle 里才出现的新身份记入 `respawned`，不补发 deadline KILL，退出码为 2。
每次发信号前都会重新确认 owner、PID 启动时间和 argv。若无法安全读取当前 owner 或
命令行，输出 `process_state = unavailable` 并停止发送信号，而不是误报 `clean`。

诊断会区分 profile pointer、route、credential 与 Provider 有效性。CCR 自己的一致性判定只比较本地保存的 secret 和实际凭据来源，不新增第三方凭据探测，也不会输出 key、掩码片段、长度或 fingerprint。默认路径不调用上游 `codex doctor`；需要补充证据时传入 `--doctor`。因此 `provider_auth_validity = not_checked` 不代表失败，也不代表 key 已被 Provider 接受。

进程清理、CCR runtime inspection/repair 与可选的上游 doctor 会分别报告。runtime 阶段失败时输出
`runtime_consistency = unavailable`，后续独立阶段仍继续；原始进程 argv 和阶段错误
中的敏感内容不会回显。`--repair-runtime` 不隐含 `--doctor`。

退出码：

| 退出码 | 含义 |
|---|---|
| `0` | 未发现确定的本地漂移；Provider 有效性仍可能未检查 |
| `1` | CCR runtime inspection/repair 阶段失败 |
| `2` | app-server 仍存在，或无法安全完成进程发现/清理 |
| `3` | 本地 profile/runtime 漂移仍存在，或 doctor 期间快照发生变化 |
| `127` | 传入 `--doctor` 且 `codex` 不在 `PATH` 中 |

## `sync-history`

保留原有用途：修复 `openai` / `custom` provider namespace 切换后，旧历史在 Codex CLI / App 中不可见的问题。

常用模式：

```bash
# 保持旧行为：显式写入某个 provider，默认只处理最近 7 天
ccr codex sync-history --provider custom --dry-run
ccr codex sync-history --provider openai

# 新 bridge 模式：把 openai/custom/缺失 provider 历史桥接到当前 runtime provider
ccr codex sync-history --bridge official-custom --dry-run
ccr codex sync-history --bridge official-custom --all-history

# 诊断 provider、SQLite、preview、cwd、Desktop 首屏限制与 encrypted_content
ccr codex sync-history status
```

补充约束：

- `--provider` 继续保持兼容行为；未指定 provider 时仍读取当前 `~/.codex/config.toml`。
- `--bridge official-custom` 会根据当前 runtime 决定目标：官方/隐式 OpenAI 为 `openai`，第三方 profile 为 `custom`。
- `--all-history` 取消 7 天过滤；普通模式默认仍只处理最近 7 天。
- bridge / all-history 的 SQLite 修复默认只碰 `openai`、`custom` 与缺失 provider；需要额外 provider 时使用可重复的 `--include-provider <name>`。
- 写入前会备份 rollout 首行、`state_5.sqlite` 与 `.codex-global-state.json`；`--dry-run` 只输出计划，不写文件。
- `encrypted_content` 只做统计和警告，不解密、不重加密，也不修改消息正文或文件 mtime。
