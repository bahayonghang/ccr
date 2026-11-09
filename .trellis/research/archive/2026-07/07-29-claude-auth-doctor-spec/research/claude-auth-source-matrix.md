# Claude Code 认证来源矩阵

核实日期:2026-07-29。

## 官方契约优先级

依据 Claude Code authentication/settings 文档,本任务采用以下来源顺序:

1. 云厂商模式 env:`CLAUDE_CODE_USE_BEDROCK`、`CLAUDE_CODE_USE_VERTEX`、`CLAUDE_CODE_USE_FOUNDRY`。
2. `ANTHROPIC_AUTH_TOKEN`。
3. `ANTHROPIC_API_KEY`;交互模式下是否接管还受一次性批准状态影响。
4. settings 顶层 `apiKeyHelper`;存在脚本不等于脚本当前能成功返回 key。
5. `CLAUDE_CODE_OAUTH_TOKEN`。
6. `/login` 官方订阅 OAuth;Windows/Linux 为 config dir 下 `.credentials.json`,macOS 为 Keychain。

`customApiKeyResponses` 是 API key 批准/拒绝状态记录,不是独立凭据来源。

## 非官方契约行为

anthropics/claude-code issue #80713 报告 `.claude.json.primaryApiKey` 可能压制活跃订阅。该项只能标注为 `issue_report` + `potential`,不得写成官方承诺。

## 诊断置信度

- `confirmed`:ccr 在当前进程/目标文件中读到足以确认来源存在且按官方契约可用的值。输出只含键名和位置,不含值。
- `potential`:来源存在,但 ccr 无法确认批准状态、helper 执行结果或 issue 行为是否在当前版本生效。
- `unobservable`:ccr 明确无法观察的层,作为能力边界列出,不伪装为未发现。

## 可观测范围

可观测:

- 当前 ccr 进程 env。
- 共享路径解析命中的 user settings.json env / `apiKeyHelper`。
- 共享 state_file 的 `customApiKeyResponses` / `primaryApiKey` 是否存在。
- Windows/Linux `.credentials.json` 是否含可解析 OAuth 凭据。

不可观测:

- 其他 shell/父进程不同环境。
- 任意未知 cwd 的 project shared/local settings。
- Claude Code 外部进程的 CLI 参数。
- managed settings 的动态/组织策略有效值。
- helper 脚本不执行时的返回值与外部 secret store。
- macOS Keychain 内容。

## 参考

- https://code.claude.com/docs/en/authentication
- https://code.claude.com/docs/en/settings
- https://code.claude.com/docs/en/llm-gateway
- https://support.claude.com/en/articles/12304248
- https://github.com/anthropics/claude-code/issues/80713
