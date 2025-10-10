# 贡献指南

感谢你对 CCR 项目的关注！我们欢迎各种形式的贡献。

## 🤝 如何贡献

### 贡献类型

- 🐛 **报告 Bug** - 提交问题报告
- ✨ **功能建议** - 提出新功能想法
- 📝 **改进文档** - 修正或完善文档
- 💻 **提交代码** - 修复 Bug 或实现新功能
- 🧪 **编写测试** - 增加测试覆盖率
- 🎨 **改进界面** - 优化 CLI 或 Web 界面

## 🐛 报告 Bug

### 提交前检查

1. 搜索 [现有 Issues](https://github.com/bahayonghang/ccs/issues) 确认问题未被报告
2. 使用最新版本的 CCR
3. 收集诊断信息

### Bug 报告模板

```markdown
## 问题描述
简要描述问题

## 复现步骤
1. 运行 `ccr list`
2. 执行 `ccr switch test`
3. 看到错误

## 预期行为
应该切换成功

## 实际行为
出现错误：...

## 环境信息
- CCR 版本: `ccr --version`
- 操作系统: Ubuntu 22.04
- Rust 版本: `rustc --version`

## 诊断信息
\`\`\`bash
# 配置文件（移除敏感信息）
cat ~/.ccs_config.toml | sed 's/auth_token.*/auth_token = "***"/'

# 验证输出
ccr validate

# 历史记录
ccr history --limit 3
\`\`\`

## 其他信息
任何其他相关信息
```

## ✨ 功能建议

### 提交前考虑

1. 功能是否符合 CCR 的定位
2. 是否有替代方案
3. 实现的复杂度

### 功能请求模板

```markdown
## 功能描述
简要描述建议的功能

## 使用场景
为什么需要这个功能？什么场景下会用到？

## 建议的实现
（可选）如何实现这个功能

## 替代方案
（可选）是否有其他实现方式

## 额外信息
其他相关信息
```

## 💻 提交代码

### 开发流程

1. **Fork 仓库**
   ```bash
   # 在 GitHub 上 Fork
   # 然后克隆你的 Fork
   git clone https://github.com/YOUR_USERNAME/ccs.git
   cd ccs/ccr
   ```

2. **创建分支**
   ```bash
   git checkout -b feature/new-feature
   # 或
   git checkout -b fix/bug-description
   ```

3. **开发和测试**
   ```bash
   # 编写代码
   vim src/...
   
   # 运行测试
   cargo test
   
   # 代码检查
   cargo clippy
   
   # 格式化
   cargo fmt
   ```

4. **提交代码**
   ```bash
   git add .
   git commit -m "feat: add new feature"
   # 或
   git commit -m "fix: resolve bug"
   ```

5. **推送分支**
   ```bash
   git push origin feature/new-feature
   ```

6. **创建 Pull Request**
   - 访问 GitHub 仓库
   - 点击 "Pull Request"
   - 填写 PR 描述
   - 提交 PR

### Commit 规范

使用 [Conventional Commits](https://www.conventionalcommits.org/)：

```bash
# 功能
git commit -m "feat: add export command"

# 修复
git commit -m "fix: resolve file lock timeout"

# 文档
git commit -m "docs: update installation guide"

# 样式
git commit -m "style: format code with rustfmt"

# 重构
git commit -m "refactor: simplify config validation"

# 测试
git commit -m "test: add unit tests for ConfigManager"

# 性能
git commit -m "perf: optimize file lock acquisition"

# 构建
git commit -m "build: update dependencies"
```

### 代码规范

#### Rust 风格

```rust
// 1. 使用 rustfmt
cargo fmt

// 2. 遵循 Clippy 建议
cargo clippy

// 3. 添加文档注释
/// 加载配置文件
///
/// # Errors
///
/// 如果文件不存在或格式错误，返回 `CcrError`
pub fn load(&self) -> Result<CcsConfig> {
    // ...
}

// 4. 编写测试
#[cfg(test)]
mod tests {
    #[test]
    fn test_load() {
        // ...
    }
}
```

#### 命名约定

```rust
// 类型：PascalCase
pub struct ConfigManager { }
pub enum CcrError { }

// 函数：snake_case
pub fn load_config() { }
pub fn save_atomic() { }

// 常量：SCREAMING_SNAKE_CASE
const MAX_RETRIES: u32 = 3;
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

// 变量：snake_case
let config_path: PathBuf;
let is_valid: bool;
```

### Pull Request 要求

✅ **必需**:
- [ ] 代码通过 `cargo test`
- [ ] 代码通过 `cargo clippy`
- [ ] 代码格式符合 `cargo fmt`
- [ ] 添加了必要的测试
- [ ] 更新了相关文档

⭐ **推荐**:
- [ ] 添加了单元测试
- [ ] 添加了集成测试
- [ ] 更新了 CHANGELOG.md
- [ ] 添加了示例代码

### PR 描述模板

```markdown
## 变更类型
- [ ] Bug 修复
- [ ] 新功能
- [ ] 文档更新
- [ ] 性能优化
- [ ] 代码重构

## 变更说明
简要描述你的修改

## 测试
描述如何测试你的修改

## 截图
（如果是 UI 变更）

## Checklist
- [ ] 代码通过所有测试
- [ ] 代码通过 clippy 检查
- [ ] 代码已格式化
- [ ] 添加了测试
- [ ] 更新了文档
```

## 📝 文档贡献

### 文档结构

```
docs/
├── architecture/    # 架构文档
├── installation/    # 安装指南
├── commands/        # 命令参考
├── api/            # API 参考
└── development/    # 开发指南
```

### 编写规范

**Markdown 格式**:
```markdown
# 一级标题

简要介绍本页内容

## 二级标题

### 三级标题

正文内容...

## 代码示例

\`\`\`rust
pub fn example() -> Result<()> {
    Ok(())
}
\`\`\`

## 提示框

::: tip
这是一个提示
:::

::: warning
这是一个警告
:::
```

**图表**:
```markdown
\`\`\`mermaid
graph LR
    A[Start] --> B[Process]
    B --> C[End]
\`\`\`
```

### 本地预览

```bash
cd docs
npm install
npm run dev
```

## 🧪 测试贡献

### 添加测试

```rust
// 1. 在模块中添加测试
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_new_feature() {
        // 测试代码
    }
}

// 2. 或创建集成测试
// tests/integration_test.rs
#[test]
fn test_full_workflow() {
    // 集成测试
}
```

### 测试覆盖率

```bash
# 检查覆盖率
cargo install cargo-tarpaulin
cargo tarpaulin --out Html

# 目标：覆盖率 > 80%
```

## 🎨 Web 界面贡献

### 修改 Web 界面

```bash
# 1. 编辑 HTML
vim ccr/web/index.html

# 2. 测试
cargo run -- web

# 3. 在浏览器中验证
```

### Web 界面规范

- 保持深色主题风格
- 使用现有的 CSS 变量
- 确保响应式设计
- 测试移动端兼容性

## 🏆 贡献者指南

### 首次贡献

如果这是你第一次为开源项目贡献：

1. 从简单的任务开始
   - 修正文档拼写
   - 改进注释
   - 添加测试

2. 阅读现有代码
   - 了解项目结构
   - 学习代码风格
   - 理解设计决策

3. 寻求帮助
   - 在 Issue 中提问
   - 在 PR 中请求 Review
   - 参与 Discussions

### 成为维护者

活跃的贡献者可能会被邀请成为维护者：

**要求**:
- 持续的高质量贡献
- 对项目的深入理解
- 帮助其他贡献者
- 遵循项目规范

## 📋 开发任务

当前需要帮助的领域：

### 高优先级

- [ ] 增加测试覆盖率
- [ ] 性能基准测试
- [ ] 更多平台测试（Windows）
- [ ] 完善文档

### 中优先级

- [ ] 配置导入/导出
- [ ] 配置模板系统
- [ ] Web 界面增强
- [ ] 更多统计功能

### 低优先级

- [ ] 插件系统
- [ ] 自定义主题
- [ ] 国际化支持
- [ ] 配置加密

## 🔍 Code Review 标准

我们会检查以下方面：

### 代码质量

- ✅ 遵循 Rust 风格指南
- ✅ 通过 clippy 检查
- ✅ 代码格式正确
- ✅ 没有编译警告

### 测试

- ✅ 有相应的单元测试
- ✅ 测试覆盖关键路径
- ✅ 测试通过

### 文档

- ✅ 公共 API 有文档注释
- ✅ 复杂逻辑有代码注释
- ✅ 更新了用户文档

### 设计

- ✅ 符合项目架构
- ✅ 保持向后兼容
- ✅ 考虑了性能影响

## 🎉 致谢

感谢所有贡献者！你们的贡献让 CCR 变得更好。

## 🔗 相关资源

- [开发指南](/development/)
- [项目结构](/development/structure)
- [测试指南](/development/testing)
- [添加新命令](/development/add-command)
- [GitHub 仓库](https://github.com/bahayonghang/ccs)

