# 测试指南

CCR 采用全面的测试策略，包括单元测试、集成测试和手动测试。

## 🧪 测试类型

### 1. 单元测试

每个模块都包含 `#[cfg(test)]` 测试模块：

```rust
// src/config.rs
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
        let mut section = create_test_section();
        section.base_url = Some("not-a-url".into());
        assert!(section.validate().is_err());
    }
}
```

### 2. 集成测试

使用 `tests/` 目录的集成测试：

```rust
// tests/config_integration.rs
use ccr::config::{ConfigManager, ConfigSection, CcsConfig};
use tempfile::tempdir;

#[test]
fn test_full_config_workflow() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("test_config.toml");
    
    // 创建管理器
    let manager = ConfigManager::new(&config_path);
    
    // 创建配置
    let mut config = CcsConfig::new("default".into());
    
    // 添加配置节
    let section = ConfigSection {
        base_url: Some("https://api.test.com".into()),
        auth_token: Some("test-token".into()),
        ..Default::default()
    };
    config.set_section("test".into(), section);
    
    // 保存
    manager.save(&config).unwrap();
    assert!(config_path.exists());
    
    // 加载
    let loaded = manager.load().unwrap();
    assert_eq!(loaded.default_config, "default");
    assert!(loaded.get_section("test").is_ok());
}
```

### 3. 手动测试

使用 shell 脚本进行端到端测试：

```bash
#!/bin/bash
# tests/manual/test_switch.sh

set -e

echo "测试配置切换功能"

# 1. 列出配置
echo "1. 列出配置"
cargo run -- list

# 2. 切换配置
echo "2. 切换配置"
cargo run -- switch test-config

# 3. 验证切换
echo "3. 验证切换"
cargo run -- current | grep "test-config"

# 4. 查看历史
echo "4. 查看历史"
cargo run -- history --limit 1

echo "✓ 所有测试通过"
```

## 🏃 运行测试

### 运行所有测试

```bash
cargo test
```

### 运行特定测试

```bash
# 运行单个测试
cargo test test_config_section_validate

# 运行模块的所有测试
cargo test config::tests

# 运行集成测试
cargo test --test config_integration
```

### 显示测试输出

```bash
# 显示 println! 等输出
cargo test -- --nocapture

# 显示成功的测试
cargo test -- --show-output
```

### 并行控制

```bash
# 单线程运行（用于调试）
cargo test -- --test-threads=1

# 指定线程数
cargo test -- --test-threads=4
```

## 📊 测试覆盖率

### 安装 tarpaulin

```bash
cargo install cargo-tarpaulin
```

### 生成覆盖率报告

```bash
# HTML 报告
cargo tarpaulin --out Html

# 终端输出
cargo tarpaulin --out Stdout

# 多种格式
cargo tarpaulin --out Html --out Lcov
```

### 查看报告

```bash
# 打开 HTML 报告
open tarpaulin-report.html
```

## 🎯 测试最佳实践

### 1. 使用临时目录

```rust
#[test]
fn test_with_temp_dir() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("test_config.toml");
    
    // 测试代码
    // ...
    
    // temp_dir 自动清理
}
```

### 2. 测试成功和失败情况

```rust
#[test]
fn test_config_validation() {
    // 测试成功情况
    let valid_section = create_valid_section();
    assert!(valid_section.validate().is_ok());
    
    // 测试失败情况
    let invalid_section = create_invalid_section();
    assert!(invalid_section.validate().is_err());
}
```

### 3. 使用 helper 函数

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    // Helper 函数
    fn create_test_section() -> ConfigSection {
        ConfigSection {
            description: Some("Test".into()),
            base_url: Some("https://api.test.com".into()),
            auth_token: Some("sk-test-token".into()),
            model: Some("test-model".into()),
            small_fast_model: None,
        }
    }
    
    #[test]
    fn test_something() {
        let section = create_test_section();
        // 使用 section
    }
}
```

### 4. 测试错误消息

```rust
#[test]
fn test_error_messages() {
    let err = CcrError::ConfigSectionNotFound("test".into());
    
    assert_eq!(err.exit_code(), 12);
    assert!(!err.is_fatal());
    assert!(err.user_message().contains("test"));
    assert!(err.user_message().contains("建议"));
}
```

## 🔍 测试示例

### 测试 ConfigManager

```rust
#[test]
fn test_config_manager_load_save() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("test_config.toml");
    
    // 创建测试配置
    let mut config = CcsConfig::new("test".into());
    config.set_section("test".into(), create_test_section());
    
    // 保存
    let manager = ConfigManager::new(&config_path);
    manager.save(&config).unwrap();
    assert!(config_path.exists());
    
    // 加载
    let loaded = manager.load().unwrap();
    assert_eq!(loaded.default_config, "test");
    assert!(loaded.get_section("test").is_ok());
}
```

### 测试 SettingsManager

```rust
#[test]
fn test_settings_manager_save_load() {
    let temp_dir = tempfile::tempdir().unwrap();
    let settings_path = temp_dir.path().join("settings.json");
    let backup_dir = temp_dir.path().join("backups");
    let lock_dir = temp_dir.path().join("locks");
    
    let lock_manager = LockManager::new(lock_dir);
    let manager = SettingsManager::new(settings_path, backup_dir, lock_manager);
    
    // 创建并保存设置
    let mut settings = ClaudeSettings::new();
    settings.update_from_config(&create_test_config_section());
    
    manager.save_atomic(&settings).unwrap();
    
    // 加载并验证
    let loaded = manager.load().unwrap();
    assert_eq!(
        loaded.env.get("ANTHROPIC_BASE_URL"),
        Some(&"https://api.test.com".to_string())
    );
}
```

### 测试 FileLock

```rust
#[test]
fn test_file_lock_basic() {
    let temp_dir = tempfile::tempdir().unwrap();
    let lock_path = temp_dir.path().join("test.lock");
    
    // 获取锁
    let lock = FileLock::new(&lock_path, Duration::from_secs(5)).unwrap();
    assert!(lock_path.exists());
    
    // 释放锁
    lock.unlock().unwrap();
}

#[test]
fn test_file_lock_timeout() {
    let temp_dir = tempfile::tempdir().unwrap();
    let lock_path = temp_dir.path().join("test.lock");
    
    // 第一个锁
    let _lock1 = FileLock::new(&lock_path, Duration::from_secs(5)).unwrap();
    
    // 第二个锁应该超时
    let lock2_result = FileLock::new(&lock_path, Duration::from_millis(200));
    assert!(lock2_result.is_err());
}
```

## 🐛 调试技巧

### 1. 使用 dbg! 宏

```rust
let config = load_config()?;
dbg!(&config);  // 打印调试信息
```

### 2. 设置日志级别

```bash
export CCR_LOG_LEVEL=debug
cargo test -- --nocapture
```

### 3. 运行特定测试

```bash
# 只运行一个测试，方便调试
cargo test test_specific_case -- --nocapture
```

### 4. 检查测试输出

```bash
# 显示详细输出
cargo test -- --nocapture --test-threads=1
```

## 🔧 持续集成

### GitHub Actions 示例

```yaml
# .github/workflows/test.yml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Run tests
        run: cargo test --all-features
      
      - name: Run clippy
        run: cargo clippy -- -D warnings
      
      - name: Check formatting
        run: cargo fmt -- --check
```

## 🔗 相关文档

- [开发指南](/development/)
- [代码规范](/development/code-style)
- [添加新命令](/development/add-command)

