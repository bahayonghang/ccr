# 官方 logout / 凭据存储（网络检索）

检索日期：2026-08-20。只记录官方文档与本机 Grok user-guide 中的事实，不含 CCR 产品决策。

## Claude Code

来源：

- https://code.claude.com/docs/en/authentication
- https://code.claude.com/docs/en/cli-reference
- https://code.claude.com/docs/en/commands
- https://code.claude.com/docs/en/troubleshoot-install

官方登出入口：

| 入口 | 行为 |
| --- | --- |
| `claude auth logout` | 从 Anthropic 账号登出。无额外 flags 记录 |
| 交互 `/logout` | 登出；同时重置 first-launch setup state。下次运行 `claude` 会再次走登录与 setup |
| `claude auth login` | 登录。`--email` / `--sso` / `--console` |
| `claude auth status` | JSON 登录状态。已登录退出码 0，未登录退出码 1 |

凭据存储：

| OS | 位置 |
| --- | --- |
| macOS | 加密 Keychain。CCR 当前诊断将其标为 `unobservable` |
| Linux | `~/.claude/.credentials.json`，文件模式 `0600` |
| Windows | `%USERPROFILE%\.claude\.credentials.json` |
| 覆盖 | `CLAUDE_CONFIG_DIR` 时，`.credentials.json` 落在该目录 |

官方说明：Claude Code 通过 `/login` 和 `/logout` 管理 `.credentials.json`。文档不鼓励手工删文件作为常规路径。

`/logout` 比“只删凭据文件”多一步：重置 first-launch setup。CCR 现有 profile 切换会尝试写 `~/.claude.json` 的 `hasCompletedOnboarding = true`，但 profile off 不改 `.credentials.json`，也不重置 onboarding。

## Codex

来源：

- https://developers.openai.com/codex/auth
- https://developers.openai.com/codex/cli/reference
- https://developers.openai.com/codex/cli/slash-commands
- https://github.com/openai/codex/pull/1932（历史实现：删除存储的 `auth.json`）

官方登出入口：

| 入口 | 行为 |
| --- | --- |
| `codex logout` | 清除 API key 与 ChatGPT 两类已保存凭据。无 flags |
| 交互 `/logout` | 清除当前用户会话的本地凭据 |
| Desktop / IDE 个人资料菜单 Log out | 与 CLI 共享缓存；一侧登出后另一侧下次需要重新登录 |

凭据存储（`cli_auth_credentials_store`）：

| 值 | 位置 |
| --- | --- |
| `file` | `$CODEX_HOME/auth.json`，默认 `~/.codex/auth.json` |
| `keyring` | 操作系统凭据库 |
| `auto` | 有 OS store 则用 keyring，否则回退 `auth.json` |

CLI 与 IDE 共用同一缓存。`codex logout` 同时处理 ChatGPT OAuth 与 API key。

## Grok Build

来源：

- https://docs.x.ai/build/cli/reference
- https://github.com/xai-org/grok-build/blob/main/crates/codegen/xai-grok-pager/docs/user-guide/02-authentication.md
- 本机 `~/.grok/docs/user-guide/02-authentication.md`（2026-08-20）

官方登出入口：

| 入口 | 行为 |
| --- | --- |
| `grok logout` | 登出并清除缓存凭据。无 flags |
| 交互 `/logout` | 登出当前账号 |
| `grok login` | 重新登录并替换缓存会话。`--oauth` 默认；`--device-auth` 用于无浏览器环境 |

凭据存储：

- 会话 token：`~/.grok/auth.json`（`$GROK_HOME` 可覆盖），Unix `0600`
- MCP OAuth：`~/.grok/mcp_credentials.json`（官方 logout 文档未声明会一并删除）
- 环境回退：`XAI_API_KEY`。有会话 token 时会话优先；要回退到 API key，需 `grok logout` 或删除 `auth.json`

认证优先级（高到低）：

1. `config.toml` 里 per-model `api_key` / `env_key`（含 CCR 写入的 `[model.custom]`）
2. `auth.json` 会话 token
3. `XAI_API_KEY`

因此：Grok `profile off` 去掉 `[model.custom]` 后，运行时会落到会话 token 或 `XAI_API_KEY`。`auth off` 若只删 `auth.json`，第三方 profile 的 inline `api_key` 仍可工作；官方模型则会落到 `XAI_API_KEY` 或未登录。

Grok 会热加载 `auth.json` 变更，外部删除后下一请求即生效。

## 对照

| 平台 | 官方 CLI 登出 | 默认文件 | 非文件存储 | 额外副作用 |
| --- | --- | --- | --- | --- |
| Claude | `claude auth logout` / `/logout` | `.credentials.json`（Win/Linux） | macOS Keychain | `/logout` 重置 first-launch setup |
| Codex | `codex logout` / `/logout` | `auth.json` | `keyring` / `auto` | CLI 与 IDE 共享缓存 |
| Grok | `grok logout` / `/logout` | `auth.json` | 无（文件） | 会话清除后回退 `XAI_API_KEY` |

三家官方 logout 均无 flags、非交互。CCR 若 spawn 这些二进制，不需要登录浏览器，但依赖 PATH 中的官方 CLI，且无法做与 `profile off` 相同的 backup/rollback。
