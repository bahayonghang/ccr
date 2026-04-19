# CCR

**Rust 编写的 AI CLI 配置与工作入口。**  
以 CLI 为主线，配套 TUI、Legacy Web API 和完整 CCR UI，统一管理 Claude Code、Codex、Gemini、Qwen、Droid 等多种 AI CLI 平台。

> 历史说明：CCR 最早来自 `Claude Code Configuration Switcher`。现在仓库已经演进为多平台 AI CLI 工作区。

![Version](https://img.shields.io/badge/version-5.9.4-blue.svg) ![License](https://img.shields.io/badge/license-MIT-green.svg) ![Build](https://img.shields.io/badge/build-passing-brightgreen.svg)

---

## ✨ 核心特性

- **统一平台注册表**：统一管理 Claude、Codex、Gemini、Qwen、Droid 等 AI CLI 平台的独立 profile、历史和备份。
- **企业级安全**：支持原子写入、文件锁 (`fs4`)、完整审计日志，且在每次修改前自动备份。
- **多端接口**：
  - **CLI**：功能强大的命令行接口。
  - **TUI**：交互式终端配置选择器，支持Tab切换。
  - **Legacy Web API**：面向脚本、CI 和兼容场景的嵌入式 Axum 服务。
  - **CCR UI**：基于 Vue 3 + Tauri 的浏览器/桌面图形入口。
- **认证迁移友好**：支持保存、导出（加密）、导入 Codex 账号，并可将兼容的已保存 Codex 账号安全迁移为 OpenCode 的已保存账号，且不覆盖现有 OpenCode 条目。
- **智能同步**：基于 WebDAV 的多文件夹同步能力，保持多机配置一致。
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
cargo install --path crates/ccr
```

> 工作区说明：当前可安装的 CLI crate 位于 `crates/ccr`。旧路径到新路径的映射请参考 `docs/reference/migration.md`。

### 构建要求
- **Rust**: 1.90+（面向可安装 CLI crate）
- **Node.js**: 18+
- **Bun**: 1.3+（`ccr-ui` 与内嵌 Web 资源构建的推荐路径；若无 Bun，Legacy 内嵌 Web 构建仍可兼容回退到 npm）

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
| `Tab` | 在可用页签之间切换 |
| `←` / `→` | 翻页（配置超过 20 个时） |
| `↑` / `↓` / `j` / `k` | 选择配置 |
| `Enter` | 应用选中的配置并退出 |
| `Space` | 应用选中的配置（保持 TUI） |
| `q` / `Esc` | 退出 |

**功能特性：**
- 多页签界面：Claude、Codex 与 OpenCode 相关视图
- 分页支持（每页 20 个配置）
- 底部实时状态消息
- 平台专属配色（Claude 橙色，Codex 紫色）

## 📖 帮助与版本

```bash
# 任务导向总帮助
ccr --help

# 查看嵌套命令帮助
ccr help platform
ccr help codex auth
ccr help opencode auth

# 给脚本 / CI 用的简短版本号
ccr --version
ccr -V

# 给人看的详细版本信息
ccr version
```

只需要版本号时，用 `ccr --version`。  
想看当前安装说明和主要入口时，用 `ccr version`。

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

包含敏感信息的导出**默认加密**，使用 AES-256-GCM + Argon2id 密钥派生，确保跨设备传输账号凭据时的安全性。

```bash
# 导出所有账号（加密，会提示设置密码）
ccr codex auth export

# 导出时不包含敏感数据（无需加密）
ccr codex auth export --no-secrets

# 从文件导入账号（自动检测加密/明文格式）
ccr codex auth import

# 使用替换模式导入（覆盖同名账号）
ccr codex auth import --replace

# 使用强制模式导入（合并模式下覆盖已存在账号）
ccr codex auth import --force
```

**加密说明：**
- 包含敏感信息的导出使用用户设置的密码加密（AES-256-GCM + Argon2id）
- 导入时自动检测文件是加密还是明文，加密文件会提示输入密码
- 旧版明文导出文件仍然可以正常导入（向后兼容）

**导入模式说明：**
- **合并模式（默认）**：跳过已存在的账号，只添加新账号
- **合并 + --force**：强制覆盖已存在的账号
- **替换模式**：始终覆盖同名账号

### Codex -> OpenCode Auth 迁移

```bash
# 预览有哪些已保存的 Codex 账号可以导入 OpenCode
ccr opencode auth import-codex --dry-run

# 将兼容的已保存 Codex 账号导入 OpenCode
ccr opencode auth import-codex

# 输出机器可读的 JSON 迁移报告
ccr opencode auth import-codex --json
```

**迁移行为：**
- 只导入已经保存到 CCR 的 Codex 账号，不会读取未保存的临时运行时登录态
- 仅支持带 ChatGPT OAuth Token 的兼容 Codex 账号
- 会跳过仅 API Key、快照损坏或缺少快照的账号
- 不会覆盖已存在的 OpenCode 账号
- 不会在导入过程中切换当前 OpenCode 运行时登录
- 会按原因报告跳过项，包括同名冲突、`accountId` 冲突、缺少快照和无效快照

### 交互式 TUI

启动 Codex 账号管理界面：
```bash
ccr codex

# 直接进入 OpenCode Auth 页签
ccr opencode
```

**功能特性：**
- 可视化账号列表，带 Token 新鲜度指示器
- 🟢 新鲜 (<1天) | 🟡 陈旧 (1-7天) | 🔴 过期 (>7天)
- 切换前进程检测警告
- 邮箱脱敏保护隐私（如 `use***@example.com`）
- 在 OpenCode Auth 页签中，按 `i` 可预览并确认导入兼容的已保存 Codex 账号

## 🔄 自动更新

CCR 支持从 GitHub 自动更新到最新版本。

> 注意：`ccr update` 在构建内嵌 Web 资源时会优先使用 Bun；如果系统没有 Bun，仍可兼容回退到 Node.js 18+ 与 npm。

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
├── Cargo.toml      # Workspace 清单 + 共享依赖
├── crates/
│   ├── ccr/        # 可安装的 CLI crate + 库
│   ├── ccr-db/     # 数据库服务与数据模型
│   └── ccr-types/  # 共享类型定义
├── ccr-ui/         # 全栈 Web/桌面应用 (Vue 3 + Tauri)
├── docs/           # VitePress 文档
├── scripts/        # 仓库自动化与维护脚本
├── examples/       # 示例配置与用法
├── outputs/        # 汇总/生成产物（如存在）
└── justfile        # 任务运行配置
```

## 📄 许可证
MIT License
