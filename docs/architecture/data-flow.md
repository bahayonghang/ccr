# 数据流程

本章详细介绍 CCR 各个操作的完整数据流程，帮助理解系统的工作原理。

## 🔄 配置切换流程

### 完整流程图

```mermaid
sequenceDiagram
    autonumber
    participant User as 用户
    participant CLI as CLI 层
    participant Switch as switch_command
    participant Config as ConfigManager
    participant Settings as SettingsManager
    participant Lock as LockManager
    participant History as HistoryManager
    participant FS as 文件系统

    User->>CLI: ccr switch anyrouter
    CLI->>Switch: 调用 switch_command("anyrouter")
    
    Note over Switch: 步骤 1/5: 读取配置文件
    Switch->>Config: load()
    Config->>FS: 读取 ~/.ccs_config.toml
    FS-->>Config: TOML 内容
    Config->>Config: 解析 TOML
    Config-->>Switch: CcsConfig
    
    Switch->>Config: get_section("anyrouter")
    Config-->>Switch: ConfigSection
    
    Switch->>Config: validate()
    Config-->>Switch: Ok(()) 验证通过
    
    Note over Switch: 步骤 2/5: 备份当前设置
    Switch->>Settings: backup(Some("anyrouter"))
    Settings->>FS: 复制 settings.json
    FS-->>Settings: backup_path
    Switch->>CLI: 显示备份信息
    
    Note over Switch: 步骤 3/5: 更新 Claude Code 设置
    Switch->>Lock: lock_settings(10s)
    Lock->>FS: 创建锁文件
    FS-->>Lock: FileLock
    
    Switch->>Settings: load()
    Settings->>FS: 读取 ~/.claude/settings.json
    FS-->>Settings: JSON 内容
    Settings-->>Switch: old_settings
    
    Switch->>Settings: update_from_config()
    Note over Settings: 清空 ANTHROPIC_*<br/>设置新值
    
    Switch->>Settings: save_atomic()
    Settings->>FS: 创建临时文件
    Settings->>FS: 写入 JSON 内容
    Settings->>FS: 原子替换
    FS-->>Settings: Ok()
    
    Switch->>Lock: 释放锁
    Lock->>FS: 删除锁文件
    
    Note over Switch: 步骤 4/5: 更新配置文件
    Switch->>Config: set_current("anyrouter")
    Config->>Config: 更新 current_config
    Switch->>Config: save()
    Config->>FS: 写入 ~/.ccs_config.toml
    
    Note over Switch: 步骤 5/5: 记录操作历史
    Switch->>History: 创建 HistoryEntry
    Switch->>History: add_env_change()
    Note over History: 敏感信息掩码
    Switch->>History: add(entry)
    History->>Lock: lock_history(10s)
    History->>FS: 追加到 ccr_history.json
    
    Switch-->>CLI: Ok(())
    CLI->>CLI: 显示成功信息
    CLI-->>User: ✓ 配置切换成功
```

### 流程细节

#### 阶段 1: 配置验证

```rust
// 1.1 加载配置文件
let config_manager = ConfigManager::default()?;
let mut config = config_manager.load()?;

// 1.2 获取目标配置节
let target_section = config.get_section(config_name)?;

// 1.3 验证配置完整性
target_section.validate()?;
```

**验证检查项**:
- `base_url` 必须存在且非空
- `base_url` 必须以 `http://` 或 `https://` 开头
- `auth_token` 必须存在且非空
- `model` 如果提供则不能为空字符串

#### 阶段 2: 备份当前设置

```rust
// 2.1 检查设置文件是否存在
if settings_manager.settings_path().exists() {
    // 2.2 执行备份
    let backup_path = settings_manager.backup(Some(config_name))?;
    
    // 2.3 输出备份路径
    ColorOutput::success(&format!("设置已备份: {}", backup_path.display()));
}
```

**备份文件命名**:
```
settings.{config_name}.{timestamp}.json.bak

示例:
settings.anyrouter.20250110_143022.json.bak
```

#### 阶段 3: 更新 Claude Code 设置

```rust
// 3.1 加载现有设置（保留其他字段）
let old_settings = settings_manager.load().ok();
let mut new_settings = old_settings.unwrap_or_default();

// 3.2 更新环境变量
new_settings.update_from_config(&target_section);

// 3.3 原子保存
settings_manager.save_atomic(&new_settings)?;
```

**原子保存细节**:
```rust
pub fn save_atomic(&self, settings: &ClaudeSettings) -> Result<()> {
    // a. 获取文件锁（10 秒超时）
    let _lock = self.lock_manager.lock_settings(Duration::from_secs(10))?;
    
    // b. 序列化为 JSON
    let content = serde_json::to_string_pretty(settings)?;
    
    // c. 写入临时文件
    let temp_file = NamedTempFile::new_in(
        self.settings_path.parent().unwrap()
    )?;
    fs::write(temp_file.path(), content)?;
    
    // d. 原子替换（这一步是原子操作）
    temp_file.persist(&self.settings_path)?;
    
    Ok(())
}
```

#### 阶段 4: 更新配置指针

```rust
// 4.1 更新 current_config 字段
config.set_current(config_name)?;

// 4.2 保存配置文件
config_manager.save(&config)?;
```

#### 阶段 5: 记录历史

```rust
// 5.1 创建历史条目
let mut history_entry = HistoryEntry::new(
    OperationType::Switch,
    OperationDetails {
        from_config: Some(old_config.clone()),
        to_config: Some(config_name.to_string()),
        backup_path: Some(backup_path_str),
        extra: None,
    },
    OperationResult::Success,
);

// 5.2 记录环境变量变更
for (var_name, new_value) in new_env {
    let old_value = old_env.get(&var_name).and_then(|v| v.clone());
    history_entry.add_env_change(var_name, old_value, new_value);
}

// 5.3 保存历史
history_manager.add(history_entry)?;
```

## 📋 列出配置流程

```mermaid
graph TD
    A[list_command] --> B[ConfigManager::default]
    B --> C[加载配置文件]
    C --> D{配置文件存在?}
    
    D -->|是| E[解析 TOML]
    D -->|否| F[返回 ConfigMissing 错误]
    
    E --> G[遍历所有配置节]
    G --> H{是否为当前配置?}
    
    H -->|是| I[显示详细信息]
    H -->|否| J[仅显示名称]
    
    I --> K[validate 配置]
    K --> L{验证通过?}
    
    L -->|是| M[显示 ✓ 配置完整]
    L -->|否| N[显示 ✗ 配置不完整]
    
    M --> O[继续下一个]
    N --> O
    J --> O
    
    O --> P{还有配置?}
    P -->|是| G
    P -->|否| Q[输出统计信息]
    
    style A fill:#e1f5fe
    style E fill:#f3e5f5
    style I fill:#e8f5e8
    style M fill:#c8e6c9
    style N fill:#ffccbc
```

## 🔍 验证流程

### 配置验证流程

```mermaid
flowchart TD
    A[validate_command] --> B[验证配置文件]
    B --> C[加载 CcsConfig]
    C --> D[验证所有配置节]
    
    D --> E{遍历配置节}
    E --> F[检查 base_url]
    F --> G{URL 格式正确?}
    
    G -->|否| H[记录错误]
    G -->|是| I[检查 auth_token]
    
    I --> J{Token 非空?}
    J -->|否| H
    J -->|是| K[检查 model]
    
    K --> L{配置完整?}
    L -->|是| M[✓ 通过]
    L -->|否| H
    
    M --> N{还有配置?}
    H --> N
    N -->|是| E
    N -->|否| O[验证 Claude Code 设置]
    
    O --> P[加载 settings.json]
    P --> Q[检查环境变量]
    
    Q --> R{必需变量存在?}
    R -->|是| S[✓ 验证通过]
    R -->|否| T[✗ 验证失败]
    
    S --> U[生成验证报告]
    T --> U
    
    style M fill:#c8e6c9
    style S fill:#c8e6c9
    style H fill:#ffccbc
    style T fill:#ffccbc
```

## 📜 历史记录流程

### 添加历史记录

```mermaid
sequenceDiagram
    participant Cmd as 命令
    participant History as HistoryManager
    participant Lock as LockManager
    participant FS as 文件系统

    Cmd->>History: 创建 HistoryEntry
    Note over Cmd: UUID, 时间戳, 操作者
    
    Cmd->>History: add_env_change()
    Note over History: 敏感信息掩码
    
    Cmd->>History: add(entry)
    
    History->>Lock: lock_history(10s)
    Lock->>FS: 创建锁文件
    FS-->>Lock: FileLock
    
    History->>FS: 读取现有历史
    FS-->>History: Vec<HistoryEntry>
    
    History->>History: 追加新条目
    
    History->>FS: 序列化为 JSON
    History->>FS: 写入文件
    
    Lock->>FS: 释放锁
    
    History-->>Cmd: Ok(())
```

### 查询历史记录

```rust
// 获取最近 N 条
pub fn get_recent(&self, limit: usize) -> Result<Vec<HistoryEntry>> {
    let mut entries = self.load()?;
    
    // 按时间戳降序排序
    entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    
    // 截断到 limit
    entries.truncate(limit);
    
    Ok(entries)
}

// 按类型筛选
pub fn filter_by_operation(&self, op_type: OperationType) -> Result<Vec<HistoryEntry>> {
    let entries = self.load()?;
    Ok(entries.into_iter()
        .filter(|e| e.operation == op_type)
        .collect())
}
```

## 🔒 文件锁流程

### 锁获取流程

```mermaid
flowchart TD
    A[请求文件锁] --> B[创建锁目录]
    B --> C[打开/创建锁文件]
    C --> D[记录开始时间]
    
    D --> E[尝试获取排他锁]
    E --> F{成功?}
    
    F -->|是| G[返回 FileLock]
    F -->|否| H{超时?}
    
    H -->|否| I[等待 100ms]
    I --> E
    
    H -->|是| J[返回 LockTimeout 错误]
    
    G --> K[执行受保护操作]
    K --> L[FileLock 离开作用域]
    L --> M[Drop trait 自动释放锁]
    
    style G fill:#c8e6c9
    style J fill:#ffccbc
    style M fill:#e8f5e8
```

### 并发场景

```rust
// 场景：两个 CCR 进程同时切换配置

// 进程 A
let _lock_a = lock_manager.lock_settings(Duration::from_secs(10))?;
// 成功获取锁，执行操作
settings_manager.save_atomic(&settings)?;
// 锁自动释放

// 进程 B（几乎同时启动）
let _lock_b = lock_manager.lock_settings(Duration::from_secs(10))?;
// 等待进程 A 释放锁
// 超时前获取到锁，继续执行
```

## 🌐 Web API 流程

### HTTP 请求处理

```mermaid
sequenceDiagram
    participant Browser as 浏览器
    participant Web as Web 服务器
    participant API as API 处理器
    participant Cmd as 命令层
    participant Mgr as 管理器

    Browser->>Web: POST /api/switch
    Note over Browser,Web: {"config_name": "anyrouter"}
    
    Web->>Web: 解析请求
    Web->>API: handle_switch_config()
    
    API->>API: 读取请求体
    API->>API: 解析 JSON
    
    API->>Cmd: switch_command("anyrouter")
    
    Cmd->>Mgr: 执行切换流程
    Note over Mgr: 验证、备份、更新、记录
    
    Mgr-->>Cmd: Ok(())
    Cmd-->>API: Ok(())
    
    API->>API: 构造成功响应
    API-->>Web: ApiResponse::success()
    
    Web-->>Browser: HTTP 200 OK
    Note over Browser,Web: {"success": true, "data": "配置切换成功"}
    
    Browser->>Browser: 显示通知
    Browser->>Web: GET /api/configs
    Web-->>Browser: 刷新配置列表
```

## 📊 环境变量更新流程

### 更新机制

```rust
impl ClaudeSettings {
    pub fn update_from_config(&mut self, section: &ConfigSection) {
        // 1. 清空旧的 ANTHROPIC_* 变量
        self.env.retain(|key, _| !key.starts_with("ANTHROPIC_"));
        
        // 2. 设置新的环境变量
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

### 变更追踪

```mermaid
graph LR
    A[旧配置] --> B{环境变量}
    C[新配置] --> B
    
    B --> D[ANTHROPIC_BASE_URL]
    B --> E[ANTHROPIC_AUTH_TOKEN]
    B --> F[ANTHROPIC_MODEL]
    B --> G[ANTHROPIC_SMALL_FAST_MODEL]
    
    D --> H[EnvChange 记录]
    E --> H
    F --> H
    G --> H
    
    H --> I[敏感信息掩码]
    I --> J[写入历史文件]
    
    style A fill:#ffccbc
    style C fill:#c8e6c9
    style H fill:#e1f5fe
    style I fill:#fff3e0
```

## 💾 原子写入机制

### NamedTempFile + persist()

```rust
// 1. 在目标目录创建临时文件
let temp_file = NamedTempFile::new_in(
    self.settings_path.parent().unwrap()
)?;

// 2. 写入内容到临时文件
fs::write(temp_file.path(), content)?;

// 3. 原子替换（这一步是原子的）
temp_file.persist(&self.settings_path)?;
```

**为什么是原子的？**

```mermaid
graph TB
    A[写入临时文件] -->|完成| B[调用 persist]
    B --> C{rename 系统调用}
    
    C -->|成功| D[原子替换完成]
    C -->|失败| E[临时文件保留]
    
    D --> F[settings.json 更新]
    E --> G[原文件不受影响]
    
    style D fill:#c8e6c9
    style F fill:#c8e6c9
    style G fill:#fff3e0
```

**关键点**:
- `rename()` 系统调用是原子操作
- 要么完全成功，要么完全失败
- 不会出现部分写入的情况
- 崩溃时原文件保持完整

## 🔍 配置解析流程

### TOML 解析

```rust
// 使用 toml crate 反序列化
let content = fs::read_to_string(&self.config_path)?;
let config: CcsConfig = toml::from_str(&content)?;
```

**Serde 自动处理**:
- `#[serde(flatten)]` - 扁平化 sections HashMap
- `#[serde(skip_serializing_if)]` - 跳过空值
- 自动类型转换和验证

### JSON 序列化

```rust
// 设置保留机制
#[derive(Serialize, Deserialize)]
pub struct ClaudeSettings {
    #[serde(default)]
    pub env: HashMap<String, String>,
    
    #[serde(flatten)]  // ← 关键：保留其他字段
    pub other: HashMap<String, Value>,
}
```

**工作原理**:
```json
// 读取时
{
  "env": { "ANTHROPIC_BASE_URL": "..." },
  "otherField": "preserved",
  "anotherField": { ... }
}

// ↓ Serde 反序列化

ClaudeSettings {
  env: { "ANTHROPIC_BASE_URL": "..." },
  other: {
    "otherField": "preserved",
    "anotherField": { ... }
  }
}

// ↓ 修改 env

// ↓ Serde 序列化

// 写入时
{
  "env": { "ANTHROPIC_AUTH_TOKEN": "new" },  // 更新
  "otherField": "preserved",                 // 保留
  "anotherField": { ... }                    // 保留
}
```

## ⚠️ 错误传播流程

### 错误传播链

```rust
// 1. 底层错误
fs::read_to_string(&path)  // io::Error

// 2. 转换为 CcrError
.map_err(|e| CcrError::ConfigError(format!("读取失败: {}", e)))?

// 3. 向上传播
pub fn load(&self) -> Result<CcsConfig> {
    let content = fs::read_to_string(&self.config_path)?;  // ← 传播
    // ...
}

// 4. 命令层捕获
pub fn list_command() -> Result<()> {
    let config = config_manager.load()?;  // ← 传播
    // ...
}

// 5. main 函数处理
fn main() {
    let result = commands::list_command();
    
    if let Err(e) = result {
        ColorOutput::error(&e.user_message());
        std::process::exit(e.exit_code());
    }
}
```

## 🔗 相关文档

- [整体架构](/architecture/)
- [核心模块](/architecture/modules)
- [设计决策](/architecture/design-decisions)
- [文件锁机制](/architecture/locking)

