# delete - 删除配置

删除指定的配置方案，支持安全检查和确认提示。

## 用法

```bash
ccr delete <config_name> [OPTIONS]
```

## 参数

- `<config_name>`: 要删除的配置名称（必需）

## 选项

- `-f, --force`: 跳过确认提示，直接删除（危险操作）

## 功能特性

- ⚠️ **安全检查** - 检测当前配置和默认配置
- 🤔 **删除确认** - 默认需要用户确认（除非使用 --force）
- 📊 **配置预览** - 删除前显示配置详情
- 💡 **智能提示** - 删除当前/默认配置时给出警告和建议
- 🚫 **不可恢复** - 操作不可逆，请谨慎使用

## 示例

### 基本用法

```bash
# 删除指定配置（需要确认）
ccr delete test_config

# 强制删除（跳过确认）
ccr delete test_config --force

# 使用短选项
ccr delete test_config -f
```

## 示例输出

### 正常删除流程

```bash
$ ccr delete test_provider

删除配置: test_provider
══════════════════════════════════════════════════════════════

▶ 步骤 1/3: 检查配置
✓ 配置 'test_provider' 存在

▶ 步骤 2/3: 安全检查
✓ 安全检查完成

▶ 将要删除的配置信息

  配置名称: test_provider
  描述: 测试 API 提供商
  Base URL: https://api.test-provider.com
  提供商: test_provider

▶ 步骤 3/3: 确认删除
⚠ 此操作不可恢复！

? 确认删除配置 'test_provider'? [y/N]: y

──────────────────────────────────────────────────────────────

✓ 配置 'test_provider' 已删除

ℹ 后续操作:
  • 运行 'ccr list' 查看剩余配置
```

### 强制删除（--force）

```bash
$ ccr delete test_provider --force

删除配置: test_provider
══════════════════════════════════════════════════════════════

▶ 步骤 1/3: 检查配置
✓ 配置 'test_provider' 存在

▶ 步骤 2/3: 安全检查
✓ 安全检查完成

▶ 步骤 3/3: 执行删除 (--force 模式)
⚠ 跳过确认，直接删除

──────────────────────────────────────────────────────────────

✓ 配置 'test_provider' 已删除

ℹ 后续操作:
  • 运行 'ccr list' 查看剩余配置
```

### 删除当前配置（警告）

```bash
$ ccr delete anyrouter

删除配置: anyrouter
══════════════════════════════════════════════════════════════

▶ 步骤 1/3: 检查配置
✓ 配置 'anyrouter' 存在

▶ 步骤 2/3: 安全检查
⚠ 配置 'anyrouter' 是当前激活的配置

ℹ 删除当前配置后，您需要:
  1. 运行 'ccr list' 查看其他配置
  2. 运行 'ccr switch <name>' 切换到其他配置

✓ 安全检查完成

▶ 将要删除的配置信息

  配置名称: anyrouter
  描述: Anyrouter 代理服务
  Base URL: https://anyrouter.top
  提供商: anyrouter

▶ 步骤 3/3: 确认删除
⚠ 此操作不可恢复！

? 确认删除配置 'anyrouter'? [y/N]: y

──────────────────────────────────────────────────────────────

✓ 配置 'anyrouter' 已删除

⚠ 重要提示: 您刚刚删除了当前配置
ℹ 后续操作:
  1. 运行 'ccr list' 查看剩余配置
  2. 运行 'ccr switch <name>' 切换到其他配置
```

### 删除默认配置（警告）

```bash
$ ccr delete anthropic

删除配置: anthropic
══════════════════════════════════════════════════════════════

▶ 步骤 1/3: 检查配置
✓ 配置 'anthropic' 存在

▶ 步骤 2/3: 安全检查
⚠ 配置 'anthropic' 是默认配置

ℹ 删除后，请记得编辑 ~/.ccs_config.toml 设置新的 default_config

✓ 安全检查完成

...
```

### 配置不存在

```bash
$ ccr delete nonexistent

删除配置: nonexistent
══════════════════════════════════════════════════════════════

▶ 步骤 1/3: 检查配置
✗ 配置节 'nonexistent' 不存在

建议: 运行 'ccr list' 查看可用配置，或检查配置文件路径是否正确
```

### 取消删除

```bash
...
▶ 步骤 3/3: 确认删除
⚠ 此操作不可恢复！

? 确认删除配置 'test_config'? [y/N]: n

ℹ 已取消删除
```

## 使用场景

### 场景 1: 清理临时测试配置

```bash
# 添加临时测试配置
ccr add
# 配置名称: temp_test

# 测试完成后删除
ccr delete temp_test
```

### 场景 2: 移除过期配置

```bash
# 查看所有配置
ccr list

# 删除不再使用的配置
ccr delete old_config_2023
ccr delete deprecated_api
```

### 场景 3: 批量清理

```bash
# 删除多个旧配置
for config in test1 test2 test3; do
  ccr delete $config --force
done
```

### 场景 4: 账号迁移

```bash
# 查看当前配置
ccr current

# 添加新账号配置
ccr add
# 配置名称: new_account

# 切换到新配置
ccr switch new_account

# 删除旧账号配置
ccr delete old_account
```

## 安全检查机制

### 检查 1: 配置存在性

验证要删除的配置是否存在于配置文件中。

```bash
# 配置存在 → 继续
✓ 配置 'test' 存在

# 配置不存在 → 终止
✗ 配置节 'nonexistent' 不存在
```

### 检查 2: 当前配置检测

如果删除的是当前激活的配置，会显示警告：

```bash
⚠ 配置 'xxx' 是当前激活的配置

ℹ 删除当前配置后，您需要:
  1. 运行 'ccr list' 查看其他配置
  2. 运行 'ccr switch <name>' 切换到其他配置
```

**重要：** 即使删除当前配置，操作也会继续（在用户确认后）。删除后需要手动切换到其他配置。

### 检查 3: 默认配置检测

如果删除的是默认配置，会显示警告：

```bash
⚠ 配置 'xxx' 是默认配置

ℹ 删除后，请记得编辑 ~/.ccs_config.toml 设置新的 default_config
```

**注意：** 删除默认配置后，需要手动编辑配置文件更新 `default_config` 字段。

## 确认提示

### 默认模式（需要确认）

```bash
? 确认删除配置 'xxx'? [y/N]: 
```

**接受的输入：**
- `y` 或 `Y` - 确认删除
- `n`、`N` 或直接 Enter - 取消删除

### --force 模式（跳过确认）

使用 `--force` 参数可跳过确认：

```bash
ccr delete test --force
# 直接删除，不询问
```

::: danger 危险操作
使用 `--force` 会立即删除配置，无法恢复！仅在自动化脚本或确信要删除时使用。
:::

## 删除效果

### 配置文件变化

删除前：
```toml
[test_config]
base_url = "https://api.test.com"
auth_token = "test-token"

[production]
base_url = "https://api.prod.com"
auth_token = "prod-token"
```

删除后（`ccr delete test_config`）：
```toml
[production]
base_url = "https://api.prod.com"
auth_token = "prod-token"
```

### 不影响其他配置

删除操作仅移除指定配置，不影响：
- ✅ 其他配置
- ✅ 配置文件结构
- ✅ 当前 Claude Code 设置（除非删除的是当前配置）

## 恢复已删除的配置

::: warning 重要提示
删除操作不可逆！删除后配置无法通过 CCR 命令恢复。
:::

### 恢复方法

#### 方法 1: 从备份恢复

如果之前导出过配置：

```bash
# 查找备份文件
ls ~/backups/ccr-*.toml

# 导入备份
ccr import ~/backups/ccr-20250110.toml --merge
```

#### 方法 2: 从配置备份恢复

如果使用过 `ccr init --force` 或 `ccr import`，会有自动备份：

```bash
# 查找配置备份
ls -lt ~/.ccs_config.toml.*.bak

# 查看备份内容
cat ~/.ccs_config.toml.20250110_120530.bak

# 手动恢复配置节
vim ~/.ccs_config.toml
# 复制粘贴删除的配置节
```

#### 方法 3: 重新添加

使用 `ccr add` 重新创建配置：

```bash
ccr add
# 重新输入配置信息
```

## 批量删除

### 谨慎使用

批量删除前建议先备份：

```bash
# 备份当前配置
ccr export -o backup-before-cleanup.toml

# 批量删除
ccr delete test1 --force
ccr delete test2 --force
ccr delete test3 --force
```

### 脚本示例

```bash
#!/bin/bash
# 批量删除临时配置

# 要删除的配置列表
CONFIGS_TO_DELETE=(
  "temp_test1"
  "temp_test2"
  "old_config"
)

# 备份
echo "备份当前配置..."
ccr export -o "backup-$(date +%Y%m%d_%H%M%S).toml"

# 删除
for config in "${CONFIGS_TO_DELETE[@]}"; do
  echo "删除配置: $config"
  ccr delete "$config" --force
done

echo "完成！运行 'ccr list' 查看剩余配置"
```

### 按标签删除

虽然 CCR 不直接支持按标签删除，但可以结合脚本实现：

```bash
# 查找带有 temporary 标签的配置
# 需要手动查看或解析配置文件

# 手动删除
ccr delete temp_config1 --force
ccr delete temp_config2 --force
```

## 常见问题

### Q: 删除配置会影响 Claude Code 吗？

**A:** 取决于删除的配置：
- 如果删除**其他配置** → 不影响，Claude Code 继续使用当前配置
- 如果删除**当前配置** → Claude Code 仍使用旧的设置，直到你切换到其他配置

### Q: 可以恢复已删除的配置吗？

**A:** 不能直接恢复，但可以：
1. 从备份文件导入（如果有）
2. 从配置备份中复制（`~/.ccs_config.toml.*.bak`）
3. 使用 `ccr add` 重新创建

### Q: 删除默认配置会怎样？

**A:** 可以删除，但需要：
1. 手动编辑 `~/.ccs_config.toml`
2. 将 `default_config` 改为其他配置名称

### Q: 删除当前配置后会自动切换吗？

**A:** 不会自动切换，需要手动运行：
```bash
ccr list           # 查看可用配置
ccr switch <name>  # 切换到其他配置
```

### Q: 如何安全地删除配置？

**A:** 建议步骤：
```bash
# 1. 备份配置
ccr export -o backup.toml

# 2. 查看要删除的配置详情
ccr list

# 3. 如果是当前配置，先切换
ccr switch other_config

# 4. 删除配置
ccr delete old_config

# 5. 验证
ccr list
```

## 最佳实践

### 1. 删除前备份

```bash
# 养成习惯：删除前先备份
ccr export -o backup-before-delete.toml
ccr delete old_config
```

### 2. 避免删除当前配置

```bash
# 先切换，再删除
ccr switch new_config
ccr delete old_config
```

### 3. 使用描述性名称

便于识别和管理：

```bash
# 清晰的命名
ccr delete test_2024_jan_temp      # 容易识别的临时配置
ccr delete old_anthropic_backup    # 明确的备份配置
```

### 4. 定期清理

```bash
# 定期清理不再使用的配置
ccr list  # 查看所有配置
# 删除不需要的配置
```

### 5. 批量操作前测试

```bash
# 先测试单个删除
ccr delete test1

# 确认无误后再批量删除
for config in test2 test3 test4; do
  ccr delete $config --force
done
```

## 与手动编辑的对比

| 方式 | 优点 | 缺点 | 适用场景 |
|------|------|------|----------|
| **ccr delete** | • 安全检查<br>• 确认提示<br>• 智能警告<br>• 不会语法错误 | • 一次只能删除一个<br>• 需要交互 | • 删除单个配置<br>• 不熟悉配置文件<br>• 需要安全保护 |
| **手动编辑** | • 批量删除快速<br>• 可以编辑其他内容<br>• 无需确认 | • 可能出错<br>• 需要了解 TOML<br>• 无安全检查 | • 批量删除<br>• 同时修改其他配置<br>• 熟悉配置格式 |

## 注意事项

::: danger 危险操作
- 删除操作**不可恢复**
- 使用 `--force` 会**跳过确认**，立即删除
- 删除**当前配置**后需要手动切换
- 删除**默认配置**需要更新配置文件
:::

::: warning 注意
- 删除配置不会自动创建备份（建议手动备份）
- 删除不会影响已导出的配置文件
- 删除当前配置不会立即影响 Claude Code（直到切换）
:::

::: tip 建议
- 删除前使用 `ccr export` 备份
- 删除当前配置前先 `ccr switch` 到其他配置
- 对于重要配置，避免使用 `--force`
- 定期清理不再使用的临时配置
:::

## 相关命令

- [add](./add) - 添加新配置
- [list](./list) - 查看所有配置
- [switch](./switch) - 切换到其他配置
- [export](./export) - 备份配置
- [import](./import) - 从备份恢复
- [validate](./validate) - 验证配置完整性

