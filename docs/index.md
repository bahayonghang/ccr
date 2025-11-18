---
layout: home

hero:
  name: "CCR"
  text: "Claude Code Configuration Switcher"
  tagline: 强大的 Claude Code 配置管理工具
  image:
    src: /logo.svg
    alt: CCR
  actions:
    - theme: brand
      text: 快速开始
      link: /guide/quick-start
    - theme: alt
      text: 核心命令
      link: /reference/commands/
    - theme: alt
      text: English
      link: /en/

features:
  - icon: 🚀
    title: 快速切换配置
    details: 直接操作 settings.json，配置立即生效，无需重启或手动设置环境变量

  - icon: 📊
    title: 精美表格界面
    details: 使用 comfy-table 展示配置信息，一目了然对比不同配置，支持颜色高亮和图标标识

  - icon: 🖥️
    title: 交互式 TUI
    details: 全功能终端界面，3 个标签页（配置/历史/系统），键盘导航，支持 Vim 快捷键

  - icon: 🔐
    title: 并发安全
    details: 文件锁定机制确保多进程安全，原子写入操作防止数据损坏

  - icon: 📝
    title: 完整审计追踪
    details: 记录所有操作历史，跟踪环境变量变更，自动脱敏敏感信息

  - icon: 💾
    title: 智能备份管理
    details: 自动保留最近10个备份，无需手动清理，支持从备份恢复，时间戳标记备份文件

  - icon: ✅
    title: 配置验证
    details: 自动验证配置完整性，检查必填字段，验证 URL 格式

  - icon: 🌐
    title: Web 界面
    details: 内置轻量级 Axum API 服务器（Legacy），提供 14 个 RESTful API 端点，智能端口绑定，支持 --no-browser 选项，适合编程访问；**新的 Web 界面推荐使用下方的 CCR UI 应用（`ccr ui`）**

  - icon: 📊
    title: 统计与成本分析
    details: 完整的使用统计和成本追踪系统，支持按时间/模型/项目/提供商多维度分析，提供 CLI 命令、Web API 和可视化仪表板

  - icon: 🎨
    title: CCR UI 应用
    details: 完整的 Vue.js 3 + Axum 应用（端口 5173/8081），提供可视化仪表板和统计分析，支持多 CLI 工具管理，首次使用自动从 GitHub 下载

  - icon: 🔄
    title: CCS 完全兼容
    details: 共享 ~/.ccs_config.toml 配置文件，命令行接口一致，可与 CCS 共存

  - icon: ⚡
    title: 高性能
    details: Rust 实现，性能卓越，快速响应，资源占用低
---

## 安装

### 快速安装(推荐)

使用 cargo 从 GitHub 直接安装：

```bash
cargo install --git https://github.com/bahayonghang/ccr ccr
```

### 从源码构建

```bash
# 克隆仓库
cd ccs/ccr

# 构建 release 版本
cargo build --release

# 安装到系统路径(可选)
cargo install --path .
```

## 快速使用

```bash
# 初始化配置文件
ccr init

# 查看所有配置
ccr list

# 切换配置
ccr switch anthropic

# 查看当前状态
ccr current

# 启动交互式 TUI
ccr tui

# 启动完整 CCR UI 应用（推荐 Web 界面）
ccr ui

# 启动轻量级 Legacy Web API 服务器（仅用于兼容性和编程访问）
ccr web

# Web API 不自动打开浏览器、并使用指定端口（适合远程/CI 场景）
ccr web --port 8080 --no-browser

# 查看操作历史
ccr history

# 启动交互式 TUI
ccr tui
```

## 文件结构

```
~/.ccs_config.toml          # 配置文件(与 CCS 共享)
~/.claude/settings.json     # Claude Code 设置文件
~/.claude/backups/          # 自动备份目录
~/.claude/ccr_history.json  # 操作历史日志
~/.claude/.locks/           # 文件锁目录
```

## 特性亮点

### 🎯 直接操作 Claude Code 设置

CCR 直接修改 `~/.claude/settings.json` 文件,无需手动配置环境变量,配置立即生效。

### 🔒 多进程安全保证

通过文件锁定机制确保并发操作安全,支持超时保护避免死锁,原子写入防止数据损坏。

### 📊 操作审计追踪

记录每次操作的完整信息：
- 操作 ID(UUID)
- 时间戳
- 操作者(系统用户名)
- 操作类型
- 环境变量变更(脱敏)
- 操作结果和备注

### 💡 智能备份管理

- 切换配置前自动备份
- **自动保留最近10个备份**，无需手动清理
- 备份文件包含时间戳和配置名称
- 支持清理更早期的备份释放空间
- 可从备份恢复配置

## 与 CCS 的区别

| 特性 | CCS (Shell) | CCR (Rust) |
|------|-------------|-----------|
| 配置切换 | ✅ | ✅ |
| 环境变量设置 | ✅ | ✅ |
| 直接写入 settings.json | ❌ | ✅ |
| 文件锁定机制 | ❌ | ✅ |
| 操作历史 | ❌ | ✅ |
| 自动备份 | ❌ | ✅ |
| 配置验证 | 基础 | 完整 |
| 并发安全 | ❌ | ✅ |
| Web 界面 | ❌ | ✅ |
| 性能 | 快 | 极快 |

## 开源协议

MIT License

## 贡献

欢迎提交 Issue 和 Pull Request！

## 相关链接

- [GitHub 仓库](https://github.com/bahayonghang/ccr)
- [问题反馈](https://github.com/bahayonghang/ccr/issues)
- [CCS 项目](https://github.com/bahayonghang/ccs)
