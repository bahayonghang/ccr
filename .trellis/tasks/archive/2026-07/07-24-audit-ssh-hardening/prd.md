# SSH 信任与传输加固

> 父任务：`07-24-audit-remediation` ｜ 覆盖：P1-02、P1-09 ｜ 报告 Epic B1-B5

## Goal

关闭两条 SSH 安全缺口：(1) `remote_home` 可绕过手写 double-quote escaping 形成远端 shell command injection；(2) UI 中的 host key trust 决策不约束真实 OpenSSH 连接。

## 背景 / 证据（已核实）

### P1-02 shell 注入
- `ccr-ui/src-tauri/src/platform/ssh.rs:91-96` — `shell_escape_single` 只 `replace` `"`、`` ` ``、`$`，**不处理原始反斜杠**
- `ccr-ui/src-tauri/src/platform/ssh.rs:66-89` — `remote_home` 未做 grammar 校验直接拼进 base dir
- `ccr-ui/src-tauri/src/platform/ssh.rs:154-207` — 命令构造为 `cat "{escaped}"` / `mkdir -p "..." && cat > "..."`（double-quoted）
- 可复现：`remote_home = \$(touch /tmp/ccr-poc)`，`$→\$` 后 `\\$()` 在 double quotes 中折叠出 command substitution

### P1-09 trust 脱节
- `ccr-ui/src-tauri/src/commands/ssh.rs:206-249` — `connect_internal` 不握手即 `connected: true`
- `ccr-ui/src-tauri/src/commands/ssh.rs:466-487` — `ssh_confirm_host_fingerprint` 直接接受前端提供的 fingerprint 写 app DB
- `ccr-ui/src-tauri/src/ssh/connection.rs:119` — 连通测试用 `StrictHostKeyChecking=accept-new`
- `ccr-ui/src-tauri/src/platform/ssh.rs:98-117` — 实际 read/write 的 `run_ssh` 未指定 app-owned known_hosts

## Requirements

### 阻断层（hotfix，优先）
- [ ] `remote_home` 仅允许 `~` 或绝对 POSIX path；segment 字符集限制 `[A-Za-z0-9._-]`；拒绝 `\`、`$`、`` ` ``、引号、控制字符、newline/CR
- [ ] host / user / identity_file 同样做 option boundary 校验
- [ ] 若短期仍走 shell，改用标准 POSIX single-quote encoder（`'` → `'"'"'`），不再嵌 double quotes

### 目标层
- [ ] read/write 改 SFTP，不调用远端 shell（B2）
- [ ] 生成 app-owned known_hosts；所有 SSH invocation 强制 `-o UserKnownHostsFile=<app-path>` + `-o StrictHostKeyChecking=yes` + `-o BatchMode=yes`（B3）
- [ ] probe 返回 backend challenge token（challenge_id + host + port + key_type + raw key + fingerprint）；confirm 只接受 challenge_id，不接受任意前端 fingerprint（B4）
- [ ] `connect` 只有在真实握手（如 `ssh ... echo nonce`）成功后才标记 connected
- [ ] host key mismatch 作为 blocking security error，不是普通 network error

## Acceptance Criteria

- [ ] adversarial 语料全部被拒（报告 §9.1 SSH）：`$(id)`、`` `id` ``、`\$(id)`、`\\$(id)`、`";id;"`、`' ; id ; '`、`line\nbreak`、`--option`；合法 `/home/user`、`~` 通过
- [ ] property test 断言：无 shell command interpolation；path grammar 确定性
- [ ] host key new/match/mismatch 三态测试；mismatch 阻断连接
- [ ] `just lint-strict` + `just test` 通过

## Out of Scope

- 不提供任意远端 shell/终端能力
- 不通过命令行参数、日志或前端状态保存密码/私钥内容
- 不新增 Rust SSH 协议栈依赖；本轮复用系统 OpenSSH 的 SSH/SFTP/known_hosts 工具链

## Notes

- SFTP 实现需评估 `russh`/系统 SSH 兼容与 agent/password auth（报告 §5 2C 回滚：SFTP 与 legacy system SSH 双实现，legacy 只读 + feature flag）
- 触发 rust-security-reviewer 复查
- B1（remote_home hotfix）可先独立落地作为发版阻断项，SFTP/known_hosts 为后续
