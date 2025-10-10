# 设计决策

本文档记录了 CCR 开发过程中的关键技术决策、权衡考虑和选择理由。

## 🎯 核心决策

### 1. 为什么选择 Rust？

**决策**: 使用 Rust 而非继续使用 Shell 脚本

**理由**:

✅ **类型安全**: 编译期捕获大部分错误
```rust
// 编译器会检查所有可能的错误路径
pub fn load(&self) -> Result<CcsConfig> {
    // 必须处理所有错误情况
}
```

✅ **内存安全**: 无需手动管理内存
```rust
// 自动管理资源，无内存泄漏
impl Drop for FileLock {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}
```

✅ **并发安全**: 编译器保证线程安全
```rust
// 编译器会检查并发访问
// 使用 Arc<Mutex<T>> 或文件锁保证安全
```

✅ **性能优势**: 
- 启动时间: &lt;10ms (Shell: ~50ms)
- 内存占用: ~2MB (Shell: ~10MB)
- 配置切换: &lt;20ms (Shell: ~50ms)

✅ **独立部署**: 
- 编译后的二进制无运行时依赖
- 不依赖特定 Shell 版本
- 跨平台一致性

**权衡**:

❌ **构建复杂度**: 需要 Rust 工具链  
❌ **二进制大小**: ~2MB (Shell: ~50KB)  
❌ **开发速度**: 编译时间较长  

**结论**: 对于需要高可靠性和性能的工具，Rust 是最佳选择。

---

### 2. 为什么直接写入 settings.json？

**决策**: 直接操作 `~/.claude/settings.json` 而非设置环境变量

**CCS 方式 (Shell)**:
```bash
# 设置环境变量
export ANTHROPIC_BASE_URL="https://api.anthropic.com"
export ANTHROPIC_AUTH_TOKEN="sk-ant-..."

# 问题：
# 1. 需要重启 shell 才能生效
# 2. 子进程会继承，可能导致冲突
# 3. 不同 shell 会话配置不一致
```

**CCR 方式 (Rust)**:
```rust
// 直接写入 settings.json
settings_manager.save_atomic(&settings)?;

// 优势：
// 1. 配置立即生效（Claude Code 直接读取）
// 2. 所有进程使用同一配置
// 3. 重启后自动加载
```

**实现细节**:
```rust
#[derive(Serialize, Deserialize)]
pub struct ClaudeSettings {
    pub env: HashMap<String, String>,
    
    #[serde(flatten)]  // ← 关键设计
    pub other: HashMap<String, Value>,
}
```

使用 `#[serde(flatten)]` 确保：
- 只修改 `env` 对象
- 保留所有其他设置
- 用户自定义配置不受影响

**结论**: 直接写入提供更好的用户体验和更高的可靠性。

---

### 3. 为什么使用文件锁？

**决策**: 使用 `fs4` 提供的文件锁而非进程锁或数据库锁

**问题场景**:
```bash
# 终端 1
ccr switch anthropic  # 正在执行

# 终端 2（同时）
ccr switch anyrouter  # 可能导致冲突
```

**解决方案**:
```rust
// 获取文件锁
let _lock = lock_manager.lock_settings(Duration::from_secs(10))?;

// 执行受保护操作
settings_manager.save_atomic(&settings)?;

// 锁自动释放（Drop）
```

**实现选择**:

| 方案 | 优点 | 缺点 | 选择 |
|------|------|------|------|
| **文件锁** | 简单、跨进程、系统级 | 需要清理死锁 | ✅ 选择 |
| 进程锁 | 轻量级 | 仅同进程 | ❌ |
| 数据库锁 | 功能强大 | 引入重依赖 | ❌ |
| 互斥锁 | 性能最好 | 不支持跨进程 | ❌ |

**超时保护**:
```rust
// 超时时间选择：10 秒
// - 足够完成操作（实际 <100ms）
// - 防止长时间等待
// - 快速失败反馈
let _lock = lock_manager.lock_settings(Duration::from_secs(10))?;
```

**结论**: 文件锁是配置管理场景的最佳选择。

---

### 4. 为什么使用原子操作？

**决策**: 使用 `NamedTempFile` + `persist()` 实现原子写入

**问题**:
```rust
// ❌ 危险：直接写入
fs::write(&settings_path, content)?;
// 如果在写入过程中崩溃，文件可能损坏
```

**解决方案**:
```rust
// ✅ 安全：原子写入
let temp_file = NamedTempFile::new_in(parent_dir)?;
fs::write(temp_file.path(), content)?;
temp_file.persist(&settings_path)?;  // ← 原子操作
```

**原子性保证**:
```
rename("temp_file", "settings.json")  // 系统调用
- 要么完全成功（新文件替换旧文件）
- 要么完全失败（旧文件保持不变）
- 不存在"部分写入"的情况
```

**崩溃场景测试**:
```rust
// 场景 1: 写入临时文件时崩溃
fs::write(temp_file.path(), content)?;  // ← 崩溃
// 结果：临时文件可能损坏，但原文件完好

// 场景 2: persist 前崩溃
temp_file.persist(&settings_path)?;  // ← 崩溃前
// 结果：原文件完好

// 场景 3: persist 期间（极小概率）
// 结果：系统保证原子性，不会损坏
```

**结论**: 原子操作是数据完整性的关键保证。

---

### 5. 为什么记录操作历史？

**决策**: 实现完整的审计追踪系统

**动机**:
1. **问题追溯**: 出问题时可以查看历史
2. **安全审计**: 谁在何时做了什么
3. **统计分析**: 使用模式分析
4. **合规要求**: 企业环境的审计需求

**实现**:
```rust
pub struct HistoryEntry {
    pub id: String,                    // UUID 唯一标识
    pub timestamp: DateTime<Local>,    // 精确时间
    pub actor: String,                 // 操作者（whoami）
    pub operation: OperationType,      // 操作类型
    pub env_changes: Vec<EnvChange>,   // 变更明细
    pub result: OperationResult,       // 执行结果
    pub notes: Option<String>,         // 备注
}
```

**敏感信息保护**:
```rust
fn mask_if_sensitive(var_name: &str, value: &str) -> String {
    if var_name.contains("TOKEN") || 
       var_name.contains("KEY") || 
       var_name.contains("SECRET") {
        ColorOutput::mask_sensitive(value)  // sk-a...cdef
    } else {
        value.to_string()
    }
}
```

**存储格式**:
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2025-01-10T14:30:22+08:00",
  "actor": "user",
  "operation": "Switch",
  "env_changes": [
    {
      "var_name": "ANTHROPIC_AUTH_TOKEN",
      "old_value": "sk-a...here",
      "new_value": "your...here"
    }
  ]
}
```

**结论**: 审计追踪是企业级工具的必备特性。

---

### 6. 为什么使用 TOML 而非 JSON/YAML？

**决策**: 使用 TOML 作为配置文件格式

**对比分析**:

| 格式 | 优点 | 缺点 | 评分 |
|------|------|------|------|
| **TOML** | 人类可读、注释支持、简洁 | 嵌套稍复杂 | ⭐⭐⭐⭐⭐ |
| JSON | 广泛支持、解析快 | 无注释、不易编辑 | ⭐⭐⭐ |
| YAML | 简洁、强大 | 缩进敏感、陷阱多 | ⭐⭐ |
| INI | 极简单 | 功能有限 | ⭐⭐ |

**TOML 示例**:
```toml
# 全局设置
default_config = "anthropic"
current_config = "anthropic"

# 配置节 1
[anthropic]
description = "Anthropic 官方 API"
base_url = "https://api.anthropic.com"
auth_token = "sk-ant-your-api-key"
```

**与 CCS 兼容**: CCS 使用 TOML，CCR 保持一致以确保兼容性。

**结论**: TOML 是配置文件的最佳选择。

---

### 7. 为什么使用 Clap 做 CLI？

**决策**: 使用 `clap` crate（derive 模式）

**代码示例**:
```rust
#[derive(Parser)]
#[command(name = "ccr")]
#[command(about = "Claude Code Configuration Switcher", version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    
    config_name: Option<String>,  // 简写形式支持
}

#[derive(Subcommand)]
enum Commands {
    /// 列出所有配置
    #[command(alias = "ls")]
    List,
    
    /// 切换配置
    Switch {
        /// 配置名称
        config_name: String,
    },
}
```

**优势**:
- ✅ 自动生成帮助信息
- ✅ 自动参数验证
- ✅ 支持子命令和别名
- ✅ 零运行时成本（编译期生成）

**对比其他方案**:

| Crate | 优点 | 缺点 |
|-------|------|------|
| **clap** | 功能完整、文档好 | 编译时间长 |
| structopt | 已弃用 | - |
| argh | 轻量级 | 功能有限 |
| 手动解析 | 灵活 | 维护成本高 |

**结论**: Clap 是 Rust CLI 的事实标准。

---

### 8. 为什么使用 tiny_http？

**决策**: 使用 `tiny_http` 而非 actix-web/axum

**对比**:

| 框架 | 二进制大小 | 依赖数量 | 启动时间 | 功能 |
|------|-----------|---------|---------|------|
| **tiny_http** | ~2MB | 少 | &lt;10ms | 基础 ✅ |
| actix-web | ~8MB | 多 | ~50ms | 完整 |
| axum | ~6MB | 中 | ~30ms | 现代 |
| warp | ~7MB | 中 | ~40ms | 类型安全 |

**使用场景**:
```rust
// CCR 的 Web 界面需求：
// 1. 提供静态 HTML 文件
// 2. 少量 RESTful API（~7 个端点）
// 3. 本地使用，无需高并发
// 4. 轻量级，快速启动

// tiny_http 完全满足需求
let server = Server::http("0.0.0.0:8080")?;
for request in server.incoming_requests() {
    self.handle_request(request)?;
}
```

**结论**: 简单场景使用简单工具，避免过度工程。

---

## 🔧 技术选型

### Serde 序列化

**决策**: 使用 Serde 生态系统

**依赖**:
- `serde` - 序列化框架
- `serde_json` - JSON 支持
- `toml` - TOML 支持

**关键特性使用**:

#### 1. #[serde(flatten)]

```rust
#[derive(Serialize, Deserialize)]
pub struct ClaudeSettings {
    pub env: HashMap<String, String>,
    
    #[serde(flatten)]  // ← 扁平化
    pub other: HashMap<String, Value>,
}
```

**效果**:
- 将未知字段收集到 `other`
- 保留用户自定义设置
- 向后兼容

#### 2. #[serde(skip_serializing_if)]

```rust
#[derive(Serialize, Deserialize)]
pub struct ConfigSection {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}
```

**效果**:
- 省略 `None` 值
- 生成更简洁的 TOML
- 减少文件大小

---

### 错误处理策略

**决策**: 使用 `thiserror` + 自定义 `Result` 类型

```rust
// 自定义错误类型
#[derive(Error, Debug)]
pub enum CcrError {
    #[error("配置文件错误: {0}")]
    ConfigError(String),
    
    #[error("配置文件不存在: {0}")]
    ConfigMissing(String),
    
    // ...13 种错误类型
}

// 自定义 Result 类型
pub type Result<T> = std::result::Result<T, CcrError>;

// 使用
pub fn load(&self) -> Result<CcsConfig> {
    // 简洁的错误处理
}
```

**对比其他方案**:

| 方案 | 优点 | 缺点 | 选择 |
|------|------|------|------|
| **thiserror** | 简洁、类型安全 | - | ✅ |
| anyhow | 灵活 | 失去类型信息 | ❌ |
| failure | 功能丰富 | 已废弃 | ❌ |
| `Box<dyn Error>` | 标准库 | 不够具体 | ❌ |

**错误码设计**:
```rust
pub fn exit_code(&self) -> i32 {
    match self {
        CcrError::ConfigError(_) => 10,      // 配置错误 10-19
        CcrError::SettingsError(_) => 20,    // 设置错误 20-29
        CcrError::FileLockError(_) => 30,    // 锁错误 30-39
        CcrError::JsonError(_) => 40,        // JSON错误 40-49
        CcrError::IoError(_) => 50,          // IO错误 50-59
        CcrError::ValidationError(_) => 90,  // 验证错误 90-99
        // ...分类清晰
    }
}
```

---

### 日志系统

**决策**: 使用 `colored` + 自定义 `ColorOutput`

```rust
pub struct ColorOutput;

impl ColorOutput {
    pub fn success(msg: &str) {
        println!("{} {}", "✓".green().bold(), msg.green());
    }
    
    pub fn error(msg: &str) {
        eprintln!("{} {}", "✗".red().bold(), msg.red());
    }
}
```

**为什么不用现成的日志框架？**

| 框架 | 适用场景 | CCR 需求 |
|------|---------|---------|
| `log` + `env_logger` | 库和长期运行服务 | CLI 工具 |
| `tracing` | 分布式追踪 | 简单输出 |
| `colored` | 彩色输出 | ✅ 匹配 |

**结论**: CLI 工具需要简单直观的输出，不需要复杂的日志系统。

---

## 🏗️ 架构模式

### 分层架构

**决策**: 采用三层架构

```
CLI Layer          # 用户交互
    ↓
Business Layer     # 业务逻辑
    ↓
Infrastructure     # 基础设施
```

**优势**:
- 清晰的职责划分
- 易于测试（可以单独测试每层）
- 易于扩展（添加新功能不影响其他层）

**示例**:
```rust
// CLI 层
fn main() {
    let result = commands::switch_command("anyrouter");
    handle_result(result);
}

// 业务层
pub fn switch_command(config_name: &str) -> Result<()> {
    let manager = ConfigManager::default()?;
    manager.switch_config(config_name)?;
    Ok(())
}

// 基础设施层
pub fn save_atomic(&self, data: &Data) -> Result<()> {
    let _lock = self.lock_manager.lock()?;
    // 原子写入
    Ok(())
}
```

---

### 依赖注入

**决策**: 使用依赖注入而非全局状态

```rust
// ❌ 避免：全局状态
lazy_static! {
    static ref CONFIG: Mutex<Config> = Mutex::new(Config::new());
}

// ✅ 推荐：依赖注入
pub struct SettingsManager {
    lock_manager: LockManager,  // ← 依赖注入
}

impl SettingsManager {
    pub fn new(lock_manager: LockManager) -> Self {
        Self { lock_manager }
    }
}
```

**优势**:
- 易于测试（可以注入 mock）
- 无全局状态竞争
- 更清晰的依赖关系

---

## 🎨 用户体验设计

### 渐进式披露

**决策**: 默认简洁输出，可选详细信息

```bash
# 简洁输出
$ ccr list
可用配置列表
────────────────────────
▶ anthropic - Anthropic 官方 API
  anyrouter - AnyRouter 代理

# 详细输出（当前配置）
▶ anthropic - Anthropic 官方 API
    Base URL: https://api.anthropic.com
    Token: sk-a...here
    Model: claude-sonnet-4-5-20250929
    状态: ✓ 配置完整
```

### 彩色输出

**决策**: 使用颜色区分不同类型的消息

```rust
✓ 成功消息 (绿色)
ℹ 信息消息 (蓝色)
⚠ 警告消息 (黄色)
✗ 错误消息 (红色)
▶ 步骤消息 (青色)
```

**可访问性考虑**:
- 不仅依赖颜色（同时使用符号）
- 支持 NO_COLOR 环境变量
- 适配不同终端

---

## 🔐 安全考虑

### 敏感信息掩码

**决策**: 自动掩码敏感信息

```rust
pub fn mask_sensitive(value: &str) -> String {
    if value.len() <= 10 {
        "*".repeat(value.len())
    } else {
        format!("{}...{}", &value[..4], &value[value.len() - 4..])
    }
}

// sk-ant-1234567890abcdef → sk-a...cdef
```

**应用场景**:
- CLI 输出
- 历史记录
- Web 界面
- 日志文件

### 文件权限

**决策**: 严格的文件权限控制

```bash
# 配置文件
chmod 600 ~/.claude/settings.json  # 仅所有者读写

# 锁文件
chmod 600 ~/.claude/.locks/*        # 仅所有者读写
```

---

## 🔗 相关文档

- [整体架构](/architecture/)
- [核心模块](/architecture/modules)
- [数据流程](/architecture/data-flow)
- [与 CCS 对比](/architecture/ccs-comparison)

