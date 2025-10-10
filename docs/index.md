---
layout: home

hero:
  name: "CCR 技术文档"
  text: "Claude Code Configuration Switcher"
  tagline: "Rust 实现的企业级 API 配置管理工具"
  image:
    src: /logo.svg
    alt: CCR
  actions:
    - theme: brand
      text: 快速开始
      link: /installation/
    - theme: alt
      text: 架构设计
      link: /architecture/
    - theme: alt
      text: 命令参考
      link: /commands/
    - theme: alt
      text: GitHub
      link: https://github.com/bahayonghang/ccs

features:
  - icon: 🦀
    title: Rust 强力驱动
    details: 采用 Rust 编写，提供极致性能、内存安全和并发保证，编译后无运行时依赖
  - icon: 🎯
    title: 直接写入设置
    details: 直接操作 ~/.claude/settings.json 文件，配置立即生效，无需手动重启或刷新环境
  - icon: 🔒
    title: 并发安全保证
    details: 内置文件锁机制和原子操作，确保多进程环境下的数据一致性和并发安全
  - icon: 📝
    title: 完整审计追踪
    details: 记录所有操作历史，包含环境变量变更、操作者信息和敏感数据自动掩码
  - icon: 💾
    title: 自动备份恢复
    details: 切换前自动备份当前设置，支持从备份恢复，带时间戳的备份文件管理
  - icon: ✅
    title: 配置验证系统
    details: 自动验证配置完整性，检查必填字段和 URL 格式，提供详细的验证报告
  - icon: 🌐
    title: Web 管理界面
    details: 现代化的 Web 界面，支持可视化配置管理、实时验证和历史记录查看
  - icon: 🔄
    title: CCS 完全兼容
    details: 共享 ~/.ccs_config.toml 配置文件，可与 Shell 版本 CCS 无缝切换使用
  - icon: ⚡
    title: 极致性能
    details: 毫秒级配置切换，低内存占用，编译优化后的二进制文件仅约 2MB
  - icon: 🛡️
    title: 企业级可靠性
    details: 13种错误类型，统一错误处理，详细的用户提示和故障排除指南
  - icon: 📊
    title: 历史统计分析
    details: 操作历史记录、统计信息生成、按类型和时间筛选，支持数据分析
  - icon: 🎨
    title: 现代化体验
    details: 彩色终端输出、进度指示、友好的用户界面和详细的帮助信息
---

<div class="vp-doc">

## 🚀 为什么选择 CCR？

CCR (Claude Code Configuration Switcher) 是 CCS (Claude Code Configuration Switcher) 的 Rust 实现版本，专为追求**性能、安全和可靠性**的用户设计。

<div class="comparison-grid">

### 🎯 CCR 的核心优势

**相比 Shell 版本 CCS 的改进**：

<div class="feature-comparison">

| 特性 | CCS (Shell) | CCR (Rust) |
|-----|------------|-----------|
| **配置生效** | 环境变量 | 直接写入 settings.json ✨ |
| **并发安全** | ❌ | 文件锁 + 原子操作 ✅ |
| **操作历史** | ❌ | 完整审计追踪 ✅ |
| **自动备份** | ❌ | 时间戳备份 ✅ |
| **性能** | 快 | 极快 ⚡ |
| **内存占用** | 低 | 极低 (~2MB) |
| **跨平台** | Bash/Zsh/Fish | 编译后独立运行 |
| **错误处理** | 基础 | 13种错误类型 + 详细提示 |

</div>

</div>

<div class="advantage-card">

### 🏆 关键技术优势

**1. 直接写入 Claude Code 设置**
- 操作 `~/.claude/settings.json` 而非环境变量
- 配置立即生效，无需 shell 重启
- 使用 `#[serde(flatten)]` 保留其他设置

**2. 原子操作保证**
- 使用 `NamedTempFile` + `persist()` 原子替换
- 文件锁防止并发写入冲突
- 超时保护避免死锁

**3. 完整审计追踪**
- UUID 标识每个操作
- 记录时间戳和操作者
- 敏感信息自动掩码

**4. 企业级错误处理**
```rust
pub enum CcrError {
    ConfigError(String),       // 配置文件错误
    ConfigMissing(String),      // 配置文件缺失
    SettingsError(String),      // 设置文件错误
    FileLockError(String),      // 文件锁错误
    ValidationError(String),    // 验证失败
    // ... 13种错误类型
}
```

</div>

## 📚 文档结构

<div class="doc-grid">

### 🏗️ [架构设计](/architecture/)
深入了解 CCR 的技术实现和设计决策
- **整体架构** - 分层设计和模块关系
- **核心模块** - 配置、设置、历史、锁管理
- **数据流程** - 配置切换的完整流程
- **与 CCS 对比** - 技术选型和优势分析

### 🔧 [安装指南](/installation/)
从源码构建和配置 CCR
- **从源码构建** - Rust 工具链和编译步骤
- **配置文件** - TOML 格式和字段说明
- **环境变量** - 管理的环境变量列表
- **故障排除** - 常见问题和解决方案

### 📖 [命令参考](/commands/)
完整的 CLI 命令使用指南
- **list** - 列出所有可用配置
- **current** - 显示当前配置状态
- **switch** - 切换到指定配置
- **validate** - 验证配置完整性
- **history** - 查看操作历史
- **web** - 启动 Web 管理界面

### 🔌 [API 参考](/api/)
Rust 模块和函数接口文档
- **配置管理** - ConfigManager API
- **设置管理** - SettingsManager API
- **历史记录** - HistoryManager API
- **文件锁** - LockManager API
- **Web API** - RESTful 接口规范

### 👨‍💻 [开发指南](/development/)
参与 CCR 开发的完整指南
- **项目结构** - 代码组织和模块划分
- **构建系统** - Cargo 和测试
- **代码规范** - Rust 最佳实践
- **添加新命令** - 扩展 CLI 功能
- **贡献指南** - 如何提交 PR

</div>

## 🎯 快速开始

<div class="quick-start">

### 1️⃣ 从源码构建

```bash
# 克隆仓库
cd ccs/ccr

# 构建发布版本
cargo build --release

# 运行 CCR
./target/release/ccr --help
```

### 2️⃣ 配置文件准备

CCR 使用与 CCS 相同的配置文件 `~/.ccs_config.toml`：

```toml
default_config = "anthropic"
current_config = "anthropic"

[anthropic]
description = "Anthropic 官方 API"
base_url = "https://api.anthropic.com"
auth_token = "sk-ant-your-api-key"
model = "claude-sonnet-4-5-20250929"
small_fast_model = "claude-3-5-haiku-20241022"

[anyrouter]
description = "AnyRouter 代理服务"
base_url = "https://api.anyrouter.ai/v1"
auth_token = "your-anyrouter-token"
model = "claude-sonnet-4-5-20250929"
```

### 3️⃣ 基本使用

```bash
# 列出所有配置
ccr list

# 查看当前配置
ccr current

# 切换配置
ccr switch anyrouter

# 验证配置
ccr validate

# 查看历史
ccr history

# 启动 Web 界面
ccr web
```

</div>

## 🔗 相关链接

<div class="link-grid">

- 📦 [GitHub 仓库](https://github.com/bahayonghang/ccs)
- 🐛 [问题反馈](https://github.com/bahayonghang/ccs/issues)
- 💬 [讨论区](https://github.com/bahayonghang/ccs/discussions)
- 📝 [更新日志](/changelog)
- 🔄 [迁移指南](/migration)
- 🐚 [CCS Shell 版本](https://github.com/bahayonghang/ccs)

</div>

## 🎨 技术栈

<div class="tech-stack">

**核心技术**：
- **Rust 2024 Edition** - 主要编程语言
- **Clap 4.5** - 命令行参数解析
- **Serde & Serde JSON** - 序列化和反序列化
- **TOML** - 配置文件解析
- **fs4** - 文件锁支持
- **tempfile** - 原子文件操作
- **chrono** - 时间戳处理
- **uuid** - 唯一标识符
- **whoami** - 用户识别

**Web 界面**：
- **tiny_http** - 轻量级 HTTP 服务器
- **纯 HTML/CSS/JavaScript** - 无前端框架依赖

</div>

---

<div class="footer-note">

<div class="version-info">
📌 <strong>当前版本</strong>: v0.2.0 | 🦀 <strong>最低 Rust 版本</strong>: 1.70+ | 📅 <strong>最后更新</strong>: 2025-01-10
</div>

💡 **提示**: CCR 与 CCS 完全兼容，可以共享同一个配置文件，随时切换使用。

</div>

</div>

<style>
.vp-doc {
  max-width: 1200px;
  margin: 0 auto;
}

.comparison-grid {
  background: var(--vp-c-bg-soft);
  border-radius: 12px;
  padding: 2rem;
  margin: 2rem 0;
  border: 1px solid var(--vp-c-divider);
}

.feature-comparison {
  overflow-x: auto;
  margin: 1.5rem 0;
}

.feature-comparison table {
  width: 100%;
  border-collapse: collapse;
}

.feature-comparison th,
.feature-comparison td {
  padding: 0.75rem;
  text-align: left;
  border-bottom: 1px solid var(--vp-c-divider);
}

.feature-comparison th {
  background: var(--vp-c-bg-soft);
  font-weight: 600;
  color: var(--vp-c-brand);
}

.advantage-card {
  background: linear-gradient(135deg, rgba(139, 92, 246, 0.1), rgba(168, 85, 247, 0.1));
  border-radius: 12px;
  padding: 2rem;
  margin: 2rem 0;
  border: 1px solid var(--vp-c-brand);
}

.advantage-card h3 {
  margin-top: 0;
  color: var(--vp-c-brand);
}

.doc-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 2rem;
  margin: 2rem 0;
}

.doc-grid > div {
  background: var(--vp-c-bg-soft);
  border-radius: 12px;
  padding: 1.5rem;
  border: 1px solid var(--vp-c-divider);
  transition: all 0.3s ease;
}

.doc-grid > div:hover {
  border-color: var(--vp-c-brand);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  transform: translateY(-2px);
}

.quick-start {
  background: var(--vp-c-bg-soft);
  border-radius: 12px;
  padding: 2rem;
  margin: 2rem 0;
  border: 1px solid var(--vp-c-divider);
}

.quick-start h3 {
  margin-top: 1.5rem;
  color: var(--vp-c-brand);
}

.quick-start h3:first-of-type {
  margin-top: 0;
}

.link-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 1rem;
  margin: 2rem 0;
}

.link-grid a {
  display: inline-flex;
  align-items: center;
  padding: 0.75rem 1.5rem;
  background: var(--vp-c-bg-soft);
  border: 1px solid var(--vp-c-divider);
  border-radius: 8px;
  text-decoration: none;
  color: var(--vp-c-text-1);
  transition: all 0.3s ease;
  font-weight: 500;
  justify-content: center;
}

.link-grid a:hover {
  background: var(--vp-c-brand);
  color: white;
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.tech-stack {
  background: var(--vp-c-bg-soft);
  border-radius: 12px;
  padding: 2rem;
  margin: 2rem 0;
  border: 1px solid var(--vp-c-divider);
}

.tech-stack strong {
  color: var(--vp-c-brand);
}

.footer-note {
  background: var(--vp-c-bg-soft);
  border-radius: 8px;
  padding: 1.5rem;
  margin: 3rem 0 1rem;
  border: 1px solid var(--vp-c-divider);
}

.version-info {
  margin-bottom: 1rem;
  padding-bottom: 1rem;
  border-bottom: 1px solid var(--vp-c-divider);
}

@media (max-width: 768px) {
  .doc-grid,
  .link-grid {
    grid-template-columns: 1fr;
  }
  
  .comparison-grid,
  .advantage-card,
  .quick-start,
  .tech-stack {
    padding: 1rem;
  }
}
</style>

