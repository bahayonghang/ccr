# CCR - Claude Code Configuration Switcher

Rust 编写的高性能多平台配置管理工具，版本 3.9.4。支持 Claude Code、Codex、Gemini CLI、Qwen等主流 AI CLI 工具。提供 CLI、TUI、Web API 三种使用方式，以及基于 Vue 3 + Axum + Tauri 的全栈 CCR UI 应用。

## 功能亮点

- **安全写入机制**：原子写入 + 文件锁 + 审计日志 + 自动备份，直接管理 `settings.json` 与配置文件
- **统一多平台模式**：默认使用 `~/.ccr/`（按平台分别存放 profiles/history/backups），兼容传统 `~/.ccs_config.toml` 单文件模式
- **配置全生命周期**：`init`、`add`、`list/current/switch`、`enable/disable`、`validate`、`history`、`optimize`，支持 `--yes` 跳过确认
- **多平台支持**：统一管理 Claude、Codex、Gemini、Qwen、iFlow 五大平台，独立配置、历史记录和备份
- **导入/导出/清理**：导出可选择去除敏感信息，导入支持合并/替换并自动备份，清理旧备份
- **WebDAV 多目录同步**（`web` 特性）：支持多文件夹独立管理，单目录或全量 push/pull/status
- **平台管理**：`ccr platform list/switch/current/info/init`，可输出 JSON 便于脚本
- **临时凭据与更新**：`ccr temp-token set/show/clear`，`ccr update --check` 检查或更新版本
- **可观测性**：完整历史记录、JSON 输出选项、成本统计（`ccr stats ...`，`web` 特性）
- **TUI 终端界面**：Ratatui 驱动的交互式界面，支持配置管理、平台切换、同步状态查看
- **技能与提示词管理**：`ccr skills` 和 `ccr prompts` 管理 AI 技能和提示词
- **多种界面**：CLI 通用，TUI（`--features tui`），兼容型 `ccr web` 轻量 API，现代 CCR UI（`ccr ui`，Vue 3 + Axum 后端，可打包 Tauri 桌面版）

## 安装

依赖：Rust 1.85+ (Edition 2024)、Cargo。开发 CCR UI 需要 Node.js 18+ 和 [Bun](https://bun.sh/) 1.0+（包管理器），建议安装 `just` 以使用快捷命令。

### 一行安装
```bash
cargo install --git https://github.com/bahayonghang/ccr ccr
```

### 源码安装
```bash
git clone https://github.com/bahayonghang/ccr.git
cd ccr
cargo install --path .
```

### 构建选项
```bash
# 仅 CLI（最快）
cargo build --no-default-features

# CLI + Web API + WebDAV 同步 + UI 入口
cargo build --features web

# CLI + TUI 终端界面
cargo build --features tui

# 全功能（推荐）
cargo build --all-features

# 工作区构建和测试
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

## 工作区结构

```
ccr/
|-- src/                # CLI + 共享库（99 个文件，~28K 行代码）
|   |-- commands/       # 27 个 CLI 命令实现
|   |-- services/       # 7 个业务服务层
|   |-- managers/       # 11 个数据管理器
|   |-- platforms/      # 5 个 AI 平台支持
|   |-- sync/           # WebDAV 多文件夹同步
|   |-- web/            # Axum Web API（14 个端点）
|   |-- tui/            # Ratatui 终端界面
|   |-- models/         # 数据模型定义
|   |-- core/           # 基础设施（错误、锁、日志）
|   `-- utils/          # 验证、掩码等工具
|
|-- ccr-ui/             # 全栈 Web 应用
|   |-- backend/        # Axum 后端服务（workspace 成员）
|   `-- frontend/       # Vue 3 + Vite + Pinia + Tauri
|
|-- tests/              # 集成测试（6 个测试文件）
|-- docs/               # VitePress 文档站
|-- examples/           # 示例与演示
`-- justfile            # 常用开发任务
```

## CLI 快速上手

### 1. 初始化配置

**统一多平台模式**（默认）：
```bash
ccr init
```
生成 `~/.ccr/config.toml` 以及 `~/.ccr/platforms/` 下的各平台目录。

**传统单文件模式**：
```bash
export CCR_LEGACY_MODE=1
ccr init
```
继续使用 `~/.ccs_config.toml` 单文件。

### 2. 查看并切换平台

```bash
# 列出所有可用平台
ccr platform list

# 切换到指定平台（claude/codex/gemini/qwen/iflow）
ccr platform switch claude

# 查看当前平台及配置
ccr platform current

# 查看平台详细信息
ccr platform info claude
```

### 3. 创建与管理配置

```bash
# 交互式创建配置
ccr add

# 列出当前平台的所有配置
ccr list

# 查看当前配置
ccr current

# 切换配置（两种方式）
ccr switch <name>
ccr <name>           # 快捷方式

# 启用/禁用配置
ccr enable <name>
ccr disable <name> [--force]

# 验证配置文件完整性
ccr validate

# 查看历史记录（最近50条）
ccr history -l 50

# 优化配置排序（按字母排序）
ccr optimize
```

### 4. 导入/导出/清理

```bash
# 导出配置（可选择去除敏感信息）
ccr export -o configs.toml --no-secrets

# 导入配置（支持合并或替换模式）
ccr import configs.toml --merge --backup

# 清理超过30天的旧备份（预览模式）
ccr clean --days 30 --dry-run

# 正式清理
ccr clean --days 30

# 清理 CCR 写入的配置（恢复 settings.json 默认状态）
ccr clear                  # 交互式确认
ccr clear --force          # 跳过确认
```

### 5. WebDAV 多目录同步（需要 `web` 特性）

#### 配置 WebDAV
```bash
# 配置 WebDAV 连接信息
ccr sync config

# 系统会自动创建默认文件夹：
# - claude: ~/.claude/ → /ccr/claude
# - gemni: ~/.gemini/ → /ccr/gemini
# - conf: ~/.ccs_config.toml → /ccr/config.toml
```

#### 文件夹管理
```bash
# 列出所有同步文件夹
ccr sync folder list

# 添加自定义同步文件夹
ccr sync folder add scripts ~/my-scripts \
  -r /ccr/scripts \
  -d "My custom scripts"

# 查看文件夹详情
ccr sync folder info claude

# 启用/禁用同步
ccr sync folder enable claude
ccr sync folder disable gemini
```

#### 同步操作
```bash
# 同步指定文件夹
ccr sync claude push      # 上传 Claude 配置
ccr sync gemini pull      # 下载 Gemini 配置
ccr sync conf status      # 查看配置状态

# 批量同步所有启用文件夹
ccr sync all push --force    # 上传所有配置
ccr sync all pull --force    # 下载所有配置
ccr sync all status          # 查看所有状态

# 兼容旧版命令
ccr sync push        # 等同于：ccr sync all push
ccr sync pull        # 等同于：ccr sync all pull
ccr sync status      # 显示所有状态
```

### 6. 平台管理

```bash
# 初始化指定平台（如果不存在）
ccr platform init claude
ccr platform init codex
ccr platform init gemini

# 平台切换工作流程
ccr platform switch claude    # 切换到 Claude 平台
ccr list                       # 显示 Claude 平台的配置
ccr add                        # 为 Claude 添加配置
ccr platform switch codex      # 切换到 Codex 平台
ccr list                       # 显示 Codex 平台的配置
```

### 7. 迁移模式

```bash
# 检查是否需要从 Legacy 迁移到 Unified
ccr migrate --check

# 迁移所有平台
ccr migrate

# 迁移指定平台
ccr migrate --platform claude
```

### 8. 临时凭据管理

```bash
# 设置临时覆盖凭据（不修改配置文件）
ccr temp-token set sk-ant-api03-xxxx \
  --base-url https://api.anthropic.com \
  --model claude-sonnet-4-5-20250929

# 查看当前临时凭据
ccr temp-token show

# 清除临时凭据
ccr temp-token clear
```

### 9. 技能与提示词管理

```bash
# 技能管理
ccr skills list                  # 列出技能
ccr skills scan ~/skills         # 扫描技能目录
ccr skills install ~/skills/<skill>  # 安装技能

# 提示词管理
ccr prompts list                 # 列出提示词
ccr prompts add                  # 添加提示词
ccr prompts apply <name>         # 应用提示词
```

### 10. 统计分析（需要 `web` 特性）

```bash
# 成本统计
ccr stats cost --today           # 今日成本
ccr stats cost --by-model        # 按模型统计
ccr stats cost --this-month      # 本月成本
```

### 11. 界面与服务

```bash
# 启动完整 UI（Vue 3 + Axum）
# 自动检测：workspace → ~/.ccr/ccr-ui → GitHub 下载
ccr ui -p 3000 --backend-port 38081

# 启动 TUI（需要 `tui` 特性）
ccr tui

# 启动轻量 Web API Server（兼容模式）
ccr web -p 8080
```

### 12. 系统命令

```bash
# 检查冲突
ccr check conflicts

# 自动更新
ccr update --check     # 检查更新
ccr update             # 更新到最新版本
ccr update dev         # 从 dev 分支更新

# 显示版本信息
ccr version
```

## TUI 终端界面使用指南

### 启动 TUI
```bash
ccr tui          # 基础模式
ccr tui --yes    # YOLO 模式（自动确认）
```

### 快捷键

#### 导航
- `Tab` / `Shift+Tab` - 切换标签页
- `1-4` - 快速切换到 1-4 标签页
- `↑↓` / `j/k` - 在列表中上下移动
- `Enter` - 选中/切换配置
- `PgUp` / `PgDn` - 翻页

#### 操作
- `d` - 删除选中的配置（需要 YOLO 模式）
- `Y` - 切换 YOLO 模式（自动确认危险操作）
- `Ctrl+C` / `q` - 退出 TUI

### 标签页说明

#### Tab 1: 配置列表
- 显示当前平台的所有配置
- 绿色高亮显示当前激活的配置
- 支持切换、删除操作

#### Tab 2: 平台管理
- 显示所有可用平台（Claude/Codex/Gemini/Qwen/iFlow）
- 高亮显示当前平台
- 支持快速切换平台

#### Tab 3: 同步状态
- 显示所有配置的同步状态
- 支持刷新状态查看

#### Tab 4: 系统信息
- 显示 CCR 版本、构建信息
- 显示系统资源使用情况
- 显示各路径位置

## Web API 接口文档

### 启动 Web 服务
```bash
ccr web                # 默认端口 8080
ccr web -p 8080        # 指定端口
```

### API 端点

#### 配置管理
```
GET    /api/configs           # 获取所有配置列表
POST   /api/switch            # 切换配置
       { "config_name": "anthropic" }

POST   /api/config            # 创建新配置
       { "name": "anthropic", "config": {...} }

POST   /api/config/{name}     # 更新配置
       { "description": "...", "base_url": "...", ... }

DELETE /api/config/{name}     # 删除配置
```

#### 设置管理
```
GET    /api/settings          # 获取当前 Claude settings.json
GET    /api/settings/backups  # 列出备份文件
POST   /api/settings/restore  # 恢复备份
       { "backup_path": "~/.claude/backups/xxx.json.bak" }
```

#### 历史记录
```
GET    /api/history           # 获取操作历史
       ?limit=50&type=switch
```

#### 操作接口
```
POST   /api/validate          # 验证配置完整性
POST   /api/export            # 导出配置
       { "output_path": "~/config.toml", "no_secrets": true }

POST   /api/import            # 导入配置
       { "input_path": "~/config.toml", "mode": "merge" }

POST   /api/clean             # 清理旧备份
       { "days": 30, "dry_run": true }
```

#### 系统信息
```
GET    /api/system            # 获取系统信息
       ?cached=false
```

#### 同步管理（需要 `web` 特性）
```
POST   /api/sync/folder/{name}/push    # 上传文件夹
POST   /api/sync/folder/{name}/pull    # 下载文件夹
GET    /api/sync/folder/{name}/status  # 查看状态
POST   /api/sync/all/push              # 批量上传
POST   /api/sync/all/pull              # 批量下载
GET    /api/sync/all/status            # 批量状态
```

### 响应格式

所有 API 返回 JSON 格式，包含 `success`、`data` 和 `message` 字段：

```json
{
  "success": true,
  "data": { ... },
  "message": "操作成功"
}
```

错误响应：
```json
{
  "success": false,
  "error": {
    "type": "ConfigNotFound",
    "message": "配置 'xxx' 未找到"
  }
}
```

### CORS 支持

Web API 默认支持 CORS，允许来自任何源的请求。适用于开发环境的前后端分离调试。

## CCR UI（Vue 3 + Axum + Tauri）

CCR UI 是全栈 Web 应用，提供完整的可视化界面管理所有 CCR 功能。

### 功能特性

- **可视化管理**：配置管理、验证、历史记录与备份
- **命令执行器**：覆盖全部 CLI 命令的可视化执行，实时输出
- **WebDAV 同步面板**：多文件夹管理（新增、启用、禁用、推送、拉取、状态、批量操作）
- **系统信息**：平台概览、健康检查、资源监控
- **多界面支持**：Web 模式（HTTP API）与桌面模式（Tauri）

### 通过 CLI 启动（推荐）

```bash
ccr ui                          # 自动检测：
                                # 1. workspace 中的 ccr-ui
                                # 2. ~/.ccr/ccr-ui 目录
                                # 3. GitHub 自动下载

# 自定义端口
ccr ui -p 3000 --backend-port 38081
```

**默认端口**：
- 前端：3000
- 后端 API：38081

### 从仓库开发

首次使用：
```bash
cd ccr-ui
just quick-start           # 依赖检查 + 安装 + 启动
```

开发模式：
```bash
cd ccr-ui
just s                     # 一键启动前后端开发模式
```

手动启动（显式命令）：
```bash
# 后端（workspace 成员，端口 38081）
cd ccr-ui/backend
cargo run -- --port 38081

# 前端（新开终端）
cd ccr-ui/frontend
bun install
bun run dev                # http://localhost:5173
```

### 生产构建

```bash
cd ccr-ui
just build                 # 构建后端 + 前端
just run-prod              # 启动编译后的后端并服务前端产物
```

### 桌面版（Tauri）

```bash
cd ccr-ui
just tauri-dev             # 桌面开发窗口
just tauri-build           # 打包桌面安装包
```

## 高级功能

### 配置模式详解

CCR 支持两种配置模式，满足不同需求：

#### 1. Legacy 模式（单平台）
适合只使用 Claude Code 的用户：
```
~/.ccs_config.toml         # 所有配置在一个文件
~/.claude/                  # Claude 配置目录
  -- settings.json
  -- ccr_history.json
  -- backups/
```

#### 2. Unified 模式（多平台，推荐）
适合使用多个 AI CLI 工具的用户：
```
~/.ccr/
├── config.toml            # 平台注册表
├── sync_folders.toml      # 同步配置
└── platforms/
    ├── claude/
    │   ├── profiles.toml
    │   ├── history/
    │   └── backups/
    ├── codex/
    │   ├── profiles.toml
    │   ├── history/
    │   └── backups/
    └── gemini/
        ├── profiles.toml
        ├── history/
        └── backups/
```

**切换模式**：
```bash
export CCR_LEGACY_MODE=1    # Legacy 模式
unset CCR_LEGACY_MODE       # Unified 模式
```

### 环境变量

#### 运行时配置
```bash
export CCR_LOG_LEVEL=debug  # 日志级别：trace/debug/info/warn/error
export CCR_LEGACY_MODE=1    # 启用传统模式
export CCR_ROOT=~/.custom    # 自定义配置根目录
```

#### Claude Code 设置（在 settings.json 中）
```json
{
  "env": {
    "ANTHROPIC_BASE_URL": "https://api.anthropic.com",
    "ANTHROPIC_AUTH_TOKEN": "sk-ant-api03-xxxx",
    "ANTHROPIC_MODEL": "claude-sonnet-4-5-20250929",
    "ANTHROPIC_SMALL_FAST_MODEL": "claude-3-5-haiku-20241022"
  }
}
```

### 并发安全机制

CCR 提供企业级的并发安全保护：

1. **文件锁机制**：基于 `fs4` 的咨询锁，防止多进程同时修改
2. **原子写入**：所有文件修改使用临时文件 + 原子重命名
3. **自动备份**：每次修改前自动创建备份，保留 7 天
4. **审计日志**：所有操作记录到历史文件，包含 UUID、时间戳、操作者

### 错误处理

CCR 提供详细的错误信息和退出码：

| 错误类型 | 退出码 | 说明 |
|---------|--------|------|
| 成功 | 0 | 操作成功 |
| 通用错误 | 1 | 未分类错误 |
| 配置未找到 | 2 | 配置文件或配置项不存在 |
| 验证失败 | 3 | 配置验证不通过 |
| 文件锁错误 | 4 | 无法获取文件锁（可能其他进程在使用） |
| IO 错误 | 5 | 文件读写错误 |
| 设置错误 | 6 | settings.json 格式错误或权限问题 |
| 操作取消 | 7 | 用户取消操作（Ctrl+C） |

### 敏感信息保护

CCR 自动掩码所有输出中的敏感信息：

**自动掩码**：
- API Keys: `sk-ant-api03-abcde...6789` → `sk-ant-********`
- Tokens: `ghp_abcdefghijklmnop` → `ghp_********`
- Passwords: 完全掩码

**配置导出**：
```bash
ccr export --no-secrets    # 去除所有敏感信息
```

## 开发指南

### 快速检查

```bash
# 代码检查
cargo check
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings

# 运行测试
cargo test --workspace

# 仅运行平台测试（需要串行执行）
cargo test --test platform_tests -- --test-threads=1
```

### just 命令

项目提供 `justfile` 简化开发流程：

```bash
just dev            # 开发模式编译
just watch          # 监听文件变化自动编译
just ci             # 完整 CI 流程（检查 + 测试）
just build          # 构建 release 版本
just release        # 发布构建（带优化）

cd ccr-ui
just s              # CCR UI 开发模式
just build          # 构建 UI
just tauri-dev      # Tauri 桌面开发
```

### 测试策略

CCR 拥有 95%+ 的测试覆盖率：

**已测试组件**：
- **单元测试**：各模块内嵌入 `#[cfg(test)]`
- **集成测试**：`tests/` 目录下的 6 个测试文件
- **并发测试**：`concurrent_tests.rs` 验证文件锁机制
- **端到端测试**：`end_to_end_tests.rs` 完整工作流

**运行测试**：
```bash
# 所有测试
cargo test

# 指定测试文件（平台测试需要串行）
cargo test --test platform_integration_tests -- --test-threads=1
cargo test --test concurrent_tests

# 带输出
cargo test -- --nocapture

# 覆盖率报告（需要 cargo-tarpaulin）
cargo tarpaulin --out Html
```

## 故障排查

### 开启日志调试

```bash
export CCR_LOG_LEVEL=debug    # 设置调试级别
ccr switch anthropic          # 执行命令查看详细日志

# 日志级别选项：
# - trace: 最详细，包括每个函数调用
# - debug: 调试信息，包括文件操作
# - info: 基本信息（默认）
# - warn: 警告信息
# - error: 仅错误信息

# 日志输出：
# - 终端：ANSI 彩色输出
# - 文件：~/.ccr/logs/ccr.YYYY-MM-DD.log（按天轮转，保留14天）

# 查看日志文件
tail -f ~/.ccr/logs/ccr.$(date +%Y-%m-%d).log
```

### 常见问题

#### 1. 锁超时错误
**症状**：`Lock acquisition timeout: config`

**原因**：另一个 CCR 进程正在使用配置文件

**解决**：
```bash
# 检查进程
ps aux | grep ccr

# 清理锁文件（谨慎操作）
rm -rf ~/.claude/.locks/*
```

#### 2. 权限错误
**症状**：`Permission denied` 读取或写入配置文件

**解决**：
```bash
# 确保文件归当前用户所有
sudo chown -R $USER:$USER ~/.claude/
sudo chown -R $USER:$USER ~/.ccr/

# 检查文件权限
ls -la ~/.claude/settings.json
ls -la ~/.ccr/config.toml
```

#### 3. 配置文件损坏
**症状**：`ValidationError: invalid TOML structure`

**解决**：
```bash
# CCR 会自动创建备份，找到最近备份
ls -lt ~/.claude/backups/*.bak | head -5

# 手动恢复
cp ~/.claude/backups/config_20250101_120000.toml.bak ~/.ccs_config.toml

# 或使用 CCR 恢复
ccr history -t backup  # 查看备份历史
ccr import ~/.claude/backups/xxx.toml --merge
```

#### 4. CCR UI 下载失败
**症状**：`ccr ui` 无法自动下载 UI

**原因**：网络问题或 GitHub API 限制

**解决**：
```bash
# 手动克隆
mkdir -p ~/.ccr
cd ~/.ccr
git clone https://github.com/bahayonghang/ccr.git
cd ccr
git checkout v3.9.4
mv ccr-ui ~/.ccr/

# 从 workspace 启动
cd /path/to/ccr/ccr-uiccr ui
```

#### 5. 同步冲突
**症状**：`CONFLICT: Remote file newer than local`

**解决**：
```bash
# 强制推送（覆盖远程）
ccr sync claude push --force

# 强制拉取（覆盖本地）
ccr sync claude pull --force

# 交互式选择
ccr sync claude push --interactive
```

#### 6. 传统配置迁移
**症状**：需要将所有配置迁移到 Unified 模式

**解决**：
```bash
# 检查可迁移内容
ccr migrate --check

# 自动迁移所有平台
ccr migrate

# 指定平台迁移
ccr migrate --platform claude
ccr migrate --platform codex

# 验证迁移结果
ccr platform list
ccr platform switch claude
ccr list
```

### WebDAV 同步配置示例

#### 坚果云
```toml
[webdav]
url = "https://dav.jianguoyun.com/dav/"
username = "your-email@example.com"
password = "your-app-password"  # 应用密码，非登录密码
base_remote_path = "/ccr"
```

#### Nextcloud
```toml
[webdav]
url = "https://cloud.example.com/remote.php/dav/files/username/"
username = "username"
password = "your-password"
base_remote_path = "/ccr"
```

#### ownCloud
```toml
[webdav]
url = "https://owncloud.example.com/remote.php/dav/files/username/"
username = "username"
password = "your-password"
base_remote_path = "/ccr"
```

## 性能优化

### 编译优化

CCR 使用多种编译优化策略：

**开发模式**：
```toml
[profile.dev]
opt-level = 1          # 基本优化，平衡编译速度和运行速度
incremental = true     # 启用增量编译
debug = 1              # 减少调试信息
```

**依赖包优化**：
```toml
[profile.dev.package."*"]
opt-level = 2          # 依赖包使用更高优化
```

**Release 模式**：
```toml
[profile.release]
opt-level = 3          # 最大优化
lto = true             # 启用链接时优化
codegen-units = 1      # 单个代码生成单元
```

### 运行性能

**并行处理**：
- 使用 `rayon` 进行并行文件操作
- WebDAV 批量同步使用并行上传/下载

**内存优化**：
- 小向量优化（`smallvec`）减少堆分配
- 惰性初始化（`once_cell`）减少启动时间

**异步 I/O**：
- Web 服务器基于 Tokio 异步运行时
- WebDAV 操作使用异步 HTTP 客户端

## 安全提示

### 保护敏感信息

1. **不要提交到版本控制**：
   ```bash
   # 在 .gitignore 中添加：
   *.toml
   *.json
   *.bak
   ```

2. **使用环境变量**：
   ```bash
   export ANTHROPIC_AUTH_TOKEN="sk-ant-xxx"
   ```

3. **定期更换 Token**：
   ```bash
   ccr temp-token set sk-ant-api03-new-token
ccr export -o backup.toml --no-secrets
   ```

4. **限制 Token 权限**：
   - 使用最小权限原则
   - 为不同环境创建不同 Token

### 同步安全

1. **使用 HTTPS**：始终使用 HTTPS 协议的 WebDAV 服务
2. **应用密码**：使用应用专用密码而非主密码
3. **访问控制**：限制 WebDAV 目录的访问权限
4. **定期审计**：检查同步日志，确认无异常访问

## 许可证

MIT License

## 贡献指南

欢迎提交 Issue 和 Pull Request！

### 开发环境搭建

```bash
# 1. Fork 仓库
git clone https://github.com/yourname/ccr.git
cd ccr

# 2. 安装依赖
cargo build --all-features

# 3. 安装 just（可选）
cargo install just

# 4. 运行测试
just ci
```

### PR 检查清单

- [ ] 代码遵循 Rust 2024 风格
- [ ] 通过 `cargo clippy` 无警告
- [ ] 通过 `cargo fmt` 格式化
- [ ] 添加单元测试或集成测试
- [ ] 更新相关文档（README、CHANGELOG）
- [ ] 在本地手动测试主要功能

### 报告问题

报告问题时请包含：

1. **环境信息**：
   ```bash
   ccr version
   rustc --version
   uname -a
   ```

2. **日志信息**：
   ```bash
   export CCR_LOG_LEVEL=debug
   ccr <command> 2>&1 | tee debug.log
   # 或查看日志文件：~/.ccr/logs/ccr.$(date +%Y-%m-%d).log
   ```

3. **配置文件示例**（去除敏感信息）：
   ```bash
   ccr export --no-secrets
   ```

4. **复现步骤**：详细操作步骤

## 更新日志

### v3.6.2 (2025-11-24)
- 添加技能和提示词管理命令（skills、prompts）
- 优化 TUI 交互体验
- 改进错误提示信息
- 修复 Codex 配置兼容性问题

### v3.6.1 (2025-11-20)
- 重构技能管理系统为异步架构
- 优化 UI 组件性能
- 移除 Codex 冗余功能

### v3.6.0 (2025-11-18)
- 完整的多平台支持（Claude/Codex/Gemini）
- 统一配置管理（~/.ccr/）
- 平台和配置的迁移工具
- 增强的成本统计功能

### v3.5.0 (2025-11-06)
- WebDAV 多文件夹同步 v2.5+
- 批量操作（push/pull/status）
- 文件夹启用/禁用控制
- 排除模式支持（类似 .gitignore）

### v3.4.1 (2025-10-25)
- 完整的多平台配置文档
- 改进的 TUI 界面
- 修复服务器启动问题

### v3.4.0 (2025-10-22)
- Vue 3 迁移完成
- 完整的前端重构
- 新增 Tauri 桌面支持

查看更多历史版本，请访问：[CHANGELOG.md](CHANGELOG.md)

## Star 历史

[![Star History Chart](https://api.star-history.com/svg?repos=bahayonghang/ccr&type=Date)](https://star-history.com/#bahayonghang/ccr&Date)

## 相关资源

- **GitHub 仓库**：https://github.com/bahayonghang/ccr
- **问题反馈**：https://github.com/bahayonghang/ccr/issues
- **PR 提交**：https://github.com/bahayonghang/ccr/pulls
- **Wiki 文档**：https://github.com/bahayonghang/ccr/wiki
- **讨论区**：https://github.com/bahayonghang/ccr/discussions

## 致谢

感谢以下项目和社区：

- [Claude](https://claude.ai) - 优秀的 AI 助手
- [Rust](https://rust-lang.org) - 高性能系统编程语言
- [Ratatui](https://github.com/ratatui-org/ratatui) - Rust TUI 框架
- [Vue.js](https://vuejs.org) - 渐进式 JavaScript 框架
- [Axum](https://github.com/tokio-rs/axum) - Rust Web 框架
- [Tauri](https://tauri.app) - 构建桌面应用

---

**CCR** - Claude Code Configuration Switcher

MIT © 2025 Yonghang Li
