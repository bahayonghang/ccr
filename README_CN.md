# 🚀 CCR - Claude Code Configuration Switcher

**Rust 驱动的 Claude Code 配置管理工具**

CCR 通过原子操作、文件锁、完整审计追踪和自动备份直接管理 Claude Code 的 `settings.json`。CCS 的 Rust 实现版,提供更高的可靠性和性能。

## ✨ 为什么选择 CCR？

| 特性 | 说明 |
|------|------|
| 🎯 **直接控制设置** | 直接写入 `~/.claude/settings.json` - 立即生效 |
| 🔒 **并发安全** | 文件锁 + 原子操作防止多进程并发损坏 |
| 📝 **完整审计追踪** | 每个操作都有日志记录(UUID、时间戳、操作者),敏感数据已掩码 |
| 💾 **自动备份** | 更改前自动备份,生成带时间戳的 `.bak` 文件 |
| ✅ **配置验证** | 全面验证(URL、必填字段、格式) |
| 🔤 **配置优化** | 按字母顺序整理配置,保持顺序不被打乱 |
| 🌐 **Web 界面** | 11 个完整 RESTful API 端点,浏览器管理界面 |
| 🖥️ **桌面应用** | 基于 Tauri 的原生桌面应用,Vue 3 现代界面,支持深色模式 |
| 🏗️ **现代架构** | Service 层模式,模块化设计,95%+ 测试覆盖率 |
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

## 🖥️ 桌面应用

CCR 现在提供基于 **Tauri 2.0 + Vue 3** 构建的原生桌面应用！

### 功能特性

- 🎨 **现代化界面**：精美的三栏布局设计,支持深色/浅色主题切换
- 🔄 **配置管理**：通过图形界面切换、创建、编辑、删除配置
- 🏷️ **智能筛选**：按类型筛选配置（官方中转、第三方模型、未分类）
- 📚 **历史追踪**：查看所有操作记录和详细日志
- 💾 **备份管理**：轻松列出和恢复备份文件
- 📤 **导入导出**：图形化配置导入导出功能
- ⚙️ **系统信息**：一目了然地显示主机名、用户名、路径等信息

### 快速启动

```bash
cd ccr-tauri

# 安装前端依赖
cd src-ui && npm install && cd ..

# 开发模式运行
cargo tauri dev

# 构建生产版本
cargo tauri build
```

**📖 完整文档**：查看 `ccr-tauri/docs/` 获取完整的架构文档、API 参考和开发指南（基于 VitePress 构建）。

**🎯 桌面 vs CLI vs Web**：
- **桌面应用**：适合可视化管理和频繁切换配置
- **CLI 工具**：适合脚本和自动化场景
- **Web 界面**：适合远程访问和团队管理

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
ccr list              # 📋 列出所有配置
ccr switch anthropic  # 🔄 切换配置 (或简写: ccr anthropic)
ccr current           # 🔍 显示当前状态
ccr validate          # ✅ 验证所有配置
ccr history           # 📚 查看操作历史
ccr web               # 🌐 启动 Web 界面 (端口 8080)
```

## 📚 命令参考

| 命令 | 别名 | 说明 |
|------|------|------|
| `ccr init [--force]` | - | 🎬 从模板初始化配置 |
| `ccr list` | `ls` | 📜 列出所有配置及验证状态 |
| `ccr current` | `show`, `status` | 🔍 显示当前配置和环境变量 |
| `ccr switch <name>` | `<name>` | 🔄 切换配置(5 步原子操作) |
| `ccr validate` | `check` | ✅ 验证所有配置和设置 |
| `ccr optimize` | - | 🔤 按字母顺序优化配置文件结构 |
| `ccr history [-l N] [-t TYPE]` | - | 📚 显示操作历史(限制数量/按类型筛选) |
| `ccr web [-p PORT]` | - | 🌐 启动 Web 界面(默认 8080 端口) |
| `ccr export [-o FILE] [--no-secrets]` | - | 📤 导出配置(包含/不含 API 密钥) |
| `ccr import FILE [--merge]` | - | 📥 导入配置(合并或替换) |
| `ccr clean [-d DAYS] [--dry-run]` | - | 🧹 清理旧备份(默认 7 天) |
| `ccr update [--check]` | - | ⚡ 从 GitHub 更新 CCR(实时进度显示) |
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
- `GET /api/configs` - 列出所有
- `POST /api/switch` - 切换配置
- `GET /api/history` - 查看历史
- `POST /api/validate` - 验证所有
- `POST /api/clean` - 清理备份
- `POST/PUT/DELETE /api/config` - 增删改操作

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
├── web/              # 🌐 Web 层（HTTP 服务器 + API）
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

ccr-tauri/            # 🖥️ 桌面应用
├── src/              # 🦀 Rust 后端（Tauri commands）
│   ├── main.rs       # 应用入口
│   ├── lib.rs        # 库导出
│   └── commands/     # Tauri 命令定义
├── src-ui/           # 🎨 Vue 3 前端
│   ├── src/
│   │   ├── App.vue   # 主组件
│   │   ├── api/      # API 层（Tauri invoke）
│   │   ├── types/    # TypeScript 类型定义
│   │   └── style.css # 全局样式
│   └── package.json  # 前端依赖
├── docs/             # 📚 VitePress 文档
│   ├── guide/        # 使用指南
│   ├── api/          # API 参考
│   ├── architecture/ # 架构文档
│   └── development/  # 开发指南
├── capabilities/     # 🔐 Tauri 2.0 权限配置
└── tauri.conf.json   # Tauri 配置文件
```

**命令：**
```bash
cargo test            # 🧪 运行测试
cargo clippy          # 🔍 代码检查
cargo fmt             # 💅 格式化
cargo build --release # 🏗️ 生产构建
```

## 🏗️ 架构

CCR v1.1.0 采用严格的分层架构：

```
CLI/Web 层 → Services 层 → Managers 层 → Core/Utils 层
```

- **Service 层**: 4 个服务(Config, Settings, History, Backup)- 26 个方法
- **Web 模块**: 模块化设计(models, server, handlers, routes)- 11 个 API 端点
- **基础设施**: 原子写入器、文件管理器 trait、验证 trait
- **测试覆盖**: 95%+ (79/83 测试通过)

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
