# 🚀 CCR - Claude Code Configuration Switcher

**Rust 驱动的 Claude Code 配置管理工具**

CCR 通过原子操作、文件锁、完整审计追踪和自动备份直接管理 Claude Code 的 `settings.json`。CCS 的 Rust 实现版,提供更高的可靠性和性能。

## ✨ 为什么选择 CCR？

| 特性 | 说明 |
|------|------|
| 🎯 **直接控制设置** | 直接写入 `~/.claude/settings.json` - 立即生效 |
| 📊 **精美表格界面** | 使用 comfy-table 展示配置信息，一目了然对比不同配置，支持颜色高亮和图标标识 |
| 🖥️ **交互式 TUI** | 全功能终端界面，3 个标签页（配置/历史/系统），键盘导航，实时反馈 |
| 🔒 **并发安全** | 文件锁 + 原子操作防止多进程并发损坏 |
| 📝 **完整审计追踪** | 每个操作都有日志记录（UUID、时间戳、操作者），敏感数据已掩码 |
| 💾 **自动备份** | 更改前自动备份，生成带时间戳的 `.bak` 文件 |
| ☁️ **云端同步** | 基于 WebDAV 的配置同步（支持坚果云、Nextcloud、ownCloud 等） |
| ✅ **配置验证** | 全面验证（URL、必填字段、格式） |
| 🔤 **配置优化** | 按字母顺序整理配置，保持顺序不被打乱 |
| 🌐 **Web 服务器** | 内置 Axum Web 服务器，提供 14 个 RESTful API 端点（配置、历史、备份、系统信息等） |
| 🖥️ **全栈 Web UI** | 基于 Next.js 16（React 19）+ Axum 的可视化管理界面 |
| 🏗️ **现代架构** | Service 层模式，模块化设计，95%+ 测试覆盖率 |
| ⚡ **智能更新** | 实时显示编译进度的自动更新功能 |
| 🔄 **CCS 兼容** | 共享 `~/.ccs_config.toml` - 与 Shell 版本无缝共存 |

## 📦 安装

首先需要安装 Rust 和 Cargo,然后执行以下命令：

**一行命令从 GitHub 安装：**

```bash
cargo install --git https://github.com/bahayonghang/ccr ccr
```

**从源码构建：**

```bash
git clone https://github.com/bahayonghang/ccr.git
cd ccr
cargo install --path .
```

**系统要求：** Rust 1.85+ (支持 edition 2024 特性)

## 🌐 CCR UI - 全栈 Web 应用

CCR UI 是一个现代化的 **Vue.js 3 + Axum** 全栈应用，用于 CCR 配置管理！

Vue.js 3 前端提供响应式的交互体验，结合 TypeScript 和 Tailwind 构建 UI；Axum 后端包装 CCR CLI，并额外提供 MCP 服务器、斜杠命令、智能体与插件的管理 API。

### 功能特性

- ⚛️ **Vue.js 3 前端**：Vue.js 3.5 配合 Composition API，使用 TypeScript 与 Tailwind CSS
- 🦀 **Axum 后端**：高性能 Rust 异步 Web 服务器
- 🖥️ **配置管理**：可视化配置切换和验证
- 💻 **命令执行器**：执行所有 13 个 CCR 命令，可视化输出
- 📊 **语法高亮**：终端风格输出，带颜色编码
- ⚡ **实时执行**：异步命令执行，带进度显示
- 🧩 **扩展控制台**：内置 MCP、斜杠命令、智能体与插件管理 API
- 🔄 **GitHub 自动下载**：首次使用自动下载 ccr-ui 到用户目录

### 快速启动

**方式一：直接使用 `ccr ui` 命令（推荐）**

```bash
# 首次使用 - 自动下载并启动
ccr ui

# 💬 提示: CCR UI 是一个完整的 Vue.js 3 + Axum Web 应用
#    可以从 GitHub 下载到用户目录:
#    /home/user/.ccr/ccr-ui/
#
# ❓ 是否立即从 GitHub 下载 CCR UI? [Y/n]: y
# 📦 克隆仓库: https://github.com/bahayonghang/ccr.git
# ⏳ 下载中 (这可能需要几分钟)...
# ✅ CCR UI 下载完成
#
# [自动检查依赖并启动...]
```

**三级优先级检测系统：**
1. **开发环境** - 检测当前/父目录的 `ccr-ui/`（适合开发者）
2. **用户目录** - 检测 `~/.ccr/ccr-ui/`（适合日常使用）
3. **GitHub 下载** - 自动提示从 GitHub 下载（首次使用）

**方式二：手动进入 ccr-ui 目录**

```bash
cd ccr-ui

# 一键命令 - 就这么简单！
just s    # 启动开发环境

# 第一次使用？一条命令搞定：
just quick-start    # 检查前置条件 + 安装 + 启动
```

**可用的简化命令：**
- `just s` - 启动开发环境（最常用！）
- `just i` - 安装依赖
- `just b` - 构建生产版本
- `just c` - 检查代码
- `just t` - 运行测试
- `just f` - 格式化代码

**不确定做什么？** 直接运行 `just` 查看帮助！

**📖 完整文档**：查看 `ccr-ui/START_HERE.md` 获取超简单指南，或 `ccr-ui/README.md` 查看完整文档。

**🎯 CLI vs TUI vs Web 服务器 vs CCR UI**：
- **CLI 工具**：适合脚本、自动化和快速操作（`ccr switch`、`ccr list` 等）
- **TUI** (`ccr tui`)：基于终端的交互式界面，支持键盘导航
- **Web 服务器** (`ccr web`)：内置轻量级 Axum API 服务器（8080 端口），用于编程访问
- **CCR UI** (`ccr ui`)：完整功能的 Vue.js 3 + Axum Web 应用，提供可视化仪表板（3000/8081 端口）

## 🚀 快速开始

**1️⃣ 初始化配置文件：**

```bash
ccr init  # 创建 ~/.ccs_config.toml 并包含示例
```

**2️⃣ 编辑配置：**

```toml
# ~/.ccs_config.toml
default_config = "anthropic"
current_config = "anthropic"

[anthropic]
description = "Anthropic 官方 API"
base_url = "https://api.anthropic.com"
auth_token = "sk-ant-your-api-key"
model = "claude-sonnet-4-5-20250929"

[anyrouter]
description = "AnyRouter 代理"
base_url = "https://api.anyrouter.ai/v1"
auth_token = "your-anyrouter-token"
model = "claude-sonnet-4-5-20250929"
```

**3️⃣ 使用 CCR：**

```bash
ccr list              # 📋 以表格形式列出所有配置（一目了然）
ccr switch anthropic  # 🔄 切换配置（表格展示变化，或简写: ccr anthropic）
ccr current           # 🔍 以表格显示当前配置和环境变量状态
ccr validate          # ✅ 验证所有配置
ccr history           # 📚 查看操作历史
ccr sync config       # ☁️ 配置 WebDAV 同步（交互式设置）
ccr sync status       # 📊 检查同步状态和远程文件
ccr sync push         # 🔼 上传配置到云端
ccr sync pull         # 🔽 从云端下载配置
ccr tui               # 🖥️ 启动交互式 TUI（推荐用于可视化管理！）
ccr web               # 🌐 启动轻量级 Web API 服务器（端口 8080）
ccr ui                # 🎨 启动完整 CCR UI 应用（Next.js + Actix，端口 3000/8081）
```

**4️⃣ 多平台使用:**

```bash
# 列出所有支持的平台
ccr platform list

# 切换到 Codex (GitHub Copilot)
ccr platform switch codex

# 初始化 Gemini 平台
ccr platform init gemini

# 向当前平台添加配置
ccr add

# 多平台工作流示例
ccr platform switch claude    # 使用 Claude Code
ccr switch my-claude-api      # 切换到特定的 Claude 配置
ccr platform switch codex     # 切换到 Codex
ccr switch my-github-token    # 切换到特定的 Codex 配置
```

**📖 详细的多平台设置和示例,请查看** [docs/examples/multi-platform-setup.md](docs/examples/multi-platform-setup.md)

## 📚 命令参考

| 命令 | 别名 | 说明 |
|------|------|------|
| `ccr init [--force]` | - | 🎬 从模板初始化配置 |
| `ccr list` | `ls` | 📊 以表格形式列出所有配置（状态、提供商、URL、模型、验证） |
| `ccr current` | `show`, `status` | 🔍 以双表格展示当前配置详情和环境变量状态 |
| `ccr switch <name>` | `<name>` | 🔄 切换配置（表格展示新配置和环境变量变化对比） |
| `ccr temp-token set <TOKEN> [选项]` | - | 🎯 设置临时Token覆盖（不修改toml文件） |
| `ccr temp-token show` | - | 👁️ 显示当前临时配置状态 |
| `ccr temp-token clear` | - | 🧹 清除临时配置覆盖 |
| `ccr validate` | `check` | ✅ 验证所有配置和设置 |
| `ccr optimize` | - | 🔤 按字母顺序优化配置文件结构 |
| `ccr history [-l N] [-t TYPE]` | - | 📚 显示操作历史（限制数量/按类型筛选） |
| `ccr web [-p PORT]` | - | 🌐 启动轻量级 Web API 服务器（默认 8080 端口） |
| `ccr ui [-p PORT] [--backend-port PORT]` | - | 🎨 启动完整 CCR UI 应用（Next.js + Actix，默认 3000/8081） |
| `ccr tui [--yolo]` | - | 🖥️ 启动交互式终端界面（可视化管理） |
| `ccr export [-o FILE] [--no-secrets]` | - | 📤 导出配置（包含/不含 API 密钥） |
| `ccr import FILE [--merge]` | - | 📥 导入配置（合并或替换） |
| `ccr clean [-d DAYS] [--dry-run]` | - | 🧹 清理旧备份（默认 7 天） |
| `ccr sync config` | - | ☁️ 配置 WebDAV 同步（交互式） |
| `ccr sync status` | - | 📊 检查同步状态和远程文件 |
| `ccr sync push [--force]` | - | 🔼 上传配置到云端 |
| `ccr sync pull [--force]` | - | 🔽 从云端下载配置 |
| `ccr update [--check]` | - | ⚡ 从 GitHub 更新 CCR（实时进度显示） |
| `ccr version` | `ver` | ℹ️ 显示版本和功能 |

**平台管理命令:**

| 命令 | 说明 | 示例 |
|------|------|------|
| `ccr platform list` | 🌟 列出所有平台及状态和当前配置 | 显示已启用平台、当前平台标记(▶)、配置数量 |
| `ccr platform current` | ▶️ 显示当前活动平台的详细信息 | 显示平台名称、当前配置、启用状态、最后使用时间 |
| `ccr platform switch <name>` | 🔄 切换到不同平台(自动更新设置路径) | `ccr platform switch codex` → 从 Claude 切换到 Codex |
| `ccr platform init <name>` | 🎬 初始化新平台及默认 profiles.toml | `ccr platform init gemini` → 创建 `~/.ccr/platforms/gemini/` |
| `ccr platform info <name>` | ℹ️ 显示平台详细信息 | `ccr platform info claude` → 显示所有 Claude 配置和设置 |

**切换操作流程：**
1. 📖 读取并验证目标配置
2. 💾 备份当前 settings.json
3. ✏️ 更新 ~/.claude/settings.json(原子写入 + 文件锁)
4. 📝 更新 current_config 标记
5. 📚 记录到历史(环境变量变化已掩码)

## 📁 文件与目录

**Legacy 模式(单平台):**
```
~/.ccs_config.toml           # 📝 配置文件(与 CCS 共享)
~/.claude/settings.json      # 🎯 Claude Code 设置(CCR 管理)
~/.claude/temp_override.json # 🎯 临时配置覆盖(temp-token命令)
~/.claude/backups/           # 💾 自动备份(带时间戳的 .bak 文件)
~/.claude/ccr_history.json   # 📚 操作审计日志
~/.claude/.locks/            # 🔒 文件锁(自动清理)
```

**Unified 模式(多平台):**
```
~/.ccr/                      # 🏠 CCR 根目录
  ├── config.toml            # 📝 平台配置注册表
  ├── backups/               # 💾 平台配置备份
  ├── claude/                # 🤖 Claude Code 平台
  │   ├── profiles.toml      # 📋 Claude 配置
  │   ├── settings.json      # ⚙️ Claude 设置
  │   ├── history.json       # 📚 Claude 操作历史
  │   └── backups/           # 💾 Claude 备份
  ├── codex/                 # 💻 Codex (GitHub Copilot) 平台
  │   ├── profiles.toml      # 📋 Codex 配置
  │   ├── settings.json      # ⚙️ Codex 设置
  │   ├── history.json       # 📚 Codex 操作历史
  │   └── backups/           # 💾 Codex 备份
  └── gemini/                # ✨ Gemini CLI 平台
      ├── profiles.toml      # 📋 Gemini 配置
      ├── settings.json      # ⚙️ Gemini 设置
      ├── history.json       # 📚 Gemini 操作历史
      └── backups/           # 💾 Gemini 备份
```

## 🔧 核心功能

### 🌍 环境变量

CCR 在 `settings.json` 中管理这些变量：
- `ANTHROPIC_BASE_URL` - API 端点
- `ANTHROPIC_AUTH_TOKEN` - 认证令牌(显示/日志中自动掩码)
- `ANTHROPIC_MODEL` - 默认模型
- `ANTHROPIC_SMALL_FAST_MODEL` - 快速模型(可选)

### 🎯 临时Token覆盖

需要临时测试免费Token吗？CCR提供临时配置覆盖功能，无需修改永久的 `~/.ccs_config.toml` 文件：

```bash
# 设置临时token（一次性使用，switch后自动清除）
ccr temp-token set sk-free-test-xxx

# 可选：覆盖更多字段
ccr temp-token set sk-xxx \
  --base-url https://api.temp.com \
  --model claude-opus-4

# 查看当前临时配置
ccr temp-token show

# 应用临时配置（会自动应用并清除）
ccr switch duck

# 下次 switch 将使用永久配置
ccr switch duck
```

**功能特性：**
- 🔒 独立存储 (`~/.claude/temp_override.json`)
- 🎯 一次性使用（应用后自动清除）
- 🎯 部分字段覆盖（只覆盖token，或token+url等）
- 🔄 优先级高于永久配置
- 🧹 应用后自动清理
- 👁️ 敏感信息自动脱敏显示

### 📚 历史与审计

每个操作都会记录：
- UUID + 时间戳 + 系统用户名
- 操作类型(switch/backup/restore/validate/update)
- 环境变量变化(已掩码)
- 源/目标配置 + 备份路径
- 结果(成功/失败/警告)

### 🌟 多平台配置

CCR 支持从单一工具管理多个 AI CLI 平台的配置:

**支持的平台:**

| 平台 | 状态 | 说明 | 设置路径 |
|------|------|------|----------|
| **Claude Code** | ✅ 已完整实现 | Anthropic 官方 CLI | `~/.claude/settings.json` |
| **Codex** | ✅ 已完整实现 | GitHub Copilot CLI | `~/.codex/settings.json` |
| **Gemini CLI** | ✅ 已完整实现 | Google Gemini CLI | `~/.gemini/settings.json` |
| **Qwen CLI** | 🚧 计划中 | 阿里巴巴通义千问 CLI | `~/.qwen/settings.json` |
| **iFlow CLI** | 🚧 计划中 | iFlow AI CLI | `~/.iflow/settings.json` |

**配置模式:**

- **Legacy 模式**: 单平台(向后兼容 CCS)
  - 使用 `~/.ccs_config.toml`
  - 仅管理 Claude Code
  - 与 Shell 版本的 CCS 兼容

- **Unified 模式**: 多平台(v1.4+ 新功能)
  - 使用 `~/.ccr/config.toml` 作为平台注册表
  - 每个平台独立的 `~/.ccr/{platform}/` 目录
  - 平台特定的配置、历史和备份
  - 平台之间完全隔离

**平台特性:**

- ✅ **平台隔离**: 每个平台拥有独立的配置、历史和备份
- ✅ **平台切换**: 使用 `ccr platform switch` 在平台间切换
- ✅ **配置管理**: 独立管理平台特定的配置
- ✅ **平台检测**: 根据目录结构自动检测 Unified/Legacy 模式
- ✅ **统一历史**: 在集中式日志中跟踪所有平台的操作
- ✅ **并发安全**: 文件锁防止多平台操作时的数据损坏
- ✅ **自动迁移**: 轻松从 Legacy 迁移到 Unified 模式

**平台检测逻辑:**

CCR 自动检测使用哪种配置模式:

1. **检查 `CCR_ROOT` 环境变量** → 如果已设置,使用 Unified 模式
2. **检查 `~/.ccr/config.toml` 是否存在** → 如果存在,使用 Unified 模式
3. **回退到 Legacy 模式** → 使用 `~/.ccs_config.toml`(向后兼容)

**从 Legacy 迁移到 Unified:**

```bash
# 检查是否应该迁移
ccr migrate --check

# 迁移所有配置到 Unified 模式
ccr migrate

# 迁移特定平台
ccr migrate --platform claude
```

**示例工作流:**

```bash
# 初始化多个平台
ccr platform init claude
ccr platform init codex
ccr platform init gemini

# 使用 Claude Code
ccr platform switch claude
ccr add                          # 添加 Claude 配置
ccr switch my-anthropic-api      # 使用特定配置

# 使用 GitHub Copilot
ccr platform switch codex
ccr add                          # 添加 Codex 配置
ccr switch my-github-token       # 使用特定配置

# 使用 Gemini CLI
ccr platform switch gemini
ccr add                          # 添加 Gemini 配置
ccr switch my-google-api         # 使用特定配置

# 查看所有平台
ccr platform list
```


### 🖥️ TUI - 终端用户界面

CCR 提供交互式终端界面，用于可视化配置管理。启动命令：

```bash
ccr tui [--yolo]  # --yolo: 启用 YOLO 模式（跳过确认）
```

**功能特性：**
- **🖥️ 四个标签页**：
  - **配置页** 📋：浏览和管理所有配置
  - **历史页** 📜：查看带时间戳的操作历史
  - **同步页** ☁️：WebDAV 同步状态和远程文件检查
  - **系统页** ⚙️：显示系统信息和文件路径

- **⌨️ 键盘快捷键**：
  - `1-4` / `Tab` / `Shift+Tab`：切换标签页
  - `↑↓` / `j`/`k`：导航列表（支持 Vim 风格）
  - `Enter`：切换到选中的配置
  - `d`：删除选中的配置（需要 YOLO 模式）
  - `y` / `Y`：切换 YOLO 模式
  - `q` / `Ctrl+C`：退出 TUI

- **🎨 视觉特性**：
  - 彩色编码配置列表（当前=绿色，默认=青色）
  - 实时状态消息（成功/错误）
  - 带结果指示器的操作历史（✅❌⚠️）
  - 系统信息显示（主机名、操作系统、路径、版本）

- **⚡ YOLO 模式**：
  - 跳过所有确认提示
  - TUI 中删除操作必需
  - 可使用 `Y` 键切换或启动时使用 `--yolo` 标志
  - 页脚显示状态（🔴 YOLO / 🟢 SAFE）

**示例工作流：**
```bash
ccr tui              # 启动 TUI
# 按 '1' → 浏览配置 → Enter 切换
# 按 '2' → 查看历史
# 按 '3' → 检查同步状态（P/L/S 在 CLI 中执行 push/pull/status）
# 按 '4' → 检查系统信息
# 按 'Y' → 启用 YOLO 模式 → 'd' 删除配置
# 按 'q' → 退出
```

### ☁️ 云端同步（WebDAV）

CCR 支持基于 WebDAV 的配置同步，方便多设备管理。

**支持的服务：**
- 🥜 **坚果云** - 国内用户推荐（免费套餐可用）
- 📦 **Nextcloud / ownCloud** - 自建或托管
- 🌐 **任何标准 WebDAV 服务器**

**设置指南：**

1. **配置 WebDAV 连接：**
```bash
ccr sync config
# 交互式提示：
# - WebDAV 服务器地址（默认: https://dav.jianguoyun.com/dav/）
# - 用户名/邮箱
# - 密码/应用密码（坚果云：账户信息 → 安全选项 → 添加应用 → 生成密码）
# - 远程文件路径（默认: /ccr/.ccs_config.toml）
# - 自动进行连接测试
```

2. **检查同步状态：**
```bash
ccr sync status
# 显示：
# - 同步配置（服务器、用户名、远程路径）
# - 自动同步状态
# - 远程文件存在检查
```

3. **上传配置到云端（首次）：**
```bash
ccr sync push
# 如果远程文件存在会提示确认
# 使用 --force 跳过确认
```

4. **从云端下载配置：**
```bash
ccr sync pull
# 覆盖前会备份本地配置
# 使用 --force 跳过确认
```

**配置方式：**

同步设置存储在 `~/.ccs_config.toml` 中：
```toml
[settings.sync]
enabled = true
webdav_url = "https://dav.jianguoyun.com/dav/"
username = "user@example.com"
password = "your-app-password"
remote_path = "/ccr/.ccs_config.toml"
auto_sync = false  # 尚未实现
```

**使用场景：**
- 📱 多台设备间同步配置
- 💼 团队协作共享配置
- 🔄 备份配置到云存储
- 🚀 新设备快速设置

**安全注意：**
- ✅ 密码存储在本地配置文件中
- ✅ 使用应用密码而非账户密码（坚果云）
- ✅ 确保正确的文件权限：`chmod 600 ~/.ccs_config.toml`
- ⚠️ 远程文件未由 CCR 加密（依赖 WebDAV 服务器安全性）

### 🌐 Web API

RESTful 端点(运行 `ccr web`)：
当前内置服务器提供 14 个端点，覆盖配置管理、备份生命周期与系统监控。
- `GET /api/configs` - 列出所有配置
- `POST /api/switch` - 切换指定配置
- `POST /api/config` - 新增配置节
- `POST /api/config/{name}` - 更新配置节
- `DELETE /api/config/{name}` - 删除配置节
- `GET /api/history` - 查看审计历史
- `POST /api/validate` - 验证配置与设置文件
- `POST /api/clean` - 清理备份
- `GET /api/settings` - 获取 Claude Code 设置快照
- `GET /api/settings/backups` - 列出设置备份
- `POST /api/settings/restore` - 恢复设置备份
- `POST /api/export` - 导出配置文件
- `POST /api/import` - 导入配置文件
- `GET /api/system` - 查看缓存的系统信息

### 🐛 调试

```bash
export CCR_LOG_LEVEL=debug  # trace|debug|info|warn|error
ccr switch anthropic        # 查看详细日志
```

## 🆚 CCR vs CCS

| 特性 | CCS (Shell) | CCR (Rust) |
|------|:-----------:|:----------:|
| 配置切换 | ✅ | ✅ |
| 直接写入 settings.json | ❌ | ✅ |
| 文件锁 | ❌ | ✅ |
| 审计历史 | ❌ | ✅ |
| 自动备份 | ❌ | ✅ |
| 配置验证 | 基础 | 完整 |
| Web 界面 | ❌ | ✅ |
| 性能 | 快 | 极快 |

**💡 完全兼容** - 共享 `~/.ccs_config.toml`,可以无缝共存和切换。

## 🛠️ 开发

**项目结构：**
```
src/
├── main.rs           # 🚀 CLI 入口
├── lib.rs            # 📚 库入口
├── commands/         # 🎯 CLI 层（13 个命令）
├── web/              # 🌐 Web 层（Axum 服务器 + API）
├── services/         # 🎯 Service 层（业务逻辑）
├── managers/         # 📁 Manager 层（数据访问）
│   ├── config.rs     # ⚙️ 配置管理
│   ├── settings.rs   # ⭐ 设置管理
│   └── history.rs    # 📚 审计追踪
├── core/             # 🏗️ Core 层（基础设施）
│   ├── error.rs      # ⚠️ 错误类型 + 退出码
│   ├── lock.rs       # 🔒 文件锁
│   ├── logging.rs    # 🎨 彩色输出
│   └── ...           # 更多核心模块
└── utils/            # 🛠️ 工具（掩码、验证）

ccr-ui/               # 🌐 全栈 Web 应用
├── backend/          # 🦀 Axum 服务器
│   ├── src/
│   │   ├── main.rs               # 服务器入口
│   │   ├── executor/             # CCR CLI 子进程执行器
│   │   ├── handlers/             # API 路由处理器（配置、命令、MCP 等）
│   │   ├── models.rs             # 请求/响应类型
│   │   ├── settings_manager.rs   # Claude 设置文件原子读写
│   │   ├── plugins_manager.rs    # 插件仓库管理
│   │   ├── claude_config_manager.rs # 配置文件辅助工具
│   │   └── markdown_manager.rs   # Markdown 知识库管理
│   └── Cargo.toml
└── frontend/         # ⚛️ Vue.js 3 配合 Vite
    ├── src/
    │   ├── views/            # 页面视图（Dashboard、Configs、Commands 等）
    │   ├── components/       # 可复用 UI 组件
    │   ├── router/           # Vue Router 配置
    │   └── store/            # Pinia 状态管理
    ├── package.json
    └── vite.config.ts
```

**命令：**
```bash
# 开发工作流（使用 justfile）
just dev              # 快速检查 + 测试
just watch            # 文件变化时自动重建
just ci               # 完整 CI 流程

# 或直接使用 cargo
cargo test            # 🧪 运行测试
cargo clippy          # 🔍 代码检查
cargo fmt             # 💅 格式化
cargo build --release # 🏗️ 生产构建
```

## 🏗️ 架构

CCR v1.1.5 采用**严格的分层架构**，职责清晰分离：

```
CLI/Web 层 → Services 层 → Managers 层 → Core/Utils 层
```

**核心组件：**
- **Service 层**: 4 个服务（Config、Settings、History、Backup）- 26 个方法
- **Manager 层**: 3 个管理器（Config、Settings、History）- 数据访问与文件操作
- **Web 模块**: 基于 Axum 的服务器，提供 14 个 RESTful API 端点
- **Core 基础设施**: 原子写入器、文件锁、错误处理、日志记录
- **测试覆盖**: 95%+ 全面测试套件

**设计模式：**
- 原子文件操作（临时文件 + 重命名）
- 通过文件锁实现多进程安全
- 完整的 UUID 追踪审计日志
- 破坏性操作前自动备份

详细架构文档见 [ARCHITECTURE.md](ARCHITECTURE.md)。

## 🐛 故障排除

| 问题 | 解决方法 |
|------|----------|
| 配置文件不存在 | 运行 `ccr init` 创建 `~/.ccs_config.toml` |
| 锁超时 | 检查僵死进程: `ps aux \| grep ccr`<br>清理锁文件: `rm -rf ~/.claude/.locks/*` |
| 权限被拒绝 | 修复权限:<br>`chmod 600 ~/.claude/settings.json`<br>`chmod 644 ~/.ccs_config.toml` |
| 设置文件不存在 | 首次切换时自动创建: `ccr switch <config>` |

## 📄 许可证与贡献

- **许可证：** MIT
- **Issues & PRs：** 欢迎！🤝
- **GitHub：** https://github.com/bahayonghang/ccr
- **状态：** 活跃开发中 - 生产环境使用前请充分测试

---

用 💙 在 Rust 中构建 | [CCS 项目](https://github.com/bahayonghang/ccs)的一部分
