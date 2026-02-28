# CCR (Claude Code Configuration Switcher)

**Rust 编写的高性能多平台配置管理工具。**  
统一管理 **Claude Code**、**Codex**、**Gemini**、**Qwen** 等多种 AI CLI 工具的配置。

![Version](https://img.shields.io/badge/version-3.17.3-blue.svg) ![License](https://img.shields.io/badge/license-MIT-green.svg) ![Build](https://img.shields.io/badge/build-passing-brightgreen.svg)

---

## ✨ 核心特性

- **多平台支持**：统一管理 Claude、Codex、Gemini、Qwen 和 iFlow。每个平台拥有独立的配置档案、历史记录和备份。
- **企业级安全**：支持原子写入、文件锁 (`fs4`)、完整审计日志，且在每次修改前自动备份。
- **多端接口**：
  - **CLI**：功能强大的命令行接口。
  - **TUI**：交互式终端配置选择器，支持Tab切换。
  - **Web API**：内置 Axum 服务，便于外部集成。
  - **Desktop UI**：基于 Vue 3 + Tauri 构建的全栈桌面应用。
- **智能同步**：基于 WebDAV 的多文件夹同步（`web` 特性），保持多机配置一致。
- **隐私保护**：输出时自动掩码 API Key 等敏感数据。

## 📦 安装

### 一行命令安装
```bash
cargo install --git https://github.com/bahayonghang/ccr ccr
```

### 从 dev 分支安装（获取最新功能）
```bash
cargo install --git https://github.com/bahayonghang/ccr --branch dev ccr
```

### 源码安装
```bash
git clone https://github.com/bahayonghang/ccr.git
cd ccr
cargo install --path .
```

### 构建要求
- **Rust**: 1.88+ (2024 版本)
- **Node.js**: 18+（默认启用 `web` 特性时，安装/更新需用于构建内嵌 Web 资源）

## 🚀 快速开始

### 1. 初始化
在 `~/.ccr/` 下初始化统一配置结构：
```bash
ccr init
```

### 2. 选择平台
切换到你需要的平台（默认为 `claude`）：
```bash
# 列出可用平台
ccr platform list

# 切换到 Gemini（示例）
ccr platform switch gemini
```

### 3. 管理配置
```bash
# 交互式向导添加新配置
ccr add

# 列出当前平台的所有配置
ccr list

# 查看当前配置状态
ccr status

# 切换到指定配置
ccr switch my-work-config

# 快捷切换（省略 switch）
ccr my-work-config

```


### 4. 交互式 TUI
启动终端配置选择器：
```bash
# 直接运行 ccr 即可
ccr
```

**键盘快捷键：**
| 按键 | 功能 |
|------|------|
| `Tab` | 切换 Claude/Codex 平台 |
| `←` / `→` | 翻页（配置超过 20 个时） |
| `↑` / `↓` / `j` / `k` | 选择配置 |
| `Enter` | 应用选中的配置并退出 |
| `Space` | 应用选中的配置（保持 TUI） |
| `q` / `Esc` | 退出 |

**功能特性：**
- 双 Tab 界面：Claude Code 和 Codex CLI
- 分页支持（每页 20 个配置）
- 底部实时状态消息
- 平台专属配色（Claude 橙色，Codex 紫色）

## 🖥️ CCR UI

提供现代化的图形界面来管理您的配置。

```bash
# 启动 UI（自动检测工作区或下载发布版）
ccr ui

# 指定自定义端口
ccr ui -p 3000
```

## 🔐 Codex 多账号管理

CCR 为 Codex CLI 提供强大的多账号管理功能，让您可以轻松在不同的 GitHub 账号之间切换。

### 基础命令

```bash
# 保存当前登录为命名账号
ccr codex auth save work

# 保存时添加描述
ccr codex auth save personal -d "个人 GitHub 账号"

# 保存时设置过期时间
ccr codex auth save temp --expires-at 2026-02-01T00:00:00Z

# 强制覆盖已存在的账号
ccr codex auth save work --force

# 列出所有已保存的账号
ccr codex auth list

# 切换到指定账号
ccr codex auth switch work

# 显示当前账号信息
ccr codex auth current

# 删除账号
ccr codex auth delete old-account

# 删除时跳过确认
ccr codex auth delete old-account --force
```

### 导出与导入

```bash
# 导出所有账号到 Downloads 文件夹
ccr codex auth export

# 导出时不包含敏感数据（Token）
ccr codex auth export --no-secrets

# 从文件导入账号（交互式）
ccr codex auth import

# 使用替换模式导入（覆盖同名账号）
ccr codex auth import --replace

# 使用强制模式导入（合并模式下覆盖已存在账号）
ccr codex auth import --force
```

**导入模式说明：**
- **合并模式（默认）**：跳过已存在的账号，只添加新账号
- **合并 + --force**：强制覆盖已存在的账号
- **替换模式**：始终覆盖同名账号

### 交互式 TUI

启动 Codex 账号管理界面：
```bash
ccr codex
```

**功能特性：**
- 可视化账号列表，带 Token 新鲜度指示器
- 🟢 新鲜 (<1天) | 🟡 陈旧 (1-7天) | 🔴 过期 (>7天)
- 切换前进程检测警告
- 邮箱脱敏保护隐私（如 `use***@example.com`）

## 🔄 自动更新

CCR 支持从 GitHub 自动更新到最新版本。

> 注意：`ccr update` 会编译默认 `web` 特性，需要 Node.js 18+ 和 npm。

```bash
# 从 main 分支更新（稳定版）
ccr update

# 从 dev 分支更新（最新功能）
ccr update dev

# 仅检查更新，不实际安装
ccr update --check

# 检查 dev 分支的更新
ccr update dev --check
```

| 命令 | 说明 |
|------|------|
| `ccr update` | 从 `main` 分支更新到最新稳定版 |
| `ccr update dev` | 从 `dev` 分支更新，获取最新功能 |
| `ccr update --check` | 预览更新命令，不实际执行 |


## 🛠️ 开发指南

本项目使用 `just` 进行任务自动化管理。

```bash
# 构建所有特性
just build

# 运行测试
just test

# 代码检查
just check
just lint
```

## 📂 项目结构
```text
ccr/
├── src/            # 核心 Rust 逻辑 (CLI, TUI, Web API)
├── ccr-ui/         # 全栈 Web/桌面应用 (Vue 3 + Tauri)
├── tests/          # 集成测试
└── justfile        # 任务运行配置
```

## 📄 许可证
MIT License
