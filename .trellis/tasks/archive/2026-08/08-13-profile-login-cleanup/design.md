# Design: profile 登录预备清理

## Architecture

单一写盘核：`profile_off_for_platform(Platform) -> ProfileOffResult`。

CLI、TUI、Tauri、UI 只调用该核。禁止各面复制清字段列表。

```
CLI / TUI / Tauri
        │
        ▼
profile_off_for_platform
        │
        ├── Claude: 托管 env + 指针 + 诊断
        ├── Codex:  runtime 路由 + auth 快照/残留 + 指针
        └── Grok:   restore entry state + 指针
```

`ProfileOffResult` 扩展为三家共用：

- `platform`
- `previous_profile: Option<String>`
- `changed: bool`
- `runtime_mode: &'static str`（`official_auth` / `grok_native`）
- `auth_outcome: Option<ClaudeAuthActionOutcome>`（仅 Claude）
- `warnings: Vec<String>`（Grok 失败关闭走 `Err`，不放这里）

Grok 从「暂不支持」改为走该函数，内部仍调用 `GrokPlatform::clear_active_profile_runtime`。Tauri `grok_profile_off` 改为调用共享核，响应枚举 `Off { changed, previous_profile }` 保持不变。

## Trigger: 何时算「有东西可清」

`needs_login_prep(platform) -> bool`，off 与 UI `canOff` 共用。

| 平台 | true 当 |
| --- | --- |
| Claude | registry/`profiles.toml` 指针非空，或 `settings.json` 含任意 `CCR_MANAGED_KEYS` |
| Codex | 指针非空，或存在入口 auth 快照，或 `config.toml` 含 CCR 第三方运行时（`experimental_bearer_token` / `forced_login_method=api` 且 custom provider 非官方 OpenAI 形态），或（指针或第三方 runtime）且 `auth.json` 含 `OPENAI_API_KEY` |
| Grok | 沿用 `inspect_activation_state`：`Active` / `Drifted` / `UnsafeMissingEntryState` |

无指针且仅为官方 API-key / 官方 OAuth 时，Codex 不得改 `auth.json`。

## Data flow

### Claude

1. 备份 settings、registry、profiles。
2. `clear_ccr_managed_vars`。
3. 清 registry `current_profile` 与 `current_config`。
4. `ClaudeAuthService::action_outcome`。

不改 `.credentials.json`。不删用户 `ANTHROPIC_API_KEY`。

### Codex

1. 备份 `profiles.toml`；runtime 仍由 Codex 自己的 lock/atomic 写。
2. `apply_runtime_route_without_auth(Official, File)`：去掉 `forced_login_method`、`preferred_auth_method`、`model_catalog_json`、bearer。
3. Auth：
   - 有 `profile_entry_auth_state.json` → `restore_profile_entry_auth_state`。
   - 无快照，且（指针或第三方 runtime），且 `auth.json` 有 `OPENAI_API_KEY`、无 `tokens` → 删除该 key；对象变空则删文件。
   - 其余 → 不改 `auth.json`。
4. 清指针。

### Grok

不变：有入口状态则 restore；缺失且仍有意图/managed shape 则 `Err`，文件不变。CLI/Tauri 把该 `Err` 原样交给用户。

## TUI

`App::apply_selected`（仅 `TabVariant::Profile`）：

1. `profile_off_for_platform(tab.platform)`。
2. 失败 → toast 错误，不 apply。
3. 成功 → 现有 `apply_profile`。

Claude/Codex Auth `switch_selected_account`：

1. 对应平台 `profile_off_for_platform`。
2. 失败 → 不 `switch_account`。
3. 成功 → 现有 switch。Claude toast 可合并 off 的 `cleared_managed_sources` 与 switch 的清理数，避免重复吓人；实现时若 switch 在 off 后看到无托管键，只会报「已切换」，可接受。

Profile 页按键 `o`：只调用 off，刷新列表。Footer 仅在 Profile tab 显示 `o` / 退出 Profile。窄宽度 footer 可省略文案只留键。

Grok 无独立 Auth tab。OpenCode Auth 不调用本核。

## Tauri / UI

新命令：

- `claude_profile_off` → 映射 `ProfileOffResult`（含 `warnings` / `remaining_suppressors`，无密钥）。
- `codex_profile_off` → `{ ok, changed, previous_profile, runtime_mode }`。

走 `handler_registry` 生成绑定。domain 包装放在 `ccr-ui/src/api/domains/claude.ts` / `codex.ts`。更新冻结命令计数。

I/O 必须 `spawn_blocking`。非 local 环境：与 Grok 一样返回 unsupported，不写盘。

### UI 放置（D7）

Profiles（Claude / Codex / Grok）：

- Header 与 StatStrip 之间的横幅。
- 显示条件：`needs_login_prep`（可由 current/list/runtime 已有字段推导；缺字段时给 current DTO 加 `can_off: bool`，禁止前端猜文件）。
- 按钮：退出 Profile。确认 `type=warning`，走现有 `useConfirmAction`（与 Grok `handleOff` 一致）。
- 命令面板 `__off`。

不把 Off 放进 Header 溢出菜单，不放进平台 Home。

Auth：

- Claude：诊断面板 header 在 `can_off` 时放同一按钮。
- Codex：运行时/账号区顶部在 `can_off` 时放同一按钮。
- 调用与 Profiles 相同的 domain API。

Grok 横幅已存在：只把后端从直接 `clear_active_profile_runtime` 换成共享核（若行为字节兼容可保持 Tauri 函数体只改内部调用）。

可选：抽出 `ProfilesRuntimeBanner`。若抽取会扩大 Grok 回归面，允许 Claude/Codex 先复制 Grok 横幅结构，三页 class 仍走 `--cp-*`。

## Compatibility

- CLI 子命令名仍是 `off`。JSON 字段只增不改：现有 `ok` / `changed` / `previous_profile` / `runtime_mode` 保留；Claude 已有 `warnings`。
- Grok Tauri `status: off` 不变。
- `ccr clear` 不改语义。
- 不迁移旧备份目录名 `profile-off`。

## Trade-offs

| 选择 | 代价 |
| --- | --- |
| TUI 每次 apply 先 off | 多一次写盘；入口快照重建。保证跨 profile 无串档 |
| 无快照只在「指针或第三方 runtime」时删 `OPENAI_API_KEY` | 快照丢失时官方 API key 已被 switch 覆盖，用户需重新登录。避免误删纯官方 API-key 会话 |
| Grok 仍失败关闭 | 入口状态缺失时 UI/TUI/CLI 都不能自动猜删 |

## Rollback

- Claude/Codex 继续用 `ProfileOffBackup` RAII。
- Grok 入口状态即回滚源；缺失则不写。
- 发布后若需撤回 UI，可只藏按钮；核与 CLI 保留。

## Tests

- CLI：扩展 `crates/ccr/tests/commands/{claude,codex,grok}_profile.rs`：无指针托管残留、Codex 无快照 API key、无指针官方 API key 不动、Grok 走共享核。
- 单测：`profile_off.rs`、`CodexPlatform` auth 残留分支、Grok fail-closed。
- TUI：apply/switch 在 off 失败时不继续；`o` 只 off。
- UI：横幅可见性、确认取消、Auth 按钮；Grok 现有 off 用例仍过。
- 密钥：stdout / JSON / DTO 断言不含 token。
