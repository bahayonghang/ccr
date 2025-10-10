# API 参考

CCR 提供了清晰的 Rust API，可以作为库使用，也可以通过 Web API 进行集成。

## 📚 模块结构

```
ccr
├── config      # 配置管理模块
├── settings    # 设置管理模块
├── history     # 历史记录模块
├── lock        # 文件锁模块
├── error       # 错误类型模块
├── logging     # 日志输出模块
├── commands    # CLI 命令模块
└── web         # Web 服务器模块
```

## 🔧 核心 API

### ConfigManager

配置管理器，负责读写 `~/.ccs_config.toml`

```rust
pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    // 创建管理器
    pub fn new<P: AsRef<Path>>(config_path: P) -> Self;
    pub fn default() -> Result<Self>;
    
    // 配置操作
    pub fn load(&self) -> Result<CcsConfig>;
    pub fn save(&self, config: &CcsConfig) -> Result<()>;
    pub fn update_current(&self, config_name: &str) -> Result<()>;
    
    // 备份恢复
    pub fn backup(&self) -> Result<PathBuf>;
    pub fn restore<P: AsRef<Path>>(&self, backup_path: P) -> Result<()>;
    
    // 路径访问
    pub fn config_path(&self) -> &Path;
}
```

### CcsConfig

配置文件结构

```rust
pub struct CcsConfig {
    pub default_config: String,
    pub current_config: String,
    pub sections: HashMap<String, ConfigSection>,
}

impl CcsConfig {
    // 创建配置
    pub fn new(default_config: String) -> Self;
    
    // 配置节操作
    pub fn get_section(&self, name: &str) -> Result<&ConfigSection>;
    pub fn get_current_section(&self) -> Result<&ConfigSection>;
    pub fn set_current(&mut self, name: &str) -> Result<()>;
    pub fn set_section(&mut self, name: String, section: ConfigSection);
    pub fn remove_section(&mut self, name: &str) -> Result<ConfigSection>;
    pub fn list_sections(&self) -> Vec<String>;
    
    // 验证
    pub fn validate_all(&self) -> HashMap<String, Result<()>>;
}
```

### ConfigSection

单个配置节

```rust
pub struct ConfigSection {
    pub description: Option<String>,
    pub base_url: Option<String>,
    pub auth_token: Option<String>,
    pub model: Option<String>,
    pub small_fast_model: Option<String>,
}

impl ConfigSection {
    // 验证配置
    pub fn validate(&self) -> Result<()>;
    
    // 检查完整性
    pub fn is_complete(&self) -> bool;
    
    // 获取描述
    pub fn display_description(&self) -> String;
}
```

### SettingsManager

设置管理器，负责 `~/.claude/settings.json`

```rust
pub struct SettingsManager {
    settings_path: PathBuf,
    backup_dir: PathBuf,
    lock_manager: LockManager,
}

impl SettingsManager {
    // 创建管理器
    pub fn new<P, Q>(
        settings_path: P, 
        backup_dir: Q, 
        lock_manager: LockManager
    ) -> Self;
    pub fn default() -> Result<Self>;
    
    // 设置操作
    pub fn load(&self) -> Result<ClaudeSettings>;
    pub fn save_atomic(&self, settings: &ClaudeSettings) -> Result<()>;
    
    // 备份恢复
    pub fn backup(&self, config_name: Option<&str>) -> Result<PathBuf>;
    pub fn restore<P: AsRef<Path>>(&self, backup_path: P) -> Result<()>;
    pub fn list_backups(&self) -> Result<Vec<PathBuf>>;
    
    // 路径访问
    pub fn settings_path(&self) -> &Path;
}
```

### ClaudeSettings

Claude Code 设置结构

```rust
pub struct ClaudeSettings {
    pub env: HashMap<String, String>,
    pub other: HashMap<String, Value>,
}

impl ClaudeSettings {
    // 创建设置
    pub fn new() -> Self;
    
    // 环境变量操作
    pub fn clear_anthropic_vars(&mut self);
    pub fn update_from_config(&mut self, section: &ConfigSection);
    pub fn anthropic_env_status(&self) -> HashMap<String, Option<String>>;
    
    // 验证
    pub fn validate(&self) -> Result<()>;
}
```

### HistoryManager

历史记录管理器

```rust
pub struct HistoryManager {
    history_path: PathBuf,
    lock_manager: LockManager,
}

impl HistoryManager {
    // 创建管理器
    pub fn new<P: AsRef<Path>>(
        history_path: P, 
        lock_manager: LockManager
    ) -> Self;
    pub fn default() -> Result<Self>;
    
    // 历史操作
    pub fn load(&self) -> Result<Vec<HistoryEntry>>;
    pub fn add(&self, entry: HistoryEntry) -> Result<()>;
    
    // 筛选查询
    pub fn filter_by_operation(
        &self, 
        op_type: OperationType
    ) -> Result<Vec<HistoryEntry>>;
    pub fn filter_by_time_range(
        &self,
        start: DateTime<Local>,
        end: DateTime<Local>
    ) -> Result<Vec<HistoryEntry>>;
    pub fn get_recent(&self, limit: usize) -> Result<Vec<HistoryEntry>>;
    
    // 统计
    pub fn stats(&self) -> Result<HistoryStats>;
    
    // 清理
    pub fn cleanup_old(&self, max_age_days: i64) -> Result<usize>;
}
```

### HistoryEntry

历史记录条目

```rust
pub struct HistoryEntry {
    pub id: String,
    pub timestamp: DateTime<Local>,
    pub actor: String,
    pub operation: OperationType,
    pub details: OperationDetails,
    pub env_changes: Vec<EnvChange>,
    pub result: OperationResult,
    pub notes: Option<String>,
}

impl HistoryEntry {
    // 创建条目
    pub fn new(
        operation: OperationType,
        details: OperationDetails,
        result: OperationResult
    ) -> Self;
    
    // 添加变更
    pub fn add_env_change(
        &mut self,
        var_name: String,
        old_value: Option<String>,
        new_value: Option<String>
    );
    
    // 设置备注
    pub fn set_notes(self, notes: String) -> Self;
}
```

### LockManager

文件锁管理器

```rust
pub struct LockManager {
    lock_dir: PathBuf,
}

impl LockManager {
    // 创建管理器
    pub fn new<P: AsRef<Path>>(lock_dir: P) -> Self;
    pub fn default() -> Result<Self>;
    
    // 获取锁
    pub fn lock_config(&self, timeout: Duration) -> Result<FileLock>;
    pub fn lock_settings(&self, timeout: Duration) -> Result<FileLock>;
    pub fn lock_history(&self, timeout: Duration) -> Result<FileLock>;
    pub fn lock_resource(
        &self, 
        resource_name: &str, 
        timeout: Duration
    ) -> Result<FileLock>;
    
    // 清理
    pub fn cleanup_stale_locks(&self, max_age: Duration) -> Result<usize>;
}
```

### CcrError

错误类型

```rust
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

impl CcrError {
    // 错误信息
    pub fn exit_code(&self) -> i32;
    pub fn is_fatal(&self) -> bool;
    pub fn user_message(&self) -> String;
}
```

## 🌐 Web API

CCR 提供 RESTful API 接口，用于 Web 界面和第三方集成。

### 基础信息

- **服务器**: tiny_http
- **默认端口**: 8080
- **内容类型**: application/json
- **编码**: UTF-8

### 端点列表

#### GET /api/configs

获取所有配置列表

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
      }
    ]
  }
}
```

#### POST /api/switch

切换配置

**请求**:
```json
{
  "config_name": "anyrouter"
}
```

**响应**:
```json
{
  "success": true,
  "data": "配置切换成功"
}
```

#### POST /api/config

添加配置

**请求**:
```json
{
  "name": "new-config",
  "description": "新配置",
  "base_url": "https://api.example.com",
  "auth_token": "your-token",
  "model": "model-name",
  "small_fast_model": "small-model"
}
```

**响应**:
```json
{
  "success": true,
  "data": "配置添加成功"
}
```

#### PUT /api/config/{name}

更新配置

**请求**:
```json
{
  "name": "updated-name",
  "description": "更新的配置",
  "base_url": "https://api.example.com",
  "auth_token": "your-token",
  "model": "model-name",
  "small_fast_model": "small-model"
}
```

**响应**:
```json
{
  "success": true,
  "data": "配置更新成功"
}
```

#### DELETE /api/config/{name}

删除配置

**响应**:
```json
{
  "success": true,
  "data": "配置删除成功"
}
```

#### GET /api/history

获取历史记录

**响应**:
```json
{
  "success": true,
  "data": {
    "total": 50,
    "entries": [
      {
        "id": "uuid",
        "timestamp": "2025-01-10T14:30:22+08:00",
        "operation": "切换配置",
        "actor": "user",
        "from_config": "anthropic",
        "to_config": "anyrouter",
        "changes": [
          {
            "key": "ANTHROPIC_BASE_URL",
            "old_value": "api.anthropic.com",
            "new_value": "api.anyrouter.ai/v1"
          }
        ]
      }
    ]
  }
}
```

#### POST /api/validate

验证所有配置

**响应**:
```json
{
  "success": true,
  "data": "验证通过"
}
```

### 错误响应

```json
{
  "success": false,
  "message": "错误描述"
}
```

## 🔗 相关文档

- [配置管理](/api/config)
- [设置管理](/api/settings)
- [历史记录](/api/history)
- [文件锁](/api/lock)
- [错误类型](/api/errors)
- [Web API](/api/web-api)

