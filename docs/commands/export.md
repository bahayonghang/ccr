# export - 导出配置

将当前配置导出到文件,用于备份或迁移。

## 用法

```bash
ccr export [OPTIONS]
```

## 选项

- `-o, --output <FILE>`: 指定输出文件(默认：自动生成带时间戳的文件名)
- `--no-secrets`: 导出时脱敏 API 密钥

## 功能特性

- 自动生成带时间戳的文件名
- 默认包含完整 API 密钥(便于迁移)
- 可选的敏感信息脱敏
- TOML 格式便于编辑
- 适合备份和迁移场景

## 示例

```bash
# 导出包含完整 API 密钥(默认)
ccr export

# 导出时脱敏敏感信息
ccr export --no-secrets

# 导出到指定文件
ccr export -o my-config.toml
```

## 示例输出

### 默认导出

```bash
$ ccr export
Exporting configuration...
✓ Configuration exported to: ccr_config_20250110_120530.toml
```

### 指定文件

```bash
$ ccr export -o backup.toml
Exporting configuration...
✓ Configuration exported to: backup.toml
```

## 导出内容

### 完整导出(默认)

包含所有配置和完整的 API 密钥：

```toml
default_config = "anthropic"
current_config = "anthropic"

[anthropic]
description = "Anthropic Official API"
base_url = "https://api.anthropic.com"
auth_token = "sk-ant-api03-abc123def456ghi789jkl012mno345pqr678"
model = "claude-sonnet-4-5-20250929"
small_fast_model = "claude-3-5-haiku-20241022"

[anyrouter]
description = "AnyRouter Proxy Service"
base_url = "https://api.anyrouter.ai/v1"
auth_token = "your-anyrouter-token-full"
model = "claude-sonnet-4-5-20250929"
```

### 脱敏导出(--no-secrets)

敏感信息被脱敏处理：

```toml
default_config = "anthropic"
current_config = "anthropic"

[anthropic]
description = "Anthropic Official API"
base_url = "https://api.anthropic.com"
auth_token = "sk-a...r678"  # 脱敏
model = "claude-sonnet-4-5-20250929"
small_fast_model = "claude-3-5-haiku-20241022"

[anyrouter]
description = "AnyRouter Proxy Service"
base_url = "https://api.anyrouter.ai/v1"
auth_token = "you...full"  # 脱敏
model = "claude-sonnet-4-5-20250929"
```

## 使用场景

### 1. 定期备份

定期备份配置以防意外丢失：

```bash
# 手动备份
ccr export -o ~/backups/ccr-backup.toml

# 使用 crontab 自动备份(每天)
0 0 * * * ccr export -o ~/backups/ccr-$(date +\%Y\%m\%d).toml
```

### 2. 迁移配置

迁移配置到新机器：

```bash
# 在旧机器上
ccr export -o ccr-config.toml

# 复制到新机器
scp ccr-config.toml user@new-machine:~

# 在新机器上导入
ccr import ccr-config.toml
```

### 3. 分享配置模板

分享配置模板给团队(不含密钥)：

```bash
# 导出脱敏版本
ccr export --no-secrets -o team-template.toml

# 团队成员导入并添加自己的密钥
ccr import team-template.toml --merge
```

### 4. 版本控制

将配置纳入版本控制(脱敏)：

```bash
# 导出脱敏配置
ccr export --no-secrets -o config.toml

# 添加到 git(不会泄露密钥)
git add config.toml
git commit -m "Update CCR configuration template"
```

### 5. 配置审查

导出配置供审查或文档化：

```bash
ccr export --no-secrets -o review.toml
```

## 文件名规则

### 默认文件名

格式：`ccr_config_<timestamp>.toml`

示例：
- `ccr_config_20250110_120530.toml`
- `ccr_config_20250110_143022.toml`

### 自定义文件名

```bash
# 使用日期
ccr export -o "ccr-$(date +%Y%m%d).toml"

# 使用描述性名称
ccr export -o production-backup.toml

# 使用版本号
ccr export -o ccr-config-v1.0.toml
```

## 导出后操作

### 验证导出

```bash
# 导出后验证文件
ccr export -o backup.toml
cat backup.toml

# 或使用 toml 验证工具
toml-verify backup.toml
```

### 安全存储

```bash
# 加密存储(包含密钥的导出)
ccr export -o config.toml
gpg -c config.toml  # 加密
rm config.toml      # 删除明文
```

### 远程备份

```bash
# 备份到云存储
ccr export -o ccr-backup.toml
rclone copy ccr-backup.toml remote:backups/
```

## 与配置文件的区别

| 项目 | 导出文件 | 配置文件 |
|------|---------|----------|
| 位置 | 用户指定 | `~/.ccs_config.toml` |
| 用途 | 备份/迁移 | 日常使用 |
| 修改 | 不影响运行 | 立即生效 |
| 密钥 | 可选脱敏 | 完整密钥 |

## 注意事项

::: warning 安全提示
- 默认导出包含完整 API 密钥,请妥善保管
- 使用 `--no-secrets` 导出用于分享的配置
- 不要将包含密钥的导出文件提交到公开仓库
:::

::: tip 最佳实践
- 定期备份配置
- 备份文件使用加密存储
- 分享配置时使用 `--no-secrets` 选项
- 重要备份保存多个副本
:::

## 相关命令

- [import](./import) - 导入配置
- [init](./init) - 初始化配置
- [validate](./validate) - 验证导出的配置
