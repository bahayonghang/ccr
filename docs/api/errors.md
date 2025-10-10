# 错误类型参考

CCR 使用统一的错误类型系统，提供详细的错误信息和用户友好的提示。

## 🎯 错误类型总览

```rust
// src/error.rs
#[derive(Error, Debug)]
pub enum CcrError {
    ConfigError(String),
    ConfigMissing(String),
    ConfigSectionNotFound(String),
    ConfigFieldMissing { section: String, field: String },
    ConfigFormatInvalid(String),
    SettingsError(String),
    SettingsMissing(String),
    FileLockError(String),
    LockTimeout(String),
    JsonError(serde_json::Error),
    TomlError(toml::de::Error),
    IoError(std::io::Error),
    NetworkError(String),
    PermissionDenied(String),
    HistoryError(String),
    ValidationError(String),
    InvalidArgument(String),
    Unknown(String),
}
```

## 📋 错误类型详解

### ConfigError

**代码**: 10  
**描述**: 配置文件相关的一般错误

**示例**:
```rust
CcrError::ConfigError("读取配置文件失败: Permission denied".into())
```

**场景**:
- 读取配置文件失败
- 序列化/反序列化错误
- 配置文件格式问题

---

### ConfigMissing

**代码**: 11  
**描述**: 配置文件不存在  
**致命**: ✅

**示例**:
```rust
CcrError::ConfigMissing("/home/user/.ccs_config.toml".into())
```

**用户消息**:
```
配置文件不存在: /home/user/.ccs_config.toml
建议: 请运行安装脚本创建配置文件，或检查配置文件路径是否正确
```

**解决方案**:
```bash
# 创建配置文件
vim ~/.ccs_config.toml

# 或从 CCS 安装
cd ccs
./scripts/install/install.sh
```

---

### ConfigSectionNotFound

**代码**: 12  
**描述**: 指定的配置节不存在

**示例**:
```rust
CcrError::ConfigSectionNotFound("nonexistent".into())
```

**用户消息**:
```
配置节 'nonexistent' 不存在
建议: 运行 'ccr list' 查看可用配置，或编辑 ~/.ccs_config.toml 添加新配置
```

**解决方案**:
```bash
# 查看可用配置
ccr list

# 添加配置节
vim ~/.ccs_config.toml
```

---

### ConfigFieldMissing

**代码**: 13  
**描述**: 配置缺少必填字段

**示例**:
```rust
CcrError::ConfigFieldMissing {
    section: "anthropic".into(),
    field: "base_url".into()
}
```

**用户消息**:
```
配置 'anthropic' 缺少必填字段: base_url
建议: 请在 ~/.ccs_config.toml 中为配置 'anthropic' 添加 'base_url' 字段
```

---

### ConfigFormatInvalid

**代码**: 14  
**描述**: 配置文件格式无效

**示例**:
```rust
CcrError::ConfigFormatInvalid("TOML 解析失败: unexpected character".into())
```

**常见原因**:
- 缺少引号
- 语法错误
- 编码问题

---

### SettingsError

**代码**: 20  
**描述**: 设置文件相关的一般错误

**示例**:
```rust
CcrError::SettingsError("读取设置文件失败: IO error".into())
```

---

### SettingsMissing

**代码**: 21  
**描述**: 设置文件不存在  
**致命**: ✅

**示例**:
```rust
CcrError::SettingsMissing("/home/user/.claude/settings.json".into())
```

**用户消息**:
```
Claude Code 设置文件不存在: /home/user/.claude/settings.json
建议: 请确保已安装 Claude Code，或检查 ~/.claude 目录是否存在
```

**解决方案**:
```bash
# 创建目录
mkdir -p ~/.claude

# 运行 ccr 初始化
ccr switch anthropic
```

---

### FileLockError

**代码**: 30  
**描述**: 文件锁相关错误

**示例**:
```rust
CcrError::FileLockError("无法打开锁文件: Permission denied".into())
```

---

### LockTimeout

**代码**: 31  
**描述**: 获取文件锁超时

**示例**:
```rust
CcrError::LockTimeout("获取文件锁超时 (10s): claude_settings".into())
```

**用户消息**:
```
获取文件锁超时: claude_settings
建议: 可能有其他 ccr 进程正在运行，请稍后重试或检查是否有僵死进程
```

**解决方案**:
```bash
# 检查进程
ps aux | grep ccr

# 清理锁文件
rm -rf ~/.claude/.locks/*
```

---

### JsonError

**代码**: 40  
**描述**: JSON 序列化/反序列化错误

**示例**:
```rust
CcrError::JsonError(serde_json::Error::from(...))
```

---

### TomlError

**代码**: 41  
**描述**: TOML 序列化/反序列化错误

**示例**:
```rust
CcrError::TomlError(toml::de::Error::from(...))
```

---

### IoError

**代码**: 50  
**描述**: IO 操作错误  
**致命**: ✅

**示例**:
```rust
CcrError::IoError(std::io::Error::from(...))
```

---

### PermissionDenied

**代码**: 70  
**描述**: 权限拒绝  
**致命**: ✅

**示例**:
```rust
CcrError::PermissionDenied("/home/user/.claude/settings.json".into())
```

**用户消息**:
```
权限拒绝: /home/user/.claude/settings.json
建议: 请检查文件权限，确保当前用户有读写权限
```

**解决方案**:
```bash
# 修复权限
chmod 600 ~/.claude/settings.json
chmod 644 ~/.ccs_config.toml

# 检查所有者
ls -la ~/.claude/settings.json
```

---

### HistoryError

**代码**: 80  
**描述**: 历史记录错误

**示例**:
```rust
CcrError::HistoryError("解析历史文件失败: invalid JSON".into())
```

---

### ValidationError

**代码**: 90  
**描述**: 配置或设置验证失败

**示例**:
```rust
CcrError::ValidationError("base_url 不能为空".into())
```

---

### InvalidArgument

**代码**: 100  
**描述**: 无效的命令参数

**示例**:
```rust
CcrError::InvalidArgument("limit 必须是正整数".into())
```

---

### Unknown

**代码**: 255  
**描述**: 未知错误

## 📊 错误码分类

| 范围 | 类型 | 说明 |
|------|------|------|
| 10-19 | 配置错误 | 配置文件相关 |
| 20-29 | 设置错误 | 设置文件相关 |
| 30-39 | 文件锁错误 | 并发控制相关 |
| 40-49 | 序列化错误 | JSON/TOML 相关 |
| 50-59 | IO 错误 | 文件操作相关 |
| 60-69 | 网络错误 | 网络请求相关 |
| 70-79 | 权限错误 | 权限和访问相关 |
| 80-89 | 历史错误 | 历史记录相关 |
| 90-99 | 验证错误 | 配置验证相关 |
| 100+ | 其他错误 | 参数、未知错误等 |

## 🔧 错误处理方法

### exit_code()

获取错误退出码

```rust
pub fn exit_code(&self) -> i32
```

**示例**:
```rust
let error = CcrError::ConfigMissing("...".into());
let code = error.exit_code();  // 11
std::process::exit(code);
```

---

### is_fatal()

判断错误是否致命

```rust
pub fn is_fatal(&self) -> bool
```

**示例**:
```rust
let error = CcrError::ConfigMissing("...".into());
if error.is_fatal() {
    eprintln!("致命错误!");
    std::process::exit(error.exit_code());
}
```

**致命错误**:
- `ConfigMissing`
- `SettingsMissing`
- `PermissionDenied`
- `IoError`

---

### user_message()

生成用户友好的错误消息

```rust
pub fn user_message(&self) -> String
```

**示例**:
```rust
let error = CcrError::ConfigSectionNotFound("test".into());
println!("{}", error.user_message());
```

**输出**:
```
配置节 'test' 不存在
建议: 运行 'ccr list' 查看可用配置，或编辑 ~/.ccs_config.toml 添加新配置
```

## 💡 使用示例

### 错误传播

```rust
pub fn load_config() -> Result<CcsConfig> {
    let manager = ConfigManager::default()?;
    let config = manager.load()?;  // ← 错误传播
    Ok(config)
}
```

### 错误转换

```rust
use crate::error::CcrError;

pub fn read_file(path: &Path) -> Result<String> {
    fs::read_to_string(path)
        .map_err(|e| CcrError::ConfigError(
            format!("读取文件失败: {}", e)
        ))
}
```

### 错误处理

```rust
fn main() {
    let result = run_command();
    
    if let Err(e) = result {
        // 显示错误
        ColorOutput::error(&e.user_message());
        
        // 致命错误提示
        if e.is_fatal() {
            ColorOutput::error("这是一个致命错误，程序无法继续");
        }
        
        // 退出
        std::process::exit(e.exit_code());
    }
}
```

## 🧪 测试错误

### 测试错误码

```rust
#[test]
fn test_error_codes() {
    assert_eq!(CcrError::ConfigError("test".into()).exit_code(), 10);
    assert_eq!(CcrError::ConfigMissing("test".into()).exit_code(), 11);
    assert_eq!(CcrError::SettingsError("test".into()).exit_code(), 20);
}
```

### 测试错误消息

```rust
#[test]
fn test_user_message() {
    let err = CcrError::ConfigSectionNotFound("test".into());
    let msg = err.user_message();
    
    assert!(msg.contains("test"));
    assert!(msg.contains("建议"));
}
```

## 🔗 相关文档

- [API 概览](/api/)
- [故障排除](/installation/troubleshooting)
- [错误处理架构](/architecture/error-handling)

