# 命令参考

CCR 提供了一组简洁而强大的命令行工具，用于管理 Claude Code 的 API 配置。

## 📖 命令概览

| 命令 | 别名 | 描述 |
|------|------|------|
| `list` | `ls` | 列出所有可用配置 |
| `current` | `show`, `status` | 显示当前配置状态 |
| `switch &lt;config&gt;` | - | 切换到指定配置 |
| `validate` | `check` | 验证配置完整性 |
| `history` | - | 查看操作历史记录 |
| `web` | - | 启动 Web 管理界面 |
| `version` | `ver` | 显示版本信息 |

## 🎯 快速开始

### 基本用法

```bash
# 显示帮助信息
ccr --help

# 显示版本
ccr --version

# 列出配置
ccr list

# 查看当前状态
ccr current

# 切换配置
ccr switch anyrouter

# 验证配置
ccr validate

# 查看历史
ccr history

# 启动 Web 界面
ccr web
```

### 简写形式

```bash
# 直接切换配置（无需 switch 关键字）
ccr anyrouter

# 命令别名
ccr ls           # list
ccr show         # current
ccr status       # current
ccr check        # validate
ccr ver          # version
```

## 📚 详细命令说明

### list / ls

列出所有可用的配置，显示详细信息和验证状态。

**用法**:
```bash
ccr list
ccr ls
```

**输出示例**:
```
可用配置列表
════════════════════════════════════════════════════════════════
配置文件: /home/user/.ccs_config.toml
默认配置: anthropic
当前配置: anthropic
────────────────────────────────────────────────────────────────
▶ anthropic - Anthropic 官方 API
    Base URL: https://api.anthropic.com
    Token: sk-a...here
    Model: claude-sonnet-4-5-20250929
    Small Fast Model: claude-3-5-haiku-20241022
    状态: ✓ 配置完整
  anyrouter - AnyRouter 代理服务
  openrouter - OpenRouter 服务

✓ 共找到 3 个配置
```

**特性**:
- 显示当前活跃配置（用 ▶ 标记）
- 自动验证每个配置
- 脱敏显示 API Token
- 彩色输出增强可读性

**退出码**:
- `0` - 成功
- `11` - 配置文件不存在

---

### current / show / status

显示当前配置的详细状态，包括环境变量设置。

**用法**:
```bash
ccr current
ccr show
ccr status
```

**输出示例**:
```
当前配置状态
═══════════════════════════════════════

  配置文件: /home/user/.ccs_config.toml
  当前配置: anthropic
  默认配置: anthropic

▶ 配置详情:
  描述: Anthropic 官方 API
  Base URL: https://api.anthropic.com
  Auth Token: sk-a...here
  Model: claude-sonnet-4-5-20250929
  Small Fast Model: claude-3-5-haiku-20241022

✓ 配置验证通过

──────────────────────────────────────────────────────────────

▶ Claude Code 环境变量状态:

  ANTHROPIC_BASE_URL: https://api.anthropic.com
  ANTHROPIC_AUTH_TOKEN: sk-a...here
  ANTHROPIC_MODEL: claude-sonnet-4-5-20250929
  ANTHROPIC_SMALL_FAST_MODEL: claude-3-5-haiku-20241022

✓ Claude Code 设置验证通过
```

**特性**:
- 显示配置文件信息
- 显示当前配置详情
- 显示 Claude Code 环境变量
- 配置验证结果

**退出码**:
- `0` - 成功
- `11` - 配置文件不存在
- `21` - 设置文件不存在

---

### switch &lt;config&gt;

切换到指定的配置，包含完整的备份、验证和历史记录功能。

**用法**:
```bash
ccr switch <config_name>

# 简写形式
ccr <config_name>
```

**示例**:
```bash
# 完整命令
ccr switch anyrouter

# 简写
ccr anyrouter
```

**执行流程**:

```
步骤 1/5: 读取配置文件
✓ 目标配置 'anyrouter' 验证通过

步骤 2/5: 备份当前设置
✓ 设置已备份: /home/user/.claude/backups/settings.anyrouter.20250110_143022.json.bak

步骤 3/5: 更新 Claude Code 设置
✓ Claude Code 设置已更新

步骤 4/5: 更新配置文件
✓ 当前配置已设置为: anyrouter

步骤 5/5: 记录操作历史
✓ 操作历史已记录

──────────────────────────────────────────────────────────────

配置切换成功

  配置名称: anyrouter
  描述: AnyRouter 代理服务
  Base URL: https://api.anyrouter.ai/v1
  Auth Token: your...here
  Model: claude-sonnet-4-5-20250929

✓ 配置已生效，Claude Code 可以使用新的 API 配置

ℹ 提示: 重启 Claude Code 以确保配置完全生效
```

**特性**:
- ✅ 配置验证
- ✅ 自动备份
- ✅ 原子操作
- ✅ 文件锁保护
- ✅ 历史记录
- ✅ 详细进度显示

**退出码**:
- `0` - 成功
- `11` - 配置文件不存在
- `12` - 配置节不存在
- `20` - 设置文件错误
- `30` - 文件锁错误
- `90` - 配置验证失败

---

### validate / check

验证配置文件和 Claude Code 设置的完整性。

**用法**:
```bash
ccr validate
ccr check
```

**输出示例**:
```
配置验证报告

▶ 验证配置文件 (~/.ccs_config.toml)
✓ 配置文件存在: /home/user/.ccs_config.toml

  ✓ anthropic
  ✓ anyrouter
  ✗ openrouter - base_url 不能为空

✓ 所有 2 个配置节验证通过

▶ 当前配置验证
✓ 当前配置 'anthropic' 存在

──────────────────────────────────────────────────────────────

▶ 验证 Claude Code 设置 (~/.claude/settings.json)
✓ 设置文件存在: /home/user/.claude/settings.json

▶ 环境变量验证
  ✓ ANTHROPIC_BASE_URL: https://api.anthropic.com
  ✓ ANTHROPIC_AUTH_TOKEN: sk-a...here
  ✓ ANTHROPIC_MODEL: claude-sonnet-4-5-20250929
  ○ ANTHROPIC_SMALL_FAST_MODEL: 未设置（可选）

✓ 设置验证通过

──────────────────────────────────────────────────────────────

验证总结

✓ 所有验证通过，配置状态正常
```

**验证项目**:
- ✅ 配置文件存在性
- ✅ 配置文件格式
- ✅ 所有配置节的完整性
- ✅ 当前配置存在性
- ✅ Claude Code 设置文件
- ✅ 必需环境变量
- ✅ URL 格式验证

**退出码**:
- `0` - 验证通过
- `11` - 配置文件不存在
- `90` - 验证失败

---

### history

查看操作历史记录，支持筛选和限制显示数量。

**用法**:
```bash
ccr history [选项]
```

**选项**:
- `-l, --limit &lt;NUM&gt;` - 限制显示的记录数量（默认 20）
- `-t, --filter-type &lt;TYPE&gt;` - 按操作类型筛选

**类型**:
- `switch` - 切换配置
- `backup` - 备份操作
- `restore` - 恢复操作
- `validate` - 验证操作
- `update` - 更新操作

**示例**:
```bash
# 显示最近 20 条
ccr history

# 显示最近 50 条
ccr history --limit 50
ccr history -l 50

# 只显示切换操作
ccr history --filter-type switch
ccr history -t switch
```

**输出示例**:
```
操作历史记录

ℹ 总操作数: 42
ℹ 成功: 40, 失败: 1, 警告: 1

──────────────────────────────────────────────────────────────

1. [2025-01-10 14:30:22] 切换配置 - 成功
   操作者: user
   从: anthropic
   到: anyrouter
   环境变量变化:
     ANTHROPIC_BASE_URL api.anthropic.com -> api.anyrouter.ai/v1
     ANTHROPIC_AUTH_TOKEN sk-a...here -> your...here

2. [2025-01-10 13:15:10] 验证 - 成功
   操作者: user

3. [2025-01-10 12:00:05] 切换配置 - 成功
   操作者: user
   从: anyrouter
   到: anthropic

ℹ 显示了最近 20 条记录
```

**特性**:
- 完整的操作记录
- 敏感信息自动掩码
- 环境变量变更追踪
- 统计信息
- 灵活的筛选和限制

**退出码**:
- `0` - 成功
- `80` - 历史文件错误

---

### web

启动 Web 管理界面，提供可视化配置管理。

**用法**:
```bash
ccr web [选项]
```

**选项**:
- `-p, --port &lt;PORT&gt;` - 指定端口（默认 8080）

**示例**:
```bash
# 使用默认端口
ccr web

# 指定端口
ccr web --port 3000
ccr web -p 3000
```

**输出**:
```
✓ CCR Web 服务器已启动
ℹ 地址: http://localhost:8080
ℹ 按 Ctrl+C 停止服务器
```

**Web 界面功能**:
- 📋 列出所有配置
- ➕ 添加新配置
- ✏️ 编辑现有配置
- 🗑️ 删除配置
- 🔄 切换配置
- ✅ 验证配置
- 📜 查看历史记录

**API 端点**:
- `GET /` - Web 界面
- `GET /api/configs` - 获取配置列表
- `POST /api/switch` - 切换配置
- `POST /api/config` - 添加配置
- `PUT /api/config/{name}` - 更新配置
- `DELETE /api/config/{name}` - 删除配置
- `GET /api/history` - 获取历史记录
- `POST /api/validate` - 验证配置

**退出码**:
- `0` - 成功启动
- `10` - HTTP 服务器错误

---

### version / ver

显示 CCR 版本信息和特性说明。

**用法**:
```bash
ccr version
ccr ver
ccr --version
ccr -V
```

**输出示例**:
```
╔══════════════════════════════════════════════════════════════╗
║                                                              ║
║   ██████╗  ██████╗██████╗                                   ║
║  ██╔════╝ ██╔════╝██╔══██╗                                  ║
║  ██║      ██║     ██████╔╝                                  ║
║  ██║      ██║     ██╔══██╗                                  ║
║  ╚██████╗ ╚██████╗██║  ██║                                  ║
║   ╚═════╝  ╚═════╝╚═╝  ╚═╝                                  ║
║                                                              ║
║  Claude Code Configuration Switcher - Configuration Management Tool         ║
║  Version: 0.2.0                                              ║
║                                                              ║
╚══════════════════════════════════════════════════════════════╝

  版本: 0.2.0
  作者: Yonghang Li
  描述: Claude Code Configuration Switcher - Configuration management tool

ℹ CCR 特性:
  • 直接写入 Claude Code 设置文件 (~/.claude/settings.json)
  • 文件锁机制确保并发安全
  • 完整的操作历史和审计追踪
  • 配置备份和恢复功能
  • 自动配置验证
  • 与 CCS 完全兼容

ℹ 常用命令:
  ccr list              列出所有配置
  ccr current           显示当前状态
  ccr switch &lt;name&gt;     切换配置
  ccr validate          验证配置
  ccr history           查看历史

ℹ 更多帮助: ccr --help
```

**退出码**:
- `0` - 成功

## 🔗 相关文档

- [命令详细参考](/commands/list)
- [API 参考](/api/)
- [故障排除](/installation/troubleshooting)

