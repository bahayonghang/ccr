# temp - 临时配置快速设置

交互式快速设置临时配置，无需依赖现有 TOML 配置文件。直接输入 base_url、token、model 并立即写入 settings.json。

## 用法

```bash
ccr temp
```

执行后将进入交互式提示，依次输入：
1. **Base URL** (必填) - API 端点地址
2. **Auth Token** (必填) - 认证令牌
3. **Model** (可选) - 模型名称，支持智能解析

## 与 temp-token 的区别

| 特性 | `ccr temp` | `ccr temp-token` |
|------|------------|------------------|
| 输入方式 | 交互式提示 | 命令行参数 |
| 依赖配置 | 无需任何配置 | 基于现有 TOML 配置 |
| 使用场景 | 快速测试新提供商 | 临时覆盖现有配置 |
| 模型解析 | 支持智能解析 | 直接使用输入值 |

## 智能模型解析

`ccr temp` 支持模型名称的智能解析，让你可以用简短的别名代替完整的模型名称：

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

> 💡 如果输入完整的模型名称（如 `claude-3-5-sonnet-20241022`），将直接使用，不做转换。

## 使用场景

### 场景 1: 快速测试新的 API 提供商

当你想要快速测试一个新的 API 提供商，但不想创建永久配置时：

```bash
$ ccr temp

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  临时配置快速设置
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📝 本小姐来帮你快速设置临时配置！
   此配置将直接写入 settings.json，无需创建 TOML 配置
   下次执行 'ccr switch' 时将恢复为 TOML 配置中的值

────────────────────────────────────────

1️⃣ 请输入 Base URL (API 端点地址)
   示例: https://api.anthropic.com
         https://api.example.com/v1

* Base URL: https://api.newprovider.com/v1

2️⃣ 请输入 Auth Token (认证令牌)
   示例: sk-ant-api03-xxxxx
         sk-xxxxx

* Auth Token: sk-test-xxxxx

3️⃣ 请输入 Model (模型名称，可选)
   示例: claude-sonnet-4-20250514
         claude-3-5-sonnet-20241022
         gpt-4o
   提示: 直接按 Enter 跳过，使用服务商默认模型

  Model: sonnet
   🧠 智能解析: sonnet → claude-sonnet-4-20250514

────────────────────────────────────────

步骤 1/2: 配置预览

╔════════════════╤═══════════════════════════════════════════╗
║ 字段           │ 值                                        ║
╠════════════════╪═══════════════════════════════════════════╣
║ Base URL       │ https://api.newprovider.com/v1            ║
║ Auth Token     │ sk-t...xxxxx                              ║
║ Model          │ claude-sonnet-4-20250514                  ║
╚════════════════╧═══════════════════════════════════════════╝

确认应用此临时配置? (Y/n): y

────────────────────────────────────────

步骤 2/2: 应用临时配置
✅ 临时配置已应用到 settings.json

💡 提示:
   • 临时配置已立即生效
   • 执行 'ccr switch <配置名>' 可恢复为 TOML 配置
   • 执行 'ccr current' 可查看当前配置状态
```

### 场景 2: 使用简短模型别名

```bash
$ ccr temp
...
  Model: opus
   🧠 智能解析: opus → claude-opus-4-20250514
```

### 场景 3: 跳过模型设置

如果你想使用服务商的默认模型：

```bash
$ ccr temp
...
  Model: [直接按 Enter]
   ℹ️  已跳过，将使用服务商默认模型
```

## 恢复配置

临时配置会直接修改 `~/.claude/settings.json`。要恢复到 TOML 配置中的值：

```bash
# 切换到任意配置即可恢复
ccr switch <配置名>
```

## 注意事项

1. **不创建配置文件**：此命令不会创建或修改 TOML 配置文件
2. **立即生效**：配置立即写入 settings.json，无需其他操作
3. **一次性使用**：下次 `ccr switch` 时会被覆盖
4. **安全脱敏**：Token 在显示时自动脱敏
5. **URL 验证**：Base URL 必须以 `http://` 或 `https://` 开头

## 相关命令

- [`ccr temp-token`](./temp-token.md) - 基于现有配置的临时覆盖
- [`ccr switch`](./switch.md) - 切换配置（恢复 TOML 配置）
- [`ccr current`](./current.md) - 查看当前配置状态
- [`ccr add`](./add.md) - 交互式添加永久配置

## 文件位置

- 临时配置写入：`~/.claude/settings.json`
- 永久配置文件：`~/.ccs_config.toml` 或 `~/.ccr/platforms/<platform>/profiles.toml`
