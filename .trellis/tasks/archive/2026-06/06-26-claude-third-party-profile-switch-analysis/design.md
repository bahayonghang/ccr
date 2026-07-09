# Claude 第三方 Profile 切换问题设计分析

## Evidence Chain

### 本机运行态

- `ccr claude profile current --json` 返回 `profile = "axiom-guomo-qq"`，`base_url = "https://axiomcode.dev"`。
- `C:\Users\lyh\.ccr\platforms\claude\profiles.toml` 中 `[glm]` 存在，但 `auth_token` 是占位符形态。
- `C:\Users\lyh\.claude\settings.json` 当前 base URL 来自 `axiomcode.dev`，不是 `open.bigmodel.cn/api/anthropic`。
- `settings.json` 当前缺 `CLAUDE_CODE_AUTO_COMPACT_WINDOW`、`API_TIMEOUT_MS`、`ANTHROPIC_DEFAULT_FABLE_MODEL` 和四层 `*_MODEL_NAME`。
- `C:\Users\lyh\.claude.json` 已有 `hasCompletedOnboarding = true`，所以“顶层 onboarding 不在 settings.json”本身不一定是当前阻塞点。

### CCR 代码路径

- `crates/ccr-cli/src/platforms/claude.rs::apply_profile` 是切换入口：`Subscription` 清理托管 env，`ApiKey` 调 `settings.update_from_config(&section)` 写 env。
- `crates/ccr-cli/src/managers/settings.rs::update_from_config` 只写当前 typed `ConfigSection` 支持的字段。
- `clear_managed_vars` 清理全部 `ANTHROPIC_*` 和少数 `CLAUDE_CODE_*` typed key；未建模的非 Anthropic key 不会随 profile 清理。
- `crates/ccr-config/src/managers/config/types.rs::ConfigSection`、`crates/ccr-config/src/models/platform.rs::ProfileConfig` 已支持 fable 与 model names，但不支持 compact window / API timeout / disable nonessential traffic。
- `ccr-ui/src/configs/providerPresets/claude.ts` 的 `zhipu-glm` 预设仍是 `model = "glm-5"` / `small_fast_model = "glm-5"` 形态，不等价于官方 1M 示例。
- `ccr-ui/src/utils/providerTemplates.ts::mapTemplateToClaudeProfilePatch` 已映射 fable，但没有运行时 env patch。

### 官方配置事实

- 智谱官方文档的 Claude Code 配置写入 `~/.claude/settings.json.env`，包含 `ANTHROPIC_AUTH_TOKEN`、`ANTHROPIC_BASE_URL`、`ANTHROPIC_DEFAULT_HAIKU_MODEL`、`ANTHROPIC_DEFAULT_SONNET_MODEL`、`ANTHROPIC_DEFAULT_OPUS_MODEL`、`CLAUDE_CODE_AUTO_COMPACT_WINDOW`、`CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC`、`API_TIMEOUT_MS`。
- 智谱文档说明 `hasCompletedOnboarding` 写入 `~/.claude.json`。
- 智谱文档说明成功配置后界面看到 Claude 模型但实际是 GLM 模型属于服务端映射行为。
- Claude Code 官方 env 文档说明 `CLAUDE_CODE_AUTO_COMPACT_WINDOW` 是 auto-compaction 计算用的上下文容量参数。

## Root Cause

我认为当前问题的根因是 **CCR 的 Claude profile schema 只覆盖了模型映射主路径，未覆盖智谱官方 Claude Code 接入所需的运行时 env/top-level 状态；同时本机当前 active profile 不是 `glm` 且 `[glm]` token 仍是占位符**。证据是本机 `current_profile = axiom-guomo-qq`，`settings.json` 也写着 `https://axiomcode.dev`，而源码 `update_from_config` 没有任何 `CLAUDE_CODE_AUTO_COMPACT_WINDOW` / `API_TIMEOUT_MS` 写出路径。

这不是已经修复过的 `auth_mode = subscription` 问题：当前 `axiom-guomo-qq` 和 `[glm]` 都是 `auth_mode = api_key`，doctor 也确认当前 profile validation 通过。

## Design Direction

### 1. Profile Env Contract

给 Claude profile 增加“受控运行时 env”层：

- typed fields:
  - `claude_code_auto_compact_window: Option<String>`
  - `api_timeout_ms: Option<String>`
  - `claude_code_disable_nonessential_traffic: Option<String>`
  - 可选：`claude_code_disable_experimental_betas`、`enable_tool_search`，只在有官方/兼容文档或现有任务证据时纳入。
- optional `extra_env: IndexMap<String, String>` 高级字段：
  - 仅允许字符串值；
  - 禁止覆盖 `ANTHROPIC_AUTH_TOKEN`、`ANTHROPIC_BASE_URL` 等核心字段；
  - 写入前对 key 做 allow/deny 校验；
  - 切换 profile 时清理上一个 profile 写入的 `extra_env` key，避免串档。

推荐先实现 typed fields，不急于引入泛化 `extra_env`，除非需要支持多个第三方厂商的私有 key。

### 2. Managed Key Cleanup

新增 key 必须同时进入：

- `ConfigSection` / `ProfileConfig`；
- `section_to_profile` / `profile_to_section`；
- `ClaudeSettings::update_from_config`；
- `ClaudeSettings::clear_managed_vars`；
- `ClaudePlatform::get_env_var_names`；
- CLI JSON / UI JSON profile serialization；
- UI form types / defaults / patch apply。

否则会出现“写得出但切走清不掉”或“存得下但 apply 不生效”的半状态。

### 3. Onboarding Handling

`hasCompletedOnboarding` 的正确位置按智谱文档是 `~/.claude.json`。CCR 不应盲目把该字段塞进 `settings.json`，因为这会制造两个来源。

建议新增一个只在 Claude third-party API profile apply 时执行的 idempotent helper：

- 读取 `~/.claude.json`；
- 若缺 `hasCompletedOnboarding`，写入 `true`；
- 保留所有未知字段；
- 使用原子写；
- 不修改 OAuth token 或账号信息。

如果无法安全读写 `~/.claude.json`，doctor 给出 actionable warning。

### 4. GLM Preset

更新 GLM 预设时不要内置真实 token。预设应只提供：

- base URL；
- provider/provider_type；
- Opus/Sonnet = `glm-5.2[1m]`；
- Haiku = `glm-4.7`（按官方示例）或项目明确选择 `glm-5.2[1m]`；
- compact window = `1000000`；
- API timeout = `3000000`；
- disable nonessential traffic = `"1"`。

如果保留本机 `[glm]` 四层全是 `glm-5.2[1m]` 的策略，文档必须说明这是 CCR 的高配模板，和智谱官网默认示例有差异。

### 5. Diagnostics

Doctor 增加 GLM / third-party profile 运行时一致性检查：

- `profile.auth_token` 是占位符时 warn/fail；
- active profile 与 `settings.json` base URL 不一致时 warn；
- profile 中有 fable/model names 但 settings 缺失时 warn；
- GLM 1M 模型但缺 compact window 时 warn；
- `hasCompletedOnboarding` 缺失时 warn，并说明目标文件是 `~/.claude.json`。

### 6. Compatibility

- 订阅 profile 不写 API key env，保持现有行为。
- 未知 `settings.json` 字段继续 preserve。
- `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC` 官方示例中是数字 `1`，但 CCR 当前 settings 类型是 `HashMap<String, String>`。写出时统一字符串 `"1"`，因为环境变量语义是字符串，并避免 serde 读取失败。

## Risk

- 泛化 `extra_env` 容易让用户保存任意敏感变量，需要明确 masking 和 denylist。
- 写 `~/.claude.json` 属于新文件面，必须避免破坏 OAuth metadata。
- GLM 模型名当前变化快，预设可能漂移；后续应从 provider catalog 或文档刷新流程维护。

## Rollback

- 新字段属于向后兼容 TOML 增量，旧 profile 可不填。
- 如运行时 env 写出引发问题，可切换到订阅 profile 或执行 `ccr claude profile off` 清理托管 env。
- 本机真实配置修改必须先备份 `profiles.toml`、`settings.json`、`.claude.json`。
