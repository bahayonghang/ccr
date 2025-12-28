# optimize - 优化配置文件

优化配置文件结构，按字母顺序重新排列配置节，提升可读性和可维护性。

## 用法

```bash
ccr optimize
```

## 功能特性

- 📊 **自动排序** - 按配置名称字母顺序重新排列
- 💾 **保持内容** - 仅调整顺序，不修改配置内容
- 🔒 **自动备份** - 优化前自动备份原配置文件
- ✅ **验证完整性** - 优化后验证配置文件有效性
- 📋 **历史记录** - 记录优化操作到审计日志

## 示例

### 基本用法

```bash
# 优化配置文件
ccr optimize
```

## 示例输出

### 成功优化

```bash
$ ccr optimize

优化配置文件
══════════════════════════════════════════════════════════════

▶ 步骤 1/4: 加载配置
✓ 读取配置文件: ~/.ccs_config.toml
✓ 解析配置: 5 个配置节

▶ 步骤 2/4: 分析配置
配置节列表 (优化前):
  1. anthropic
  2. test_config
  3. anyrouter
  4. backup_config
  5. openai

配置节列表 (优化后):
  1. anyrouter
  2. anthropic
  3. backup_config
  4. openai
  5. test_config

ℹ 将按字母顺序重新排列配置节

▶ 步骤 3/4: 创建备份
✓ 备份创建: ~/.ccs_config.toml.20250110_150030.bak

▶ 步骤 4/4: 优化并保存
✓ 重新排列配置节
✓ 保存优化后的配置
✓ 验证配置有效性

──────────────────────────────────────────────────────────────

✓ 配置文件已优化

ℹ 优化详情:
  • 配置节数量: 5
  • 顺序调整: anyrouter, anthropic, backup_config, openai, test_config
  • 备份位置: ~/.ccs_config.toml.20250110_150030.bak

ℹ 后续操作:
  • 运行 'ccr list' 查看优化后的配置列表
  • 运行 'ccr validate' 验证配置完整性
```

### 配置已是最优状态

```bash
$ ccr optimize

优化配置文件
══════════════════════════════────────────────────────────────

▶ 步骤 1/4: 加载配置
✓ 读取配置文件: ~/.ccs_config.toml
✓ 解析配置: 3 个配置节

▶ 步骤 2/4: 分析配置
配置节列表 (当前):
  1. anthropic
  2. openai
  3. test_config

ℹ 配置节已按字母顺序排列

✓ 配置文件已是最优状态，无需优化
```

### 无配置节

```bash
$ ccr optimize

优化配置文件
══════════════════════════════────────────────────────────────

▶ 步骤 1/4: 加载配置
✓ 读取配置文件: ~/.ccs_config.toml

▶ 步骤 2/4: 分析配置
ℹ 配置文件中没有配置节

✓ 无需优化
```

## 使用场景

### 场景 1: 配置文件维护

```bash
# 定期优化配置文件，保持结构清晰
ccr optimize
```

### 场景 2: 添加多个配置后

```bash
# 添加多个配置
ccr add  # 添加 config1
ccr add  # 添加 config2
ccr add  # 添加 config3

# 优化配置文件顺序
ccr optimize

# 查看优化后的列表
ccr list
```

### 场景 3: 导入配置后

```bash
# 导入配置文件
ccr import external-config.toml --merge

# 优化配置文件
ccr optimize
```

### 场景 4: 团队协作

```bash
# 优化配置文件，确保团队成员看到相同顺序
ccr optimize

# 导出标准化配置
ccr export -o team-config.toml
```

## 优化前后对比

### 优化前的配置文件

```toml
[global]
default_config = "anthropic"

[test_config]
base_url = "https://api.test.com"
auth_token = "test-token"

[anthropic]
base_url = "https://api.anthropic.com"
auth_token = "sk-ant-xxx"

[anyrouter]
base_url = "https://anyrouter.top"
auth_token = "any-token"

[backup_config]
base_url = "https://api.backup.com"
auth_token = "backup-token"
```

### 优化后的配置文件

```toml
[global]
default_config = "anthropic"

[anyrouter]
base_url = "https://anyrouter.top"
auth_token = "any-token"

[anthropic]
base_url = "https://api.anthropic.com"
auth_token = "sk-ant-xxx"

[backup_config]
base_url = "https://api.backup.com"
auth_token = "backup-token"

[test_config]
base_url = "https://api.test.com"
auth_token = "test-token"
```

**变化：**
- ✅ 配置节按字母顺序排列
- ✅ `[global]` 等特殊节保持在顶部
- ✅ 配置内容完全保持不变
- ✅ 注释和空行保留（如果有）

## 优化规则

### 排序规则

1. **特殊配置节**（保持在顶部）：
   - `[global]`
   - `[settings]`

2. **普通配置节**：
   - 按字母顺序（A-Z）排列
   - 不区分大小写

### 保持不变

- ✅ 配置节内部的字段顺序
- ✅ 配置值和内容
- ✅ 注释（如果有）
- ✅ 空行格式（在合理范围内）

## 备份机制

### 自动备份

每次优化前都会自动备份配置文件：

```bash
备份位置: ~/.ccs_config.toml.20250110_150030.bak
          或
          ~/.ccr/platforms/<platform>/profiles.toml.20250110_150030.bak
```

### 备份文件命名

```
<原文件名>.<时间戳>.bak
```

时间戳格式：`YYYYMMDD_HHMMSS`

### 恢复备份

如果需要恢复到优化前：

```bash
# 找到最近的备份
ls -lt ~/.ccs_config.toml.*.bak | head -1

# 恢复备份
cp ~/.ccs_config.toml.20250110_150030.bak ~/.ccs_config.toml

# 验证恢复
ccr list
```

## 优化效果

### 提升可读性

优化后的配置文件：
- 📖 配置节按字母顺序排列，易于查找
- 🔍 结构清晰，一目了然
- 🤝 团队协作时减少冲突

### 易于维护

- 📊 添加新配置时位置固定
- 🔄 合并配置文件时冲突更少
- 📋 文件对比时更容易识别变化

## 与其他命令配合

### 优化后验证

```bash
# 优化配置
ccr optimize

# 验证配置
ccr validate

# 查看配置列表
ccr list
```

### 优化后导出

```bash
# 优化配置
ccr optimize

# 导出标准化配置
ccr export -o optimized-config.toml
```

### 导入后优化

```bash
# 导入配置
ccr import external-config.toml --merge

# 优化顺序
ccr optimize
```

## 常见问题

### Q: 优化会修改配置内容吗？

**A:** 不会，优化只调整配置节的顺序：
- ✅ 配置名称保持不变
- ✅ 配置值保持不变
- ✅ 字段顺序保持不变
- ✅ 只改变配置节的排列顺序

### Q: 优化会影响当前配置吗？

**A:** 不会：
- ✅ 当前激活的配置不受影响
- ✅ Claude Code 继续正常工作
- ✅ 只是重新排列配置文件的顺序

### Q: 优化前需要备份吗？

**A:** 不需要手动备份，优化命令会自动备份：
- ✅ 自动创建备份文件
- ✅ 备份文件带时间戳
- ✅ 可以随时恢复

### Q: 优化后需要重新切换配置吗？

**A:** 不需要：
- ✅ 当前配置保持不变
- ✅ 所有配置仍然可用
- ✅ 无需任何额外操作

### Q: 优化的排序规则是什么？

**A:** 字母顺序（A-Z）：
- 按配置名称排序
- 不区分大小写
- 特殊节（如 `[global]`）保持在顶部

## 最佳实践

### 1. 定期优化

```bash
# 每月或添加多个配置后优化一次
ccr optimize
```

### 2. 优化后验证

```bash
# 优化后验证配置完整性
ccr optimize
ccr validate
```

### 3. 优化后查看

```bash
# 优化后查看配置列表
ccr optimize
ccr list
```

### 4. 团队协作

```bash
# 优化配置后导出给团队
ccr optimize
ccr export -o team-standard-config.toml
```

### 5. 合并前优化

```bash
# 导入外部配置后优化
ccr import external.toml --merge
ccr optimize
```

## 注意事项

::: tip 提示
- 优化操作是**幂等的**，多次执行不会出错
- 优化不会修改配置内容，仅调整顺序
- 优化会自动创建备份，可以随时恢复
:::

::: warning 注意
- 优化会修改配置文件
- 优化后配置节顺序会变化
- 如果使用版本控制（Git），会看到配置文件变更
:::

::: tip 建议
- 定期运行 `optimize` 保持配置文件整洁
- 添加多个配置后运行 `optimize`
- 团队协作时使用优化后的配置
- 优化后运行 `validate` 确保配置有效
:::

## 相关命令

- [validate](./validate) - 验证配置完整性
- [list](./list) - 查看所有配置
- [export](./export) - 导出配置
- [import](./import) - 导入配置
- [clean](./clean) - 清理过期备份
