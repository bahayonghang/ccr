# Antigravity CLI 平台指南

> [!IMPORTANT]
> **保留兼容适配器（`legacy_adapter`）说明**
> 在 CCR 7.x 中，`gemini`（面向用户的显示名称为 **Antigravity CLI**，同时识别输入别名 `gemini-cli`、`antigravity`、`antigravity-cli` 与 `agy`）属于**保留兼容适配器（`legacy_adapter`）**。
> `Platform::auth_profile_supported()` 仅包含 `claude`、`codex` 与 `grok`——**`gemini` 不提供 `ccr gemini profile ...` 或 `ccr gemini auth ...` 等 CLI/TUI 子命令**，也不属于 `ccr validate` 的校验范围（`ccr validate` 仅检查注册表与 `claude` / `codex` / `grok`）。

## 概览

CCR 继续把持久化平台键保持为 `gemini`，同时将面向用户的 Google CLI 集成指向 **Antigravity CLI**（推荐二进制 `agy`），以便在跟随 Google 从 Gemini CLI 迁移到 Antigravity CLI 的过程中保留既有同步目录、历史会话索引、技能路径与桌面 UI 工作区。

官方迁移参考：

- Google Developers Blog: <https://developers.googleblog.com/an-important-update-transitioning-gemini-cli-to-antigravity-cli/>
- Antigravity CLI Overview: <https://antigravity.google/docs/cli-overview>
- Gemini CLI migration docs: <https://antigravity.google/docs/gcli-migration>

## 平台信息

| 项目 | 值 |
|------|----|
| CCR 持久化平台键 | `gemini` |
| 识别输入别名 | `gemini-cli`、`antigravity`、`antigravity-cli`、`agy` |
| 显示名称 | Antigravity CLI |
| CLI 能力层级 | `legacy_adapter`（无 CLI/TUI `profile` 或 `auth` 子命令） |
| 推荐宿主二进制 | `agy` |
| 宿主设置文件 | `~/.gemini/antigravity-cli/settings.json` |
| 宿主 MCP 配置 | `~/.gemini/antigravity-cli/mcp_config.json` |
| 全局技能目录 | `~/.gemini/antigravity-cli/skills` |
| 旧版/共享 Gemini 技能目录 | `~/.gemini/skills` |
| 工作区 MCP 配置 | `.agents/mcp_config.json` |
| 工作区技能目录 | `.agents/skills` |

旧 `/gemini-cli` UI 路由会重定向到 `/antigravity`；内部平台键继续保持为 `gemini`。

## 当前真实受支持的 CCR 能力

### 1. 平台注册表识别

`ccr platform list` 与 `ccr platform list --json` 会列出 `gemini` 平台的启用状态与描述信息：

```bash
ccr platform list
ccr platform list --json
```

### 2. 只读健康体检（`ccr doctor`）

使用 `ccr doctor` 对本地 `~/.gemini/antigravity-cli/settings.json` 结构与 `GEMINI_API_KEY` 环境变量执行无副作用的只读检查：

```bash
ccr doctor --platform gemini
ccr doctor --platform gemini --verbose
ccr doctor --all-platforms
```

### 3. 周边生态命令支持

`gemini` 平台键可用于以下跨平台工具面：

```bash
# 会话索引与检索（兼容历史 ~/.gemini/tmp/*/chats/session-*.json 日志）
ccr sessions list --platform gemini
ccr sessions search "keyword" --platform gemini

# 技能安装与列表
ccr skills list --platform gemini
ccr skills install <skill-name> --platform gemini

# 提示词预设管理（写入 ~/.gemini/ 下的目标文件）
ccr prompts list --platform gemini
ccr prompts current gemini --platform gemini

# WebDAV 目录注册同步
ccr sync folder list
```

> 注：历史 Gemini CLI 会话日志保持导入兼容；Antigravity CLI 本地会话格式在确认稳定前不单独声明原生导入支持。

### 4. 图形界面工作区（`ccr ui` -> `/antigravity`）

通过 `ccr ui` 进入 `/antigravity` 模块，可以可视化管理本地 Antigravity 设置、MCP 服务器（`mcp_config.json`）、斜杠命令、自定义 Agents 与插件：

```bash
ccr ui -p 15173 --backend-port 38081
```

## 宿主配置文件参考

### `~/.gemini/antigravity-cli/settings.json`

`ccr doctor --platform gemini` 通过 `GeminiSettings` 结构只读检查 `~/.gemini/antigravity-cli/settings.json` 中的 `google` 对象，在体检读取时忽略无关的顶层 JSON 字段（`GeminiSettings` 本身不展开额外顶层字段；`ccr ui` 的 `/antigravity` 模块则通过原始 JSON 编辑保留无关顶层字段）：

```json
{
  "google": {
    "api_key": "AIzaSyExampleKeyWithAtLeast30Chars",
    "project_id": "my-project",
    "region": "us-central1",
    "api_version": "v1"
  }
}
```

校验与凭据规则：

- `google.api_key`：`GeminiSettings` 中的**必填**字段（`GoogleConfig::api_key: String`），需以 `AIza` 开头且长度不少于 `30` 个字符。
- `google.project_id`、`google.region`、`google.api_version`：可选的 Google Cloud / Vertex 关联元数据字段。
- 环境变量补充：`ccr doctor --platform gemini` 同时识别当前环境中的可选 `GEMINI_API_KEY`。
- 请为含明文密钥的 `~/.gemini/antigravity-cli/settings.json` 保持严格权限（例如 `chmod 600`），切勿提交至版本库。

### MCP 与技能路径说明

Antigravity MCP 服务器与主 `settings.json` 分文件存储在 `~/.gemini/antigravity-cli/mcp_config.json`（工作区级配置位于 `.agents/mcp_config.json`）。CCR 写入远程 MCP 服务器时优先使用 `serverUrl` 字段，读取时兼容旧版 `url` / `httpUrl`：

```json
{
  "mcpServers": {
    "example": {
      "serverUrl": "https://example.com/mcp",
      "type": "http"
    }
  }
}
```

技能查找顺序包括：

1. 工作区 `.agents/skills`
2. Antigravity 全局 `~/.gemini/antigravity-cli/skills`
3. 旧版/共享 `~/.gemini/skills`

旧版路径（`~/.gemini/skills`、`~/.gemini/commands`、`.gemini/commands` 以及 `~/.gemini/tmp/*/chats/session-*.json`）仅作为兼容来源保留，不应作为当前 Antigravity 设置或 MCP 的主配置路径。

## 相关页面

- [平台支持概览](./index)
- [`doctor` 统一体检](/reference/commands/doctor)
- [UI 模块地图](/guide/ui-modules)
- [Claude 平台指南](./claude)
- [Codex 平台指南](./codex)
