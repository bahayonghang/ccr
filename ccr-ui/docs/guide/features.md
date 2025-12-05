# 功能概览（v3.6.2）

CCR UI 为 CCR 提供完整的图形化管理能力，覆盖 CLI 全部功能并可视化操作。

## 🎯 仪表盘 (Dashboard)
系统整体状态概览，包含快捷操作入口和系统健康信息。

**主要功能**：
- 服务健康状态检查
- 快捷操作面板
- 系统信息和性能指标
- 版本信息和更新检查
- 多平台切换入口

## ⚙️ 配置管理 (Configs)
可视化查看、切换、验证、导入导出所有配置文件。

**核心能力**：
- 📋 **配置列表**：查看所有配置文件详细信息
- 🔄 **配置切换**：一键切换激活的配置
- ✅ **配置验证**：验证配置文件格式和完整性
- 📤 **导入/导出**：导出配置或导入外部配置
- 📝 **历史记录**：查看所有配置操作的历史
- 💾 **备份管理**：创建、查看、恢复配置备份
- 🛠️ **配置清理**：清理旧的备份和临时文件

## 💬 命令执行 (Commands)
图形化运行所有 CCR 命令，实时查看输出结果。

**支持命令**：
- `ccr init` - 初始化配置
- `ccr list` - 列出所有配置
- `ccr current` - 显示当前配置
- `ccr switch` - 切换配置
- `ccr add` - 添加新配置
- `ccr delete` - 删除配置
- `ccr validate` - 验证配置
- `ccr history` - 查看历史
- `ccr config` - 查看原始配置
- `ccr clear` - 清理 CCR 写入的配置

## 🔌 MCP 服务器管理 (v3.5+)
管理 Claude Code 的 Model Context Protocol (MCP) 服务器。

**功能**：
- 查看所有 MCP 服务器状态
- 添加新的 MCP 服务器
- 编辑服务器配置
- 启用/禁用服务器（无需重启）
- 删除服务器配置
- 实时查看连接状态

## 🤖 Agents 管理 (v3.5+)
管理 AI Agents 定义和配置。

**核心功能**：
- 列出所有 Agent 定义
- 创建新的 Agent
- 编辑 Agent 名称、描述、指令
- 启用/禁用 Agent
- 删除 Agent
- Agent 配置导入/导出

## 💬 Slash Commands 管理 (v3.5+)
自定义 Slash 命令管理。

**功能**：
- 查看所有 Slash 命令
- 创建自定义命令
- 编辑命令名称、描述、执行命令
- 启用/禁用命令
- 删除命令
- 命令模板管理

## 🧩 Plugins 管理 (v3.5+)
插件安装、配置和管理。

**支持操作**：
- 浏览已安装插件
- 安装新插件
- 配置插件参数
- 启用/禁用插件（无需重启）
- 卸载插件
- 插件版本管理

## ☁️ WebDAV 多目录同步 (v2.5+)
多文件夹独立同步管理，支持批量操作。

**核心功能**：
- 📁 **目录注册**：注册多个同步目录（configs、.claude/、.gemini/ 等）
- ✓ **启用/禁用**：灵活控制每个目录的同步状态
- ⬆️ **单目录同步**：对特定目录执行 push/pull/status
- ⬆️⬇️ **批量同步**：一键同步所有启用的目录
- 🔄 **自动过滤**：自动排除备份和锁文件
- 📊 **同步状态**：实时查看同步进度和状态
- ⚡ **快速操作**：支持 force 模式强制同步

**配置示例**：
```toml
[webdav]
url = "https://dav.jianguoyun.com/dav/"
username = "user@example.com"
password = "app-password"

[[folders]]
name = "claude"
local_path = "~/.claude"
remote_path = "/ccr/claude"
enabled = true

[[folders]]
name = "gemini"
local_path = "~/.gemini"
remote_path = "/ccr/gemini"
enabled = true
exclude_patterns = ["*.log", "cache/"]
```

## 🔧 转换器 (Converter)
不同平台配置格式互转。

**支持转换**：
- Claude Code ↔ Codex
- Claude Code ↔ Gemini CLI
- Claude Code ↔ Qwen
- 自定义格式转换

## 🌐 多平台支持 (v3.6+)
统一管理多个 AI CLI 平台的配置。

**支持平台**：
- ✅ **Claude Code** - Anthropic 官方 CLI
- ✅ **Codex** - GitHub Copilot CLI
- ✅ **Gemini CLI** - Google Gemini
- 🚧 **Qwen** - 阿里通义千问（开发中）
- 🚧 **iFlow** - iFlow CLI（开发中）

**平台专属管理**：
- 独立配置文件
- 独立历史记录
- 独立备份存储
- 独立 MCP/Agent/Plugin 管理

## 💻 系统信息监控
实时监控系统状态和资源使用情况。

**监控指标**：
- CPU 使用率
- 内存使用情况
- 磁盘空间
- 运行时间
- 系统健康状态

## 🖥️ 多运行模式
支持 Web 模式和 Tauri 桌面模式。

### Web 模式
- 基于 HTTP API 通信
- 浏览器访问
- 跨平台支持
- 默认端口：前端 3000，后端 8081

### Tauri 桌面模式 (v3.4+)
- 本地应用体验
- 使用 Tauri invoke（Rust 直接调用）
- 自动切换调用方式
- 支持离线使用

**系统依赖**：
- Linux: `libwebkit2gtk-4.0-dev build-essential`
- macOS: Xcode Command Line Tools
- Windows: Visual Studio C++ Build Tools

## 📦 生产构建与部署
支持多种部署方式。

**构建命令**：
```bash
# Web 模式
cd ccr-ui
just build            # 构建后端 + 前端
just run-prod         # 使用构建产物运行

# Tauri 模式
just tauri-build      # 打包桌面安装包
```

**产出物**：
- 后端：`backend/target/release/ccr-ui-backend`
- 前端：`frontend/dist/`

## 📚 开发者工具
内置开发辅助工具。

- **just 任务**：快速启动开发环境
- **热重载**：前端 HMR 支持
- **API 文档**：自动生成 API 文档
- **日志系统**：结构化日志，支持文件轮转
- **错误处理**：详细的错误信息和堆栈追踪
