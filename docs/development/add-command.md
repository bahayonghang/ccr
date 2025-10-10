# 添加新命令

本指南详细介绍如何为 CCR 添加新的 CLI 命令，包含完整的示例和最佳实践。

## 🎯 添加命令的步骤

### 1. 创建命令文件

在 `src/commands/` 目录创建新文件：

```bash
touch src/commands/info.rs
```

### 2. 实现命令逻辑

```rust
// src/commands/info.rs
use crate::config::ConfigManager;
use crate::error::Result;
use crate::logging::ColorOutput;

/// 显示配置详细信息
pub fn info_command(config_name: &str) -> Result<()> {
    ColorOutput::title(&format!("配置信息: {}", config_name));
    println!();
    
    // 加载配置
    let config_manager = ConfigManager::default()?;
    let config = config_manager.load()?;
    
    // 获取配置节
    let section = config.get_section(config_name)?;
    
    // 显示信息
    ColorOutput::step("配置详情:");
    ColorOutput::key_value("  配置名称", config_name, 2);
    ColorOutput::key_value("  描述", &section.display_description(), 2);
    
    if let Some(base_url) = &section.base_url {
        ColorOutput::key_value("  Base URL", base_url, 2);
    }
    
    if let Some(auth_token) = &section.auth_token {
        ColorOutput::key_value_sensitive("  Auth Token", auth_token, 2);
    }
    
    if let Some(model) = &section.model {
        ColorOutput::key_value("  Model", model, 2);
    }
    
    println!();
    
    // 验证配置
    match section.validate() {
        Ok(_) => ColorOutput::success("✓ 配置验证通过"),
        Err(e) => ColorOutput::error(&format!("✗ 配置验证失败: {}", e)),
    }
    
    Ok(())
}
```

### 3. 导出命令

在 `src/commands/mod.rs` 中添加：

```rust
// src/commands/mod.rs
pub mod list;
pub mod current;
pub mod switch;
pub mod validate;
pub mod history_cmd;
pub mod info;  // ← 新增

pub use list::list_command;
pub use current::current_command;
pub use switch::switch_command;
pub use validate::validate_command;
pub use history_cmd::history_command;
pub use info::info_command;  // ← 新增
```

### 4. 添加 CLI 定义

在 `src/main.rs` 中添加命令：

```rust
// src/main.rs

#[derive(Subcommand)]
enum Commands {
    /// 列出所有可用配置
    #[command(alias = "ls")]
    List,
    
    /// 显示当前配置状态
    #[command(alias = "show")]
    #[command(alias = "status")]
    Current,
    
    /// 切换到指定配置
    Switch {
        /// 要切换到的配置名称
        config_name: String,
    },
    
    /// 验证配置和设置的完整性
    #[command(alias = "check")]
    Validate,
    
    /// 显示操作历史
    History {
        /// 限制显示的记录数量
        #[arg(short, long, default_value_t = 20)]
        limit: usize,
        
        /// 按操作类型筛选
        #[arg(short = 't', long)]
        filter_type: Option<String>,
    },
    
    /// 启动 Web 配置界面
    Web {
        /// 指定端口
        #[arg(short, long, default_value_t = 8080)]
        port: u16,
    },
    
    /// 显示配置详细信息
    Info {
        /// 配置名称
        config_name: String,
    },  // ← 新增
    
    /// 显示版本信息
    #[command(alias = "ver")]
    Version,
}
```

### 5. 添加命令路由

在 `main()` 函数的 match 块中添加：

```rust
fn main() {
    init_logger();
    let cli = Cli::parse();
    
    let result = match cli.command {
        Some(Commands::List) => commands::list_command(),
        Some(Commands::Current) => commands::current_command(),
        Some(Commands::Switch { config_name }) => commands::switch_command(&config_name),
        Some(Commands::Validate) => commands::validate_command(),
        Some(Commands::History { limit, filter_type }) => {
            commands::history_command(Some(limit), filter_type)
        }
        Some(Commands::Web { port }) => web::web_command(Some(port)),
        Some(Commands::Info { config_name }) => {
            commands::info_command(&config_name)  // ← 新增
        }
        Some(Commands::Version) => {
            show_version();
            Ok(())
        }
        None => {
            if let Some(config_name) = cli.config_name {
                commands::switch_command(&config_name)
            } else {
                commands::current_command()
            }
        }
    };
    
    // 错误处理
    if let Err(e) = result {
        eprintln!();
        ColorOutput::error(&e.user_message());
        std::process::exit(e.exit_code());
    }
}
```

### 6. 测试命令

```bash
# 构建
cargo build

# 测试新命令
cargo run -- info anthropic

# 测试帮助
cargo run -- info --help
```

## 📝 完整示例

### 示例：export 命令

导出配置为 JSON 格式。

#### 1. 创建命令文件

```rust
// src/commands/export.rs
use crate::config::ConfigManager;
use crate::error::Result;
use crate::logging::ColorOutput;
use serde_json;

/// 导出配置为 JSON
pub fn export_command(config_name: &str, output: Option<String>) -> Result<()> {
    ColorOutput::title(&format!("导出配置: {}", config_name));
    println!();
    
    // 加载配置
    let config_manager = ConfigManager::default()?;
    let config = config_manager.load()?;
    let section = config.get_section(config_name)?;
    
    // 序列化为 JSON
    let json = serde_json::to_string_pretty(&section)
        .map_err(|e| CcrError::JsonError(e))?;
    
    // 输出
    if let Some(output_path) = output {
        // 写入文件
        std::fs::write(&output_path, json)
            .map_err(|e| CcrError::IoError(e))?;
        ColorOutput::success(&format!("配置已导出到: {}", output_path));
    } else {
        // 输出到标准输出
        println!("{}", json);
    }
    
    Ok(())
}
```

#### 2. 添加到模块

```rust
// src/commands/mod.rs
pub mod export;
pub use export::export_command;
```

#### 3. 添加 CLI 定义

```rust
// src/main.rs
#[derive(Subcommand)]
enum Commands {
    // ...
    
    /// 导出配置为 JSON
    Export {
        /// 配置名称
        config_name: String,
        
        /// 输出文件路径（可选，默认输出到标准输出）
        #[arg(short, long)]
        output: Option<String>,
    },
}

// 添加路由
Some(Commands::Export { config_name, output }) => {
    commands::export_command(&config_name, output)
}
```

#### 4. 使用示例

```bash
# 导出到标准输出
ccr export anthropic

# 导出到文件
ccr export anthropic --output config.json
ccr export anthropic -o config.json
```

## 🎨 命令设计原则

### 1. 单一职责

每个命令只做一件事：

```rust
// ✅ 好：只列出配置
pub fn list_command() -> Result<()> {
    // 只负责列出配置
}

// ❌ 不好：做太多事情
pub fn list_and_switch_command(maybe_switch: Option<String>) -> Result<()> {
    // 既列出又切换，职责不清
}
```

### 2. 返回 Result

所有命令都返回 `Result<()>`：

```rust
pub fn my_command() -> Result<()> {
    // 使用 ? 传播错误
    let config = load_config()?;
    
    // 成功时返回 Ok
    Ok(())
}
```

### 3. 使用 ColorOutput

统一使用 ColorOutput 进行输出：

```rust
// ✅ 使用 ColorOutput
ColorOutput::success("操作成功");
ColorOutput::error("操作失败");

// ❌ 直接 println
println!("操作成功");  // 不推荐
```

### 4. 详细的用户反馈

```rust
// ✅ 好：详细的进度信息
ColorOutput::step("步骤 1/3: 读取配置");
// 执行操作
ColorOutput::success("配置读取成功");

ColorOutput::step("步骤 2/3: 验证配置");
// 执行操作
ColorOutput::success("验证通过");

// ❌ 不好：静默执行
load_config()?;
validate()?;
println!("完成");
```

## 🧪 测试新命令

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_info_command() {
        // 创建临时配置
        let temp_dir = tempfile::tempdir().unwrap();
        // ...
        
        // 测试命令
        let result = info_command("test");
        assert!(result.is_ok());
    }
}
```

### 集成测试

```bash
# 手动测试
cargo run -- info anthropic
cargo run -- info nonexistent  # 测试错误情况
cargo run -- info --help       # 测试帮助信息
```

## 📚 参考现有命令

学习现有命令的实现：

- **简单命令**: `src/commands/list.rs` (约 70 行)
- **中等复杂**: `src/commands/current.rs` (约 90 行)
- **复杂命令**: `src/commands/switch.rs` (约 135 行)

## 🔗 相关文档

- [开发指南](/development/)
- [项目结构](/development/structure)
- [代码规范](/development/code-style)
- [API 参考](/api/)

