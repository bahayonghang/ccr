# init - 初始化配置文件

初始化 CCR 配置文件，创建 Unified Mode 多平台配置结构（`~/.ccr/`）或兼容 Legacy Mode（`~/.ccs_config.toml`）。

## 用法

```bash
ccr init [--force]
```

## 选项

- `--force`: 强制覆盖现有配置（会自动备份）

## 配置模式

### 🆕 Unified Mode（默认）

现代多平台配置结构，支持多个 AI CLI 工具：

```
~/.ccr/
├── config.toml              # 平台注册表
├── platforms/
│   ├── claude/              # Claude Code 平台（默认）
│   │   ├── profiles.toml    # Claude 配置文件（首次使用时创建）
│   │   ├── history/         # 操作历史目录
│   │   └── backups/         # 备份目录
│   ├── codex/               # Codex 平台（可选）
│   │   ├── profiles.toml
│   │   ├── history/
│   │   └── backups/
│   └── gemini/              # Gemini 平台（可选）
│       ├── profiles.toml
│       ├── history/
│       └── backups/
├── history/                 # 全局历史记录
└── backups/                 # 全局备份目录
```

**优点：**
- ✅ 支持多平台管理（Claude、Codex、Gemini 等）
- ✅ 平台配置完全隔离
- ✅ 独立的平台历史和备份
- ✅ 支持平台切换
- ✅ 更好的文件组织

### 🔙 Legacy Mode（兼容模式）

传统单文件配置，仅在设置 `CCR_LEGACY_MODE=1` 环境变量时使用：

```bash
export CCR_LEGACY_MODE=1
ccr init
```

**功能特性：**
- 创建单个 `~/.ccs_config.toml` 文件
- 与 CCS（Shell 版本）完全兼容
- 仅管理 Claude Code
- 推荐用于维护旧项目

## 功能特性

### Unified Mode 特性
- 从模板创建多平台配置结构
- 安全模式：未使用 `--force` 时拒绝覆盖
- 自动初始化 Claude 平台（默认）
- 自动创建必要的子目录（history, backups）
- 创建平台注册表 `config.toml`

### Legacy Mode 特性
- 从内置模板创建配置文件
- 安全模式：未使用 `--force` 时拒绝覆盖
- 自动备份现有配置
- 智能备份管理：自动保留最近 10 个配置备份
- 设置适当的文件权限（Unix: 644）

## 行为说明

### Unified Mode

1. **首次初始化**
   ```bash
   ccr init
   ```
   - 创建 `~/.ccr/` 目录结构
   - 初始化 Claude 平台目录
   - 创建 `config.toml` 平台注册表
   - 提示后续步骤

2. **配置已存在**
   ```bash
   ccr init
   ```
   - 显示配置存在的警告
   - 提示使用平台命令管理配置
   - 退出（安全）

3. **强制重新初始化**
   ```bash
   ccr init --force
   ```
   - 备份现有配置到 `~/.ccr/config.toml.bak`
   - 覆盖现有配置
   - 显示确认提示

### Legacy Mode

- 仅在设置 `CCR_LEGACY_MODE=1` 时启用
- 行为与旧版本相同

::: tip 自动备份管理
使用 `--force` 时，CCR 会自动备份现有配置文件。Unified Mode 使用 `~/.ccr/config.toml.bak`，Legacy Mode 使用 `~/.ccs_config.toml.init_<timestamp>.bak`
:::

## 示例

### Unified Mode（推荐）

```bash
# 初始化新的 Unified Mode 配置
ccr init

# 查看初始化结果
ccr platform list

# 初始化其他平台
ccr platform init codex
ccr platform init gemini

# 强制覆盖（会自动备份）
ccr init --force
```

### Legacy Mode（兼容）

```bash
# 使用 Legacy Mode 初始化
export CCR_LEGACY_MODE=1
ccr init

# 强制覆盖
ccr init --force
```

## 生成的配置文件

### Unified Mode - config.toml

```toml
default_platform = "claude"
current_platform = "claude"

[claude]
enabled = true
description = "Claude Code AI Assistant"
```

每个平台的 profiles 单独存储在 `~/.ccr/platforms/{platform}/profiles.toml`

### Legacy Mode - ~/.ccs_config.toml

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

### Unified Mode 初始化后

1. 查看所有平台：`ccr platform list`
2. 初始化其他平台：`ccr platform init <platform>`
3. 添加配置 profile：`ccr add`
4. 查看配置列表：`ccr list`
5. 如果仍保留旧单文件配置，把它当参考源并逐步搬到当前目录布局

### Legacy Mode 初始化后

1. 编辑配置文件添加 API 密钥
2. 使用 `ccr list` 查看所有配置
3. 使用 `ccr validate` 验证配置
4. 使用 `ccr switch <config>` 切换配置

## 相关命令

- [platform init](../commands/platform#init) - 初始化特定平台
- [platform list](../commands/platform#list) - 查看所有平台
- [platform switch](../commands/platform#switch) - 切换平台
- [list](./list) - 查看所有配置
- [add](./add) - 添加新配置
- [validate](./validate) - 验证配置完整性
- [export](./export) - 导出配置备份
