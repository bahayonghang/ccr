# 配置管理

CCR 提供了强大而灵活的配置管理功能。本页面详细介绍配置文件格式、环境变量管理、备份策略等高级功能。

## 配置文件格式

CCR 使用 TOML 格式的配置文件 `~/.ccs_config.toml`，与 CCS 完全兼容。

### 基本结构

```toml
# 全局设置
default_config = "anthropic"    # 默认配置名称
current_config = "anthropic"    # 当前使用的配置

# 配置段 1
[anthropic]
description = "Anthropic Official API"
base_url = "https://api.anthropic.com"
auth_token = "sk-ant-your-api-key"
model = "claude-sonnet-4-5-20250929"
small_fast_model = "claude-3-5-haiku-20241022"

# 配置段 2
[anyrouter]
description = "AnyRouter Proxy Service"
base_url = "https://api.anyrouter.ai/v1"
auth_token = "your-anyrouter-token"
model = "claude-sonnet-4-5-20250929"
```

### 字段说明

#### 全局字段

| 字段 | 类型 | 必需 | 说明 |
|------|------|------|------|
| `default_config` | String | 是 | 默认使用的配置名称 |
| `current_config` | String | 是 | 当前激活的配置名称 |

#### 配置段字段

| 字段 | 类型 | 必需 | 说明 |
|------|------|------|------|
| `description` | String | 否 | 配置描述 |
| `base_url` | String | 是 | API 端点地址 |
| `auth_token` | String | 是 | 认证令牌 |
| `model` | String | 是 | 默认使用的模型 |
| `small_fast_model` | String | 否 | 快速小模型（可选） |

### 配置示例

#### Anthropic 官方 API

```toml
[anthropic]
description = "Anthropic Official API"
base_url = "https://api.anthropic.com"
auth_token = "sk-ant-api03-xxxxx"
model = "claude-sonnet-4-5-20250929"
small_fast_model = "claude-3-5-haiku-20241022"
```

#### 第三方代理服务

```toml
[anyrouter]
description = "AnyRouter Proxy Service"
base_url = "https://api.anyrouter.ai/v1"
auth_token = "your-anyrouter-token"
model = "claude-sonnet-4-5-20250929"
```

#### 自建代理

```toml
[selfhosted]
description = "Self-hosted Proxy"
base_url = "http://localhost:8000"
auth_token = "custom-token"
model = "claude-sonnet-4-5-20250929"
```

---

## 环境变量管理

CCR 通过修改 `~/.claude/settings.json` 来管理环境变量，确保配置立即生效。

### 管理的环境变量

CCR 管理以下 Claude Code 环境变量：

| 变量名 | 映射字段 | 说明 |
|--------|----------|------|
| `ANTHROPIC_BASE_URL` | `base_url` | API 端点地址 |
| `ANTHROPIC_AUTH_TOKEN` | `auth_token` | 认证令牌 |
| `ANTHROPIC_MODEL` | `model` | 默认模型 |
| `ANTHROPIC_SMALL_FAST_MODEL` | `small_fast_model` | 小型快速模型 |

### 切换机制

当执行 `ccr switch <config>` 时，CCR 会：

1. **清除现有变量**：删除所有 `ANTHROPIC_*` 前缀的环境变量
2. **设置新变量**：根据目标配置设置新的环境变量
3. **保留其他设置**：保持 Claude Code 的其他设置不变

### settings.json 示例

```json
{
  "environmentVariables": {
    "ANTHROPIC_BASE_URL": "https://api.anthropic.com",
    "ANTHROPIC_AUTH_TOKEN": "sk-ant-your-api-key",
    "ANTHROPIC_MODEL": "claude-sonnet-4-5-20250929",
    "ANTHROPIC_SMALL_FAST_MODEL": "claude-3-5-haiku-20241022"
  },
  "otherSettings": {
    // 其他 Claude Code 设置保持不变
  }
}
```

---

## 备份与恢复

CCR 提供自动备份机制，确保配置安全。

### 自动备份

**触发时机：**
- 执行 `ccr switch` 切换配置前
- 执行 `ccr import` 导入配置前（除非使用 `--no-backup`）
- 执行 `ccr init --force` 强制初始化前

**备份位置：**
```
~/.claude/backups/
```

**备份文件命名：**
```
settings_<timestamp>_<config_name>.json.bak
```

**示例：**
```
settings_20250110_120530_anthropic.json.bak
```

### 备份清理

使用 `ccr clean` 命令清理旧备份：

```bash
# 清理 7 天前的备份（默认）
ccr clean

# 清理 30 天前的备份
ccr clean --days 30

# 预览清理
ccr clean --dry-run
```

### 手动恢复

虽然 CCR 暂不支持命令行恢复，但你可以手动恢复：

```bash
# 1. 查看可用备份
ls -lh ~/.claude/backups/

# 2. 手动恢复（复制备份到设置文件）
cp ~/.claude/backups/settings_20250110_120530_anthropic.json.bak \
   ~/.claude/settings.json
```

::: tip 提示
未来版本将添加 `ccr restore` 命令以支持命令行恢复。
:::

---

## 操作历史

CCR 记录所有操作的详细历史，存储在 `~/.claude/ccr_history.json`。

### 历史记录格式

```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2025-01-10T12:05:30Z",
    "actor": "username",
    "operation_type": "switch",
    "details": {
      "from_config": "anthropic",
      "to_config": "anyrouter",
      "env_changes": {
        "ANTHROPIC_BASE_URL": {
          "old": "https://api.anthropic.com",
          "new": "https://api.anyrouter.ai/v1"
        },
        "ANTHROPIC_AUTH_TOKEN": {
          "old": "sk-a...key",
          "new": "you...ken"
        }
      }
    },
    "result": "success",
    "notes": "Configuration switched successfully"
  }
]
```

### 历史记录字段

| 字段 | 说明 |
|------|------|
| `id` | 操作唯一标识（UUID） |
| `timestamp` | 操作时间戳 |
| `actor` | 操作者（系统用户名） |
| `operation_type` | 操作类型（switch、backup、validate 等） |
| `details` | 操作详情（包括环境变量变更） |
| `result` | 操作结果（success、failure） |
| `notes` | 备注信息 |

### 敏感信息保护

历史记录中的敏感信息（如 API Token）会自动脱敏，仅显示首尾字符：

```
原始: sk-ant-api03-xxxxxxxxxxxxxxxxxxxxx
脱敏: sk-a...xxxx
```

---

## Web API

CCR 的 Web 界面提供完整的 RESTful API，基于全新的 Service 层架构。

### 启动 Web 服务

```bash
ccr web --port 8080
```

浏览器将自动打开 `http://localhost:8080`。

### 架构说明

Web API 采用分层架构：
- **Handlers** - 处理 HTTP 请求
- **Services** - 业务逻辑层（ConfigService, SettingsService 等）
- **Managers** - 数据访问层

这确保了 API 的可靠性和可维护性。

### 完整 API 端点列表

#### 配置管理

| 方法 | 路径 | 功能 | Service |
|------|------|------|---------|
| GET | `/api/configs` | 列出所有配置 | ConfigService |
| POST | `/api/config` | 添加新配置 | ConfigService |
| PUT | `/api/config/:name` | 更新配置 | ConfigService |
| DELETE | `/api/config/:name` | 删除配置 | ConfigService |
| POST | `/api/switch` | 切换配置 | Commands |

#### 历史记录

| 方法 | 路径 | 功能 | Service |
|------|------|------|---------|
| GET | `/api/history` | 获取操作历史 | HistoryService |

#### 验证和工具

| 方法 | 路径 | 功能 | Service |
|------|------|------|---------|
| POST | `/api/validate` | 验证配置 | Commands |
| POST | `/api/clean` | 清理备份 | BackupService |

#### 设置管理

| 方法 | 路径 | 功能 | Service |
|------|------|------|---------|
| GET | `/api/settings` | 获取当前设置 | SettingsService |
| GET | `/api/settings/backups` | 获取备份列表 | SettingsService |
| POST | `/api/settings/restore` | 恢复设置 | SettingsService |

### API 使用示例

#### 获取所有配置

```http
GET /api/configs
```

**响应：**
```json
{
  "success": true,
  "data": {
    "current_config": "anthropic",
    "default_config": "anthropic",
    "configs": [
      {
        "name": "anthropic",
        "description": "Anthropic Official API",
        "base_url": "https://api.anthropic.com",
        "auth_token": "sk-a...key",
        "model": "claude-sonnet-4-5-20250929",
        "small_fast_model": "claude-3-5-haiku-20241022",
        "is_current": true,
        "is_default": true
      }
    ]
  }
}
```

#### 切换配置

```http
POST /api/switch
Content-Type: application/json

{
  "config_name": "anyrouter"
}
```

**响应：**
```json
{
  "success": true,
  "data": "配置切换成功"
}
```

#### 添加配置

```http
POST /api/config
Content-Type: application/json

{
  "name": "newconfig",
  "description": "New Configuration",
  "base_url": "https://api.example.com",
  "auth_token": "your-token",
  "model": "claude-sonnet-4-5-20250929"
}
```

#### 获取历史记录

```http
GET /api/history
```

**响应：**
```json
{
  "success": true,
  "data": {
    "entries": [
      {
        "id": "uuid",
        "timestamp": "2025-01-10T12:05:30Z",
        "operation": "切换配置",
        "actor": "username",
        "from_config": "anthropic",
        "to_config": "anyrouter",
        "changes": [
          {
            "key": "ANTHROPIC_BASE_URL",
            "old_value": "https://api.anthropic.com",
            "new_value": "https://api.anyrouter.ai/v1"
          }
        ]
      }
    ],
    "total": 1
  }
}
```

#### 清理备份

```http
POST /api/clean
Content-Type: application/json

{
  "days": 7,
  "dry_run": false
}
```

**响应：**
```json
{
  "success": true,
  "data": {
    "deleted_count": 10,
    "skipped_count": 5,
    "total_size_mb": 5.2,
    "dry_run": false
  }
}
```

#### 获取设置备份列表

```http
GET /api/settings/backups
```

**响应：**
```json
{
  "success": true,
  "data": {
    "backups": [
      {
        "filename": "settings.anthropic.20250110_120530.json.bak",
        "path": "/home/user/.claude/backups/...",
        "created_at": "2025-01-10T12:05:30Z",
        "size_bytes": 1024
      }
    ]
  }
}
```

### 错误响应格式

所有 API 错误响应统一格式：

```json
{
  "success": false,
  "data": null,
  "message": "错误详细信息"
}
```

常见 HTTP 状态码：
- `200` - 成功
- `400` - 请求参数错误
- `500` - 服务器内部错误

---

## 安全特性

CCR 实现了多层安全保护机制。

### 1. 敏感信息保护

**API Token 脱敏：**
- 显示和日志中自动脱敏
- 仅显示首尾字符
- 历史记录自动脱敏

**示例：**
```
原始: sk-ant-api03-abc123def456ghi789jkl012mno345pqr678
显示: sk-a...r678
```

### 2. 文件权限

**settings.json 权限：**
- 自动设置为 600（仅所有者可读写）
- 防止其他用户访问敏感信息

**配置文件权限：**
- 推荐设置为 644（所有者可写，其他人只读）

### 3. 并发控制

**文件锁定机制：**
- 跨进程文件锁定
- 超时保护（默认 10 秒）
- 自动锁资源释放（RAII）

**原子写入：**
- 使用临时文件 + rename 操作
- 防止部分更新
- 避免竞态条件

### 4. 备份保护

- 所有危险操作前自动备份
- 备份包含时间戳和配置名称
- 支持从备份恢复

---

## 高级用法

### 1. 批量配置管理

**导出所有配置：**
```bash
ccr export -o all-configs.toml
```

**合并导入：**
```bash
ccr import new-configs.toml --merge
```

### 2. 配置模板

创建配置模板便于快速添加新配置：

**template.toml:**
```toml
[template]
description = "Template Configuration"
base_url = "https://api.example.com"
auth_token = "REPLACE_WITH_YOUR_TOKEN"
model = "claude-sonnet-4-5-20250929"
```

**使用：**
```bash
# 编辑模板
vim template.toml

# 导入模板
ccr import template.toml --merge
```

### 3. 自动化脚本

**配置切换脚本：**
```bash
#!/bin/bash
# switch-to-prod.sh

echo "Switching to production configuration..."
ccr switch production

if [ $? -eq 0 ]; then
  echo "Switched successfully"
  ccr current
else
  echo "Failed to switch"
  exit 1
fi
```

### 4. 定期备份

**Crontab 配置：**
```bash
# 每天午夜导出配置
0 0 * * * ccr export -o ~/backups/ccr-$(date +\%Y\%m\%d).toml

# 每周清理旧备份
0 0 * * 0 ccr clean --days 30
```

### 5. 配置验证钩子

在切换前验证配置：

```bash
#!/bin/bash
# pre-switch-hook.sh

ccr validate
if [ $? -ne 0 ]; then
  echo "Configuration validation failed!"
  exit 1
fi

ccr switch "$1"
```

### 6. 多环境管理

为不同环境创建配置：

```toml
[dev]
description = "Development Environment"
base_url = "https://api-dev.example.com"
auth_token = "dev-token"
model = "claude-3-5-haiku-20241022"  # 使用快速模型

[staging]
description = "Staging Environment"
base_url = "https://api-staging.example.com"
auth_token = "staging-token"
model = "claude-sonnet-4-5-20250929"

[production]
description = "Production Environment"
base_url = "https://api.anthropic.com"
auth_token = "prod-token"
model = "claude-sonnet-4-5-20250929"
```

---

## 故障排除

### 配置验证失败

**问题：** 配置验证失败，提示缺少必需字段

**解决：**
```bash
# 查看详细错误
ccr validate

# 检查配置文件
cat ~/.ccs_config.toml

# 确保包含所有必需字段
vim ~/.ccs_config.toml
```

### 文件锁超时

**问题：** 执行命令时提示文件锁超时

**可能原因：**
- 另一个 CCR 进程正在运行
- 上次操作异常退出

**解决：**
```bash
# 检查正在运行的进程
ps aux | grep ccr

# 如果没有进程，清理锁文件
rm -rf ~/.claude/.locks/*
```

### 备份空间不足

**问题：** 备份文件占用过多磁盘空间

**解决：**
```bash
# 查看备份大小
du -sh ~/.claude/backups/

# 清理旧备份
ccr clean --days 7

# 或手动删除
rm ~/.claude/backups/settings_2024*.bak
```

### 权限错误

**问题：** 无法读取或写入配置文件

**解决：**
```bash
# 检查权限
ls -la ~/.claude/settings.json
ls -la ~/.ccs_config.toml

# 修复权限
chmod 600 ~/.claude/settings.json
chmod 644 ~/.ccs_config.toml
```

---

## 最佳实践

### 1. 配置命名

使用清晰、描述性的配置名称：

```toml
✅ 好的命名:
[anthropic-prod]
[anyrouter-dev]
[selfhosted-staging]

❌ 避免的命名:
[config1]
[test]
[temp]
```

### 2. 定期备份

定期导出配置以防数据丢失：

```bash
# 每周导出
ccr export -o ~/backups/ccr-weekly.toml
```

### 3. 验证配置

添加或修改配置后立即验证：

```bash
vim ~/.ccs_config.toml
ccr validate
```

### 4. 查看历史

切换配置后查看历史以确认：

```bash
ccr switch production
ccr history --limit 1
```

### 5. 使用 Web 界面

对于频繁的配置管理，使用 Web 界面更方便：

```bash
ccr web --port 8080
```

### 6. 保护敏感信息

- 不要在公共场所显示完整 Token
- 使用 `--no-secrets` 选项导出用于分享的配置
- 定期更新 API Token

---

## 下一步

- 查看 [核心命令](/commands/) 了解所有可用命令
- 查看 [快速开始](/quick-start) 了解基本使用流程
- 查看 [更新日志](/changelog) 了解最新功能
