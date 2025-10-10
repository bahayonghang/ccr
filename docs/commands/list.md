# list - 列出配置

`list` 命令列出所有可用的 API 配置，显示详细信息和验证状态。

## 📖 命令格式

```bash
ccr list

# 别名
ccr ls
```

## 💡 使用示例

### 基本使用

```bash
ccr list
```

### 输出示例

```
可用配置列表
════════════════════════════════════════════════════════════════
配置文件: /home/user/.ccs_config.toml
默认配置: anthropic
当前配置: anthropic
────────────────────────────────────────────────────────────────
▶ anthropic - Anthropic 官方 API
    Base URL: https://api.anthropic.com
    Token: sk-a...here
    Model: claude-sonnet-4-5-20250929
    Small Fast Model: claude-3-5-haiku-20241022
    状态: ✓ 配置完整
  anyrouter - AnyRouter 代理服务
  openrouter - OpenRouter 服务

✓ 共找到 3 个配置
```

## 📊 输出说明

### 标题部分

```
可用配置列表
════════════════════════════════════════════════════════════════
配置文件: /home/user/.ccs_config.toml
默认配置: anthropic
当前配置: anthropic
```

显示：
- 配置文件路径
- 默认配置（`default_config` 字段）
- 当前活跃配置（`current_config` 字段）

### 配置列表

```
▶ anthropic - Anthropic 官方 API
    Base URL: https://api.anthropic.com
    Token: sk-a...here
    Model: claude-sonnet-4-5-20250929
    Small Fast Model: claude-3-5-haiku-20241022
    状态: ✓ 配置完整
```

**标记说明**:
- `▶` - 当前活跃配置（绿色高亮）
- `  ` - 其他配置

**详细信息**（仅当前配置）:
- Base URL - API 端点地址
- Token - 认证令牌（已掩码）
- Model - 默认模型名称
- Small Fast Model - 快速小模型（可选）
- 状态 - 配置验证结果

### Token 掩码

```rust
// sk-ant-1234567890abcdef
// 显示为: sk-a...cdef

pub fn mask_sensitive(value: &str) -> String {
    if value.len() <= 10 {
        "*".repeat(value.len())
    } else {
        format!("{}...{}", &value[..4], &value[value.len() - 4..])
    }
}
```

**规则**:
- 长度 ≤ 10: 全部掩码为 `*`
- 长度 > 10: 显示前 4 位和后 4 位

### 验证状态

```
✓ 配置完整      # 绿色 - 所有必填字段都存在
✗ 配置不完整    # 红色 - 缺少必填字段
```

## 🔍 配置验证

list 命令会自动验证每个配置的完整性：

```rust
match section.validate() {
    Ok(_) => println!("    状态: {}", "✓ 配置完整".green()),
    Err(e) => println!("    状态: {} - {}", "✗ 配置不完整".red(), e),
}
```

**验证项**:
- ✅ `base_url` 存在且非空
- ✅ `base_url` 以 `http://` 或 `https://` 开头
- ✅ `auth_token` 存在且非空
- ✅ `model` 如果提供则非空

## 📝 配置排序

配置按字母顺序排序：

```rust
pub fn list_sections(&self) -> Vec<String> {
    let mut names: Vec<String> = self.sections.keys().cloned().collect();
    names.sort();  // ← 字母排序
    names
}
```

**示例**:
```
anyrouter
anthropic
openrouter
```

## 🎨 彩色输出

### 颜色方案

```rust
ColorOutput::title("可用配置列表");           // 蓝色粗体
ColorOutput::info("配置文件: ...");          // 蓝色
ColorOutput::config_status("anthropic", true, ...);  // 绿色（当前）
ColorOutput::success("共找到 3 个配置");      // 绿色
```

### 符号使用

- `▶` - 当前配置指示器（绿色）
- `✓` - 验证通过（绿色）
- `✗` - 验证失败（红色）
- `ℹ` - 信息提示（蓝色）
- `═` - 标题分隔线
- `─` - 内容分隔线

## ⚠️ 错误处理

### 配置文件不存在

```bash
$ ccr list

✗ 配置文件不存在: /home/user/.ccs_config.toml
  建议: 请运行安装脚本创建配置文件
```

**退出码**: 11

**解决方案**:
```bash
# 创建配置文件
vim ~/.ccs_config.toml

# 或从 CCS 安装
cd ccs
./scripts/install/install.sh
```

### 配置文件格式错误

```bash
$ ccr list

✗ 配置格式无效: TOML 解析失败: unexpected character
```

**退出码**: 14

**解决方案**:
```bash
# 检查 TOML 语法
cat ~/.ccs_config.toml

# 验证格式
ccr validate
```

### 无配置节

```bash
$ ccr list

可用配置列表
────────────────────────────────────────
⚠ 未找到任何配置节
```

**退出码**: 0（不是错误）

**解决方案**:
```bash
# 添加配置节
vim ~/.ccs_config.toml
```

## 💻 编程接口

### 在代码中使用

```rust
use ccr::commands::list_command;

fn main() -> Result<()> {
    list_command()?;
    Ok(())
}
```

### 获取配置列表

```rust
use ccr::config::ConfigManager;

let manager = ConfigManager::default()?;
let config = manager.load()?;
let sections = config.list_sections();

for section_name in sections {
    let section = config.get_section(&section_name)?;
    println!("{}: {}", section_name, section.display_description());
}
```

## 🌐 Web API

### 获取配置列表

```http
GET /api/configs
```

**响应**:
```json
{
  "success": true,
  "data": {
    "current_config": "anthropic",
    "default_config": "anthropic",
    "configs": [
      {
        "name": "anthropic",
        "description": "Anthropic 官方 API",
        "base_url": "https://api.anthropic.com",
        "auth_token": "sk-a...here",
        "model": "claude-sonnet-4-5-20250929",
        "small_fast_model": "claude-3-5-haiku-20241022",
        "is_current": true,
        "is_default": true
      },
      {
        "name": "anyrouter",
        "description": "AnyRouter 代理服务",
        "base_url": "https://api.anyrouter.ai/v1",
        "auth_token": "your...here",
        "model": "claude-sonnet-4-5-20250929",
        "small_fast_model": null,
        "is_current": false,
        "is_default": false
      }
    ]
  }
}
```

## 🔗 相关命令

- [current](/commands/current) - 查看当前配置详情
- [switch](/commands/switch) - 切换配置
- [validate](/commands/validate) - 验证所有配置

## 📚 相关文档

- [配置文件格式](/installation/configuration)
- [配置管理 API](/api/config)

