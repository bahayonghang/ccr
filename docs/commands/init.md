# init - 初始化配置文件

初始化 CCR 配置文件，从内置模板创建 `~/.ccs_config.toml`。

## 用法

```bash
ccr init [--force]
```

## 选项

- `--force`: 强制覆盖现有配置（会自动备份）

## 功能特性

- 从内置模板创建配置文件
- **安全模式**：不使用 `--force` 时拒绝覆盖现有配置
- 使用 `--force` 时自动备份现有配置
- 设置适当的文件权限（Unix: 644）
- 提供下一步操作提示

## 行为说明

- 如果配置已存在：显示警告并退出（安全）
- 使用 `--force`：备份并覆盖现有配置

## 示例

```bash
# 初始化配置文件
ccr init

# 强制覆盖（会备份现有配置）
ccr init --force
```

## 生成的配置文件

初始化后会在 `~/.ccs_config.toml` 创建如下结构的配置文件：

```toml
default_config = "anthropic"
current_config = "anthropic"

[anthropic]
description = "Anthropic Official API"
base_url = "https://api.anthropic.com"
auth_token = "sk-ant-your-api-key"
model = "claude-sonnet-4-5-20250929"
small_fast_model = "claude-3-5-haiku-20241022"

[anyrouter]
description = "AnyRouter Proxy Service"
base_url = "https://api.anyrouter.ai/v1"
auth_token = "your-anyrouter-token"
model = "claude-sonnet-4-5-20250929"
```

::: tip 提示
配置文件包含示例配置，你需要修改 `auth_token` 字段为你自己的 API 密钥。
:::

## 下一步

初始化完成后：

1. 编辑配置文件添加你的 API 密钥
2. 使用 `ccr list` 查看所有配置
3. 使用 `ccr validate` 验证配置
4. 使用 `ccr switch <config>` 切换配置

## 相关命令

- [list](./list) - 查看所有配置
- [validate](./validate) - 验证配置完整性
- [export](./export) - 导出配置备份
