# CCR 命令重复Bug修复步骤

## 问题分析

经过全面排查,发现:
1. **当前源码没有bug** - 所有测试通过,命令定义正确
2. **问题来源** - 可能是之前安装的旧版CCR或Backend缓存的旧依赖
3. **验证结果** - `cargo test` 和 `ccr check conflicts` 均正常工作

## 解决方案

### 步骤1: 重新编译并安装CCR

```powershell
# 清理旧的构建产物
cd d:\Documents\Code\Github\ccr
cargo clean

# 重新构建release版本
cargo build --release --bin ccr

# 安装到系统
cargo install --path .  --force

# 验证版本
ccr --version  # 应该显示 ccr 3.12.3
```

### 步骤2: 清理Backend缓存并重新构建

```powershell
# 清理Backend构建缓存
cd ccr-ui/backend
cargo clean

# 重新构建
cd ..
cargo build --package ccr-ui-backend

# 或者使用just命令
cd ccr-ui
just build-backend
```

### 步骤3: 测试修复效果

```powershell
# 测试后端能否正常启动
cd backend
cargo run

# 应该不再出现 "command name `check` is duplicated" 错误
```

### 步骤4: 验证完整的开发环境

```powershell
cd ccr-ui
just dev  # 应该正常启动,不弹新窗口
```

## 技术细节

### Workspace Patch机制

`Cargo.toml` 第193-197行配置了patch:

```toml
[patch."https://github.com/bahayonghang/ccr"]
ccr = { path = "." }
```

这意味着在workspace中:
- Backend的 `ccr = { git =  "..." }` 会自动被替换为本地路径
- 确保使用最新的本地代码而不是远程版本

### 为什么之前会失败?

可能的原因:
1. **Cargo缓存** - 旧的git依赖被缓存
2. **Debug断言** - clap在debug模式下执行更严格的检查
3. **已安装的旧版本** - 系统PATH中的ccr是旧版本

### 清理建议

如果问题仍然存在,尝试深度清理:

```powershell
# 清理所有cargo缓存
cargo clean

# 删除git依赖缓存
Remove-Item -Recurse -Force ~/.cargo/git/checkouts/ccr-*

# 重新获取依赖
cargo fetch

# 重新构建
cargo build --release
```

## 预防措施

建议在开发时:
1. 定期运行 `cargo test` 确保没有引入新bug
2. 使用 `just ci` 运行完整的CI检查
3. 提交前运行 `cargo clippy` 检查潜在问题

## 验证命令

```powershell
# 验证CCR命令定义
ccr --help | Select-String "check|validate"

# 测试check命令
ccr check conflicts

# 测试backend启动
cd ccr-ui/backend
cargo run
```

## 如果问题持续

如果重新安装后仍有问题,请:
1. 查看详细错误: `$env:RUST_BACKTRACE="full"`; `cargo run`
2. 检查clap版本: `cargo tree | Select-String "clap"  
3. 报告issue并附上完整的backtrace
