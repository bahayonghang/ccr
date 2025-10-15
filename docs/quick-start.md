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

### 1. 初始化配置文件

首次使用 CCR 时,需要初始化配置文件：

```bash
ccr init
```

这将在 `~/.ccs_config.toml` 创建一个包含示例配置的文件。如果你已经有 CCS 的配置文件,可以跳过此步骤。

**示例配置文件：**

```toml
default_config = "anthropic"
current_config = "anthropic"

[anthropic]
description = "Anthropic Official API"
base_url = "https://api.anthropic.com"
auth_token = "sk-ant-your-api-key"
model = "claude-sonnet-4-5-20250929"
small_fast_model = "claude-3-5-haiku-20241022"

[anyrouter]
description = "AnyRouter Proxy Service"
base_url = "https://api.anyrouter.ai/v1"
auth_token = "your-anyrouter-token"
model = "claude-sonnet-4-5-20250929"
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

### 2. 查看可用配置

```bash
ccr list
# 或使用别名
ccr ls
```

**输出示例：**

```
Available Configurations
════════════════════════════════════════════════════════════════
Configuration File: /home/user/.ccs_config.toml
Default Config: anthropic
Current Config: anthropic
────────────────────────────────────────────────────────────────
▶ anthropic - Anthropic Official API
    Base URL: https://api.anthropic.com
    Token: sk-a...key
    Model: claude-sonnet-4-5-20250929
    Small Fast Model: claude-3-5-haiku-20241022
    Status: ✓ Configuration Complete
  anyrouter - AnyRouter Proxy Service
    Base URL: https://api.anyrouter.ai/v1
    Token: you...ken
    Model: claude-sonnet-4-5-20250929
    Status: ✓ Configuration Complete

✓ Found 2 configurations
```

### 3. 切换配置

切换到指定配置非常简单：

```bash
ccr switch anyrouter
# 或使用简写形式
ccr anyrouter
```

**执行流程：**

1. ✓ 读取并验证目标配置
2. ✓ 备份当前 Claude Code 设置
3. ✓ 更新 `~/.claude/settings.json`
4. ✓ 更新配置文件 `current_config`
5. ✓ 记录操作历史

::: warning 注意
切换配置会立即修改 Claude Code 的设置文件,配置立即生效。
:::

### 4. 查看当前配置状态

```bash
ccr current
# 或使用别名
ccr status
ccr show
```

这将显示当前配置的详细信息,包括环境变量设置。

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

### 7. 启动 Web 界面

CCR 提供了友好的 Web 配置界面：

```bash
# 使用默认端口 8080
ccr web

# 指定端口
ccr web --port 3000
```

浏览器会自动打开配置界面,你可以：
- 查看所有配置
- 切换配置
- 添加/编辑/删除配置
- 查看操作历史
- 验证配置
- 清理备份

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
