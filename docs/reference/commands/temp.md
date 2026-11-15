# temp - 临时配置快速设置

交互式快速设置 Claude 临时运行参数，无需预先创建或修改平台 Profile 配置库（`profiles.db`）。直接输入 `base_url`、`token` 与可选 `model`，即可立即写入 `~/.claude/settings.json`。

## 用法

```bash
ccr temp
```

执行后进入交互式提示，依次输入：
1. **Base URL**（必填）— API 端点地址
2. **Auth Token**（必填）— 认证令牌
3. **Model**（可选）— 模型名称，支持简短别名智能解析

## 与 `temp-token` 的区别

| 特性 | `ccr temp` | `ccr temp-token` |
|------|------------|------------------|
| 输入方式 | 交互式提示 | 命令行参数 |
| 依赖配置 | 无需已有 Profile | 基于当前活动 settings 做临时覆盖 |
| 适用场景 | 手动快速测试新服务商 | 脚本或命令行快速覆盖当前运行参数 |
| 模型解析 | 支持简短别名解析 | 直接使用传入值 |

## 智能模型解析

`ccr temp` 支持常见模型别名解析：

### Claude 模型

| 输入 | 解析为 |
|------|--------|
| `sonnet` / `claude-sonnet` | `claude-sonnet-4-20250514` |
| `opus` / `claude-opus` | `claude-opus-4-20250514` |
| `haiku` / `claude-haiku` | `claude-3-5-haiku-20241022` |

### GPT 模型

| 输入 | 解析为 |
|------|--------|
| `gpt4` / `gpt-4` / `gpt4o` | `gpt-4o` |
| `gpt4-mini` / `gpt-4o-mini` | `gpt-4o-mini` |

### Gemini 模型

| 输入 | 解析为 |
|------|--------|
| `gemini` / `gemini-pro` | `gemini-2.0-flash` |

> 💡 如果输入完整的模型名称（如 `claude-sonnet-4-5-20250929`），将原样写入，不做转换。

## 使用场景

### 场景 1：快速测试新的 API 服务商

当你想要快速验证某个新服务商连通性，而不想向 `profiles.db` 写入永久 Profile 时：

```text
$ ccr temp

1) Base URL: https://api.newprovider.com/v1
2) Auth Token: sk-test-xxxxx
3) Model: sonnet
   智能解析: sonnet -> claude-sonnet-4-20250514

确认应用此临时配置? (Y/n): y
✓ 临时配置已应用到 settings.json

下一步:
  • 执行 'ccr claude profile switch <name>' 可恢复为已保存的 Profile 配置
  • 执行 'ccr current' 可查看当前运行时状态
```

### 场景 2：跳过模型设置

直接按 `Enter` 留空模型项时，将保持服务商默认模型。

## 恢复已保存的 Profile 配置

临时配置直接修改 `~/.claude/settings.json`。测试结束后，切换到任意已保存的 Claude Profile（或退出 Profile 模式）即可恢复：

```bash
# 恢复为 profiles.db 中已保存的指定配置
ccr claude profile switch <name>

# 或退出 Profile 模式回到官方登录态
ccr claude profile off
```

## 注意事项

1. **不写入配置库**：此命令不会在 `~/.ccr/platforms/claude/profiles.db` 中新增或修改记录。
2. **立即生效**：确认后直接写入 `~/.claude/settings.json`。
3. **一次性覆盖**：下次运行 `ccr claude profile switch <name>` 或 `ccr claude profile off` 时会被覆盖。
4. **安全脱敏**：终端预览时自动对 Token 脱敏展示。
5. **URL 校验**：Base URL 必须以 `http://` 或 `https://` 开头。

## 文件位置

- 临时配置写入目标：`~/.claude/settings.json`
- 永久 Profile 配置库：`~/.ccr/platforms/claude/profiles.db`（Legacy 模式为 `~/.ccs_config.toml`）

## 相关命令

- [`temp-token`](./temp-token.md) - 命令式临时覆盖
- [`claude`](./claude.md) - 管理永久 Claude Profile（`create` / `switch` / `off`）
- [`current`](./current.md) - 查看当前运行时状态
