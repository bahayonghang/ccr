# 官方 GLM profile（写入 ~/.ccr + 仓库内置预设）

## Background

承接父任务。按用户提供的 GLM 官方接入截图（bigmodel），创建一个可直接使用的「官方 GLM profile」。包含两个交付物：
1. **运行时**：写入用户真实 `~/.ccr/platforms/claude/profiles.toml` 的一个完整 profile（api key 占位符）。
2. **仓库内置**：在单一事实源 `crates/ccr-checkin/data/providers-catalog.json` 新增 GLM/bigmodel provider 条目（claude 平台 override），前端与 Rust 共享。

## 依赖

- **强依赖 `06-20-fable-model-support`**：fable 字段（`default_fable_model` 等）与 catalog 的 `ClaudeProviderTemplateOverride` 新字段必须先落地，否则：
  - profile TOML 里的 fable 字段无法被 apply 进 settings.json；
  - catalog 条目无法表达 fable/显示名。
- 故本任务在子任务 1 合入后再 start。

## Requirements

### R1 — 运行时官方 GLM profile（写入 ~/.ccr）
- R1.1 profile 内容对齐截图（除 token）：
  - `base_url = "https://open.bigmodel.cn/api/anthropic"`
  - `auth_token`：占位符（**非真实可用值**，如 `"REPLACE_WITH_YOUR_BIGMODEL_API_KEY"`）
  - `default_opus_model = default_sonnet_model = default_haiku_model = default_fable_model = "glm-5.2[1m]"`
  - 四层 `*_model_name = "GLM-5.2"`
  - `provider = "glm"`，`provider_type = "third_party_model"`，`auth_mode = "api_key"`（避免前置任务记录的 subscription 清空坑）
- R1.2 通过 ccr 正常路径写入（CLI 或直接 TOML），遵守原子写/备份/锁；不破坏既有 profiles。
- R1.3 非模型 env（`ENABLE_TOOL_SEARCH=0` / `CLAUDE_CODE_DISABLE_EXPERIMENTAL_BETAS=1` / `CLAUDE_CODE_AUTO_COMPACT_WINDOW=1000000`）的处理在 implement 阶段决策：默认**不**写入 profile（父任务 Out of Scope），如用户需要则经 settings 直写并说明。

### R2 — 仓库内置 GLM 预设
- R2.1 `crates/ccr-checkin/data/providers-catalog.json` 新增 provider 条目：id/name/domain（open.bigmodel.cn）/icon/bizCategory（official 或 cn_official 语义）/websiteUrl，claude 平台 override 填上述模型映射（含 fable + 显示名）。
- R2.2 api key 不入库（预设只含 base_url/模型映射等非密字段，token 留空由用户填）。
- R2.3 schemaVersion 与 Rust `PROVIDERS_CATALOG_SCHEMA_VERSION` 对齐；不破坏既有条目解析。

## Constraints

- 占位 api key 不得是任何真实可用值。
- 遵守 masking / 备份 / 文件锁 / 原子写。
- catalog 改动需前端（`just frontend-check-quick`）与 Rust（`just test`，含 builtin_providers 解析）双侧通过。

## Acceptance Criteria

- [ ] AC1：`~/.ccr/platforms/claude/profiles.toml` 出现 GLM profile，字段齐全、token 为占位符、`auth_mode=api_key`。
- [ ] AC2：`ccr` 切换到该 profile 后 `~/.claude/settings.json` 复现截图四层模型 + fable + 显示名（除 token）。
- [ ] AC3：catalog 新增 GLM 条目，前端模板选择器可见并可一键套用；Rust builtin_providers 解析通过。
- [ ] AC4：`just frontend-check-quick`、`just test`、`just lint-strict` 全绿。

## Out of Scope

- fable 后端能力本身（属子任务 1）。
- 其它第三方厂商预设。

## References

- 截图配置；父任务 `06-20-claude-fable-and-glm-profile`
- catalog 单一事实源：`crates/ccr-checkin/data/providers-catalog.json`（Rust `builtin_providers.rs` include_str! + 前端 `providersCatalog.ts` import）
- 前置任务 auth_mode 坑：`.trellis/tasks/archive/2026-06/06-19-claude-third-party-model-authmode`

## Resolution（实际落地，偏离原计划，如实记录）

实现期发现两点，对 R2 做了调整：

1. **`providers-catalog.json` 是「公益站签到目录」**：23 个 provider 全部 `bizCategory=community` 且全部带 `checkin`，并有测试不变量（标准站需 claude/codex override + checkin baseUrl 一致、platforms 块禁含敏感字段）。GLM 是官方付费、无签到的 API，塞进去语义不符且会破坏测试。→ **未改该 JSON**。
2. **GLM 内置预设已存在**：`ccr-ui/src/configs/providerPresets/claude.ts` 已有 `zhipu-glm`（base_url=open.bigmodel.cn、model=glm-5、provider_type=third_party_model）。预设/模板模型为「单 model」，无按层映射、无显示名概念。

实际交付：
- **R1（运行时官方 profile）✅**：在用户真实 `~/.ccr/platforms/claude/profiles.toml` 追加 `[glm]` 段（改前已备份 `profiles.toml.bak-before-glm`），含四层模型+fable+四显示名、`auth_mode=api_key`、`provider_type=third_party_model`、`auth_token` 占位符 `REPLACE_WITH_YOUR_BIGMODEL_API_KEY`。TOML 经 tomllib 校验通过，其余 16 个 profile 全部保留。
- **R2（内置预设）✅（调整版）**：沿用既有 `zhipu-glm` 预设，并把 **fable 接入「套用模板」链路**（`ClaudeProviderTemplateOverride.defaultFableModel`、`ClaudeProfileTemplatePatch.default_fable_model`、`mapTemplateToClaudeProfilePatch` 中 `default_fable_model = override.defaultFableModel || model`、表单 patch-apply、claudeTemplateDraft override）。效果：套用 GLM 类模板时 fable 自动指向该 provider 的 model（glm-5），而非回落到真 Anthropic Fable。显示名不走模板（无此概念，表单手填）。
- 未生效约束：用户已安装的 `ccr` 二进制需重新编译/安装后，`[glm]` 的 fable 字段才会在 `apply` 时写入 `~/.claude/settings.json`（源码已支持）。
