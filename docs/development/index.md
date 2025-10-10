# 开发指南

欢迎参与 CCR 的开发！本指南将帮助你了解项目结构、开发流程和最佳实践。

## 🚀 快速开始

### 1. 克隆仓库

```bash
git clone https://github.com/bahayonghang/ccs.git
cd ccs/ccr
```

### 2. 安装依赖

CCR 使用 Cargo 管理依赖，无需额外安装步骤：

```bash
# 检查 Rust 版本（需要 1.70+）
rustc --version

# 构建项目
cargo build
```

### 3. 运行测试

```bash
# 运行所有测试
cargo test

# 运行测试并显示输出
cargo test -- --nocapture

# 运行特定测试
cargo test test_config_section_validate

# 运行特定模块的测试
cargo test config::tests
```

### 4. 代码检查

```bash
# 类型检查
cargo check

# Linter 检查
cargo clippy

# 格式化代码
cargo fmt
```

### 5. 运行程序

```bash
# 运行调试版本
cargo run -- --help
cargo run -- list
cargo run -- switch test-config

# 运行发布版本
cargo run --release -- list
```

## 📁 项目结构

```
ccr/
├── src/                    # 源代码目录
│   ├── main.rs            # 程序入口 (165 行)
│   ├── error.rs           # 错误类型定义 (200 行)
│   ├── logging.rs         # 彩色输出工具 (250 行)
│   ├── lock.rs            # 文件锁机制 (250 行)
│   ├── config.rs          # 配置管理 (350 行)
│   ├── settings.rs        # 设置管理 (400 行) ⭐
│   ├── history.rs         # 历史记录 (400 行)
│   ├── web.rs             # Web 服务器 (490 行)
│   └── commands/          # CLI 命令实现
│       ├── mod.rs         # 模块导出
│       ├── list.rs        # list 命令
│       ├── current.rs     # current 命令
│       ├── switch.rs      # switch 命令
│       ├── validate.rs    # validate 命令
│       └── history_cmd.rs # history 命令
├── web/                   # Web 界面
│   └── index.html         # 单页应用 (1346 行)
├── Cargo.toml             # 项目配置
├── Cargo.lock             # 依赖锁定
├── README.md              # 用户文档
├── CLAUDE.md              # 开发文档
├── justfile               # Just 构建脚本
└── tests/                 # 集成测试（如有）
```

### 模块职责

| 模块 | 职责 | 核心功能 |
|------|------|---------|
| `main.rs` | 程序入口 | CLI 参数解析、命令路由 |
| `error.rs` | 错误处理 | 错误类型、退出码、用户消息 |
| `logging.rs` | 日志输出 | 彩色输出、敏感信息掩码 |
| `lock.rs` | 文件锁 | 跨进程锁、超时保护 |
| `config.rs` | 配置管理 | TOML 解析、配置验证 |
| `settings.rs` | 设置管理 | JSON 操作、原子写入 ⭐ |
| `history.rs` | 历史记录 | 审计追踪、统计信息 |
| `web.rs` | Web 服务 | HTTP 服务器、RESTful API |
| `commands/` | CLI 命令 | 各命令的具体实现 |

## 🛠️ 开发工具

### Cargo 命令

```bash
# 开发
cargo build              # 构建调试版本
cargo run -- <args>      # 运行程序
cargo test               # 运行测试
cargo check              # 快速类型检查

# 代码质量
cargo clippy             # Linter
cargo fmt                # 格式化
cargo doc                # 生成文档
cargo doc --open         # 生成并打开文档

# 发布
cargo build --release    # 构建发布版本
cargo install --path .   # 安装到 ~/.cargo/bin
```

### Just 命令（推荐）

如果安装了 `just`（`cargo install just`）：

```bash
# 查看所有任务
just --list

# 构建
just build               # 调试版本
just release             # 发布版本

# 测试
just test                # 运行测试

# 代码质量
just check               # 类型检查
just clippy              # Linter
just fmt                 # 格式化

# 安装
just install             # 安装到系统
just reinstall           # 强制重新安装
just uninstall           # 卸载
```

## 📝 代码规范

### Rust 风格指南

CCR 遵循标准的 Rust 风格指南：

```rust
// 1. 命名约定
pub struct ConfigManager { }     // 类型：PascalCase
pub fn load_config() { }          // 函数：snake_case
const MAX_RETRIES: u32 = 3;       // 常量：SCREAMING_SNAKE_CASE
let config_path: PathBuf;         // 变量：snake_case

// 2. 错误处理
pub fn load(&self) -> Result<CcsConfig> {  // 返回 Result
    let content = fs::read_to_string(&self.config_path)?;  // 使用 ?
    // ...
}

// 3. 文档注释
/// 加载配置文件
///
/// # Errors
///
/// 如果文件不存在或格式错误，返回 `CcrError`
pub fn load(&self) -> Result<CcsConfig> { }

// 4. 生命周期和所有权
pub fn get_section(&self, name: &str) -> Result<&ConfigSection> {
    self.sections.get(name)
        .ok_or_else(|| CcrError::ConfigSectionNotFound(name.to_string()))
}
```

### 代码组织

```rust
// 1. 导入顺序
use std::collections::HashMap;           // 标准库
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};     // 外部 crate

use crate::error::{CcrError, Result};    // 本地模块
use crate::logging::ColorOutput;

// 2. 模块结构
mod config {
    // 类型定义
    pub struct ConfigManager { }
    
    // 实现
    impl ConfigManager {
        // 公共方法在前
        pub fn new() -> Self { }
        pub fn load(&self) -> Result<()> { }
        
        // 私有方法在后
        fn validate_path(&self) -> bool { }
    }
}

// 3. 测试模块
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_load_config() {
        // 测试代码
    }
}
```

### 错误处理最佳实践

```rust
// 使用 ? 操作符传播错误
pub fn load(&self) -> Result<CcsConfig> {
    let content = fs::read_to_string(&self.config_path)?;
    let config: CcsConfig = toml::from_str(&content)?;
    Ok(config)
}

// 使用 map_err 转换错误
pub fn load(&self) -> Result<CcsConfig> {
    let content = fs::read_to_string(&self.config_path)
        .map_err(|e| CcrError::ConfigError(format!("读取失败: {}", e)))?;
    Ok(())
}

// 使用 ok_or_else 将 Option 转换为 Result
pub fn get_section(&self, name: &str) -> Result<&ConfigSection> {
    self.sections.get(name)
        .ok_or_else(|| CcrError::ConfigSectionNotFound(name.to_string()))
}
```

## 🧪 测试指南

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_section_validate() {
        let section = ConfigSection {
            description: Some("Test".into()),
            base_url: Some("https://api.test.com".into()),
            auth_token: Some("sk-test-token".into()),
            model: Some("test-model".into()),
            small_fast_model: None,
        };
        
        assert!(section.validate().is_ok());
    }
    
    #[test]
    fn test_invalid_base_url() {
        let section = ConfigSection {
            base_url: Some("not-a-url".into()),
            auth_token: Some("token".into()),
            ..Default::default()
        };
        
        assert!(section.validate().is_err());
    }
}
```

### 集成测试

在 `tests/` 目录创建集成测试：

```rust
// tests/integration_test.rs
use ccr::config::ConfigManager;
use tempfile::tempdir;

#[test]
fn test_config_lifecycle() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("test_config.toml");
    
    let manager = ConfigManager::new(&config_path);
    // 测试加载、保存等操作
}
```

### 运行测试

```bash
# 所有测试
cargo test

# 单个测试
cargo test test_config_section_validate

# 带输出
cargo test -- --nocapture

# 测试覆盖率（需要 tarpaulin）
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

## 🔧 添加新功能

### 添加新命令

1. 在 `src/commands/` 创建新文件 `newcmd.rs`：

```rust
use crate::error::Result;
use crate::logging::ColorOutput;

pub fn newcmd_command(args: YourArgs) -> Result<()> {
    ColorOutput::title("新命令");
    
    // 实现逻辑
    
    ColorOutput::success("命令执行成功");
    Ok(())
}
```

2. 在 `src/commands/mod.rs` 导出：

```rust
pub mod newcmd;
pub use newcmd::newcmd_command;
```

3. 在 `src/main.rs` 添加命令：

```rust
#[derive(Subcommand)]
enum Commands {
    // ... 现有命令
    
    /// 新命令描述
    Newcmd {
        /// 参数描述
        #[arg(short, long)]
        param: String,
    },
}

// 在 match 块中添加处理
Some(Commands::Newcmd { param }) => {
    commands::newcmd_command(param)
}
```

### 添加新错误类型

在 `src/error.rs` 添加：

```rust
#[derive(Error, Debug)]
pub enum CcrError {
    // ... 现有错误类型
    
    #[error("新错误类型: {0}")]
    NewError(String),
}

impl CcrError {
    pub fn exit_code(&self) -> i32 {
        match self {
            // ...
            CcrError::NewError(_) => 110,  // 分配新的错误码
        }
    }
}
```

## 🔗 相关文档

- [项目结构](/development/structure)
- [构建系统](/development/build)
- [测试指南](/development/testing)
- [代码规范](/development/code-style)
- [贡献指南](/development/contributing)
- [添加新命令](/development/add-command)

