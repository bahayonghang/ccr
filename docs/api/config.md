# 配置管理 API

`config` 模块提供了完整的配置文件管理功能，包括加载、保存、验证和备份。

## 📦 模块概览

```rust
// src/config.rs
pub struct ConfigSection { ... }
pub struct CcsConfig { ... }
pub struct ConfigManager { ... }
```

## 🔧 ConfigManager

配置管理器，负责 TOML 配置文件的读写操作。

### 创建管理器

```rust
use ccr::config::ConfigManager;

// 使用默认路径（~/.ccs_config.toml）
let manager = ConfigManager::default()?;

// 使用自定义路径
let manager = ConfigManager::new("/path/to/config.toml");
```

### 主要方法

#### load()

加载配置文件

```rust
pub fn load(&self) -> Result<CcsConfig>
```

**示例**:
```rust
let config = manager.load()?;
println!("当前配置: {}", config.current_config);
```

**错误**:
- `CcrError::ConfigMissing` - 配置文件不存在
- `CcrError::ConfigFormatInvalid` - TOML 格式错误

---

#### save()

保存配置文件

```rust
pub fn save(&self, config: &CcsConfig) -> Result<()>
```

**示例**:
```rust
let mut config = manager.load()?;
config.set_current("anyrouter")?;
manager.save(&config)?;
```

---

#### update_current()

更新当前配置并保存

```rust
pub fn update_current(&self, config_name: &str) -> Result<()>
```

**示例**:
```rust
manager.update_current("anthropic")?;
```

**等价于**:
```rust
let mut config = manager.load()?;
config.set_current("anthropic")?;
manager.save(&config)?;
```

---

#### backup()

备份配置文件

```rust
pub fn backup(&self) -> Result<PathBuf>
```

**示例**:
```rust
let backup_path = manager.backup()?;
println!("备份到: {}", backup_path.display());
```

**备份文件命名**:
```
~/.ccs_config.toml.{timestamp}.bak

示例:
~/.ccs_config.toml.20250110_143022.bak
```

---

#### restore()

从备份恢复配置

```rust
pub fn restore<P: AsRef<Path>>(&self, backup_path: P) -> Result<()>
```

**示例**:
```rust
manager.restore("~/.ccs_config.toml.20250110_143022.bak")?;
```

**注意**:
- 恢复前会先备份当前配置
- 会验证备份文件格式

---

#### config_path()

获取配置文件路径

```rust
pub fn config_path(&self) -> &Path
```

**示例**:
```rust
let path = manager.config_path();
println!("配置文件: {}", path.display());
```

## 📋 CcsConfig

配置文件的数据结构

### 字段

```rust
pub struct CcsConfig {
    pub default_config: String,
    pub current_config: String,
    pub sections: HashMap<String, ConfigSection>,
}
```

### 方法

#### new()

创建新配置

```rust
pub fn new(default_config: String) -> Self
```

**示例**:
```rust
let config = CcsConfig::new("anthropic".into());
```

---

#### get_section()

获取指定配置节

```rust
pub fn get_section(&self, name: &str) -> Result<&ConfigSection>
```

**示例**:
```rust
let section = config.get_section("anthropic")?;
println!("Base URL: {:?}", section.base_url);
```

---

#### get_current_section()

获取当前配置节

```rust
pub fn get_current_section(&self) -> Result<&ConfigSection>
```

**示例**:
```rust
let current = config.get_current_section()?;
```

---

#### set_current()

设置当前配置

```rust
pub fn set_current(&mut self, name: &str) -> Result<()>
```

**示例**:
```rust
config.set_current("anyrouter")?;
```

**错误**:
- `CcrError::ConfigSectionNotFound` - 配置节不存在

---

#### set_section()

添加或更新配置节

```rust
pub fn set_section(&mut self, name: String, section: ConfigSection)
```

**示例**:
```rust
let section = ConfigSection {
    description: Some("新配置".into()),
    base_url: Some("https://api.example.com".into()),
    auth_token: Some("token".into()),
    model: None,
    small_fast_model: None,
};

config.set_section("new-config".into(), section);
```

---

#### remove_section()

删除配置节

```rust
pub fn remove_section(&mut self, name: &str) -> Result<ConfigSection>
```

**示例**:
```rust
let removed = config.remove_section("old-config")?;
println!("已删除: {}", removed.display_description());
```

---

#### list_sections()

列出所有配置节名称

```rust
pub fn list_sections(&self) -> Vec<String>
```

**示例**:
```rust
let sections = config.list_sections();
for name in sections {
    println!("- {}", name);
}
```

**说明**: 返回的列表已按字母顺序排序

---

#### validate_all()

验证所有配置节

```rust
pub fn validate_all(&self) -> HashMap<String, Result<()>>
```

**示例**:
```rust
let results = config.validate_all();

for (name, result) in results {
    match result {
        Ok(_) => println!("✓ {}", name),
        Err(e) => println!("✗ {} - {}", name, e),
    }
}
```

## 📄 ConfigSection

单个配置节的数据结构

### 字段

```rust
pub struct ConfigSection {
    pub description: Option<String>,
    pub base_url: Option<String>,
    pub auth_token: Option<String>,
    pub model: Option<String>,
    pub small_fast_model: Option<String>,
}
```

### 方法

#### validate()

验证配置节的完整性

```rust
pub fn validate(&self) -> Result<()>
```

**示例**:
```rust
section.validate()?;
```

**验证规则**:
- `base_url` 必须存在且非空
- `base_url` 必须以 `http://` 或 `https://` 开头
- `auth_token` 必须存在且非空
- `model` 如果提供则不能为空字符串

---

#### is_complete()

检查配置是否完整

```rust
pub fn is_complete(&self) -> bool
```

**示例**:
```rust
if section.is_complete() {
    println!("配置完整");
} else {
    println!("配置不完整");
}
```

---

#### display_description()

获取配置描述

```rust
pub fn display_description(&self) -> String
```

**示例**:
```rust
let desc = section.display_description();
// 如果有 description: 返回 description
// 否则返回: "(无描述)"
```

## 💡 使用示例

### 完整示例：配置管理

```rust
use ccr::config::{ConfigManager, ConfigSection, CcsConfig};

fn main() -> Result<()> {
    // 1. 创建管理器
    let manager = ConfigManager::default()?;
    
    // 2. 加载配置
    let mut config = manager.load()?;
    
    // 3. 列出所有配置
    for name in config.list_sections() {
        println!("配置: {}", name);
    }
    
    // 4. 获取配置节
    let section = config.get_section("anthropic")?;
    
    // 5. 验证配置
    section.validate()?;
    
    // 6. 添加新配置
    let new_section = ConfigSection {
        description: Some("新配置".into()),
        base_url: Some("https://api.new.com".into()),
        auth_token: Some("new-token".into()),
        model: Some("claude-sonnet-4-5-20250929".into()),
        small_fast_model: None,
    };
    config.set_section("new-config".into(), new_section);
    
    // 7. 保存配置
    manager.save(&config)?;
    
    // 8. 切换配置
    config.set_current("new-config")?;
    manager.save(&config)?;
    
    Ok(())
}
```

### 示例：批量验证

```rust
use ccr::config::ConfigManager;

fn validate_all_configs() -> Result<()> {
    let manager = ConfigManager::default()?;
    let config = manager.load()?;
    
    let results = config.validate_all();
    
    let mut valid = 0;
    let mut invalid = 0;
    
    for (name, result) in results {
        match result {
            Ok(_) => {
                println!("✓ {}", name);
                valid += 1;
            }
            Err(e) => {
                eprintln!("✗ {} - {}", name, e);
                invalid += 1;
            }
        }
    }
    
    println!("\n验证结果: {} 通过, {} 失败", valid, invalid);
    
    if invalid > 0 {
        Err(CcrError::ValidationError(
            format!("{} 个配置验证失败", invalid)
        ))
    } else {
        Ok(())
    }
}
```

## 🧪 测试

### 单元测试示例

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_manager() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("test.toml");
        
        let manager = ConfigManager::new(&config_path);
        
        let mut config = CcsConfig::new("default".into());
        manager.save(&config).unwrap();
        
        let loaded = manager.load().unwrap();
        assert_eq!(loaded.default_config, "default");
    }
}
```

## 🔗 相关 API

- [SettingsManager](/api/settings) - 设置管理
- [HistoryManager](/api/history) - 历史记录
- [LockManager](/api/lock) - 文件锁

## 📚 相关文档

- [配置文件格式](/installation/configuration)
- [数据流程](/architecture/data-flow)
- [API 概览](/api/)

