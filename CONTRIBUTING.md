# 贡献指南 (Contributing Guide)

感谢你对 CCR 项目的关注！本文档说明如何为 CCR 做出贡献。

## 📋 目录

- [代码规范](#代码规范)
- [错误处理规范](#错误处理规范)
- [测试要求](#测试要求)
- [提交规范](#提交规范)
- [Pull Request 流程](#pull-request-流程)

## 代码规范

### Rust 代码风格

- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 进行静态检查
- 遵循 Rust 2024 Edition 规范

### 命名约定

- 模块名：`snake_case`
- 类型名：`PascalCase`
- 函数名：`snake_case`
- 常量名：`SCREAMING_SNAKE_CASE`

## 错误处理规范

### ⚠️ 避免使用 `unwrap()`

**重要**：在生产代码中，应避免使用 `.unwrap()` 和 `.expect()`，除非有充分理由。

#### 为什么要避免 unwrap？

1. **不优雅的错误处理**：unwrap 会在遇到 None 或 Err 时直接 panic，导致程序崩溃
2. **难以调试**：panic 信息可能不够详细，难以定位问题
3. **用户体验差**：应该向用户返回友好的错误信息，而不是直接崩溃

#### 正确的错误处理方式

❌ **不推荐**：
```rust
let value = some_option.unwrap();
let result = some_result.unwrap();
```

✅ **推荐**：
```rust
// Option 处理
let value = some_option.ok_or_else(|| {
    CcrError::ValidationError("缺少必需的值".into())
})?;

// Result 处理 (如果错误类型支持 From)
let result = some_result?;

// Result 处理 (需要自定义错误消息)
let result = some_result.map_err(|e| {
    CcrError::IoError(format!("操作失败: {}", e))
})?;

// match 表达式 (更精细的控制)
match some_result {
    Ok(value) => value,
    Err(e) => {
        eprintln!("错误: {}", e);
        return Err(CcrError::CustomError(e));
    }
}
```

#### 何时可以使用 unwrap？

在以下情况下可以使用 `unwrap()` 或 `expect()`：

1. **测试代码中**：测试失败时应该立即 panic
   ```rust
   #[test]
   fn test_example() {
       let result = some_function().unwrap();
       assert_eq!(result, expected);
   }
   ```

2. **程序初始化阶段**：某些初始化失败时程序无法继续
   ```rust
   fn main() {
       let config = load_config().expect("无法加载配置文件");
       // ...
   }
   ```

3. **使用 expect() 提供上下文**：如果使用 unwrap，必须用 expect() 并提供详细说明
   ```rust
   let value = option.expect("开发者错误：此处的 option 一定有值，因为...");
   ```

### RwLock 和 Mutex 处理

对于 `RwLock` 和 `Mutex`，使用 `unwrap_or_else` 处理 poisoned 情况：

```rust
let guard = LOCK.read().unwrap_or_else(|poisoned| {
    eprintln!("⚠️  锁被毒化，尝试恢复");
    poisoned.into_inner()
});
```

### flush() 和 I/O 操作

对于非关键的 I/O 操作（如 stdout flush），可以忽略错误：

```rust
let _ = io::stdout().flush();  // 允许：flush 失败通常不是致命错误
```

对于关键的 I/O 操作，应该返回 Result：

```rust
io::stdin().read_line(&mut buffer)?;  // 正确：用 ? 传播错误
```

## 测试要求

### 测试覆盖率

- 目标：95%+ 覆盖率
- 所有新功能必须包含单元测试
- 修改现有功能需要更新相关测试

### 运行测试

```bash
# 运行所有测试
cargo test

# 使用 justfile (推荐)
just test

# 运行特定测试
cargo test test_name

# 运行严格 lint 检查（包括 unwrap 检查）
just lint-strict
```

## 提交规范

### Commit 消息格式

遵循 [Conventional Commits](https://www.conventionalcommits.org/) 规范：

```
<type>(<scope>): <subject>

[optional body]

[optional footer]
```

**Type**:
- `feat`: 新功能
- `fix`: 修复 bug
- `docs`: 文档更新
- `style`: 代码格式（不影响功能）
- `refactor`: 重构（不是新功能也不是修复）
- `perf`: 性能优化
- `test`: 添加测试
- `chore`: 构建过程或辅助工具的变动

**示例**:
```
feat(web): 添加 ValidateService 层级

- 创建 ValidateService 封装验证逻辑
- 修复 Web Handler 层级违规问题
- 添加单元测试

Close #123
```

## Pull Request 流程

1. **Fork 项目**并创建你的分支
   ```bash
   git checkout -b feat/my-feature
   ```

2. **进行修改**并确保通过所有检查
   ```bash
   just ci  # 运行完整 CI 流程
   ```

3. **提交变更**
   ```bash
   git add .
   git commit -m "feat(scope): 描述"
   ```

4. **推送到你的 Fork**
   ```bash
   git push origin feat/my-feature
   ```

5. **创建 Pull Request**
   - 描述你的更改
   - 链接相关 issue
   - 等待 review

### PR 检查清单

在提交 PR 前，请确认：

- [ ] 代码通过 `cargo fmt --check`
- [ ] 代码通过 `cargo clippy` (无警告)
- [ ] 代码通过 `just lint-strict` (可能有测试代码的 unwrap 警告，这是正常的)
- [ ] 所有测试通过 `cargo test`
- [ ] 添加了必要的测试
- [ ] 更新了相关文档
- [ ] Commit 消息遵循规范

## 开发环境设置

### 必需工具

- Rust 1.85+ (支持 edition 2024)
- cargo
- just (任务运行器)

### 推荐工具

- rust-analyzer (IDE 支持)
- cargo-watch (自动重编译)
- cargo-audit (安全审计)

### 快速开始

```bash
# 克隆项目
git clone https://github.com/bahayonghang/ccr.git
cd ccr

# 运行测试
just test

# 运行 lint 检查
just lint

# 构建项目
just build

# 安装到本地
just install
```

## 获取帮助

- 查看 [README.md](README.md) 了解项目概况
- 查看 [CLAUDE.md](CLAUDE.md) 了解架构细节
- 提交 [Issue](https://github.com/bahayonghang/ccr/issues) 报告问题
- 加入讨论区交流

## License

MIT License - 详见 [LICENSE](LICENSE) 文件

---

再次感谢你的贡献！🎉
