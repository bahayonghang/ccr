# 分析 Claude 第三方模型 Profile 切换问题

## Goal

定位并修复 CCR 在切换 Claude Code 第三方模型 profile 时无法完整复现官方 `settings.json` 配置的问题，使 `ccr claude profile switch <name>`、TUI / UI profile 切换后能让 Claude Code 按目标第三方模型运行，并给用户明确的状态/错误解释。

## User Value

用户希望在 CCR 中管理 Claude Code 第三方模型，不再手动编辑 `~/.claude/settings.json`。切换到智谱 GLM 等第三方 profile 后，`~/.claude/settings.json` 应接近官方示例，避免启动 Claude Code 时因缺字段、占位 token、错误 current profile 或运行时 env 漏写而报错。

## Confirmed Facts

- 本机 CCR 任务创建前工作区干净；当前仅新增本 Trellis 任务目录。
- 本机 `ccr --version` 为 `6.4.2`，`claude --version` 为 `2.1.193 (Claude Code)`。
- 本机 `C:\Users\lyh\.ccr\platforms\claude\profiles.toml` 当前 `current_config = "axiom-guomo-qq"`，`C:\Users\lyh\.ccr\config.toml` 中 `claude.current_profile = "axiom-guomo-qq"`。
- `ccr claude profile current --json` 与 `ccr current --json` 都确认当前 Claude profile 是 `axiom-guomo-qq`，base URL 为 `https://axiomcode.dev`，不是 `glm`。
- 本机 `[glm]` profile 已存在，字段包含 `base_url = "https://open.bigmodel.cn/api/anthropic"`、四层模型/显示名、`provider_type = "third_party_model"`、`auth_mode = "api_key"`；但 `auth_token` 是占位符形态，不是可用 API key。
- 当前 `C:\Users\lyh\.claude\settings.json` 有 `ANTHROPIC_BASE_URL = "https://axiomcode.dev"`，有 Opus/Sonnet/Haiku 的 `glm-5.2[1m]` 映射，但缺 `ANTHROPIC_DEFAULT_FABLE_MODEL`、各层 `*_MODEL_NAME`、`CLAUDE_CODE_AUTO_COMPACT_WINDOW`、`API_TIMEOUT_MS`。
- `C:\Users\lyh\.claude\settings.json` 顶层没有 `hasCompletedOnboarding`；但 `C:\Users\lyh\.claude.json` 中已经有 `hasCompletedOnboarding = true`。
- 智谱官方 Claude Code 文档要求 `ANTHROPIC_BASE_URL = "https://open.bigmodel.cn/api/anthropic"`，并在 GLM 5.2 1M 场景中配置 `glm-5.2[1m]` 与 `CLAUDE_CODE_AUTO_COMPACT_WINDOW = "1000000"`；文档还说明界面上看到 Claude 模型但实际是 GLM 模型属于正常服务端映射。
- CCR 已修过旧问题：`auth_mode = "subscription"` 的第三方 profile 会被自愈为 `api_key`；fable 与 `*_MODEL_NAME` 字段也已进入后端 profile 类型与 env 映射。
- 当前源码仍没有把官方示例中的 `CLAUDE_CODE_AUTO_COMPACT_WINDOW`、`API_TIMEOUT_MS`、`CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC` 作为 Claude profile 一等字段，也没有在 profile apply 时写 `hasCompletedOnboarding`。
- 当前 `ccr doctor --json` 通过，只有 registry 中未知 `test` 平台和跨平台 model 值差异两个 warning；这说明 CCR 本地校验没有覆盖官方 GLM 配置完整性。

## Problem Statement

当前报错不应归结为单一代码缺陷，而是三个问题叠加：

1. **当前运行 profile 与用户期望不一致**：本机 active profile 是 `axiom-guomo-qq`，不是官方示例对应的 `glm`。
2. **`glm` profile 仍不可直接运行**：`glm` 的 token 是占位符，切过去也会因无真实智谱 API key 而失败。
3. **CCR profile schema 无法完整表达官方示例**：切换 profile 只会写已建模的 Anthropic/model env，不能托管 `CLAUDE_CODE_AUTO_COMPACT_WINDOW`、`API_TIMEOUT_MS`、`CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC`，也不会处理 onboarding 标记。因此即便 profile 切换成功，生成的 `settings.json` 也不能稳定等价于官网配置。

## Requirements

- R1. 明确区分“切错 profile / token 占位符 / CCR schema 缺字段 / 上游 Claude Code 行为”四类问题，避免误判。
- R2. 增强 Claude profile 配置模型，让官方示例中的非模型运行时 env 可由 profile 持久化、切换、清理，并在 CLI/UI/TUI 中可见。
- R3. 决定并实现 onboarding 标记写入策略：优先写入正确文件 `~/.claude.json`，同时保持 `~/.claude/settings.json` 未知字段保留。
- R4. 更新 GLM 内置预设/模板，使新建 GLM profile 默认包含官方当前推荐模型、1M compact window、API timeout、disable nonessential traffic 等字段，但不内置真实 token。
- R5. 增强验证/doctor/current 输出，能指出：
  - active profile 不是 `glm`；
  - profile token 是占位符；
  - 运行时 `settings.json` 缺官方 GLM 必需字段；
  - `settings.json` 与 profile 期望不一致。
- R6. 文档说明 `/model` 或界面仍显示 Claude 模型名不一定代表未生效；以 base URL、API key source、请求实际响应/日志为准。
- R7. 所有改动不得打印或提交真实 token，不得破坏官方订阅 profile 行为，不得清理用户无关配置。

## Acceptance Criteria

- [x] AC1: 在测试 fixture 中构造 `glm` profile，apply 后 `settings.json.env` 包含 `ANTHROPIC_BASE_URL`、token、Opus/Sonnet/Haiku/Fable、四层 `*_MODEL_NAME`、`CLAUDE_CODE_AUTO_COMPACT_WINDOW`、`API_TIMEOUT_MS`、`CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC`。
- [x] AC2: 从 `glm` 切到不含这些运行时 env 的其他 profile 时，旧的 GLM 托管 env 被清理，不串档。
- [x] AC3: 订阅 profile 仍不会写第三方 API key 覆盖，现有 subscription 测试保持通过。
- [x] AC4: `ccr doctor --json` 能在 token 占位、active profile 不匹配、运行时 env 缺失时给出 warning 或 failed check。
- [x] AC5: UI/TUI/CLI 新建或套用 GLM profile 时能表达新增字段；字段 roundtrip 后不落入 `platform_data` 孤儿区。
- [x] AC6: 文档更新包含智谱 GLM 官方配置、1M compact window、onboarding 文件位置、`/model` 显示说明。
- [x] AC7: 验证命令通过：`just fmt-check`、`cargo test -p ccr-cli -- --test-threads=1`、相关 `cargo test -p ccr --test commands -- --test-threads=1`、`just frontend-check-quick`；跨层改动最终跑 `just lint-strict` 或 `just ci`。

## Out Of Scope

- 不写入真实智谱 API key。
- 不自动切换用户本机当前 profile，除非用户明确授权实施并允许修改本机配置。
- 不改变 Claude Code 上游 `/model` 文案。
- 不重构 Codex/Gemini/Droid profile 模型。

## Open Questions

- Q1. 实施时是否允许把官方示例中的非模型 env 作为 Claude profile 一等字段，而不是仅作为 `extra_env` 任意映射？
  - 推荐：使用显式字段加受控 `extra_env`。显式字段覆盖已知官方 key，`extra_env` 只作为高级逃生口，并禁止覆盖 token/source 关键字段。
- Q2. 是否要把本机 `[glm]` 的占位 token 替换为真实 token 并切为 current profile？
  - 推荐：不在代码任务中处理真实 token。实现完成后由用户手动填 token，或走安全交互流程。
