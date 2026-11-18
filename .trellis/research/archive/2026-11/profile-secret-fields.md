# Profile 密钥字段与导出遮蔽调查

- 工单：#169（地图 #167 的子项）
- 调查基线：`origin/dev` @ `16cbc6fa`
- 证据范围：仅源码与测试。没有读取任何真实用户配置，也没有运行会打印配置的 `ccr` 命令。
- 行号均相对上述基线提交。标注"代码推断"的结论只来自读代码，没有现成测试覆盖，也没有执行验证。

## 1. 规范 TOML 是怎么来的

三个平台共用同一套目录模型，区别只在适配器保存前后做了什么。

- 目录落在各平台目录下的 `profiles.db`（SQLite）。`profiles` 表里只有 `auth_token` 一列是密钥列，以明文 `TEXT` 存储；其它未知字段整体序列化成一段 TOML 放进 `extra_toml` 列。
  `crates/ccr-config/src/managers/config/catalog/mod.rs:42-73`
- `ConfigManager::canonical_toml` → `canonical_document` → `render_catalog` / `render_profile`：先把 `extra_toml` 解析成表，再把类型化列（含 `auth_token`）逐个写回，全部是明文。
  `crates/ccr-config/src/managers/config/manager.rs:85-88`，`catalog/mod.rs:509-525`，`catalog/mod.rs:1608-1660`
- 读入方向相同：`profile_from_table` 取走已知键，剩下的整张表原样进 `extra_toml`，包括嵌套表。
  `catalog/mod.rs:1271-1345`
- 内存模型 `ConfigSection` 只有 `auth_token` 是 `Option<Secret>`，其余未知字段进 `other: IndexMap<String, toml::Value>`（`#[serde(flatten)]`）。
  `crates/ccr-config/src/managers/config/types.rs:54-60`，`types.rs:154-156`
- 平台适配器看到的 `ProfileConfig.platform_data` 就是 `other` 的 JSON 投影，两个方向都是无损搬运，没有任何密钥识别。
  `crates/ccr-config/src/platforms/base.rs:119-153`，`base.rs:157-199`

结论：在目录模型这一层，系统只认识一个密钥字段 `auth_token`。其它任何承载密钥的字段都是"普通扩展字段"。

## 2. `export_config(include_secrets = false)` 实际做了什么

`crates/ccr-config/src/services/config_service.rs:359-383`

- `include_secrets = true`：直接返回 `canonical_toml()` 的文本。
- `include_secrets = false`：`load()` 得到 `CcsConfig`，对每个 section 只做一件事——把 `auth_token` 换成 `Secret::new(token.to_string())`，再 `toml::to_string_pretty(&config)`。
- `Secret` 的 `Display` 调 `mask_sensitive`：长度不超过 10 个字符时全部替换为 `*`，否则保留前 4 和后 4 个字符，中间用 `...`。
  `crates/ccr-core/src/core/secret.rs:81-85`，`crates/ccr-core/src/utils/mask.rs:20-31`

因此"遮蔽"是把 `auth_token` 改写成部分可见的掩码串，不是删除字段；`other` 里的内容完全没有被触碰。

调用方：

| 入口 | 平台范围 | 位置 |
| --- | --- | --- |
| `ccr export [--no-secrets] [--platform <p>]` | 任意平台，默认 `claude` | `crates/ccr-cli/src/commands/data/export.rs:24-40`、`:93-99`，`crates/ccr-cli/src/cli/definitions.rs:222-240` |
| Tauri `export_config(include_secrets)` | 固定 Claude（`ConfigService::with_default`） | `ccr-ui/src-tauri/src/commands/config.rs:274-291` |
| Tauri `claude_export_profiles` / `codex_export_profiles` | 只支持含密钥导出，`include_secrets = false` 直接报 `Redacted profiles export is not supported` | `crates/ccr-app/src/config/source.rs:57-69`，`ccr-ui/src-tauri/src/commands/profile_lifecycle.rs:148-165` |

## 3. 各平台字段表

"遮蔽"一列指今天的 `--no-secrets`（即 `export_config(false)`）是否处理该字段。

### 3.1 Claude

| 字段 | 承载内容 | 存放位置 | 今天被遮蔽 | 源码位置 |
| --- | --- | --- | --- | --- |
| `auth_token` | API 密钥明文；应用时写入 Claude `settings.json` 的 `env.ANTHROPIC_AUTH_TOKEN` | 目录 `profiles.auth_token` 列，规范 TOML 中为明文 | 是（部分掩码，非删除） | `types.rs:54-60`、`types.rs:328-334`；`crates/ccr-cli/src/platforms/claude.rs:45-57`、`:246-259` |
| `base_url` | 端点地址；若用户把凭据写进 userinfo 或查询串，则间接带密钥 | 目录 `base_url` 列 | 否 | `types.rs:50-52` |
| `auth_mode`（`subscription` / `api_key`） | 不是密钥。`subscription` 表示密钥不在 profile 里，由 Claude Code 自己的登录态提供 | 目录 `extra_toml` | 不适用 | `claude.rs:44`、`:176-192` |
| 任意未知扩展字段 | 用户或旧版本写入的任何键，可能是密钥 | 目录 `extra_toml` | 否 | `types.rs:154-156`，`catalog/mod.rs:1338-1342` |

目录之外：订阅模式依赖 Claude 运行时的 `.credentials.json`（`crates/ccr-config/src/models/claude_runtime_paths.rs:36`）和 Claude 自己管理的登录态，CCR 的 profile 导出不包含它。

### 3.2 Codex

| 字段 | 承载内容 | 存放位置 | 今天被遮蔽 | 源码位置 |
| --- | --- | --- | --- | --- |
| `auth_token`（经适配器保存，认证模式为 `openai_api_key` / `provider_env_key` / `provider_bearer_token`） | OpenAI API Key、provider 密钥或 bearer token | **不在目录里**。保存时写入平台目录下的 `profile_secrets.json`（明文 JSON，按 profile 名索引），随后把目录副本的 `auth_token` 清成 `None`；读取时再叠加回来 | 不适用：规范 TOML 里本来就没有 | `crates/ccr-codex/src/platforms/codex.rs:2144-2174`；`crates/ccr-codex/src/services/codex_runtime_service.rs:98-100`、`:114-144`、`:146-178`、`:236-248`、`:394-411`；`crates/ccr-codex/src/models/codex_auth.rs:65-93` |
| `auth_token`（残留在目录中的情况） | 同上 | 目录 `auth_token` 列。出现条件：认证模式解析为 `openai_chatgpt` 或 `no_auth` 时不清除；或经 `ccr import --platform codex`、`replace_toml` 等绕过 `CodexPlatform::save_profile` 的路径写入（代码推断） | 是（部分掩码） | `codex_runtime_service.rs:236-248`；`config_service.rs:388-438`；`crates/ccr-cli/src/commands/data/import.rs:160-174` |
| `env_key` | 环境变量**名**，不是值。值是上面那条 `auth_token`，存在 `profile_secrets.json` 里（`provider_env_key` 模式要求两者同时存在） | 目录 `extra_toml`；`profile_secrets.json` 里另有一份 `env_key` | 否（名字照常导出） | `codex.rs:535-537`、`:2284-2294`；`codex_runtime_service.rs:188-219` |
| `auth_mode`、`requires_openai_auth`、`openai_login_method`、`forced_login_method`、`preferred_auth_method` | 不是密钥，决定密钥从哪里取 | 目录 `extra_toml` | 不适用 | `codex.rs:510-598` |
| `cli_auth_credentials_store`（`file` / `keyring` / `auto`） | 不是密钥，指向 Codex CLI 的凭据存储方式 | 目录 `extra_toml` | 不适用 | `codex.rs:620-635` |
| `base_url` | 端点地址，可能内嵌凭据 | 目录 `base_url` 列 | 否 | `types.rs:50-52` |
| 任意未知扩展字段 | 同 Claude | 目录 `extra_toml` | 否 | `types.rs:154-156` |

目录之外的密钥位置：

- `profile_secrets.json`：上表第一行，Codex 第三方 / API Key profile 的唯一密钥来源。`rg` 全仓库只有运行时服务、生命周期回滚清单和测试引用它，没有任何导出或导入路径读写它（`crates/ccr-cli/src/application/profile_lifecycle.rs:502-520`）。
- Codex 运行时 `auth.json` 与系统钥匙链：`openai_chatgpt` 模式的 OAuth 令牌在这里，由 `codex login` 管理；非 `file` 存储时 CCR 只删不写。
  `codex.rs:1232-1262`，`codex_runtime_service.rs:250-262`
- Codex 运行时 `config.toml` 的 `experimental_bearer_token`：`provider_bearer_token` 模式应用时写入，是运行时落点而非 profile 字段。
  `codex.rs:1217-1222`
- `profile_entry_auth_state.json`：进入 profile 前的认证快照，用于恢复，不参与 profile 可用性。
  `codex.rs:34`、`:121-123`

### 3.3 Grok

| 字段 | 承载内容 | 存放位置 | 今天被遮蔽 | 源码位置 |
| --- | --- | --- | --- | --- |
| `api_key` | 内联 API 密钥明文；应用时写入 Grok `config.toml` 的 `[model.custom].api_key` | 目录 `extra_toml`（经 `platform_data` → `other`），规范 TOML 中为明文 | **否** | `crates/ccr-cli/src/platforms/grok.rs:22-37`、`:673-684`、`:1088-1094`；创建路径 `crates/ccr-cli/src/commands/platform/profile.rs:153-176` |
| `auth_token` | `api_key` 的兼容写法，语义相同；与 `api_key` 互斥 | 目录 `auth_token` 列 | 是（部分掩码） | `grok.rs:686-701`；`profile.rs:137-139` |
| `env_key` | 单个环境变量**名**；密钥值在目标机器的进程环境里，CCR 不保存 | 目录 `extra_toml` | 否（名字照常导出） | `grok.rs:659-671`、`:1095-1100` |
| （无凭据字段，`session` 模式） | 官方 profile 不允许带凭据，由 Grok 自己的会话或全局 `XAI_API_KEY` / `GROK_CODE_XAI_API_KEY` 提供 | 不在目录里 | 不适用 | `grok.rs:540-553`、`:1352-1358`、`:1383-1385` |
| `base_url` | 端点地址，可能内嵌凭据。Grok 适配器专门有 `safe_base_url_for_display` 在展示时剥离 userinfo 和查询串，说明这种情况被认为会出现；但该函数只用于展示，不用于导出 | 目录 `base_url` 列 | 否 | `grok.rs:555-562` |
| 任意未知扩展字段 | 同 Claude | 目录 `extra_toml` | 否 | `types.rs:154-156` |

目录之外的密钥位置：

- Grok 运行时 `config.toml`：应用后的 `[model.custom].api_key` 明文落点。
- `profile_entry_config_state.json`：进入 profile 前的运行时配置整文件快照（`content` 字段），如果用户原本的 `config.toml` 里有内联密钥，这里就有一份明文副本。用于 `off` 恢复，不参与 profile 可用性。
  `grok.rs:19`、`:99-107`、`:641-645`
- Grok 自己的 `auth.json` / `mcp_credentials.json`：profile 操作明确不读不写。
  `.trellis/spec/ccr-cli/backend/grok-profile-runtime.md:92-100`

## 4. 未知字段与扩展字段如何序列化

- 落库：`extra_from_other` 把 `other` 原样序列化为 TOML 字符串，写入 `extra_toml` 列。`catalog/mod.rs:1537-1546`
- 规范 TOML：`render_profile` 以 `extra_toml` 解析出的表为底，再覆盖类型化键。`catalog/mod.rs:1645-1647`
- `--no-secrets` 导出：`other` 带 `#[serde(flatten)]`，`toml::to_string_pretty` 直接平铺输出，遮蔽循环不遍历它。`config_service.rs:373-377`
- 嵌套表同样整张透传（`profile_from_table` 剩余表整体保留）。

所以任何不叫 `auth_token` 的键，无论含义，都会以明文出现在两种导出里。契约里对未知字段的要求是"保留"，没有遮蔽要求：`.trellis/spec/ccr-config/backend/profile-repository.md:58`。同一份契约也写明"All profile content is secret"（`:41`），与 `--no-secrets` 只处理一个字段的现状不一致。

## 5. 缺口清单

1. **Grok `api_key` 不被遮蔽。** 它是 Grok 当前推荐的内联密钥字段，却走扩展字段通道，`ccr export --platform grok --no-secrets` 会原样输出明文。（代码推断，无测试。）
2. **所有扩展字段都不被遮蔽**，包括嵌套表。任何平台只要 profile 里有不叫 `auth_token` 的密钥键，都会泄漏。
3. **`base_url` 内嵌凭据不被处理。** 展示路径有剥离逻辑，导出路径没有。
4. **"遮蔽"不是删除。** 超过 10 个字符的令牌保留首尾各 4 个字符。CLI 的帮助文本和结束提示却写的是"排除敏感信息""敏感信息已移除"（`definitions.rs:232-235`，`export.rs:72-75`）。
5. **`--no-secrets` 的产物不能安全回导。** 文件里没有任何"已遮蔽"标记，`import_config` 会把掩码串当成真实 `auth_token` 写入；合并模式下同名 profile 被整段替换（`config_service.rs:507-511`），本机原有的真实令牌会被掩码串覆盖。（代码推断，无测试。）
6. **Codex 的密钥根本不在导出范围内。** 经适配器保存的 Codex profile，密钥只在 `profile_secrets.json`。`ccr export --platform codex` 即使不加 `--no-secrets` 也不含密钥，但结束提示仍会说"已包含敏感信息(API密钥)"。没有任何路径导出或导入 `profile_secrets.json`。
7. **Codex 目录里是否有 `auth_token` 取决于写入路径。** 走适配器保存会被清除；走 `ccr import` / `replace_toml` 直写目录则保留；认证模式解析为 `openai_chatgpt` / `no_auth` 时也不清除。同一平台有两种密钥落点，遮蔽逻辑只覆盖其中一种。（代码推断。）
8. **两种导出模式的文档形状不同。** 含密钥走 `render_catalog`；不含密钥走 `CcsConfig` 的 serde 序列化。后者不携带目录级的顶层非表扩展键（`document.extra_toml`），`default_config` 缺失时会被填成第一个 profile 名，`[settings]` 总是输出。两份导出不是"同一文档去掉密钥"的关系。（代码推断，对照 `catalog/mod.rs:1127-1152` 与 `:1608-1629`。）
9. **桌面端的专用 profile 导出完全不支持遮蔽**，只能导出含密钥版本；通用的 Tauri `export_config` 只针对 Claude。
10. **测试缺口**，见第 7 节。

## 6. 跨机器可移植性

问题：在 A 机器导出（含密钥），在 B 机器导入后，profile 能否直接使用？

| 平台 | 结论 | 说明 |
| --- | --- | --- |
| Claude | **API Key 模式：是。订阅模式：否。** | `auth_token` 在目录里，随导出走。订阅模式的 profile 本身不含密钥，B 机器需要自己完成 Claude 登录。 |
| Codex | **否**（`no_auth` 模式除外）。 | `openai_api_key` / `provider_env_key` / `provider_bearer_token` 的密钥在 `profile_secrets.json`，不在导出里；导入后 `load_profiles` 叠加不到密钥，`validate_profile` 会因缺少 `auth_token` 报错（`codex.rs:2276-2301`）。`openai_chatgpt` 模式依赖 B 机器上的 `codex login`。只有导出时目录里恰好残留 `auth_token`（缺口 7）才可用。 |
| Grok | **内联密钥（`api_key` 或 `auth_token`）：是。`env_key`：否。`session`：否。** | 内联密钥在目录里，随导出走。`env_key` 只导出变量名，B 机器必须自己定义该环境变量。`session` 依赖 B 机器上 Grok 自己的登录或全局环境变量。 |

使用 `--no-secrets` 导出时，三个平台都不可移植，并且存在缺口 5 的覆盖风险；Grok 的 `api_key` profile 是例外——因为它没被遮蔽，反而仍然可用，这正是缺口 1。

## 7. 现有测试覆盖

直接覆盖 `export_config` 遮蔽行为的只有一个测试：

| 场景 | 测试 | 位置 |
| --- | --- | --- |
| `export_config(true)` 含明文 `auth_token`；`export_config(false)` 不含明文且包含 `...` | `test_config_service_export_import` | `crates/ccr/tests/workflows/services.rs:165-217` |

它只用了一个 Claude 形态、只带 `auth_token` 的 section，没有 `other` 字段，没有指定平台，也没有把遮蔽产物再导回。

相邻但不等同于导出遮蔽的覆盖：

| 场景 | 测试 | 位置 |
| --- | --- | --- |
| `Secret` 的 `Display` / `Debug` 掩码，默认序列化掩码 | `test_display_and_debug_are_masked`、`test_default_serialize_masks_json_and_toml` | `crates/ccr-core/src/core/secret.rs:167`、`:182` |
| 掩码算法边界（10 / 11 个字符） | `test_mask_sensitive` | `crates/ccr-core/src/utils/mask.rs:60-66` |
| `ConfigSection` 落盘为明文、`Debug` 掩码 | `test_legacy_plaintext_section_round_trip_lossless`、`test_debug_output_masks_auth_token` | `crates/ccr-config/src/managers/config/types.rs:417`、`:440` |
| 目录文件权限与 `Debug` 不泄密 | `catalog_files_are_owner_only_and_debug_hides_secrets` | `crates/ccr-config/src/managers/config/catalog/tests.rs:627` |
| TOML 解析错误不回显密钥行 | `test_load_profiles_from_toml_reports_path_on_parse_error`、`test_parse_profiles_from_str_redacts_typed_secret_field_failure`、`test_parse_profiles_from_str_redacts_rejected_enum_value`、`secret_bearing_toml_parse_errors_do_not_echo_source_lines` | `crates/ccr-config/src/platforms/base.rs:992`、`:1037`、`:1052`；`crates/ccr-cli/src/platforms/grok.rs:2181` |
| Codex 保存时把密钥移出目录、读取时还原 | `test_save_profile_scrubs_secret_and_restores_on_load` | `crates/ccr-codex/src/platforms/codex.rs:3116` |
| Codex 重命名时 `profile_secrets.json` 跟随 | Tauri 命令测试 | `ccr-ui/src-tauri/src/commands/codex_profiles.rs:382-395`、`:473-479` |
| Grok CLI 的 `create` / `current` / `list` 输出不含内联密钥 | `grok_profile_command_flow_masks_secrets_and_clears_managed_runtime` | `crates/ccr/tests/commands/grok_profile.rs:285` |
| 桌面端专用导出：含密钥读目录、拒绝遮蔽模式 | `claude_export_profiles_reads_catalog_toml_and_filename`、`claude_export_profiles_rejects_redacted_mode`、Codex 同名两项 | `ccr-ui/src-tauri/src/commands/claude_profiles.rs:224`、`:259`；`ccr-ui/src-tauri/src/commands/codex_profiles.rs:248`、`:283` |
| 导出文件只新建不覆盖 | `explicit_export_creates_only_new_absolute_files` | `crates/ccr-app/tests/profile_services.rs:529` |
| `ccr codex auth export --no-secrets`（账号导出，不是 profile 导出） | `codex_cancel_and_empty_export_have_no_success_result` | `crates/ccr/tests/commands/output_presentation.rs:398` |

没有测试覆盖的场景：

- Grok `api_key` 或任何扩展字段在 `export_config(false)` 下的表现。
- `base_url` 内嵌凭据的导出。
- `ccr export --no-secrets` 的 CLI 端到端行为（`export.rs` 的单元测试只测输出路径，`:106-120`）。
- 按平台（`--platform codex` / `grok`）的导出内容。
- 遮蔽产物再导入，尤其是合并模式覆盖同名 profile。
- Codex 导出后在另一份空环境导入能否通过校验。

## 8. 未能验证的事项

- 缺口 1、5、7、8 是读代码得出的，没有执行验证，也没有现成测试佐证。任务边界不允许改动源码，因此没有补测试来确认。
- 没有核对 Claude Code、Codex CLI、Grok CLI 上游文档中凭据存储的实际行为；关于"由对应 CLI 自己的登录态提供"的说法只依据本仓库源码中的注释、路径常量和契约文件。
- 没有检查 TUI 与 `ccr-sync` 是否另有导出或遮蔽路径；`ccr-sync` 对目录的使用只确认到它调用 `replace_toml`（`crates/ccr-sync/src/sync/service.rs:1978`）。
