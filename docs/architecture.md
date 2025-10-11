# CCR 架构文档

## 📐 架构概览

CCR 采用严格的**分层架构**设计，确保代码职责清晰、易于维护和扩展。

```
┌─────────────────────────────────────┐
│   CLI Layer (main.rs + commands/)   │  ← 命令行界面
├─────────────────────────────────────┤
│   Web Layer (web/)                  │  ← Web 界面  
├─────────────────────────────────────┤
│   Service Layer (services/)         │  ← 业务逻辑
├─────────────────────────────────────┤
│   Manager Layer (managers/)         │  ← 数据访问
├─────────────────────────────────────┤
│   Core Layer (core/)                │  ← 基础设施
├─────────────────────────────────────┤
│   Utils Layer (utils/)              │  ← 工具函数
└─────────────────────────────────────┘
```

## 🗂️ 目录结构

```
src/
├── main.rs                          # CLI 入口
├── lib.rs                           # 库入口
│
├── commands/                        # 🎯 CLI Layer
│   ├── mod.rs
│   ├── clean.rs                     # 清理备份命令
│   ├── current.rs                   # 显示当前状态
│   ├── export.rs                    # 导出配置
│   ├── history_cmd.rs               # 查看历史
│   ├── import.rs                    # 导入配置
│   ├── init.rs                      # 初始化配置
│   ├── list.rs                      # 列出配置
│   ├── optimize.rs                  # 优化配置文件
│   ├── switch.rs                    # 切换配置(核心)
│   ├── update.rs                    # 自更新
│   └── validate.rs                  # 验证配置
│
├── web/                             # 🌐 Web Layer
│   ├── mod.rs
│   ├── handlers.rs                  # HTTP 请求处理器
│   ├── models.rs                    # API 数据模型
│   ├── routes.rs                    # 路由定义
│   ├── server.rs                    # Web 服务器
│   └── system_info_cache.rs         # 系统信息缓存
│
├── services/                        # 🎯 Service Layer
│   ├── mod.rs
│   ├── backup_service.rs            # 备份服务
│   ├── config_service.rs            # 配置服务
│   ├── history_service.rs           # 历史服务
│   └── settings_service.rs          # 设置服务
│
├── managers/                        # 📁 Manager Layer
│   ├── mod.rs
│   ├── config.rs                    # ConfigManager - 管理 ~/.ccs_config.toml
│   ├── history.rs                   # HistoryManager - 管理操作历史
│   └── settings.rs                  # SettingsManager - 管理 ~/.claude/settings.json
│
├── core/                            # 🏗️ Core Layer
│   ├── mod.rs
│   ├── atomic_writer.rs             # 原子文件写入
│   ├── error.rs                     # 错误类型定义
│   ├── file_manager.rs              # 文件管理 trait
│   ├── lock.rs                      # 文件锁机制
│   └── logging.rs                   # 日志和彩色输出
│
└── utils/                           # 🛠️ Utils Layer
    ├── mod.rs
    ├── mask.rs                      # 敏感信息掩码
    └── validation.rs                # 验证 trait
```

## 📦 各层职责

### 🎯 CLI Layer (`commands/`)

**职责：** 命令行界面实现

- 解析命令行参数
- 调用 Service 层执行业务逻辑
- 格式化输出结果
- 处理用户交互

**关键原则：**
- 每个命令一个文件
- 只负责 UI 交互，不包含业务逻辑
- 通过 Service 层访问数据

**示例：**
```rust
pub fn switch_command(config_name: &str) -> Result<()> {
    ColorOutput::title(&format!("切换配置: {}", config_name));
    
    // 调用 Service 层
    let config_service = ConfigService::default()?;
    let settings_service = SettingsService::default()?;
    
    // 业务逻辑...
    
    ColorOutput::success("配置切换成功");
    Ok(())
}
```

### 🌐 Web Layer (`web/`)

**职责：** Web 界面和 RESTful API

- HTTP 服务器管理
- 路由分发
- 请求/响应处理
- API 数据模型定义

**架构特点：**
- 基于 `tiny_http` 的轻量级 HTTP 服务器
- RESTful API 设计
- 统一的响应格式 (`ApiResponse<T>`)
- 系统信息缓存优化性能

**API 端点：**
```
GET  /api/configs          # 列出所有配置
POST /api/switch           # 切换配置
POST /api/config           # 添加配置
PUT  /api/config/:name     # 更新配置
DELETE /api/config/:name   # 删除配置
GET  /api/history          # 获取历史记录
POST /api/validate         # 验证配置
POST /api/clean            # 清理备份
GET  /api/settings         # 获取设置
POST /api/export           # 导出配置
POST /api/import           # 导入配置
```

### 🎯 Service Layer (`services/`)

**职责：** 业务逻辑封装

- 协调多个 Manager 的操作
- 实现事务性业务流程
- 数据转换和验证
- 提供统一的业务接口

**服务列表：**

#### ConfigService
- 配置 CRUD 操作
- 配置列表和查询
- 配置验证
- 导入/导出

#### SettingsService
- 应用配置到 settings.json
- 备份和恢复设置
- 列出备份文件

#### HistoryService
- 记录操作历史
- 查询历史记录
- 按类型筛选
- 统计信息

#### BackupService
- 清理旧备份
- 扫描备份目录
- 计算备份大小

**示例：**
```rust
pub struct ConfigService {
    config_manager: Arc<ConfigManager>,
}

impl ConfigService {
    pub fn list_configs(&self) -> Result<ConfigList> {
        let config = self.config_manager.load()?;
        
        let configs: Vec<ConfigInfo> = config
            .list_sections()
            .map(|name| ConfigInfo { /* ... */ })
            .collect();
            
        Ok(ConfigList { configs, /* ... */ })
    }
}
```

### 📁 Manager Layer (`managers/`)

**职责：** 数据访问和持久化

- 文件读写操作
- 数据序列化/反序列化
- 数据结构管理
- 原子性保证

**Manager 列表：**

#### ConfigManager (`config.rs`)
- 管理 `~/.ccs_config.toml`
- 解析 TOML 配置
- 配置节增删改查
- 配置排序优化

#### SettingsManager (`settings.rs`)
- 管理 `~/.claude/settings.json`
- 环境变量更新
- 自动备份机制
- 原子性写入

#### HistoryManager (`history.rs`)
- 管理 `~/.claude/ccr_history.json`
- 历史记录持久化
- 查询和筛选
- 统计计算

**关键原则：**
- 使用文件锁保证并发安全
- 原子写入防止数据损坏
- 保留未知字段（向后兼容）

### 🏗️ Core Layer (`core/`)

**职责：** 基础设施和通用抽象

- 错误类型定义
- 文件锁机制
- 日志系统
- 原子文件操作
- 通用 trait 定义

**核心模块：**

#### error.rs
- `CcrError` 枚举定义
- 13 种错误类型
- 错误码映射
- 用户友好的错误消息

#### lock.rs
- `LockManager` - 文件锁管理器
- `FileLock` - RAII 风格锁
- 超时保护
- 跨平台支持

#### logging.rs
- `ColorOutput` - 彩色输出工具
- 日志初始化
- 统一的输出格式

#### atomic_writer.rs
- `AtomicWriter` - 原子文件写入
- 临时文件 + 原子重命名
- 防止数据损坏

### 🛠️ Utils Layer (`utils/`)

**职责：** 通用工具函数

- 敏感信息掩码
- 验证 trait
- 辅助函数

## 🔄 数据流示例

### 配置切换流程

```
用户命令
   ↓
main.rs (解析参数)
   ↓
switch_command() [commands/switch.rs]
   ↓
ConfigService::get_current() [services/config_service.rs]
   ↓
ConfigManager::load() [managers/config.rs]
   ↓
读取 ~/.ccs_config.toml
   ↓
返回 CcsConfig
   ↓
SettingsService::apply_config()
   ↓
SettingsManager::save_atomic()
   ↓
1. 获取文件锁
2. 备份当前设置
3. 原子写入新设置
4. 释放锁
   ↓
HistoryService::record_operation()
   ↓
HistoryManager::add()
   ↓
写入 ~/.claude/ccr_history.json
   ↓
返回成功
```

## 🔐 关键设计模式

### 1. Repository 模式
Manager 层实现了 Repository 模式，封装数据访问逻辑：
```rust
pub trait FileManager<T> {
    fn load(&self) -> Result<T>;
    fn save(&self, data: &T) -> Result<()>;
    fn path(&self) -> &Path;
}
```

### 2. Service 模式
Service 层协调多个 Manager，实现业务流程：
```rust
pub struct ConfigService {
    config_manager: Arc<ConfigManager>,
}

impl ConfigService {
    pub fn switch_config(&self, name: &str) -> Result<()> {
        // 协调多个操作
        let config = self.config_manager.load()?;
        let section = config.get_section(name)?;
        // 验证、备份、切换、记录历史...
        Ok(())
    }
}
```

### 3. RAII 模式
使用 RAII 管理资源（文件锁）：
```rust
let _lock = lock_manager.lock_settings(Duration::from_secs(10))?;
// 锁会在作用域结束时自动释放
```

### 4. Builder 模式
配置构建使用 Builder 模式：
```rust
let config = CcsConfig {
    default_config: "anthropic".into(),
    current_config: "anthropic".into(),
    sections: IndexMap::new(),
};
```

## 📊 依赖关系

```
commands/  ──→  services/  ──→  managers/  ──→  core/
   │                                              ↑
   └──────────────────────────────────────────────┘
   
web/  ──→  services/  ──→  managers/  ──→  core/

utils/  ←── (所有层都可以使用)
```

**依赖原则：**
- 上层可以依赖下层
- 下层不能依赖上层
- 同层之间尽量避免相互依赖

## 🚀 性能优化

### 1. 缓存机制
- Web 层系统信息缓存（2秒更新一次）
- 减少系统调用开销

### 2. 并行处理
- 配置验证使用 `rayon` 并行处理
- 提升大量配置验证速度

### 3. 原子操作
- 使用 `tempfile` + `persist()` 实现原子写入
- 避免文件损坏风险

### 4. 智能锁定
- 短暂的锁持有时间
- 超时保护避免死锁

## 🧪 测试策略

### 单元测试
每个模块包含独立的单元测试：
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_manager_load_save() {
        // 测试 ConfigManager 的加载和保存
    }
}
```

### 集成测试
`tests/integration_test.rs` 测试端到端工作流：
```rust
#[test]
fn test_config_service_workflow() {
    // 测试完整的配置管理流程
}
```

### 临时目录测试
所有测试使用 `tempfile::tempdir()` 避免污染系统：
```rust
let temp_dir = tempdir().unwrap();
let config_path = temp_dir.path().join("config.toml");
```

## 📚 扩展指南

### 添加新命令

1. 在 `src/commands/` 创建新文件
2. 实现命令函数
3. 在 `mod.rs` 导出
4. 在 `main.rs` 添加 CLI 路由

### 添加新 API 端点

1. 在 `web/models.rs` 定义数据模型
2. 在 `web/handlers.rs` 实现处理器
3. 在 `web/routes.rs` 添加路由（可选）
4. 在 `web/server.rs` 注册路由

### 添加新 Service

1. 在 `services/` 创建新文件
2. 定义 Service 结构体
3. 实现业务逻辑方法
4. 在 `mod.rs` 导出

## 🔧 开发工具

### 构建
```bash
cargo build                # Debug 构建
cargo build --release      # Release 构建
cargo check               # 快速类型检查
```

### 测试
```bash
cargo test                # 运行所有测试
cargo test --lib          # 只运行库测试
cargo test integration    # 运行集成测试
```

### 代码质量
```bash
cargo clippy              # 代码检查
cargo fmt                 # 代码格式化
cargo doc --no-deps       # 生成文档
```

## 📖 相关文档

- [快速开始](./quick-start.md)
- [命令参考](./commands/)
- [配置文件](./configuration.md)
- [更新日志](./changelog.md)

## 🤝 贡献指南

在贡献代码时，请遵循以下原则：

1. **分层原则** - 将代码放在正确的层次
2. **单一职责** - 每个模块只做一件事
3. **依赖注入** - 通过构造函数传递依赖
4. **错误处理** - 使用 `Result<T>` 统一错误处理
5. **测试覆盖** - 为新功能编写测试
6. **文档注释** - 使用 `///` 编写公共 API 文档

---

**版本:** 1.1.1  
**最后更新:** 2025-10-11

