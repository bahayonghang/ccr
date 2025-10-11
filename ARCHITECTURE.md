# CCR 架构设计文档

## 概述

CCR (Claude Code Configuration Switcher) 采用分层架构设计，清晰分离关注点，提高代码的可维护性和可扩展性。

## 架构层次

```
┌────────────────────────────────────────┐
│   Presentation Layer                   │
│   ├─ CLI (commands/)                   │  ← 命令行界面
│   └─ Web (web/)                        │  ← Web 界面
├────────────────────────────────────────┤
│   Business Logic Layer                 │
│   └─ Services (services/)              │  ← 业务逻辑封装
├────────────────────────────────────────┤
│   Data Access Layer                    │
│   ├─ ConfigManager                     │  ← 配置文件访问
│   ├─ SettingsManager                   │  ← 设置文件访问
│   └─ HistoryManager                    │  ← 历史记录访问
├────────────────────────────────────────┤
│   Infrastructure Layer                 │
│   ├─ Core (core/)                      │  ← 核心抽象
│   ├─ Utils (utils/)                    │  ← 工具函数
│   ├─ Lock                              │  ← 文件锁
│   └─ Error                             │  ← 错误处理
└────────────────────────────────────────┘
```

## 模块详解

### 📁 Presentation Layer（表示层）

#### CLI 子系统 (`commands/`)
负责处理命令行交互和用户界面。

**模块**：
- `list.rs` - 列出所有配置
- `current.rs` - 显示当前配置状态
- `switch.rs` - 切换配置
- `validate.rs` - 验证配置
- `history_cmd.rs` - 显示历史记录
- `init.rs` - 初始化配置文件
- `export.rs` / `import.rs` - 导入导出配置
- `clean.rs` - 清理旧备份
- `update.rs` - 自更新
- `optimize.rs` - 优化配置结构

**职责**：
- 解析命令行参数
- 调用 Service 层执行业务逻辑
- 格式化输出结果
- 处理用户交互

#### Web 子系统 (`web/`)
提供 Web 界面和 RESTful API。

**模块**：
- `server.rs` - HTTP 服务器核心
- `handlers.rs` - 请求处理器
- `models.rs` - API 数据模型
- `routes.rs` - 路由定义

**职责**：
- HTTP 服务器管理
- API 请求路由
- JSON 数据序列化
- 静态文件服务

### 🎯 Business Logic Layer（业务逻辑层）

#### Services (`services/`)
封装核心业务逻辑，协调多个 Manager 的操作。

**ConfigService** (`config_service.rs`)：
- `list_configs()` - 列出所有配置
- `get_current()` - 获取当前配置
- `get_config()` - 获取指定配置
- `add_config()` - 添加新配置
- `update_config()` - 更新配置
- `delete_config()` - 删除配置
- `set_current()` - 设置当前配置
- `validate_all()` - 验证所有配置

**SettingsService** (`settings_service.rs`)：
- `get_current_settings()` - 获取当前设置
- `apply_config()` - 应用配置到设置
- `backup_settings()` - 备份设置
- `restore_settings()` - 恢复设置
- `list_backups()` - 列出所有备份

**HistoryService** (`history_service.rs`)：
- `record_operation()` - 记录操作
- `get_recent()` - 获取最近记录
- `filter_by_type()` - 按类型筛选
- `get_stats()` - 获取统计信息

**BackupService** (`backup_service.rs`)：
- `clean_old_backups()` - 清理旧备份
- `scan_backup_directory()` - 扫描备份目录

**优势**：
- ✅ 业务逻辑集中，易于测试
- ✅ 可被 CLI 和 Web 共享
- ✅ 事务性操作封装（备份+修改+历史记录）
- ✅ 统一错误处理

### 📊 Data Access Layer（数据访问层）

#### Managers
直接管理数据文件的读写。

**ConfigManager** (`config.rs`)：
- 管理 `~/.ccs_config.toml`
- TOML 解析和序列化
- 配置验证

**SettingsManager** (`settings.rs`)：
- 管理 `~/.claude/settings.json`
- 原子性写入（temp file + rename）
- 文件锁保护
- 自动备份

**HistoryManager** (`history.rs`)：
- 管理 `~/.claude/ccr_history.json`
- 操作审计追踪
- 敏感信息掩码

**特点**：
- 🔒 文件锁确保并发安全
- 💾 原子操作防止数据损坏
- 📝 自动备份机制

### 🏗️ Infrastructure Layer（基础设施层）

#### Core (`core/`)
核心抽象和基础设施。

**AtomicWriter** (`atomic_writer.rs`)：
- 原子文件写入
- 临时文件 + rename 模式
- 防止部分写入

**FileManager Trait** (`file_manager.rs`)：
- 统一的文件管理接口
- 泛型设计，支持多种数据类型

#### Utils (`utils/`)
通用工具函数。

**Mask** (`mask.rs`)：
- `mask_sensitive()` - 掩码敏感信息
- `mask_if_sensitive()` - 条件掩码

**Validation** (`validation.rs`)：
- `Validatable` trait - 统一验证接口

#### Lock (`lock.rs`)
文件锁机制。

**FileLock**：
- 跨进程互斥锁
- 超时机制
- RAII 自动释放

**LockManager**：
- 统一锁管理
- `lock_resource()` - 通用资源锁
- `lock_settings()` - 设置文件锁
- `lock_history()` - 历史文件锁

#### Error (`error.rs`)
统一错误处理。

**CcrError**：
- 13 种错误类型
- 唯一退出码（10-99 范围）
- 用户友好消息
- 致命错误标识

**Exit Codes**：
- 10-19: 配置错误
- 20-29: 设置错误
- 30-39: 文件锁错误
- 40-49: 序列化错误
- 50-59: IO 错误
- 80-89: 历史记录错误
- 90-99: 验证错误

## 设计模式

### 1. 分层架构 (Layered Architecture)
严格的层次依赖：
- Presentation → Business Logic → Data Access → Infrastructure
- 每层只依赖其下层，不跨层调用

### 2. 服务模式 (Service Pattern)
业务逻辑封装在 Service 层：
- 单一职责
- 可测试性
- 可复用性

### 3. 管理器模式 (Manager Pattern)
数据访问封装在 Manager 中：
- 文件操作抽象
- 锁管理
- 备份策略

### 4. RAII (Resource Acquisition Is Initialization)
资源自动管理：
- `FileLock` 自动释放
- `NamedTempFile` 自动清理

### 5. 原子操作 (Atomic Operations)
所有文件写入使用原子模式：
```
1. 创建临时文件
2. 写入内容
3. 原子 rename
```

### 6. 策略模式 (Strategy Pattern)
- `ImportMode::Merge` vs `ImportMode::Replace`
- 可扩展的导入策略

## 数据流

### 配置切换流程

```
用户命令
  ↓
switch_command (CLI)
  ↓
ConfigService::get_config()  ← 验证目标配置
  ↓
SettingsService::backup()    ← 备份当前设置
  ↓
SettingsService::apply()     ← 应用新配置
  ↓
SettingsManager::save()      ← 原子写入
  ↓
HistoryService::record()     ← 记录操作
  ↓
响应用户
```

### Web API 流程

```
HTTP Request
  ↓
WebServer::start()
  ↓
Handlers::handle_request()
  ↓
Route Matching
  ↓
Service Layer Call
  ↓
Manager Layer Operation
  ↓
JSON Response
```

## 文件组织

```
src/
├── main.rs              # 程序入口
├── lib.rs               # 库入口（供测试使用）
│
├── commands/            # 命令实现
│   ├── mod.rs
│   ├── list.rs
│   ├── switch.rs
│   └── ...
│
├── services/            # 业务逻辑
│   ├── mod.rs
│   ├── config_service.rs
│   ├── settings_service.rs
│   ├── history_service.rs
│   └── backup_service.rs
│
├── web/                 # Web 界面
│   ├── mod.rs
│   ├── server.rs
│   ├── handlers.rs
│   ├── models.rs
│   └── routes.rs
│
├── core/                # 核心抽象
│   ├── mod.rs
│   ├── atomic_writer.rs
│   └── file_manager.rs
│
├── utils/               # 工具函数
│   ├── mod.rs
│   ├── mask.rs
│   └── validation.rs
│
├── config.rs            # 配置管理
├── settings.rs          # 设置管理
├── history.rs           # 历史记录
├── lock.rs              # 文件锁
├── logging.rs           # 日志输出
└── error.rs             # 错误处理

tests/
└── integration_test.rs  # 集成测试

web/
└── index.html           # Web 界面
```

## 关键设计决策

### 1. 为什么引入 Service 层？

**问题**：
- 业务逻辑分散在 commands 和 web 中
- 代码重复（CLI 和 Web 都要实现相同逻辑）
- 难以测试（需要模拟完整命令流程）

**解决方案**：
- Service 层封装业务逻辑
- CLI 和 Web 都调用同样的 Service
- Service 可独立测试

### 2. 为什么保留 Manager 层？

**问题**：
- 直接在 Service 中操作文件过于底层
- 文件操作需要锁、备份等基础设施

**解决方案**：
- Manager 专注于数据访问
- Service 专注于业务流程
- 清晰的职责分离

### 3. 为什么使用 Trait（Validatable, FileManager）？

**问题**：
- ConfigSection 和 ClaudeSettings 都需要验证
- 验证逻辑重复

**解决方案**：
- Validatable trait 统一验证接口
- FileManager trait 统一文件操作
- 减少代码重复，提高一致性

### 4. 为什么拆分 Web 模块？

**问题**：
- web.rs 753 行，过于臃肿
- 混合了模型、路由、处理器、服务器

**解决方案**：
- models.rs - 数据模型
- routes.rs - 路由定义
- handlers.rs - 请求处理
- server.rs - 服务器核心
- 单一职责，易于维护

## 测试策略

### 单元测试
- 每个模块都有 `#[cfg(test)]` 测试
- 使用 tempfile 创建隔离环境
- 测试边界条件和错误路径

### 集成测试
- `tests/integration_test.rs`
- 测试跨模块的工作流
- 验证端到端功能

### 测试覆盖率
- Lib 测试：35 tests
- Bin 测试：45 tests
- 集成测试：3 tests
- **总计**：83 tests (79 passed, 4 failed due to WSL env)

## 扩展指南

### 添加新的 Service

1. 创建 `src/services/new_service.rs`
2. 定义 Service 结构和方法
3. 在 `services/mod.rs` 中导出
4. 在 lib.rs 中重新导出（如需公开）

示例：
```rust
pub struct NewService {
    manager: Arc<SomeManager>,
}

impl NewService {
    pub fn new(manager: Arc<SomeManager>) -> Self {
        Self { manager }
    }
    
    pub fn default() -> Result<Self> {
        Ok(Self::new(Arc::new(SomeManager::default()?)))
    }
    
    pub fn some_operation(&self) -> Result<Output> {
        // Business logic here
    }
}
```

### 添加新的 Web API

1. 在 `web/models.rs` 添加请求/响应模型
2. 在 `web/routes.rs` 添加路由枚举
3. 在 `web/handlers.rs` 添加处理函数
4. 在 `web/server.rs` 或 `handlers.rs` 的路由匹配中添加路由

### 使用 Service 层

命令实现应优先使用 Service：

```rust
// ✅ 推荐：使用 Service 层
pub fn my_command() -> Result<()> {
    let service = ConfigService::default()?;
    let result = service.list_configs()?;
    
    // 展示结果
    for config in result.configs {
        ColorOutput::info(&config.name);
    }
    
    Ok(())
}

// ❌ 不推荐：直接使用 Manager
pub fn my_command() -> Result<()> {
    let manager = ConfigManager::default()?;
    let config = manager.load()?;
    // ... 手动处理业务逻辑
}
```

## 性能考虑

### 文件锁超时
- 默认 10 秒超时
- 避免死锁
- 可通过 `lock_resource()` 自定义超时

### 原子写入
- 使用 tempfile crate
- 在同一文件系统创建临时文件
- 原子 rename 确保一致性

### 内存使用
- 使用 Arc 共享 Manager 引用
- 避免不必要的克隆
- 延迟加载配置

## 安全考虑

### 敏感信息保护
- API Token 自动掩码显示
- 历史记录中自动掩码
- 导出时可选移除密钥

### 并发安全
- 文件锁保护并发写入
- 原子操作防止数据损坏
- RAII 确保锁释放

### 备份策略
- 每次修改前自动备份
- 时间戳命名避免冲突
- 可手动恢复任意备份

## 未来优化方向

### 短期优化
1. ✅ 完成 Web handlers 的所有 API 端点
2. ✅ 让更多命令使用 Service 层
3. ✅ 添加更多集成测试

### 中期优化
1. 实现 Command trait 统一命令接口
2. 添加配置缓存机制
3. 支持配置热重载

### 长期优化
1. 插件系统支持自定义配置源
2. 配置版本控制
3. 多用户支持

## 参考资料

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Clean Architecture](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
- [The Rust Programming Language Book](https://doc.rust-lang.org/book/)

