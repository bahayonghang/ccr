# import - 导入配置

从文件导入配置，支持合并或替换模式。

## 用法

```bash
ccr import <FILE> [OPTIONS]
```

## 参数

- `<FILE>`: 要导入的配置文件路径（必需）

## 选项

- `--merge`: 合并模式（保留现有配置，添加新配置）
- `--no-backup`: 导入前不备份当前配置

## 功能特性

- 支持合并或替换模式
- 导入前自动备份
- 配置验证
- 详细的导入摘要

## 示例

```bash
# 合并模式（推荐）
ccr import config.toml --merge

# 替换模式（完全替换）
ccr import config.toml

# 导入时不备份
ccr import config.toml --no-backup
```

::: danger 警告
使用替换模式会完全覆盖现有配置。除非你确定要这样做，否则建议使用 `--merge` 选项。
:::

## 导入模式

### 合并模式（--merge）

保留现有配置，添加或更新新配置：

```bash
# 现有配置
[anthropic]
...

# 导入文件
[anyrouter]
...
[newconfig]
...

# 导入后（合并）
[anthropic]  # 保留
...
[anyrouter]  # 添加/更新
...
[newconfig]  # 添加
...
```

### 替换模式（默认）

完全替换现有配置：

```bash
# 现有配置
[anthropic]
...
[oldconfig]
...

# 导入文件
[anyrouter]
...

# 导入后（替换）
[anyrouter]  # 仅保留导入的配置
...
# anthropic 和 oldconfig 被删除
```

## 示例输出

### 合并模式

```bash
$ ccr import new-configs.toml --merge
Importing configuration...
✓ Configuration file validated
✓ Current configuration backed up
✓ Merging configurations...
  + Added: anyrouter
  ↻ Updated: anthropic
  ✓ Preserved: oldconfig
✓ Configuration imported successfully

Summary:
  Added: 1
  Updated: 1
  Preserved: 1
  Total configurations: 3
```

### 替换模式

```bash
$ ccr import new-configs.toml
⚠ Warning: This will replace all existing configurations!
Continue? (y/N): y
Importing configuration...
✓ Configuration file validated
✓ Current configuration backed up
✓ Replacing configurations...
  - Removed: oldconfig
  + Added: anyrouter
  + Added: production
✓ Configuration imported successfully

Summary:
  Removed: 1
  Added: 2
  Total configurations: 2
```

## 使用场景

### 1. 恢复备份

从备份恢复配置：

```bash
# 恢复完整备份（替换）
ccr import backup.toml

# 恢复部分配置（合并）
ccr import partial-backup.toml --merge
```

### 2. 迁移到新机器

将配置迁移到新环境：

```bash
# 在旧机器导出
ccr export -o migration.toml

# 在新机器导入
ccr import migration.toml
```

### 3. 团队配置同步

同步团队配置模板：

```bash
# 获取团队模板
curl -o team-config.toml https://example.com/team-config.toml

# 合并到本地配置
ccr import team-config.toml --merge

# 手动添加个人 API 密钥
vim ~/.ccs_config.toml
```

### 4. 批量添加配置

批量添加多个新配置：

```bash
# 创建新配置文件
cat > new-configs.toml << EOF
[dev]
base_url = "https://api-dev.example.com"
auth_token = "dev-token"
model = "claude-3-5-haiku-20241022"

[staging]
base_url = "https://api-staging.example.com"
auth_token = "staging-token"
model = "claude-sonnet-4-5-20250929"
EOF

# 合并导入
ccr import new-configs.toml --merge
```

### 5. 配置重置

重置到默认配置：

```bash
# 导出当前配置作为备份
ccr export -o before-reset.toml

# 重置到初始模板
ccr init --force

# 如需恢复
ccr import before-reset.toml
```

## 导入验证

导入前会自动验证配置文件：

### 验证项目

1. **文件格式**：TOML 语法正确性
2. **必需字段**：所有配置包含必需字段
3. **URL 格式**：base_url 格式正确
4. **配置名称**：配置名称唯一且有效

### 验证失败

```bash
$ ccr import invalid-config.toml
Importing configuration...
✗ Configuration validation failed:
  - Invalid TOML syntax at line 5
  - Configuration 'test' missing required field: auth_token
  - Invalid URL in 'demo': "not-a-url"

✗ Import aborted
```

## 自动备份

默认情况下，导入前会自动备份当前配置：

```bash
# 备份位置
~/.ccs_config.toml.bak.<timestamp>

# 示例
~/.ccs_config.toml.bak.20250110_120530
```

禁用备份：

```bash
ccr import config.toml --no-backup
```

::: warning 注意
禁用备份后，如果导入出错，将无法恢复原配置！
:::

## 导入后操作

### 验证导入

```bash
# 导入后立即验证
ccr import config.toml --merge && ccr validate

# 查看导入的配置
ccr list
```

### 切换到导入的配置

```bash
# 导入并切换
ccr import config.toml --merge && ccr switch newconfig
```

### 查看变更历史

```bash
# 查看导入操作
ccr history -t import
```

## 常见问题

### Q: 合并时如何处理重复配置？

**A:** 导入的配置会覆盖同名的现有配置。

```bash
# 现有
[anthropic]
auth_token = "old-token"

# 导入
[anthropic]
auth_token = "new-token"

# 合并后
[anthropic]
auth_token = "new-token"  # 使用导入的值
```

### Q: 如何仅导入特定配置？

**A:** 编辑导入文件，仅保留需要的配置：

```bash
# 创建临时文件，仅包含需要的配置
cat > temp-import.toml << EOF
[needed-config]
...
EOF

ccr import temp-import.toml --merge
rm temp-import.toml
```

### Q: 导入失败如何恢复？

**A:** 使用自动备份恢复：

```bash
# 查找备份
ls -lt ~/.ccs_config.toml.bak.*

# 恢复最新备份
cp ~/.ccs_config.toml.bak.20250110_120530 ~/.ccs_config.toml
```

### Q: 可以导入部分字段吗？

**A:** 不可以。每个配置必须包含所有必需字段。如需部分更新，请手动编辑配置文件。

## 最佳实践

1. **优先使用合并模式**
   ```bash
   ccr import config.toml --merge
   ```

2. **导入前先备份**
   ```bash
   ccr export -o manual-backup.toml
   ccr import new-config.toml --merge
   ```

3. **导入后验证**
   ```bash
   ccr import config.toml --merge
   ccr validate
   ```

4. **测试导入**
   ```bash
   # 先在测试目录验证
   cat config.toml
   ccr validate  # 验证语法

   # 确认无误后再导入
   ccr import config.toml --merge
   ```

5. **记录导入操作**
   ```bash
   ccr import config.toml --merge
   ccr history -t import
   ```

## 相关命令

- [export](./export) - 导出配置
- [init](./init) - 初始化配置
- [validate](./validate) - 验证配置
- [list](./list) - 查看导入结果
