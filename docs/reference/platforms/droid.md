# Factory Droid 平台指南

> [!IMPORTANT]
> **保留兼容适配器（`legacy_adapter`）说明**
> 在 CCR 7.x 中，`droid`（显示名称为 **Factory Droid**，识别输入别名 `droid` 与 `factory`）属于**保留兼容适配器（`legacy_adapter`）**。
> `Platform::auth_profile_supported()` 仅包含 `claude`、`codex` 与 `grok`——**`droid` 不提供 `ccr droid profile ...` 或 `ccr droid auth ...` 等 CLI/TUI 子命令**，没有独立的 CCR UI 平台工作区，也不属于 `ccr validate` 的校验范围（`ccr validate` 仅检查注册表与 `claude` / `codex` / `grok`）。

## 概览

Factory Droid 是 Factory AI 提供的终端编码助手。CCR 在平台枚举与诊断域中保留了 `droid` 兼容适配器，用于支持注册表状态展示、宿主配置文件只读健康检查以及跨平台周边工具。

上游安装方式：

```powershell
# Windows
irm https://app.factory.ai/cli/windows | iex
```

```bash
# macOS / Linux
curl -fsSL https://app.factory.ai/cli/install | bash
```

## 平台信息

| 项目 | 值 |
|------|----|
| CCR 持久化平台键 | `droid` |
| 识别输入别名 | `droid`、`factory` |
| 显示名称 | Factory Droid |
| CLI 能力层级 | `legacy_adapter`（无 CLI/TUI `profile` 或 `auth` 子命令） |
| 宿主设置文件 | `~/.factory/settings.json` |

## 当前真实受支持的 CCR 能力

### 1. 平台注册表识别

通过 `ccr platform list` 查看 `droid` 在统一注册表中的状态：

```bash
ccr platform list
ccr platform list --json
```

### 2. 只读健康体检（`ccr doctor`）

使用 `ccr doctor` 对本地 `~/.factory/settings.json` 执行无副作用的只读结构与字段检查：

```bash
ccr doctor --platform droid
ccr doctor --platform droid --verbose
ccr doctor --all-platforms
```

### 3. 周边生态命令支持

`droid` 平台键可用于受支持的跨平台工具命令（例如会话过滤、提示词预设与自定义目录同步）：

```bash
# 按 droid 平台过滤会话列表
ccr sessions list --platform droid

# 注册或查看 WebDAV 同步目录（如按需同步 ~/.factory）
ccr sync folder list
```

> 仓库中的 `examples/droid/profiles.toml` 仅作为历史字段结构参考保留，不对应现行 CLI 导入或切换入口。

## 宿主配置文件参考（`~/.factory/settings.json`）

Factory Droid 直接读取 `~/.factory/settings.json`。CCR 的 `DroidPlatform` 只读检查器按 `DroidSettings` 结构解析其中的 `customModels` 数组（文件中其他顶层字段会原样保留）：

```json
{
  "customModels": [
    {
      "model": "claude-sonnet-4-5-20250929",
      "displayName": "Anthropic Sonnet",
      "baseUrl": "https://api.anthropic.com/v1",
      "apiKey": "sk-ant-...",
      "provider": "anthropic",
      "maxOutputTokens": 8192
    },
    {
      "model": "gpt-4o",
      "displayName": "OpenAI GPT-4o",
      "baseUrl": "https://api.openai.com/v1",
      "apiKey": "sk-...",
      "provider": "openai",
      "maxOutputTokens": 4096
    }
  ]
}
```

### `customModels` 条目字段说明

| JSON 字段 | 必填 | 说明 |
|-----------|------|------|
| `model` | ✅ | 上游模型标识符 |
| `displayName` | ❌ | Factory Droid 界面中的可读显示名称 |
| `baseUrl` | ✅ | 服务商 API 端点（以 `http://` 或 `https://` 开头） |
| `apiKey` | ✅ | 服务商 API 凭据 |
| `provider` | ✅ | 服务商协议类型：`anthropic`、`openai` 或 `generic-chat-completion-api` |
| `maxOutputTokens` | ❌ | 单次响应最大输出 Token 数限制 |

### 安全建议

- `~/.factory/settings.json` 中的 `apiKey` 以明文存储，建议设置严格文件权限（如 `chmod 600 ~/.factory/settings.json`）。
- 切勿将包含真实密钥的 `settings.json` 提交到 Git 仓库或附加到公开 Issue。

## 相关文档

- [平台支持概览](./index)
- [`doctor` 统一体检](/reference/commands/doctor)
- [Claude 平台指南](./claude)
- [Codex 平台指南](./codex)
- [Antigravity CLI 平台指南](./gemini)
