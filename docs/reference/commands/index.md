# 核心命令

CCR 提供了丰富的命令来管理 Claude Code 配置。本页面概览所有可用命令。

## 命令概览

| 命令 | 别名 | 说明 | 版本 |
|------|------|------|------|
| [init](./init) | - | 初始化配置（默认 Unified，可兼容 Legacy） | v1.0+ |
| [platform](./platform) | - | 平台注册表管理（list/switch/current/info/init） | v3.6+ |
| [migrate](./migrate) | - | Legacy → Unified 迁移 | v3.6+ |
| [list](./list) | `ls` | 列出当前平台的 profiles | v1.0+ |
| [current](./current) | `status`, `show` | 当前 profile 状态 | v1.0+ |
| [switch](./switch) | - | 切换 profile（支持快捷 `ccr <name>`） | v1.0+ |
| [add](./add) | - | 交互式添加新配置 | v1.0+ |
| [delete](./delete) | - | 删除指定配置 | v1.0+ |
| enable | - | 启用 profile（当前平台） | v1.0+ |
| disable | - | 禁用 profile（支持 --force） | v1.0+ |
| [validate](./validate) | `check` | 验证配置和 settings 文件 | v1.0+ |
| optimize | - | 重新排序配置 | v1.0+ |
| clear | - | 清除 settings.json 中 CCR 写入 | v2.0+ |
| [temp-token](./temp-token) | - | 临时覆盖 token/base_url/model | v2.0+ |
| [history](./history) | - | 操作历史 | v1.0+ |
| [stats](./stats) | - | 成本/调用统计（web 特性） | v2.0+ |
| [export](./export) | - | 导出配置到文件 | v1.0+ |
| [import](./import) | - | 从文件导入配置 | v1.0+ |
| [clean](./clean) | - | 清理旧备份文件 | v2.0+ |
| [sync](./sync) | - | WebDAV 同步（目录注册/批量/交互式） | v2.0+ |
| [ui](./ui) | - | 启动完整 CCR UI | v1.4+ |
| [tui](./tui) | - | 终端界面 | v2.0+ |
| [web](./web) | - | 轻量 Web API（兼容/脚本） | v2.0+ |
| [skills](./skills) | - | 技能管理 | v3.5+ |
| [prompts](./prompts) | - | 提示词模板管理 | v3.5+ |
| [sessions](./sessions) | - | Session 会话管理（索引/搜索/恢复） | v3.12+ |
| [provider](./provider) | - | Provider 健康检查（连通性/Key验证） | v3.12+ |
| [check](./check) | - | 配置冲突检测 | v3.6+ |
| [update](./update) | - | 更新到最新版本 | v1.0+ |
| [version](./version) | `ver` | 显示版本信息 | v1.0+ |

## 命令分类

### 初始化与平台

- **[init](./init)** - 初始化配置（默认 Unified）
- **[platform](./platform)** - 平台注册表管理
- **[migrate](./migrate)** - Legacy → Unified 迁移

### Profile 管理

- **[list](./list)** / **[current](./current)** / **[switch](./switch)** - 查看与切换
- **[add](./add)** / **[delete](./delete)** - 增删 profile
- **enable/disable** - 启用/禁用 profile
- **[validate](./validate)** / optimize / clear - 校验、排序、清除写入
- **[temp-token](./temp-token)** - 临时覆盖 token/base_url/model

### 数据与同步

- **[export](./export)** / **[import](./import)** / **[clean](./clean)** - 导出、导入、清理备份
- **[sync](./sync)** - WebDAV 同步（注册目录、批量/单目录、交互式过滤）
- **[history](./history)** / **[stats](./stats)** - 审计历史与统计

### 界面

- **[ui](./ui)** - 启动完整 CCR UI
- **[tui](./tui)** - 终端界面（tui 特性）
- **[web](./web)** - 轻量 Web API（兼容/脚本）

### 扩展与维护

- **[skills](./skills)** / **[prompts](./prompts)** - 扩展管理
- **[sessions](./sessions)** - Session 会话管理（索引/搜索/恢复）
- **[provider](./provider)** - Provider 健康检查
- **[check](./check)** - 配置冲突检测
- **[update](./update)** / **[version](./version)** - 更新与版本信息

## 常用命令速查

### 快速开始

```bash
# 初始化配置
ccr init

# 查看所有配置
ccr list

# 切换配置(两种方式)
ccr switch anthropic
ccr anthropic

# 添加新配置
ccr add

# 删除配置
ccr delete old_config
```

### 日常使用

```bash
# 查看当前配置
ccr current

# 验证配置
ccr validate

# 查看历史
ccr history --limit 10

# 查看成本统计
ccr stats cost --range today
ccr stats cost --range month --details

# 临时使用免费 token（不修改配置文件）
ccr temp-token set sk-free-xxx --expires 24

# 查看临时配置
ccr temp-token show

# 清除临时配置
ccr temp-token clear
```

### 数据管理

```bash
# 导出配置
ccr export -o backup.toml

# 导入配置
ccr import config.toml --merge

# 清理备份
ccr clean --days 30
```

### 高级功能

```bash
# 启动交互式 TUI
ccr tui

# 启动 TUI（启用自动确认模式）
ccr tui --yes      # 或 ccr tui -y

# 启动 CCR UI（推荐 Web 界面）
ccr ui

# 启动 Legacy Web 界面 / Web API（适合脚本/CI 等场景）
ccr web --port 8080

# 更新 CCR
ccr update

# 查看版本
ccr version
```

### Sessions 管理

```bash
# 重建会话索引
ccr sessions reindex

# 列出最近会话
ccr sessions list

# 按平台过滤
ccr sessions list --platform claude --today

# 搜索会话
ccr sessions search "refactoring"

# 查看详情并恢复
ccr sessions show <id>
ccr sessions resume <id>

# 查看统计
ccr sessions stats
```

### Provider 健康检查

```bash
# 测试单个配置
ccr provider test my-provider

# 测试所有配置
ccr provider test --all

# 验证 API Key
ccr provider verify my-provider
```

## 环境变量

CCR 支持以下环境变量：

### CCR_LOG_LEVEL

设置日志级别,用于调试。

**可选值：**
- `trace` - 最详细的日志
- `debug` - 调试信息
- `info` - 一般信息(默认)
- `warn` - 警告信息
- `error` - 仅错误信息

**日志输出：**
- **终端**：带 ANSI 彩色
- **文件**：`~/.ccr/logs/ccr.YYYY-MM-DD.log`（按天轮转，自动清理超过14天的旧日志）

**示例：**

```bash
export CCR_LOG_LEVEL=debug
ccr switch anthropic

# 查看日志文件
tail -f ~/.ccr/logs/ccr.$(date +%Y-%m-%d).log
```

## 使用技巧

### 1. 命令别名

许多命令都有简短的别名,可以加快输入速度：

```bash
ccr ls              # 等同于 ccr list
ccr status          # 等同于 ccr current
ccr check           # 等同于 ccr validate
ccr ver             # 等同于 ccr version
```

### 2. 快速切换

直接使用配置名称作为参数：

```bash
ccr anthropic       # 等同于 ccr switch anthropic
```

### 3. 命令组合

使用 `&&` 组合多个命令：

```bash
# 切换后立即查看历史
ccr switch anthropic && ccr history --limit 1

# 导入后验证
ccr import config.toml --merge && ccr validate
```

### 4. 定期维护

设置定期任务：

```bash
# 每周清理旧备份(添加到 crontab)
0 0 * * 0 ccr clean --days 30

# 每天导出配置备份
0 0 * * * ccr export -o ~/backups/ccr-$(date +\%Y\%m\%d).toml
```

## 下一步

- 查看 [快速开始](/quick-start) 了解基本使用流程
- 查看 [配置管理](/configuration) 了解高级配置选项
- 点击上方表格中的命令查看详细文档
