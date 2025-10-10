# switch - 切换配置

切换到指定配置，CCR 会自动处理备份、验证和更新。

## 用法

```bash
ccr switch <config_name>
# 或使用简写形式
ccr <config_name>
```

## 参数

- `<config_name>`: 目标配置名称（必需）

## 执行流程

1. ✓ 读取并验证目标配置
2. ✓ 备份当前 Claude Code 设置
3. ✓ 更新 `~/.claude/settings.json`
4. ✓ 更新配置文件 `current_config`
5. ✓ 记录操作历史

## 示例

```bash
# 切换到 anthropic 配置
ccr switch anthropic

# 使用简写形式
ccr anyrouter
```

::: warning 重要提示
切换配置会立即修改 Claude Code 的设置文件，配置立即生效。操作前会自动备份当前设置。
:::

## 切换机制

### 自动备份

每次切换前，CCR 会自动备份当前的 `settings.json`：

```
~/.claude/backups/settings_20250110_120530_anthropic.json.bak
```

备份文件包含：
- 时间戳
- 源配置名称
- 完整的设置内容

### 环境变量更新

切换时会更新以下环境变量：

| 环境变量 | 来源字段 |
|---------|----------|
| `ANTHROPIC_BASE_URL` | `base_url` |
| `ANTHROPIC_AUTH_TOKEN` | `auth_token` |
| `ANTHROPIC_MODEL` | `model` |
| `ANTHROPIC_SMALL_FAST_MODEL` | `small_fast_model` |

### 原子操作

CCR 使用原子写入操作：
1. 写入临时文件
2. 验证内容
3. 重命名覆盖原文件

这确保了即使操作被中断，也不会损坏配置文件。

## 使用场景

### 开发环境切换

在不同的开发环境间切换：

```bash
# 切换到开发环境
ccr dev

# 切换到测试环境
ccr staging

# 切换到生产环境
ccr production
```

### 代理切换

在官方 API 和代理服务间切换：

```bash
# 使用官方 API
ccr anthropic

# 使用代理服务
ccr anyrouter
```

### 成本优化

根据任务复杂度切换模型：

```bash
# 简单任务使用快速模型
ccr haiku-config

# 复杂任务使用完整模型
ccr sonnet-config
```

## 错误处理

### 配置不存在

```bash
$ ccr switch nonexistent
Error: Configuration 'nonexistent' not found
Available configurations: anthropic, anyrouter
```

### 配置不完整

```bash
$ ccr switch incomplete
Error: Configuration 'incomplete' is missing required fields
Missing: auth_token
Run 'ccr validate' for details
```

### 文件锁定

```bash
$ ccr switch anthropic
Error: Settings file is locked by another process
Please wait and try again, or run: rm ~/.claude/.locks/*
```

## 切换后验证

切换后建议立即验证：

```bash
# 切换并验证
ccr switch anthropic && ccr current

# 查看切换历史
ccr switch anthropic && ccr history --limit 1
```

## 相关命令

- [list](./list) - 查看可用配置
- [current](./current) - 查看当前配置
- [validate](./validate) - 验证配置
- [history](./history) - 查看切换历史
