# `ccr codex fix` 后仍出现 `INVALID_API_KEY` 的根因分析

## 1. 结论摘要

本次失败不是“旧 app-server 继续使用旧 URL/Key”的证据，反而已有三项证据排除了这条主因：

1. `ccr codex fix` 在调用时未发现 app-server。
2. 随后启动的 Codex 请求已经到达 `https://www.futureapi.cc/responses`，说明切换到 `future` 后的新 route 生效。
3. `futureapi.cc` 对无凭据请求返回 `API_KEY_REQUIRED`，而用户请求返回 `INVALID_API_KEY`，说明服务端收到了某个凭据但拒绝了它。

高置信结论是：**`futureapi.cc` 拒绝了 Codex 实际提交的 key**。仍需本地比较才能进一步区分：

- CCR 保存的 `future` secret 本身已失效、被撤销、复制错误或不属于该服务；
- `profile_secrets.json` 与 `~/.codex/auth.json` 发生漂移，Codex 提交的并不是 CCR 当前保存的 secret；
- `future` 的 auth mode 与 Provider 要求不一致。

现有 `ccr codex fix` 无法区分这三种情况。它只证明进程状态和 Codex 识别到某种 API-key auth，不证明 key 与当前 profile 一致，更不证明 Provider 接受该 key。

## 2. 事件顺序证据

用户提供的终端顺序是：

1. `ccr codex fix`
   - 未发现 Codex app-server；
   - 当前环境变量 `CODEX_HOME` / `OPENAI_BASE_URL` / `OPENAI_API_KEY` 均未设置；
   - doctor 报告 provider name 为 `owcl sub 订阅`；
   - credential store 为 `File`，stored auth mode 为 `api_key`。
2. `git push`，与 Codex runtime auth 无关。
3. 运行 `ccr` TUI，退出时显示 `[Codex Profile] Switched to profile: future`。
4. 再启动 Codex 0.145.0，请求 `https://www.futureapi.cc/responses`，收到 `401 INVALID_API_KEY`。

因此 doctor 快照发生在 `future` 切换之前；它检查的是 `owcl sub 订阅`，不能作为 `future` profile 的诊断结果。现有输出没有显示 CCR profile 名称，容易让用户把两个时点误认为同一个运行时。

## 3. 仓库实现证据

### 3.1 Profile switch 确实写入 file-based API key

- `crates/ccr-codex/src/platforms/codex.rs:776-850`
  - profile secret 先 `trim`；
  - `openai_api_key` 解析为 `WriteOpenAiApiKey`；
  - 该模式默认强制 `cli_auth_credentials_store = "file"`。
- `crates/ccr-codex/src/platforms/codex.rs:901-1040`
  - custom route 写入 `model_provider = "custom"` 和 provider table；
  - API-key 选择会清除 OAuth tokens，并把 secret 写入 `OPENAI_API_KEY`；
  - config 与 auth 通过 `CodexRuntimeCommitPlan` 提交。
- `crates/ccr-codex/src/services/codex_runtime_service.rs:219-305`
  - commit 使用备份和原子写；auth 写失败会回滚 config/auth。
- `crates/ccr-codex/src/services/codex_runtime_service.rs:91-150`
  - profile secret 从 `profile_secrets.json` overlay 到内存 profile；保存时单独持久化。

本次本地验证：

```text
cargo test -p ccr --test commands codex_profile_switch_and_off_are_consistent_and_off_keeps_auth_json -- --test-threads=1
1 passed

cargo test -p ccr-codex test_third_party_default_auth_key_clears_stale_chatgpt_auth_metadata -- --test-threads=1
1 passed
```

这证明当前仓库的标准切换路径会写入 key；它不能证明远端 Ubuntu 上 `future` 保存的 key 内容正确或仍有效。

### 3.2 现有“current/valid”只检查结构，不检查一致性或远端有效性

- `crates/ccr-codex/src/platforms/codex.rs:1293-1343` 的 `spec_matches_runtime_without_auth` 只比较 route 字段。
- `crates/ccr-codex/src/platforms/codex.rs:1389-1429` 用上述 auth-blind 比较决定当前 profile。
- `crates/ccr-codex/src/services/codex_auth_service.rs:281-342` 只要 `OPENAI_API_KEY` 是非空字符串，就把 `AuthStateStatus` 标为 `Valid`。

所以当前界面/命令中的 `Valid` 实际含义是“本地存在可识别字段”，不是“key 与 profile 相同”或“Provider 已验证”。

### 3.3 `ccr codex fix` 没有 reconciliation

- `crates/ccr-cli/src/commands/codex/fix.rs:33-57` 只编排进程清理、环境提示和 `codex doctor`。
- `crates/ccr-cli/src/commands/codex/fix.rs:119-127` 只显示 `OPENAI_API_KEY` 等三个环境变量；它遗漏了单次 Codex 调用可使用的 `CODEX_API_KEY` 和 custom provider 自己声明的 `env_key`。file store 下环境变量未设置仍可能是正常状态。
- `crates/ccr-cli/src/commands/codex/fix.rs:242-275` 只从 doctor JSON 按标签提取 provider/auth 等字段，没有读取 CCR profile 或比较 secret。

上一任务 `.trellis/tasks/archive/2026-07/07-21-ccr-codex-fix/prd.md` 还明确把 `auth.json` / `config.toml` 修改和 profile 路由排除在外。因此当前命令名虽然是 `fix`，实际只修复一种进程残留场景。

## 4. Provider 侧证据

2026-07-22 对用户截图中的同一域名进行了不携带任何 key 的只读 GET 探测：

```text
GET https://www.futureapi.cc/responses
401 {"code":"API_KEY_REQUIRED", ...}

GET https://www.futureapi.cc/v1/responses
401 {"code":"API_KEY_REQUIRED", ...}
```

用户的 Codex 请求得到：

```text
401 {"code":"INVALID_API_KEY","message":"Invalid API key"}
```

服务端明确区分“缺少 key”和“key 无效”。结合 Codex 请求已到达新的 futureapi route，可排除：

- base URL 完全没有切换；
- 请求完全没有携带任何受支持凭据；
- 旧 app-server 是本次唯一根因。

因为没有读取或发送用户的 secret，本次调查不能验证该 key 是否过期、被撤销、属于错误账号，或 Provider 是否要求不同 auth mode。

## 5. Codex 官方契约核对

2026-07-22 获取的官方 Codex manual 说明：

- `cli_auth_credentials_store = "file"` 时，登录凭据位于 `CODEX_HOME/auth.json`；环境中没有 `OPENAI_API_KEY` 并不等于 file credential 缺失。
- custom provider 的 provider-owned API key 标准配置是 `env_key`；`requires_openai_auth` 只应当用于由 OpenAI auth 支持的 provider。
- custom provider 可以选择 `env_key`、command-backed auth 或 `requires_openai_auth`，诊断必须按实际 mode 判断 key 来源，不能统一用环境变量存在性推断。
- 非交互/单次调用还可以使用 `CODEX_API_KEY`；诊断至少需要报告它的存在性，不能只检查 `OPENAI_API_KEY`。

参考：

- <https://learn.chatgpt.com/docs/config-file/config-advanced#custom-model-providers>
- <https://learn.chatgpt.com/docs/auth>

CCR 目前允许第三方 profile 通过 `openai_api_key + requires_openai_auth` 把一个 opaque bearer key 放入 Codex 的 file auth cache。这条路径有测试覆盖，但诊断输出必须把它描述为“本地传输方式”，不能据此宣称 Provider key 有效。

## 6. 产品缺口

| 层级 | 当前能看到 | 当前看不到 | 本次应补齐 |
| --- | --- | --- | --- |
| 进程 | app-server found/respawned | 与哪个 profile 对应 | 保留现状并纳入总判定 |
| CCR profile | TUI 切换成功 | doctor 调用时的 profile 名称 | 显示 raw pointers 与 active profile |
| Runtime route | doctor provider 摘要 | profile 与 config 的逐项一致性 | route reconciliation |
| Runtime secret | auth mode=`api_key` | secret 是否缺失/与 profile 不同 | 仅内存相等比较 |
| Provider | 用户看到 401 | `fix` 不知道远端是否接受 key | 明示 `not_checked`，给出条件化建议 |

## 7. 推荐范围

MVP 应做：

1. 在 `ccr-codex` 提供非泄密的 current-profile/runtime diagnostic snapshot。
2. 在 `ccr codex fix` 中先显示该 snapshot，再运行 doctor。
3. 对本地 route/secret 漂移提供显式 `--repair-runtime`，复用 `apply_profile` 后二次验证。
4. 把 Provider 有效性固定描述为 `not_checked`，并针对已有 `401 INVALID_API_KEY` 给出“更新 profile secret”的建议。
5. 文档明确要求先切换目标 profile，再运行 `fix`。

MVP 不应做：

- 默认向第三方 API 发请求；
- 通过 key 前缀、后缀、长度或哈希展示 identity；
- 自动生成/轮换 Provider key；
- 把现有所有 `AuthStateStatus::Valid` 改名，扩大 API 兼容面。

## 8. 已决策

裸 `ccr codex fix` 遇到本地漂移时保持诊断 + 进程清理，不重写 runtime。只有用户显式传入 `--repair-runtime` 才可通过既有 `apply_profile` 路径写 `config.toml` / `auth.json`；这既保留现有副作用边界，也能在 SSH 环境中一条命令完成明确授权的本地修复。
