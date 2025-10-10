# CCR - Claude Code Configuration Switcher

🚀 **Claude Code 配置管理工具 (Rust 实现版)**

CCR 是 Claude Code Configuration Switcher (CCS) 的 Rust 实现版本，提供更强大的配置管理功能，包括完整的审计追踪、文件锁机制和自动备份恢复功能。

## ✨ 核心特性

### 🎯 直接写入 Claude Code 设置
- 直接操作 `~/.claude/settings.json` 文件
- 无需手动配置环境变量
- 配置立即生效

### 🔐 并发安全
- 文件锁机制确保多进程安全
- 原子写入操作防止数据损坏
- 超时保护避免死锁

### 📝 完整审计追踪
- 记录所有操作历史
- 环境变量变更追踪
- 敏感信息自动掩码

### 💾 自动备份恢复
- 切换前自动备份
- 支持从备份恢复
- 带时间戳的备份文件

### ✅ 配置验证
- 自动验证配置完整性
- 检查必填字段
- URL 格式验证

### 🌐 Web 界面
- 浏览器配置管理界面
- RESTful API 支持
- 实时配置切换

### 🔄 与 CCS 完全兼容
- 共享 `~/.ccs_config.toml` 配置文件
- 命令行接口保持一致
- 可与 CCS 共存使用

## 📦 安装

### 快速安装（推荐）

使用 cargo 直接从 GitHub 安装 CCR：

```bash
cargo install --git https://github.com/bahayonghang/ccr
```

安装完成后，`ccr` 命令将自动添加到系统 PATH 中。

### 从源码构建

```bash
# 克隆仓库
cd ccs/ccr

# 构建发布版本
cargo build --release

# 安装到系统路径（可选）
cargo install --path .
```

### 运行程序

```bash
# 直接运行
cargo run -- <command>

# 或使用编译后的二进制
./target/release/ccr <command>
```

## 🚀 快速开始

### 1. 初始化配置文件

使用示例模板初始化 CCR 配置文件：

```bash
ccr init
```

这将创建 `~/.ccs_config.toml` 并包含示例配置。如果已经安装了 CCS，也可以直接使用现有配置。

示例配置文件：

```toml
default_config = "anthropic"
current_config = "anthropic"

[anthropic]
description = "Anthropic 官方 API"
base_url = "https://api.anthropic.com"
auth_token = "sk-ant-your-api-key"
model = "claude-sonnet-4-5-20250929"
small_fast_model = "claude-3-5-haiku-20241022"

[anyrouter]
description = "AnyRouter 代理服务"
base_url = "https://api.anyrouter.ai/v1"
auth_token = "your-anyrouter-token"
model = "claude-sonnet-4-5-20250929"
```

### 2. 查看可用配置

```bash
ccr list
# 或
ccr ls
```

### 3. 切换配置

```bash
ccr switch anthropic
# 或简写
ccr anthropic
```

### 4. 查看当前状态

```bash
ccr current
# 或
ccr status
```

### 5. 验证配置

```bash
ccr validate
# 或
ccr check
```

### 6. 查看历史

```bash
ccr history
# 限制显示数量
ccr history --limit 10
# 按类型筛选
ccr history -t switch
```

### 7. 启动 Web 界面

```bash
ccr web
# 指定端口
ccr web --port 8080
```

### 8. 更新到最新版本

```bash
# 仅检查更新
ccr update --check

# 更新到最新版本
ccr update
```

### 9. 配置导入导出

```bash
# 导出配置（默认包含 API 密钥）
ccr export

# 导出配置（不含密钥）
ccr export --no-secrets

# 导出到指定文件
ccr export -o my-config.toml

# 导入配置（合并模式）
ccr import config.toml --merge

# 导入配置（替换模式）
ccr import config.toml
```

### 10. 清理旧备份

```bash
# 清理 7 天前的备份（默认）
ccr clean

# 清理 30 天前的备份
ccr clean --days 30

# 模拟运行（预览不删除）
ccr clean --dry-run
```

## 📚 命令详解

### init
从模板初始化配置文件

```bash
ccr init
```

功能：
- 从内置模板创建 `~/.ccs_config.toml`
- **安全模式**：如果配置已存在，拒绝覆盖（除非使用 --force）
- 使用 --force 时自动备份现有配置
- 设置正确的文件权限（Unix: 644）
- 提供有用的后续操作提示

行为：
- 如果配置存在：显示警告并退出（安全）
- 使用 `--force`：备份并覆盖现有配置

选项：
```bash
ccr init --force    # 强制覆盖，会自动备份
```

### list / ls
列出所有可用配置，标注当前配置和验证状态

```bash
ccr list
```

输出示例：
```
可用配置列表
════════════════════════════════════════════════════════════════
配置文件: /home/user/.ccs_config.toml
默认配置: anthropic
当前配置: anthropic
────────────────────────────────────────────────────────────────
▶ anthropic - Anthropic 官方 API
    Base URL: https://api.anthropic.com
    Token: sk-a...key
    Model: claude-sonnet-4-5-20250929
    Small Fast Model: claude-3-5-haiku-20241022
    状态: ✓ 配置完整
  anyrouter - AnyRouter 代理服务

✓ 共找到 2 个配置
```

### current / show / status
显示当前配置的详细状态，包括环境变量

```bash
ccr current
```

### switch <config>
切换到指定配置

```bash
ccr switch anyrouter
```

执行流程：
1. ✓ 读取并验证目标配置
2. ✓ 备份当前 Claude Code 设置
3. ✓ 更新 `~/.claude/settings.json`
4. ✓ 更新配置文件 `current_config`
5. ✓ 记录操作历史

### validate / check
验证配置和设置的完整性

```bash
ccr validate
```

检查内容：
- 配置文件格式
- 所有配置节的完整性
- Claude Code 设置文件
- 必需的环境变量

### history
显示操作历史记录

```bash
# 默认显示最近 20 条
ccr history

# 自定义数量
ccr history --limit 50

# 按类型筛选
ccr history -t switch   # 只显示切换操作
ccr history -t backup   # 只显示备份操作
```

### web
启动 Web 配置界面

```bash
# 默认端口 8080
ccr web

# 指定端口
ccr web --port 3000
```

### update
检查并安装 CCR 最新版本

```bash
# 仅检查更新
ccr update --check

# 更新到最新版本
ccr update
```

特性：
- 自动从 GitHub releases 下载
- 自动选择适配当前平台的二进制文件
- 原子更新，带备份保护
- 安装后自动验证

### export
导出配置到文件

```bash
# 导出完整配置（默认包含 API 密钥）
ccr export

# 导出配置（隐藏密钥）
ccr export --no-secrets

# 导出到指定文件
ccr export -o backup.toml
```

特性：
- 自动生成带时间戳的文件名
- 默认包含 API 密钥，方便迁移
- 可选的密钥掩码保护（--no-secrets）
- TOML 格式易于编辑
- 完美适用于备份和迁移

### import
从文件导入配置

```bash
# 合并模式（保留现有配置，添加新的）
ccr import config.toml --merge

# 替换模式（完全替换当前配置）
ccr import config.toml

# 导入时不备份
ccr import config.toml --no-backup
```

特性：
- 合并或替换模式
- 导入前自动备份
- 配置验证
- 详细的导入摘要

### clean
清理旧备份文件，释放磁盘空间

```bash
# 清理 7 天前的备份（默认）
ccr clean

# 清理 30 天前的备份
ccr clean --days 30

# 模拟运行（预览不删除）
ccr clean --dry-run
```

特性：
- 自动清理旧备份文件
- 可配置保留天数（默认 7 天）
- 模拟运行模式预览
- 显示释放的磁盘空间
- 仅删除 `~/.claude/backups/` 中的 `.bak` 文件

选项：
```bash
ccr clean --days 14      # 清理 14 天前的备份
ccr clean --dry-run      # 预览清理，不实际删除
```

### version / ver
显示版本信息和帮助

```bash
ccr version
```

## 📁 文件结构

CCR 使用以下文件和目录：

```
~/.ccs_config.toml          # 配置文件（与 CCS 共享）
~/.claude/settings.json     # Claude Code 设置文件
~/.claude/backups/          # 自动备份目录
~/.claude/ccr_history.json  # 操作历史记录
~/.claude/.locks/           # 文件锁目录
```

## 🔧 高级功能

### 环境变量管理

CCR 管理以下环境变量：

- `ANTHROPIC_BASE_URL` - API 端点地址
- `ANTHROPIC_AUTH_TOKEN` - 认证令牌
- `ANTHROPIC_MODEL` - 默认模型
- `ANTHROPIC_SMALL_FAST_MODEL` - 快速小模型（可选）

切换配置时，CCR 会：
1. 清空所有 `ANTHROPIC_*` 前缀的环境变量
2. 根据目标配置设置新的环境变量
3. 保留其他设置不变

### 备份与恢复

自动备份：
- 每次切换配置前自动备份
- 备份文件包含时间戳和配置名称
- 存储在 `~/.claude/backups/` 目录

手动恢复功能（未来版本）：
```bash
# 计划支持
ccr restore <backup-file>
```

### 历史记录

历史记录包含：
- 操作ID (UUID)
- 时间戳
- 操作者（系统用户名）
- 操作类型
- 环境变量变更（已掩码）
- 操作结果
- 备注信息

### Web API

CCR 提供 RESTful API 供程序化访问：

```bash
# 列出配置
GET /api/configs

# 切换配置
POST /api/switch
Body: {"config_name": "anthropic"}

# 获取历史
GET /api/history

# 验证配置
POST /api/validate

# 清理备份
POST /api/clean
Body: {"days": 7, "dry_run": false}

# 添加/更新/删除配置
POST /api/config
PUT /api/config/{name}
DELETE /api/config/{name}
```

### 日志调试

设置日志级别：

```bash
# 设置环境变量
export CCR_LOG_LEVEL=debug  # trace, debug, info, warn, error

# 运行命令
ccr switch anthropic
```

## 🔒 安全特性

### 敏感信息保护
- API Token 自动掩码显示
- 历史记录中的敏感值脱敏
- 仅显示 Token 首尾字符

### 文件权限
- 设置文件权限为 600（仅所有者读写）
- 锁文件自动清理
- 原子操作避免竞态条件

### 并发控制
- 跨进程文件锁
- 超时保护（默认 10 秒）
- 自动释放锁资源

## 🆚 CCR vs CCS

| 特性 | CCS (Shell) | CCR (Rust) |
|------|------------|-----------|
| 配置切换 | ✅ | ✅ |
| 环境变量设置 | ✅ | ✅ |
| 直接写入 settings.json | ❌ | ✅ |
| 文件锁机制 | ❌ | ✅ |
| 操作历史 | ❌ | ✅ |
| 自动备份 | ❌ | ✅ |
| 配置验证 | 基础 | 完整 |
| 并发安全 | ❌ | ✅ |
| Web 界面 | ❌ | ✅ |
| 性能 | 快 | 极快 |

## 🤝 与 CCS 的兼容性

CCR 完全兼容 CCS：

1. **共享配置文件** - 使用相同的 `~/.ccs_config.toml`
2. **无缝切换** - 可以交替使用 CCS 和 CCR
3. **命令一致** - 核心命令保持一致
4. **共存使用** - 可以同时安装两者

## 📝 开发说明

### 项目结构

```
ccr/
├── src/
│   ├── main.rs          # 主程序入口
│   ├── error.rs         # 错误处理
│   ├── logging.rs       # 日志与彩色输出
│   ├── lock.rs          # 文件锁
│   ├── config.rs        # 配置管理
│   ├── settings.rs      # 设置管理
│   ├── history.rs       # 历史记录
│   ├── web.rs           # Web 服务器
│   └── commands/        # CLI 命令
│       ├── mod.rs
│       ├── list.rs
│       ├── current.rs
│       ├── switch.rs
│       ├── validate.rs
│       └── history_cmd.rs
├── web/                 # Web 界面 HTML
├── Cargo.toml           # 项目配置
└── README.md            # 本文件
```

### 运行测试

```bash
cargo test
```

### 代码检查

```bash
cargo check
cargo clippy
```

### 格式化

```bash
cargo fmt
```

## 🐛 故障排除

### 配置文件不存在

```bash
# 检查配置文件
ls -la ~/.ccs_config.toml

# 如果不存在，先安装 CCS 或手动创建
```

### Claude Code 设置文件不存在

```bash
# 检查 Claude Code 目录
ls -la ~/.claude/

# 首次使用时会自动创建
ccr switch <config>
```

### 文件锁超时

```bash
# 检查是否有僵死进程
ps aux | grep ccr

# 清理锁文件（谨慎操作）
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

## 🛣️ 未来计划

- [x] Web 界面支持
- [x] RESTful API
- [x] 在线更新功能
- [x] 配置导入/导出
- [x] 配置初始化
- [ ] 配置模板系统
- [ ] 更多统计和报表
- [ ] 跨平台安装包

## 📄 许可证

MIT License

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📮 联系方式

- GitHub: https://github.com/bahayonghang/ccr
- 项目主页: https://github.com/bahayonghang/ccs/tree/main/ccr

---

**注意**: CCR 目前处于活跃开发阶段，建议在生产环境使用前进行充分测试。
