# 核心模块详解

本章深入介绍 CCR 的核心模块实现细节、API 设计和最佳实践。

## 📦 ConfigManager - 配置管理器

### 模块位置
`src/config.rs` (约 400 行代码)

### 核心数据结构

```rust
/// 配置节 - 表示单个 API 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSection {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_token: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub small_fast_model: Option<String>,
}

/// CCS 配置文件结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CcsConfig {
    pub default_config: String,
    pub current_config: String,
    
    #[serde(flatten)]
    pub sections: HashMap<String, ConfigSection>,
}

/// 配置管理器
pub struct ConfigManager {
    config_path: PathBuf,
}
```

### 主要方法

#### 加载配置
```rust
pub fn load(&self) -> Result<CcsConfig> {
    // 1. 检查文件存在性
    if !self.config_path.exists() {
        return Err(CcrError::ConfigMissing(
            self.config_path.display().to_string()
        ));
    }
    
    // 2. 读取文件内容
    let content = fs::read_to_string(&self.config_path)?;
    
    // 3. 解析 TOML
    let config: CcsConfig = toml::from_str(&content)?;
    
    Ok(config)
}
```

#### 保存配置
```rust
pub fn save(&self, config: &CcsConfig) -> Result<()> {
    // 1. 序列化为 TOML
    let content = toml::to_string_pretty(config)?;
    
    // 2. 写入文件
    fs::write(&self.config_path, content)?;
    
    Ok(())
}
```

#### 配置验证
```rust
impl ConfigSection {
    pub fn validate(&self) -> Result<()> {
        // 检查 base_url
        let base_url = self.base_url.as_ref()
            .ok_or(CcrError::ValidationError("base_url 不能为空"))?;
        
        if !base_url.starts_with("http://") && 
           !base_url.starts_with("https://") {
            return Err(CcrError::ValidationError(
                "base_url 必须以 http:// 或 https:// 开头"
            ));
        }
        
        // 检查 auth_token
        let auth_token = self.auth_token.as_ref()
            .ok_or(CcrError::ValidationError("auth_token 不能为空"))?;
        
        if auth_token.trim().is_empty() {
            return Err(CcrError::ValidationError(
                "auth_token 不能为空"
            ));
        }
        
        Ok(())
    }
}
```

### 使用示例

```rust
// 创建管理器
let manager = ConfigManager::default()?;

// 加载配置
let mut config = manager.load()?;

// 获取配置节
let section = config.get_section("anthropic")?;

// 验证配置
section.validate()?;

// 修改配置
config.set_current("anyrouter")?;

// 保存配置
manager.save(&config)?;
```

## 🔧 SettingsManager - 设置管理器

### 模块位置
`src/settings.rs` (约 450 行代码) ⭐ **核心模块**

### 核心数据结构

```rust
/// Claude Code 设置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeSettings {
    /// 环境变量配置
    #[serde(default)]
    pub env: HashMap<String, String>,
    
    /// 其他设置字段（扁平化存储）
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

/// 设置管理器
pub struct SettingsManager {
    settings_path: PathBuf,
    backup_dir: PathBuf,
    lock_manager: LockManager,
}
```

### 关键特性

#### 1. 直接写入 settings.json
```rust
pub fn save_atomic(&self, settings: &ClaudeSettings) -> Result<()> {
    // 获取文件锁
    let _lock = self.lock_manager
        .lock_settings(Duration::from_secs(10))?;
    
    // 序列化为 JSON
    let content = serde_json::to_string_pretty(settings)?;
    
    // 写入临时文件
    let temp_file = NamedTempFile::new_in(
        self.settings_path.parent().unwrap()
    )?;
    
    fs::write(temp_file.path(), content)?;
    
    // 原子替换
    temp_file.persist(&self.settings_path)?;
    
    Ok(())
}
```

#### 2. 保留其他设置
使用 `#[serde(flatten)]` 确保只修改 `env` 对象，其他设置保持不变：

```rust
// 加载现有设置
let mut settings = manager.load().unwrap_or_default();

// 更新环境变量
settings.update_from_config(&config_section);

// 保存（其他字段自动保留）
manager.save_atomic(&settings)?;
```

#### 3. 自动备份
```rust
pub fn backup(&self, config_name: Option<&str>) -> Result<PathBuf> {
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let backup_filename = if let Some(name) = config_name {
        format!("settings.{}.{}.json.bak", name, timestamp)
    } else {
        format!("settings.{}.json.bak", timestamp)
    };
    
    let backup_path = self.backup_dir.join(backup_filename);
    fs::copy(&self.settings_path, &backup_path)?;
    
    Ok(backup_path)
}
```

### 环境变量管理

```rust
impl ClaudeSettings {
    /// 清空所有 ANTHROPIC_* 环境变量
    pub fn clear_anthropic_vars(&mut self) {
        self.env.retain(|key, _| !key.starts_with("ANTHROPIC_"));
    }
    
    /// 从配置更新环境变量
    pub fn update_from_config(&mut self, section: &ConfigSection) {
        self.clear_anthropic_vars();
        
        if let Some(base_url) = &section.base_url {
            self.env.insert("ANTHROPIC_BASE_URL".into(), base_url.clone());
        }
        
        if let Some(auth_token) = &section.auth_token {
            self.env.insert("ANTHROPIC_AUTH_TOKEN".into(), auth_token.clone());
        }
        
        if let Some(model) = &section.model {
            self.env.insert("ANTHROPIC_MODEL".into(), model.clone());
        }
        
        if let Some(small_model) = &section.small_fast_model {
            self.env.insert("ANTHROPIC_SMALL_FAST_MODEL".into(), small_model.clone());
        }
    }
}
```

## 📜 HistoryManager - 历史记录管理器

### 模块位置
`src/history.rs` (约 490 行代码)

### 核心数据结构

```rust
/// 历史记录条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,                      // UUID
    pub timestamp: DateTime<Local>,      // 时间戳
    pub actor: String,                   // 操作者
    pub operation: OperationType,        // 操作类型
    pub details: OperationDetails,       // 详情
    pub env_changes: Vec<EnvChange>,     // 环境变量变更
    pub result: OperationResult,         // 结果
    pub notes: Option<String>,           // 备注
}

/// 操作类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OperationType {
    Switch,    // 切换配置
    Backup,    // 备份
    Restore,   // 恢复
    Validate,  // 验证
    Update,    // 更新
}

/// 环境变量变更
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvChange {
    pub var_name: String,
    pub old_value: Option<String>,  // 已掩码
    pub new_value: Option<String>,  // 已掩码
}
```

### 敏感信息掩码

```rust
impl HistoryEntry {
    pub fn add_env_change(
        &mut self, 
        var_name: String, 
        old_value: Option<String>, 
        new_value: Option<String>
    ) {
        // 对敏感信息进行掩码处理
        let old_masked = old_value.map(|v| 
            Self::mask_if_sensitive(&var_name, &v)
        );
        let new_masked = new_value.map(|v| 
            Self::mask_if_sensitive(&var_name, &v)
        );
        
        self.env_changes.push(EnvChange {
            var_name,
            old_value: old_masked,
            new_value: new_masked,
        });
    }
    
    fn mask_if_sensitive(var_name: &str, value: &str) -> String {
        if var_name.contains("TOKEN") || 
           var_name.contains("KEY") || 
           var_name.contains("SECRET") {
            ColorOutput::mask_sensitive(value)
        } else {
            value.to_string()
        }
    }
}
```

### 历史操作

```rust
// 添加历史记录
pub fn add(&self, entry: HistoryEntry) -> Result<()> {
    let _lock = self.lock_manager.lock_history(Duration::from_secs(10))?;
    let mut entries = self.load()?;
    entries.push(entry);
    self.save(&entries)?;
    Ok(())
}

// 按操作类型筛选
pub fn filter_by_operation(&self, op_type: OperationType) -> Result<Vec<HistoryEntry>> {
    let entries = self.load()?;
    Ok(entries.into_iter()
        .filter(|e| e.operation == op_type)
        .collect())
}

// 获取最近 N 条
pub fn get_recent(&self, limit: usize) -> Result<Vec<HistoryEntry>> {
    let mut entries = self.load()?;
    entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    entries.truncate(limit);
    Ok(entries)
}
```

## 🔒 LockManager - 文件锁管理器

### 模块位置
`src/lock.rs` (约 270 行代码)

### 核心数据结构

```rust
/// 文件锁
pub struct FileLock {
    file: File,
    lock_path: PathBuf,
}

/// 锁管理器
pub struct LockManager {
    lock_dir: PathBuf,
}
```

### 锁获取机制

```rust
impl FileLock {
    pub fn new<P: AsRef<Path>>(lock_path: P, timeout: Duration) -> Result<Self> {
        let lock_path = lock_path.as_ref().to_path_buf();
        
        // 创建锁文件目录
        if let Some(parent) = lock_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // 打开或创建锁文件
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&lock_path)?;
        
        // 尝试获取锁（带超时）
        let start = Instant::now();
        loop {
            match file.try_lock_exclusive() {
                Ok(_) => return Ok(FileLock { file, lock_path }),
                Err(_) if start.elapsed() < timeout => {
                    std::thread::sleep(Duration::from_millis(100));
                    continue;
                }
                Err(e) => return Err(CcrError::LockTimeout(
                    format!("获取文件锁超时: {:?}", lock_path)
                )),
            }
        }
    }
}

/// 自动释放锁
impl Drop for FileLock {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}
```

### 锁使用示例

```rust
// 方式 1: 使用 LockManager
let lock_manager = LockManager::default()?;
let _lock = lock_manager.lock_settings(Duration::from_secs(10))?;
// 执行受保护的操作
// 锁在 _lock 离开作用域时自动释放

// 方式 2: 直接使用 FileLock
let _lock = FileLock::new("/path/to/lock", Duration::from_secs(10))?;
// 执行受保护的操作
```

## 🎨 ColorOutput - 彩色输出工具

### 模块位置
`src/logging.rs` (约 250 行代码)

### 输出方法

```rust
impl ColorOutput {
    /// 成功消息 (绿色)
    pub fn success(msg: &str) {
        println!("{} {}", "✓".green().bold(), msg.green());
    }
    
    /// 信息消息 (蓝色)
    pub fn info(msg: &str) {
        println!("{} {}", "ℹ".blue().bold(), msg);
    }
    
    /// 警告消息 (黄色)
    pub fn warning(msg: &str) {
        println!("{} {}", "⚠".yellow().bold(), msg.yellow());
    }
    
    /// 错误消息 (红色)
    pub fn error(msg: &str) {
        eprintln!("{} {}", "✗".red().bold(), msg.red());
    }
    
    /// 掩码敏感信息
    pub fn mask_sensitive(value: &str) -> String {
        if value.len() <= 10 {
            "*".repeat(value.len())
        } else {
            format!("{}...{}", 
                &value[..4], 
                &value[value.len() - 4..]
            )
        }
    }
}
```

### 使用示例

```rust
ColorOutput::success("配置切换成功");
ColorOutput::info("当前配置: anthropic");
ColorOutput::warning("配置验证警告");
ColorOutput::error("配置文件不存在");

let masked = ColorOutput::mask_sensitive("sk-ant-1234567890abcdef");
// 输出: sk-a...cdef
```

## ❌ CcrError - 错误处理

### 模块位置
`src/error.rs` (约 210 行代码)

### 错误类型

```rust
#[derive(Error, Debug)]
pub enum CcrError {
    #[error("配置文件错误: {0}")]
    ConfigError(String),
    
    #[error("配置文件不存在: {0}")]
    ConfigMissing(String),
    
    #[error("配置节 '{0}' 不存在")]
    ConfigSectionNotFound(String),
    
    #[error("设置文件错误: {0}")]
    SettingsError(String),
    
    #[error("文件锁错误: {0}")]
    FileLockError(String),
    
    #[error("获取文件锁超时: {0}")]
    LockTimeout(String),
    
    #[error("验证失败: {0}")]
    ValidationError(String),
    
    // ... 更多错误类型
}
```

### 错误码映射

```rust
impl CcrError {
    pub fn exit_code(&self) -> i32 {
        match self {
            CcrError::ConfigError(_) => 10,
            CcrError::ConfigMissing(_) => 11,
            CcrError::SettingsError(_) => 20,
            CcrError::FileLockError(_) => 30,
            CcrError::ValidationError(_) => 90,
            // ...
        }
    }
    
    pub fn is_fatal(&self) -> bool {
        matches!(
            self,
            CcrError::ConfigMissing(_)
                | CcrError::SettingsMissing(_)
                | CcrError::PermissionDenied(_)
        )
    }
}
```

## 🔗 相关文档

- [整体架构](/architecture/)
- [数据流程图](/architecture/data-flow)
- [设计决策](/architecture/design-decisions)

