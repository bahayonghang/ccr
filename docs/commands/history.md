# history - 查看操作历史

显示 CCR 的操作历史记录，包括所有切换、备份等操作。

## 用法

```bash
ccr history [OPTIONS]
```

## 选项

- `--limit <N>`: 显示最近 N 条记录（默认：20）
- `-t, --type <TYPE>`: 按操作类型过滤（如：switch、backup）

## 历史记录内容

每条历史记录包含：
- 操作 ID（UUID）
- 时间戳
- 操作者（系统用户名）
- 操作类型
- 环境变量变更（脱敏）
- 操作结果
- 备注信息

## 示例

### 基本用法

```bash
# 显示最近 20 条记录
ccr history

# 显示最近 50 条记录
ccr history --limit 50
```

### 类型过滤

```bash
# 仅显示切换操作
ccr history -t switch

# 仅显示备份操作
ccr history -t backup

# 仅显示验证操作
ccr history -t validate
```

## 示例输出

```
Operation History
════════════════════════════════════════════════════════════════
Showing last 20 operations
────────────────────────────────────────────────────────────────

[2025-01-10 12:05:30] switch
ID: 550e8400-e29b-41d4-a716-446655440000
Actor: username
From: anthropic -> anyrouter
Changes:
  ANTHROPIC_BASE_URL: https://api.anthropic.com -> https://api.anyrouter.ai/v1
  ANTHROPIC_AUTH_TOKEN: sk-a...key -> you...ken
Result: ✓ Success
Notes: Configuration switched successfully

[2025-01-10 11:30:15] backup
ID: 660e8400-e29b-41d4-a716-446655440001
Actor: username
Backup: settings_20250110_113015_anthropic.json.bak
Result: ✓ Success
Notes: Automatic backup before switch

[2025-01-10 10:00:00] validate
ID: 770e8400-e29b-41d4-a716-446655440002
Actor: username
Result: ✓ Success
Notes: All configurations valid
────────────────────────────────────────────────────────────────
✓ Showing 3 of 3 total operations
```

## 操作类型

CCR 记录以下类型的操作：

| 类型 | 说明 |
|------|------|
| `switch` | 配置切换 |
| `backup` | 手动或自动备份 |
| `validate` | 配置验证 |
| `export` | 配置导出 |
| `import` | 配置导入 |
| `clean` | 备份清理 |
| `init` | 初始化 |
| `update` | 更新 CCR |

## 历史记录格式

历史记录存储在 `~/.claude/ccr_history.json` 中：

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

## 使用场景

### 审计追踪

查看谁在什么时候进行了配置切换：

```bash
ccr history --limit 100 -t switch
```

### 故障排查

当出现问题时，查看最近的操作：

```bash
ccr history --limit 5
```

### 回溯变更

查找特定时间的配置变更：

```bash
ccr history | grep "2025-01-10"
```

### 统计分析

统计切换次数：

```bash
ccr history -t switch | grep "switch" | wc -l
```

## 敏感信息保护

历史记录中的敏感信息会自动脱敏：

```
原始 Token: sk-ant-api03-abc123def456ghi789jkl012mno345pqr678
历史记录: sk-a...r678
```

这确保了即使历史文件被意外暴露，API 密钥也不会泄露。

## 历史文件管理

### 查看历史文件

```bash
cat ~/.claude/ccr_history.json | jq .
```

### 备份历史

```bash
cp ~/.claude/ccr_history.json ~/backups/ccr_history_$(date +%Y%m%d).json
```

### 清空历史

::: danger 警告
清空历史记录将无法恢复，请谨慎操作！
:::

```bash
# 备份后清空
cp ~/.claude/ccr_history.json ~/backups/ccr_history_backup.json
echo "[]" > ~/.claude/ccr_history.json
```

## 与日志的区别

- **History**: 记录成功的操作，用于审计和追踪
- **Logs**: 包含所有操作（成功和失败），用于调试

## 相关命令

- [switch](./switch) - 切换配置（会记录历史）
- [export](./export) - 导出配置（会记录历史）
- [import](./import) - 导入配置（会记录历史）
