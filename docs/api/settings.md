# 设置管理 API

`settings` 模块是 CCR 的核心，负责直接操作 `~/.claude/settings.json` 文件。

## 📦 模块概览

```rust
// src/settings.rs
pub struct ClaudeSettings { ... }
pub struct SettingsManager { ... }
```

## 🔧 SettingsManager

设置管理器，提供原子操作和备份功能。

### 创建管理器

```rust
use ccr::settings::SettingsManager;

// 使用默认路径
let manager = SettingsManager::default()?;
// settings_path: ~/.claude/settings.json
// backup_dir: ~/.claude/backups/

// 使用自定义路径
let manager = SettingsManager::new(
    settings_path,
    backup_dir,
    lock_manager
);
```

### 主要方法

#### load()

加载设置文件

```rust
pub fn load(&self) -> Result<ClaudeSettings>
```

**示例**:
```rust
let settings = manager.load()?;

for (key, value) in &settings.env {
    println!("{} = {}", key, value);
}
```

**错误**:
- `CcrError::SettingsMissing` - 文件不存在
- `CcrError::SettingsError` - JSON 格式错误

---

#### save_atomic()

原子保存设置文件

```rust
pub fn save_atomic(&self, settings: &ClaudeSettings) -> Result<()>
```

**示例**:
```rust
let mut settings = manager.load()?;
settings.env.insert("ANTHROPIC_BASE_URL".into(), "https://api.anthropic.com".into());
manager.save_atomic(&settings)?;
```

**实现细节**:
```rust
pub fn save_atomic(&self, settings: &ClaudeSettings) -> Result<()> {
    // 1. 获取文件锁（10 秒超时）
    let _lock = self.lock_manager.lock_settings(Duration::from_secs(10))?;
    
    // 2. 序列化为 JSON
    let content = serde_json::to_string_pretty(settings)?;
    
    // 3. 写入临时文件
    let temp_file = NamedTempFile::new_in(parent_dir)?;
    fs::write(temp_file.path(), content)?;
    
    // 4. 原子替换
    temp_file.persist(&self.settings_path)?;
    
    Ok(())
}
```

**保证**:
- ✅ 原子操作（要么成功要么失败，无中间状态）
- ✅ 文件锁保护（防止并发写入）
- ✅ 崩溃安全（临时文件独立于原文件）

---

#### backup()

备份设置文件

```rust
pub fn backup(&self, config_name: Option<&str>) -> Result<PathBuf>
```

**示例**:
```rust
// 带配置名称
let backup_path = manager.backup(Some("anyrouter"))?;
// 生成: settings.anyrouter.20250110_143022.json.bak

// 不带配置名称
let backup_path = manager.backup(None)?;
// 生成: settings.20250110_143022.json.bak
```

**备份位置**:
```
~/.claude/backups/
```

---

#### restore()

从备份恢复

```rust
pub fn restore<P: AsRef<Path>>(&self, backup_path: P) -> Result<()>
```

**示例**:
```rust
manager.restore("~/.claude/backups/settings.anyrouter.20250110_143022.json.bak")?;
```

**流程**:
1. 验证备份文件格式
2. 备份当前设置（前缀 `pre_restore`）
3. 获取文件锁
4. 复制备份文件到 settings.json

---

#### list_backups()

列出所有备份文件

```rust
pub fn list_backups(&self) -> Result<Vec<PathBuf>>
```

**示例**:
```rust
let backups = manager.list_backups()?;

for backup in backups {
    println!("备份: {}", backup.display());
}
```

**排序**: 按修改时间降序（最新的在前）

---

#### settings_path()

获取设置文件路径

```rust
pub fn settings_path(&self) -> &Path
```

**示例**:
```rust
let path = manager.settings_path();
println!("设置文件: {}", path.display());
```

## 📄 ClaudeSettings

Claude Code 设置的数据结构

### 结构定义

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeSettings {
    #[serde(default)]
    pub env: HashMap<String, String>,
    
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}
```

**关键设计**:
- `env` - 环境变量 HashMap
- `other` - 其他所有设置（使用 `#[serde(flatten)]`）
- 修改 `env` 时，`other` 保持不变

### 方法

#### new()

创建新设置

```rust
pub fn new() -> Self
```

**示例**:
```rust
let settings = ClaudeSettings::new();
```

---

#### clear_anthropic_vars()

清空所有 ANTHROPIC_* 环境变量

```rust
pub fn clear_anthropic_vars(&mut self)
```

**示例**:
```rust
settings.clear_anthropic_vars();
// 只删除 ANTHROPIC_ 前缀的变量
// 其他变量保留
```

**实现**:
```rust
self.env.retain(|key, _| !key.starts_with("ANTHROPIC_"));
```

---

#### update_from_config()

从配置节更新环境变量

```rust
pub fn update_from_config(&mut self, section: &ConfigSection)
```

**示例**:
```rust
let section = config.get_section("anthropic")?;
settings.update_from_config(&section);
```

**执行操作**:
1. 清空所有 `ANTHROPIC_*` 变量
2. 设置 `ANTHROPIC_BASE_URL`
3. 设置 `ANTHROPIC_AUTH_TOKEN`
4. 设置 `ANTHROPIC_MODEL`（如果提供）
5. 设置 `ANTHROPIC_SMALL_FAST_MODEL`（如果提供）

---

#### validate()

验证设置完整性

```rust
pub fn validate(&self) -> Result<()>
```

**示例**:
```rust
settings.validate()?;
```

**验证项**:
- ✅ `ANTHROPIC_BASE_URL` 必须存在且非空
- ✅ `ANTHROPIC_AUTH_TOKEN` 必须存在且非空

---

#### anthropic_env_status()

获取环境变量状态

```rust
pub fn anthropic_env_status(&self) -> HashMap<String, Option<String>>
```

**示例**:
```rust
let status = settings.anthropic_env_status();

for (var_name, value) in status {
    match value {
        Some(v) => println!("{} = {}", var_name, v),
        None => println!("{} = (未设置)", var_name),
    }
}
```

**返回**:
```rust
{
    "ANTHROPIC_BASE_URL": Some("https://api.anthropic.com"),
    "ANTHROPIC_AUTH_TOKEN": Some("sk-ant-..."),
    "ANTHROPIC_MODEL": Some("claude-sonnet-4-5-20250929"),
    "ANTHROPIC_SMALL_FAST_MODEL": None,
}
```

## 💡 使用示例

### 完整示例：切换配置

```rust
use ccr::config::ConfigManager;
use ccr::settings::SettingsManager;

fn switch_config(config_name: &str) -> Result<()> {
    // 1. 加载配置
    let config_manager = ConfigManager::default()?;
    let config = config_manager.load()?;
    let target_section = config.get_section(config_name)?;
    
    // 2. 验证配置
    target_section.validate()?;
    
    // 3. 备份当前设置
    let settings_manager = SettingsManager::default()?;
    let backup_path = settings_manager.backup(Some(config_name))?;
    println!("备份到: {}", backup_path.display());
    
    // 4. 更新设置
    let mut settings = settings_manager.load().unwrap_or_default();
    settings.update_from_config(&target_section);
    
    // 5. 原子保存
    settings_manager.save_atomic(&settings)?;
    
    println!("配置切换成功");
    Ok(())
}
```

### 示例：查看设置状态

```rust
use ccr::settings::SettingsManager;

fn show_settings() -> Result<()> {
    let manager = SettingsManager::default()?;
    let settings = manager.load()?;
    
    println!("环境变量:");
    let status = settings.anthropic_env_status();
    
    for (var, value) in status {
        match value {
            Some(v) => println!("  {} = {}", var, v),
            None => println!("  {} = (未设置)", var),
        }
    }
    
    // 验证
    match settings.validate() {
        Ok(_) => println!("\n✓ 设置验证通过"),
        Err(e) => println!("\n✗ 设置验证失败: {}", e),
    }
    
    Ok(())
}
```

## 🧪 测试

### 单元测试示例

```rust
#[test]
fn test_settings_update() {
    let mut settings = ClaudeSettings::new();
    let config = ConfigSection {
        base_url: Some("https://api.test.com".into()),
        auth_token: Some("test-token".into()),
        model: Some("test-model".into()),
        ..Default::default()
    };
    
    settings.update_from_config(&config);
    
    assert_eq!(
        settings.env.get("ANTHROPIC_BASE_URL"),
        Some(&"https://api.test.com".to_string())
    );
    assert_eq!(
        settings.env.get("ANTHROPIC_AUTH_TOKEN"),
        Some(&"test-token".to_string())
    );
}
```

## 🔗 相关 API

- [ConfigManager](/api/config) - 配置管理
- [HistoryManager](/api/history) - 历史记录
- [LockManager](/api/lock) - 文件锁

## 📚 相关文档

- [数据流程](/architecture/data-flow)
- [原子操作](/architecture/atomic-operations)
- [环境变量](/installation/environment)

