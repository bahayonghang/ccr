# CCR Platform & Login Concepts

CCR 管理多个 AI CLI 工具的登录与配置切换。本文件锚定相关术语，避免命令命名和 UI 描述漂移。

## Language

**Platform**:
CCR 当前管理的 AI CLI 工具种类，仅保留 `claude` 和 `codex` 两个值。
_Avoid_: provider、tool、cli

**Auth**:
某 Platform 的原生 OAuth 登录态，对应官方订阅账号。
_Avoid_: account、login、credential、token

**Profile**:
某 Platform 下的 API key + base URL 配置，用于第三方中转或自部署 API。
_Avoid_: config、preset、setting

**Mode**:
某 Platform 同一时间生效的凭据类型，取值 `oauth` 或 `api-key`，二者互斥。
_Avoid_: 模式、active type、effective

## Relationships

- 一个 **Platform** 同时最多一个激活 **Auth**、一个激活 **Profile**。
- **Profile** 激活时 **Mode** = `api-key`，覆盖 **Auth**。
- **Profile** 未激活时 **Mode** = `oauth`，使用 **Auth** 凭据。
- 切 **Auth** 时同步把 **Mode** 切回 `oauth`（清平台对应的 API key 覆盖）。
- 切 **Profile** 时同步把 **Mode** 切到 `api-key`（写平台对应的 API key 覆盖）。

## Storage

**Profile 数据**: 统一存于 `~/.ccr/platforms/{claude,codex}/profiles.toml`（CCR 内部）。
**激活时写入**（让 CLI 启动能读到）：

| 概念    | claude                                              | codex                                                                       |
| ------- | --------------------------------------------------- | --------------------------------------------------------------------------- |
| Auth    | `~/.claude/.credentials.json`                       | `~/.codex/auth.json` (`auth_method = chatgpt`)                              |
| Profile | `~/.claude/settings.json` 注入 `ANTHROPIC_*`        | `~/.codex/auth.json` (`auth_method = apikey` + `OPENAI_API_KEY`) + `~/.codex/config.toml` (`model_provider = custom`) |

注：codex 上 auth 和 profile **共用同一个 auth.json**，靠 `auth_method` 字段区分；切换时该字段会被改写。

## Example dialogue

> **Dev:** "`ccr claude switch xxx` 是切 **Auth** 还是 **Profile**？"
> **Domain expert:** "歧义。必须写 `ccr claude auth switch` 或 `ccr claude profile switch`。"

> **Dev:** "切了 **Profile**，原来的 **Auth** 还在吗？"
> **Domain expert:** "在文件里，但运行时被 **api-key** **Mode** 覆盖。切回 **Auth** 时同时清 API key 环境变量。"

## Flagged ambiguities

- 用户提议的 `ccr claude list/switch` 没有限定子层 — 解决：必须带 `auth` 或 `profile`，无限定形式直接报错。
- 老命令 `ccr platform switch <p>` 暗示存在"当前平台"全局状态 — 解决：删除该命令族 + `current_platform` 字段；CLI 通过命名空间显式指定。
- 已删除平台：`gemini`、`qwen`、`droid` — 仅保留 `claude` 和 `codex`。
- "从 api-key 回到 oauth（不换账号）"是否独立命令 — **待解决**（grilling 中）。
