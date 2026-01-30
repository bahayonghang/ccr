# 功能概览（v3.9.4）

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

## 🛠️ 技能 (Skills) 管理 (v3.8+)
管理 AI 技能库，扩展 AI 能力。

**功能**：
- **技能市场**：浏览和安装官方/社区技能
- **我的技能**：管理已安装的技能
- **技能搜索**：快速查找所需技能
- **自动更新**：支持技能版本检查和更新
- **本地仓库**：支持加载本地技能目录


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

## 📅 签到管理 (v3.7+)

管理 AI 中转站的自动签到，追踪账号余额和用量统计。灵感来源于 [NeuraDock](https://github.com/neuraledge/neuradock) 项目。

> 📖 **详细文档**：[签到功能详细指南](./checkin-detailed.md)

### 核心功能

#### 🏪 提供商管理
- **内置中转站**：预置 AnyRouter、AgentRouter、CodeRouter 三大主流中转站配置
  - 一键添加，无需手动配置域名和 API 路径
  - 内置签到、查询、用量统计逻辑
- **自定义提供商**：支持添加任意中转站
  - 灵活配置 API 端点和认证方式
  - 自定义签到和查询逻辑

#### 👤 账号管理
- **多账号支持**：每个提供商可以管理多个账号
- **安全存储**：使用 **AES-256-GCM** 加密存储敏感信息（密码、Token）
- **批量操作**：支持批量添加、启用、禁用账号
- **账号状态**：实时显示账号状态（活跃/已禁用/异常）

#### ✅ 签到功能
- **自动签到**：一键批量签到所有启用的账号
- **智能签到**：
  - AgentRouter 在查询用户信息时自动签到
  - 自动跳过不支持签到的提供商
- **WAF 绕过**：AnyRouter 使用 **chromiumoxide** 自动化浏览器绕过 Cloudflare 防火墙
- **签到记录**：完整记录每次签到的时间、状态和奖励

#### 💰 余额查询
- **实时查询**：查询所有账号的剩余额度
- **余额统计**：
  - 总余额汇总
  - 按提供商分组显示
  - 余额变化趋势
- **低余额提醒**：余额不足时自动提醒

#### 📊 数据管理
- **签到历史**：查看所有签到记录（时间、状态、奖励）
- **导入导出**：
  - 导出配置为 JSON 文件
  - 从 JSON 文件导入配置
  - 支持批量迁移账号

### 技术实现

#### 数据存储
```
~/.ccr/checkin/
├── checkin.db              # SQLite 数据库（提供商、账号、记录）
├── waf_cookies/            # WAF Cookie 缓存（绕过 Cloudflare）
│   └── <provider_id>.json
└── exports/                # 导出文件存储目录
    └── checkin_export_<timestamp>.json
```

**数据库表结构**：
- `providers` - 提供商信息（ID、名称、域名、API 路径）
- `accounts` - 账号信息（ID、提供商ID、用户名、加密密码、状态）
- `checkin_records` - 签到记录（ID、账号ID、时间、状态、奖励）

#### 安全机制
- **AES-256-GCM 加密**：密码和 Token 使用系统密钥加密存储
- **密钥派生**：使用硬件和系统信息生成唯一密钥
- **传输安全**：所有 API 请求使用 HTTPS + Bearer Token 认证

#### WAF 绕过技术
- **chromiumoxide**：基于 Chrome DevTools Protocol 的浏览器自动化
- **Cookie 缓存**：保存通过验证的 Cookie，避免重复绕过
- **智能重试**：Cookie 失效时自动重新绕过并更新缓存

### 内置中转站

| 中转站 | 域名 | 签到支持 | 余额查询 | 特性 |
|--------|------|----------|----------|------|
| 🌐 **AnyRouter** | anyrouter.top | ✅ 支持 | ✅ 支持 | 需要 WAF 绕过（Cloudflare） |
| 🤖 **AgentRouter** | agentrouter.org | ⚠️ 自动签到 | ✅ 支持 | 查询用户信息时自动签到 |
| 💻 **CodeRouter** | api.codemirror.codes | ❌ 不支持 | ✅ 支持 | 无签到功能，仅余额查询 |

### 代理支持

签到服务默认使用系统代理（用于访问国外中转站），通过以下环境变量配置：

```bash
# Socks5 代理（推荐）
export ALL_PROXY=socks5://127.0.0.1:7890

# 或 HTTP/HTTPS 代理
export HTTPS_PROXY=http://127.0.0.1:7890
export HTTP_PROXY=http://127.0.0.1:7890
```

**后端启动时会自动使用代理**，无需额外配置。

### 常见使用场景

#### 场景 1：新用户快速开始
1. 点击"添加内置提供商" → 选择 AnyRouter
2. 输入账号密码 → 启用账号
3. 点击"批量签到" → 自动完成签到

#### 场景 2：管理多个账号
1. 添加多个提供商（AnyRouter、AgentRouter）
2. 为每个提供商添加多个账号
3. 使用"批量签到"一键签到所有账号
4. 查看"余额统计"了解总余额

#### 场景 3：配置迁移
1. 在旧设备导出配置：导出 → 保存 JSON 文件
2. 在新设备导入配置：导入 → 选择 JSON 文件
3. 自动恢复所有提供商和账号（密码已加密）

### 故障排查

**常见问题**：
- **AnyRouter 签到失败**：检查代理配置，确保能访问 anyrouter.top
- **WAF 验证超时**：增加超时时间（默认 60s），或重试
- **余额查询失败**：检查账号状态，确认 Token 有效
- **导入失败**：检查 JSON 格式，确保符合导出格式

更多问题请参阅 [签到功能详细指南](./checkin-detailed.md) 的 FAQ 部分。


管理 AI 中转站的自动签到，追踪账号余额，和 NeuraDock 项目类似。

**核心功能**：
- 🏪 **内置中转站**：预置 AnyRouter、AgentRouter、CodeRouter 配置，一键添加
- 📝 **自定义提供商**：支持添加任意中转站配置
- 👤 **多账号管理**：每个提供商支持多个账号
- ✅ **自动签到**：一键批量签到所有启用的账号
- 💰 **余额查询**：查询账号剩余额度
- 📊 **签到记录**：查看历史签到状态
- 📦 **导入导出**：配置备份和迁移

**内置中转站**：

| 中转站 | 域名 | 签到支持 | 特性 |
|--------|------|----------|------|
| 🌐 AnyRouter | anyrouter.top | ✅ 支持 | 需要 WAF 绕过 |
| 🤖 AgentRouter | agentrouter.org | ⚠️ 自动签到 | 查询用户信息时自动签到 |
| 💻 CodeRouter | api.codemirror.codes | ❌ 不支持 | 无签到功能 |

**代理支持**：
签到服务默认使用系统代理，通过以下环境变量配置：
```bash
# Socks5 代理
export ALL_PROXY=socks5://127.0.0.1:7890

# 或 HTTP 代理
export HTTPS_PROXY=http://127.0.0.1:7890
export HTTP_PROXY=http://127.0.0.1:7890
```

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
