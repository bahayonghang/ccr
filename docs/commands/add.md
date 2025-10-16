# add - 添加新配置

交互式地添加新的配置方案到 CCR 配置文件中。

## 用法

```bash
ccr add
```

## 功能特性

- 📝 **交互式输入** - 友好的提示引导配置填写
- ✅ **实时验证** - 自动验证配置完整性
- 👀 **配置预览** - 添加前显示完整配置预览
- 🔒 **敏感信息保护** - Auth Token 自动脱敏显示
- 🔍 **重名检查** - 自动检测配置名称冲突
- 🏷️ **完整字段支持** - 支持所有配置字段（必填和可选）

## 交互流程

### 必填字段（带 * 标记）

1. **配置名称** *
   - 用途：配置的唯一标识符
   - 示例：`my_provider`, `dev_api`, `prod_config`
   - 要求：不能与现有配置重名

2. **Base URL** *
   - 用途：API 端点地址
   - 示例：`https://api.example.com`
   - 要求：必须以 `http://` 或 `https://` 开头

3. **Auth Token** *
   - 用途：API 认证令牌
   - 示例：`sk-ant-xxxxx`, `your-api-key`
   - 要求：不能为空

### 可选字段

4. **描述**
   - 用途：配置说明文字
   - 示例：`我的 API 提供商`, `开发环境配置`

5. **主模型**
   - 用途：默认使用的 AI 模型
   - 示例：`claude-3-5-sonnet-20241022`

6. **快速小模型**
   - 用途：用于快速响应的轻量级模型
   - 示例：`claude-3-5-haiku-20241022`

### 分类字段（可选）

7. **提供商名称**
   - 用途：标识服务提供商
   - 示例：`anyrouter`, `glm`, `moonshot`

8. **提供商类型**
   - 选项 1：官方中转
   - 选项 2：第三方模型
   - 用途：配置分类显示

9. **账号标识**
   - 用途：区分同一提供商的不同账号
   - 示例：`github_5953`, `work_account`

10. **标签**
    - 用途：灵活分类和筛选
    - 示例：`free,stable,high-speed`
    - 格式：多个标签用逗号分隔

## 示例输出

### 完整添加流程

```bash
$ ccr add

添加新配置
══════════════════════════════════════════════════════════════

ℹ 请按照提示输入配置信息
ℹ 标记 * 的为必填项，其他为可选项

* 配置名称: test_provider
  示例: my_provider

  描述: 测试 API 提供商
  示例: 我的API提供商 (按 Enter 跳过)

* Base URL: https://api.test-provider.com
  示例: https://api.example.com

* Auth Token: sk-test-1234567890abcdef
  示例: sk-ant-xxxxx

  主模型: test-model-v1
  示例: claude-3-5-sonnet-20241022 (按 Enter 跳过)

  快速小模型: test-small-v1
  示例: claude-3-5-haiku-20241022 (按 Enter 跳过)

──────────────────────────────────────────────────────────────

ℹ 以下为分类字段（可选）

  提供商名称: test_provider
  示例: anyrouter, glm, moonshot (按 Enter 跳过)

  提供商类型:
    1) 官方中转
    2) 第三方模型
    留空跳过
  请选择 [1/2]: 2

  账号标识: test_account_001
  示例: github_5953 (按 Enter 跳过)

  标签 (逗号分隔): test,temporary,demo
  示例: 'free,stable,high-speed' (按 Enter 跳过)

──────────────────────────────────────────────────────────────

▶ 验证配置
✓ 配置验证通过

▶ 配置预览

  配置名称: test_provider
  描述: 测试 API 提供商
  Base URL: https://api.test-provider.com
  Auth Token: sk-t...cdef (已脱敏)
  主模型: test-model-v1
  快速小模型: test-small-v1
  提供商: test_provider
  提供商类型: 第三方模型
  账号: test_account_001
  标签: test, temporary, demo

? 确认添加此配置? [Y/n]: Y

──────────────────────────────────────────────────────────────

▶ 保存配置
✓ 配置 'test_provider' 添加成功

ℹ 后续操作:
  • 运行 'ccr list' 查看所有配置
  • 运行 'ccr switch test_provider' 切换到此配置
```

### 配置名称重复

```bash
$ ccr add

添加新配置
══════════════════════════════════════════════════════════════

ℹ 请按照提示输入配置信息
ℹ 标记 * 的为必填项，其他为可选项

* 配置名称: anthropic

✗ 配置 'anthropic' 已存在
ℹ 提示: 使用 'ccr list' 查看已有配置
```

### 取消添加

```bash
...
▶ 配置预览

  配置名称: test_config
  Base URL: https://api.test.com
  Auth Token: sk-t...ken

? 确认添加此配置? [Y/n]: n

ℹ 已取消添加
```

## 使用场景

### 场景 1: 添加新的 API 提供商

```bash
# 获取了新的 API 服务
ccr add

# 输入配置信息
# - 配置名称: newapi
# - Base URL: https://api.newprovider.com
# - Auth Token: your-new-api-key

# 切换使用
ccr switch newapi
```

### 场景 2: 添加不同环境的配置

```bash
# 添加开发环境
ccr add
# 配置名称: dev
# Base URL: https://api-dev.example.com

# 添加测试环境
ccr add
# 配置名称: staging
# Base URL: https://api-staging.example.com

# 添加生产环境
ccr add
# 配置名称: production
# Base URL: https://api.example.com

# 快速切换环境
ccr dev
ccr staging
ccr production
```

### 场景 3: 管理多个账号

```bash
# 添加个人账号
ccr add
# 配置名称: personal_account
# 账号标识: personal_github_123
# 标签: personal

# 添加工作账号
ccr add
# 配置名称: work_account
# 账号标识: work_company_456
# 标签: work,priority

# 查看所有账号
ccr list
```

### 场景 4: 临时测试配置

```bash
# 添加临时测试配置
ccr add
# 配置名称: test_temp
# 标签: temporary,test

# 使用测试
ccr switch test_temp

# 测试完成后删除
ccr delete test_temp
```

## 字段验证规则

### Base URL 验证

✅ **有效格式：**
```
https://api.example.com
https://api.example.com/v1
http://localhost:8000
https://subdomain.example.com:8080/path
```

❌ **无效格式：**
```
api.example.com          # 缺少协议
ftp://api.example.com    # 不支持的协议
not-a-url                # 无效格式
```

### Auth Token 验证

✅ **有效：**
- 任何非空字符串

❌ **无效：**
- 空字符串
- 仅包含空格

### 配置名称要求

✅ **推荐格式：**
```
anthropic
anyrouter_main
dev_config
test-provider
```

❌ **避免使用：**
- 包含特殊字符（除了 `-` 和 `_`）
- 中文或非 ASCII 字符（可能导致兼容性问题）

## 输入技巧

### 1. 快速跳过可选字段

对于可选字段，直接按 Enter 即可跳过：

```bash
  描述: <直接按 Enter>
  示例: 我的API提供商 (按 Enter 跳过)
```

### 2. 粘贴长文本

Auth Token 等长字段支持直接粘贴：

```bash
* Auth Token: <Ctrl+V 粘贴完整 token>
```

### 3. 标签输入

标签之间用逗号分隔，空格会被自动去除：

```bash
  标签 (逗号分隔): free, stable, high-speed
  # 结果：["free", "stable", "high-speed"]
```

## 常见错误处理

### 错误 1: 配置名称已存在

**提示：**
```
✗ 配置 'xxx' 已存在
ℹ 提示: 使用 'ccr list' 查看已有配置
```

**解决：**
- 使用不同的配置名称
- 或使用 `ccr delete xxx` 删除现有配置后重新添加

### 错误 2: Base URL 格式无效

**提示：**
```
✗ 配置验证失败: base_url 必须以 http:// 或 https:// 开头
```

**解决：**
- 确保 URL 包含正确的协议前缀
- 示例：`https://api.example.com`

### 错误 3: 必填字段为空

**提示：**
```
* Base URL: <直接按 Enter>
  提示: 例如: https://api.example.com (必填)
* Base URL: <需要重新输入>
```

**解决：**
- 必填字段必须输入内容
- 系统会循环提示直到输入有效值

## 与手动编辑的对比

| 方式 | 优点 | 缺点 | 适用场景 |
|------|------|------|----------|
| **ccr add** | • 交互友好<br>• 实时验证<br>• 不会语法错误<br>• 字段提示完整 | • 输入较慢<br>• 不适合批量操作 | • 添加单个配置<br>• 不熟悉 TOML 格式<br>• 需要字段提示 |
| **手动编辑** | • 批量修改快速<br>• 可复制粘贴<br>• 灵活性高 | • 可能出现语法错误<br>• 需要了解格式<br>• 容易漏字段 | • 批量添加配置<br>• 复制现有配置<br>• 熟悉 TOML 格式 |

## 添加后操作

### 1. 查看新配置

```bash
# 查看所有配置
ccr list

# 查看配置详情（需先切换）
ccr switch test_provider
ccr current
```

### 2. 验证配置

```bash
# 验证所有配置
ccr validate
```

### 3. 切换使用

```bash
# 切换到新配置
ccr switch test_provider

# 或简写
ccr test_provider
```

### 4. 导出备份

```bash
# 导出包含新配置的完整配置
ccr export -o backup-with-new-config.toml
```

## 最佳实践

### 1. 使用描述性配置名称

✅ **推荐：**
```
anthropic_official
anyrouter_primary
glm_dev_account
```

❌ **不推荐：**
```
config1
test
a
```

### 2. 填写描述信息

即使是可选字段，也建议填写描述：

```
描述: Anthropic 官方 API - 主要用于生产环境
```

这样在 `ccr list` 时更容易识别。

### 3. 合理使用标签

使用标签进行分类管理：

```
标签: production,primary,high-priority
标签: development,test,temporary
标签: free,backup
```

### 4. 设置账号标识

对于同一提供商的多个账号，使用账号标识区分：

```
配置名称: anyrouter_personal
账号标识: github_5953

配置名称: anyrouter_work
账号标识: company_account
```

### 5. 添加后立即测试

```bash
# 添加配置
ccr add

# 立即测试
ccr switch new_config
ccr validate
```

## 批量添加建议

如需批量添加多个配置，建议：

### 方法 1: 使用脚本辅助

```bash
# 准备配置数据
cat > configs.txt << EOF
dev|https://api-dev.example.com|dev-token|开发环境
staging|https://api-staging.example.com|staging-token|测试环境
prod|https://api.example.com|prod-token|生产环境
EOF

# 逐个添加（仍需手动确认）
while IFS='|' read -r name url token desc; do
  echo "添加配置: $name"
  # 需要手动运行 ccr add
done < configs.txt
```

### 方法 2: 手动编辑配置文件

```bash
# 编辑配置文件
vim ~/.ccs_config.toml

# 添加多个配置
[dev]
description = "开发环境"
base_url = "https://api-dev.example.com"
auth_token = "dev-token"

[staging]
description = "测试环境"
base_url = "https://api-staging.example.com"
auth_token = "staging-token"

# 验证
ccr validate
```

### 方法 3: 使用导入功能

```bash
# 创建配置文件
cat > new-configs.toml << EOF
[dev]
base_url = "https://api-dev.example.com"
auth_token = "dev-token"

[staging]
base_url = "https://api-staging.example.com"
auth_token = "staging-token"
EOF

# 导入（合并模式）
ccr import new-configs.toml --merge
```

## 注意事项

::: warning 重要提示
- 配置名称不可重复，添加前会自动检查
- Auth Token 等敏感信息会在预览时自动脱敏
- 添加的配置立即生效，但不会自动切换
:::

::: tip 提示
- 必填字段不能为空，会循环提示直到输入有效值
- 可选字段可以直接按 Enter 跳过
- 提供商类型和标签有助于配置分类管理
:::

## 相关命令

- [list](./list) - 查看所有配置（包括新添加的）
- [delete](./delete) - 删除不需要的配置
- [switch](./switch) - 切换到新添加的配置
- [validate](./validate) - 验证配置完整性
- [import](./import) - 批量导入配置
- [export](./export) - 导出配置备份

