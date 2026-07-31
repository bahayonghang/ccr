# Claude Code 2.1.220 onboarding 探针

## 目的

确认当前 Claude Code 在全新配置目录、缺少 `hasCompletedOnboarding` 时,非交互路径是否仍被 onboarding 阻断,并确认 `CLAUDE_CONFIG_DIR` 下状态文件的位置。

## 环境与命令

- 日期:2026-07-29
- 平台:Windows
- Claude Code:`2.1.220 (Claude Code)`
- 配置目录:唯一系统临时目录,运行前为空
- 命令:

```powershell
$env:CLAUDE_CONFIG_DIR = '<empty-temp-dir>'
claude --bare -p 'Reply with OK' --output-format json
```

`--bare` 明确跳过 keychain/OAuth 读取,本次未提供真实凭据,没有 API 调用费用。

## 结果

- 进程在约 206 ms 内到达认证阶段并返回 `Not logged in - Please run /login`。
- `duration_api_ms = 0`、`total_cost_usd = 0`。
- Claude Code 在 `<CLAUDE_CONFIG_DIR>/.claude.json` 创建状态文件。
- 新文件不含 `hasCompletedOnboarding`;仅含 first-start、machine/migration/notification 等运行时状态。

## 结论与边界

1. 对 2.1.220 的 `--bare -p` 非交互路径,缺少 `hasCompletedOnboarding` 不会阻止程序进入认证阶段。
2. ccr 为 API-key profile 主动整写 `.claude.json` 已没有可证明的必要性,却会与 Claude Code 自身写入竞争;优先方案应是停止写入。
3. 该探针不声称覆盖交互式首启 UI 的全部呈现;交互式 onboarding 应由 Claude Code 自己管理,ccr 不应通过私有状态键跳过它。
4. 当前版本把状态文件放在 `CLAUDE_CONFIG_DIR` 内;共享路径解析必须覆盖这一行为。
