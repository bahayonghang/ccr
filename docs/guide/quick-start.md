# 快速开始

CCR 提供了简单而强大的配置管理功能。本指南将帮助你快速上手。

## 安装

### 方式一：快速安装(推荐)

使用 cargo 从 GitHub 直接安装：

```bash
cargo install --git https://github.com/bahayonghang/ccr ccr
```

安装完成后,`ccr` 命令将可在你的 PATH 中使用。

### 方式二：从源码构建

```bash
# 克隆仓库
cd ccs/ccr

# 构建 release 版本
cargo build --release

# 安装到系统路径(可选)
cargo install --path .
```

## 初次使用

### 1. 初始化配置结构

首次使用 CCR 时，需要初始化配置文件：

```bash
ccr init
```

这将创建 **Unified Mode** 多平台配置结构：

```
~/.ccr/
├── config.toml                 # 平台注册表
├── platforms/
│   └── claude/                 # Claude 平台（默认）
│       ├── profiles.toml       # 将在首次使用时创建
│       ├── history/            # 历史记录目录
│       └── backups/            # 备份目录
├── history/                    # 全局历史记录
└── backups/                    # 全局备份
```

::: info 信息
CCR 现在默认使用 Unified Mode，支持多平台配置管理（Claude、Codex、Gemini 等）。

如需使用传统的单文件配置（`.ccs_config.toml`），设置环境变量：
```bash
export CCR_LEGACY_MODE=1
ccr init
```
:::

**初始化输出示例：**

```
CCR 配置初始化
═════════

▶ 创建 CCR 目录结构
✓ CCR 根目录: /home/user/.ccr
✓ 平台目录: /home/user/.ccr/platforms

▶ 初始化默认平台: Claude
✓ Claude 平台目录: /home/user/.ccr/platforms/claude
✓ 历史目录: /home/user/.ccr/history
✓ 备份目录: /home/user/.ccr/backups/claude

▶ 创建平台注册表
✓ 配置文件: /home/user/.ccr/config.toml

────────────────────────────────────────────────────────

✓ CCR 配置初始化成功 (Unified Mode)
```

### 2. 查看所有平台

```bash
ccr platform list
```

**输出示例：**

```
平台列表
════

ℹ 配置文件: /home/user/.ccr/config.toml
ℹ 默认平台: claude
ℹ 当前平台: claude

┌────────┬──────────┬──────┬──────────────┬──────────────────────────┐
│ 状态   ┆ 平台名称 ┆ 启用 ┆ 当前 Profile ┆ 描述                     │
╞════════╪══════════╪══════╪══════════════╪══════════════════════════╡
│ ▶ 当前 ┆ claude   ┆ ✓    ┆ -            ┆ Claude Code AI Assistant │
├╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│        ┆ codex    ┆ ✗    ┆ -            ┆ Codex                    │
├╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│        ┆ gemini   ┆ ✗    ┆ -            ┆ Gemini CLI               │
└────────┴──────────┴──────┴──────────────┴──────────────────────────┘

✓ 共找到 5 个平台

ℹ 提示:
  • 使用 'ccr platform switch <平台名>' 切换平台
  • 使用 'ccr platform current' 查看当前平台详情
  • 使用 'ccr platform info <平台名>' 查看平台信息
```

### 3. 添加配置 Profile

```bash
ccr add
```

这将启动交互式配置向导，帮你添加第一个 API 配置 profile。

**配置示例：**

```toml
[anthropic_main]
description = "Anthropic Official API"
base_url = "https://api.anthropic.com"
auth_token = "sk-ant-your-api-key"
model = "claude-sonnet-4-5-20250929"
small_fast_model = "claude-3-5-haiku-20241022"
```

### 4. 查看可用配置

```bash
ccr list
# 或使用别名
ccr ls
```

**输出示例：**

```
可用配置列表
═════════════════════════════════════════════════════════════════

ℹ 配置文件: /home/user/.ccr/config.toml
ℹ 默认配置: anthropic_main
ℹ 当前配置: anthropic_main
ℹ 当前平台: claude

╔═══════╤═══════════════╤══════════╤══════════════════════╤═══════════════════╤═══════════╤══════╗
║ 状态  │ 配置名称      │ 提供商   │ 模型                 │ Base URL          │ 账号/标签 │ 验证 ║
╠═══════╪═══════════════╪══════════╪══════════════════════╪═══════════════════╪═══════════╪══════╣
║▶ 当前 │ anthropic_main│ 🔄 claude│ claude-sonnet-4-... │ api.anthropic.com │ -         │  ✓   ║
╚═══════╧═══════════════╧══════════╧══════════════════════╧═══════════════════╧═══════════╧══════╝

✓ 共找到 1 个配置
```

### 5. 初始化其他平台（可选）

要管理多个 AI CLI 工具（Codex、Gemini），初始化相应的平台：

```bash
# 初始化 Codex (GitHub Copilot) 平台
ccr platform init codex

# 初始化 Gemini 平台
ccr platform init gemini

# 切换到 Codex 平台
ccr platform switch codex

# 在 Codex 平台上添加配置
ccr add

# 返回 Claude 平台
ccr platform switch claude
```

### 6. 配置完成！

现在你可以使用 CCR 的完整功能了：

```bash
ccr list              # 📊 列出所有配置
ccr switch <config>   # 🔄 切换配置
ccr current           # 🔍 查看当前配置
ccr validate          # ✅ 验证配置
ccr history           # 📚 查看操作历史
ccr tui               # 🖥️ 启动交互式 TUI
```

::: tip 配置说明
- `default_config`: 默认配置名称
- `current_config`: 当前使用的配置
- 每个配置块(如 `[anthropic]`)代表一个可切换的配置
- `base_url`: API 端点地址
- `auth_token`: 认证令牌
- `model`: 默认模型
- `small_fast_model`: 快速小模型(可选)
:::

**表格说明：**
- **▶ 当前** (绿色加粗) - 正在使用的配置
- **⭐ 默认** (黄色) - 默认配置
- **提供商图标** - 🔄 官方中转 / 🤖 第三方模型
- **验证列** - ✓ 配置完整 / ✗ 配置不完整

### 3. 切换配置

切换到指定配置非常简单：

```bash
ccr switch anyrouter
# 或使用简写形式（推荐）
ccr anyrouter
```

**执行流程：**

1. ✓ 读取并验证目标配置
2. ✓ 备份当前 Claude Code 设置
3. ✓ 更新 `~/.claude/settings.json`
4. ✓ 更新配置文件 `current_config`
5. ✓ 记录操作历史

**切换后输出：**

显示两个精美的表格：

1. **新配置详情表** - 展示切换后的配置信息
2. **环境变量变化表** - 对比切换前后的环境变量变化

```
╔════════════════╤═══════════════════════════════════╗
║ 属性           │ 新配置                            ║
╠════════════════╪═══════════════════════════════════╣
║ 配置名称       │ anyrouter_main                    ║
║ 提供商类型     │ 🔄 官方中转                       ║
║ Base URL       │ https://api.anyrouter.ai/v1       ║
...

╔═══════════════════════════╤══════════════════════════════╗
║ 环境变量                  │ 变化                         ║
╠═══════════════════════════╪══════════════════════════════╣
║ ANTHROPIC_BASE_URL        │ 🔄 api.anthropic.com → an... ║
║ ANTHROPIC_AUTH_TOKEN      │ 🔄 sk-a...old → sk-a...new   ║
...

✓ 配置已生效
💡 提示: 从 anthropic → anyrouter_main ✓
```

::: warning 注意
切换配置会立即修改 Claude Code 的设置文件，配置立即生效。
:::

### 4. 查看当前配置状态

```bash
ccr current
# 或使用别名
ccr status
ccr show
```

**输出示例：**

以两个表格展示当前配置的完整信息：

```
╔═══════════════╤═══════════════════════════════════════════════╗
║ 属性          │ 值                                            ║
╠═══════════════╪═══════════════════════════════════════════════╣
║ 配置名称      │ anyrouter_main                                ║
║ 描述          │ Anyrouter 主要配置                            ║
║ 提供商类型    │ 🔄 官方中转                                   ║
║ Base URL      │ https://api.anyrouter.ai/v1                   ║
...

╔═══════════════════════════╤═══════════════════════════════╤══════════╗
║ 环境变量                  │ 当前值                        │ 状态     ║
╠═══════════════════════════╪═══════════════════════════════╪══════════╣
║ ANTHROPIC_BASE_URL        │ https://api.anyrouter.ai/v1   │ ✓ 已设置 ║
║ ANTHROPIC_AUTH_TOKEN      │ sk-a...cdef                   │ ✓ 已设置 ║
...

✓ 当前配置: anyrouter_main
```

这将以清晰的表格形式显示当前配置的详细信息和环境变量设置。

### 5. 验证配置

验证配置和设置的完整性：

```bash
ccr validate
# 或使用别名
ccr check
```

**检查项目：**
- 配置文件格式
- 所有配置段的完整性
- Claude Code 设置文件
- 必需的环境变量

### 6. 查看操作历史

```bash
# 默认：显示最近 20 条记录
ccr history

# 自定义显示数量
ccr history --limit 50

# 按类型过滤
ccr history -t switch   # 仅显示切换操作
ccr history -t backup   # 仅显示备份操作
```

### 7. 启动 TUI 交互式界面

CCR 提供了全功能的终端用户界面（TUI），用于可视化配置管理：

```bash
# 启动 TUI
ccr tui

# 启动时启用自动确认模式（跳过确认提示）
ccr tui --yes      # 或使用简写：ccr tui -y
```

**TUI 功能特性：**

**📱 三个标签页：**
- **配置页（Tab 1）**：浏览和管理所有配置
  - 彩色编码（当前=绿色，默认=青色）
  - 实时切换配置
  - 删除配置（需自动确认模式）
- **历史页（Tab 2）**：查看带时间戳的操作历史
  - 操作结果指示器（✅ 成功、❌ 失败、⚠️ 警告）
  - 时间和配置信息
- **系统页（Tab 3）**：显示系统信息和文件路径
  - 主机名、用户名、操作系统
  - CCR 版本和当前配置
  - 配置文件路径

**⌨️ 键盘快捷键：**
- `1-3`：快速切换到对应标签页
- `Tab` / `Shift+Tab`：前后切换标签页
- `↑↓` 或 `j`/`k`：导航列表（支持 Vim 快捷键）
- `Enter`：切换到选中的配置
- `d`：删除选中的配置（需要自动确认模式）
- `y` / `Y`：切换自动确认模式
- `q` / `Ctrl+C`：退出 TUI

**⚡ 自动确认模式（原 YOLO 模式）：**
- 自动确认模式会跳过所有危险操作的确认提示
- TUI 中删除配置操作必须启用该模式
- 可以使用 `Y` 键在运行时切换
- 也可以启动时使用 `--yes` / `-y` 标志
- 页脚会显示当前状态（🔶 AUTO / 🟢 SAFE）

**示例工作流：**
```bash
ccr tui              # 启动 TUI
# 按 '1' → 进入配置页 → 使用 ↑↓ 或 j/k 选择 → Enter 切换配置
# 按 '2' → 查看操作历史
# 按 '3' → 查看系统信息和文件路径
# 按 'Y' → 启用自动确认模式 → 'd' 删除配置
# 按 'q' → 退出 TUI
```

::: tip TUI vs CLI
TUI 提供更友好的可视化界面，适合交互式管理。CLI 命令更适合脚本和自动化场景。
:::

### 8. 启动轻量级 Web API 服务器（Legacy）

CCR 提供了轻量级的 Web API 服务器（基于 Axum）：

> ⚠️ 提示：`ccr web` 主要用于兼容旧版 Web 界面和提供 HTTP API，适合脚本、CI 等编程访问场景。**如果只是想在浏览器中管理配置，推荐使用下一节的 `ccr ui`（CCR UI 全栈应用）作为主要网页端。**

```bash
# 使用默认端口 8080，自动打开浏览器
ccr web

# 指定端口
ccr web --port 3000

# 不自动打开浏览器（适合远程服务器）
ccr web --no-browser

# 指定端口且不打开浏览器
ccr web --port 9000 --no-browser
```

**启动输出示例：**

```
2025-10-31 16:05:12 [INFO] 🔄 系统信息缓存后台线程已启动，更新间隔: 2s
✓ 🌐 CCR Web 服务器已启动（异步模式）
ℹ 📍 地址: http://localhost:8080
ℹ ⏹️ 按 Ctrl+C 停止服务器
💡 请手动访问 http://localhost:8080
```

::: tip 智能端口绑定
如果指定的端口已被占用，CCR 会自动尝试其他可用端口（如 8081、8082 等），确保服务能够成功启动。实际使用的端口会在启动信息中显示。
:::

**功能特性：**
- 📊 **完整的 Web 管理界面**：现代化的玻璃拟态设计，支持明暗主题
- 🎯 **多平台支持**：Claude、Codex、Gemini、Qwen、iFlow 平台管理（Unified Mode）
- 📈 **实时系统监控**：CPU、内存使用率实时显示
- 🔄 **配置管理**：查看、添加、编辑、删除、切换配置
- 📚 **操作历史**：完整的审计追踪
- ☁️ **云同步**：WebDAV 云端备份与恢复
- ✅ **配置验证**：一键验证所有配置
- 🗑️ **备份清理**：智能清理旧备份文件
- 📥📤 **导入/导出**：配置文件的导入导出功能
- 🌐 **RESTful API**：14 个完整 API 端点，适合编程访问

**Web 界面亮点：**

浏览器访问后可以看到三栏式布局：
- **左侧边栏**：当前配置状态、系统信息、CPU/内存监控、统计信息
- **中央区域**：配置列表、历史记录、云同步三个标签页
- **右侧边栏**：配置快速导航

### 9. 启动 CCR UI 应用

CCR UI 是一个功能完整的 **Next.js + Actix Web** 全栈应用，提供可视化仪表板：

```bash
# 使用默认端口（前端 3000，后端 8081）
ccr ui

# 自定义前端端口
ccr ui -p 8080

# 自定义前后端端口
ccr ui -p 8080 --backend-port 9000
```

**功能特性：**

**🎨 前端技术栈：**
- Next.js 16（React 19）App Router
- TypeScript + Tailwind CSS
- 现代化的玻璃拟态设计
- 实时响应式界面

**🦀 后端技术栈：**
- Actix Web 高性能服务器
- 包装 CCR CLI 执行
- 扩展 API（MCP、Agents、Plugins 等）

**💡 主要功能：**
- 📊 **可视化配置管理**：图形化展示所有配置
- 💻 **命令执行器**：执行所有 CCR 命令，带实时输出
- 🎯 **多 CLI 支持**：管理 Claude Code、Codex、Gemini、Qwen 等
- 🧩 **MCP 服务器管理**：配置和管理 MCP 协议服务器
- ⚡ **Slash 命令编辑器**：创建和管理自定义斜杠命令
- 🤖 **Agents 管理**：配置 Claude 智能体
- 🔌 **插件系统**：扩展 CCR UI 功能

**🚀 启动流程：**

CCR 使用**三级优先级检测系统**智能选择最佳启动方式：

**优先级 1：开发环境**
- 检测当前目录或父目录的 `ccr-ui/` 目录
- 使用 `just dev` 启动源码版本
- 支持热重载和实时开发
- 适合：CCR 项目开发者

**优先级 2：用户目录**
- 检测 `~/.ccr/ccr-ui/` 目录
- 使用已下载的源码版本
- 与开发模式相同的启动方式
- 适合：日常使用

**优先级 3：GitHub 自动下载**
- 未找到本地版本时，自动提示下载
- 从 GitHub 克隆 CCR 仓库
- 自动提取 `ccr-ui/` 子目录到 `~/.ccr/ccr-ui/`
- 临时文件自动清理
- 适合：首次使用

**首次使用示例：**
```bash
ccr ui

# 💬 提示: CCR UI 是一个完整的 Next.js + Actix Web 应用
#    可以从 GitHub 下载到用户目录:
#    /home/user/.ccr/ccr-ui/
#
# ❓ 是否立即从 GitHub 下载 CCR UI? [Y/n]: y
# 📦 克隆仓库: https://github.com/bahayonghang/ccr.git
# 📁 临时目录: /tmp/.tmpXXXXXX
# ⏳ 下载中 (这可能需要几分钟)...
# 📦 正在复制文件到目标目录...
# ✅ CCR UI 下载完成
# 📁 安装位置: /home/user/.ccr/ccr-ui/
#
# 🔍 检查 just 工具...
# ✅ just 已安装: just 1.x.x
# 🔍 检查项目依赖...
# ✅ 依赖已就绪
#
# 🔧 使用开发模式启动 CCR UI
# 📍 后端: http://localhost:8081
# 📍 前端: http://localhost:3000 (Next.js)
#
# 💡 提示: 按 Ctrl+C 停止服务
```

**系统要求：**
- ✅ **必需工具**：
  - `git` - 用于从 GitHub 下载（首次使用）
  - `just` - 用于启动开发环境
  - `Node.js` / `npm` - 前端依赖（自动检测和安装）
  - `Rust` / `cargo` - 后端依赖（自动检测和安装）

- 📋 **工具安装**：
  ```bash
  # Git (通常已预装)
  sudo apt-get install git  # Debian/Ubuntu
  brew install git          # macOS

  # Just
  cargo install just

  # Node.js (推荐使用 nvm)
  curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
  nvm install --lts
  ```

**📍 访问地址：**
- 前端界面：http://localhost:3000
- 后端 API：http://localhost:8081

::: tip Web Server vs CCR UI
- **ccr web**：轻量级 Legacy API 服务器（8080 端口），保留用于兼容性和编程访问（curl、CI、脚本等），未来会逐步弃用
- **ccr ui**：推荐使用的完整全栈 Web 界面（3000/8081 端口），提供丰富的可视化功能和多 CLI 工具管理
:::

**示例工作流：**
```bash
# 1. 启动 CCR UI
ccr ui

# 2. 浏览器自动打开 http://localhost:3000

# 3. 在 UI 中：
#    - 查看和切换配置
#    - 执行 CCR 命令
#    - 管理 MCP 服务器
#    - 编辑 Slash 命令
#    - 配置 Agents

# 4. 按 Ctrl+C 停止服务
```

## 日常使用

### 快速切换配置

```bash
# 切换到 anthropic 配置
ccr anthropic

# 切换到 anyrouter 配置
ccr anyrouter
```

### 导出和导入配置

**导出配置：**

```bash
# 导出包含完整 API 密钥(默认)
ccr export

# 导出时脱敏敏感信息
ccr export --no-secrets

# 导出到指定文件
ccr export -o backup.toml
```

**导入配置：**

```bash
# 合并模式(保留现有配置,添加新配置)
ccr import config.toml --merge

# 替换模式(完全替换当前配置)
ccr import config.toml

# 导入时不备份
ccr import config.toml --no-backup
```

### 备份管理

::: tip 智能备份管理
CCR 会自动保留最近10个备份，无需手动清理。大多数情况下你不需要运行 `clean` 命令。
:::

```bash
# 清理更早期的备份(超过7天)
ccr clean

# 清理更早期的备份(超过30天)
ccr clean --days 30

# 预览清理(不实际删除)
ccr clean --dry-run
```

### 更新 CCR

```bash
# 检查更新
ccr update --check

# 更新到最新版本
ccr update
```

## 文件位置

CCR 使用以下文件和目录：

```
~/.ccs_config.toml          # 配置文件(与 CCS 共享)
~/.claude/settings.json     # Claude Code 设置文件
~/.claude/backups/          # 自动备份目录
~/.claude/ccr_history.json  # 操作历史日志
~/.claude/.locks/           # 文件锁目录
```

## 故障排除

### 配置文件未找到

```bash
# 检查配置文件
ls -la ~/.ccs_config.toml

# 如果不存在,初始化配置
ccr init
```

### Claude Code 设置文件未找到

```bash
# 检查 Claude Code 目录
ls -la ~/.claude/

# 首次使用时会自动创建
ccr switch <config>
```

### 文件锁超时

```bash
# 检查僵尸进程
ps aux | grep ccr

# 清理锁文件(谨慎使用)
rm -rf ~/.claude/.locks/*
```

### 权限问题

```bash
# 检查文件权限
ls -la ~/.claude/settings.json
ls -la ~/.ccs_config.toml

# 修复权限
chmod 600 ~/.claude/settings.json
chmod 644 ~/.ccs_config.toml
```

## 下一步

- 查看 [核心命令](/commands/) 了解所有可用命令
- 查看 [配置管理](/configuration) 了解高级配置选项
- 查看 [架构文档](/architecture) 了解项目架构设计
- 查看 [更新日志](/changelog) 了解最新功能

## 开发者指南

如果你想参与 CCR 开发或了解内部实现：

### 项目架构

CCR 采用严格的分层架构，详见 [架构文档](/architecture)：

```
CLI/Web Layer → Services → Managers → Core/Utils
```

### 目录结构

```
src/
├── commands/         # CLI 命令实现
├── web/              # Web 界面和 API
├── services/         # 业务逻辑层
├── managers/         # 数据访问层
│   ├── config.rs     # 配置文件管理
│   ├── settings.rs   # 设置文件管理
│   └── history.rs    # 历史记录管理
├── core/             # 核心基础设施
│   ├── error.rs      # 错误处理
│   ├── lock.rs       # 文件锁
│   └── logging.rs    # 日志输出
└── utils/            # 工具函数
```

### 开发命令

```bash
# 快速类型检查
cargo check

# 运行测试
cargo test

# 代码检查
cargo clippy

# 格式化代码
cargo fmt

# 构建开发版本
cargo build

# 构建生产版本
cargo build --release
```

### 添加新功能

1. **添加新命令**：在 `src/commands/` 创建新文件
2. **添加 API 端点**：在 `src/web/handlers.rs` 添加处理器
3. **添加业务逻辑**：在 `src/services/` 创建或扩展服务
4. **修改数据结构**：在 `src/managers/` 修改相应管理器

详细开发指南请查看项目根目录的 `CLAUDE.md` 文件。
