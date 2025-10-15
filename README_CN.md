# 🚀 CCR - Claude Code Configuration Switcher

**Rust 驱动的 Claude Code 配置管理工具**

CCR 通过原子操作、文件锁、完整审计追踪和自动备份直接管理 Claude Code 的 `settings.json`。CCS 的 Rust 实现版,提供更高的可靠性和性能。

## ✨ 为什么选择 CCR？

| 特性 | 说明 |
|------|------|
| 🎯 **直接控制设置** | 直接写入 `~/.claude/settings.json` - 立即生效 |
| 📊 **精美表格界面** | 使用 comfy-table 展示配置信息，一目了然对比不同配置，支持颜色高亮和图标标识 |
| 🔒 **并发安全** | 文件锁 + 原子操作防止多进程并发损坏 |
| 📝 **完整审计追踪** | 每个操作都有日志记录（UUID、时间戳、操作者），敏感数据已掩码 |
| 💾 **自动备份** | 更改前自动备份，生成带时间戳的 `.bak` 文件 |
| ✅ **配置验证** | 全面验证（URL、必填字段、格式） |
| 🔤 **配置优化** | 按字母顺序整理配置，保持顺序不被打乱 |
| 🌐 **Web 服务器** | 内置 Axum Web 服务器，提供 14 个 RESTful API 端点（配置、历史、备份、系统信息等） |
| 🖥️ **全栈 Web UI** | 基于 Next.js 16（React 19）+ Actix Web 的可视化管理界面 |
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

CCR UI 是一个现代化的 **Next.js + Actix Web** 全栈应用，用于 CCR 配置管理！

前端使用 App Router 架构与 React 19，结合 Tailwind 构建交互界面；后端基于 Actix 包装 CCR CLI，并额外提供 MCP 服务器、斜杠命令、智能体与插件的管理 API。

### 功能特性

- ⚛️ **Next.js 前端**：Next.js 16（React 19）App Router，配合 TypeScript 与 Tailwind CSS
- 🦀 **Actix Web 后端**：高性能 Rust 异步 Web 服务器
- 🖥️ **配置管理**：可视化配置切换和验证
- 💻 **命令执行器**：执行所有 13 个 CCR 命令，可视化输出
- 📊 **语法高亮**：终端风格输出，带颜色编码
- ⚡ **实时执行**：异步命令执行，带进度显示
- 🧩 **扩展控制台**：内置 MCP、斜杠命令、智能体与插件管理 API

### 超快启动

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

**🎯 CLI vs Web 服务器 vs CCR UI**：
- **CLI 工具**：适合脚本、自动化和快速操作
- **Web 服务器** (`ccr web`)：内置轻量级 Axum 服务器，用于 API 访问
- **CCR UI** (Actix + Next.js)：全功能 Web 应用，用于可视化管理

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
ccr web               # 🌐 启动 Web 界面 (端口 8080)
```

## 📚 命令参考

| 命令 | 别名 | 说明 |
|------|------|------|
| `ccr init [--force]` | - | 🎬 从模板初始化配置 |
| `ccr list` | `ls` | 📊 以表格形式列出所有配置（状态、提供商、URL、模型、验证） |
| `ccr current` | `show`, `status` | 🔍 以双表格展示当前配置详情和环境变量状态 |
| `ccr switch <name>` | `<name>` | 🔄 切换配置（表格展示新配置和环境变量变化对比） |
| `ccr validate` | `check` | ✅ 验证所有配置和设置 |
| `ccr optimize` | - | 🔤 按字母顺序优化配置文件结构 |
| `ccr history [-l N] [-t TYPE]` | - | 📚 显示操作历史（限制数量/按类型筛选） |
| `ccr web [-p PORT]` | - | 🌐 启动 Web 界面（默认 8080 端口） |
| `ccr export [-o FILE] [--no-secrets]` | - | 📤 导出配置（包含/不含 API 密钥） |
| `ccr import FILE [--merge]` | - | 📥 导入配置（合并或替换） |
| `ccr clean [-d DAYS] [--dry-run]` | - | 🧹 清理旧备份（默认 7 天） |
| `ccr update [--check]` | - | ⚡ 从 GitHub 更新 CCR（实时进度显示） |
| `ccr version` | `ver` | ℹ️ 显示版本和功能 |

**切换操作流程：**
1. 📖 读取并验证目标配置
2. 💾 备份当前 settings.json
3. ✏️ 更新 ~/.claude/settings.json(原子写入 + 文件锁)
4. 📝 更新 current_config 标记
5. 📚 记录到历史(环境变量变化已掩码)

## 📁 文件与目录

```
~/.ccs_config.toml          # 📝 配置文件(与 CCS 共享)
~/.claude/settings.json     # 🎯 Claude Code 设置(CCR 管理)
~/.claude/backups/          # 💾 自动备份(带时间戳的 .bak 文件)
~/.claude/ccr_history.json  # 📚 操作审计日志
~/.claude/.locks/           # 🔒 文件锁(自动清理)
```

## 🔧 核心功能

### 🌍 环境变量

CCR 在 `settings.json` 中管理这些变量：
- `ANTHROPIC_BASE_URL` - API 端点
- `ANTHROPIC_AUTH_TOKEN` - 认证令牌(显示/日志中自动掩码)
- `ANTHROPIC_MODEL` - 默认模型
- `ANTHROPIC_SMALL_FAST_MODEL` - 快速模型(可选)

### 📚 历史与审计

每个操作都会记录：
- UUID + 时间戳 + 系统用户名
- 操作类型(switch/backup/restore/validate/update)
- 环境变量变化(已掩码)
- 源/目标配置 + 备份路径
- 结果(成功/失败/警告)

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
├── backend/          # 🦀 Actix Web 服务器
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
└── frontend/         # ⚛️ Next.js 16 App Router
    ├── src/
    │   ├── app/              # 路由分段（configs、commands、agents 等）
    │   ├── components/       # 可复用 UI 组件
    │   └── lib/              # API 客户端与工具
    ├── package.json
    └── next.config.mjs
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
