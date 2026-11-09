# Design - `ccr codex fix` Provider auth diagnosis

## 1. Design Goal

把命令结果拆成三个独立事实：

```text
process_state            -> 是否存在/清除了旧 app-server
runtime_consistency      -> CCR profile 与 Codex config/auth 是否一致
provider_auth_validity   -> 远端是否验证过 key（MVP 始终 not_checked）
```

本地一致不等于 Provider key 有效。设计不新增默认网络请求，也不读取/展示 secret 的任何可关联表示。

## 2. Domain Boundary

### `ccr-codex`

负责构造非泄密诊断快照和执行既有 profile 重放：

- 读取 CCR registry pointer、`profiles.toml` pointer、profile + secret store；
- 读取 Codex `config.toml` / `auth.json` 与当前进程环境；
- 复用 `CodexPlatform::build_switch_spec` 的解析结果比较期望 route/auth；
- 返回结构化状态，不负责 CLI 文案、退出或启动 `codex doctor`。

不要让诊断先调用现有 `get_current_profile()`：`stable_current_profile()` 在 route mismatch 时会清除 registry pointer，破坏诊断证据。新增只读 inspection 入口必须先保存 raw pointers，且全程无修复副作用。

### `ccr-cli`

负责：

- 按顺序编排 cleanup -> inspect -> optional repair -> re-inspect -> doctor；
- 渲染人读结论和稳定退出码；
- 处理 `--dry-run` / `--repair-runtime`；
- 保留现有 doctor fallback 与报告落盘。

## 3. Data Contract

建议新增独立模型文件，例如 `crates/ccr-codex/src/models/codex_runtime_diagnostic.rs`：

```rust
pub enum RuntimeMatchStatus {
    Match,
    Missing,
    Mismatch,
    NotApplicable,
    Unsupported,
}

pub enum ProviderAuthValidity {
    NotChecked,
}

pub struct CodexRuntimeDiagnostic {
    pub registry_profile: Option<String>,
    pub profiles_file_profile: Option<String>,
    pub resolved_profile: Option<String>,
    pub runtime_provider_id: Option<String>,
    pub runtime_provider_name: Option<String>,
    pub base_url: Option<String>,
    pub wire_api: Option<String>,
    pub credential_store: CredentialStoreKind,
    pub auth_source: String,
    pub route_status: RuntimeMatchStatus,
    pub credential_status: RuntimeMatchStatus,
    pub provider_auth_validity: ProviderAuthValidity,
    pub issues: Vec<CodexRuntimeIssue>,
    pub repairable: bool,
}
```

字段不得包含 secret、fingerprint、长度或部分掩码。`auth_source` 只允许枚举值，例如 `auth_json:OPENAI_API_KEY`、`env:MISTRAL_API_KEY`、`keyring:unreadable`、`none`。

## 4. Read-Only Inspection Algorithm

1. 解析 `PlatformPaths` 与 `CodexConfigManager` 的实际路径。
2. 原样读取 registry current profile 与 `profiles.toml.current_config`，不调用会 reconcile/clear pointer 的 getter。
3. 若两个 pointer 相同且 profile 存在，选为 `resolved_profile`；否则记录 pointer issue，并仅在无歧义时继续比较。
4. 加载 profile 并通过 `CodexRuntimeService::overlay_profile_secrets` 恢复内存 secret。
5. 复用现有 switch-spec 构造逻辑得到期望 route、auth mode 与 credential store；不要复制 `resolve_profile_auth_mode` / auto-promote 规则。
6. 比较 route：
   - root `model_provider`；
   - custom provider `name` / `base_url` / `wire_api`；
   - `requires_openai_auth` / `env_key`；
   - `cli_auth_credentials_store`；
   - 与实际请求直接相关的 root model/provider 字段。
7. 比较 credential：
   - `OpenAiApiKey + file`：profile secret 与 `auth.json.OPENAI_API_KEY` trim 后直接相等；
   - `ProviderEnvKey`：profile secret 与当前进程中指定 env var 直接相等；
   - keyring/auto 中不可读取的实际值：`Unsupported`；
   - `NoAuth`：`NotApplicable`。
8. 额外检查 `CODEX_API_KEY`、`OPENAI_API_KEY` 和 active `env_key` 的存在性，按 Codex 契约标记潜在 override/冲突；不显示值。对于无法确认版本优先级的组合，报告 ambiguity 而非猜测。
9. 固定 `provider_auth_validity = NotChecked`。
10. 根据 issue 分类计算 `repairable`：仅 profile 存在、解析成功且问题可由重新 `apply_profile` 纠正时为 true。父 shell 缺少 provider env key 不能通过 `apply_profile` 修复，必须给出 export/source 动作。

直接相等比较的布尔结果只存在于当前进程内存，不进入 tracing 或错误格式化。

## 5. CLI Flow

最终 CLI 形态：

```text
ccr codex fix [--dry-run] [--repair-runtime]
```

执行顺序：

1. 运行现有 app-server cleanup；dry-run 只枚举。
2. 获取并渲染 `before` diagnostic snapshot。
3. 若 `--repair-runtime` 且 snapshot repairable：
   - dry-run：仅输出将重放的 profile；
   - 非 dry-run：调用既有 `CodexPlatform::apply_profile(profile)`；
   - 再获取 `after` snapshot，必须证明 route 和 credential 都为 Match。
4. 运行 `codex doctor --json`，在 doctor 标题旁标注 snapshot 的 profile 名称。
5. 渲染三层 summary。
6. 按优先级决定退出码：PATH missing 127；respawn 2；local drift 采用新码；其余 0。若多个错误并存，文档化固定优先级并测试。

如果最终本地一致，输出示例应类似：

```text
runtime_consistency = match
provider_auth_validity = not_checked
说明：CCR 已确认 profile secret 与 Codex runtime 一致，但未验证第三方 Provider 是否接受该 key。
```

## 6. Repair Semantics

`--repair-runtime` 只调用既有 `apply_profile`，不单独写文件：

- 保留 `CodexRuntimeCommitPlan` 的备份、原子写与回滚；
- 重放当前保存的 secret，不改变 secret store；
- 仅修复 runtime config/auth 漂移；
- 修复后强制二次 inspection；
- profile secret 缺失或 pointer 冲突时拒绝修复并给出具体原因。

裸命令不自动写 runtime；只有显式 `--repair-runtime` 才进入上述修复分支。

## 7. Security

- 不在输出、JSON、temp report、tracing 或测试 panic 中包含 secret 派生值。
- 比较函数的 Debug 输出不得携带输入值；issue 只带字段名和状态。
- doctor 原有转发继续假定上游脱敏，但新 CCR snapshot 自己实现明确 allowlist。
- 不默认调用第三方 endpoint，避免额度消耗、非幂等行为和 provider 兼容性猜测。
- 测试使用 sentinel secret，并断言捕获的 stdout/stderr/serialized diagnostic 均不包含 sentinel。

## 8. Compatibility And Rollback

- 现有 `ccr codex fix` 命令和 `--dry-run` 保留；新增 flag 向后兼容。
- 现有 app-server 匹配、doctor fallback、报告文件和退出码 2/127 保留。
- 不改变现有 `AuthStateStatus` 序列化/API 契约。
- 回滚时删除 diagnostic model/inspection 入口和 CLI 新渲染/flag；profile、auth 与 secret store 无迁移。

## 9. Test Matrix

| Case | Route | Credential | Expected |
| --- | --- | --- | --- |
| current profile fully applied | match | match | local match, provider not_checked |
| auth.json missing key | match | missing | repairable drift |
| auth.json contains another key | match | mismatch | repairable drift, no secret leak |
| config points to another provider | mismatch | any | route drift |
| registry/profiles pointers differ | ambiguous | unknown | no auto repair |
| provider env key present and equal | match | match | local match |
| provider env key absent | match | missing | actionable env issue |
| `CODEX_API_KEY` or another credential override is present | match | ambiguous/conflict | show existence only, no false match |
| keyring/auto actual value hidden | match | unsupported | explicit limitation |
| no active profile | runtime-only | structural | diagnostic only |
| dry-run + repair | mismatch | mismatch | no writes, planned action shown |
| repair succeeds | mismatch -> match | mismatch -> match | second snapshot proves result |
| repair fails/rolls back | mismatch | mismatch | nonzero, original files preserved |

## 10. External Provider Error Mapping

MVP 不主动访问 Provider。文档/输出只做条件化映射：

- `API_KEY_REQUIRED`：凭据未传输或来源缺失，先看 local credential status；
- `INVALID_API_KEY` 且 local mismatch：先 `--repair-runtime`；
- `INVALID_API_KEY` 且 local match：CCR 保存的 key 被 Provider 拒绝，更新该 profile secret；
- 其他 4xx/5xx：不硬编码为 auth 问题，保留原始 Provider/Codex 错误作为外部证据。
