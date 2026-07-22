# 优化 ccr codex fix 的自定义 Provider 凭据诊断

## Goal

让 `ccr codex fix` 从“清理 app-server 并转述 doctor”升级为可判定的本地运行时诊断：明确当前检查的是哪个 CCR profile，区分残留进程、profile/runtime 漂移和 Provider 远端拒绝凭据，并只对 CCR 能安全修复的本地漂移执行显式修复。

命令不得把“凭据字段存在”表述为“Provider 已接受该凭据”。当本地配置一致但用户仍收到 `401 INVALID_API_KEY` 时，应明确指出 Provider 有效性尚未由本命令验证，并给出更新/核验该 profile secret 的下一步。

## Background

- 用户在 SSH Ubuntu 上先运行 `ccr codex fix`。输出显示没有残留 app-server，环境中未设置 `CODEX_HOME` / `OPENAI_BASE_URL` / `OPENAI_API_KEY`，doctor 检查到的是 provider `owcl sub 订阅`、file credential store 和 `api_key` auth。
- 用户随后才在 `ccr` TUI 中切换到 `future`，Codex 0.145.0 请求 `https://www.futureapi.cc/responses` 后收到 `401 INVALID_API_KEY`。因此先前 doctor 并未检查报错的 `future` profile。
- 请求已经到达 `futureapi.cc`，证明 profile 的路由/base URL 已生效。2026-07-22 的无凭据只读探测返回 `401 API_KEY_REQUIRED`，而用户请求返回 `INVALID_API_KEY`，说明 Provider 收到了某个支持的凭据载体但拒绝了其中的 key；这不是“完全没有发送 key”。
- 当前 CCR 会把 `openai_api_key` profile 的 secret 写入 `~/.codex/auth.json` 的 `OPENAI_API_KEY`，强制 `cli_auth_credentials_store = "file"`，并清除旧 OAuth tokens。相关回归测试已通过。
- 当前 `ccr codex fix` 只高亮 doctor 的 provider/auth 字段；它不显示 CCR 当前 profile，不比较 `profile_secrets.json` 与 runtime credential，也不检查 route/profile 一致性。
- 当前 `CodexPlatform::stable_current_profile` 明确通过 `spec_matches_runtime_without_auth` 判断 profile，仅验证路由，不验证 secret；`AuthStateStatus::Valid` 对 API key 也只表示字段非空，不表示远端可用。
- 完整证据与排除过程见 `research/root-cause-analysis.md`。

## Requirements

### R1. 显示被诊断的准确上下文

- 在运行 doctor 前输出本次快照对应的 CCR profile 名称、runtime provider id/name、base URL、wire API、credential store 和认证来源。
- 同时读取 registry pointer、`profiles.toml` pointer 与实际 runtime；若三者不一致，必须报告漂移，不能静默把某个 pointer 当作事实。
- 输出应使用户一眼看出“doctor 检查发生在 profile 切换前还是切换后”。命令只对调用瞬间的状态负责。

### R2. 本地 route 与 credential 一致性检查

- 复用 Codex profile 的现有解析规则生成期望 runtime，不另写一套会漂移的认证模式推断。
- 比较当前 profile 与 `config.toml` 的 provider route：provider id/name、base URL、wire API、`requires_openai_auth` / `env_key`、credential store，以及与请求路由直接相关的字段。
- 对 `openai_api_key + file`，只在内存中比较 CCR profile secret 与 `auth.json.OPENAI_API_KEY`，输出 `match` / `missing` / `mismatch`，不得输出 secret、掩码片段、长度或哈希。
- 对 `provider_env_key`，比较 CCR 保存的 secret、声明的 `env_key` 与当前进程可见环境变量；无法读取实际凭据的 keyring/auto 模式明确标为 `unsupported`，不得假定一致。
- 对 `no_auth` 等不需要凭据的模式标为 `not_applicable`。
- 环境变量存在性继续显示，并补充 `CODEX_API_KEY` 与 active provider 声明的 `env_key`；所有 key 只显示存在性。不得把 `OPENAI_API_KEY=<未设置>` 误判为 file-based `auth.json` 缺失，也不得忽略可能覆盖单次 Codex 调用的环境凭据。

### R3. 分层结论与可执行建议

- 最终结论至少分为 `process_state`、`runtime_consistency`、`provider_auth_validity` 三层。
- `provider_auth_validity` 在没有真实 Provider 证据时必须为 `not_checked`，不能复用当前结构性 `AuthStateStatus::Valid` 的“Valid”措辞。
- 本地一致但外部仍报 `INVALID_API_KEY` 时，建议应指向“核验/更新当前 profile 保存的 Provider key，然后重新应用 profile”，不能继续建议清 app-server。
- 本地缺失/不一致时，输出具体的可本地修复原因；无当前 profile 时明确说明本命令只能诊断 runtime-only 状态。
- 保留 `codex doctor --json` 作为上游补充证据，但 CCR 自己的 reconciliation 结果不依赖 doctor schema 是否提供 secret 或 base URL。

### R4. 显式修复本地漂移

- 裸 `ccr codex fix` 不重写 `config.toml` / `auth.json`；新增 `--repair-runtime` 作为唯一 runtime 修复入口。
- `--repair-runtime` 仅在当前 profile 存在且本地 route/credential 漂移时，通过既有 `apply_profile` 原子提交路径重新应用该 profile，然后重新采样并证明一致。
- `--repair-runtime` 不生成、猜测、轮换或修改 CCR 保存的 profile secret；如果 profile secret 本身被 Provider 拒绝，只能提示用户更新 secret。
- `--dry-run` 继续禁止进程终止，并同时禁止 runtime 重写；与 `--repair-runtime` 一起使用时只展示将执行的本地修复。
- 修复继续使用现有备份、原子写和回滚机制，不直接拼接写入 `config.toml` / `auth.json`。

### R5. 退出状态与安全

- 保留现有退出码 `2`（app-server respawn）和 `127`（PATH 无 codex）。新增本地漂移且未修复/修复后仍不一致的稳定非零退出码，并在 CLI 帮助和文档中说明。
- 本地检查全部一致时可退出 `0`，但输出必须同时说明“仅本地一致，Provider key 有效性未验证”。
- 任何 stdout/stderr、tracing、临时报告和测试失败信息都不得包含 profile secret、`auth.json` 原文或可关联的 fingerprint。
- 不新增生产依赖；优先复用 `CodexPlatform`、`CodexRuntimeService`、`CodexAuthService` 和现有原子提交能力。

### R6. 文档与回归覆盖

- 更新 Codex CLI/reference 文档，说明调用顺序：先切换目标 profile，再运行 `ccr codex fix`；命令只验证本地一致性，不证明第三方 key 有效。
- 覆盖 route 一致、secret 一致、secret 缺失、secret 不同、环境变量模式、keyring 不可见、无当前 profile、dry-run 和显式修复后的二次验证。
- 添加用户本次顺序的回归场景：doctor 看到旧 profile 时不得让输出看起来像在诊断后来切换的 profile。

## Constraints

- 任务聚焦 `ccr-codex` 域诊断与 `ccr-cli` 命令呈现，不扩展到 Tauri UI 或 VS Code UI。
- 继续支持 `CCR_ROOT`、`CCR_CODEX_DIR` / `CODEX_HOME` 等既有路径覆盖；诊断必须报告实际解析路径。
- 官方 Codex 文档把 custom provider 的 `env_key` 与 `requires_openai_auth` 视为不同契约；实现必须按 profile 的实际 auth mode 判断来源，不能把所有第三方 key 都等同于 `OPENAI_API_KEY`。
- 实施必须等待规划评审与明确授权；用户于 2026-07-22 选择推荐方案并明确要求“开始实现”后，任务才进入 `in_progress`。

## Out Of Scope

- 默认向第三方 Provider 发送探测请求、调用 `/responses` / `/models` 或消耗额度。
- 判断、签发、轮换、撤销或恢复 Provider key；CCR 无法凭本地状态证明远端 key 有效。
- 自动修改 `futureapi.cc` 账号、订阅或服务端配置。
- 改造全部 `AuthStateStatus` 公共语义；本任务新增更准确的 runtime diagnostic 结论，避免扩大兼容面。
- 修改 profile 编辑 UI、secret store 格式或 Codex 上游 `auth.json` schema。

## Acceptance Criteria

- [x] `ccr codex fix` 明确显示调用瞬间的 CCR profile 与实际 runtime provider/base URL/auth source；用户示例中能看出旧 doctor 对应 `owcl sub 订阅` 而非后来切换的 `future`。
- [x] 本地诊断能在不输出任何 secret 衍生信息的前提下区分 route match、credential match、missing、mismatch、not-applicable 与 unsupported。
- [x] `auth.json` 中存在非空 key 时不再被描述成“Provider 凭据有效”；本地一致结果同时显示 `provider_auth_validity = not_checked`。
- [x] profile/runtime 漂移时给出稳定非零退出码与针对性建议；app-server、PATH 退出码保持兼容。
- [x] 裸 `ccr codex fix` 在发现本地漂移时只诊断并返回非零，不写 `config.toml` / `auth.json`。
- [x] `--repair-runtime` 只重放当前已保存 profile，使用既有原子提交/备份路径，并在修复后重新验证；它不会改变保存的 secret。
- [x] `--dry-run --repair-runtime` 不终止进程、不写 config/auth，只展示预期动作。
- [x] 本地一致但用户提供 `401 INVALID_API_KEY` 证据时，文档化结论指向 Provider 拒绝当前 key，而不是继续归因于 app-server 或未应用 URL。
- [x] 单元/集成测试覆盖 R2、R4、R5 的状态矩阵，并断言输出/错误中不含测试 secret。
- [x] `just fmt-check`、`cargo test -p ccr-codex -- --test-threads=1`、`cargo test -p ccr-cli --lib fix -- --test-threads=1`、`cargo test -p ccr --test commands -- --test-threads=1`、`just lint-strict` 全绿。
